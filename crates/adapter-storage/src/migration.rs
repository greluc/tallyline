// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Forward cache migration within the caller's SQLite transaction.

use crate::api::StorageError;
use crate::reference_cache::{decode, encode};
use rusqlite::{Connection, OptionalExtension};
use tallyline_reference_data::api::model::{EndpointData, Terminal, UexEnvironment};

pub(crate) fn schema_two(connection: &Connection) -> Result<(), StorageError> {
    connection.execute_batch("CREATE TABLE ref_terminals(id INTEGER PRIMARY KEY,name TEXT NOT NULL,nickname TEXT NOT NULL,display_name TEXT NOT NULL,location TEXT NOT NULL,available_live INTEGER NOT NULL,max_container_size INTEGER CHECK(max_container_size IS NULL OR max_container_size>0)); CREATE INDEX ref_terminals_name ON ref_terminals(name); CREATE INDEX ref_terminals_nickname ON ref_terminals(nickname); CREATE INDEX ref_terminals_display_name ON ref_terminals(display_name); CREATE INDEX ref_terminals_location ON ref_terminals(location);")?;
    for endpoint in ["commodities", "terminals"] {
        let payload: Option<String> = connection
            .query_row(
                "SELECT payload FROM ref_cache WHERE endpoint=?1",
                [endpoint],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(payload) = payload {
            let mapped = decode(endpoint, &payload)?;
            if let EndpointData::Terminals(terminals) = &mapped {
                index_terminals(connection, terminals)?;
            }
            let (_, payload) = encode(&mapped)?;
            connection.execute(
                "UPDATE ref_cache SET payload=?1 WHERE endpoint=?2",
                (&payload, endpoint),
            )?;
        }
    }
    connection.pragma_update(None, "user_version", 2)?;
    Ok(())
}

pub(crate) fn index_terminals(
    connection: &Connection,
    terminals: &[Terminal],
) -> Result<(), StorageError> {
    connection.execute("DELETE FROM ref_terminals", [])?;
    for terminal in terminals {
        let text = terminal.text();
        connection.execute("INSERT INTO ref_terminals(id,name,nickname,display_name,location,available_live,max_container_size) VALUES(?1,?2,?3,?4,?5,?6,?7)",(terminal.id().0,&text.name,&text.nickname,&text.display_name,&text.location,terminal.available(UexEnvironment::Live),terminal.max_container_size().map(std::num::NonZeroU32::get)))?;
    }
    Ok(())
}

pub(crate) fn schema_three(connection: &Connection) -> Result<(), StorageError> {
    connection.execute_batch("CREATE TABLE reports(id INTEGER PRIMARY KEY AUTOINCREMENT,version INTEGER NOT NULL CHECK(version>=0),payload TEXT NOT NULL); CREATE TABLE report_recents(report_id INTEGER PRIMARY KEY REFERENCES reports(id),changed_order INTEGER NOT NULL); PRAGMA user_version=3;")?;
    Ok(())
}

pub(crate) fn schema_four(connection: &Connection) -> Result<(), StorageError> {
    let mut statement = connection.prepare("SELECT id,payload FROM reports")?;
    let records = statement
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (id, payload) in records {
        let mut document: serde_json::Value =
            serde_json::from_str(&payload).map_err(|_| StorageError::InvalidCache)?;
        let rows = document
            .get_mut("rows")
            .and_then(serde_json::Value::as_array_mut)
            .ok_or(StorageError::InvalidCache)?;
        for row in rows {
            let row = row.as_object_mut().ok_or(StorageError::InvalidCache)?;
            row.insert("status_confirmation".to_owned(), serde_json::Value::Null);
            row.insert("status_deviation".to_owned(), serde_json::Value::Null);
            row.insert(
                "status_confirmation_revoked".to_owned(),
                serde_json::Value::Bool(false),
            );
            row.insert(
                "status_unreviewed".to_owned(),
                serde_json::Value::Bool(false),
            );
        }
        connection.execute(
            "UPDATE reports SET payload=?1 WHERE id=?2",
            (document.to_string(), id),
        )?;
    }
    connection.pragma_update(None, "user_version", 4)?;
    Ok(())
}

pub(crate) fn schema_five(connection: &Connection) -> Result<(), StorageError> {
    let mut statement = connection.prepare("SELECT id,payload FROM reports")?;
    let records = statement
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (id, payload) in records {
        let mut document: serde_json::Value =
            serde_json::from_str(&payload).map_err(|_| StorageError::InvalidCache)?;
        document
            .as_object_mut()
            .ok_or(StorageError::InvalidCache)?
            .insert("attachment".to_owned(), serde_json::Value::Null);
        connection.execute(
            "UPDATE reports SET payload=?1 WHERE id=?2",
            (document.to_string(), id),
        )?;
    }
    connection.pragma_update(None, "user_version", 5)?;
    Ok(())
}
