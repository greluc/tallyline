// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The asynchronous typed reference-data port implemented by the HTTP leaf.

use crate::api::{ReadError, UexReader};
use tallyline_reference_data::api::ReferenceDataFetcher;
use tallyline_reference_data::api::model::{Endpoint, EndpointData, GameVersions};

impl ReferenceDataFetcher for UexReader {
    type Error = ReadError;

    async fn fetch(&self, endpoint: Endpoint) -> Result<EndpointData, ReadError> {
        match endpoint {
            Endpoint::Commodities => {
                let data = self.transport.get("commodities/", &[], &[]).await?;
                crate::reference_wire::commodities(data).map(EndpointData::Commodities)
            }
            Endpoint::Terminals => self.terminals().await.map(EndpointData::Terminals),
            Endpoint::GameVersions => {
                let versions = self.game_versions().await?;
                Ok(EndpointData::GameVersions(GameVersions {
                    live: versions.live,
                    ptu: versions.ptu,
                }))
            }
        }
    }
}
