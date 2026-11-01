# crates

The workspace crates of Tallyline: the core (`shared-kernel`, the context crates, `workflows`), the adapters, `ui` and `app` of `docs/plan/02-architecture.md` §2. Each directory is one crate; its package name is the directory name with the prefix `tallyline-` (O-122), and its purpose is the crate-level `//!` documentation of its `lib.rs`. A crate is created with its first test (04 M0). `xtask/` and `tools/` are not here.

| Directory | Package | State |
|---|---|---|
| `adapter-uex/` | `tallyline-adapter-uex` | the read-only UEX client of the M0 API spike and its CLI `tallyline-uex-spike` (issue #24) |
| `egui/` | `tallyline-egui` | Tallyline theme tokens and reusable widgets |
| `workflows/` | `tallyline-workflows` | application preferences and save-before-apply workflows |
| `adapter-files/` | `tallyline-adapter-files` | versioned atomic settings and installed/portable directory self-tests |
| `ui/` | `tallyline-ui` | owned shell state, message catalogue, legal unit and user-driven dialogs |
| `app/` | `tallyline-app` | native `tallyline` composition root and bounded settings worker |
| `adapter-platform/` | `tallyline-adapter-platform` | two-lock writer election and bounded local activation; the three approved Windows FFI sites |
