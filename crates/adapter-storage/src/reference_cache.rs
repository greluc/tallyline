// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Atomic reference cache projection and mapped JSON persistence.

use crate::api::StorageError;
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tallyline_reference_data::api::ReferenceDataCache;
use tallyline_reference_data::api::model::{
    CachedReference, Commodity, EndpointData, UexEnvironment,
};
use tallyline_shared_kernel::api::model::{CommodityId, Timestamp};

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

impl ReferenceDataCache for SqliteReferenceCache {
    type Error = StorageError;

    fn load(&mut self) -> Result<Vec<CachedReference>, Self::Error> {
        let cached = self
            .connection
            .query_row(
                "SELECT fetched_at,payload FROM ref_cache WHERE endpoint=?1",
                ["commodities"],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;
        let Some((fetched_at, payload)) = cached else {
            return Ok(Vec::new());
        };
        let millis = fetched_at
            .parse::<u128>()
            .map_err(|_| StorageError::InvalidCache)?;
        let fetched_at = if let Ok(millis) = u64::try_from(millis) {
            Timestamp::from_millis(millis)
        } else if millis.is_multiple_of(1_000) {
            Timestamp::from_seconds(
                u64::try_from(millis / 1_000).map_err(|_| StorageError::InvalidCache)?,
            )
        } else {
            return Err(StorageError::InvalidCache);
        };
        let documents: Vec<CommodityDocument> =
            serde_json::from_str(&payload).map_err(|_| StorageError::InvalidCache)?;
        let commodities = documents
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
            .collect();
        Ok(vec![CachedReference::new(
            fetched_at,
            EndpointData::Commodities(commodities),
        )])
    }

    fn replace(&mut self, entry: &CachedReference) -> Result<(), Self::Error> {
        let EndpointData::Commodities(commodities) = entry.data();
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
        let payload = serde_json::to_string(&documents).map_err(|_| StorageError::InvalidCache)?;
        let transaction = self.connection.transaction()?;
        transaction.execute("INSERT INTO ref_cache(endpoint,fetched_at,payload) VALUES(?1,?2,?3) ON CONFLICT(endpoint) DO UPDATE SET fetched_at=excluded.fetched_at,payload=excluded.payload",("commodities",entry.fetched_at().millis().to_string(),payload))?;
        transaction.execute("DELETE FROM ref_commodities", [])?;
        for commodity in commodities {
            transaction.execute(
                "INSERT INTO ref_commodities(id,name,code,available_live) VALUES(?1,?2,?3,?4)",
                (
                    commodity.id().0,
                    commodity.name(),
                    commodity.code(),
                    commodity.available(UexEnvironment::Live),
                ),
            )?;
        }
        transaction.commit()?;
        Ok(())
    }
}
