# ADR-0010: Release token delivery

> **Doc type:** Decision record — Accepted. Last reviewed: 2026-11-01.

- **Status:** Accepted (2026-11-01)
- **Decider:** @greluc
- **Related:** O-85, O-139, R-NF-4, R-SUB-11, 10 S-14, S-15

## Context

The Tallyline app token identifies the client and is extractable from a release. UEX requires each write to carry the user’s own secret key as well. The token must stay out of the repository, logs and jobs that run project build code. Recreating the UEX app invalidates the old token immediately; installed releases need a replacement.

## Decision

Secret-free build jobs produce the binaries and SBOM. Separate packaging jobs in the protected `release` environment add the token as a data file next to each prebuilt executable and package the binaries without running project build code. The publish job downloads, hashes, attests and publishes the packages. The platform adapter reads the file through `SecretStore`; a missing or empty file prompts for the user’s own token. Logs mask the token and the UI never shows it.

A replacement token is delivered in a PATCH release. Until that update is installed, `access_denied` pauses the whole queue and shows the invalid-token state; the existing expert setting also permits a user’s own token. The owner creates the app and stores its token only as a secret in the protected `release` environment. Each external write requires its own approval.

## Alternatives and verification

Compile-time injection would expose the token to project build code. Injection in the publish job would combine packaging and publishing duties. The M1 packaging spike verifies that the pinned tools and configuration package prebuilt binaries without invoking project build code, that every distribution contains the data file, and that logs and intermediate build outputs do not contain the token. The existing gitleaks rule detects an Authorization Bearer assignment, not every possible bare token file; the M1 work must verify coverage for the actual token format without committing the token. The workflow and packaged-app smoke test remain M1 work; this decision does not claim they exist or that the packaging method has been verified.
