# Tools

Developer tools that are never shipped with the app. Each subdirectory says what it holds and how to run it.

| Path | What belongs there |
|---|---|
| `ocr-eval/` | The offline metadata inspection and annotation CLI; recognition evaluation follows in M2 ([guide](../docs/user/corpus-capture.md)) |

New repository tooling is Rust ([ADR-0005](../docs/adr/0005-rust-and-egui.md) D12): checks and generators are tasks of an `xtask` crate once the workspace exists (M0). The Java programs and JavaFX spikes that converted, validated and contrast-checked the design tokens were removed on 2026-10-12; they stay in the Git history, and their `xtask` replacement is part of open point O-112. Nothing a tool generates is committed unless a document says so.
