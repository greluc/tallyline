# Documentation

> **Doc type:** Living reference — current. Last reviewed: 2026-11-02.

`docs/` is the project's single source of truth ([`CLAUDE.md`](../CLAUDE.md), "The documentation is the project"). This file says what belongs where. A new directory or top-level file under `docs/` gets its row here in the same commit.

| Path | What belongs there |
|---|---|
| [`plan/`](plan/) | The plan, numbered 01–11: requirements (01), architecture (02), language decision ADR-0001 (03, superseded by ADR-0005), roadmap (04), bug analysis of the predecessor (05), UEX API notes (06), OCR concept (07), plan reviews (08), engineering principles (09), supply-chain security (10), DDD and TDD (11) |
| [`adr/`](adr/) | `0000-open-points.md`, the single register of open owner decisions and open verifications, and the decision records `NNNN-kebab-title.md` (ADR-0001 lives in `plan/03-language-decision.md`; ADR-0005 `0005-rust-and-egui.md` is the current implementation-language, toolkit, supply-chain and packaging decision; ADR-0006 `0006-tls-provider-aws-lc-rs.md` chooses the `rustls` crypto provider; ADR-0007 `0007-ort-rc14-and-onnx-runtime-link-mode.md` fixes the `ort` version and how the ONNX Runtime library is shipped and loaded; ADR-0008 `0008-onnx-runtime-telemetry.md` records how ONNX Runtime's telemetry is switched off and what is accepted; ADR-0009 `0009-visual-cpp-runtime-as-prerequisite.md` makes the Visual C++ runtime of ONNX Runtime a prerequisite instead of shipping it; ADR-0010 `0010-release-token-delivery.md` records release token delivery and replacement) |
| [`agents/`](agents/) | Per-repository configuration for the engineering agent skills: where issues are tracked (GitHub, writes need approval), the triage label vocabulary and how the domain documents are read |
| [`prompts/`](prompts/) | Prompts for AI tools, each with its run status |
| [`design-system/`](design-system/) | The design system Tallyline as the repository holds it: token documentation, states, accessibility report; the tokens themselves are in `ui/design-tokens/`; the JavaFX parts are historical since ADR-0005, and the recreated egui crate lives in `crates/egui/` (O-112) |
| [`images/fankit/`](images/fankit/) | The Star Citizen Fan Kit logo files only; they are not under the project licence (`NOTICE` §4, `REUSE.toml`) |
| [`user/`](user/) | The user guide (R-DOC-1) and contributor corpus capture protocol; its images are redacted screenshots only |
| [`release-process.md`](release-process.md) | Versioning, release checklist, upgrade and downgrade, Star Citizen patch-day and UEX API-change procedures |
| [`game-log-validation.md`](game-log-validation.md) | Completed initial R-OCR-20 Game.log spike (issue #107): validated build/format and finite mapping allowlist, measured capture offsets, synthetic specimens and explicit unsupported scope; initial #108 parser/privacy scope, local header check and later integration obligations |
| [`dependency-pins.md`](dependency-pins.md) | The tags of SHA-pinned GitHub Actions and the versions and checksums of CI tools (including the pinned WiX and `cargo-packager`) |
| [`ci-release-reference.md`](ci-release-reference.md) | Reviewed P4KX build/release patterns, further M1–M5 applications, adaptation boundaries and upstream feedback |
| [`local-vision-runtime-research.md`](local-vision-runtime-research.md) | Source-backed comparison of bundled local VLM runtimes, complementary OCR readers, hardware tiers and evaluation plans; original research scope and later experimental evidence are distinguished |
| [`local-vision-first-campaign.md`](local-vision-first-campaign.md) | Completed 310-response local VLM experiment: exact inputs, structural coverage failures, separately labelled exploratory discrepancies and evidence limits; no validated accuracy or backend selection |
| [`local-vision-small-model-campaign.md`](local-vision-small-model-campaign.md) | Completed author-bound 2B CPU/GPU smoke and a separate literal-field development condition, with raw-value discrepancies, structural failures and independent cleanup evidence |
| [`local-vision-ministral3b-experiment.md`](local-vision-ministral3b-experiment.md) | Completed author-bound Ministral 3 3B development smoke: fixed-parser field concordance, paired 2B comparison, shared tab error, geometry and independent cleanup evidence |
| [`adr/0012-strictly-local-recognition-delivery.md`](adr/0012-strictly-local-recognition-delivery.md) | Owner-required local-only inference and complete initial recognition profiles; engine choice and the Windows OCR prerequisite remain under evaluation |
| [`cla-signatures.md`](cla-signatures.md) | The public roster of CLA signatures |

Diagrams are Mermaid or plain-text trees, never binary files. Images in `docs/` are only redacted user-guide screenshots and the Fan Kit logo files.
