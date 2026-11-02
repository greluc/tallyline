// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Real-file contracts for the isolated research command.

use std::{fs, path::PathBuf, process::Command};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> std::io::Result<Self> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .join(format!(
                "research-contract-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ));
        fs::create_dir_all(&path)?;
        Ok(Self(path))
    }
    fn run(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        Command::new(env!("CARGO_BIN_EXE_tesseract-research"))
            .args(args)
            .env("DOCKER_HOST", "tcp://192.0.2.1:2375")
            .env("DOCKER_CONTEXT", "RECORDED_POISON")
            .env("TESSDATA_PREFIX", "RECORDED_POISON")
            .env("CFLAGS", "RECORDED_POISON")
            .current_dir(&self.0)
            .output()
    }
    fn bitmap(&self) -> std::io::Result<()> {
        let mut bytes = vec![
            66, 77, 70, 0, 0, 0, 0, 0, 0, 0, 54, 0, 0, 0, 40, 0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 0, 1,
            0, 24, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        bytes.extend([255, 0, 0, 255, 255, 255, 0, 0, 0, 0, 255, 0, 255, 0, 0, 0]);
        fs::write(self.0.join("input.bmp"), bytes)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        assert!(fs::remove_dir_all(&self.0).is_ok());
    }
}

#[test]
fn recorded_tsv_preserves_literal_currency_sign_and_unit() {
    let fixture = Fixture::new().unwrap();
    fixture.bitmap().unwrap();
    fs::write(fixture.0.join("recorded.tsv"), "level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext\n1\t1\t0\t0\t0\t0\t0\t0\t2\t2\t-1\t\n5\t1\t1\t1\t1\t1\t0\t0\t2\t1\t87.125\t−1,234.50¤/SCU\n").unwrap();
    let output = fixture
        .run(&["tsv", "input.bmp", "recorded.tsv", "parsed.json"])
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.0.join("parsed.json")).unwrap()).unwrap();
    assert_eq!(parsed["words"][0]["text"], "−1,234.50¤/SCU");
    assert!(!String::from_utf8(output.stdout).unwrap().contains("SCU"));
}

#[test]
fn preparation_rejects_unfrozen_metadata_before_creating_owned_state() {
    let fixture = Fixture::new().unwrap();
    fs::create_dir(fixture.0.join("campaign")).unwrap();
    fs::create_dir(fixture.0.join("proposal")).unwrap();
    fs::write(fixture.0.join("proposal/scope.json"), b"{}").unwrap();
    let output = fixture.run(&["prepare", "campaign", "proposal"]).unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("scope digest mismatch"));
    assert!(!fixture.0.join("campaign/controller").exists());
}

#[test]
fn bounded_recorded_process_contracts_reap_deadline_output_and_blocked_stdin() {
    let fixture = Fixture::new().unwrap();
    for mode in [
        "deadline",
        "output",
        "blocked-stdin",
        "error",
        "environment",
        "row",
        "cancel",
        "stderr",
        "runtime-stderr",
        "admission-reject",
        "admission-accept",
        "admission-limit-mismatch",
    ] {
        let output = fixture.run(&["contract", mode]).unwrap();
        assert!(
            output.status.success(),
            "{mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("contract passed"));
    }
}

fn png_chunk(kind: &[u8; 4], data: &[u8], output: &mut Vec<u8>) {
    output.extend((data.len() as u32).to_be_bytes());
    output.extend(kind);
    output.extend(data);
    let mut crc = 0xffff_ffffu32;
    for byte in kind.iter().chain(data) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ if crc & 1 == 1 { 0xedb8_8320 } else { 0 };
        }
    }
    output.extend((!crc).to_be_bytes());
}

fn tiny_png() -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
    png_chunk(
        b"IHDR",
        &[0, 0, 0, 2, 0, 0, 0, 2, 8, 2, 0, 0, 0],
        &mut bytes,
    );
    let pixels = [0u8, 255, 0, 0, 0, 255, 0, 0, 0, 0, 255, 255, 255, 255];
    let mut zlib = vec![0x78, 0x01, 0x01, 14, 0, 0xf1, 0xff];
    zlib.extend(pixels);
    let (mut a, mut b) = (1u32, 0u32);
    for byte in pixels {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    zlib.extend(((b << 16) | a).to_be_bytes());
    png_chunk(b"IDAT", &zlib, &mut bytes);
    png_chunk(b"IEND", &[], &mut bytes);
    bytes
}

#[test]
fn png_conversion_preserves_worked_bgr_bottom_up_rows_and_zero_padding() {
    let fixture = Fixture::new().unwrap();
    fs::write(fixture.0.join("colours.png"), tiny_png()).unwrap();
    let output = fixture.run(&["bmp", "colours.png", "colours.bmp"]).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes = fs::read(fixture.0.join("colours.bmp")).unwrap();
    assert_eq!(bytes.len(), 70);
    assert_eq!(
        bytes.get(54..),
        Some([255, 0, 0, 255, 255, 255, 0, 0, 0, 0, 255, 0, 255, 0, 0, 0].as_slice())
    );
    assert!(
        !fixture
            .run(&["bmp", "colours.png", "colours.bmp"])
            .unwrap()
            .status
            .success()
    );
}

#[test]
fn malformed_bmp_and_tsv_are_refused_without_creating_parsed_output() {
    let fixture = Fixture::new().unwrap();
    fixture.bitmap().unwrap();
    let header = "level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext\n";
    for (index, body) in [
        "1\t1\t0\t0\t0\t0\t0\t0\t2\t2\t-1\t\n5\t1\t1\t1\t1\t1\t1\t0\t2\t1\t87\t12¤\n",
        "1\t1\t0\t0\t0\t0\t0\t0\t2\t2\t-1\t\n5\t1\t1\t1\t1\t1\t0\t0\t2\t1\tNaN\t12¤\n",
        "1\t1\t0\t0\t0\t0\t0\t0\t2\t2\t-1\t\n2\t1\t1\t1\t0\t0\t0\t0\t2\t1\t-1\t\n",
    ]
    .iter()
    .enumerate()
    {
        fs::write(fixture.0.join("bad.tsv"), format!("{header}{body}")).unwrap();
        let destination = format!("bad-{index}.json");
        assert!(
            !fixture
                .run(&["tsv", "input.bmp", "bad.tsv", &destination])
                .unwrap()
                .status
                .success(),
            "invalid TSV{index} accepted"
        );
        assert!(!fixture.0.join(destination).exists());
    }
    fs::write(fixture.0.join("bad.tsv"), [0xff]).unwrap();
    assert!(
        !fixture
            .run(&["tsv", "input.bmp", "bad.tsv", "invalid-utf8.json"])
            .unwrap()
            .status
            .success()
    );
    let original = fs::read(fixture.0.join("input.bmp")).unwrap();
    for (index, (offset, value)) in [(18usize, 255u8), (28, 32), (30, 1), (60, 1)]
        .iter()
        .enumerate()
    {
        let mut changed = original.clone();
        *changed.get_mut(*offset).unwrap() = *value;
        fs::write(fixture.0.join("bad.bmp"), changed).unwrap();
        assert!(
            !fixture
                .run(&[
                    "tsv",
                    "bad.bmp",
                    "bad.tsv",
                    &format!("bad-bmp-{index}.json")
                ])
                .unwrap()
                .status
                .success()
        );
    }
}

#[test]
fn changed_generated_native_evidence_is_refused_before_any_process_start() {
    let fixture = Fixture::new().unwrap();
    fs::create_dir(fixture.0.join("work")).unwrap();
    fs::write(fixture.0.join("work/config_auto.h"), b"abc").unwrap();
    fs::write(fixture.0.join("build-evidence.json"),br#"{"generated_artifacts":[{"path":"work/config_auto.h","sha256":"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"}]}"#).unwrap();
    assert!(
        fixture
            .run(&["verify-evidence", "."])
            .unwrap()
            .status
            .success()
    );
    fs::write(fixture.0.join("work/config_auto.h"), b"abd").unwrap();
    let output = fixture.run(&["verify-evidence", "."]).unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("generated build evidence digest mismatch")
    );
    assert!(!fixture.0.join("synthetic.started").exists());
}

#[test]
fn missing_and_tampered_models_are_refused_at_the_file_boundary() {
    let fixture = Fixture::new().unwrap();
    let missing = fixture
        .run(&["verify-model", "best", "missing.traineddata"])
        .unwrap();
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("unavailable"));
    fs::write(
        fixture.0.join("tampered.traineddata"),
        b"recorded native bytes; not a model",
    )
    .unwrap();
    let tampered = fixture
        .run(&["verify-model", "fast", "tampered.traineddata"])
        .unwrap();
    assert!(!tampered.status.success());
    assert!(String::from_utf8_lossy(&tampered.stderr).contains("model digest mismatch"));
}

#[test]
fn preprocessing_plan_refuses_shells_plugins_and_foreign_outputs_without_execution() {
    let fixture = Fixture::new().unwrap();
    let mut plan = String::from("tallyline-preprocess-v1\n");
    for index in 0..334 {
        plan.push_str(&format!("/work/build-lept\t/usr/bin/x86_64-linux-gnu-gcc-12\t-E\t/src/leptonica-1.87.0/src/numabasic.c\t-o\t/work/preprocessed-{index}.ii\t-dD\n"));
    }
    fs::write(fixture.0.join("preprocess.plan"), &plan).unwrap();
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_isolation-probe"))
            .args(["check-preprocess-plan", "preprocess.plan"])
            .current_dir(&fixture.0)
            .output()
            .unwrap()
    };
    assert!(run().status.success());
    for changed in [
        plan.replace("/usr/bin/x86_64-linux-gnu-gcc-12", "/bin/sh"),
        plan.replace("\t-E\t", "\t-fplugin=/work/foreign.so\t-E\t"),
        plan.replace("/work/preprocessed-0.ii", "/tmp/foreign.ii"),
    ] {
        fs::write(fixture.0.join("preprocess.plan"), changed).unwrap();
        assert!(!run().status.success());
    }
}

#[test]
fn recorded_owned_cgroup_peak_is_a_lower_bound_with_exact_limits() {
    let fixture = Fixture::new().unwrap();
    fs::write(fixture.0.join("cgroup.txt"),b"populated 1\nfrozen 0\n200000 100000\n4294967296\n0\n1\n783780\n8187904\nlow 0\nhigh 0\nmax 0\noom 0\noom_kill 0\noom_group_kill 0\npopulated 1\nfrozen 0\n").unwrap();
    let output = fixture.run(&["resource", "cgroup.txt", "4g"]).unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["observed_memory_peak_bytes"], 8187904);
    assert_eq!(value["whole_attempt_peak_known"], false);
    assert!(
        !fixture
            .run(&["resource", "cgroup.txt", "8g"])
            .unwrap()
            .status
            .success()
    );
}

#[test]
fn unpopulated_starting_cgroup_cannot_be_misclassified_as_a_live_limit_refusal() {
    let fixture = Fixture::new().unwrap();
    fs::write(fixture.0.join("starting.txt"),b"populated 0\nfrozen 0\nmax 100000\nmax\nmax\nmax\n0\nlow 0\nhigh 0\nmax 0\noom 0\noom_kill 0\noom_group_kill 0\npopulated 0\nfrozen 0\n").unwrap();
    let output = fixture.run(&["resource", "starting.txt", "8g"]).unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("owned cgroup has no live population")
    );
}
