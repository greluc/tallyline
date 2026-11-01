// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Installed and portable state locations.

use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Separate locations for configuration, durable data, logs and regenerable files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StorageLayout {
    config: PathBuf,
    data: PathBuf,
    logs: PathBuf,
    cache: PathBuf,
}

/// A failed directory probe or an overly broad Unix permission.
#[derive(Debug)]
pub struct DirectoryIssue {
    /// The directory that needs attention.
    pub path: PathBuf,
    /// The underlying filesystem or permissions failure.
    pub error: io::Error,
}

impl StorageLayout {
    /// Resolves installed Windows state under the supplied profile directories.
    #[must_use]
    pub fn installed_windows(roaming: &Path, local: &Path) -> Self {
        Self {
            config: roaming.join("Tallyline"),
            data: local.join("Tallyline/data"),
            logs: local.join("Tallyline/logs"),
            cache: local.join("Tallyline/cache"),
        }
    }

    /// Resolves installed Linux state under the supplied XDG directories.
    #[must_use]
    pub fn installed_linux(config: &Path, data: &Path, state: &Path, cache: &Path) -> Self {
        Self {
            config: config.join("tallyline"),
            data: data.join("tallyline"),
            logs: state.join("tallyline/logs"),
            cache: cache.join("tallyline"),
        }
    }
    /// Resolves the portable layout below the executable's data directory.
    #[must_use]
    pub fn portable(executable_directory: &Path) -> Self {
        let root = executable_directory.join("data");
        Self {
            config: root.join("config"),
            data: root.join("data"),
            logs: root.join("logs"),
            cache: root.join("cache"),
        }
    }

    /// Returns the configuration directory.
    #[must_use]
    pub fn config(&self) -> &Path {
        &self.config
    }

    /// Returns the durable-data directory.
    #[must_use]
    pub fn data(&self) -> &Path {
        &self.data
    }

    /// Returns the log directory.
    #[must_use]
    pub fn logs(&self) -> &Path {
        &self.logs
    }

    /// Returns the regenerable-cache directory.
    #[must_use]
    pub fn cache(&self) -> &Path {
        &self.cache
    }

    /// Creates and probes every directory without falling back to another location.
    #[must_use]
    pub fn self_test(&self) -> Vec<DirectoryIssue> {
        [&self.config, &self.data, &self.logs, &self.cache]
            .into_iter()
            .filter_map(|path| {
                probe_directory(path).err().map(|error| DirectoryIssue {
                    path: path.clone(),
                    error,
                })
            })
            .collect()
    }
}

fn probe_directory(path: &Path) -> io::Result<()> {
    let mut builder = DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        builder.mode(0o700);
        builder.create(path)?;
        if fs::metadata(path)?.permissions().mode() & 0o077 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "directory is accessible to another user",
            ));
        }
    }
    #[cfg(not(unix))]
    builder.create(path)?;
    let probe = path.join(".tallyline-write-test");
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&probe)?;
    let result = file
        .write_all(b"Tallyline storage self-test")
        .and_then(|()| file.sync_all());
    drop(file);
    fs::remove_file(probe)?;
    result
}
