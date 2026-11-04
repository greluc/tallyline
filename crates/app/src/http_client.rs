// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Private typed proxies whose HTTP work belongs to one long-lived driver.

use crate::api::model::HttpAdmissionFailure;
use crate::http_protocol::Command;
use tallyline_adapter_uex::api::ReadError;
use tallyline_adapter_uex::api::model::{PreparedUexSubmission, ServerTimeTracker};
use tallyline_reference_data::api::ReferenceDataFetcher;
use tallyline_reference_data::api::model::{Endpoint, EndpointData, TerminalPriors};
use tallyline_shared_kernel::api::model::TerminalId;
use tallyline_submission::api::model::{
    CredentialFailure, CredentialValues, PayloadHash, PreparationFailure, SubmissionRequest,
    SubmissionResult, ValidatedAccount,
};
use tallyline_submission::api::model::{
    JobPhase, SubmissionAccountCheck, SubmissionJob, SubmissionPauseReason, VerifiedUexReport,
};
use tallyline_submission::api::{CredentialValidator, SubmissionGateway};
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::oneshot::Receiver;
use tokio::sync::{mpsc, oneshot};

/// A bounded private handle without secret, body or query diagnostics.
#[derive(Clone)]
pub struct HttpClient {
    pub(crate) requests: mpsc::Sender<Command>,
    pub(crate) server_time: Option<ServerTimeTracker>,
}
impl ReferenceDataFetcher for HttpClient {
    type Error = ReadError;
    async fn fetch(&self, endpoint: Endpoint) -> Result<EndpointData, ReadError> {
        let (reply, result) = oneshot::channel();
        self.requests
            .send(Command::Fetch(endpoint, reply))
            .await
            .map_err(|_| unavailable())?;
        result.await.map_err(|_| unavailable())?
    }
    async fn fetch_terminal_priors(
        &self,
        terminals: &[TerminalId],
    ) -> Result<Vec<TerminalPriors>, ReadError> {
        let (reply, result) = oneshot::channel();
        self.requests
            .send(Command::Priors(terminals.to_vec(), reply))
            .await
            .map_err(|_| unavailable())?;
        result.await.map_err(|_| unavailable())?
    }
}
impl CredentialValidator for HttpClient {
    async fn validate(
        &self,
        values: &CredentialValues,
    ) -> Result<ValidatedAccount, CredentialFailure> {
        let (reply, result) = oneshot::channel();
        self.requests
            .send(Command::Validate(values.clone(), reply))
            .await
            .map_err(|_| CredentialFailure::Unavailable)?;
        result.await.map_err(|_| CredentialFailure::Unavailable)?
    }
}
impl SubmissionGateway for HttpClient {
    type Prepared = PreparedUexSubmission;
    fn prepare(
        &self,
        request: &SubmissionRequest,
    ) -> Result<(Self::Prepared, PayloadHash), PreparationFailure> {
        self.prepare_with_screenshot(request, None)
    }
    async fn submit(
        &self,
        prepared: Self::Prepared,
        values: &CredentialValues,
        release_token: Option<&str>,
    ) -> SubmissionResult {
        let (reply, result) = oneshot::channel();
        if self
            .requests
            .send(Command::Submit(
                prepared,
                values.clone(),
                release_token.map(str::to_owned),
                reply,
            ))
            .await
            .is_err()
        {
            return SubmissionResult::Unknown;
        }
        result.await.unwrap_or(SubmissionResult::Unknown)
    }
}
fn unavailable() -> ReadError {
    ReadError::Transport("shared HTTP driver is unavailable".to_owned())
}

impl HttpClient {
    /// Prepares the workflow-validated private crop on its bounded CPU worker.
    ///
    /// # Errors
    /// Refuses malformed or excessive screenshot data and unsupported payload values.
    pub fn prepare_with_screenshot(
        &self,
        request: &SubmissionRequest,
        screenshot: Option<&str>,
    ) -> Result<(PreparedUexSubmission, PayloadHash), PreparationFailure> {
        let observed = request.header().observed_at;
        let date_added = self
            .server_time
            .as_ref()
            .map_or(observed, |tracker| tracker.clamp(observed));
        PreparedUexSubmission::new_with_screenshot(request, date_added, screenshot)
    }
}

impl tallyline_submission::api::SubmissionReceiptVerifier for HttpClient {
    async fn verify_receipts(
        &self,
        job: &SubmissionJob,
        values: &CredentialValues,
        release_token: Option<&str>,
    ) -> Option<Vec<VerifiedUexReport>> {
        let (reply, result) = oneshot::channel();
        self.requests
            .send(Command::VerifyReceipts(
                job.clone(),
                values.clone(),
                release_token.map(str::to_owned),
                reply,
            ))
            .await
            .ok()?;
        result.await.ok().flatten()
    }
}

impl HttpClient {
    /// Admits one optional receipt read without waiting for I/O or exposing credentials in its reply.
    ///
    /// # Errors
    /// Refuses non-successful jobs, a full command queue or a stopped shared driver.
    pub fn begin_receipt_verification(
        &self,
        job: &SubmissionJob,
        values: &CredentialValues,
        release_token: Option<&str>,
    ) -> Result<crate::api::PendingReceiptVerification, crate::api::model::HttpAdmissionFailure>
    {
        use crate::api::model::HttpAdmissionFailure;
        use tallyline_submission::api::model::JobPhase;
        if job.phase() != JobPhase::Succeeded
            || !matches!(job.result(), Some(SubmissionResult::ProductionAccepted(_)))
        {
            return Err(HttpAdmissionFailure::InvalidJob);
        }
        let (reply, result) = oneshot::channel();
        self.requests
            .try_send(Command::VerifyReceipts(
                job.clone(),
                values.clone(),
                release_token.map(str::to_owned),
                reply,
            ))
            .map_err(|error| match error {
                mpsc::error::TrySendError::Full(_) => HttpAdmissionFailure::Busy,
                mpsc::error::TrySendError::Closed(_) => HttpAdmissionFailure::Stopped,
            })?;
        Ok(crate::receipt_lookup::PendingReceiptVerification::new(
            job.clone(),
            result,
        ))
    }
}

impl HttpClient {
    /// Requests the sole ready POST route before preparing or committing an attempt.
    /// This bounded command carries no credentials, body or mutable Report data.
    ///
    /// # Errors
    /// Refuses a full command queue or stopped driver without recording an attempt.
    pub fn begin_submission_reservation(
        &self,
    ) -> Result<crate::api::PendingSubmissionReservation, crate::api::model::HttpAdmissionFailure>
    {
        use crate::api::model::HttpAdmissionFailure;
        let (reply, result) = oneshot::channel();
        self.requests
            .try_send(Command::ReserveSubmission(reply))
            .map_err(|error| match error {
                mpsc::error::TrySendError::Full(_) => HttpAdmissionFailure::Busy,
                mpsc::error::TrySendError::Closed(_) => HttpAdmissionFailure::Stopped,
            })?;
        Ok(crate::submission_slot::PendingSubmissionReservation::new(
            result,
        ))
    }
}

impl HttpClient {
    pub(crate) fn begin_refusal_recheck(
        &self,
        job: &SubmissionJob,
        values: &CredentialValues,
        release_token: Option<&str>,
    ) -> Result<oneshot::Receiver<Option<SubmissionResult>>, crate::api::model::HttpAdmissionFailure>
    {
        use crate::api::model::HttpAdmissionFailure;
        let account = job
            .attempts()
            .last()
            .and_then(|attempt| attempt.account_evidence())
            .is_some_and(|evidence| {
                matches!(
                    evidence.check(),
                    SubmissionAccountCheck::Pending
                        | SubmissionAccountCheck::Completed(
                            SubmissionPauseReason::KeyCheckUnavailable
                        )
                )
            });
        let receipt = job.phase() == JobPhase::Held
            && matches!(
                job.result().map(SubmissionResult::final_result),
                Some(SubmissionResult::NeedsOutcomeCheck { .. })
            );
        if (!account && !receipt)
            || matches!(
                job.phase(),
                JobPhase::Queued | JobPhase::Sending | JobPhase::Succeeded
            )
        {
            return Err(HttpAdmissionFailure::InvalidJob);
        }
        let (reply, result) = oneshot::channel();
        self.requests
            .try_send(Command::RecheckRefusal(
                job.clone(),
                values.clone(),
                release_token.map(str::to_owned),
                reply,
            ))
            .map_err(|error| match error {
                mpsc::error::TrySendError::Full(_) => HttpAdmissionFailure::Busy,
                mpsc::error::TrySendError::Closed(_) => HttpAdmissionFailure::Stopped,
            })?;
        Ok(result)
    }
}

impl HttpClient {
    pub(crate) fn begin_unknown_receipt_verification(
        &self,
        job: &SubmissionJob,
        values: &CredentialValues,
        release_token: Option<&str>,
    ) -> Result<Receiver<Option<Vec<VerifiedUexReport>>>, HttpAdmissionFailure> {
        use crate::api::model::HttpAdmissionFailure;
        let (reply, result) = oneshot::channel();
        self.requests
            .try_send(Command::VerifyUnknownReceipts(
                job.clone(),
                values.clone(),
                release_token.map(str::to_owned),
                reply,
            ))
            .map_err(|error| match error {
                TrySendError::Full(_) => HttpAdmissionFailure::Busy,
                TrySendError::Closed(_) => HttpAdmissionFailure::Stopped,
            })?;
        Ok(result)
    }
}
