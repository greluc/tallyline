# Tallyline

Tallyline is a cross-platform desktop client (Windows + Linux) for Star Citizen DataRunners. It captures commodity terminal data – **manually or via screenshot OCR** – and submits it to [UEX Corp](https://uexcorp.space). Current UEX data serves as default values and plausibility bounds for recognition.

> Corrected 2026-11-03: the GitHub repository has transferred to [tallyline-sc/tallyline](https://github.com/tallyline-sc/tallyline).

While the game is running, lightweight local OCR does the work. Optional local AI can re-check results once the game is closed. The planned installation supplies its selected recognition runtime and models without a separate Ollama installation; the engine and model combinations are still being evaluated. Images are never sent to cloud or LAN inference services ([ADR-0012](docs/adr/0012-strictly-local-recognition-delivery.md)).

> Status: **M1 desktop foundation in progress**. The native shell provides first-launch onboarding, themes, legal notices and atomic application settings. Manual commodity Drafts can be created, edited by keyboard and resumed after restart without credentials. Reviewed manual Reports now use a durable serial submission queue; capture and OCR workflows remain in progress. Native and real-service acceptance remain open. The implementation uses Rust and egui, with native code only in registered exceptions ([ADR-0005](docs/adr/0005-rust-and-egui.md)).

Local builds identify themselves as **Development 0.0.0**. `target/debug/tallyline --build-info` (Windows: `tallyline.exe`) prints public compiled metadata and exits before desktop startup, without assets or user state. Release versions derive from compile-time tag metadata; a sealed prebuilt handoff binds its report to the executable hash before fixture insertion. Signed release provenance and final package/installed acceptance remain separate release gates.

## Run the desktop shell from source

Build with `cargo build --locked -p tallyline-app`, then run `target/debug/tallyline --assets crates/ui/assets` (on Windows, `target/debug/tallyline.exe`). The explicit assets argument selects the separately shipped font files; an installed or portable image instead keeps `assets/fonts/` next to the executable. See the [desktop guide](docs/user/desktop.md) for the storage paths, portable marker and settings errors. An explicit second launch restores the existing window even when it is minimised. Reference browsing loads SQLite before refreshing public UEX endpoints in the background. Reference-data adapters use exact decimal values and environment/static Windows proxy support; PAC and per-protocol Windows proxy lists remain unsupported by the current upstream path.

Mutation scores also require retained cause logs; unrelated infrastructure failures invalidate the measurement. See [the evidence boundary and corrected historical results](docs/mutation-evidence.md).

Source builds can keep their build products and automatic xtask work outside the checkout through `CARGO_TARGET_DIR`; executable paths shown under `target/` use the default location. Set external `TEMP`/`TMP` as well for test and mutation scratch directories. See [external workspace output commands](xtask/README.md#external-workspace-outputs).

The optional **Game log witness** in Settings reads only version-validated local facts after explicit opt-in. Choose a channel, enable the witness and refresh to inspect sanitised buy-side observations; `cargo run -p ocr-eval --locked -- witness --channel <directory>` offers the same bounded read-only inspection. The finite allowlist now includes MIC-L2 Long Forest Station and Orbituary, with seven additional literal commodity names; [the validation record](docs/game-log-validation.md) lists the exact pairs and supported build. Prices, quantities and status values are never filled from the log (R-OCR-20).

The **Reference data** page lets you choose LIVE, HOTFIX, PTU, EPTU or TECH-PREVIEW and search terminal names, locations, commodity names and codes. It retains your selection and search during passive refreshes and displays the supplied cache state and endpoint ages. **Settings → UEX environment mapping** changes each channel independently of the optional Game.log witness. Global staff notices remain capped plain text, including while submissions are paused. Corrected 2026-11-02: an app-owned native worker publishes SQLite snapshots before refreshing from UEX.

Onboarding and **Settings → UEX account** accept a masked secret key, use the packaged app token when available and offer an expert own-token override. Read-only validation precedes durable saving; an explicit session-only choice keeps credentials in memory. Linux offers an owner-only plaintext fallback only after Secret Service is unavailable and its warning is confirmed; unsupported filesystems retain the session-only option. Offline browsing and Drafts remain usable. Real OS-provider and native visual acceptance are still pending (R-API-4, R-API-7, R-NF-4).

The manual editor offers **Release Report in shown mode** and **Send test through queue**. Test always fixes dry-run mode. Queue/history preserves the released account and mode, shows holds and permits cancellation before Sending; uncertain sends never replay automatically. See [the queue implementation and acceptance boundaries](docs/submission-queue.md).

Settings also offers optional **daily release updates**, off by default. A newer standard GitHub release appears as a passive notice with release notes; offline checks never interrupt work or take focus, and failed settings validation preserves saved preferences. Settings backup retention also survives clock changes. Downloads and installation remain your choice (R-NF-7, R-UI-16).

Starting Tallyline again activates the existing installed or portable instance. If activation fails, an already-running notice appears and no second writer starts (R-NF-10).

Open-source licence rows show documented licence choices; their upstream declarations and complete legal texts remain in the offline catalogue. The current source graph uses egui/eframe 0.37.0; [the adoption record](supply-chain/latest-stable-adoption.json) lists eligible releases, upstream constraints and the approved native limits. The [image intake record](supply-chain/image-intake-adoption.json) records decoder selection and the test-only encoder scope.

The shared image intake decodes SDR PNG (including full-precision 16-bit samples), JPEG and BMP into sRGB. It identifies HDR containers and unsupported colour encodings by content, gives SDR-copy advice and bounds file, header, pixel, profile and codec memory before decoding. Large supported inputs receive integer box reduction with their original dimensions recorded (R-CAP-8, R-OCR-17). Capture import controls and the recognition pipeline remain in progress. The headless OCR diagnostic now starts through an app-owned worker that verifies below-normal OS priority before running; production OCR scheduling remains open (R-NF-3).

## Plan

Maintainers can check public UEX field/type/code changes without credentials through the existing API-spike CLI. The [contract drift guide](docs/public-api-contract-drift.md) explains the metadata-only baseline and the default-off, main-only weekly workflow; hosted activation remains pending.

The [support-export foundation](docs/diagnostics-export.md) supplies masked previews and exclusive private JSON writes with fixed application-log tails. The desktop Diagnostics and logging flow is still being implemented.

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
- the M1 installer spike: source-compiled WiX 7.0.1 through checked xtask for MSI, cargo-packager for Debian, offline prebuilt assembly and plain archives (corrected 2026-11-02; ADR-0011).

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

Game preferences now persist independent channel-to-UEX mappings in settings schema 4, preserving witness and release-check choices across migration; the browsing controls save these mappings before applying them.

Startup config migration coordinates existing SQLite and exact settings bytes in one verified backup set, and refuses processing if the backup fails.

Corrected 2026-11-02: the schema-eight application database stores manual Drafts, price/stock/status review decisions and attachment metadata while preserving exact buy/sell averages and known terminal container limits; PTU snapshots contain no LIVE averages. Existing databases require a coordinated backup before migration. Manual Report now shows exact price/SCU/reference differences and price/stock/status counters, F8/Shift+F8 navigation and explicit value-bound confirmation dialogs. Passive reassessment preserves USER values and revokes approvals only under the documented worsening rule. Broader SCU empirical validation, complete release/submission and native visual acceptance remain open.

Reference snapshots now retain UEX acceptance switches, commodity tolerances and capped literal staff information independently of browsing availability.

Terminal priors now cache exact price/stock values and source provenance independently for 30 minutes; the typed reader accepts at most ten distinct terminal IDs and PTU snapshots omit LIVE priors.

The native reference browser now receives SQLite snapshots before its first network response and refreshes through the primary UEX host on an app-owned worker. Closing cancels pending GETs and joins the cache owner before releasing writer locks.

Reference status retains bounded Cloudflare Ray metadata and the injected UTC observation time for the last non-JSON response. The primary app uses the fixed `.uk` origin; the technical CLI requires explicit expert-host disclosure before sending credentials to an alternative host. Cached browsing remains usable after a transport failure.

Tallyline prepares local OCR telemetry mitigation before startup work. If required, it relaunches its own executable once with a private child environment; no separate launcher is needed. Windows keeps a waiting bootstrap parent without opening a helper console. The native OCR and packaged network verification remain in progress (R-NF-12).

Recognition integration now includes fixed Small model/dictionary/native data, verified private loading and an explicit headless Windows tensor diagnostic. Synthetic model outputs match the fixed dictionary classes. A developer-only portable raw-image/textline reader adds DB boxes, original-raster crops and lazy reusable sessions with explicit inactivity cleanup; desktop capture/recognition, Linux and complete installation-profile acceptance remain in progress (R-OCR-1, R-SEC-7).

Developer-only typed Tiny/Small/Medium OCR research pairs now retain exact model/dictionary identities and private raw experiment receipts. They reuse the existing verified native runtime; production model selection, hardware evidence and complete delivery remain open (R-OCR-1, R-SEC-7; ADR-0012).

Corrected 2026-11-02: confirmed manual screenshot regions now persist at source scale in the private application data directory and remain viewable and encodable after the original is deleted or the process restarts. Missing or changed working copies block attachment use. SQLite schema 8 migrates after a coordinated backup and stores crop integrity plus account-specific screenshot requirement dates; an explicit clear action permits trying without an image. A headless-tested attachment editor component supports explicit local selection/cropping, full-size privacy review, time decisions and actionable blocks. The controls are composed into Manual Report using the current accepted account, including offline reference state. Complete release/submission and native visual acceptance remain in progress.

Prebuilt distribution helpers verify version/MSI limits, approved tools and sealed payload/legal/source inputs before fixture insertion. Rebuilt Development MSI/ZIP/deb/tar.gz packages have independent source, payload, legal and metadata inspections; see [their exact signed-source tuples](docs/prebuilt-package-verification.md) and [remaining packaging gates](docs/packaging-spike.md). Portable shared-library data keeps non-executable permissions when archives are written on Windows. Windows OS isolation, authenticated release and clean-OS/native acceptance remain open. Windows OCR retains the separately installed Visual C++ Redistributable prerequisite of ADR-0009; these checks do not establish a no-extra-prerequisite installation.


Prebuilt preparation now seals the existing app, approved native/model/font resources, complete legal material and exact packaging tools before token insertion. The retained seal detects changed bytes, unlisted files or a different helper; physical notice text must match the legal catalogue. The original IBM font licence encoding is restored to its selected upstream hash. Protected installer/source-duty acceptance remains in progress; see the packaging record.

Packaging verification now pins each target’s exact legal catalogue to reviewed source inputs. Secret-free source evidence also rejects a source archive that differs from its unpacked tree. Complete native source scopes and protected installer assembly remain in progress; see [the packaging record](docs/packaging-spike.md).

Manual Report now includes [guarded SCU review](docs/scu-review.md), with provisional 20 % / 16 SCU bands, independent tighten-only Settings, durable exact-value approvals, counters and F8/Shift+F8 navigation. The [policy evidence](docs/scu-policy-calibration.md) records the empirical limits. Corrected 2026-11-03: database schema 9 and settings schema 6 migrate after coordinated backups; manual release/submission uses the durable queue, while native visual acceptance remains in progress (R-VAL-2, R-VAL-7, R-UI-10..12).

Source preparation now verifies actual archived component bytes and seals a source reference before harmless fixture insertion. The normal check rejects stale legal source bindings. Protected installer and installed-system acceptance remain pending; see [the packaging record](docs/packaging-spike.md).

Distribution source checks also bind every workspace manifest, so feature-only changes invalidate stale legal source bindings even when Cargo.lock stays unchanged.

The packaging helper now includes fixed native backend adapters and independently inspected Debian archive/CRT prerequisite contracts. Actual final protected distributions and clean-OS/native acceptance remain pending; the Windows Visual C++ runtime prerequisite remains unchanged.

Corrected 2026-11-03: remembered validated accounts can release reviewed Reports to the durable queue offline; dispatch waits for successful validation in the current session. Replacement credentials close both release and sending until validated (R-API-7, R-SUB-2). Native/provider and approved real-service acceptance remain open.

The source foundation also contains a versioned local-VLM transcription parser and bounded answer retry contract. It does not select or run an inference engine or change the desktop recognition flow (R-VLM-5, R-VLM-10).
