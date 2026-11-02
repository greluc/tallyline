// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Source-scope evidence bound into the immutable legal payload.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

pub(super) fn write(
    payload: &Path,
    source: &Path,
    archive: &Path,
    output: &Path,
) -> io::Result<()> {
    let source = source.canonicalize()?;
    let parent = output
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()?;
    if parent.starts_with(&source)
        || !source.is_dir()
        || !fs::symlink_metadata(archive)?.file_type().is_file()
    {
        return Err(invalid(
            "source evidence needs a regular archive and separate output",
        ));
    }
    let archive_digest = archive_hash(archive)?;
    if super::archive::tar_gzip_digest(&source)? != archive_digest {
        return Err(invalid(
            "source archive differs from the deterministic unpacked source profile",
        ));
    }
    let catalogue = super::manifest::legal_catalogue(payload)?;
    let components = catalogue
        .get("components")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("missing source components"))?;
    let mut mapped = Vec::new();
    for component in components {
        let patterns = source_scopes(component)?;
        let mut files = Vec::new();
        for pattern in &patterns {
            let directory = pattern
                .strip_suffix("/*")
                .ok_or_else(|| invalid("unsupported source scope"))?;
            let mut path = source.clone();
            for component in Path::new(directory).components() {
                path.push(component);
                if fs::symlink_metadata(&path)?.file_type().is_symlink() {
                    return Err(invalid("unpacked source scope contains a symbolic link"));
                }
            }
            collect(&source, &path, &mut files)?;
        }
        files.sort_by(|left, right| {
            left.get("path")
                .and_then(Value::as_str)
                .cmp(&right.get("path").and_then(Value::as_str))
        });
        if files.is_empty() {
            return Err(invalid("source scope does not match any unpacked file"));
        }
        mapped.push(json!({"name":text(component,"name")?,"version":text(component,"version")?,"licence":text(component,"licence")?,"source":text(component,"source")?,"patterns":patterns,"files":files}));
    }
    let filename = archive
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| invalid("source archive name must be UTF-8"))?;
    let evidence = json!({"schema_version":1,"archive":{"file":filename,"sha256":archive_digest},"components":mapped});
    let mut file = File::options().write(true).create_new(true).open(output)?;
    file.write_all(&serde_json::to_vec_pretty(&evidence)?)?;
    file.sync_all()
}

fn collect(root: &Path, path: &Path, files: &mut Vec<Value>) -> io::Result<()> {
    let kind = fs::symlink_metadata(path)?.file_type();
    if kind.is_symlink() {
        return Err(invalid("unpacked source contains a symbolic link"));
    }
    if kind.is_dir() {
        for (child, _) in super::archive::children(path)? {
            collect(root, &child.path(), files)?;
        }
    } else if kind.is_file() {
        let relative = path
            .strip_prefix(root)
            .map_err(io::Error::other)?
            .to_str()
            .ok_or_else(|| invalid("source file path must be UTF-8"))?
            .replace('\\', "/");
        if !normal(&relative) {
            return Err(invalid("unsafe unpacked source path"));
        }
        files.push(json!({"path":relative,"sha256":crate::hashlist::sha256_hex(&fs::read(path)?)}));
    } else {
        return Err(invalid("unpacked source contains a special file"));
    }
    Ok(())
}

fn normal(value: &str) -> bool {
    !value.is_empty()
        && !value.contains(['\\', ':', '\n', '\r', '\0'])
        && value
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

fn text<'a>(value: &'a Value, key: &str) -> io::Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid("invalid source map text"))
}
fn invalid(message: &str) -> io::Error {
    io::Error::other(message)
}

fn source_scopes(component: &Value) -> io::Result<Vec<String>> {
    let name = text(component, "name")?;
    let version = text(component, "version")?;
    if component.get("declared_licence").is_none()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        || !version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+' | b'_'))
    {
        return Err(invalid(
            "component needs an explicitly reviewed source scope",
        ));
    }
    Ok(vec![format!("vendor/{name}-{version}/*")])
}

fn archive_hash(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 65536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(
            buffer
                .get(..count)
                .ok_or_else(|| invalid("archive read exceeds buffer"))?,
        );
    }
    Ok(digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
