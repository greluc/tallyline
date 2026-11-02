// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Manual Draft identities and immutable read models.

pub use crate::report::{
    FieldOrigin, InvalidReport, Report, ReportKey, ReportRow, StoredReport, StoredRow,
};

pub use tallyline_shared_kernel::api::model::{ReportId, RowId};

/// Report kinds reserved by the domain model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportType {
    /// Commodity terminal observations.
    Commodity,
    /// Item observations, without an entry UI.
    Item,
    /// Vehicle purchase observations, without an entry UI.
    VehicleBuy,
    /// Vehicle rental observations, without an entry UI.
    VehicleRent,
}
pub use crate::assessment::{Deviation, StatusAssessment};
pub use crate::attachment::{
    AttachmentBinding, AttachmentBlock, AttachmentRegion, AttachmentTime, ImageHash,
    ManualAttachment, StoredAttachment,
};
pub use crate::confirmation::StatusConfirmation;
pub use crate::manual_review::{ManualReview, ManualReviewSummary, ReviewField};
pub use crate::price_assessment::{AssessmentUnavailable, PriceAssessment};
pub use crate::price_confirmation::{PriceConfirmation, StoredPriceDecision};
pub use crate::stock_assessment::StockAssessment;
pub use tallyline_reference_data::api::model::PriceReferenceSource;
