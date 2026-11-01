// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Atomic reference cache projection and mapped JSON persistence.

use crate::api::StorageError;
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tallyline_reference_data::api::ReferenceDataCache;
use tallyline_reference_data::api::model::{
    CachedReference, Commodity, EndpointData, Terminal, TerminalText, UexEnvironment,
};
use tallyline_shared_kernel::api::model::{CommodityId, TerminalId, Timestamp};

const SCHEMA_VERSION: u32 = 1;

const MIGRATION:&str="CREATE TABLE ref_cache(endpoint TEXT PRIMARY KEY, fetched_at TEXT NOT NULL, payload TEXT NOT NULL);
CREATE TABLE ref_commodities(id INTEGER PRIMARY KEY, name TEXT NOT NULL, code TEXT NOT NULL, available_live INTEGER NOT NULL);
CREATE INDEX ref_commodities_name ON ref_commodities(name);
CREATE INDEX ref_commodities_code ON ref_commodities(code);
PRAGMA user_version=1;";

/// Owns the reference cache connection on its dedicated storage worker.
pub struct SqliteReferenceCache {
    connection: Connection,
}

impl SqliteReferenceCache {
    /// Opens the application database and applies forward migrations.
    ///
    /// # Errors
    /// Returns an inaccessible database, unknown schema or failed migration.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref();
        if path.try_exists().map_err(StorageError::Filesystem)? {
            let readonly = Connection::open_with_flags(
                path,
                OpenFlags::SQLITE_OPEN_READ_ONLY
                    | OpenFlags::SQLITE_OPEN_NO_MUTEX
                    | OpenFlags::SQLITE_OPEN_NOFOLLOW,
            )?;
            let found: u32 = readonly.pragma_query_one(None, "user_version")?;
            if found > SCHEMA_VERSION {
                return Err(StorageError::UnsupportedSchema {
                    found,
                    supported: SCHEMA_VERSION,
                });
            }
        }
        let mut connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        let version: u32 = connection.pragma_query_one(None, "user_version")?;
        match version {
            0 => {
                let transaction = connection.transaction()?;
                transaction.execute_batch(MIGRATION)?;
                transaction.commit()?;
            }
            SCHEMA_VERSION => {}
            found => {
                return Err(StorageError::UnsupportedSchema {
                    found,
                    supported: SCHEMA_VERSION,
                });
            }
        }
        Ok(Self { connection })
    }
}

#[derive(Serialize, Deserialize)]
struct CommodityDocument {
    id: u32,
    name: String,
    code: String,
    available_live: bool,
    average_price: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct TerminalDocument {
    id: u32,
    name: String,
    nickname: String,
    display_name: String,
    location: String,
    available_live: bool,
}

fn timestamp(fetched_at: &str) -> Result<Timestamp, StorageError> {
    let millis = fetched_at
        .parse::<u128>()
        .map_err(|_| StorageError::InvalidCache)?;
    if let Ok(millis) = u64::try_from(millis) {
        Ok(Timestamp::from_millis(millis))
    } else if millis.is_multiple_of(1_000) {
        Ok(Timestamp::from_seconds(
            u64::try_from(millis / 1_000).map_err(|_| StorageError::InvalidCache)?,
        ))
    } else {
        Err(StorageError::InvalidCache)
    }
}

fn decode(endpoint: &str, payload: &str) -> Result<EndpointData, StorageError> {
    match endpoint {
        "commodities" => {
            let documents: Vec<CommodityDocument> =
                serde_json::from_str(payload).map_err(|_| StorageError::InvalidCache)?;
            Ok(EndpointData::Commodities(
                documents
                    .into_iter()
                    .map(|document| {
                        Commodity::new(
                            CommodityId(document.id),
                            &document.name,
                            &document.code,
                            document.available_live,
                            document.average_price,
                        )
                    })
                    .collect(),
            ))
        }
        "terminals" => {
            let documents: Vec<TerminalDocument> =
                serde_json::from_str(payload).map_err(|_| StorageError::InvalidCache)?;
            Ok(EndpointData::Terminals(
                documents
                    .into_iter()
                    .map(|document| {
                        Terminal::new(
                            TerminalId(document.id),
                            TerminalText {
                                name: document.name,
                                nickname: document.nickname,
                                display_name: document.display_name,
                                location: document.location,
                            },
                            document.available_live,
                        )
                    })
                    .collect(),
            ))
        }
        _ => Err(StorageError::InvalidCache),
    }
}

fn encode(data: &EndpointData) -> Result<(&'static str, String), StorageError> {
    let (endpoint, payload) = match data {
        EndpointData::Commodities(commodities) => {
            let documents = commodities
                .iter()
                .map(|commodity| CommodityDocument {
                    id: commodity.id().0,
                    name: commodity.name().to_owned(),
                    code: commodity.code().to_owned(),
                    available_live: commodity.available(UexEnvironment::Live),
                    average_price: commodity.average_price(UexEnvironment::Live),
                })
                .collect::<Vec<_>>();
            ("commodities", serde_json::to_string(&documents))
        }
        EndpointData::Terminals(terminals) => {
            let documents = terminals
                .iter()
                .map(|terminal| TerminalDocument {
                    id: terminal.id().0,
                    name: terminal.text().name.clone(),
                    nickname: terminal.text().nickname.clone(),
                    display_name: terminal.text().display_name.clone(),
                    location: terminal.text().location.clone(),
                    available_live: terminal.available(UexEnvironment::Live),
                })
                .collect::<Vec<_>>();
            ("terminals", serde_json::to_string(&documents))
        }
    };
    Ok((endpoint, payload.map_err(|_| StorageError::InvalidCache)?))
}

impl ReferenceDataCache for SqliteReferenceCache {
    type Error = StorageError;

    fn load(&mut self) -> Result<Vec<CachedReference>, Self::Error> {
        let mut entries = Vec::new();
        for endpoint in ["commodities", "terminals"] {
            let cached = self
                .connection
                .query_row(
                    "SELECT fetched_at,payload FROM ref_cache WHERE endpoint=?1",
                    [endpoint],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                )
                .optional()?;
            if let Some((fetched_at, payload)) = cached {
                entries.push(CachedReference::new(
                    timestamp(&fetched_at)?,
                    decode(endpoint, &payload)?,
                ));
            }
        }
        Ok(entries)
    }

    fn replace(&mut self, entry: &CachedReference) -> Result<(), Self::Error> {
        let (endpoint, payload) = encode(entry.data())?;
        let transaction = self.connection.transaction()?;
        transaction.execute("INSERT INTO ref_cache(endpoint,fetched_at,payload) VALUES(?1,?2,?3) ON CONFLICT(endpoint) DO UPDATE SET fetched_at=excluded.fetched_at,payload=excluded.payload",(endpoint,entry.fetched_at().millis().to_string(),payload))?;
        match entry.data() {
            EndpointData::Commodities(commodities) => {
                transaction.execute("DELETE FROM ref_commodities", [])?;
                for commodity in commodities {
                    transaction.execute("INSERT INTO ref_commodities(id,name,code,available_live) VALUES(?1,?2,?3,?4)",
                        (commodity.id().0,commodity.name(),commodity.code(),commodity.available(UexEnvironment::Live)))?;
                }
            }
            EndpointData::Terminals(_) => {}
        }
        transaction.commit()?;
        Ok(())
    }
}
