# Desktop shell

> **Doc type:** Living reference — M1 foundation. Last reviewed: 2026-11-01.

Tallyline starts with the onboarding welcome screen and its coupled Star Citizen Fan Kit unit. Choose **Continue** to acknowledge it and open the workspace. The workspace shows its empty capture state and keeps Settings and Help available. That acknowledgement and the selected theme survive a restart; subsequent starts open the workspace. **Done** or **Home** leaves Settings. If Settings was opened before the start-screen acknowledgement, Done returns to onboarding. This foundation provides the shell; capture, account setup and submission arrive through the remaining M1 work.

Choose **Settings** to select Follow OS (the default), Dark, Light or High contrast. Follow OS reacts to toolkit light/dark theme events; when no system value is available it uses Dark. An explicit choice overrides those events. Automatic Windows high-contrast detection remains part of the M4 platform acceptance checks; High contrast can be selected manually here. A selection is applied after it has been saved. **Help → About** shows the copyright, the fixed development package version and platform, warranty and redistribution notice, full GPL text, source links and Fan Kit unit. **Open-source licences** reads the generated list bundled with the application and makes no network request. A missing or unreadable list shows an error message and does not prevent startup. A source link opens the system browser only after it is clicked.

The minimum window size is 800 × 640 logical pixels. Both Fan Kit notices use 14 egui points, above the 10 pt floor (13.33 egui points at 100 % scale). Their stored strings have no inserted line break; the layout can flow them over several lines. The logo is not tinted or modified. The existing light/dark asset mapping follows the README; the owner's kit check remains O-27.

## Recognition delivery under evaluation

Corrected 2026-11-01: the owner requires image recognition to stay entirely on this computer and the chosen initial installation profile to include its runtime and models. A separate Ollama or Python installation is not a product prerequisite. [The runtime and reader comparison](../local-vision-runtime-research.md) and [ADR-0012](../adr/0012-strictly-local-recognition-delivery.md) record this direction; the desktop does not yet perform recognition. Additional OCR and local VLM combinations are being evaluated rather than advertised as proven improvements. The existing Visual C++ Redistributable prerequisite for the planned Windows OCR runtime remains a delivery obstacle to resolve before release.

## Running a source build

Keep the committed lockfile when building. The [dependency adoption record](../../supply-chain/latest-stable-adoption.json) records the current eligible stable versions, retained upstream constraints and native boundaries; complete third-party legal texts remain available offline.

Run `cargo build --locked -p tallyline-app`, followed by `target/debug/tallyline --assets crates/ui/assets` (Windows: `target/debug/tallyline.exe --assets crates/ui/assets`). The `--assets` argument selects the asset directory for a source build. Packaged images keep the same `fonts/` subtree under `assets/` next to the executable; font files and their OFL texts stay separate and unmodified.

For a local image with its complete offline licence catalogue, run `cargo xtask notices` (network access is required by this explicit CI workflow, O-135), then copy `target/notices/<target>/legal/` beside the executable. The supported target names are `x86_64-pc-windows-msvc` and `x86_64-unknown-linux-gnu`.

## State locations

| State | Windows installed | Linux installed |
|---|---|---|
| `settings.json` | `%APPDATA%\Tallyline` | `$XDG_CONFIG_HOME/tallyline`, default `~/.config/tallyline` |
| Durable data | `%LOCALAPPDATA%\Tallyline\data` | `$XDG_DATA_HOME/tallyline`, default `~/.local/share/tallyline` |
| Logs | `%LOCALAPPDATA%\Tallyline\logs` | `$XDG_STATE_HOME/tallyline/logs`, default `~/.local/state/tallyline/logs` |
| Regenerable cache | `%LOCALAPPDATA%\Tallyline\cache` | `$XDG_CACHE_HOME/tallyline`, default `~/.cache/tallyline` |

The shell persists application preferences and the local game-channel selection and witness opt-in. Witness facts remain in memory; no raw game log or witness snapshot is written into application storage. The durable-data, log and cache directories are created and tested for the later workflows; no database is placed in the cache.

For portable mode, place `portable.marker` next to the executable. All state then lives below that executable's `data/{config,data,logs,cache}` directory. Installed builds write no state into their installation directory. A failed directory self-test stays visible and never falls back to the installation directory. New Linux state directories use mode 0700; a directory accessible to another user produces a warning.

## Settings errors

Corrected 2026-11-01: the settings file has schema version 3; version-one files load with the witness off, and version-one/two files load with release checks off. They migrate forwards during startup only after a successful backup of the exact prior settings. When no database exists, this is a config-only backup; otherwise a coordinated database backup is required. A failed backup leaves the old settings untouched. Backup retention keeps the newest three completed creations even when the clock moves backwards or the app restarts; it never removes the snapshot it has just created. Older builds refuse the newer schema. Writes preserve other contexts' sections and replace the prior file atomically after syncing a temporary file in the same directory. A rejected theme or onboarding value is replaced with its default and reported with its key, rejected value and default. An unsupported schema is never overwritten. An update-check time beyond the supported settings range is refused without replacing saved preferences. A failed save keeps the active settings unchanged. Malformed JSON and filesystem errors stay visible; correct or move the affected file before retrying.

Background results update the status area and persistent notices. They open no dialog and do not raise, restore or focus the window. The shell repaints on input and channel results; it uses static progress text and no spinner or continuous animation. It does not inspect game memory, inject input or create an overlay.

For example, a persisted onboarding value `"true"` is a string, not a Boolean.
The visible issue identifies `settings.onboarding_complete`, the rejected
`"true"`, and default `false`; unrelated account fields are not included.

Where the project has explicitly elected one dual-licence option, its licence
row shows that selected SPDX expression. The bundled catalogue separately
retains the upstream declaration and every complete original legal text.

## Optional local Game.log witness

Open **Settings → Game log witness**, enter the directory of the game channel containing `Game.log` and `build_manifest.id`, and choose **Use channel and switch witness off**. Then select **Enable read-only Game.log witness**. Access is off by default; selecting another channel revokes the old opt-in. **Refresh witness facts** reads the selected files once on the background settings worker. There is no watcher or network request.

The status explains an unselected channel, unreadable files, an unvalidated build or payload, or a size limit. Settings shows the major/minor version and latest 50 sanitised buy-side observations; unknown terminal or commodity IDs remain absent. An enabled source can have zero observations when no kiosk was opened. The validated initial scope is build `4.10.197.2878` with branch `sc-alpha-4.10.2`, three terminal pairs and four commodity aliases listed in [the validation record](../game-log-validation.md). Similar names and other builds do not extend that scope.

These facts prepare later recognition resolution; this shell does not resolve or submit reports from them. The log is never a Reader. Sell captures may eventually use location only; buy-side assortment and sizes are never sell-side evidence. No price, quantity or status from any log record is retained as a reported value. The witness only provides major/minor version evidence, never a UEX patch or certainty. Capture matching, the latest preceding load and the five-minute window belong to the later resolution flow.

Raw lines, player fields, session and endpoint data never cross the parser boundary. The selected directory stays in the local settings file and is omitted from witness debug/eval output. Neither Game.log nor witness observations belong in diagnostics. Future Scan persistence and the diagnostics exporter must preserve these exclusions; those workflows are not supplied by this source implementation.

For complete sanitised observations, explicitly run `cargo run -p ocr-eval --locked -- witness --channel <directory>`. Its JSON contains only safe state/reason codes, major/minor version, UTC Unix milliseconds, allowlisted IDs and buy-side container-size sets. The manifest is limited to 64 KiB, Game.log to 128 MiB and an individual line to 16 KiB. Exceeding a limit disables this inspection rather than truncating evidence.

## Starting the app again

Installed and portable launches share a fixed per-user instance lock and local activation socket. A second launch restores the existing window, including while it is minimised and no view is being drawn; Windows may flash its taskbar button instead. If the existing instance cannot acknowledge activation within two seconds, a localised already-running notice appears. Close the notice and switch to the existing window. The second launch never opens another writable application store.

The data lock is `instance.lock` in the selected data directory. Windows keeps the shared lock at `%LOCALAPPDATA%/Tallyline/instance.lock` and the socket at `%LOCALAPPDATA%/Tallyline/run/a`. Linux always holds `$XDG_STATE_HOME/tallyline/instance.lock`, with the normal `~/.local/state` fallback, even when a runtime directory is available. It additionally uses `$XDG_RUNTIME_DIR/tallyline.lock` and `$XDG_RUNTIME_DIR/tl-a` when that directory is absolute, owned by the user and 0700. Otherwise it uses `$XDG_STATE_HOME/tallyline/instance.lock` and the private `run/a` directory/socket there, with the normal `~/.local/state` fallback. Portable mode does not move these per-user locations. Launches share the normal per-user state root. Different runtime environments still exclude a second writer, but may show the already-running notice instead of activating a differently located endpoint. Independently overriding HOME or XDG_STATE_HOME is outside this discovery guarantee. A socket pathname that exceeds the OS limit is refused visibly.

Crash recovery releases OS locks automatically. The next lock holder removes the stale socket; no manual lock-file deletion is needed. Do not delete lock files while an instance is running.

## UEX proxy support under implementation

The UEX adapter uses proxy environment variables and Windows static Internet Settings through the system-proxy feature. Tallyline does not alter those settings. PAC/autodiscovery and per-protocol Windows ProxyServer lists are not supported by the current upstream path; full system-proxy acceptance remains pending.

Corrected 2026-11-01: the native reference worker now makes credential-free GET requests to the primary UEX host, `api.uexcorp.uk`. It publishes the local cache before refreshing and keeps browsing available when a request fails. The earlier statement that the desktop made no UEX requests described the shell before this worker was connected.

## Release updates

In **Settings → Release updates**, enable **Check for release updates daily** to check the public Tallyline GitHub Releases endpoint. It is off by default. Only the request and app name/version are sent; no UEX credentials, paths or observations are included. The preference and last attempted check persist across restarts. Failed attempts count towards the full 24-hour interval, and moving the clock backwards delays the next check. The app checks at startup when due, after opt-in, and while open once the interval elapses.

A newer standard release appears as a persistent notice with **Open release notes** and **Dismiss update notice**. Only clicking the link opens your system browser. Drafts and prereleases are never offered. The notice and unavailable status never open dialogs, focus or restore the window; core work remains available offline. Disable the setting to stop future checks and hide the notice. An already-started request can finish, but its result stays hidden while checks are disabled. There is no automatic download or installation. The validated available version survives restarts without another request. Dismissing the notice also persists, and the same release stays hidden on later checks; a different newer release can be offered. Installing that version or a newer one resolves its cached notice. Invalid saved version tags are discarded with a settings error and never open arbitrary links.

Source builds compare against the fixed development package version. The signed-release build identity is supplied separately by the release workflow; development builds make no claim to be signed releases.

Settings schema 4 retains the independent UEX environment choice for each game channel. Migration backs up the exact old settings before replacement and preserves release checks, saved notices and the optional Game.log witness. The mapping controls save before applying; changing the witness directory preserves these mappings.

If a local database already exists, settings migration backs it up consistently alongside the old settings in `data/backup`. A failed backup stops startup processing; it never falls back to a settings-only copy of an existing database.

Database schema 2 retains separate exact buy/sell commodity averages and known terminal container limits. Before an older database migrates, startup saves it together with the original settings. Old side-ambiguous averages are discarded; cached names and fetch times remain available. PTU never borrows LIVE averages.

Reference priors distinguish reported sell inventory from forecast demand and retain their source version and modification time. Each terminal has a separate 30-minute fetch lifetime; a failed refresh keeps its prior data. PTU has no LIVE prior defaults.

## Browsing cached reference data

Open **Reference data** to select LIVE, HOTFIX, PTU, EPTU or TECH-PREVIEW. The selected game channel uses its saved UEX mapping: LIVE and HOTFIX default to UEX LIVE; the other three default to UEX PTU. **Settings → UEX environment mapping** changes one channel at a time after a successful save. These choices do not enable the Game.log witness or read game files.

Search terminals by name, nickname, display name or location and commodities by name or code. Rows show the UEX ID to distinguish duplicate labels and codes. Selecting a terminal issues a background terminal-price request; selecting a commodity only changes the local selection. PTU browsing includes entries unavailable on LIVE. It never displays LIVE prices as PTU reference data.

The global status distinguishes **No reference data yet**, usable cached data and an offline cache. Each loaded catalogue endpoint shows **Age at last update**, the elapsed seconds from its last successful fetch to the worker's clock observation; missing endpoints show **Not loaded yet**. A reversed clock shows an unknown age rather than a fresh value. **Refresh reference data** queues a background refresh without clearing records, search, selection or keyboard focus. A failed refresh retains the last successful data and fetch times. Background results never switch pages, open dialogs or change the window state.

The browser loads the local SQLite cache before the first network response, then refreshes in the background through the primary UEX host. Reference requests need no credential. If the client cannot initialise or a request fails, cached records remain usable; an empty offline installation shows the explicit no-data state.

UEX staff notices appear in the global status area in every view, limited to 280 Unicode characters and displayed literally without interpreting markup or links. Paused submissions or Datacenter maintenance leave cached reference browsing available; the last successful parameters remain available offline.

A successful reference update clears a resolved reference-refresh delivery notice. It preserves an unrelated settings error until that settings operation succeeds. The reference page uses the same global status area without displaying a second copy.

## Manual commodity Drafts

Open **Manual Report**, then choose the environment, terminal and Buy/Sell side explicitly and select **Create Draft**. On first use no environment or side is selected. The environment of the most recently edited manual Draft is offered on later starts. Terminal search covers names, nicknames, screen display names and location/system text, including fuzzy character sequences; the separate location filter narrows results. Previously used terminals appear first. Shared names still show each terminal and location separately; select the intended result explicitly.

Known terminal commodities already loaded when the Draft is created appear immediately as empty rows. Reference data loaded later offers **Add commodity** actions for the remaining rows. The global vocabulary search adds an unexpected commodity explicitly. Added rows start with empty price, SCU and status inputs. UEX values stay beside those inputs, with their age from the server modification timestamp. **Take over UEX values explicitly** copies the displayed row only after that action; typed and taken-over fields have USER origin. Tab or Enter commits a field; enter a displayed status code with its number key. A zero SCU quantity is preserved as zero, while a traded price must be positive. Invalid numeric input stays visible for correction and is not committed. A pending save disables further field edits; a failure keeps the input and shows an error.

**Resume Draft** opens a saved Draft. Values are stored in the durable application database and available without credentials. Closing the application after a committed field preserves it for restart. Select another terminal, side or environment to change a Draft's context; user values are kept and references are selected again for that context. An environment change clears the old game-version assignment rather than silently relabelling the observation. The full shared deviation/confirmation validation, game-version resolution and release/submission actions remain in progress; this editor does not send a Report.

SQLite schema 3 adds manual Draft records to the shared reference database. Existing schema-one/two databases require a coordinated exact database/config backup before their forward migration. New databases are created at schema 3; older binaries refuse this newer database. The local Draft worker starts only after the startup coordinator completes and is joined before the writer lease ends. Native GUI acceptance has not been performed for this slice.
