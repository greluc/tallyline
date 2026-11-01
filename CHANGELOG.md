# Changelog

All notable changes are recorded here.

## [Unreleased]

### Added

- Added optional daily GitHub release checks, off by default, and a persistent dismissible release-notes link. Offline or invalid replies remain passive; checks never download or install updates, and the permission, last attempt, validated notice and dismissal survive restarts in settings schema 3 (R-NF-7, R-UI-16).

- Added an optional, default-off Game.log witness with channel selection, safe unavailability reasons and sanitised facts in Settings and the offline eval tool. Only the documented build and finite mapping allowlist are accepted; logs never supply reported prices, quantities or status values, and settings migrate forwards to schema 2 (R-OCR-20, R-NF-5).

- A second installed or portable launch activates the existing window through local IPC. Both writer locks remain held for the application lifetime; failed activation shows an already-running notice and never starts another writer (R-NF-10).

- Added the native desktop shell with first-launch onboarding, dark, light and high-contrast themes, offline legal notices and persistent application settings. The workspace and Settings have clear Home/Done navigation, and Settings failures remain visible without taking focus; installed and portable builds keep state in their documented directories (R-UI-7:theme, R-UI-8, R-UI-16, R-UI-18, R-UI-19:app, R-L10N-1, R-NF-2:portable, R-NF-5, R-NF-8).

- Offline corpus inspection reports schema-v2 conditions, expected values, frozen reference metadata, sample readiness and unavailable inputs. Anchor annotation writes a separate unverified proposal, and a contributor capture guide explains reproducible ladders and privacy protections (R-QA-1, R-QA-3, R-DOC-1).

### Fixed

- Settings upgrades require a successful backup before replacement. The bootstrap without a database preserves exact previous settings and retains the last three recognised backup sets; a database requires coordinated backup (R-NF-11).

- A second launch restores a minimised window even when no view is being drawn. Ordinary background results remain unable to restore or focus it (R-NF-10, R-UI-16).

- OCR test fixtures retry transient Windows directory-allocation denial within their existing limit, without skipping assertions or changing product storage behaviour.

- Open-source licence rows show the documented elected option while retaining the upstream declaration and every original legal text; stale or incompatible elections fail generation (R-UI-18).

- Invalid onboarding values now report the actual rejected JSON value and default without exposing unrelated account settings (R-NF-5).

- Installed Windows data and Linux logs now use the dedicated directories required by the storage specification; portable paths retain their separate layout (R-NF-5).

- About identifies the fixed development package version together with the OS and architecture (R-UI-18).

- Theme changes now repaint the native root surface and text together, including dark/light/dark transitions. The offline licence catalogue excludes Apache boilerplate from copyright-holder metadata and preserves the actual AccessKit source attribution.

### Changed

- Migrated the Pyro Gateway transcription to schema version two with unknown conditions left null; human verification is still required (R-QA-1).
