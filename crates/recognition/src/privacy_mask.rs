// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Full-width upward-closed source-pixel exclusion before any sampling filter.

pub(crate) struct PrivacyMask {
    prohibited_through_y: f64,
}

impl PrivacyMask {
    pub(crate) fn new(corners: [[f64; 2]; 4]) -> Self {
        Self {
            prohibited_through_y: corners
                .into_iter()
                .map(|[_, y]| y)
                .fold(f64::NEG_INFINITY, f64::max),
        }
    }

    pub(crate) fn covers(&self, quad: [[f64; 2]; 4]) -> bool {
        quad.iter().all(|[_, y]| *y <= self.prohibited_through_y)
    }

    pub(crate) fn excludes(&self, y: u32) -> bool {
        f64::from(y) <= self.prohibited_through_y
    }
}
