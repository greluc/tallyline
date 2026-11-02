// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Filtered dependency diagnostics before credential argument formatting.

/// Bridges ordinary dependency diagnostics while excluding credential targets.
pub struct CredentialLogBridge;
impl log::Log for CredentialLogBridge {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        !credential_target(metadata.target())
    }
    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        match record.level() {
            log::Level::Error => {
                tracing::error!(target: "tallyline_dependency", source = record.target(), message = %record.args())
            }
            log::Level::Warn => {
                tracing::warn!(target: "tallyline_dependency", source = record.target(), message = %record.args())
            }
            log::Level::Info => {
                tracing::info!(target: "tallyline_dependency", source = record.target(), message = %record.args())
            }
            log::Level::Debug => {
                tracing::debug!(target: "tallyline_dependency", source = record.target(), message = %record.args())
            }
            log::Level::Trace => {
                tracing::trace!(target: "tallyline_dependency", source = record.target(), message = %record.args())
            }
        }
    }
    fn flush(&self) {}
}
fn credential_target(target: &str) -> bool {
    let root = target.split("::").next().unwrap_or(target);
    matches!(
        root,
        "keyring_core"
            | "keyring-core"
            | "windows_native_keyring_store"
            | "windows-native-keyring-store"
            | "secret_service"
            | "secret-service"
            | "zbus"
            | "zvariant"
            | "zbus_names"
            | "zvariant_utils"
    )
}
/// Installs the single filtered bridge before native providers or workers start.
///
/// # Errors
/// Returns a closed failure if another logger has already been installed.
pub fn install_credential_log_bridge() -> std::io::Result<()> {
    log::set_logger(&CredentialLogBridge)
        .map_err(|_| std::io::Error::other("dependency logging is already configured"))?;
    log::set_max_level(log::LevelFilter::Trace);
    Ok(())
}
