# Desktop shell

> **Doc type:** Living reference — M1 foundation. Last reviewed: 2026-11-03.

Tallyline starts with the onboarding welcome screen and its coupled Star Citizen Fan Kit unit. Choose **Continue** to acknowledge it and open the workspace. The workspace shows its empty capture state and keeps Settings and Help available. That acknowledgement and the selected theme survive a restart; subsequent starts open the workspace. **Done** or **Home** leaves Settings. If Settings was opened before the start-screen acknowledgement, Done returns to onboarding. The shell includes reviewed manual submission; capture and OCR workflows remain in progress. Account setup is available below the welcome acknowledgement and in Settings.

Choose **Settings** to select Follow OS (the default), Dark, Light or High contrast. Follow OS reacts to toolkit light/dark theme events; when no system value is available it uses Dark. An explicit choice overrides those events. Automatic Windows high-contrast detection remains part of the M4 platform acceptance checks; High contrast can be selected manually here. A selection is applied after it has been saved. **Help → About** shows the copyright, the compiled version and platform (Development `0.0.0` for local builds), warranty and redistribution notice, full GPL text, source links and Fan Kit unit. **Open-source licences** reads the generated list bundled with the application and makes no network request. A missing or unreadable list shows an error message and does not prevent startup. A source link opens the system browser only after it is clicked.

The minimum window size is 800 × 640 logical pixels. Both Fan Kit notices use 14 egui points, above the 10 pt floor (13.33 egui points at 100 % scale). Their stored strings have no inserted line break; the layout can flow them over several lines. The logo is not tinted or modified. The existing light/dark asset mapping follows the README; the owner's kit check remains O-27.

For source builds with `CARGO_TARGET_DIR` set, find the executable under that chosen build directory instead of the default `target/`. The [source-build instructions](../../README.md#run-the-desktop-shell-from-source) link the corresponding check and scratch-output commands.

## UEX account setup

Enter your secret key in the masked **UEX account** form during onboarding, or expand **Settings → UEX account**. **Continue** remains available for local browsing and Draft preparation without credentials. The packaged Tallyline app token is never displayed; if its file is absent, empty, malformed or unreadable, the form asks for your own token or a current release. **Expert: use my own app token** overrides a packaged token. A refused built-in token offers the same update/own-token actions.

**Validate credentials** makes only read-only account and token checks. The accepted username appears only after validation and the selected save succeed. A failed replacement leaves the previously accepted account intact. **Keep credentials for this session only** avoids durable saving and requires entry again after exit. Saved values are never filled back into the form. Startup revalidation preserves the last successful identity while offline and persists known account refusals without opening a provider prompt. Credential work runs off the UI thread; failures stay inline and never open a background dialog or take focus.

Windows uses the owned local-machine Credential Manager entry. Linux uses the authenticated local Secret Service. Only when that service is unavailable, and a private fallback location has passed ownership, permission and filesystem checks, does the form offer the plaintext warning and **I understand: use the protected local file**. Confirmation is required before reading or writing that file, including after a restart. A locked, malformed or uncertain OS-store result never selects the fallback. You can always choose session-only credentials instead.

The fallback file is `<durable-data>/credentials/uex-credentials`, below the durable-data directory in the table below. Its directory must remain owner-only 0700 and its regular single-link file owner-only 0600. The adapter refuses symlink traversal, changed ownership/access and unsupported filesystem types, including FAT/exFAT. It creates each same-directory temporary file exclusively with 0600 before writing, verifies the descriptor, synchronises it and atomically replaces the record. No credentials go into settings or SQLite. A failure after replacement may be uncertain; a successful load is required before another explicit durable save, with no automatic retry. Other programs running as your user can read a plaintext fallback; use only a trusted device.

The fallback accepts the checked ext-family, XFS, Btrfs, tmpfs and overlayfs profiles and verifies private creation on that actual location. Other filesystem profiles use session-only credentials. No config or database schema changes are introduced by account setup; the private credential bundle remains version one. Actual Credential Manager/Secret Service provider prompts, FAT/exFAT media, installed clean-OS and native visual checks remain pending. Optional local AI is unrelated to credential startup; no Ollama service is required.

## Recognition delivery under evaluation

Corrected 2026-11-01: the owner requires image recognition to stay entirely on this computer and the chosen initial installation profile to include its runtime and models. A separate Ollama or Python installation is not a product prerequisite. [The runtime and reader comparison](../local-vision-runtime-research.md) and [ADR-0012](../adr/0012-strictly-local-recognition-delivery.md) record this direction; the desktop does not yet perform recognition. Additional OCR and local VLM combinations are being evaluated rather than advertised as proven improvements. The existing Visual C++ Redistributable prerequisite for the planned Windows OCR runtime remains a delivery obstacle to resolve before release.

## Running a source build

Keep the committed lockfile when building. The [dependency adoption record](../../supply-chain/latest-stable-adoption.json) and [image intake record](../../supply-chain/image-intake-adoption.json) document the reviewed stable selections, retained upstream constraints and native boundaries; complete third-party legal texts remain available offline.

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

For portable mode, place `portable.marker` next to the executable. All state then lives below that executable's `data/{config,data,logs,cache}` directory. Installed builds write no state into their installation directory. A failed read/write directory self-test stays visible, stops storage bootstrap and never falls back to the installation directory. Corrected 2026-11-02: successful read/write probes on group/world-accessible Linux directories produce a persistent warning and permit ordinary settings/data startup, as required by R-NF-5; existing permissions are never repaired silently. New Linux state directories still request mode 0700. Credential fallback permissions remain a separate strict refusal gate.

## Settings errors

Corrected 2026-11-02: the settings file has schema version 4; version-one files load with the witness off, and version-one/two files load with release checks off. They migrate forwards during startup only after a successful backup of the exact prior settings. When no database exists, this is a config-only backup; otherwise a coordinated database backup is required. A failed backup leaves the old settings untouched. Backup retention keeps the newest three completed creations even when the clock moves backwards or the app restarts; it never removes the snapshot it has just created. Older builds refuse the newer schema. Writes preserve other contexts' sections and replace the prior file atomically after syncing a temporary file in the same directory. A rejected theme or onboarding value is replaced with its default and reported with its key, rejected value and default. An unsupported schema is never overwritten. An update-check time beyond the supported settings range is refused without replacing saved preferences. A failed save keeps the active settings unchanged. Malformed JSON and filesystem errors stay visible; correct or move the affected file before retrying.

Background results update the status area and persistent notices. They open no dialog and do not raise, restore or focus the window. The shell repaints on input and channel results; it uses static progress text and no spinner or continuous animation. It does not inspect game memory, inject input or create an overlay.

For example, a persisted onboarding value `"true"` is a string, not a Boolean.
The visible issue identifies `settings.onboarding_complete`, the rejected
`"true"`, and default `false`; unrelated account fields are not included.

Where the project has explicitly elected one dual-licence option, its licence
row shows that selected SPDX expression. The bundled catalogue separately
retains the upstream declaration and every complete original legal text.

## Optional local Game.log witness

Open **Settings → Game log witness**, enter the directory of the game channel containing `Game.log` and `build_manifest.id`, and choose **Use channel and switch witness off**. Then select **Enable read-only Game.log witness**. Access is off by default; selecting another channel revokes the old opt-in. **Refresh witness facts** reads the selected files once on the background settings worker. There is no watcher or network request.

The status explains an unselected channel, unreadable files, an unvalidated build or payload, or a size limit. Settings shows the major/minor version and latest 50 sanitised buy-side observations; unknown terminal or commodity IDs remain absent. An enabled source can have zero observations when no kiosk was opened. The validated scope is build `4.10.197.2878` with branch `sc-alpha-4.10.2`, five exact terminal pairs and eleven commodity names listed in [the validation record](../game-log-validation.md). Extended 2026-11-03: MIC-L2 Long Forest Station and Orbituary join the initial stations, with Astatine, Carbon, Copper, Diamond, Potassium, Scrap and Waste. Similar names and other builds do not extend that scope.

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

Corrected 2026-11-02: source builds use fixed Development `0.0.0` for About, client headers and release-update comparison. Complete release metadata uses the CI-injected tag version and source commit; development builds make no claim to signed-release provenance. `tallyline --build-info` prints bounded public compiled metadata and exits before desktop startup, without needing resources or creating user state. Runtime environment variables cannot change this identity.

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

A non-JSON reference response adds a **Last non-JSON reference response** line with its HTTP status, validated `cf-ray` if available, and observation time as UTC Unix milliseconds. This is the last incident, so it remains visible after a successful refresh; the separate cache status and endpoint ages describe the current reference data. It contains no HTML response, other response headers or credentials, and it does not change the selected page or browsing choices.

## Screenshot formats

The shared decoder accepts SDR PNG, JPEG and BMP by their contents, even when a file extension is wrong. Sixteen-bit PNG samples retain their full precision through conversion to eight-bit sRGB. ICC colour profiles and supported PNG colour metadata are applied explicitly.

HDR PNG encoding and unsupported colour profiles receive **HDR or unsupported colour encoding – use an SDR copy**. JPEG XR, AVIF, HEIF and EXR originals are recognised but are not decoded or repeatedly retried. For a Game Bar JPEG XR original, use its SDR PNG copy when one is available; this does not claim that every capture tool creates such a copy. Other tools need an SDR PNG or JPEG capture. Capture-method recommendations still await the paired-capture checks in M2.

Files exceeding a hard size, metadata, pixel, profile or decoder-memory limit are set aside with a reason. Supported inputs above the 20 MP working budget are reduced with integer boxes, including edge pixels; the result records the original dimensions. Import controls and recognition are still being connected to this shared intake.

At startup Tallyline prepares the local OCR telemetry mitigation before opening storage or starting workers. It may relaunch the same executable once; no separate programme or launcher is required. On Windows a waiting bootstrap parent remains until the app closes and opens no helper console. Failed preparation stops startup before any OCR library can initialise. Native OCR loading and packaged telemetry checks are still in progress (R-NF-12).

## Manual commodity Drafts

Open **Manual Report**, then choose the environment, terminal and Buy/Sell side explicitly and select **Create Draft**. On first use no environment or side is selected. The environment of the most recently edited manual Draft is offered on later starts. Terminal search covers names, nicknames, screen display names and location/system text, including fuzzy character sequences; the separate location filter narrows results. Previously used terminals appear first. Shared names still show each terminal and location separately; select the intended result explicitly.

Known terminal commodities already loaded when the Draft is created appear immediately as empty rows. Reference data loaded later offers **Add commodity** actions for the remaining rows. The global vocabulary search adds an unexpected commodity explicitly. Added rows start with empty price, SCU and status inputs. UEX values stay beside those inputs, with their age from the server modification timestamp. **Take over UEX values explicitly** copies the displayed row only after that action; typed and taken-over fields have USER origin. Tab or Enter commits a field; enter a displayed status code with its number key. A zero SCU quantity is preserved as zero, while a traded price must be positive. Invalid numeric input stays visible for correction and is not committed. A pending save disables further field edits; a failure keeps the input and shows an error.

**Resume Draft** opens a saved Draft. Values are stored in the durable application database and available without credentials. Closing the application after a committed field preserves it for restart. Select another terminal, side or environment to change a Draft's context; user values are kept and references are selected again for that context. An environment change clears the old game-version assignment rather than silently relabelling the observation. The reviewed manual release controls below use the complete gate and durable queue; capture/OCR and older-capture version-history resolution remain in progress.

Corrected 2026-11-03 (durable queue): SQLite schema 9 supersedes the earlier schema-five description and stores manual Drafts, durable price/stock/status decisions and attachment metadata. Existing schemas 1–8 require the coordinated exact database/config backup before migration. New databases reach schema 9; older binaries refuse it. Migration preserves values, existing decisions, observation times, versions, recency, private crop digests and dated account screenshot requirements, without inventing price or stock approvals. The local Draft worker starts only after the startup coordinator completes and is joined before the writer lease ends. Native visual acceptance remains pending.

### Review prices and statuses

The top of a saved Draft shows price/stock/status counts for major differences, differences within tolerance, required confirmations and unreviewed decisions. The review section shows the exact entered price or SCU quantity, comparable reference, its age at review, signed unit difference and percentage. A missing prior is informational; a permitted commodity average is labelled separately and uses twice the effective price tolerance. An outdated reference suppresses minor colour marking but never lowers a major deviation or its confirmation requirement. USER values carry no invented reading confidence.

Press **F8** or **Shift+F8** to visit required decisions, or use the Previous/Next decision buttons. Focus clears only the durable unreviewed marker. Activate **Confirm price/stock/status for commodity**, cross-check the exact value and reference details in the dialog, then choose **Approve this exact value**. Cancel sends no approval. A currently loaded, privacy-confirmed private region from this Draft is shown at source scale with scrolling; use the attachment section's View action first if the saved private preview is not loaded. The review never loads an original screenshot.

Fresh local reference snapshots trigger passive reassessment without changing entered values, current selection or keyboard focus. While updating, previous reference details remain visible with an updating notice and cannot authorise approval. Uncommitted typing delays reassessment. A failed review retains the input and waits for **Review latest saved values**, rather than retrying continuously. Approvals bind the exact value, mapped UEX environment and unlowered deviation level: changing a value or context clears its approval; a worsening level revokes it. A larger amount or percentage within the same Major level does not revoke approval. A tighter effective tolerance revokes it if it makes the level worse; fresh-to-stale alone changes presentation, not approval. A revoked decision remains required after focus. Corrected 2026-11-02: these controls assess prices, SCU quantities and statuses. Broader SCU empirical validation remains open; complete release checks and sending remain unavailable.

Recognition integration now has a fixed hash-bound Small baseline, private native-loading guards and complete upstream legal texts. A separate synthetic Windows diagnostic checks model output classes; the desktop does not yet perform recognition. Linux, the full pipeline and installation-profile acceptance remain in progress (R-OCR-1, R-SEC-7).

The separate headless OCR diagnostic now verifies below-normal worker priority before it runs. This is a developer diagnostic; desktop recognition and bounded OCR scheduling remain in progress; the developer reader provides explicit inactivity cleanup (R-NF-3).

A developer-only headless raw-reader checkpoint is now implemented with explicit per-model session reuse and idle cleanup. Its portable geometry/crop profile and raw-image/textline probe modes are described in [the example guide](../../crates/app/examples/README.md); it does not enable desktop screenshot recognition. Production scheduling, field interpretation, packaged Linux checks and installation-profile acceptance remain in progress (R-OCR-1, R-NF-3).

Developer-only typed Tiny/Small/Medium OCR research pairs now retain exact model/dictionary identities and private raw experiment receipts. They reuse the existing verified native runtime; production model selection, hardware evidence and complete delivery remain open (R-OCR-1, R-SEC-7; ADR-0012).

## Private manual attachments

Corrected 2026-11-02: the local attachment workflow accepts an explicitly selected file and a shop-panel rectangle in source pixels. Include the side tabs and needed location information, while excluding everything above the panels, especially the balance and personal data. Only the selected region is retained for the full-size preview; decoding and hashing run on a bounded below-normal worker. HDR input needs an SDR copy, unsupported colour metadata is refused, and a source reduced by the intake budget cannot reuse source-pixel coordinates.

The attachment editor component provides a local path input or file drop, **Prepare selected region**, a scrollable **100% source-scale preview**, and **Contains no balance or personal data**. Confirming this preview writes only the private crop to the application data directory, then commits its digest and privacy decision together against the current Draft version. A changed source or rectangle requires another confirmation. File modification time is uncertain: confirm the shown time or enter an earlier UTC Unix time. The display uses a readable UTC date; confirmation, replacement and removal never advance the Report observation time.

**View saved private region** recovers the same source-scale pixels after restart, even if the original was moved or deleted. Missing or changed private files block attachment readiness and encoding. Select the source again, prepare its region and explicitly confirm the preview to repair the working copy. The original screenshot and its path are never stored in Report metadata. Removing an attachment does not delete retained Draft evidence automatically or refresh observation time.

Screenshot policy belongs to the currently validated account. A known requirement is shown with the date of its newest actual `screenshot_required` answer. **Try without screenshot** explicitly clears only this account's memory; unknown policy permits omitting an image, while a known requirement blocks that omission. `GET /user` supplies no requirement flag. A storage failure remains a visible block rather than becoming unknown policy. The submission queue commits actual screenshot-required answers before returning the Report to review; a dry-run test uses the same path.

Corrected 2026-11-03: SQLite schema 9 retains private-copy integrity and account-scoped dates introduced by schema 6, with a coordinated backup before migrating schemas 1–8. Configuration uses schema 6. Draft working copies under `manual-attachments/` in the durable data directory are never purged automatically. Installed Windows uses `%LOCALAPPDATA%/Tallyline/data`, Linux uses `$XDG_DATA_HOME/tallyline` (default `~/.local/share/tallyline`), and portable mode uses `<executable directory>/data/data`. Linux creates owner-only directories/files; Windows inherits the selected application data directory ACL. Crop files contain a bounded lossless ARGB region and source hash/rectangle metadata, never the original screen.

Open **Manual Report** and resume a Draft to use these controls, including offline reference state. The view uses the currently accepted credential account; another account never inherits its displayed requirement. Headless critical editor flows and real worker/storage restart paths are tested; no native visual acceptance is claimed. Attachment readiness checks do not complete the remaining Report release, observation-age, queue or sending rules; numeric review has its own gate.

## Diagnostics verification status

Diagnostics support exports are still being implemented. [The implementation record](../diagnostics-export.md) describes the tested masked-preview/file foundations; they do not yet expose an export action in this desktop checkpoint.

## Distribution verification status

The packaging helper now writes plain archives from prebuilt directories and verifies approved tool bytes and harmless-fixture boundaries. See [the packaging record](../packaging-spike.md). These foundation commands do not establish a release, a complete application distribution or successful MSI/Debian installation. Windows OCR still requires the separately installed Visual C++ Redistributable described by ADR-0009; packages must not bundle the forbidden Microsoft runtime DLLs. Installed/upgrade/native-display and no-extra-prerequisite acceptance remain pending.


Prebuilt staging now includes the approved native OCR library, Small model/dictionary and original font licences, and checks their exact identities alongside the legal catalogue. A restored original font-licence encoding corrects a prior packaging mismatch. These byte-level checks still do not establish installed or native-display acceptance.

The distribution checks now reject a self-consistent legal catalogue that differs from the approved target inventory. Source evidence checks also reject changed source against a frozen archive. These preparation checks do not complete the protected package or installed-system acceptance gates.

## Guarded SCU review

Corrected 2026-11-02: **Manual Report → Price, stock and status review** now compares entered SCU quantities against the matching inventory reference. It shows the prior and its age, signed SCU and percentage differences, effective bands and the provisional policy source. A missing prior has no commodity-average substitute; a known zero is shown as zero, with percentage unavailable. Future timestamps have unknown age. **Reference outdated** lowers only a within-tolerance display; a Major deviation still requires confirmation. SELL status 1 never changes stock.

The provisional starting bands are **20 % and 16 SCU**. A quantity is Major only when it strictly exceeds both. **Settings → SCU review tolerance** accepts independent stricter values: whole percentage 0–20 and exact SCU minimum 0–16. **Save SCU tolerance** applies only after the save succeeds. Invalid or looser input shows an explanation and is not applied. Invalid stored choices independently fall back to their baseline with a visible error. [The evidence note](../scu-policy-calibration.md) explains the narrow real movement and synthetic sensitivity support; broader empirical validation remains open.

Counters include stock decisions. **F8** and **Shift+F8** visit required decisions; focus clears an unreviewed marker but does not approve a value. **Confirm stock for commodity** opens the explicit confirmation dialog with exact value/reference details and only a current confirmed private attachment preview when available. Approving records this quantity and context. A changed quantity/context clears approval; fresh references revoke it when the unlowered class worsens. Same-class magnitude changes or ageing alone preserve it. Background reviews never open dialogs or move keyboard focus, and typing delays them.

Corrected 2026-11-03: database schema 9 stores SCU decisions and settings schema 6 stores the two bands, after coordinated backups of older files. Legacy quantities receive no invented confirmation. Manual release/submission uses the durable queue; native visual acceptance remains in progress; [the implementation record](../scu-review.md) describes the numeric readiness boundary.

Distribution preparation now validates mapped source files in the actual source archive and seals the source reference before harmless fixture insertion. Stale legal source bindings fail the normal check. This remains preparatory evidence: complete protected installers, actual UEX token-format confirmation and installed/native acceptance are pending.

The preparation check now also rejects changed workspace manifests, including feature-only changes that leave the dependency lockfile unchanged. Installer and installed-system acceptance remain pending.

Packaging preparation now has fixed MSI/Debian assembly adapters, original component terms and source scopes in Debian metadata, and an independently inspected core CRT prerequisite probe. Prebuilt preparation also refuses mismatched compiled identity and executable bytes before inserting fixture data. These are development checks, not a completed release or installation result. Windows OCR still requires the separately installed Visual C++ Redistributable; final protected package and clean-OS/native acceptance remain pending.

Corrected 2026-11-02: Development installer/archive payloads rebuilt from a signed source revision have been independently unpacked and checked with harmless fixtures, preferred source and complete listed legal material. Portable archives written on Windows keep shared libraries as non-executable data; Debian retains its separate package permission profile. The Debian launcher omits an optional icon reference when no icon is supplied, and WiX uses actual file metadata without an unnecessary language fallback. See [the exact verification scope](../prebuilt-package-verification.md). No authenticated release, Windows OS isolation or installed/native acceptance is implied; the Windows runtime prerequisite remains.


## Release a reviewed manual Report

Corrected 2026-11-03: manual submission now uses the [durable queue](../submission-queue.md). After reviewing the selected Draft, choose **Release Report in shown mode**, or **Send test through queue** for dry-run mode. The test action is also available from onboarding. It creates no UEX report IDs or local price priors. The action waits for an actual reference refresh attempt before checking the exact Draft, account and current review decisions. Offline cache and the last validated account may remain usable for durable release. Sending waits for successful account validation in the current session; unavailable reference ownership blocks release.

The global mode is always shown. **Settings → Submission** selects a mode for future releases and can tighten the observation-age limit from 60 minutes; the separate 24-hour hard limit remains fixed. Development builds and incomplete setup default to dry run; release builds after setup default to production unless overridden. Changing this setting or switching accounts never rewrites saved jobs. Starting a replacement credential candidate closes release and sending until current validation succeeds; offline revalidation of the remembered identity permits release while sending waits.

Open **Submission queue and history** to inspect the fixed account/mode, outcome and current waits. Cancel only unsent work; Sending cannot be cancelled. After restart, unfinished unsent work returns to Draft for renewed release. An uncertain in-flight outcome remains unknown and is never sent again automatically. Account changes offer an inline keep/review choice without moving focus. Older-row and age confirmations retain the original values and observation time.

Database schema 9 and settings schema 6 supersede the earlier schema descriptions above. Startup backs up the exact prior database/settings together before forward migration; older binaries refuse these schemas. Existing manual values, numeric decisions, attachments and account screenshot evidence are preserved. Native visual/provider acceptance and an explicitly approved real dry-run remain pending.
