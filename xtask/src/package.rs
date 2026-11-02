// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Source-independent commands for packaging prebuilt distributions.

mod archive;
mod legal;
mod manifest;
mod payload;
mod scan;
mod source_map;
pub mod tools;
mod verify;

use std::io::{self, Write};
use std::path::Path;

/// Parses the canonical stable release tag within Windows Installer version limits.
///
/// This checks spelling and numeric limits only; callers must authenticate the signed tag.
///
/// # Errors
/// Returns an error for a noncanonical tag or a component outside MSI limits.
pub fn release_version(tag: &str) -> io::Result<&str> {
    let version = tag
        .strip_prefix('v')
        .ok_or_else(|| io::Error::other("release identity requires vMAJOR.MINOR.PATCH"))?;
    let fields = version.split('.').collect::<Vec<_>>();
    if fields.len() != 3
        || fields.iter().any(|field| {
            field.is_empty()
                || !field.bytes().all(|byte| byte.is_ascii_digit())
                || (field.len() > 1 && field.starts_with('0'))
        })
    {
        return Err(io::Error::other(
            "release tag is not canonical stable SemVer",
        ));
    }
    let numbers = fields
        .iter()
        .map(|field| field.parse::<u16>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(io::Error::other)?;
    if !matches!(numbers.as_slice(), [major, minor, _] if *major <= 255 && *minor <= 255) {
        return Err(io::Error::other(
            "release version exceeds MSI limits 255.255.65535",
        ));
    }
    Ok(version)
}

/// Dispatches packaging without discovering a source checkout or invoking Cargo.
///
/// # Errors
/// Returns an invalid command, rejected release identity or output failure.
pub fn run(arguments: &[String]) -> io::Result<()> {
    let arguments = arguments.iter().map(String::as_str).collect::<Vec<_>>();
    match arguments.as_slice() {
        [] | ["help"] => writeln!(
            io::stdout(),
            "usage: xtask package version <vMAJOR.MINOR.PATCH> | tools-verify <windows|linux> <tools> | source-map <payload> <unpacked-source> <archive> <fresh-output> | legal-verify [windows|linux] <payload> | prepare <windows|linux> <vMAJOR.MINOR.PATCH> <payload> <tools> <fresh-stage> | verify <stage> <seal> | archive <zip|tar.gz> <input> <fresh-output> | scan <unpacked-tree> <fixture-file> [allowed-relative-file]"
        ),
        ["archive", format, input, output] => {
            write_archive(format, Path::new(input), Path::new(output))
        }
        ["prepare", target, tag, input, tools, stage] => {
            let target = match *target {
                "windows" => Target::Windows,
                "linux" => Target::Linux,
                _ => return Err(io::Error::other("unsupported packaging platform")),
            };
            let seal = prepare(
                Path::new(input),
                Path::new(tools),
                Path::new(stage),
                target,
                tag,
            )?;
            writeln!(io::stdout(), "{seal}")
        }
        ["verify", stage, seal] => verify_stage(Path::new(stage), seal),
        ["source-map", payload, source, archive, output] => source_map::write(
            Path::new(payload),
            Path::new(source),
            Path::new(archive),
            Path::new(output),
        ),
        ["legal-verify", target, root] => {
            let target = match *target {
                "windows" => Target::Windows,
                "linux" => Target::Linux,
                _ => return Err(io::Error::other("unsupported packaging platform")),
            };
            manifest::legal_catalogue(Path::new(root))?;
            legal::verify_profile(Path::new(root), target)
        }
        ["legal-verify", root] => manifest::legal_catalogue(Path::new(root)).map(|_| ()),
        ["tools-verify", target, root] => tools::verify(target, Path::new(root)),
        ["scan", root, fixture] => scan_command(Path::new(root), Path::new(fixture), None),
        ["scan", root, fixture, allowed] => scan_command(
            Path::new(root),
            Path::new(fixture),
            Some(Path::new(allowed)),
        ),
        ["version", tag] => writeln!(io::stdout(), "{}", release_version(tag)?),
        _ => Err(io::Error::other(
            "usage: xtask package version <vMAJOR.MINOR.PATCH> | tools-verify <windows|linux> <tools> | source-map <payload> <unpacked-source> <archive> <fresh-output> | legal-verify [windows|linux] <payload> | prepare <windows|linux> <vMAJOR.MINOR.PATCH> <payload> <tools> <fresh-stage> | verify <stage> <seal> | archive <zip|tar.gz> <input> <fresh-output> | scan <unpacked-tree> <fixture-file> [allowed-relative-file]",
        )),
    }
}

/// Writes a deterministic plain application archive without invoking build tools.
///
/// # Errors
/// Returns invalid formats, unsafe inputs, archive limits or filesystem errors.
pub fn write_archive(format: &str, source: &Path, output: &Path) -> io::Result<()> {
    let source = source.canonicalize()?;
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    if !source.is_dir() || parent.canonicalize()?.starts_with(&source) {
        return Err(io::Error::other(
            "archive output must be outside the application directory",
        ));
    }
    if !matches!(format, "tar.gz" | "zip") {
        return Err(io::Error::other("unsupported application archive format"));
    }
    let file = std::fs::File::options()
        .write(true)
        .create_new(true)
        .open(output)?;
    let result = match format {
        "tar.gz" => archive::tar_gzip(&source, file),
        _ => archive::zip(&source, file),
    };
    if let Err(error) = result {
        std::fs::remove_file(output).map_err(|cleanup| {
            io::Error::other(format!(
                "archive failed ({error}); partial output cleanup failed ({cleanup})"
            ))
        })?;
        return Err(error);
    }
    Ok(())
}

fn scan_command(root: &Path, fixture: &Path, allowed: Option<&Path>) -> io::Result<()> {
    let findings = scan::leaks(root, fixture, allowed)?;
    if findings.is_empty() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "token boundary violation in {} file(s)",
            findings.len()
        )))
    }
}

/// The supported x64 distribution platform.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// Native Windows MSI and ZIP.
    Windows,
    /// Native Linux Debian and tar.gz.
    Linux,
}

impl Target {
    fn name(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Linux => "linux",
        }
    }

    fn is_native(self) -> bool {
        matches!(self, Self::Windows) && cfg!(windows)
            || matches!(self, Self::Linux) && cfg!(target_os = "linux")
    }

    fn executable(self) -> &'static str {
        match self {
            Self::Windows => "tallyline.exe",
            Self::Linux => "tallyline",
        }
    }

    fn helper(self) -> &'static str {
        match self {
            Self::Windows => "xtask.exe",
            Self::Linux => "xtask",
        }
    }

    fn tool(self) -> &'static str {
        match self {
            Self::Windows => "wix.exe",
            Self::Linux => "cargo-packager",
        }
    }

    fn runtime(self) -> &'static str {
        match self {
            Self::Windows => "onnxruntime.dll",
            Self::Linux => "libonnxruntime.so.1.31.0",
        }
    }

    fn runtime_hash(self) -> &'static str {
        match self {
            Self::Windows => "0c42dc4e278f0968fdf9ebf1b1e54a74c657a9942e70803e453c91993451cf4c",
            Self::Linux => "469849977cb60744ea160927e1a3a0643fd4c55c0aff743251c6d293402e334e",
        }
    }

    fn triple(self) -> &'static str {
        match self {
            Self::Windows => "x86_64-pc-windows-msvc",
            Self::Linux => "x86_64-unknown-linux-gnu",
        }
    }
}

/// Seals prebuilt application resources, this helper and the exact approved tool inventory.
///
/// Preparation is native-platform-only and never reads a token fixture or executes supplied files.
///
/// # Errors
/// Returns invalid release tags, unapproved tools, unsafe/incomplete payloads or filesystem failures.
pub fn prepare(
    input: &Path,
    tools: &Path,
    stage: &Path,
    target: Target,
    tag: &str,
) -> io::Result<String> {
    let version = release_version(tag)?;
    if !target.is_native() {
        return Err(io::Error::other(
            "stage preparation requires its native operating system",
        ));
    }
    self::tools::verify(target.name(), tools)?;
    manifest::prepare(input, tools, stage, target, version)
}

/// Verifies every staged byte and mode against an independently retained seal and approved tools.
///
/// The running helper must match the sealed helper; no supplied file is executed.
///
/// # Errors
/// Returns changed, missing, extra or unsafe files, a different helper or malformed metadata.
pub fn verify_stage(stage: &Path, seal: &str) -> io::Result<()> {
    verify::read(stage, seal).map(|_| ())
}
