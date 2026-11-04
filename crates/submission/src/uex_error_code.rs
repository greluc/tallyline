// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Bounded technical UEX status identifiers without remote message text.

/// A bounded ASCII status identifier safe for passive queue presentation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UexErrorCode(Box<str>);
impl UexErrorCode {
    /// Accepts nonempty ASCII alphanumeric, underscore or hyphen identifiers up to 128 bytes.
    #[must_use]
    pub fn new(value: String) -> Option<Self> {
        (!value.is_empty()
            && value.len() <= 128
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')))
        .then(|| Self(value.into_boxed_str()))
    }
    /// Borrows the exact bounded technical identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
