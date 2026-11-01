// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keeps the crate's design token snapshot aligned with the DTCG source files.

use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[test]
fn tallyline_egui_tokens_match_the_design_system_sources() -> Result<(), Box<dyn std::error::Error>>
{
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let generated: Value =
        serde_json::from_slice(&fs::read(root.join("crates/egui/design/tokens.json"))?)?;
    let base: Value =
        serde_json::from_slice(&fs::read(root.join("ui/design-tokens/base.tokens.json"))?)?;
    assert_eq!(generated.get("base"), Some(&base));
    for (theme, file) in [
        ("dark", "theme-dark.tokens.json"),
        ("light", "theme-light.tokens.json"),
        ("high-contrast", "theme-high-contrast.tokens.json"),
    ] {
        let source: Value =
            serde_json::from_slice(&fs::read(root.join("ui/design-tokens").join(file))?)?;
        assert_eq!(
            generated.get("themes").and_then(|themes| themes.get(theme)),
            Some(&source),
            "{theme} token snapshot drifted"
        );
    }
    Ok(())
}
