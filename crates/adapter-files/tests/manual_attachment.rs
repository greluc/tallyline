// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Real-directory manual privacy preview and upload contracts.

use tallyline_adapter_files::api::model::{
    AttachmentEncodingSettings, ImageIntake, ImageIntakeSettings,
};
use tallyline_adapter_files::api::{encode_manual_attachment, prepare_manual_attachment_file};
use tallyline_game::api::model::GameEnvironment;
use tallyline_reporting::api::AttachmentEdit;
use tallyline_reporting::api::model::{AttachmentRegion, Report, ReportId, ReportKey};
use tallyline_shared_kernel::api::model::{TerminalId, Timestamp, TradeSide};

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_man_5_r_sub_7_r_cap_8__only_the_exact_confirmed_region_can_be_encoded() {
    let directory =
        std::env::temp_dir().join(format!("tallyline-private-region-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("capture.not-an-image-extension");
    let mut source = image::RgbImage::from_pixel(64, 48, image::Rgb([0, 200, 0]));
    for y in 0..12 {
        for x in 0..64 {
            source.put_pixel(x, y, image::Rgb([255, 0, 255]));
        }
    }
    source
        .save_with_format(&path, image::ImageFormat::Png)
        .unwrap();
    let region = AttachmentRegion::new(8, 16, 32, 24).unwrap();
    let preview =
        prepare_manual_attachment_file(&path, region, &ImageIntakeSettings::default()).unwrap();
    assert_eq!(
        (preview.raster().width(), preview.raster().height()),
        (32, 24)
    );
    assert!(preview.raster().argb().iter().all(|p| *p == 0xff00_c800));
    let mut report = Report::manual(
        ReportId(1),
        ReportKey::new(TerminalId(11), TradeSide::Buy, GameEnvironment::Live).unwrap(),
        Timestamp::from_seconds(u64::MAX),
        Some("4.10.2"),
    )
    .unwrap();
    report
        .edit_attachment(AttachmentEdit::Select(preview.binding(), preview.time()))
        .unwrap();
    assert!(
        encode_manual_attachment(&report, &preview, &AttachmentEncodingSettings::default())
            .is_err()
    );
    report
        .edit_attachment(AttachmentEdit::ConfirmPreview(preview.binding()))
        .unwrap();
    report
        .edit_attachment(AttachmentEdit::ConfirmTime(preview.time().at()))
        .unwrap();
    let encoded =
        encode_manual_attachment(&report, &preview, &AttachmentEncodingSettings::default())
            .unwrap();
    let ImageIntake::Decoded(decoded) =
        tallyline_adapter_files::api::read_image(encoded.png(), &ImageIntakeSettings::default())
    else {
        panic!("encoded crop is not a PNG")
    };
    assert_eq!(decoded.raster, *preview.raster());
    assert!(encoded.base64().len() <= 10_000_000);
    assert!(!format!("{encoded:?}").contains(encoded.base64()));
    assert!(
        encode_manual_attachment(
            &report,
            &preview,
            &AttachmentEncodingSettings::new(4).unwrap()
        )
        .is_err()
    );
    let changed_region = prepare_manual_attachment_file(
        &path,
        AttachmentRegion::new(9, 16, 32, 24).unwrap(),
        &ImageIntakeSettings::default(),
    )
    .unwrap();
    assert!(
        encode_manual_attachment(
            &report,
            &changed_region,
            &AttachmentEncodingSettings::default()
        )
        .is_err()
    );
    source.put_pixel(63, 0, image::Rgb([0, 0, 255]));
    source
        .save_with_format(&path, image::ImageFormat::Png)
        .unwrap();
    let changed_content =
        prepare_manual_attachment_file(&path, region, &ImageIntakeSettings::default()).unwrap();
    assert!(
        encode_manual_attachment(
            &report,
            &changed_content,
            &AttachmentEncodingSettings::default()
        )
        .is_err()
    );
    assert_eq!(changed_content.raster(), preview.raster());
    assert!(
        prepare_manual_attachment_file(
            &path,
            AttachmentRegion::new(63, 40, 2, 2).unwrap(),
            &ImageIntakeSettings::default()
        )
        .is_err()
    );
    assert!(
        prepare_manual_attachment_file(
            &path,
            region,
            &ImageIntakeSettings::new(200 * 1024 * 1024, 100_000_000, 1024 * 1024, 1).unwrap()
        )
        .is_err()
    );
    std::fs::write(&path, [0x76, 0x2f, 0x31, 0x01]).unwrap();
    assert_eq!(
        prepare_manual_attachment_file(&path, region, &ImageIntakeSettings::default()).unwrap_err(),
        tallyline_adapter_files::api::model::AttachmentImageError::UnsupportedImage(
            tallyline_adapter_files::api::model::SetAsideReason::HdrFormat(
                tallyline_adapter_files::api::model::HdrContainer::OpenExr
            )
        )
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags use a double underscore separator."
)]
fn r_man_5_r_sub_7_r_cap_8__jpeg_magic_and_imported_time_are_preserved_without_original_storage() {
    let directory =
        std::env::temp_dir().join(format!("tallyline-private-jpeg-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("capture.png");
    let mut bytes = Vec::new();
    let mut pixels = vec![80; 64 * 48 * 3];
    for row in pixels.as_chunks_mut::<{ 64 * 3 }>().0.iter_mut().take(12) {
        row.fill(240);
    }
    jpeg_encoder::Encoder::new(&mut bytes, 100)
        .encode(&pixels, 64, 48, jpeg_encoder::ColorType::Rgb)
        .unwrap();
    std::fs::write(&path, bytes).unwrap();
    std::fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(
            std::fs::FileTimes::new()
                .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(900)),
        )
        .unwrap();
    let preview = prepare_manual_attachment_file(
        &path,
        AttachmentRegion::new(4, 20, 40, 24).unwrap(),
        &ImageIntakeSettings::default(),
    )
    .unwrap();
    assert_eq!(
        preview.time(),
        tallyline_reporting::api::model::AttachmentTime::ImportedFile(Timestamp::from_seconds(900))
    );
    assert!(
        preview
            .raster()
            .argb()
            .iter()
            .all(|pixel| *pixel == 0xff50_5050)
    );
    let mut report = Report::manual(
        ReportId(1),
        ReportKey::new(TerminalId(11), TradeSide::Buy, GameEnvironment::Live).unwrap(),
        Timestamp::from_seconds(1000),
        Some("4.10.2"),
    )
    .unwrap();
    report
        .edit_attachment(AttachmentEdit::Select(preview.binding(), preview.time()))
        .unwrap();
    report
        .edit_attachment(AttachmentEdit::ConfirmPreview(preview.binding()))
        .unwrap();
    report
        .edit_attachment(AttachmentEdit::ConfirmTime(Timestamp::from_seconds(899)))
        .unwrap();
    std::fs::remove_file(&path).unwrap();
    let encoded =
        encode_manual_attachment(&report, &preview, &AttachmentEncodingSettings::default())
            .unwrap();
    let ImageIntake::Decoded(decoded) =
        tallyline_adapter_files::api::read_image(encoded.png(), &ImageIntakeSettings::default())
    else {
        panic!("encoded JPEG region is invalid")
    };
    assert_eq!(decoded.raster, *preview.raster());
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0);
    assert_eq!(report.observed_at(), Timestamp::from_seconds(899));
    std::fs::remove_dir(directory).unwrap();
}
