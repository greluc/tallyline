// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Source-independent commands for packaging prebuilt distributions.

mod archive;
mod scan;
pub mod tools;

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
            "usage: xtask package version <vMAJOR.MINOR.PATCH> | archive <zip|tar.gz> <input> <fresh-output> | tools-verify <windows|linux> <tools> | scan <unpacked-tree> <fixture-file> [allowed-relative-file]"
        ),
        ["archive", format, input, output] => {
            write_archive(format, Path::new(input), Path::new(output))
        }
        ["tools-verify", target, root] => tools::verify(target, Path::new(root)),
        ["scan", root, fixture] => scan_command(Path::new(root), Path::new(fixture), None),
        ["scan", root, fixture, allowed] => scan_command(
            Path::new(root),
            Path::new(fixture),
            Some(Path::new(allowed)),
        ),
        ["version", tag] => writeln!(io::stdout(), "{}", release_version(tag)?),
        _ => Err(io::Error::other(
            "usage: xtask package version <vMAJOR.MINOR.PATCH> | archive <zip|tar.gz> <input> <fresh-output> | tools-verify <windows|linux> <tools> | scan <unpacked-tree> <fixture-file> [allowed-relative-file]",
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
