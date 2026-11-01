# crates

The workspace crates of Tallyline: the core (`shared-kernel`, the context crates, `workflows`), the adapters, `ui` and `app` of `docs/plan/02-architecture.md` §2. Each directory is one crate; its package name is the directory name with the prefix `tallyline-` (O-122), and its purpose is the crate-level `//!` documentation of its `lib.rs`. A crate is created with its first test (04 M0). `xtask/` and `tools/` are not here.

| Directory | Package | State |
|---|---|---|
| `adapter-uex/` | `tallyline-adapter-uex` | the read-only UEX client of the M0 API spike and its CLI `tallyline-uex-spike` (issue #24) |
| `egui/` | `tallyline-egui` | Tallyline theme tokens and reusable widgets |
| `workflows/` | `tallyline-workflows` | application preferences, save-before-apply workflows and restart-safe daily release policy |
| `adapter-files/` | `tallyline-adapter-files` | versioned atomic settings, bounded read-only game witness parsing and installed/portable directory self-tests |
| `ui/` | `tallyline-ui` | owned shell state, message catalogue, legal unit and user-driven dialogs |
| `app/` | `tallyline-app` | native `tallyline` composition root and bounded settings/release workers |
| `adapter-platform/` | `tallyline-adapter-platform` | two-lock writer election, bounded local activation and credential-free release metadata; the three approved Windows FFI sites |

| `shared-kernel/` | `tallyline-shared-kernel` | precise timestamps, injected clock and shared numeric IDs |
| `reference-data/` | `tallyline-reference-data` | initial cached commodity application seam; adapter storage remains separate work |
| `game/` | `tallyline-game` | read-only witness port, typed facts and durable opt-in workflow |
