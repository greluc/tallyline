// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Explicit headless native load acceptance probe; never starts the desktop shell.

use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use tallyline_adapter_ocr::api::model::{
    OcrRuntime, OcrRuntimeLoadToken, RuntimeHostChecks, RuntimeHostError, RuntimeLoadError,
    SmallOcrAssets,
};
use tallyline_adapter_platform::api::model::{
    CrtModuleVersion, OcrChildExit, OcrNativeError, OcrProcessStartup,
};
use tallyline_adapter_platform::api::{restrict_ocr_dll_search, verify_ocr_native_modules};

fn main() -> Result<ExitCode, Box<dyn std::error::Error + Send + Sync>> {
    match tallyline_adapter_platform::api::prepare_ocr_process()? {
        OcrProcessStartup::Prepared => run(),
        OcrProcessStartup::Relaunched(status) => Ok(match OcrChildExit::from_status(&status) {
            OcrChildExit::Code(code) => ExitCode::from(code),
            OcrChildExit::Abnormal => ExitCode::FAILURE,
        }),
    }
}

fn run_worker(
    mut token: OcrRuntimeLoadToken,
) -> Result<ExitCode, Box<dyn std::error::Error + Send + Sync>> {
    let mut arguments = std::env::args_os().skip(1);
    let mode = arguments.next();
    let tensors = match mode.as_deref() {
        Some(value) if value == std::ffi::OsStr::new("--load-only") => false,
        Some(value) if value == std::ffi::OsStr::new("--small-tensors") => true,
        _ => return Err(std::io::Error::other("expected --load-only or --small-tensors").into()),
    };
    if arguments.next().is_some() {
        return Err(std::io::Error::other("unexpected probe arguments").into());
    }
    let versions = Arc::new(Mutex::new(Vec::<CrtModuleVersion>::new()));
    let failure = Arc::new(Mutex::new(None::<OcrNativeError>));
    let before_failure = Arc::clone(&failure);
    let after_failure = Arc::clone(&failure);
    let after_versions = Arc::clone(&versions);
    let checks = RuntimeHostChecks::new(
        move |_| {
            restrict_ocr_dll_search().map_err(|error| {
                if let Ok(mut last) = before_failure.lock() {
                    *last = Some(error);
                }
                RuntimeHostError::SearchPolicy
            })
        },
        move |path| match verify_ocr_native_modules(path) {
            Ok(observed) => {
                *after_versions
                    .lock()
                    .map_err(|_| RuntimeHostError::MappedLibrary)? = observed;
                Ok(())
            }
            Err(error) => {
                if let Ok(mut last) = after_failure.lock() {
                    *last = Some(error);
                }
                Err(match error {
                    OcrNativeError::CrtTooOld { .. }
                    | OcrNativeError::InvalidVersion(_)
                    | OcrNativeError::MissingModule(_) => RuntimeHostError::CrtPrerequisite,
                    OcrNativeError::UnsupportedTarget
                    | OcrNativeError::InvalidPath
                    | OcrNativeError::SearchRestriction
                    | OcrNativeError::AlreadyMapped
                    | OcrNativeError::ModuleQuery(_)
                    | OcrNativeError::ModulePath(_)
                    | OcrNativeError::MappedFileMismatch
                    | OcrNativeError::InvalidModule => RuntimeHostError::MappedLibrary,
                })
            }
        },
    );
    let mut runtime = OcrRuntime::load(&mut token, checks);
    let duplicate_refused = if token.native_load_attempted() {
        let retry = RuntimeHostChecks::new(
            |_| Err(RuntimeHostError::SearchPolicy),
            |_| Err(RuntimeHostError::MappedLibrary),
        );
        matches!(
            OcrRuntime::load(&mut token, retry),
            Err(RuntimeLoadError::AlreadyAttempted)
        )
    } else {
        false
    };
    if token.native_load_attempted() && !duplicate_refused {
        return Err(std::io::Error::other("native load token did not retain its attempt").into());
    }
    let output = match &mut runtime {
        Ok(runtime) => {
            let versions = versions
                .lock()
                .map_err(|_| std::io::Error::other("probe version result unavailable"))?;
            let versions: Vec<_> = versions.iter().map(|entry| serde_json::json!({"module": format!("{:?}", entry.module()), "version": entry.parts()})).collect();
            let tensor_report = if tensors {
                let executable = std::env::current_exe()?.canonicalize()?;
                let directory = executable
                    .parent()
                    .ok_or_else(|| std::io::Error::other("probe executable has no directory"))?
                    .join("models/small");
                let assets = SmallOcrAssets::load(&directory)?;
                let smoke = runtime.smoke_small_models(&assets)?;
                serde_json::json!({"profile": smoke.profile, "detector": {"input_shape": smoke.detector.input_shape, "output_shape": smoke.detector.output_shape, "output_elements": smoke.detector.output_elements, "probability_range": smoke.detector.probability_range}, "recognizer": {"input_shape": smoke.recognizer.input_shape, "output_shape": smoke.recognizer.output_shape, "output_elements": smoke.recognizer.output_elements, "probability_range": smoke.recognizer.probability_range}, "dictionary_classes": smoke.dictionary_classes, "ctc_emitted_labels": smoke.ctc_emitted_labels, "requested_session_options": {"intra_op_threads":1,"inter_op_threads":1,"sequential":true,"spinning":false,"cpu_arena":false,"memory_pattern":false,"graph_optimisation":3}})
            } else {
                serde_json::Value::String("not_run".to_owned())
            };
            serde_json::json!({"runtime": runtime.version(), "native_load_attempted": token.native_load_attempted(), "duplicate_native_load_refused": duplicate_refused, "mapped_library": "verified", "crt": versions, "tensor_smoke": tensor_report})
        }
        Err(error) => {
            let failure = failure
                .lock()
                .map_err(|_| std::io::Error::other("probe host result unavailable"))?;
            serde_json::json!({"runtime_load": "failed", "native_load_attempted": token.native_load_attempted(), "duplicate_native_load_refused": duplicate_refused, "reason": error.to_string(), "host_failure": format!("{failure:?}")})
        }
    };
    serde_json::to_writer_pretty(std::io::stdout().lock(), &output)?;
    Ok(if runtime.is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn run() -> Result<ExitCode, Box<dyn std::error::Error + Send + Sync>> {
    let token = OcrRuntimeLoadToken::new();
    std::thread::Builder::new()
        .name("tallyline-ocr-probe".to_owned())
        .spawn(move || run_worker(token))?
        .join()
        .map_err(|_| std::io::Error::other("native OCR probe worker failed"))?
}
