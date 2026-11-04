// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Public supplied-panel preparation contracts.

use tallyline_recognition::api::model::{
    GeometryProvenance, PanelGeometry, PanelLimits, SuppliedPanels,
};
use tallyline_recognition::api::prepare_panels;
use tallyline_shared_kernel::api::model::ImageRaster;

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags contain a double underscore."
)]
fn r_ocr_2__supplied_skewed_and_tall_panels_keep_upright_mapping_and_source_pixels() {
    let image = ImageRaster::new(8, 8, (0..64).map(|v| 0xff00_0000 | v).collect()).unwrap();
    let before = image.clone();
    let supplied = SuppliedPanels {
        shop: PanelGeometry {
            corners: [[2., 2.], [6., 2.], [5., 6.], [3., 6.]],
            output: [4, 4],
        },
        location: PanelGeometry {
            corners: [[0., 2.], [2., 2.], [2., 8.], [0., 8.]],
            output: [2, 6],
        },
        header: [[0., 0.], [8., 0.], [8., 1.], [0., 1.]],
        provenance: GeometryProvenance::Manual,
    };
    let prepared = prepare_panels(&image, &supplied, PanelLimits::default()).unwrap();
    assert_eq!(prepared.provenance(), GeometryProvenance::Manual);
    assert!(prepared.requires_privacy_confirmation());
    assert_eq!(
        [
            prepared.location().raster().width(),
            prepared.location().raster().height()
        ],
        [2, 6]
    );
    assert_eq!(prepared.location().raster().argb()[0], 0xff00_0010);
    assert_eq!(prepared.location().raster().argb()[11], 0xff00_0039);
    let source = prepared.shop().transform().to_source([2., 2.]).unwrap();
    assert!((source[0] - 4.).abs() < 1e-10);
    assert!((source[1] - 14. / 3.).abs() < 1e-10);
    let normalised = prepared
        .shop()
        .transform()
        .from_source([4., 14. / 3.])
        .unwrap();
    assert!((normalised[0] - 2.).abs() < 1e-10);
    assert!((normalised[1] - 2.).abs() < 1e-10);
    assert_eq!(image, before);
}
#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags contain a double underscore."
)]
fn r_cap_7_r_sub_7__fractional_header_pixels_are_masked_before_projective_bilinear_filtering() {
    let supplied = SuppliedPanels {
        shop: PanelGeometry {
            corners: [[1.25, 2.1], [5.75, 2.4], [5.25, 6.], [1.75, 6.]],
            output: [9, 8],
        },
        location: PanelGeometry {
            corners: [[0., 2.], [2., 2.], [2., 5.], [0., 5.]],
            output: [4, 6],
        },
        header: [[0., 0.], [8., 0.], [8., 2.25], [0., 2.25]],
        provenance: GeometryProvenance::Profile,
    };
    let make_image = |secret| {
        ImageRaster::new(
            8,
            8,
            (0..64)
                .map(|i| if i / 8 < 3 { secret } else { 0xff22_4466 })
                .collect(),
        )
        .unwrap()
    };
    let first =
        prepare_panels(&make_image(0xffff_00ff), &supplied, PanelLimits::default()).unwrap();
    let second =
        prepare_panels(&make_image(0xffff_ffff), &supplied, PanelLimits::default()).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.location().raster().argb()[0], 0xff00_0000);
    assert_eq!(
        *first.location().raster().argb().last().unwrap(),
        0xff22_4466
    );
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags contain a double underscore."
)]
fn r_ocr_2_r_cap_7__minification_averages_source_footprints_after_privacy_masking() {
    let image = ImageRaster::new(
        8,
        8,
        (0..64)
            .map(|i| if i % 8 == 1 { 0xffff_ffff } else { 0xff00_0000 })
            .collect(),
    )
    .unwrap();
    for (header_bottom, expected) in [(1., 0xff40_4040), (2.25, 0xff30_3030)] {
        let supplied = SuppliedPanels {
            shop: PanelGeometry {
                corners: [[0., 2.], [4., 2.], [4., 6.], [0., 6.]],
                output: [1, 1],
            },
            location: PanelGeometry {
                corners: [[4., 2.], [6., 2.], [6., 6.], [4., 6.]],
                output: [1, 1],
            },
            header: [[0., 0.], [8., 0.], [8., header_bottom], [0., header_bottom]],
            provenance: GeometryProvenance::Manual,
        };
        let result = prepare_panels(&image, &supplied, PanelLimits::default()).unwrap();
        assert_eq!(result.shop().raster().argb(), [expected]);
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags contain a double underscore."
)]
fn r_ocr_2__invalid_geometry_and_excessive_output_or_sampling_are_typed_refusals() {
    use tallyline_recognition::api::model::PanelRefusal;
    let image = ImageRaster::new(8, 8, vec![0xff22_4466; 64]).unwrap();
    let valid = SuppliedPanels {
        shop: PanelGeometry {
            corners: [[2., 2.], [6., 2.], [6., 6.], [2., 6.]],
            output: [4, 4],
        },
        location: PanelGeometry {
            corners: [[0., 2.], [2., 2.], [2., 6.], [0., 6.]],
            output: [2, 4],
        },
        header: [[0., 0.], [8., 0.], [8., 1.], [0., 1.]],
        provenance: GeometryProvenance::Manual,
    };
    for corners in [
        [[0., 0.]; 4],
        [[0., 0.], [2., 2.], [2., 0.], [0., 2.]],
        [[1., 1.], [2., 1.], [3., 1.], [4., 1.]],
        [[f64::NAN, 1.], [6., 2.], [6., 6.], [2., 6.]],
        [[f64::INFINITY, 1.], [6., 2.], [6., 6.], [2., 6.]],
        [[-0.1, 1.], [6., 2.], [6., 6.], [2., 6.]],
        [[2., 2.], [9., 2.], [6., 6.], [2., 6.]],
    ] {
        let mut supplied = valid;
        supplied.shop.corners = corners;
        assert_eq!(
            prepare_panels(&image, &supplied, PanelLimits::default()),
            Err(PanelRefusal::Geometry)
        );
    }
    let mut oversized = valid;
    oversized.shop.output = [u32::MAX, u32::MAX];
    oversized.location.output = [u32::MAX, u32::MAX];
    assert_eq!(
        prepare_panels(&image, &oversized, PanelLimits::default()),
        Err(PanelRefusal::OutputLimit)
    );
    let limits = PanelLimits::new(4096, 4_194_304, 1).unwrap();
    assert_eq!(
        prepare_panels(&image, &valid, limits),
        Err(PanelRefusal::SamplingLimit)
    );
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags contain a double underscore."
)]
fn r_ocr_2_r_cap_7__bounded_mapping_translation_and_privacy_invariants_hold_across_working_scales()
{
    for output in [[1, 1], [3, 4], [9, 11]] {
        let supplied = SuppliedPanels {
            shop: PanelGeometry {
                corners: [[2., 2.1], [6., 2.3], [5.5, 6.], [2.5, 6.]],
                output,
            },
            location: PanelGeometry {
                corners: [[1., 3.], [2., 3.], [2., 5.], [1., 5.]],
                output: [2, 4],
            },
            header: [[0., 0.], [8., 0.], [8., 2.25], [0., 2.25]],
            provenance: GeometryProvenance::Manual,
        };
        let mut baseline = None;
        for secret in [0xff00_0000, 0xffff_ffff, 0xfff1_02e3] {
            let image = ImageRaster::new(
                8,
                8,
                (0..64)
                    .map(|i| {
                        if i / 8 < 3 {
                            secret
                        } else {
                            0xff00_0000 | ((i * 3) << 8) | i
                        }
                    })
                    .collect(),
            )
            .unwrap();
            let prepared = prepare_panels(&image, &supplied, PanelLimits::default()).unwrap();
            if let Some(first) = &baseline {
                assert_eq!(first, &prepared);
            } else {
                baseline = Some(prepared);
            }
        }
        let image = ImageRaster::new(8, 8, (0..64).map(|i| 0xff00_0000 | i).collect()).unwrap();
        let original = prepare_panels(&image, &supplied, PanelLimits::default()).unwrap();
        let mut padded = vec![0xff12_3456; 12 * 12];
        for y in 0..8 {
            for x in 0..8 {
                padded[(y + 1) * 12 + x + 2] = image.argb()[y * 8 + x];
            }
        }
        let padded = ImageRaster::new(12, 12, padded).unwrap();
        let shift = |corners: [[f64; 2]; 4]| corners.map(|[x, y]| [x + 2., y + 1.]);
        let translated = SuppliedPanels {
            shop: PanelGeometry {
                corners: shift(supplied.shop.corners),
                ..supplied.shop
            },
            location: PanelGeometry {
                corners: shift(supplied.location.corners),
                ..supplied.location
            },
            header: shift(supplied.header),
            ..supplied
        };
        let translated = prepare_panels(&padded, &translated, PanelLimits::default()).unwrap();
        assert_eq!(translated.shop().raster(), original.shop().raster());
        assert_eq!(translated.location().raster(), original.location().raster());
        for point in [[0., 0.], [0.5, 0.5], [1., 1.]] {
            let point = [
                point[0] * f64::from(output[0]),
                point[1] * f64::from(output[1]),
            ];
            let source = original.shop().transform().to_source(point).unwrap();
            let shifted = translated.shop().transform().to_source(point).unwrap();
            assert!((shifted[0] - source[0] - 2.).abs() < 1e-9);
            assert!((shifted[1] - source[1] - 1.).abs() < 1e-9);
            let roundtrip = original.shop().transform().from_source(source).unwrap();
            assert!((roundtrip[0] - point[0]).abs() < 1e-9);
            assert!((roundtrip[1] - point[1]).abs() < 1e-9);
        }
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags contain a double underscore."
)]
fn r_cap_7__a_panel_with_no_extent_outside_the_header_is_refused() {
    let image = ImageRaster::new(8, 8, vec![0xff00_0000; 64]).unwrap();
    let supplied = SuppliedPanels {
        shop: PanelGeometry {
            corners: [[2., 2.], [6., 2.], [6., 6.], [2., 6.]],
            output: [4, 4],
        },
        location: PanelGeometry {
            corners: [[0., 2.], [2., 2.], [2., 5.], [0., 5.]],
            output: [2, 3],
        },
        header: [[0., 0.], [8., 0.], [8., 8.], [0., 8.]],
        provenance: GeometryProvenance::Manual,
    };
    assert!(prepare_panels(&image, &supplied, PanelLimits::default()).is_err());
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags contain a double underscore."
)]
fn r_ocr_2__mapping_refuses_nonfinite_points_and_overflowing_inverse_outputs() {
    let image = ImageRaster::new(8, 8, vec![0xff22_4466; 64]).unwrap();
    let supplied = SuppliedPanels {
        shop: PanelGeometry {
            corners: [[1., 2.], [2., 2.], [2., 3.], [1., 3.]],
            output: [4096, 1],
        },
        location: PanelGeometry {
            corners: [[0., 3.], [2., 3.], [2., 5.], [0., 5.]],
            output: [2, 2],
        },
        header: [[0., 0.], [8., 0.], [8., 1.], [0., 1.]],
        provenance: GeometryProvenance::Manual,
    };
    let prepared = prepare_panels(&image, &supplied, PanelLimits::default()).unwrap();
    assert!(
        prepared
            .shop()
            .transform()
            .to_source([f64::NAN, 0.])
            .is_none()
    );
    assert!(
        prepared
            .shop()
            .transform()
            .from_source([f64::MAX, 0.])
            .is_none()
    );
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags contain a double underscore."
)]
fn r_nf_5__panel_limits_validate_tighten_only_values_at_construction() {
    let limits = PanelLimits::new(64, 1024, 4096).unwrap();
    assert_eq!(limits.max_side(), 64);
    assert_eq!(limits.max_total_pixels(), 1024);
    assert_eq!(limits.max_sampled_pixels(), 4096);
    for (side, pixels, samples) in [
        (0, 1, 1),
        (4097, 1, 1),
        (1, 0, 1),
        (1, 4_194_305, 1),
        (1, 1, 0),
        (1, 1, 128_000_001),
    ] {
        assert!(PanelLimits::new(side, pixels, samples).is_err());
    }
}

#[test]
#[expect(
    non_snake_case,
    reason = "Requirement tags contain a double underscore."
)]
fn r_cap_7_r_sub_7__header_boundary_excludes_the_upper_region_before_every_filter() {
    for (header, corners, output) in [
        (
            [[0., 2.], [8., 2.], [8., 3.], [0., 3.]],
            [[0., 0.], [8., 0.], [8., 6.], [0., 6.]],
            [8, 6],
        ),
        (
            [[0., 1.8], [8., 2.2], [8., 3.2], [0., 2.8]],
            [[0.5, 0.2], [7.5, 0.4], [6.5, 6.5], [1.5, 6.5]],
            [14, 12],
        ),
        (
            [[0., 1.8], [8., 2.2], [8., 3.2], [0., 2.8]],
            [[0.5, 0.2], [7.5, 0.4], [6.5, 6.5], [1.5, 6.5]],
            [3, 2],
        ),
    ] {
        let mut unpadded = None;
        for [dx, dy] in [[0_u32, 0_u32], [2, 1]] {
            let width = 8 + 2 * dx;
            let height = 8 + 2 * dy;
            let shift =
                |quad: [[f64; 2]; 4]| quad.map(|[x, y]| [x + f64::from(dx), y + f64::from(dy)]);
            let supplied = SuppliedPanels {
                shop: PanelGeometry {
                    corners: shift(corners),
                    output,
                },
                location: PanelGeometry {
                    corners: shift([[0., 4.], [2., 4.], [2., 8.], [0., 8.]]),
                    output: [2, 4],
                },
                header: shift(header),
                provenance: GeometryProvenance::Profile,
            };
            let image = |secret| {
                ImageRaster::new(
                    width,
                    height,
                    (0..width * height)
                        .map(|index| {
                            if index / width < 4 + dy {
                                secret
                            } else {
                                0xff22_4466
                            }
                        })
                        .collect(),
                )
                .unwrap()
            };
            let first_source = image(0xffff_00ff);
            let before = first_source.clone();
            let first = prepare_panels(&first_source, &supplied, PanelLimits::default()).unwrap();
            let second =
                prepare_panels(&image(0xff00_ff11), &supplied, PanelLimits::default()).unwrap();
            assert_eq!(first, second);
            assert_eq!(first_source, before);
            assert_eq!(first.shop().raster().argb()[0], 0xff00_0000);
            assert!(
                first
                    .shop()
                    .raster()
                    .argb()
                    .iter()
                    .any(|pixel| *pixel != 0xff00_0000)
            );
            assert!(
                first
                    .location()
                    .raster()
                    .argb()
                    .iter()
                    .all(|pixel| *pixel == 0xff22_4466)
            );
            if let Some(reference) = &unpadded {
                assert_eq!(first.shop().raster(), reference);
            } else {
                unpadded = Some(first.shop().raster().clone());
            }
        }
    }
}
