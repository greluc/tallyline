// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! One copying policy for separate reviewed Drafts.

use super::{InvalidReport, Report, ReportId};
use crate::api::AttachmentEdit;
use crate::api::model::ReportState;

impl Report {
    /// Copies entered values and their original observation into a separate unreviewed Draft.
    ///
    /// # Errors
    /// Refuses another source phase or a reused or missing identity.
    pub fn duplicate_as_draft(&self, id: ReportId) -> Result<Self, InvalidReport> {
        if self.state() != ReportState::OutcomeUnknown || id == self.id() {
            return Err(InvalidReport);
        }
        let mut draft = Self::manual(id, self.key(), self.observed_at(), self.game_version())?;
        for source in self.rows() {
            let row = draft.add_commodity(source.commodity())?;
            draft.set_price(row, source.price())?;
            draft.set_stock(row, source.stock())?;
            draft.set_status(row, source.status())?;
        }
        if let Some(attachment) = self.attachment() {
            draft.edit_attachment(AttachmentEdit::Select(
                attachment.binding(),
                attachment.time(),
            ))?;
        }
        Ok(draft)
    }
}
