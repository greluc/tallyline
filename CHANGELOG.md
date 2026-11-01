# Changelog

All notable changes are recorded here.

## [Unreleased]

### Added

- Added the native desktop shell with first-launch onboarding, dark, light and high-contrast themes, offline legal notices and persistent application settings. The workspace and Settings have clear Home/Done navigation, and Settings failures remain visible without taking focus; installed and portable builds keep state in their documented directories (R-UI-7:theme, R-UI-8, R-UI-16, R-UI-18, R-UI-19:app, R-L10N-1, R-NF-2:portable, R-NF-5, R-NF-8).

- Offline corpus inspection reports schema-v2 conditions, expected values, frozen reference metadata, sample readiness and unavailable inputs. Anchor annotation writes a separate unverified proposal, and a contributor capture guide explains reproducible ladders and privacy protections (R-QA-1, R-QA-3, R-DOC-1).

### Fixed

- OCR test fixtures retry transient Windows directory-allocation denial within their existing limit, without skipping assertions or changing product storage behaviour.

- Open-source licence rows show the documented elected option while retaining the upstream declaration and every original legal text; stale or incompatible elections fail generation (R-UI-18).

- Invalid onboarding values now report the actual rejected JSON value and default without exposing unrelated account settings (R-NF-5).

- Installed Windows data and Linux logs now use the dedicated directories required by the storage specification; portable paths retain their separate layout (R-NF-5).

- About identifies the fixed development package version together with the OS and architecture (R-UI-18).

- Theme changes now repaint the native root surface and text together, including dark/light/dark transitions. The offline licence catalogue excludes Apache boilerplate from copyright-holder metadata and preserves the actual AccessKit source attribution.

### Changed

- Migrated the Pyro Gateway transcription to schema version two with unknown conditions left null; human verification is still required (R-QA-1).
