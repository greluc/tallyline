// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Public approval-boundary fixtures for literal inline assembly.

use crate::native_assembly::{SourceFile, violations};
use serde_json::{Value, json};

fn metadata() -> Value {
    json!({"packages": [{"id":"root", "name":"app", "version":"0.0.0", "source":null},
        {"id":"dep", "name":"sample", "version":"1.0.0", "source":crate::lockfile::CRATES_IO}],
        "workspace_members":["root"], "resolve":{"nodes":[
            {"id":"root","features":[],"deps":[{"pkg":"dep"}]},
            {"id":"dep","features":["std"],"deps":[]}]}})
}

fn review() -> Value {
    json!({"schema":1,"status":"approved","revisit_by":"2027-04-13",
        "targets":["windows","linux"],"packages":[{"name":"sample","version":"1.0.0",
        "archive_sha256":"a".repeat(64),"full_features":["std"],
        "targets":{"windows":["std"],"linux":["std"]},
        "files":{"src/lib.rs":"b".repeat(64)}}]})
}

fn findings(metadata: &Value, review: &Value, files: &[SourceFile]) -> Result<Vec<String>, String> {
    let lock = format!(
        "[[package]]\nname = \"sample\"\nversion = \"1.0.0\"\nsource = \"{}\"\nchecksum = \"{}\"\n",
        crate::lockfile::CRATES_IO,
        "a".repeat(64)
    );
    let targets = ["windows", "linux"]
        .map(|target| (target.to_owned(), metadata.to_string()))
        .into();
    violations(
        &metadata.to_string(),
        &lock,
        &review.to_string(),
        files,
        &targets,
    )
}

fn files() -> Vec<SourceFile> {
    vec![SourceFile {
        package_id: "dep".to_owned(),
        path: "src/lib.rs".to_owned(),
        sha256: "b".repeat(64),
    }]
}

#[test]
fn exact_approved_scope_passes() {
    assert!(
        findings(&metadata(), &review(), &files())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn unknown_package_version_hash_and_forbidden_feature_fail() {
    for field in ["name", "version"] {
        let mut input = metadata();
        input["packages"][1][field] = json!("unreviewed");
        assert!(!findings(&input, &review(), &files()).unwrap().is_empty());
    }
    let mut wrong_archive = review();
    wrong_archive["packages"][0]["archive_sha256"] = json!("c".repeat(64));
    assert!(
        !findings(&metadata(), &wrong_archive, &files())
            .unwrap()
            .is_empty()
    );
    let mut changed = files();
    changed[0].sha256 = "c".repeat(64);
    assert!(
        !findings(&metadata(), &review(), &changed)
            .unwrap()
            .is_empty()
    );
    let mut input = metadata();
    input["resolve"]["nodes"][1]["features"] = json!(["std", "hardware-lock-elision"]);
    assert!(!findings(&input, &review(), &files()).unwrap().is_empty());
}

#[test]
fn proposed_missing_and_stale_reviews_fail_closed() {
    let mut proposed = review();
    proposed["status"] = json!("proposed");
    assert!(
        !findings(&metadata(), &proposed, &files())
            .unwrap()
            .is_empty()
    );
    assert!(!findings(&metadata(), &review(), &[]).unwrap().is_empty());
    let mut extra = files();
    extra.push(SourceFile {
        package_id: "dep".to_owned(),
        path: "src/new.rs".to_owned(),
        sha256: "b".repeat(64),
    });
    assert!(!findings(&metadata(), &review(), &extra).unwrap().is_empty());
    proposed = review();
    proposed["packages"] = json!([]);
    assert!(
        !findings(&metadata(), &proposed, &files())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn excluded_scope_cannot_become_reachable() {
    let mut approved = review();
    approved["packages"][0]["targets"]["windows"] = Value::Null;
    assert!(
        !findings(&metadata(), &approved, &files())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn cached_source_collection_is_conservative_and_hash_bound() {
    let scratch = crate::scratch::Scratch::new();
    scratch.write("Cargo.toml", "[package]");
    scratch.write("src/a.rs", "core::arch::asm \n ! (\"nop\");");
    scratch.write("src/b.rs", "// global_asm ! is deliberately included");
    scratch.write("src/c.rs", "some_asm!(); assembly();");
    let mut input = metadata();
    input["packages"][1]["manifest_path"] = json!(scratch.path().join("Cargo.toml"));
    let found = crate::native_assembly::source_files(&input.to_string()).unwrap();
    assert_eq!(found.len(), 2);
    assert!(found.iter().any(|file| file.path == "src/a.rs"
        && file.sha256 == crate::hashlist::sha256_hex(b"core::arch::asm \n ! (\"nop\");")));
}

#[test]
fn expired_or_malformed_scope_cannot_pass() {
    let mut expired = review();
    expired["revisit_by"] = json!("1970-01-01");
    assert!(findings(&metadata(), &expired, &files()).is_err());
    assert!(violations("{}", "", "{}", &[], &Default::default()).is_err());
}
