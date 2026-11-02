// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Primary-host reference reader composed after coordinated storage bootstrap.

use std::io;
use std::sync::mpsc::{Receiver, SyncSender};
use tallyline_adapter_storage::api::SqliteReferenceCache;
use tallyline_adapter_uex::api::{BuildError, ClientIdentity, ReaderSettings, UexReader};
use tallyline_ui::api::model::{BackgroundEvent, ReferenceCommand};

const PRIMARY_API: &str = "https://api.uexcorp.uk/2.0/";

pub(crate) fn start(
    cache: SqliteReferenceCache,
    incoming: Receiver<ReferenceCommand>,
    outgoing: SyncSender<BackgroundEvent>,
    context: egui::Context,
) -> io::Result<tallyline_app::api::ReferenceWorker> {
    tallyline_app::api::ReferenceWorker::start(
        cache,
        crate::update_wiring::SystemClock,
        incoming,
        outgoing,
        reader,
        move || tallyline_ui::api::wake_up(&context),
    )
}
fn reader() -> Result<UexReader, BuildError> {
    let information = tallyline_app::api::compiled_build_information()
        .map_err(|_| BuildError::Client("compiled build information is invalid".to_owned()))?;
    let settings = ReaderSettings::new(
        PRIMARY_API,
        ClientIdentity::new("tallyline", information.version()),
    )
    .map_err(|_| BuildError::Client("primary UEX URL is invalid".to_owned()))?;
    UexReader::new(&settings)
}
