// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Atomic migration backup sets and recognised-set retention.

use crate::layout::StorageLayout;
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};
use tallyline_shared_kernel::api::{Clock, model::Timestamp};

const MANIFEST: &str = "manifest.json";
const CONFIG: &str = "settings.json";
const DATABASE: &str = "tallyline.db";
const RETAINED_SETS: usize = 3;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    taken_at_ms: u64,
    config_schema: Option<u32>,
    database_schema: Option<u32>,
    config_file: Option<String>,
    database_file: Option<String>,
}

/// Commits exact config bytes and an optional coordinated database backup as one set.
pub struct BackupSetStore {
    directory: PathBuf,
}

impl BackupSetStore {
    /// Selects the documented durable backup directory.
    #[must_use]
    pub const fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    /// Backs up config only after confirming that no application database exists.
    ///
    /// # Errors
    /// Refuses an existing database, unreadable location, clock overflow or failed backup.
    pub fn backup_without_database(
        &self,
        layout: &StorageLayout,
        schema: u32,
        bytes: &[u8],
        clock: &impl Clock,
    ) -> io::Result<PathBuf> {
        match fs::symlink_metadata(layout.data().join(DATABASE)) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => (),
            Err(error) => return Err(error),
            Ok(_) => {
                return Err(io::Error::other(
                    "database migration requires a coordinated SQLite backup",
                ));
            }
        }
        self.write(Some((schema, bytes)), clock, |_, _| Ok(None))
    }

    /// Writes a complete set before pruning only recognised completed sets to the last three.
    /// The callback must create the supplied database file through SQLite and return its schema.
    ///
    /// # Errors
    /// Returns clock, filesystem, callback or manifest errors; an incomplete set never replaces config.
    pub fn write(
        &self,
        config: Option<(u32, &[u8])>,
        clock: &impl Clock,
        database: impl FnOnce(&Path, Timestamp) -> io::Result<Option<u32>>,
    ) -> io::Result<PathBuf> {
        let taken_at = clock.now();
        let taken_at_ms = u64::try_from(taken_at.millis()).map_err(io::Error::other)?;
        fs::create_dir_all(&self.directory)?;
        let name = format!(
            "{taken_at_ms}-v{}",
            config.map_or_else(|| "none".to_owned(), |(schema, _)| schema.to_string())
        );
        let pending = self.directory.join(format!("{name}.pending"));
        let complete = self.directory.join(name);
        if complete.try_exists()? {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "backup timestamp already exists",
            ));
        }
        fs::create_dir(&pending)?;
        let result = (|| {
            let database_schema = database(&pending.join(DATABASE), taken_at)?;
            if database_schema.is_some() != pending.join(DATABASE).is_file() {
                return Err(io::Error::other(
                    "database backup callback returned an inconsistent set",
                ));
            }
            if let Some((_, bytes)) = config {
                write_synced(&pending.join(CONFIG), bytes)?;
            }
            let manifest = Manifest {
                schema_version: 1,
                taken_at_ms,
                config_schema: config.map(|(schema, _)| schema),
                database_schema,
                config_file: config.map(|_| CONFIG.to_owned()),
                database_file: database_schema.map(|_| DATABASE.to_owned()),
            };
            write_synced(
                &pending.join(MANIFEST),
                &serde_json::to_vec_pretty(&manifest)?,
            )?;
            fs::rename(&pending, &complete)
        })();
        if let Err(error) = result {
            fs::remove_dir_all(&pending)?;
            return Err(error);
        }
        self.prune()?;
        Ok(complete)
    }

    fn prune(&self) -> io::Result<()> {
        let mut sets = Vec::new();
        for entry in fs::read_dir(&self.directory)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            if let Some(time) = recognised_set(&entry.path())? {
                sets.push((time, entry.path()));
            }
        }
        sets.sort();
        let excess = sets.len().saturating_sub(RETAINED_SETS);
        for (_, directory) in sets.into_iter().take(excess) {
            fs::remove_dir_all(directory)?;
        }
        Ok(())
    }
}

fn write_synced(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn recognised_set(path: &Path) -> io::Result<Option<u64>> {
    let bytes = match fs::read(path.join(MANIFEST)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let Ok(manifest) = serde_json::from_slice::<Manifest>(&bytes) else {
        return Ok(None);
    };
    if manifest.schema_version != 1
        || manifest.config_file.as_deref() != manifest.config_schema.map(|_| CONFIG)
        || manifest.database_file.as_deref() != manifest.database_schema.map(|_| DATABASE)
    {
        return Ok(None);
    }
    let expected = format!(
        "{}-v{}",
        manifest.taken_at_ms,
        manifest
            .config_schema
            .map_or_else(|| "none".to_owned(), |schema| schema.to_string())
    );
    if path.file_name().and_then(|name| name.to_str()) != Some(&expected) {
        return Ok(None);
    }
    let expected_count = 1
        + usize::from(manifest.config_schema.is_some())
        + usize::from(manifest.database_schema.is_some());
    let entries = fs::read_dir(path)?.collect::<io::Result<Vec<_>>>()?;
    if entries.len() != expected_count {
        return Ok(None);
    }
    for entry in entries {
        if !entry.file_type()?.is_file()
            || ![
                Some(MANIFEST),
                manifest.config_file.as_deref(),
                manifest.database_file.as_deref(),
            ]
            .contains(&entry.file_name().to_str())
        {
            return Ok(None);
        }
    }
    Ok(Some(manifest.taken_at_ms))
}
