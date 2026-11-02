// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Original-raster projective crops with bounded portable bilinear sampling.

use super::{DbError, DetectionBox};
use tallyline_shared_kernel::api::model::ImageRaster;

impl DetectionBox {
    /// Rectifies this convex quadrilateral at source scale using replicated-edge bilinear sampling.
    /// Tall crops are rotated anticlockwise; this portable profile does not claim OpenCV cubic parity.
    ///
    /// # Errors
    /// Rejects nonfinite, out-of-raster, degenerate or nonconvex geometry and crops above
    /// 4,096 pixels per side or four megapixels before allocating output pixels.
    pub fn rectify(&self, image: &ImageRaster) -> Result<ImageRaster, DbError> {
        let [a, b, c, d] = self.corners;
        for [x, y] in self.corners {
            if !x.is_finite()
                || !y.is_finite()
                || x < 0.0
                || y < 0.0
                || x > f64::from(image.width())
                || y > f64::from(image.height())
            {
                return Err(DbError::Geometry);
            }
        }
        for (a, b, c) in [(a, b, c), (b, c, d), (c, d, a), (d, a, b)] {
            if (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]) <= 1e-9 {
                return Err(DbError::Geometry);
            }
        }
        let distance = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]);
        let width = distance(a, b).max(distance(c, d)).floor() as usize;
        let height = distance(a, d).max(distance(b, c)).floor() as usize;
        if width == 0 || height == 0 {
            return Err(DbError::Geometry);
        }
        if width > 4096
            || height > 4096
            || width
                .checked_mul(height)
                .is_none_or(|count| count > 4_194_304)
        {
            return Err(DbError::Complexity);
        }
        let dx1 = b[0] - c[0];
        let dx2 = d[0] - c[0];
        let dx3 = a[0] - b[0] + c[0] - d[0];
        let dy1 = b[1] - c[1];
        let dy2 = d[1] - c[1];
        let dy3 = a[1] - b[1] + c[1] - d[1];
        let denominator = dx1 * dy2 - dx2 * dy1;
        if denominator.abs() <= 1e-9 {
            return Err(DbError::Geometry);
        }
        let g = (dx3 * dy2 - dx2 * dy3) / denominator;
        let h = (dx1 * dy3 - dx3 * dy1) / denominator;
        let transform = [
            b[0] - a[0] + g * b[0],
            d[0] - a[0] + h * d[0],
            a[0],
            b[1] - a[1] + g * b[1],
            d[1] - a[1] + h * d[1],
            a[1],
            g,
            h,
        ];
        let rotated = height as f64 / width as f64 >= 1.5;
        let (out_width, out_height) = if rotated {
            (height, width)
        } else {
            (width, height)
        };
        let mut pixels = Vec::with_capacity(width * height);
        let [m0, m1, m2, m3, m4, m5, m6, m7] = transform;
        for y in 0..out_height {
            for x in 0..out_width {
                let (px, py) = if rotated { (width - 1 - y, x) } else { (x, y) };
                let u = px as f64 / width as f64;
                let v = py as f64 / height as f64;
                let scale = m6 * u + m7 * v + 1.0;
                if !scale.is_finite() || scale.abs() <= 1e-9 {
                    return Err(DbError::Geometry);
                }
                let sx = (m0 * u + m1 * v + m2) / scale;
                let sy = (m3 * u + m4 * v + m5) / scale;
                pixels.push(sample(image, sx, sy)?);
            }
        }
        ImageRaster::new(out_width as u32, out_height as u32, pixels).map_err(|_| DbError::Geometry)
    }
}

fn sample(image: &ImageRaster, x: f64, y: f64) -> Result<u32, DbError> {
    if !x.is_finite() || !y.is_finite() {
        return Err(DbError::Geometry);
    }
    let x = x.clamp(0.0, f64::from(image.width() - 1));
    let y = y.clamp(0.0, f64::from(image.height() - 1));
    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(image.width() as usize - 1);
    let y1 = (y0 + 1).min(image.height() as usize - 1);
    let fx = x - x0 as f64;
    let fy = y - y0 as f64;
    let mut pixel = 0xff00_0000;
    for shift in [0, 8, 16] {
        let mut sum = 0.0;
        for (px, py, weight) in [
            (x0, y0, (1.0 - fx) * (1.0 - fy)),
            (x1, y0, fx * (1.0 - fy)),
            (x0, y1, (1.0 - fx) * fy),
            (x1, y1, fx * fy),
        ] {
            let source = image
                .argb()
                .get(py * image.width() as usize + px)
                .ok_or(DbError::Geometry)?;
            sum += f64::from((source >> shift) & 255) * weight;
        }
        pixel |= (sum.round().clamp(0.0, 255.0) as u32) << shift;
    }
    Ok(pixel)
}
