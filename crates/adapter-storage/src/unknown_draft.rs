// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Atomic explicit new-Draft creation from unknown submission history.

use crate::api::SqliteReferenceCache;
use rusqlite::TransactionBehavior;
use tallyline_shared_kernel::api::model::Timestamp;
use tallyline_submission::api::model::{JobId, SubmissionRepositoryError};
use tallyline_workflows::api::{UnknownDraft, UnknownDraftStore};

impl UnknownDraftStore for SqliteReferenceCache {
    fn duplicate_unknown(
        &mut self,
        id: JobId,
        expected_revision: u64,
        at: Timestamp,
    ) -> Result<UnknownDraft, SubmissionRepositoryError> {
        use tallyline_shared_kernel::api::model::ReportId;
        use tallyline_submission::api::model::{SubmissionCommand, UnknownResolutionAction};
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| SubmissionRepositoryError::Unavailable)?;
        let origin =
            crate::queue::load_job(&transaction, id)?.ok_or(SubmissionRepositoryError::Invalid)?;
        if origin.revision() != expected_revision {
            return Err(SubmissionRepositoryError::Conflict);
        }
        if !origin.can_duplicate_unknown() {
            return Err(SubmissionRepositoryError::Invalid);
        }
        let report = crate::queue::load_report(&transaction, origin.request().header().report)?;
        transaction
            .execute("INSERT INTO reports(version,payload) VALUES(0,'')", [])
            .map_err(|_| SubmissionRepositoryError::Unavailable)?;
        let identity = ReportId(
            transaction
                .last_insert_rowid()
                .try_into()
                .map_err(|_| SubmissionRepositoryError::Invalid)?,
        );
        let draft = report
            .duplicate_as_draft(identity)
            .map_err(|_| SubmissionRepositoryError::Invalid)?;
        crate::queue::save_report(&transaction, &draft, 0)?;
        transaction.execute("INSERT INTO report_recents(report_id,changed_order) VALUES(?1,(SELECT COALESCE(MAX(changed_order),0)+1 FROM report_recents))", [transaction.last_insert_rowid()]).map_err(|_| SubmissionRepositoryError::Unavailable)?;
        let change = origin
            .apply(SubmissionCommand::ResolveUnknown(
                UnknownResolutionAction::Duplicated {
                    report: identity,
                    at,
                },
            ))
            .map_err(|_| SubmissionRepositoryError::Invalid)?;
        crate::queue::save_change(&transaction, &change)?;
        transaction
            .commit()
            .map_err(|_| SubmissionRepositoryError::Unavailable)?;
        Ok(UnknownDraft {
            origin: change.into_state(),
            draft,
        })
    }
}
