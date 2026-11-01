# Tallyline

Tallyline is a cross-platform desktop client (Windows + Linux) for Star Citizen DataRunners. It captures commodity terminal data – **manually or via screenshot OCR** – and submits it to [UEX Corp](https://uexcorp.space). Current UEX data serves as default values and plausibility bounds for recognition.

> Corrected 2026-11-01: the GitHub repository is now [greluc/tallyline](https://github.com/greluc/tallyline).

While the game is running, a lightweight local OCR does the work. Optionally, a local AI model (via Ollama) re-checks the results once the game is closed.

> Status: **M0 foundation in progress**. The workspace contains repository checks, an egui design-system crate and a UEX API spike CLI; the desktop client is not yet available. The implementation uses Rust and egui, with native code only in registered exceptions ([ADR-0005](docs/adr/0005-rust-and-egui.md)).

## Plan

| Document | Contents |
|---|---|
| [01 – Requirements](docs/plan/01-requirements.md) | Functional and non-functional requirements, assumptions |
| [02 – Architecture](docs/plan/02-architecture.md) | Crates, data flow, domain model, platform integration, data-flow inventory |
| [03 – Language decision (ADR-0001)](docs/plan/03-language-decision.md) | Java vs. Rust, evaluation; superseded by ADR-0005 (kept as the record of the evaluation) |
| [04 – Roadmap](docs/plan/04-roadmap.md) | Milestones M0–M5 with acceptance criteria, requirement coverage, risks |
| [05 – SC-Datarunner-UEX bug analysis](docs/plan/05-datarunner-bug-analysis.md) | Known bugs of the predecessor and our fixes |
| [06 – UEX API](docs/plan/06-uex-api.md) | Endpoints, payload, open verification points |
| [07 – OCR concept](docs/plan/07-ocr-concept.md) | Pipeline, screenshot observations, lessons from basetool-sc-extractor |
| [08 – Review](docs/plan/08-review.md) | Plan review: corrected errors, closed gaps, open points |
| [09 – Engineering principles](docs/plan/09-engineering-principles.md) | Modularisation, clean code, tests, definition of done, the native-code exception register – with enforcement |
| [10 – Supply-chain security](docs/plan/10-supply-chain-security.md) | Threat model, securing dependencies, build, CI and releases, licence gates |
| [11 – DDD and TDD](docs/plan/11-ddd-and-tdd.md) | Ubiquitous language, bounded contexts, aggregates and invariants; test-driven approach |
| [Release process](docs/release-process.md) | Versioning, release checklist, licence compliance, upgrade/downgrade, patch-day and API-change procedures |
| [ADR-0002 – Modules by bounded context](docs/adr/0002-modules-by-bounded-context.md) | Why the core is cut by bounded context and adapters by technology (in Rust: Cargo workspace crates) |
| [ADR-0005 – Rust and egui](docs/adr/0005-rust-and-egui.md) | Implementation in Rust with egui; native code only by registered exception (`ort`, bundled SQLite, the TLS provider); supply chain, licence allowlist and packaging for Rust |
| [ADR-0003 – Licence and contributions](docs/adr/0003-licence-and-contributions.md) | GPL-3.0-or-later, DCO and CLA, the Star Citizen Fan Kit unit, notice and source duties, licence gate |
| [ADR-0010 – Release token delivery](docs/adr/0010-release-token-delivery.md) | Separate protected packaging jobs and replacement-token releases |
| [Open points](docs/adr/0000-open-points.md) | The single register of open owner decisions and open verifications |
| [Dependency pins](docs/dependency-pins.md) | Tags of SHA-pinned GitHub Actions; versions and checksums of CI tools |
| [Design-system brief](docs/prompts/design-system.md) | Staged brief for Claude Design: an original design system and a screens canvas, at genre level close to the current Star Citizen website and forum, with the Fan Kit unit and the About dialog; with its [stage messages](docs/prompts/design-system-kickoff.md), the [owner checklist](docs/prompts/design-system-owner-checklist.md) and the [JavaFX handoff](docs/prompts/design-system-javafx-handoff.md) for Claude Code (historical since ADR-0005; the egui rendition replaces it, O-112) |

## Contributing

Contributions are welcome: plan reviews, corpus captures and, later, code. Read [CONTRIBUTING.md](CONTRIBUTING.md) first. In short:

- every commit carries a DCO sign-off made with `git commit -s` and a Conventional Commits message;
- every contributor signs the [Contributor Licence Agreement](CLA.md) once; the signatures are listed in [docs/cla-signatures.md](docs/cla-signatures.md);
- pull requests are merged without squashing;
- everyone follows the [Code of Conduct](CODE_OF_CONDUCT.md).

## Security

Please report vulnerabilities privately, never in a public issue. How to do that, and what to expect, is described in [.github/SECURITY.md](.github/SECURITY.md).

## Licence

Tallyline is free software, licensed under the GNU General Public License, version 3 or (at your option) any later version (SPDX identifier `GPL-3.0-or-later`); see [LICENSE](LICENSE). It comes with no warranty. The decision is recorded in [ADR-0003](docs/adr/0003-licence-and-contributions.md).

- The licence covers the project's own code and documentation. Not everything in this repository is ours to license: the screenshots in [corpus/](corpus/README.md) show Star Citizen game content of Cloud Imperium, the Fan Kit logo files are Cloud Imperium's artwork, and the Code of Conduct is under CC-BY-SA-4.0. [REUSE.toml](REUSE.toml) gives the licence of every file, and [LICENSES/](LICENSES/) holds the licence texts.
- [NOTICE](NOTICE) lists every third-party component that the planned installers ship, with its licence and source, and the material that keeps its own licence. Every release will ship the licence texts and notices and attach the source archives that the bundled copyleft components require.

## Open points

Decisions and checks that are still open are tracked in [docs/adr/0000-open-points.md](docs/adr/0000-open-points.md). Among them:

- the trademark and conflict search (O-8);
- the acceptance date of the Fankit Agreement (O-26), and which logo variant goes on light and on dark backgrounds (O-27; this README follows basetool's mapping until the project owner has checked the kit);
- whether Cloud Imperium's terms allow the game screenshots in this public repository (O-28);
- a legal review before the first release of the Windows app's dependency on the Microsoft Visual C++ Redistributable (x64), which ONNX Runtime needs; the packages do not ship it, so users install it as a prerequisite (O-12, ADR-0009);
- the remaining egui dependency audits and icon review (O-112);
- the M1 spike for the installers (`cargo-packager` with a pre-installed, pinned WiX 3.11.2, decided in O-116): offline build, `.deb` and archive formats.

## Star Citizen fan content

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/images/fankit/MadeByTheCommunity_White.png">
    <img alt="Made By The Community" src="docs/images/fankit/MadeByTheCommunity_Black.png" width="150" height="150">
  </picture>
</p>

Star Citizen®, Roberts Space Industries® and Cloud Imperium ® are registered trademarks of Cloud Imperium Rights LLC

This site is not endorsed by or affiliated with the Cloud Imperium or Roberts Space Industries group of companies. All game content and materials are copyright Cloud Imperium Rights LLC and Cloud Imperium Rights Ltd.. Star Citizen®, Squadron 42®, Roberts Space Industries®, and Cloud Imperium® are registered trademarks of Cloud Imperium Rights LLC. All rights reserved.

The logo, the trademark line of the Fan Kit Guidelines §2b and the notice of the Fankit Agreement clause 2(g) above form one unit and are reproduced verbatim ([CONTRIBUTING.md](CONTRIBUTING.md)). The logo files are Cloud Imperium's artwork, used under the Fankit Agreement; they are not covered by this project's licence (`LicenseRef-Fankit-Agreement` in [REUSE.toml](REUSE.toml)).

## Disclaimer

Independent community project, not made or endorsed by UEX Corp and not affiliated with UEX Corp or Cloud Imperium Games / Roberts Space Industries.

## Corpus contributors

Use the offline [corpus capture and annotation guide](docs/user/corpus-capture.md) and `cargo run -p ocr-eval --locked -- inspect` to inspect conditions, expected values and sample readiness. Recognition metrics are not implemented yet.
