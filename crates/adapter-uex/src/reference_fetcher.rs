// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The asynchronous typed reference-data port implemented by the HTTP leaf.

use crate::api::{ReadError, UexReader};
use tallyline_reference_data::api::ReferenceDataFetcher;
use tallyline_reference_data::api::model::{
    Endpoint, EndpointData, GameVersions, StatusLevel, StatusLevels,
};

impl ReferenceDataFetcher for UexReader {
    type Error = ReadError;

    async fn fetch(&self, endpoint: Endpoint) -> Result<EndpointData, ReadError> {
        match endpoint {
            Endpoint::Parameters => {
                let data = self.transport.get("data_parameters/", &[], &[]).await?;
                crate::parameter_wire::parameters(data).map(EndpointData::Parameters)
            }
            Endpoint::Commodities => {
                let data = self.transport.get("commodities/", &[], &[]).await?;
                crate::reference_wire::commodities(data).map(EndpointData::Commodities)
            }
            Endpoint::Terminals => self.terminals().await.map(EndpointData::Terminals),
            Endpoint::StatusLevels => {
                let levels = self.commodity_status_levels().await?;
                Ok(EndpointData::StatusLevels(StatusLevels {
                    buy: mapped_status_levels(levels.buy)?,
                    sell: mapped_status_levels(levels.sell)?,
                }))
            }
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

fn status_level(level: crate::api::model::StatusLevel) -> Result<StatusLevel, ReadError> {
    Ok(StatusLevel {
        code: std::num::NonZeroU8::new(level.code).ok_or_else(|| ReadError::Malformed {
            detail: "status level code is zero".to_owned(),
        })?,
        name: level.name,
        short_name: level.name_short,
        abbreviation: level.name_abbr,
    })
}

fn mapped_status_levels(
    levels: Vec<crate::api::model::StatusLevel>,
) -> Result<Vec<StatusLevel>, ReadError> {
    let mut codes = std::collections::BTreeSet::new();
    levels
        .into_iter()
        .map(|level| {
            if !codes.insert(level.code) {
                return Err(ReadError::Malformed {
                    detail: "status vocabulary repeats a code within one side".to_owned(),
                });
            }
            status_level(level)
        })
        .collect()
}
