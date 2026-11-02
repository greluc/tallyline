// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Public loader gates without initialising a native library.

#![expect(
    non_snake_case,
    reason = "Requirement tags contain a double underscore."
)]

use tallyline_adapter_ocr::api::model::{
    OcrRuntime, OcrRuntimeLoadToken, RuntimeHostChecks, RuntimeHostError, RuntimeLoadError,
};

#[test]
fn r_nf_12__the_loader_refuses_missing_startup_mitigation_before_host_or_ort_calls()
-> std::io::Result<()> {
    let result = std::process::Command::new(std::env::current_exe()?)
        .args(["--exact", "loader_probe", "--nocapture"])
        .env_remove("ORT_DISABLE_TELEMETRY")
        .env("TALLYLINE_LOADER_PROBE", "missing-telemetry")
        .output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn r_sec_7_r_sec_8__a_missing_or_same_size_wrong_sibling_cannot_reach_host_checks()
-> std::io::Result<()> {
    run_prepared_probe("missing-library", false)?;
    run_prepared_probe("wrong-library", false)
}

#[test]
fn r_sec_7_r_sec_8__verified_bytes_reach_search_policy_and_its_refusal_stops_native_loading()
-> std::io::Result<()> {
    run_prepared_probe("verified-refused", true)
}

#[test]
fn loader_probe() {
    let Some(mode) = std::env::var_os("TALLYLINE_LOADER_PROBE") else {
        return;
    };
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let before = std::sync::Arc::clone(&calls);
    let after = std::sync::Arc::clone(&calls);
    let checks = RuntimeHostChecks::new(
        move |_| {
            before.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err(RuntimeHostError::SearchPolicy)
        },
        move |_| {
            after.fetch_add(100, std::sync::atomic::Ordering::SeqCst);
            Err(RuntimeHostError::MappedLibrary)
        },
    );
    let mut token = OcrRuntimeLoadToken::new();
    let result = OcrRuntime::load(&mut token, checks);
    assert!(!token.native_load_attempted());
    let expected_calls = match mode.to_str().unwrap() {
        "missing-telemetry" => {
            assert!(matches!(
                result,
                Err(RuntimeLoadError::TelemetryNotPrepared)
            ));
            0
        }
        "missing-library" => {
            assert!(matches!(result, Err(RuntimeLoadError::Read)));
            0
        }
        "wrong-library" => {
            assert!(matches!(result, Err(RuntimeLoadError::HashMismatch)));
            0
        }
        "verified-refused" => {
            assert!(matches!(
                result,
                Err(RuntimeLoadError::Host(RuntimeHostError::SearchPolicy))
            ));
            1
        }
        other => panic!("unexpected loader probe {other}"),
    };
    assert_eq!(
        calls.load(std::sync::atomic::Ordering::SeqCst),
        expected_calls
    );
}

#[cfg(windows)]
const LIBRARY: (&str, &str, u64) = ("windows", "onnxruntime.dll", 17_253_176);
#[cfg(target_os = "linux")]
const LIBRARY: (&str, &str, u64) = ("linux", "libonnxruntime.so.1.31.0", 29_345_384);

fn run_prepared_probe(mode: &str, verified: bool) -> std::io::Result<()> {
    let base = std::env::temp_dir().canonicalize()?;
    let mut directory = None;
    for attempt in 0..100 {
        let path = base.join(format!(
            "tallyline-loader-{}-{mode}-{attempt}",
            std::process::id()
        ));
        match std::fs::create_dir(&path) {
            Ok(()) => {
                directory = Some(path);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    let directory =
        directory.ok_or_else(|| std::io::Error::other("cannot reserve loader probe directory"))?;
    let executable = directory.join(if cfg!(windows) { "probe.exe" } else { "probe" });
    std::fs::copy(std::env::current_exe()?, &executable)?;
    if verified {
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/runtime")
            .join(LIBRARY.0)
            .join(LIBRARY.1);
        std::fs::copy(source, directory.join(LIBRARY.1))?;
    } else if mode == "wrong-library" {
        std::fs::File::create(directory.join(LIBRARY.1))?.set_len(LIBRARY.2)?;
    }
    let outcome = std::process::Command::new(executable)
        .args(["--exact", "loader_probe", "--nocapture"])
        .env("ORT_DISABLE_TELEMETRY", "1")
        .env("TALLYLINE_LOADER_PROBE", mode)
        .output()?;
    let actual = directory.canonicalize()?;
    let prefix = format!("tallyline-loader-{}-", std::process::id());
    if !actual.starts_with(&base)
        || !actual
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(&prefix))
    {
        return Err(std::io::Error::other(
            "loader cleanup escaped owned directory",
        ));
    }
    std::fs::remove_dir_all(actual)?;
    assert!(
        outcome.status.success(),
        "{mode}: {} {}",
        String::from_utf8_lossy(&outcome.stdout),
        String::from_utf8_lossy(&outcome.stderr)
    );
    Ok(())
}
