// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Verifies fixed baseline OCR data before compilation.

use std::io::{self, Read};
use std::path::Path;

use sha2::{Digest, Sha256};

const HASH_BUFFER_BYTES: usize = 64 * 1024;

struct Pin {
    key: &'static str,
    file: &'static str,
    sha256: &'static str,
    bytes: u64,
}

const PINS: [Pin; 3] = [
    Pin {
        key: "TALLYLINE_SMALL_DETECTOR",
        file: "PP-OCRv6_small_det.onnx",
        sha256: "d73e0058b7a8086bbd57f3d10b8bcd4ff95363f67e06e2762b5e814fe9c9410e",
        bytes: 9_880_512,
    },
    Pin {
        key: "TALLYLINE_SMALL_RECOGNIZER",
        file: "PP-OCRv6_small_rec.onnx",
        sha256: "5435fd747c9e0efe15a96d0b378d5bd157e9492ed8fd80edf08f30d02fa24634",
        bytes: 21_159_378,
    },
    Pin {
        key: "TALLYLINE_SMALL_DICTIONARY",
        file: "ppocrv6_dict.txt",
        sha256: "b5f2bfe2bdd9448429e3e82b51c789775d9b42f2403d082b00662eb77e401c5d",
        bytes: 74_947,
    },
];

fn main() -> io::Result<()> {
    let root = std::env::var_os("CARGO_MANIFEST_DIR")
        .ok_or_else(|| io::Error::other("Cargo did not provide the OCR package directory"))?;
    let root = Path::new(&root).join("assets/small");
    for pin in PINS {
        verify(&root, &pin)?;
        println!("cargo:rerun-if-changed=assets/small/{}", pin.file);
        println!("cargo:rustc-env={}_FILE={}", pin.key, pin.file);
        println!("cargo:rustc-env={}_SHA256={}", pin.key, pin.sha256);
        println!("cargo:rustc-env={}_BYTES={}", pin.key, pin.bytes);
    }
    println!("cargo:rustc-env=TALLYLINE_SMALL_DICTIONARY_ENTRIES=18708");
    Ok(())
}

fn verify(root: &Path, pin: &Pin) -> io::Result<()> {
    let file = std::fs::File::open(root.join(pin.file))?;
    if !file.metadata()?.is_file() || file.metadata()?.len() != pin.bytes {
        return Err(io::Error::other(format!(
            "OCR asset {} has an unexpected size",
            pin.file
        )));
    }
    let mut source = file.take(pin.bytes + 1);
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; HASH_BUFFER_BYTES];
    let mut bytes = 0_u64;
    loop {
        let count = source.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let chunk = buffer
            .get(..count)
            .ok_or_else(|| io::Error::other("invalid OCR hash buffer length"))?;
        bytes += count as u64;
        digest.update(chunk);
    }
    let actual: String = digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if bytes != pin.bytes || actual != pin.sha256 {
        return Err(io::Error::other(format!(
            "OCR asset {} does not match its reviewed hash",
            pin.file
        )));
    }
    Ok(())
}
