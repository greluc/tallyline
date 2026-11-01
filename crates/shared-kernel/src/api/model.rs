// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shared identities, exact decimal values and precise Unix timestamps.

pub use crate::container_sizes::ContainerSize;
pub use crate::decimal::{InvalidDecimal, PricePerScu, ScuQuantity};

use std::time::Duration;

/// Unix time with millisecond precision and the full unsigned-second range.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(Duration);

impl Timestamp {
    /// Creates a timestamp from Unix seconds.
    #[must_use]
    pub const fn from_seconds(seconds: u64) -> Self {
        Self(Duration::from_secs(seconds))
    }

    /// Creates a timestamp from Unix milliseconds.
    #[must_use]
    pub const fn from_millis(millis: u64) -> Self {
        Self(Duration::from_millis(millis))
    }

    /// Returns whole Unix seconds.
    #[must_use]
    pub const fn seconds(self) -> u64 {
        self.0.as_secs()
    }

    /// Returns exact Unix milliseconds without overflowing the second range.
    #[must_use]
    pub const fn millis(self) -> u128 {
        self.0.as_millis()
    }
}

/// The numeric UEX commodity identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommodityId(pub u32);

/// The numeric UEX terminal identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TerminalId(pub u32);

/// The UEX universe selected by a game-channel mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UexEnvironment {
    /// Production universe.
    Live,
    /// Public test universe.
    Ptu,
}

/// The terminal's buy tab or local market value sell tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradeSide {
    /// Buy inventory from the terminal.
    Buy,
    /// Sell inventory to the terminal.
    Sell,
}
pub use crate::image_raster::{ImageRaster, InvalidRaster};
