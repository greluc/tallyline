// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Application bootstrap operations.

pub mod model;

pub use crate::storage_bootstrap::initialise_storage;

pub use crate::attachment_worker::{AttachmentWorker, AttachmentWorkerError};
pub use crate::cpu_worker::CpuWorker;
pub use crate::manual_worker::ManualWorker;
pub use crate::reference_worker::ReferenceWorker;

pub use crate::manual_attachments::ManualAttachmentStorage;
