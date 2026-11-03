# Design system

> **Doc type:** Living reference — draft (JavaFX handoff stage 2; JavaFX mechanism superseded by ADR-0005). Last reviewed: 2026-11-03.

> **Note 2026-10-12, updated 2026-11-01:** [ADR-0005](../adr/0005-rust-and-egui.md) replaces JavaFX with egui. Tallyline continues as the design system; its egui rendition (the crate `tallyline-egui`) replaces the JavaFX CSS handoff described below. The token values, state names and contrast results in this directory stay; the JavaFX-specific parts are marked where they occur. The crate was recreated for issue #32; remaining work is listed in "egui rendition (ADR-0005)" below.

The app's design system is **Tallyline** (D1; since 2026-10-12 also the name of the product, O-8; trademark search open). It was designed in Claude Design from the brief [`docs/prompts/design-system.md`](../prompts/design-system.md) its update [`design-system-update-2026-10.md`](../prompts/design-system-update-2026-10.md) and the correction brief [`design-system-update-2-2026-10.md`](../prompts/design-system-update-2-2026-10.md); the repository follows Design System version `1791542897-a5d3` and screens canvas `1791541917-46e6`. It is translated into the repository by the JavaFX handoff [`design-system-javafx-handoff.md`](../prompts/design-system-javafx-handoff.md). The decisions behind it are [ADR-0004](../adr/0004-design-system.md) (partly superseded by [ADR-0005](../adr/0005-rust-and-egui.md) on 2026-10-12: §3, §4, §6, §7 and the JavaFX spikes) and register point O-83.

| File | What belongs there |
|---|---|
| [`tokens.md`](tokens.md) | The token files, their schema, the conversion and validation rules and the JavaFX mapping |
| [`states.md`](states.md) | Every user-visible state, its state id and its pseudo-class or style class; open design defects |
| [`accessibility-report.md`](accessibility-report.md) | Contrast results on the token values; later the colour-vision, rendered-sheet and manual checks |
| [`components.md`](components.md) | Hash-bound supplied references, accepted overrides, component inventory and partial adoption status |

The Claude Design artifacts (system and screens canvas) are private references; the repository keeps no copy of them. A local export may sit in the gitignored `design-previews/`.

## egui rendition (ADR-0005)

Added 2026-10-12; implementation recreated 2026-11-01 for issue #32. [ADR-0005](../adr/0005-rust-and-egui.md) D6 keeps Tallyline and replaces its JavaFX CSS handoff by an egui rendition. The crate `tallyline-egui` v0.1.0 is in `crates/egui/`. Corrected 2026-11-01: it uses the eligible stable egui 0.37.0; the original recreation used 0.36.2.

**What the recreated crate provides:**
- resolved named colours for the dark, light and high-contrast themes;
- an egui `Style` mapping with theme, density and reduced-motion settings;
- nine confidence and deviation icons drawn with egui primitives;
- `IconButton`, `Chip`, `DeltaBadge`, `Field`, `Panel` and `Toast` widgets;
- a workspace test that checks the crate token snapshot against the DTCG source files.

**Not recreated yet:** the icon font, the remaining 147 icons, standalone `Icon` and `Button` widgets, `TableCell`, the interactive gallery and snapshot example, in-cell editing, navigation items, the window shell, DetailStrip, and screenshot overlays.

The original crate's egui limitations and contrast results have not been revalidated for this recreation. The state names in [states.md](states.md) remain the source for future component work.

**Findings and what @greluc decided on 2026-10-12 (O-112):**
1. Generators stay outside the repository; generated token and icon code is committed with the crate.
2. The planned modified Material Symbols font was unavailable with the crate source. The recreated crate draws its state icons with egui primitives and does not bundle a font.
3. About 30 plain `//` comments and many undocumented public items conflict with the code-comment rule and `#![deny(missing_docs)]`. *Decided:* cleaned up before the import (D11).
4. The licence in `Cargo.toml` is a placeholder (MIT OR Apache-2.0); the project licence is GPL-3.0-or-later and every file needs the licence header. *Decided:* fixed before the import.
5. The recreated crate carries a versioned `design/tokens.json`; the workspace reconciliation test compares its base and theme documents with `ui/design-tokens/`.
6. The Java tools under `tools/design-system/` were removed on 2026-10-12 (Git history keeps them); an `xtask` task replaces them in M0.
7. D1 to D11 of O-83 stand unchanged for the egui rendition.
8. The crate is maintained in `crates/egui/`; public Rust items carry documentation and source headers.

`ui/design-tokens/` remains the source of truth for token values.

## Product artwork source qualification — 2026-11-03

Corrected 2026-11-03: the supplied `Tallyline-Design-System.zip` and `tallyline-egui.zip` are reference exports, not a replacement for accepted requirements or ADRs. Their SHA-256 identities, extracted concept-C geometry, durable SVG/PNG/ICO exports and licence terms are recorded in [branding/](../../branding/README.md). This is the approved Line selection of ADR-0004 §1/O-83, retained by O-112; an undecided caption in the export does not reopen it. The product name remains Tallyline; working-title and JavaFX language in the archives is historical.

The app uses the fixed dark-colour icon for native window identity in every theme. Debian launcher metadata and Windows shortcut/installed-programmes authoring use the same project artwork. The supplied AppShell reserves its header for navigation and actions and explicitly excludes a product logo; it receives none. Fan Kit artwork remains its coupled legal unit in the approved placements and is never used as product branding. No archive code or generators are executed. Native and rebuilt-package acceptance are recorded separately from source and headless contracts.
Corrected 2026-11-03: both archives are available. [Component adoption](components.md) records source identities, accepted overrides and remaining work. This foundation registers 20 type styles and five approved weights, corrects scrim alpha and existing widget geometry/semantic colours, and adopts exact numeric review presentation. The icon font, full shell, review table and screen composition remain separate work.
