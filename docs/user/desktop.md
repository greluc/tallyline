# Desktop shell

> **Doc type:** Living reference — M1 foundation. Last reviewed: 2026-11-01.

Tallyline starts with the onboarding welcome screen and its coupled Star Citizen Fan Kit unit. Choose **Continue** to acknowledge it and open the workspace. The workspace shows its empty capture state and keeps Settings and Help available. That acknowledgement and the selected theme survive a restart; subsequent starts open the workspace. **Done** or **Home** leaves Settings. If Settings was opened before the start-screen acknowledgement, Done returns to onboarding. This foundation provides the shell; capture, account setup and submission arrive through the remaining M1 work.

Choose **Settings** to select Follow OS (the default), Dark, Light or High contrast. Follow OS reacts to toolkit light/dark theme events; when no system value is available it uses Dark. An explicit choice overrides those events. Automatic Windows high-contrast detection remains part of the M4 platform acceptance checks; High contrast can be selected manually here. A selection is applied after it has been saved. **Help → About** shows the copyright, the fixed development package version and platform, warranty and redistribution notice, full GPL text, source links and Fan Kit unit. **Open-source licences** reads the generated list bundled with the application and makes no network request. A missing or unreadable list shows an error message and does not prevent startup. A source link opens the system browser only after it is clicked.

The minimum window size is 800 × 640 logical pixels. Both Fan Kit notices use 14 egui points, above the 10 pt floor (13.33 egui points at 100 % scale). Their stored strings have no inserted line break; the layout can flow them over several lines. The logo is not tinted or modified. The existing light/dark asset mapping follows the README; the owner's kit check remains O-27.

## Running a source build

Run `cargo build --locked -p tallyline-app`, followed by `target/debug/tallyline --assets crates/ui/assets` (Windows: `target/debug/tallyline.exe --assets crates/ui/assets`). The `--assets` argument selects the asset directory for a source build. Packaged images keep the same `fonts/` subtree under `assets/` next to the executable; font files and their OFL texts stay separate and unmodified.

For a local image with its complete offline licence catalogue, run `cargo xtask notices` (network access is required by this explicit CI workflow, O-135), then copy `target/notices/<target>/legal/` beside the executable. The supported target names are `x86_64-pc-windows-msvc` and `x86_64-unknown-linux-gnu`.

## State locations

| State | Windows installed | Linux installed |
|---|---|---|
| `settings.json` | `%APPDATA%\Tallyline` | `$XDG_CONFIG_HOME/tallyline`, default `~/.config/tallyline` |
| Durable data | `%LOCALAPPDATA%\Tallyline\data` | `$XDG_DATA_HOME/tallyline`, default `~/.local/share/tallyline` |
| Logs | `%LOCALAPPDATA%\Tallyline\logs` | `$XDG_STATE_HOME/tallyline/logs`, default `~/.local/state/tallyline/logs` |
| Regenerable cache | `%LOCALAPPDATA%\Tallyline\cache` | `$XDG_CACHE_HOME/tallyline`, default `~/.cache/tallyline` |

The shell currently persists only application preferences. The durable-data, log and cache directories are created and tested for the later workflows; no database is placed in the cache.

For portable mode, place `portable.marker` next to the executable. All state then lives below that executable's `data/{config,data,logs,cache}` directory. Installed builds write no state into their installation directory. A failed directory self-test stays visible and never falls back to the installation directory. New Linux state directories use mode 0700; a directory accessible to another user produces a warning.

## Settings errors

The settings file has schema version 1. Writes preserve other contexts' sections and replace the prior file atomically after syncing a temporary file in the same directory. A rejected theme or onboarding value is replaced with its default and reported with its key, rejected value and default. An unsupported schema is never overwritten. A failed save keeps the active settings unchanged. Malformed JSON and filesystem errors stay visible; correct or move the affected file before retrying.

Background results update the status area and persistent notices. They open no dialog and do not raise, restore or focus the window. The shell repaints on input and channel results; it uses static progress text and no spinner or continuous animation. It does not inspect game memory, inject input or create an overlay.

For example, a persisted onboarding value `"true"` is a string, not a Boolean.
The visible issue identifies `settings.onboarding_complete`, the rejected
`"true"`, and default `false`; unrelated account fields are not included.

Where the project has explicitly elected one dual-licence option, its licence
row shows that selected SPDX expression. The bundled catalogue separately
retains the upstream declaration and every complete original legal text.
