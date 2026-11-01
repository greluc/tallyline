// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Commands and results exchanged with the desktop shell.

pub use crate::fan_kit::{FAN_KIT_LOGO_ALT_TEXT, FAN_KIT_LOGO_TEXTURE_NAME, LegalPlacement};
pub use crate::state::{BackgroundEvent, SettingsCommand, UserAction, View};

pub use crate::activation::WindowCommand;
pub use crate::fonts::FontFiles;
pub use crate::legal_views::AboutAction;
pub use crate::licences::CopyrightInformation;
pub use crate::licences::{
    LicenceCatalogue, LicenceCatalogueError, LicenceComponent, LicenceNotice,
};
