// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! User-requested exact-value approval within the dialog presenter.

use crate::api::model::{ManualReviewAction, ManualReviewCommand, ManualReviewState};
use crate::api::{Catalogue, DialogPresenter};
use tallyline_reporting::api::model::ReviewField;
impl DialogPresenter {
    /// Opens a confirmation only after an explicit action on a current required decision.
    pub fn open_manual_confirmation(&mut self, state: &ManualReviewState, field: ReviewField) {
        self.manual_confirmation = state.confirm(field);
    }
    /// Reports whether the user has requested a manual value confirmation.
    #[must_use]
    pub const fn manual_confirmation_open(&self) -> bool {
        self.manual_confirmation.is_some()
    }
    /// Draws exact value and reference details before returning an explicit approval request.
    /// Evidence must be the selected Report's already redacted attachment preview.
    pub fn show_manual_confirmation(
        &mut self,
        context: &egui::Context,
        state: &ManualReviewState,
        catalogue: &Catalogue,
        theme: tallyline_egui::Theme,
        evidence: impl FnOnce(&mut egui::Ui),
    ) -> Option<ManualReviewCommand> {
        let request = self.manual_confirmation.as_ref()?.clone();
        let ManualReviewAction::Confirm(field) = request.action else {
            self.manual_confirmation = None;
            return None;
        };
        if !state.is_current(&request) || !state.needs_confirmation(field) {
            self.manual_confirmation = None;
            return None;
        }
        let review = state.review()?;
        let row = review
            .report()
            .rows()
            .iter()
            .find(|row| row.id() == field.row())?;
        let value = match field {
            ReviewField::Price(_) => {
                format!("{} {}", row.price()?, catalogue.text("review.per_scu"))
            }
            ReviewField::Status(_) => format!(
                "{} {}",
                catalogue.text("review.status_value"),
                row.status()?
            ),
        };
        let mut approved = false;
        let mut cancelled = false;
        let modal =
            egui::Modal::new(egui::Id::unique("manual-value-confirmation")).show(context, |ui| {
                ui.set_max_width(700.0);
                ui.heading(catalogue.text("review.confirm_title"));
                ui.label(value);
                ui.label(catalogue.text("review.approval_scope"));
                match field {
                    ReviewField::Price(_) => {
                        if let Some(assessment) = review
                            .prices()
                            .iter()
                            .find(|assessment| assessment.row() == field.row())
                        {
                            crate::manual_review_details::price_details(
                                ui,
                                *assessment,
                                catalogue,
                                theme,
                            );
                        }
                    }
                    ReviewField::Status(_) => {
                        if let Some(assessment) = review
                            .statuses()
                            .iter()
                            .find(|assessment| assessment.row() == field.row())
                        {
                            crate::manual_review_details::status_details(
                                ui,
                                *assessment,
                                catalogue,
                                theme,
                            );
                        }
                    }
                }
                evidence(ui);
                ui.horizontal(|ui| {
                    approved = ui.button(catalogue.text("review.approve")).clicked();
                    cancelled = ui.button(catalogue.text("review.cancel")).clicked();
                });
            });
        if approved || cancelled || modal.should_close() {
            self.manual_confirmation = None;
        }
        approved.then_some(request)
    }
}
