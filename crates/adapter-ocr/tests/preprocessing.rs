// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Recognition tensor input contracts.

#![expect(
    non_snake_case,
    reason = "Requirement tags contain a double underscore."
)]

use tallyline_adapter_ocr::api::model::OcrTensorBuffer;
use tallyline_shared_kernel::api::model::ImageRaster;

#[test]
fn r_ocr_1__recognition_input_is_bgr_nchw_with_normalised_zero_padding() {
    let image = ImageRaster::new(1, 1, vec![0xffff_8000]).unwrap();
    let mut buffer = OcrTensorBuffer::default();
    let tensor = buffer.recognition(&image, 320).unwrap();
    assert_eq!(tensor.shape, [1, 3, 48, 320]);
    assert_eq!(tensor.resized_width, 48);
    assert_eq!(tensor.values[0], -1.0);
    assert!((tensor.values[48 * 320] - (128.0 / 255.0 - 0.5) / 0.5).abs() < 0.000001);
    assert_eq!(tensor.values[2 * 48 * 320], 1.0);
    assert_eq!(tensor.values[47], -1.0);
    assert_eq!(tensor.values[48], 0.0);
    assert_eq!(tensor.values[3 * 48 * 320 - 1], 0.0);
}
#[test]
fn r_ocr_1__recognition_resize_uses_pixel_centres_and_reuse_clears_padding() {
    let pixels = [0xff00_0000, 0xffff_ffff].repeat(24);
    let image = ImageRaster::new(2, 24, pixels).unwrap();
    let mut buffer = OcrTensorBuffer::default();
    let tensor = buffer.recognition(&image, 4).unwrap();
    let expected = [
        -1.0,
        (64.0 / 255.0 - 0.5) / 0.5,
        (191.0 / 255.0 - 0.5) / 0.5,
        1.0,
    ];
    for (actual, expected) in tensor.values.iter().take(4).zip(expected) {
        assert!((actual - expected).abs() < 0.000001);
    }
    let black = ImageRaster::new(1, 48, vec![0xff00_0000; 48]).unwrap();
    let tensor = buffer.recognition(&black, 4).unwrap();
    assert_eq!(tensor.values[0], -1.0);
    assert_eq!(tensor.values[1], 0.0);
    assert!(buffer.recognition(&image, 3).is_err());
    assert!(buffer.recognition(&image, 3201).is_err());
}
