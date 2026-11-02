# Changelog

All notable changes are recorded here.

## [Unreleased]

- Add explicit developer-only Tiny/Small/Medium OCR research pairs with exact model/dictionary bindings and private experiment receipts; production profile selection and real hardware acceptance remain open (R-OCR-1, R-SEC-7; ADR-0012).

- Add fixed Small OCR model/dictionary/native data, private Windows load and mapped CRT checks, immutable model snapshots and a bounded synthetic tensor diagnostic. Desktop OCR, Linux, production worker lifecycle and installation-profile acceptance remain in progress (R-OCR-1, R-SEC-7, R-SEC-8, R-NF-12, R-NF-13).

- Prepare local OCR telemetry mitigation before startup through a bounded relaunch of the same executable, without requiring a separate launcher or mutating a running multithreaded environment (R-NF-12).

### Added

- Add a developer-only portable Small image/textline reader with DB boxes, original-raster crops, lazy session reuse and explicit idle cleanup. Desktop screenshot recognition and full installation-profile acceptance remain in progress (R-OCR-1, R-NF-3).

- Run the headless OCR diagnostic through an owned worker that verifies below-normal OS priority before its task starts. Production OCR scheduling remains in progress (R-NF-3).

- Preserve manual status approvals, revoked decisions and keyboard-review state across restart at the application boundary. Schema-four migration requires a coordinated backup; review controls and release remain in progress (R-UI-12, R-VAL-7, R-NF-11).

- Added shared bounded PNG/JPEG/BMP intake with explicit colour conversion, full-precision 16-bit PNG handling, HDR/unsupported-colour advice and original-size findings after integer box reduction. The capture UI and recognition pipeline remain in progress (R-CAP-8, R-OCR-17).
- Reference failures now show the last non-JSON HTTP status, validated Cloudflare Ray and UTC observation time without retaining HTML or changing cached data. HTTPS and same-origin checks precede every request; credentialled technical host overrides require explicit confirmation and disclosure (R-API-3, R-NF-4).
- Added persistent manual commodity Drafts with explicit terminal, side and environment choices, keyboard field entry, aged UEX references and explicit takeover. Draft edits survive restart without credentials; schema-three storage migrates after coordinated backups. Shared validation and submission remain in progress (R-MAN-1, R-MAN-2, R-MAN-3, R-MAN-6, R-NF-11).

- The desktop reference worker now loads SQLite data before network responses, rechecks versions at startup and fetches terminal priors on opening. Failed saves retain old visible records; closing cancels pending GETs and joins the cache owner before releasing the writer lease (R-API-1, R-API-2, R-UI-13, R-NF-10).

- Per-terminal LIVE priors retain exact prices and SCU, container sizes, status and source age across restarts, with independent 30-minute freshness and bounded ten-terminal requests. PTU contains no borrowed LIVE priors (R-API-1, R-API-5, R-API-6).

- Added environment-specific terminal and commodity browsing with searchable cached records, passive refresh status and endpoint ages. Channel-to-UEX mapping controls save before applying and remain independent of the optional Game.log witness; cached records now load before HTTP and refresh on the app-owned worker. Global staff notices are capped literal text, and resolved reference failures clear without hiding settings errors (R-API-1, R-API-2, R-API-6, R-CAP-3a, R-UI-8, R-UI-13, R-UI-16).

- Cached UEX submission switches, commodity tolerances and bounded staff messages remain available offline. Submission closure leaves browsing usable; parameters refresh daily and require a fresh observation before sending (R-API-1, R-UI-8, R-SUB-5).

- Cached commodity averages retain exact buy/sell values and terminal container limits. SQLite schema 2 backs up old data before migration, preserves vocabulary and fetch times, and removes the ambiguous legacy average without introducing LIVE values into PTU (R-API-1, R-API-6, R-VAL-2a, R-NF-11).

- Game-channel mappings persist independently of the optional Game.log witness in settings schema 4. Migration preserves release-check preferences and backs up old settings before replacement (R-CAP-3a, R-NF-11).

- Added exact decimal price/quantity value types and enabled environment/static Windows proxy support in the UEX adapter. PAC and per-protocol Windows proxy lists remain unsupported; desktop reference-data wiring is still in progress (R-API-3, R-API-5).

- Added optional daily GitHub release checks, off by default, and a persistent dismissible release-notes link. Offline or invalid replies remain passive; checks never download or install updates, and the permission, last attempt, validated notice and dismissal survive restarts in settings schema 3 (R-NF-7, R-UI-16).

- Added an optional, default-off Game.log witness with channel selection, safe unavailability reasons and sanitised facts in Settings and the offline eval tool. Only the documented build and finite mapping allowlist are accepted; logs never supply reported prices, quantities or status values, and settings migrate forwards to schema 2 (R-OCR-20, R-NF-5).

- A second installed or portable launch activates the existing window through local IPC. Both writer locks remain held for the application lifetime; failed activation shows an already-running notice and never starts another writer (R-NF-10).

- Added the native desktop shell with first-launch onboarding, dark, light and high-contrast themes, offline legal notices and persistent application settings. The workspace and Settings have clear Home/Done navigation, and Settings failures remain visible without taking focus; installed and portable builds keep state in their documented directories (R-UI-7:theme, R-UI-8, R-UI-16, R-UI-18, R-UI-19:app, R-L10N-1, R-NF-2:portable, R-NF-5, R-NF-8).

- Offline corpus inspection reports schema-v2 conditions, expected values, frozen reference metadata, sample readiness and unavailable inputs. Anchor annotation writes a separate unverified proposal, and a contributor capture guide explains reproducible ladders and privacy protections (R-QA-1, R-QA-3, R-DOC-1).

### Fixed

- Give the Game channel directory input the same accessible name as its visible label (R-UI-17a, R-OCR-20).

- Settings migration with an existing SQLite database now creates one integrity-checked database/config backup set before replacement. Backup failure stops startup processing and preserves the previous files (R-NF-11).

- Backup retention preserves the newly created snapshot when the clock moves backwards and retains the newest three completed creations across restarts. Failed backups and timestamp collisions leave existing snapshots untouched (R-NF-11).

- Update-check timestamps outside the settings format are refused without panicking or replacing saved preferences (R-NF-7, R-NF-5).

- Settings upgrades require a successful backup before replacement. The bootstrap without a database preserves exact previous settings and retains the last three recognised backup sets; a database requires coordinated backup (R-NF-11).

- A second launch restores a minimised window even when no view is being drawn. Ordinary background results remain unable to restore or focus it (R-NF-10, R-UI-16).

- OCR test fixtures retry transient Windows directory-allocation denial within their existing limit, without skipping assertions or changing product storage behaviour.

- Open-source licence rows show the documented elected option while retaining the upstream declaration and every original legal text; stale or incompatible elections fail generation (R-UI-18).

- Invalid onboarding values now report the actual rejected JSON value and default without exposing unrelated account settings (R-NF-5).

- Installed Windows data and Linux logs now use the dedicated directories required by the storage specification; portable paths retain their separate layout (R-NF-5).

- About identifies the fixed development package version together with the OS and architecture (R-UI-18).

- Theme changes now repaint the native root surface and text together, including dark/light/dark transitions. The offline licence catalogue excludes Apache boilerplate from copyright-holder metadata and preserves the actual AccessKit source attribution.

### Changed

- Require strictly local optional AI and an initially complete recognition profile without a separately installed Ollama. Runtime, complementary OCR and hardware-specific model selection remain under evaluation; the existing Windows OCR prerequisite still needs a verified replacement (R-VLM-6, R-VLM-7, R-VLM-11; ADR-0012).

- Refresh the desktop UI, TLS and async dependencies to eligible stable releases while preserving the approved native boundaries and complete legal notices (R-SEC-1, R-SEC-10).

- Migrated the Pyro Gateway transcription to schema version two with unknown conditions left null; human verification is still required (R-QA-1).
