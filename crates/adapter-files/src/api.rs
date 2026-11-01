// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Filesystem adapter operations.

pub mod model;

pub use crate::config::ConfigFile;
pub use crate::game_log::GameLogFiles;
pub use crate::layout::StorageLayout;
pub use crate::logo::decode_logo;

pub use crate::backup::BackupSetStore;
pub use crate::image_intake::read_image;
pub use crate::image_intake::read_image_file;
