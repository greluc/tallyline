// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Manual worker wiring for saved numerical review commands.

use tallyline_reporting::api::model::ReviewField;
use tallyline_reporting::api::{ReportRepository, ReportingApplication, RepositoryError};
use tallyline_shared_kernel::api::Clock;
use tallyline_ui::api::model::{BackgroundEvent, ManualReviewAction, ManualReviewCommand};
pub(crate) fn execute<R: ReportRepository, C: Clock>(
    application: &mut ReportingApplication<R, C>,
    command: ManualReviewCommand,
) -> Result<BackgroundEvent, RepositoryError> {
    let ManualReviewCommand {
        id,
        expected,
        action,
        snapshot,
    } = command;
    let version = match action {
        ManualReviewAction::Reassess => expected,
        ManualReviewAction::Focus(ReviewField::Price(row)) => {
            application.focus_price(id, expected, row)?.version()
        }
        ManualReviewAction::Focus(ReviewField::Status(row)) => {
            application.focus_status(id, expected, row)?.version()
        }
        ManualReviewAction::Confirm(ReviewField::Price(row)) => application
            .confirm_price(id, expected, row, &snapshot)?
            .version(),
        ManualReviewAction::Confirm(ReviewField::Status(row)) => application
            .confirm_status(id, expected, row, &snapshot)?
            .version(),
    };
    let review = application.review_manual(id, version, &snapshot)?;
    Ok(BackgroundEvent::ReportReviewed {
        expected,
        review,
        snapshot,
    })
}
