// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Strict source-independent verification of sealed packaging inputs.

use super::Target;
use super::manifest::{Entry, MANIFEST, Role, Verified};
use serde_json::Value;
use std::fs;
use std::io;
use std::path::Path;
pub(super) fn read(stage: &Path, seal: &str) -> io::Result<Verified> {
    inventory(stage, seal)
}

fn inventory(stage: &Path, seal: &str) -> io::Result<Verified> {
    use std::collections::{BTreeMap, BTreeSet};
    if !fs::symlink_metadata(stage)?.file_type().is_dir()
        || !fs::symlink_metadata(stage.join(MANIFEST))?
            .file_type()
            .is_file()
    {
        return Err(io::Error::other(
            "packaging stage or manifest is not a regular input",
        ));
    }
    let bytes = fs::read(stage.join(MANIFEST))?;
    if !hash_is_valid(seal) || crate::hashlist::sha256_hex(&bytes) != seal {
        return Err(io::Error::other("packaging manifest seal mismatch"));
    }
    let value: Value = serde_json::from_slice(&bytes)?;
    exact_fields(&value, &["schema_version", "target", "version", "files"])?;
    if value.get("schema_version").and_then(Value::as_u64) != Some(1) {
        return Err(io::Error::other("unsupported packaging manifest schema"));
    }
    let target = match value.get("target").and_then(Value::as_str) {
        Some("x86_64-pc-windows-msvc") => Target::Windows,
        Some("x86_64-unknown-linux-gnu") => Target::Linux,
        _ => return Err(io::Error::other("unsupported packaging target")),
    };
    let version = text(&value, "version")?.to_owned();
    let mut files = Vec::new();
    let mut names = BTreeSet::new();
    for item in value
        .get("files")
        .and_then(Value::as_array)
        .ok_or_else(|| io::Error::other("invalid sealed file inventory"))?
    {
        exact_fields(item, &["path", "sha256", "mode", "role"])?;
        let path = text(item, "path")?.to_owned();
        if path.is_empty()
            || path.contains(['\\', ':'])
            || Path::new(&path)
                .components()
                .any(|part| !matches!(part, std::path::Component::Normal(_)))
            || !names.insert(path.to_lowercase())
        {
            return Err(io::Error::other("unsafe or duplicate sealed path"));
        }
        let role = match text(item, "role")? {
            "payload" if path.starts_with("payload/") => Role::Payload,
            "tool" if path.starts_with("tools/") => Role::Tool,
            "helper" if path == target.helper() => Role::Helper,
            _ => return Err(io::Error::other("invalid sealed file role")),
        };
        let mode = item["mode"]
            .as_u64()
            .and_then(|mode| u32::try_from(mode).ok())
            .filter(|mode| matches!(mode, 0o644 | 0o755))
            .ok_or_else(|| io::Error::other("invalid sealed file mode"))?;
        let sha256 = text(item, "sha256")?.to_owned();
        if !hash_is_valid(&sha256) {
            return Err(io::Error::other("invalid sealed file hash"));
        }
        files.push(Entry {
            path,
            sha256,
            mode,
            role,
        });
    }
    let description = Verified {
        target,
        version,
        files,
    };
    super::release_version(&format!("v{}", description.version))?;
    super::tools::verify(description.target.name(), &stage.join("tools"))?;
    let mut expected = BTreeMap::new();
    let mut directories = BTreeSet::new();
    for file in &description.files {
        validate_role(file, description.target)?;
        let path = Path::new(&file.path);
        expected.insert(path.to_owned(), file.sha256.clone());
        for ancestor in path.ancestors().skip(1) {
            directories.insert(ancestor.to_owned());
        }
    }
    for required in [
        format!("payload/{}", description.target.executable()),
        format!("payload/{}", description.target.runtime()),
        "payload/LICENSE".into(),
        "payload/NOTICE".into(),
        "payload/README.md".into(),
        "payload/legal/catalogue.json".into(),
        format!("tools/{}", description.target.tool()),
        description.target.helper().into(),
    ] {
        if !expected.contains_key(Path::new(&required)) {
            return Err(io::Error::other(
                "sealed packaging input is missing a required file",
            ));
        }
    }
    expected.insert(MANIFEST.into(), seal.into());
    check_tree(stage, stage, &expected, &directories)?;
    for file in &description.files {
        let path = stage.join(&file.path);
        if crate::hashlist::sha256_hex(&fs::read(&path)?) != file.sha256 {
            return Err(io::Error::other("sealed packaging file has changed"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if fs::metadata(&path)?.permissions().mode() & 0o777 != file.mode {
                return Err(io::Error::other("sealed packaging permission has changed"));
            }
        }
    }
    super::payload::verify(&stage.join("payload"), description.target)?;
    let helper = description
        .files
        .iter()
        .find(|file| file.role == Role::Helper)
        .ok_or_else(|| io::Error::other("sealed helper is missing"))?;
    if crate::hashlist::sha256_hex(&fs::read(std::env::current_exe()?)?) != helper.sha256 {
        return Err(io::Error::other(
            "running packaging helper does not match the seal",
        ));
    }
    Ok(description)
}

fn validate_role(file: &Entry, target: Target) -> io::Result<()> {
    if file.role == Role::Payload {
        if super::payload::forbidden_runtime(Path::new(&file.path)) {
            return Err(io::Error::other(
                "sealed application contains a prohibited Microsoft runtime file",
            ));
        }
        let relative = file
            .path
            .strip_prefix("payload/")
            .ok_or_else(|| io::Error::other("invalid payload path"))?;
        let root = relative.split('/').next().unwrap_or_default();
        if ![
            target.executable(),
            target.runtime(),
            "assets",
            "legal",
            "LICENSE",
            "NOTICE",
            "README.md",
        ]
        .contains(&root)
        {
            return Err(io::Error::other("unapproved payload root"));
        }
    }
    if file.role == Role::Tool
        && Path::new(&file.path).components().any(|part| {
            let name = part.as_os_str().to_string_lossy().to_lowercase();
            [
                "sdk",
                "src",
                ".git",
                ".nuget",
                "cargo",
                "cargo.exe",
                "rustc",
                "rustc.exe",
                "cl.exe",
                "msbuild.exe",
                "nuget.exe",
                "mergemod.dll",
            ]
            .contains(&name.as_str())
        })
    {
        return Err(io::Error::other(
            "unapproved compiler, source or proprietary tool component",
        ));
    }
    if (file.role == Role::Helper
        || file.path == format!("payload/{}", target.executable())
        || file.path == format!("tools/{}", target.tool()))
        && file.mode != 0o755
    {
        return Err(io::Error::other(
            "sealed executable is missing execute permission",
        ));
    }
    Ok(())
}

fn check_tree(
    root: &Path,
    path: &Path,
    expected: &std::collections::BTreeMap<std::path::PathBuf, String>,
    directories: &std::collections::BTreeSet<std::path::PathBuf>,
) -> io::Result<()> {
    let relative = path.strip_prefix(root).map_err(io::Error::other)?;
    let kind = fs::symlink_metadata(path)?.file_type();
    if kind.is_symlink() {
        return Err(io::Error::other(
            "sealed packaging tree contains a symbolic link",
        ));
    }
    if kind.is_dir() && (relative.as_os_str().is_empty() || directories.contains(relative)) {
        for child in fs::read_dir(path)? {
            check_tree(root, &child?.path(), expected, directories)?;
        }
    } else if !kind.is_file() || !expected.contains_key(relative) {
        return Err(io::Error::other(
            "sealed packaging tree contains an unlisted input",
        ));
    }
    Ok(())
}

fn exact_fields(value: &Value, fields: &[&str]) -> io::Result<()> {
    if !value.as_object().is_some_and(|object| {
        object.len() == fields.len() && fields.iter().all(|field| object.contains_key(*field))
    }) {
        return Err(io::Error::other(
            "unknown or missing packaging manifest field",
        ));
    }
    Ok(())
}

fn text<'a>(value: &'a Value, field: &str) -> io::Result<&'a str> {
    value[field]
        .as_str()
        .ok_or_else(|| io::Error::other("invalid packaging manifest field type"))
}

fn hash_is_valid(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
