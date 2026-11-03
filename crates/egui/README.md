<!-- SPDX-FileCopyrightText: 2026 Lucas Greuloch -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# `tallyline-egui`

Tallyline theme tokens and reusable widgets for egui 0.37.0.

The crate currently provides dark, light and high-contrast colour palettes;
theme, density and reduced-motion settings; nine state icons drawn with egui
primitives; and the `IconButton`, `Chip`, `DeltaBadge`, `Field`, `Panel` and
`Toast` widgets.

```rust,ignore
use tallyline_egui::{apply_theme, Density, Motion, Theme};

apply_theme(ctx, Theme::Dark, Density::Comfortable, Motion::Full);
```

`design/tokens.json` is a snapshot of `ui/design-tokens/`. The workspace test
`tallyline_egui_tokens_match_the_design_system_sources` compares the snapshot
with those DTCG source files. `src/tokens.rs` contains the generated resolved
colour tables. Token and icon generation sources are maintained outside this
repository; only generated Rust and the reconciled JSON snapshot are committed.

The earlier delivered crate and its modified Material Symbols subset were not
available when this crate was recreated. The icon font is not included. The
replacement state icons are hand-drawn egui primitives; pixel-grid and
magnified wgpu review of the original font therefore remain inapplicable to
this implementation and the full icon inventory is not recreated yet.

Corrected 2026-11-03: see [component adoption](../../docs/design-system/components.md) for the supplied archive identities and partial typography/widget adoption. All 20 named type styles and five approved face weights are installed through the separate-file desktop loader; native visual acceptance remains open.
