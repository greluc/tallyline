// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Bounded reusable recognition tensor storage.

use tallyline_shared_kernel::api::model::ImageRaster;

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
        let source_width = usize::try_from(image.width()).map_err(|_| TensorInputError)?;
        let source_height = usize::try_from(image.height()).map_err(|_| TensorInputError)?;
        for (channel, shift) in CHANNEL_SHIFTS.into_iter().enumerate() {
            for y in 0..HEIGHT {
                let (top, bottom, vertical) = sampling(y, source_height, HEIGHT);
                for x in 0..resized_width {
                    let (left, right, horizontal) = sampling(x, source_width, resized_width);
                    let pixel = |row: usize, column: usize| -> Result<f64, TensorInputError> {
                        let argb = image
                            .argb()
                            .get(row * source_width + column)
                            .ok_or(TensorInputError)?;
                        Ok(f64::from((argb >> shift) & 255))
                    };
                    let upper =
                        pixel(top, left)? * (1.0 - horizontal) + pixel(top, right)? * horizontal;
                    let lower = pixel(bottom, left)? * (1.0 - horizontal)
                        + pixel(bottom, right)? * horizontal;
                    let code = (upper * (1.0 - vertical) + lower * vertical).round() as f32;
                    let index = channel * HEIGHT * padded_width + y * padded_width + x;
                    *self.values.get_mut(index).ok_or(TensorInputError)? =
                        (code / 255.0 - 0.5) / 0.5;
                }
            }
        }
        Ok(OcrInputTensor {
            shape: [1, 3, HEIGHT, padded_width],
            resized_width,
            values: &self.values,
        })
    }
}

fn sampling(destination: usize, source_size: usize, target_size: usize) -> (usize, usize, f64) {
    let position = ((destination as f64 + 0.5) * source_size as f64 / target_size as f64 - 0.5)
        .clamp(0.0, (source_size - 1) as f64);
    let first = position.floor() as usize;
    (
        first,
        (first + 1).min(source_size - 1),
        position - first as f64,
    )
}
