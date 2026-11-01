// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Bounded reusable recognition tensor storage.

use tallyline_shared_kernel::api::model::ImageRaster;

mod linear;

const HEIGHT: usize = 48;
const MAX_WIDTH: usize = 3200;
const CHANNEL_SHIFTS: [u32; 3] = [0, 8, 16];

/// Reusable per-worker storage for model input pixels.
#[derive(Default)]
pub struct OcrTensorBuffer {
    values: Vec<f32>,
}

/// A borrowed batch-one tensor in BGR channel-first order.
pub struct OcrInputTensor<'a> {
    /// The exact portable preprocessing algorithm used for these values.
    pub preprocessing: RecognitionPreprocessing,
    /// Batch, channel, height and padded width dimensions.
    pub shape: [usize; 4],
    /// Resized image width, excluding normalised-space padding.
    pub resized_width: usize,
    /// Contiguous NCHW values, valid until the worker reuses its buffer.
    pub values: &'a [f32],
}

/// An image cannot fit the bounded tensor without changing its aspect ratio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TensorInputError;

impl OcrTensorBuffer {
    /// Builds height-48 recognition input with pixel-centre linear resize and zero padding.
    ///
    /// # Errors
    /// Returns `TensorInputError` when the padded width exceeds 3,200 or would shrink the line.
    pub fn recognition(
        &mut self,
        image: &ImageRaster,
        padded_width: usize,
    ) -> Result<OcrInputTensor<'_>, TensorInputError> {
        let width = (u64::from(image.width()) * HEIGHT as u64).div_ceil(u64::from(image.height()));
        let resized_width = usize::try_from(width).map_err(|_| TensorInputError)?;
        if padded_width == 0 || padded_width > MAX_WIDTH || resized_width > padded_width {
            return Err(TensorInputError);
        }
        self.values.resize(3 * HEIGHT * padded_width, 0.0);
        self.values.fill(0.0);
        for (channel, shift) in CHANNEL_SHIFTS.into_iter().enumerate() {
            for y in 0..HEIGHT {
                for x in 0..resized_width {
                    let code = linear::pixel(image, x, y, resized_width, HEIGHT, shift)
                        .ok_or(TensorInputError)?;
                    let index = channel * HEIGHT * padded_width + y * padded_width + x;
                    *self.values.get_mut(index).ok_or(TensorInputError)? =
                        (f32::from(code) / 255.0 - 0.5) / 0.5;
                }
            }
        }
        Ok(OcrInputTensor {
            preprocessing: RecognitionPreprocessing::Linear8uV1,
            shape: [1, 3, HEIGHT, padded_width],
            resized_width,
            values: &self.values,
        })
    }
}

/// A frozen recognition input algorithm for reproducible reader evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecognitionPreprocessing {
    /// BGR, height 48, generic eight-bit fixed-point linear resize and normalised-zero padding.
    Linear8uV1,
}

impl RecognitionPreprocessing {
    /// Returns the stable profile identifier recorded with raw reader results.
    #[must_use]
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::Linear8uV1 => "ppocr-bgr48-linear8u-v1",
        }
    }
}
