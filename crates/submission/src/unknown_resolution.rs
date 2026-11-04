// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Explicit resolution evidence retained beside an unknown attempt.

use crate::api::model::{JobPhase, QueueRefusal, SubmissionJob, VerifiedUexReport};
use std::collections::BTreeSet;
use tallyline_shared_kernel::api::model::{ReportId, Timestamp};

/// A user's statement about an uncertain submission, without remote proof.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnknownDecision {
    /// The user considers the submitted values received.
    Received,
    /// The user considers the submitted values not received.
    NotReceived,
}

/// An explicit resolution action without submission authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnknownResolutionAction {
    /// Records a user statement independently of the original attempt result.
    Decision {
        /// The user's statement.
        decision: UnknownDecision,
        /// The injected decision time.
        at: Timestamp,
    },
    /// Records positive remote identities verified for the exact attempted snapshot.
    Verified {
        /// Positively matched row identities, never inferred from absence.
        receipts: Vec<VerifiedUexReport>,
        /// The injected evidence time.
        at: Timestamp,
    },
    /// Links a separate Draft created by the explicit copying action.
    Duplicated {
        /// The fresh identity of the separate Draft.
        report: ReportId,
        /// The injected duplication time.
        at: Timestamp,
    },
}
impl UnknownResolutionAction {
    /// Returns the explicit action's recorded time.
    #[must_use]
    pub const fn at(&self) -> Timestamp {
        match self {
            Self::Decision { at, .. } | Self::Duplicated { at, .. } | Self::Verified { at, .. } => {
                *at
            }
        }
    }
}

impl SubmissionJob {
    /// Retains explicit evidence without changing the uncertain attempt.
    ///
    /// # Errors
    /// Refuses non-unknown jobs and conflicting evidence.
    pub fn resolve_unknown(
        &self,
        mut action: UnknownResolutionAction,
    ) -> Result<Self, QueueRefusal> {
        if self.phase() != JobPhase::OutcomeUnknown {
            return Err(QueueRefusal::InvalidTransition);
        }
        if self.unknown_actions.last() == Some(&action) {
            return Ok(self.clone());
        }
        match &action {
            UnknownResolutionAction::Decision { .. }
                if self.unknown_decision().is_none() && self.unknown_actions.is_empty() => {}
            UnknownResolutionAction::Verified { receipts, .. } => {
                self.validate_unknown_receipts(receipts)?
            }
            UnknownResolutionAction::Duplicated { report, .. }
                if self.can_duplicate_unknown()
                    && report.0 != 0
                    && *report != self.request().header().report => {}
            UnknownResolutionAction::Decision { .. }
            | UnknownResolutionAction::Duplicated { .. } => {
                return Err(QueueRefusal::InvalidTransition);
            }
        }
        let previous_at = self
            .unknown_actions
            .last()
            .map(UnknownResolutionAction::at)
            .or_else(|| {
                self.attempts()
                    .last()
                    .and_then(|attempt| attempt.ended_at())
            })
            .ok_or(QueueRefusal::InvalidTransition)?;
        if action.at() < previous_at {
            return Err(QueueRefusal::InvalidTransition);
        }
        if let UnknownResolutionAction::Verified { receipts, .. } = &mut action {
            receipts.retain(|receipt| !self.unknown_receipts().any(|existing| existing == receipt));
        }
        let mut next = self.advance()?;
        next.unknown_actions.push(action);
        Ok(next)
    }

    /// Borrows explicit evidence without granting sending or withdrawal authority.
    #[must_use]
    pub fn unknown_actions(&self) -> &[UnknownResolutionAction] {
        &self.unknown_actions
    }
}

impl SubmissionJob {
    /// Returns a user statement without treating it as remote receipt proof.
    #[must_use]
    pub fn unknown_decision(&self) -> Option<UnknownDecision> {
        self.unknown_actions.iter().find_map(|action| match action {
            UnknownResolutionAction::Decision { decision, .. } => Some(*decision),
            UnknownResolutionAction::Duplicated { .. }
            | UnknownResolutionAction::Verified { .. } => None,
        })
    }

    /// Reports eligibility for the separate explicit new-Draft action.
    #[must_use]
    pub fn can_duplicate_unknown(&self) -> bool {
        self.phase() == JobPhase::OutcomeUnknown
            && self.unknown_decision() == Some(UnknownDecision::NotReceived)
            && self.unknown_receipts().next().is_none()
            && !self
                .unknown_actions
                .iter()
                .any(|action| matches!(action, UnknownResolutionAction::Duplicated { .. }))
    }
}

impl SubmissionJob {
    /// Borrows positive row identities while retaining their returned dates.
    pub fn unknown_receipts(&self) -> impl Iterator<Item = &VerifiedUexReport> {
        self.unknown_actions.iter().flat_map(|action| match action {
            UnknownResolutionAction::Verified { receipts, .. } => receipts.as_slice(),
            UnknownResolutionAction::Decision { .. }
            | UnknownResolutionAction::Duplicated { .. } => &[],
        })
    }

    fn validate_unknown_receipts(
        &self,
        receipts: &[VerifiedUexReport],
    ) -> Result<(), QueueRefusal> {
        let sent_at = self
            .attempts()
            .last()
            .ok_or(QueueRefusal::InvalidTransition)?
            .date_added();
        let mut rows = BTreeSet::new();
        let mut identities = BTreeSet::new();
        let mut novel = false;
        for receipt in receipts {
            if !receipt.matches_at(self.request(), sent_at)
                || !rows.insert(receipt.row().0)
                || !identities.insert(receipt.id())
            {
                return Err(QueueRefusal::InvalidTransition);
            }
            for existing in self.unknown_receipts() {
                if (existing.row() == receipt.row() || existing.id() == receipt.id())
                    && existing != receipt
                {
                    return Err(QueueRefusal::InvalidTransition);
                }
            }
            novel |= !self.unknown_receipts().any(|existing| existing == receipt);
        }
        if !novel {
            return Err(QueueRefusal::InvalidTransition);
        }
        Ok(())
    }
}
