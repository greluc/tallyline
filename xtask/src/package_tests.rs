// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Public packaging adapter contracts against independent archive readers.

#![expect(
    non_snake_case,
    reason = "requirement traceability uses double underscores"
)]

use std::process::Command;

#[test]
fn r_nf_2_q_installers__release_tags_have_one_canonical_msi_compatible_identity() {
    for (tag, expected) in [("v0.1.0", "0.1.0"), ("v255.255.65535", "255.255.65535")] {
        assert_eq!(crate::package::release_version(tag).unwrap(), expected);
    }
    for tag in [
        "0.1.0",
        "v01.2.3",
        "v1.02.3",
        "v1.2.03",
        "v+1.2.3",
        "v1.2.3.4",
        "v1.2.3-rc.1",
        "v1.2.3+build",
        "v256.0.0",
        "v0.256.0",
        "v0.0.65536",
        "v1.2.3\n",
    ] {
        assert!(
            crate::package::release_version(tag).is_err(),
            "accepted {tag:?}"
        );
    }
}

#[test]
fn r_nf_2_q_installers__archives_extract_with_an_independent_reader() {
    let root = std::env::temp_dir().join(format!("tallyline-package-cli-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let input = root.join("input");
    std::fs::create_dir_all(input.join("assets")).unwrap();
    std::fs::write(
        input.join("assets/licence.txt"),
        b"Independent archive fixture\n",
    )
    .unwrap();
    for format in ["zip", "tar.gz"] {
        let destination = root.join(format!("application.{format}"));
        crate::package::run(&[
            "archive".into(),
            format.into(),
            input.display().to_string(),
            destination.display().to_string(),
        ])
        .unwrap();
        let reader = if format == "zip" && !cfg!(windows) {
            "unzip"
        } else {
            "tar"
        };
        let extract_argument = if reader == "unzip" { "-p" } else { "-xOf" };
        let extracted = Command::new(reader)
            .arg(extract_argument)
            .arg(&destination)
            .arg("tallyline/assets/licence.txt")
            .output()
            .unwrap();
        assert!(extracted.status.success());
        assert_eq!(extracted.stdout, b"Independent archive fixture\n");
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn r_sec_8__unapproved_packaging_tools_are_rejected_without_execution() {
    let root = std::env::temp_dir().join(format!("tallyline-package-tools-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    for name in ["wix.exe", "cargo-packager"] {
        std::fs::write(root.join(name), b"unapproved tool fixture").unwrap();
    }
    for target in ["windows", "linux"] {
        let error = crate::package::run(&[
            "tools-verify".into(),
            target.into(),
            root.display().to_string(),
        ])
        .unwrap_err();
        assert!(error.to_string().contains("approved packaging tool"));
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn r_nf_4__token_scanning_catches_bare_and_wide_values_without_disclosing_them() {
    let root = std::env::temp_dir().join(format!("tallyline-package-scan-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let payload = root.join("payload");
    std::fs::create_dir(&payload).unwrap();
    let token = format!(
        "synthetic-opaque-{}-fixture-with+punctuation/and=padding",
        std::process::id()
    );
    let fixture = root.join("fixture");
    std::fs::write(&fixture, &token).unwrap();
    std::fs::write(payload.join("tallyline-app-token"), &token).unwrap();
    let scan = || {
        crate::package::run(&[
            "scan".into(),
            payload.display().to_string(),
            fixture.display().to_string(),
            "tallyline-app-token".into(),
        ])
    };
    assert!(scan().is_ok());
    for representation in [
        token.as_bytes().to_vec(),
        token.encode_utf16().flat_map(u16::to_le_bytes).collect(),
        token.encode_utf16().flat_map(u16::to_be_bytes).collect(),
    ] {
        std::fs::write(payload.join("intermediate.bin"), &representation).unwrap();
        let error = scan().unwrap_err().to_string();
        assert!(!error.contains(&token));
        assert!(error.contains("token boundary violation"));
        std::fs::remove_file(payload.join("intermediate.bin")).unwrap();
    }
    std::fs::remove_file(payload.join("tallyline-app-token")).unwrap();
    assert!(scan().is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn r_nf_2_q_installers__archive_failures_leave_no_partial_or_replaced_output() {
    let root =
        std::env::temp_dir().join(format!("tallyline-package-failure-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let input = root.join("input");
    std::fs::create_dir(&input).unwrap();
    std::fs::write(input.join("a".repeat(101)), b"too-long USTAR basename").unwrap();
    let output = root.join("partial.tar.gz");
    assert!(crate::package::write_archive("tar.gz", &input, &output).is_err());
    assert!(!output.exists());
    std::fs::write(&output, b"existing artefact").unwrap();
    assert!(crate::package::write_archive("zip", &input, &output).is_err());
    assert_eq!(std::fs::read(&output).unwrap(), b"existing artefact");
    let nested = input.join("recursive.zip");
    assert!(crate::package::write_archive("zip", &input, &nested).is_err());
    assert!(!nested.exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn r_nf_2_q_installers__linux_names_cannot_become_windows_parent_paths() {
    let root = std::env::temp_dir().join(format!("tallyline-package-path-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let input = root.join("input");
    std::fs::create_dir(&input).unwrap();
    std::fs::write(
        input.join("..\\escaped.txt"),
        b"must never escape the extraction root",
    )
    .unwrap();
    for format in ["zip", "tar.gz"] {
        let output = root.join(format!("application.{format}"));
        assert!(
            crate::package::write_archive(format, &input, &output).is_err(),
            "accepted a Windows parent-path entry"
        );
        assert!(!output.exists(), "left a rejected archive output");
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn r_nf_2_q_installers__archives_refuse_case_aliases_before_windows_extraction() {
    let root = std::env::temp_dir().join(format!("tallyline-package-case-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let input = root.join("input");
    std::fs::create_dir(&input).unwrap();
    std::fs::write(input.join("NOTICE"), b"first file").unwrap();
    std::fs::write(input.join("notice"), b"second distinct Linux file").unwrap();
    for format in ["zip", "tar.gz"] {
        let output = root.join(format!("application.{format}"));
        assert!(
            crate::package::write_archive(format, &input, &output).is_err(),
            "accepted Windows case aliases"
        );
        assert!(!output.exists());
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn r_nf_2_q_installers__archives_refuse_windows_device_and_normalised_names() {
    let root =
        std::env::temp_dir().join(format!("tallyline-package-device-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    for (index, name) in [
        "NUL.txt",
        "COM1",
        "LPT¹.log",
        "report.",
        "report ",
        "drive:note",
        "note?",
        "note\u{1}",
    ]
    .iter()
    .enumerate()
    {
        let input = root.join(format!("input-{index}"));
        std::fs::create_dir(&input).unwrap();
        std::fs::write(input.join(name), b"unsafe Windows name").unwrap();
        for format in ["zip", "tar.gz"] {
            let output = root.join(format!("application-{index}.{format}"));
            assert!(
                crate::package::write_archive(format, &input, &output).is_err(),
                "accepted reserved or normalised name"
            );
            assert!(!output.exists());
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn r_sec_8_r_nf_2_q_installers__prepare_rejects_unapproved_tools_before_creating_a_stage() {
    let directory = crate::scratch::Scratch::new();
    let input = directory.path().join("input");
    let tools = directory.path().join("tools");
    let stage = directory.path().join("stage");
    std::fs::create_dir(&input).unwrap();
    std::fs::create_dir(&tools).unwrap();
    let (target, tool) = if cfg!(windows) {
        ("windows", "wix.exe")
    } else {
        ("linux", "cargo-packager")
    };
    std::fs::write(tools.join(tool), b"unapproved tool bytes").unwrap();
    let error = crate::package::run(&[
        "prepare".into(),
        target.into(),
        "v0.1.0".into(),
        input.display().to_string(),
        tools.display().to_string(),
        stage.display().to_string(),
    ])
    .unwrap_err();
    assert!(error.to_string().contains("approved packaging tool"));
    assert!(!stage.exists());
}

#[test]
fn r_doc_3__legal_inventory_binds_notice_bytes_and_refuses_extra_files() {
    let directory = crate::scratch::Scratch::new();
    directory.write(
        "legal/crates/fixture-1.0.0/LICENSE",
        "Known original licence fixture\n",
    );
    directory.write(
        "legal/crates/index.tsv",
        "crate\tversion\tdeclared\tdetected\nfixture\t1.0.0\tMIT\tMIT\n",
    );
    directory.write("legal/catalogue.json",r#"{"schema_version":1,"components":[{"name":"fixture","version":"1.0.0","declared_licence":"MIT","licence":"MIT","source":"https://example.invalid/fixture","copyright":{"status":"provided","notices":["Synthetic fixture author"]},"notices":[{"path":"crates/fixture-1.0.0/LICENSE","text":"Known original licence fixture\n"}]}]}"#);
    let arguments = [
        "legal-verify".into(),
        directory.path().display().to_string(),
    ];
    crate::package::run(&arguments).unwrap();
    let profile_error = crate::package::run(&[
        "legal-verify".into(),
        "windows".into(),
        directory.path().display().to_string(),
    ])
    .unwrap_err();
    assert!(
        profile_error
            .to_string()
            .contains("approved target legal catalogue")
    );
    directory.write(
        "legal/crates/fixture-1.0.0/LICENSE",
        "Changed attribution text\n",
    );
    assert!(crate::package::run(&arguments).is_err());
    directory.write(
        "legal/crates/fixture-1.0.0/LICENSE",
        "Known original licence fixture\n",
    );
    directory.write("legal/unapproved.exe", "unexpected binary-shaped payload");
    assert!(crate::package::run(&arguments).is_err());
}

#[test]
fn r_sec_8__a_wrong_seal_fails_before_any_tool_inventory_is_trusted() {
    let directory = crate::scratch::Scratch::new();
    directory.write("package-manifest.json", "{}");
    let error = crate::package::verify_stage(directory.path(), &"0".repeat(64)).unwrap_err();
    assert!(error.to_string().contains("seal mismatch"));
}

#[test]
fn r_doc_3__source_inventory_maps_real_unpacked_registry_source() {
    let directory = crate::scratch::Scratch::new();
    directory.write(
        "payload/legal/crates/fixture-1.0.0/LICENSE",
        "Known original licence fixture\n",
    );
    directory.write(
        "payload/legal/crates/index.tsv",
        "crate\tversion\tdeclared\tdetected\nfixture\t1.0.0\tMIT\tMIT\n",
    );
    directory.write("payload/legal/catalogue.json", r#"{"schema_version":1,"components":[{"name":"fixture","version":"1.0.0","declared_licence":"MIT","licence":"MIT","source":"https://example.invalid/fixture","copyright":{"status":"provided","notices":["Synthetic fixture author"]},"notices":[{"path":"crates/fixture-1.0.0/LICENSE","text":"Known original licence fixture\n"}]}]}"#);
    directory.write("source/vendor/fixture-1.0.0/src/lib.rs", "abc");
    let archive = directory.path().join("source.tar.gz");
    crate::package::write_archive("source.tar.gz", &directory.path().join("source"), &archive)
        .unwrap();
    let map = directory.path().join("source-map.json");
    crate::package::run(&[
        "source-map".into(),
        directory.path().join("payload").display().to_string(),
        directory.path().join("source").display().to_string(),
        archive.display().to_string(),
        map.display().to_string(),
    ])
    .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(map).unwrap()).unwrap();
    assert_eq!(
        value["components"][0]["files"][0]["path"],
        "vendor/fixture-1.0.0/src/lib.rs"
    );
    assert_eq!(
        value["components"][0]["files"][0]["sha256"],
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let unpacked = Command::new("tar")
        .current_dir(directory.path())
        .args(["-xOf"])
        .arg("source.tar.gz")
        .arg("tallyline/vendor/fixture-1.0.0/src/lib.rs")
        .output()
        .unwrap();
    assert!(
        unpacked.status.success(),
        "{}",
        String::from_utf8_lossy(&unpacked.stderr)
    );
    assert_eq!(unpacked.stdout, b"abc");
    let map_bytes = std::fs::read(directory.path().join("source-map.json")).unwrap();
    std::fs::write(
        directory.path().join("payload/legal/source-map.json"),
        &map_bytes,
    )
    .unwrap();
    let verify_arguments = [
        "source-verify".into(),
        directory.path().join("payload").display().to_string(),
        archive.display().to_string(),
    ];
    crate::package::run(&verify_arguments).unwrap();
    let mut changed_map: serde_json::Value = serde_json::from_slice(&map_bytes).unwrap();
    changed_map["archive"]["sha256"] = "0".repeat(64).into();
    std::fs::write(
        directory.path().join("payload/legal/source-map.json"),
        serde_json::to_vec(&changed_map).unwrap(),
    )
    .unwrap();
    assert!(crate::package::run(&verify_arguments).is_err());
    for (field, value) in [
        ("path", "vendor/fixture-1.0.0/src/missing.rs".to_owned()),
        ("sha256", "0".repeat(64)),
    ] {
        let mut incorrect: serde_json::Value = serde_json::from_slice(&map_bytes).unwrap();
        incorrect["components"][0]["files"][0][field] = value.into();
        std::fs::write(
            directory.path().join("payload/legal/source-map.json"),
            serde_json::to_vec(&incorrect).unwrap(),
        )
        .unwrap();
        assert!(
            crate::package::run(&verify_arguments).is_err(),
            "source map file evidence differs from actual archive"
        );
    }
    std::fs::write(
        directory.path().join("payload/legal/source-map.json"),
        map_bytes,
    )
    .unwrap();
    directory.write(
        "source/vendor/fixture-1.0.0/src/lib.rs",
        "changed after archive freeze",
    );
    let mismatched = directory.path().join("mismatched-source-map.json");
    assert!(
        crate::package::run(&[
            "source-map".into(),
            directory.path().join("payload").display().to_string(),
            directory.path().join("source").display().to_string(),
            archive.display().to_string(),
            mismatched.display().to_string(),
        ])
        .is_err(),
        "source archive and unpacked source mismatch was accepted"
    );
    assert!(!mismatched.exists());
}

#[test]
fn r_doc_3_r_nf_2_q_installers__source_archives_have_data_only_file_permissions() {
    let directory = crate::scratch::Scratch::new();
    directory.write("source/source.exe", "abc");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            directory.path().join("source/source.exe"),
            std::fs::Permissions::from_mode(0o777),
        )
        .unwrap();
    }
    crate::package::write_archive(
        "source.tar.gz",
        &directory.path().join("source"),
        &directory.path().join("source.tar.gz"),
    )
    .unwrap();
    let listing = Command::new("tar")
        .current_dir(directory.path())
        .args(["-tvf", "source.tar.gz"])
        .output()
        .unwrap();
    assert!(
        listing.status.success(),
        "{}",
        String::from_utf8_lossy(&listing.stderr)
    );
    let listing = String::from_utf8(listing.stdout).unwrap();
    let source = listing
        .lines()
        .find(|line| line.ends_with("tallyline/source.exe"))
        .unwrap();
    assert!(source.starts_with("-rw-r--r--"), "{source}");
}

#[test]
fn r_doc_3__native_source_scopes_require_the_exact_reviewed_identity() {
    let directory = crate::scratch::Scratch::new();
    directory.write("payload/legal/sqlite/LICENSE", "Synthetic native notice\n");
    directory.write(
        "payload/legal/crates/index.tsv",
        "crate\tversion\tdeclared\tdetected\n",
    );
    let catalogue = r#"{"schema_version":1,"components":[{"name":"SQLite","version":"3.54.1","licence":"blessing","source":"https://www.sqlite.org/2026/sqlite-amalgamation-3540100.zip","copyright":{"status":"provided","notices":["Synthetic fixture"]},"notices":[{"path":"sqlite/LICENSE","text":"Synthetic native notice\n"}]}]}"#;
    directory.write("payload/legal/catalogue.json", catalogue);
    directory.write(
        "source/project/supply-chain/notices/sqlite/NOTICE",
        "Synthetic native notice\n",
    );
    directory.write(
        "source/vendor/libsqlite3-sys-0.39.1/sqlite3/sqlite3.c",
        "abc",
    );
    let archive = directory.path().join("source.tar.gz");
    crate::package::write_archive("source.tar.gz", &directory.path().join("source"), &archive)
        .unwrap();
    let map = directory.path().join("source-map.json");
    let mut arguments = vec![
        "source-map".into(),
        directory.path().join("payload").display().to_string(),
        directory.path().join("source").display().to_string(),
        archive.display().to_string(),
        map.display().to_string(),
    ];
    crate::package::run(&arguments).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(map).unwrap()).unwrap();
    assert_eq!(
        value["components"][0]["patterns"][0],
        "vendor/libsqlite3-sys-0.39.1/sqlite3/*"
    );
    directory.write(
        "payload/legal/catalogue.json",
        &catalogue.replace("3.54.1", "3.55.0"),
    );
    let unreviewed = directory.path().join("unreviewed-source-map.json");
    *arguments.last_mut().unwrap() = unreviewed.display().to_string();
    assert!(crate::package::run(&arguments).is_err());
    assert!(!unreviewed.exists());
}

#[test]
fn r_sec_8__protected_commands_reject_a_bad_seal_before_fixture_access() {
    let directory = crate::scratch::Scratch::new();
    directory.write("package-manifest.json", "{}");
    let missing = directory.path().join("fixture-does-not-exist");
    for command in ["inject", "verify-injected"] {
        let error = crate::package::run(&[
            command.into(),
            directory.path().display().to_string(),
            "0".repeat(64),
            missing.display().to_string(),
        ])
        .unwrap_err();
        assert!(error.to_string().contains("seal mismatch"), "{error}");
    }
    assert!(
        !directory
            .path()
            .join("payload/tallyline-app-token")
            .exists()
    );
}

#[test]
fn r_doc_3_r_sec_8__source_bindings_detect_graph_neutral_lockfile_byte_changes() {
    let directory = crate::scratch::Scratch::new();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    for name in ["Cargo.lock", "NOTICE", "supply-chain/notices.toml"] {
        directory.write(name, &std::fs::read_to_string(root.join(name)).unwrap());
    }
    let arguments = [
        "source-bindings".into(),
        directory.path().display().to_string(),
    ];
    crate::package::run(&arguments).unwrap();
    let lock = std::fs::read_to_string(root.join("Cargo.lock")).unwrap();
    directory.write(
        "Cargo.lock",
        &(lock + "\n# Synthetic graph-neutral source-binding change\n"),
    );
    let error = crate::package::run(&arguments).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("stale packaging legal source binding: Cargo.lock")
    );
}
