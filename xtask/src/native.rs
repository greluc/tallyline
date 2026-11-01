// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Checks the approved target exclusions and disabled native shim feature.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// Checks full, target-filtered Cargo metadata for the approved native restrictions.
///
/// # Errors
/// Returns an error when the metadata cannot establish a complete reachable graph.
pub fn violations(metadata: &str) -> Result<Vec<String>, String> {
    let document: Value = serde_json::from_str(metadata).map_err(|error| error.to_string())?;
    let packages = indexed(array(&document, "packages")?)?;
    let resolve = document
        .get("resolve")
        .ok_or("missing metadata resolve graph")?;
    let nodes = indexed(array(resolve, "nodes")?)?;
    let roots = array(&document, "workspace_members")?;
    if roots.is_empty() {
        return Err("cargo metadata has no workspace members".to_owned());
    }
    let mut pending = roots.iter().map(string).collect::<Result<Vec<_>, _>>()?;
    let mut visited = BTreeSet::new();
    let mut findings = Vec::new();
    while let Some(id) = pending.pop() {
        if !visited.insert(id) {
            continue;
        }
        let package = packages
            .get(id)
            .ok_or_else(|| format!("missing package {id}"))?;
        let node = nodes
            .get(id)
            .ok_or_else(|| format!("missing resolve node {id}"))?;
        let name = field(package, "name")?;
        let version = field(package, "version")?;
        let features = array(node, "features")?
            .iter()
            .map(string)
            .collect::<Result<Vec<_>, _>>()?;
        if matches!(
            name,
            "android-activity"
                | "objc-sys"
                | "android-properties"
                | "core-graphics"
                | "dispatch"
                | "system-configuration"
                | "system-configuration-sys"
        ) {
            findings.push(format!("{name} {version} must be absent from the supported target graph; new owner approval is required"));
        }
        if name == "rust_decimal" && !features.is_empty() {
            findings.push(format!("{name} {version} must keep all default and optional features disabled; source/purity review is required"));
        }
        if name == "wayland-backend" && features.contains(&"log") {
            findings.push(format!("{name} {version} feature log enables native C shims; new owner approval is required"));
        }
        for dependency in array(node, "deps")? {
            field(dependency, "name")?;
            pending.push(field(dependency, "pkg")?);
        }
    }
    Ok(findings)
}

fn indexed(values: &[Value]) -> Result<BTreeMap<&str, &Value>, String> {
    let mut indexed = BTreeMap::new();
    for value in values {
        let id = field(value, "id")?;
        if indexed.insert(id, value).is_some() {
            return Err(format!("duplicate metadata ID {id}"));
        }
    }
    Ok(indexed)
}

fn array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value], String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("missing or malformed metadata array {key}"))
}

fn field<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .ok_or_else(|| format!("missing metadata field {key}"))
        .and_then(string)
}

fn string(value: &Value) -> Result<&str, String> {
    value
        .as_str()
        .filter(|text| !text.is_empty())
        .ok_or_else(|| "metadata string is empty or malformed".to_owned())
}
