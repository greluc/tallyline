// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Source-independent commands for packaging prebuilt distributions.

mod archive;
mod capture;
mod copyright;
mod deb_archive;
mod debian;
mod legal;
mod manifest;
mod msi;
mod msi_crt;
mod payload;
mod protected;
mod scan;
mod source_archive;
mod source_inputs;
mod source_map;
pub mod tools;
mod verify;

use std::io::{self, Read, Write};
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
            "usage: xtask package version <vMAJOR.MINOR.PATCH> | tools-verify <windows|linux> <tools> | source-bindings <source-root> | source-verify <payload> <archive> | source-map <payload> <unpacked-source> <archive> <fresh-output> | msi-probe <approved-tools> <fresh-output> | build <stage> <seal> <fixture> <fresh-output> | repack-deb <prepared-tree> <fresh-output> | copyright <payload> <fresh-output> | legal-verify [windows|linux] <payload> | prepare <windows|linux> <vMAJOR.MINOR.PATCH> <payload> <tools> <fresh-stage> | inject <stage> <seal> <fixture> | verify-injected <stage> <seal> <fixture> | verify <stage> <seal> | archive <zip|tar.gz|source.tar.gz> <input> <fresh-output> | scan <unpacked-tree> <fixture-file> [allowed-relative-file]"
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
        ["inject", stage, seal, fixture] => {
            inject_fixture(Path::new(stage), seal, Path::new(fixture))
        }
        ["verify-injected", stage, seal, fixture] => {
            verify_injected_stage(Path::new(stage), seal, Path::new(fixture))
        }
        ["source-map", payload, source, archive, output] => source_map::write(
            Path::new(payload),
            Path::new(source),
            Path::new(archive),
            Path::new(output),
        ),
        ["msi-probe", tools, output] => probe_msi_tools(Path::new(tools), Path::new(output)),
        ["build", stage, seal, fixture, output] => build(
            Path::new(stage),
            seal,
            Path::new(fixture),
            Path::new(output),
        ),
        ["repack-deb", tree, output] => repack_debian(Path::new(tree), Path::new(output)),
        ["copyright", payload, output] => {
            write_debian_copyright(Path::new(payload), Path::new(output))
        }
        ["source-bindings", root] => verify_source_bindings(Path::new(root)),
        ["source-verify", payload, archive] => {
            source_map::validate(Path::new(payload), Path::new(archive)).map(|_| ())
        }
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
            "usage: xtask package version <vMAJOR.MINOR.PATCH> | tools-verify <windows|linux> <tools> | source-bindings <source-root> | source-verify <payload> <archive> | source-map <payload> <unpacked-source> <archive> <fresh-output> | msi-probe <approved-tools> <fresh-output> | build <stage> <seal> <fixture> <fresh-output> | repack-deb <prepared-tree> <fresh-output> | copyright <payload> <fresh-output> | legal-verify [windows|linux] <payload> | prepare <windows|linux> <vMAJOR.MINOR.PATCH> <payload> <tools> <fresh-stage> | inject <stage> <seal> <fixture> | verify-injected <stage> <seal> <fixture> | verify <stage> <seal> | archive <zip|tar.gz|source.tar.gz> <input> <fresh-output> | scan <unpacked-tree> <fixture-file> [allowed-relative-file]",
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
    if !matches!(format, "tar.gz" | "zip" | "source.tar.gz") {
        return Err(io::Error::other("unsupported application archive format"));
    }
    let file = std::fs::File::options()
        .write(true)
        .create_new(true)
        .open(output)?;
    let result = match format {
        "tar.gz" => archive::tar_gzip(&source, file, false),
        "source.tar.gz" => archive::tar_gzip(&source, file, true),
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

/// Verifies a complete source-backed handoff before reading a harmless fixture data file.
///
/// The fixture uses the application's bounded printable-ASCII envelope, not an asserted UEX format.
///
/// # Errors
/// Returns invalid immutable inputs, missing source evidence, malformed data or existing injection.
pub fn inject_fixture(stage: &Path, seal: &str, fixture: &Path) -> io::Result<()> {
    let description = verify::read(stage, seal)?;
    require_source(&description)?;
    let bytes = read_fixture(fixture)?;
    scan_command(stage, fixture, None)?;
    let path = stage.join("payload/tallyline-app-token");
    let mut output = std::fs::File::options()
        .write(true)
        .create_new(true)
        .open(&path)?;
    output.write_all(&bytes)?;
    output.sync_all()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))?;
    }
    verify_injected_stage(stage, seal, fixture)
}

/// Checks immutable inputs before accessing the fixture and admits only its exact data file.
///
/// # Errors
/// Returns invalid sealed code/tools/source metadata, extra files, bad modes or data mismatches.
pub fn verify_injected_stage(stage: &Path, seal: &str, fixture: &Path) -> io::Result<()> {
    verify::injected(stage, seal, fixture).map(|_| ())
}

fn require_source(description: &manifest::Verified) -> io::Result<()> {
    if description.source_archive.is_none() {
        return Err(io::Error::other(
            "a verified source archive reference is required before fixture access",
        ));
    }
    Ok(())
}

fn read_fixture(path: &Path) -> io::Result<Vec<u8>> {
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() || metadata.len() == 0 || metadata.len() > 1024 {
        return Err(io::Error::other(
            "fixture must be a bounded regular data file",
        ));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(1025)
        .read_to_end(&mut bytes)?;
    if bytes.is_empty() || bytes.len() > 1024 || !bytes.iter().all(u8::is_ascii_graphic) {
        return Err(io::Error::other(
            "fixture does not match the application's bounded printable ASCII envelope",
        ));
    }
    Ok(bytes)
}

/// Checks every recorded canonical project input of the compiled legal profiles.
///
/// This source-job/check boundary reads data only and never invokes Cargo or build code.
///
/// # Errors
/// Returns stale recorded input bytes, malformed profiles or missing/linked source material.
pub fn verify_source_bindings(root: &Path) -> io::Result<()> {
    legal::verify_source_inputs(Some(root))
}

/// Retains bounded tool output only after checking both streams for fixture disclosure.
///
/// A disclosure replaces both fresh log files with a fixed diagnostic. No raw leaked
/// stream is written. This adapter executes no process and reads only the fixture.
///
/// # Errors
/// Returns invalid or linked log directories, unsafe names, streams over eight MiB,
/// invalid fixture data, existing outputs, I/O failures or discarded disclosure.
pub fn retain_tool_output(
    directory: &Path,
    name: &str,
    stdout: &[u8],
    stderr: &[u8],
    fixture: &Path,
) -> io::Result<()> {
    capture::write(directory, name, stdout, stderr, fixture)
}

/// Writes DEP-5 source-scope and licence metadata from a consistent legal payload.
///
/// This secret-free metadata adapter preserves original terms and reviewed source
/// scopes. General consistency alone does not establish the approved target identity;
/// protected packaging additionally verifies its sealed intrinsic target profile.
///
/// # Errors
/// Returns incomplete legal data, unreviewed source scopes, an existing output or I/O errors.
pub fn write_debian_copyright(payload: &Path, output: &Path) -> io::Result<()> {
    copyright::write(payload, output)
}

/// Writes a Debian archive from a prepared data/control tree without executing it.
///
/// Relocates the app beside its runtime and creates only the fixed relative launcher
/// link. Control scripts and untrusted links are refused. This layout adapter alone
/// does not authenticate a target payload; the protected backend additionally seals it.
///
/// # Errors
/// Returns unexpected layouts/control files, unsafe paths, existing outputs or I/O errors.
pub fn repack_debian(tree: &Path, output: &Path) -> io::Result<()> {
    deb_archive::repack(tree, output)
}

/// Assembles native installer and plain archive from a sealed prebuilt stage.
///
/// Verifies immutable inputs and source reference before fixture access, clones only
/// sealed files, then invokes the fixed approved native tool without Cargo or hooks.
/// The source capsule is never opened or copied by this protected boundary.
///
/// # Errors
/// Returns bad seals, missing source evidence, unsafe/existing outputs, native target
/// mismatches, tool failures or post-assembly payload/fixture discrepancies.
pub fn build(stage: &Path, seal: &str, fixture: &Path, output: &Path) -> io::Result<()> {
    protected::build(stage, seal, fixture, output)
}

/// Builds and validates a fixed secret-free MSI prerequisite-table probe.
///
/// Uses only the exact approved source-built WiX tree, core authoring and a harmless
/// text marker. It reads no fixture or app, executes no installer action and proves
/// no protected distribution or installed-system acceptance.
///
/// # Errors
/// Returns tool mismatches, unsafe/existing output, non-Windows host or WiX failures.
pub fn probe_msi_tools(tools: &Path, output: &Path) -> io::Result<()> {
    msi::probe(tools, output)
}

/// Generates the shared fixed MSI prerequisite authoring without running a tool.
///
/// # Errors
/// Returns a prerequisite condition that exceeds the MSI table limit.
pub fn msi_prerequisite_authoring() -> io::Result<String> {
    msi_crt::authoring()
}
