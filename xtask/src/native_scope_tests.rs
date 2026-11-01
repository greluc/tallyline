// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Public metadata fixtures for the approved native restrictions.

use crate::native::violations;
use serde_json::{Value, json};

fn metadata(name: &str, version: &str, features: &[&str], reachable: bool) -> Value {
    json!({
        "packages": [
            {"id": "root", "name": "tallyline-app", "version": "0.0.0"},
            {"id": "restricted", "name": name, "version": version}
        ],
        "workspace_members": ["root"],
        "resolve": {"nodes": [
            {"id": "root", "features": [], "deps": if reachable {
                vec![json!({"name": "renamed", "pkg": "restricted"})]
            } else { vec![] }},
            {"id": "restricted", "features": features, "deps": []}
        ]}
    })
}

#[test]
fn enabled_wayland_log_is_rejected_through_a_renamed_dependency() {
    let input = metadata("wayland-backend", "0.3.17", &["log"], true);
    assert!(
        violations(&input.to_string())
            .unwrap()
            .iter()
            .any(|finding| finding.contains("wayland-backend 0.3.17") && finding.contains("log"))
    );
}

#[test]
fn excluded_packages_and_inactive_wayland_log_are_allowed() {
    for (name, version, features, reachable) in [
        ("wayland-backend", "0.3.17", vec!["client_system"], true),
        ("wayland-backend", "0.3.17", vec!["log"], false),
        ("android-activity", "0.6.1", vec!["game-activity"], false),
        ("objc-sys", "0.3.5", vec!["std"], false),
    ] {
        let input = metadata(name, version, &features, reachable);
        assert!(violations(&input.to_string()).unwrap().is_empty(), "{name}");
    }
}

#[test]
fn mobile_native_packages_leaking_into_a_supported_target_are_rejected() {
    for (name, version) in [("android-activity", "0.6.1"), ("objc-sys", "0.3.5")] {
        let input = metadata(name, version, &[], true);
        assert!(
            violations(&input.to_string())
                .unwrap()
                .iter()
                .any(|finding| finding.contains(name)),
            "{name}"
        );
    }
}

#[test]
fn transitive_native_activation_is_rejected_from_every_workspace_root() {
    let mut input = metadata("wayland-backend", "0.3.17", &["log"], true);
    input["packages"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id": "second", "name": "xtask", "version": "0.0.0"}));
    input["workspace_members"] = json!(["second"]);
    input["resolve"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id": "second", "features": [], "deps": [{"name": "app", "pkg": "root"}]}));
    assert_eq!(violations(&input.to_string()).unwrap().len(), 1);
}

#[test]
fn malformed_metadata_cannot_become_an_empty_selection() {
    for input in [
        "not json",
        "{}",
        r#"{"packages":[],"workspace_members":[],"resolve":{"nodes":[]}}"#,
    ] {
        assert!(violations(input).is_err(), "{input}");
    }
    let good = metadata("wayland-backend", "0.3.17", &[], true);
    for path in [
        "/packages/1/name",
        "/packages/1/version",
        "/packages/1/id",
        "/workspace_members/0",
        "/resolve/nodes/0/id",
        "/resolve/nodes/1/features",
        "/resolve/nodes/0/deps",
        "/resolve/nodes/0/deps/0/pkg",
        "/resolve/nodes/0/deps/0/name",
    ] {
        let mut input = good.clone();
        *input.pointer_mut(path).unwrap() = Value::Null;
        assert!(violations(&input.to_string()).is_err(), "{path}");
    }
}

#[test]
fn missing_graph_nodes_and_duplicate_ids_are_rejected() {
    let good = metadata("wayland-backend", "0.3.17", &[], true);
    for path in ["/packages", "/resolve/nodes"] {
        let mut input = good.clone();
        input
            .pointer_mut(path)
            .unwrap()
            .as_array_mut()
            .unwrap()
            .pop();
        assert!(violations(&input.to_string()).is_err(), "{path}");
        let mut input = good.clone();
        let values = input.pointer_mut(path).unwrap().as_array_mut().unwrap();
        values.push(values[0].clone());
        assert!(violations(&input.to_string()).is_err(), "{path}");
    }
}

#[test]
fn pure_rust_links_markers_do_not_imply_native_code() {
    for (name, version, links) in [
        ("rayon-core", "1.13.0", "rayon-core"),
        ("wasm-bindgen-shared", "0.2.130", "wasm_bindgen"),
    ] {
        let mut input = metadata(name, version, &[], true);
        input["packages"][1]["links"] = json!(links);
        assert!(violations(&input.to_string()).unwrap().is_empty());
    }
}

#[test]
fn declared_features_do_not_activate_native_shims() {
    let mut input = metadata("wayland-backend", "0.3.17", &[], true);
    input["packages"][1]["features"] = json!({"log": ["dep:log"]});
    assert!(violations(&input.to_string()).unwrap().is_empty());
}

#[test]
fn malformed_active_features_and_missing_workspace_roots_fail_closed() {
    let good = metadata("wayland-backend", "0.3.17", &[], true);
    for features in [json!("log"), json!([null]), json!([42]), json!([""])] {
        let mut input = good.clone();
        input["resolve"]["nodes"][1]["features"] = features;
        assert!(violations(&input.to_string()).is_err());
    }
    let mut input = good;
    input["workspace_members"] = json!(["missing"]);
    assert!(violations(&input.to_string()).is_err());
}

#[test]
fn packages_excluded_from_the_resolve_graph_are_allowed() {
    for (name, version) in [("android-activity", "0.6.1"), ("objc-sys", "0.3.5")] {
        let mut input = metadata(name, version, &[], false);
        input["resolve"]["nodes"].as_array_mut().unwrap().pop();
        assert!(violations(&input.to_string()).unwrap().is_empty());
    }
}

#[test]
fn android_properties_must_stay_outside_supported_graphs() {
    for version in ["0.2.2", "0.3.0"] {
        let excluded = metadata("android-properties", version, &[], false);
        assert!(violations(&excluded.to_string()).unwrap().is_empty());
        let reachable = metadata("android-properties", version, &["unexpected"], true);
        assert!(
            violations(&reachable.to_string())
                .unwrap()
                .iter()
                .any(|finding| finding.contains("android-properties"))
        );
    }
}

#[test]
fn apple_ffi_bindings_must_stay_outside_supported_graphs() {
    for (name, version) in [
        ("core-graphics", "0.23.2"),
        ("dispatch", "0.2.0"),
        ("core-graphics", "99.0.0"),
        ("dispatch", "99.0.0"),
    ] {
        let excluded = metadata(name, version, &[], false);
        assert!(
            violations(&excluded.to_string()).unwrap().is_empty(),
            "{name}"
        );
        let reachable = metadata(name, version, &["unexpected"], true);
        assert!(
            violations(&reachable.to_string())
                .unwrap()
                .iter()
                .any(|finding| finding.contains(name)),
            "{name}"
        );
    }
}
