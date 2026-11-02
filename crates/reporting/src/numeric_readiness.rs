// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Numeric and decision readiness of a committed manual review.

use crate::api::model::{ManualReview, ReviewField};

/// A numerical or explicit-decision block for an entered manual row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericBlock {
    /// No row has an explicitly entered value.
    NoEnteredRows,
    /// A sent row lacks a mandatory value.
    MissingValue(ReviewField),
    /// An entered field still requires explicit approval.
    NeedsConfirmation(ReviewField),
}
impl ManualReview {
    /// Checks all mandatory values and decisions in this exact committed review.
    /// Age, version, account, attachment and submission policy require their separate gates.
    ///
    /// # Errors
    /// Returns every numerical and decision block in stable row order.
    pub fn numeric_readiness(&self) -> Result<(), Vec<NumericBlock>> {
        let mut blocks = Vec::new();
        let mut entered = false;
        for row in self.report().rows().iter().filter(|row| row.entered()) {
            entered = true;
            for (field, missing) in [
                (ReviewField::Price(row.id()), row.price().is_none()),
                (ReviewField::Stock(row.id()), row.stock().is_none()),
                (ReviewField::Status(row.id()), row.status().is_none()),
            ] {
                if missing {
                    blocks.push(NumericBlock::MissingValue(field));
                }
            }
            for (field, required) in [
                (ReviewField::Price(row.id()), row.price_needs_confirmation()),
                (ReviewField::Stock(row.id()), row.stock_needs_confirmation()),
                (
                    ReviewField::Status(row.id()),
                    row.status_needs_confirmation(),
                ),
            ] {
                if required {
                    blocks.push(NumericBlock::NeedsConfirmation(field));
                }
            }
        }
        if !entered {
            blocks.push(NumericBlock::NoEnteredRows);
        }
        if blocks.is_empty() {
            Ok(())
        } else {
            Err(blocks)
        }
    }
}
