// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Exact-snapshot unknown receipt checks with local-save-only retries.

use super::SubmissionControlError;
use super::queue_state::QueueState;
use crate::api::HttpClient;
use tallyline_adapter_storage::api::SqliteReferenceCache;
use tallyline_shared_kernel::api::Clock;
use tallyline_submission::api::model::{
    ReleaseToken, SubmissionChange, SubmissionCommand, SubmissionJob, UnknownResolutionAction,
    VerifiedUexReport,
};
use tallyline_submission::api::{CredentialApplication, SubmissionJobRepository};
use tokio::sync::oneshot::Receiver;
use tokio::sync::oneshot::error::TryRecvError;

struct UnknownWork {
    origin: SubmissionJob,
    reply: Option<Receiver<Option<Vec<VerifiedUexReport>>>>,
    change: Option<SubmissionChange>,
}
#[derive(Default)]
pub(super) struct UnknownResolution {
    work: Option<UnknownWork>,
}
impl UnknownResolution {
    pub(super) fn begin(
        &mut self,
        client: &HttpClient,
        origin: SubmissionJob,
        credentials: &CredentialApplication,
        token: Option<&ReleaseToken>,
    ) -> Result<(), SubmissionControlError> {
        if self.work.is_some() {
            return Err(SubmissionControlError::Unavailable);
        }
        let values = credentials
            .credentials_for(&origin.request().header().account)
            .ok_or(SubmissionControlError::Conflict)?;
        let reply = client
            .begin_unknown_receipt_verification(
                &origin,
                values,
                token.map(ReleaseToken::for_validation),
            )
            .map_err(|_| SubmissionControlError::Unavailable)?;
        self.work = Some(UnknownWork {
            origin,
            reply: Some(reply),
            change: None,
        });
        Ok(())
    }

    pub(super) fn advance(
        &mut self,
        store: &mut SqliteReferenceCache,
        queue: &mut QueueState,
        clock: &dyn Clock,
        credentials: &CredentialApplication,
    ) -> Result<bool, SubmissionControlError> {
        let Some(mut work) = self.work.take() else {
            return Ok(false);
        };
        let current = match SubmissionJobRepository::load(store, work.origin.id()) {
            Ok(Some(current)) => current,
            Ok(None) => return Err(SubmissionControlError::Invalid),
            Err(error) => {
                self.work = Some(work);
                return Err(SubmissionControlError::Repository(error));
            }
        };
        if current != work.origin
            || credentials
                .credentials_for(&work.origin.request().header().account)
                .is_none()
        {
            return Err(SubmissionControlError::Conflict);
        }
        if work.change.is_none() {
            let receipts = match work
                .reply
                .as_mut()
                .ok_or(SubmissionControlError::Invalid)?
                .try_recv()
            {
                Ok(Some(receipts)) => receipts,
                Ok(None) | Err(TryRecvError::Closed) => {
                    return Err(SubmissionControlError::Unavailable);
                }
                Err(TryRecvError::Empty) => {
                    self.work = Some(work);
                    return Ok(false);
                }
            };
            work.reply = None;
            let novel: Vec<_> = receipts
                .into_iter()
                .filter(|receipt| {
                    !work
                        .origin
                        .unknown_receipts()
                        .any(|existing| existing == receipt)
                })
                .collect();
            if novel.is_empty() {
                return Ok(false);
            }
            work.change = Some(
                work.origin
                    .apply(SubmissionCommand::ResolveUnknown(
                        UnknownResolutionAction::Verified {
                            receipts: novel,
                            at: clock.now(),
                        },
                    ))
                    .map_err(SubmissionControlError::Refused)?,
            );
        }
        let change = work
            .change
            .as_ref()
            .ok_or(SubmissionControlError::Invalid)?;
        if let Err(error) = SubmissionJobRepository::save(store, change) {
            self.work = Some(work);
            return Err(SubmissionControlError::Repository(error));
        }
        queue.update(change.state().clone());
        Ok(true)
    }
}
