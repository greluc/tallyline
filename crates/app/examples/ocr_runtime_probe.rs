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
    let reader_mode = match mode.as_deref() {
        Some(value) if value == std::ffi::OsStr::new("--small-reader") => Some(false),
        Some(value) if value == std::ffi::OsStr::new("--small-textline") => Some(true),
        _ => None,
    };
    let reader_path = if reader_mode.is_some() {
        arguments.next()
    } else {
        None
    };
    let reader_selftest = mode.as_deref() == Some(std::ffi::OsStr::new("--small-reader-selftest"));
    let tensors = match mode.as_deref() {
        Some(value) if value == std::ffi::OsStr::new("--load-only") => false,
        Some(value) if value == std::ffi::OsStr::new("--small-tensors") => true,
        _ if reader_mode.is_some() || reader_selftest => false,
        _ => {
            return Err(
                std::io::Error::other("expected an explicit bounded OCR probe mode").into(),
            );
        }
    };
    let raw_output = match arguments.next() {
        None => None,
        Some(flag) if reader_mode.is_some() && flag == std::ffi::OsStr::new("--raw-output") => {
            Some(arguments.next().ok_or_else(|| {
                std::io::Error::other("raw output requires one absolute private destination")
            })?)
        }
        Some(_) => return Err(std::io::Error::other("unexpected probe arguments").into()),
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
    if reader_mode.is_some() || reader_selftest {
        return run_reader(
            token,
            checks,
            reader_path,
            reader_mode,
            reader_selftest,
            raw_output,
        );
    }
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
    tallyline_app::api::CpuWorker::start("tallyline-ocr-probe", move || run_worker(token))?
        .finish()?
}

fn run_reader(
    mut token: OcrRuntimeLoadToken,
    checks: RuntimeHostChecks,
    path: Option<std::ffi::OsString>,
    line_mode: Option<bool>,
    selftest: bool,
    raw_output: Option<std::ffi::OsString>,
) -> Result<ExitCode, Box<dyn std::error::Error + Send + Sync>> {
    use std::time::{Duration, Instant};
    use tallyline_adapter_files::api::model::{ImageIntake, ImageIntakeSettings};
    use tallyline_adapter_ocr::api::model::{
        OcrRuntimeSettings, SmallOcrReader, SmallSessionState,
    };
    use tallyline_shared_kernel::api::model::ImageRaster;
    let executable = std::env::current_exe()?.canonicalize()?;
    let directory = executable
        .parent()
        .ok_or_else(|| std::io::Error::other("probe executable has no directory"))?
        .join("models/small");
    let assets = SmallOcrAssets::load(&directory)?;
    let synthetic_clock = std::cell::Cell::new(Duration::ZERO);
    let started = Instant::now();
    let mut reader = SmallOcrReader::new(
        &assets,
        OcrRuntimeSettings::default(),
        || {
            if selftest {
                synthetic_clock.get()
            } else {
                started.elapsed()
            }
        },
        || OcrRuntime::load(&mut token, checks),
    );
    if reader.session_state() != SmallSessionState::Empty {
        return Err(std::io::Error::other("reader did not start empty").into());
    }
    let output = if selftest {
        let image = ImageRaster::new(64, 64, vec![0xff80_8080; 64 * 64])
            .map_err(|_| std::io::Error::other("invalid synthetic raster"))?;
        let first = reader.read_line(&image)?;
        let reused = reader.read_line(&image)?;
        if first != reused || reader.session_state() != SmallSessionState::Recognizer {
            return Err(std::io::Error::other("recogniser reuse contract failed").into());
        }
        let boxes = reader.detect(&image)?;
        if reader.session_state() != SmallSessionState::Both {
            return Err(std::io::Error::other("detector did not join resident recogniser").into());
        }
        synthetic_clock.set(Duration::from_secs(1));
        let _completion_read = reader.read_line(&image)?;
        synthetic_clock.set(Duration::ZERO);
        if !matches!(
            reader.close_if_idle(),
            Err(tallyline_adapter_ocr::api::model::SmallReaderError::ClockRegression)
        ) || reader.session_state() != SmallSessionState::Both
        {
            return Err(std::io::Error::other("monotonic clock regression contract failed").into());
        }
        synthetic_clock.set(Duration::from_millis(300_999));
        if reader.close_if_idle()? {
            return Err(std::io::Error::other("sessions closed before idle boundary").into());
        }
        synthetic_clock.set(Duration::from_secs(301));
        if !reader.close_if_idle()? || reader.session_state() != SmallSessionState::Empty {
            return Err(std::io::Error::other("sessions did not close at idle boundary").into());
        }
        let reopened = reader.read_line(&image)?;
        if reopened != first {
            return Err(std::io::Error::other("recogniser changed after session reopening").into());
        }
        reader.close_sessions();
        serde_json::json!({"profile":reader.profile().identifier(),"mode":"synthetic-reader-lifecycle","labels":first.characters.len(),"detected_boxes":boxes.len(),"recogniser_repeat_equal":true,"both_sessions_resident":true,"idle_before_boundary_retained":true,"idle_at_boundary_released":true,"reopened_read_equal":true,"monotonic_regression_refused":true,"explicit_shutdown_empty":reader.session_state()==SmallSessionState::Empty})
    } else {
        let path =
            path.ok_or_else(|| std::io::Error::other("reader mode requires one image path"))?;
        let image = match tallyline_adapter_files::api::read_image_file(
            std::path::Path::new(&path),
            &ImageIntakeSettings::default(),
        ) {
            ImageIntake::Decoded(image) => image,
            ImageIntake::SetAside(_) => {
                return Err(std::io::Error::other("image intake refused the probe image").into());
            }
        };
        let (lines, labels, raw_lines) = if line_mode == Some(true) {
            let reading = reader.read_line(&image.raster)?;
            let raw = if raw_output.is_some() {
                let width = f64::from(image.raster.width());
                let height = f64::from(image.raster.height());
                vec![raw_line(
                    [[0.0, 0.0], [width, 0.0], [width, height], [0.0, height]],
                    None,
                    &reading,
                )]
            } else {
                Vec::new()
            };
            (1, reading.characters.len(), raw)
        } else {
            let readings = reader.read_image(&image.raster)?;
            let raw = if raw_output.is_some() {
                readings
                    .iter()
                    .map(|line| {
                        raw_line(
                            line.region.corners,
                            Some(line.region.probability),
                            &line.reading,
                        )
                    })
                    .collect()
            } else {
                Vec::new()
            };
            (
                readings.len(),
                readings
                    .iter()
                    .map(|line| line.reading.characters.len())
                    .sum(),
                raw,
            )
        };
        if let Some(path) = &raw_output {
            let artifact = serde_json::json!({"schema_version":1,"profile":reader.profile().identifier(),"mode":if line_mode == Some(true) {"raw-textline"} else {"raw-image"},"dimensions":[image.raster.width(),image.raster.height()],"downscaled_at_intake":image.downscaled.is_some(),"lines":raw_lines});
            let bytes = serde_json::to_vec_pretty(&artifact)?;
            tallyline_adapter_files::api::write_private_artifact(
                std::path::Path::new(path),
                &bytes,
            )?;
        }
        serde_json::json!({"profile":reader.profile().identifier(),"mode":if line_mode == Some(true) { "raw-textline" } else { "raw-image" },"image_dimensions":[image.raster.width(),image.raster.height()],"downscaled_at_intake":image.downscaled.is_some(),"lines":lines,"labels":labels,"private_raw_artifact_written":raw_output.is_some(),"session_state":format!("{:?}",reader.session_state())})
    };
    drop(reader);
    if !token.native_load_attempted() {
        return Err(std::io::Error::other("reader did not initialise the native runtime").into());
    }
    let retry = RuntimeHostChecks::new(
        |_| Err(RuntimeHostError::SearchPolicy),
        |_| Err(RuntimeHostError::MappedLibrary),
    );
    if !matches!(
        OcrRuntime::load(&mut token, retry),
        Err(RuntimeLoadError::AlreadyAttempted)
    ) {
        return Err(std::io::Error::other("reader did not retain the single load attempt").into());
    }
    serde_json::to_writer_pretty(std::io::stdout().lock(), &output)?;
    Ok(ExitCode::SUCCESS)
}

fn raw_line(
    corners: [[f64; 2]; 4],
    detection_probability: Option<f32>,
    reading: &tallyline_adapter_ocr::api::model::CtcReading,
) -> serde_json::Value {
    let characters: Vec<_> = reading.characters.iter().map(|character| serde_json::json!({"symbol":character.symbol,"probability":character.probability,"frames":[character.frames.start,character.frames.end]})).collect();
    serde_json::json!({"corners":corners,"detection_probability":detection_probability,"text":reading.text,"characters":characters})
}
