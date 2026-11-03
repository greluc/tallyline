// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Deterministic transcription parsing and bounded retry contracts.

pub use crate::parse_answer::{ParseError, parse_answer};
pub use crate::request_contract::{
    FinishReason, ModelAnswer, RequestContract, TranscriptionFailure, parse_with_retry,
};
