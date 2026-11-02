# Offline prebuilt packaging spike

> **Doc type:** Living reference — in progress. Last reviewed: 2026-11-02.

Issue #97 covers R-NF-2:installers, R-DOC-3, R-DOC-4, R-SEC-8 and R-NF-4. This new branch starts from the public M1–M5 integration, preserves Rust 1.99.0 and imports no held toolchain/dependency/native records. The owner-selected MSI direction is [ADR-0011](adr/0011-source-compiled-wix-packaging.md). Earlier Rust 1.99.1 packaging experiments are historical research; their outputs are not current package evidence.

## Implemented foundation

The prebuilt xtask executable handles these commands before source-root discovery, outside a checkout and without Cargo on PATH. During secret-free preparation only, prefix the same arguments with cargo xtask.

| Command | Contract |
| --- | --- |
| package version vMAJOR.MINOR.PATCH | Canonical stable tag spelling only: no leading zeroes, signs, prerelease/build suffixes or whitespace. MSI limits 255.255.65535. It authenticates no tag signature or protected-branch ancestry. |
| package tools-verify windows DIRECTORY | Every approved source-built WiX/private-runtime file must match the compiled 220-file manifest; missing, changed, extra and linked files fail. No tool executes. |
| package tools-verify linux DIRECTORY | Only the exact reviewed Rust 1.99.0 cargo-packager 0.11.8 binary is accepted. No tool executes. |
| package archive zip INPUT FRESH-OUTPUT | Plain stored ZIP under tallyline/, fixed timestamps, sorted paths and Unix permission attributes. No ZIP64. |
| package archive tar.gz INPUT FRESH-OUTPUT | Plain USTAR/gzip under tallyline/, fixed timestamps and root ownership, sorted paths and retained Unix permissions. USTAR path limits apply. |
| package scan UNPACKED-TREE FIXTURE [ALLOWED-RELATIVE-FILE] | Exact UTF-8 and UTF-16 LE/BE bytes fail outside the sole exact data file. Missing/changed allowed data fails. No fixture value is echoed. Compressed containers require independent unpacking first. |

Archive destinations must be fresh and outside the input tree. Links and special input files are rejected; a failed write removes its own partial output. ZIP contents currently use memory proportional to the complete archive; this foundation is intended for bounded prebuilt application payloads, not arbitrary large source capsules.

The archive implementation and scanner were reviewed and selectively reused from the preserved earlier packaging branch. The new intrinsic tool gate addresses that earlier API's provenance limitation: a self-consistent stage seal alone cannot approve arbitrary tools. [Tool notes](../supply-chain/wix/README.md), exact source/profile patch and delivered file hashes remain distinct from authentication of an arbitrary rebuild.

Initial standalone CLI tests establish real red/green for canonical tag limits, outside-checkout/PATH-free archive writing with independent platform archive readers, tool mismatch rejection and bare/wide fixture scanning without diagnostic disclosure. Additional failure contracts preserve existing output and remove partial output. Both actual retained approved tool trees pass the new verifier. These are foundation checks, not actual installer acceptance.

## Remaining acceptance

- Complete reviewed application/native/model/dictionary/legal/source payload inventory and immutable prebuilt handoff, with exact package identities and strict mismatch rejection.
- Source-free, network-denied protected MSI and Debian assembly using the same verified prebuilt helper, with no Cargo/compiler/build-script/macro/hook/download path after token injection.
- A fresh harmless fixture created after final compilation, scans of all relevant logs/intermediate builds, independent final unpacking and sole-data-file checks for MSI, ZIP, deb and tar.gz.
- Public primary evidence for the actual UEX token lexical shape is not established. The scanner is deliberately format-independent; the opaque synthetic test shape is not claimed as actual-format verification. No real token is needed or read.
- Actual DEP-5 semantics/permissions, installer streams/components, complete notices and corresponding-source duties for the final current graph.
- Clean-OS install/upgrade/uninstall, native X11/Wayland/Windows display, data retention and applicable no-extra-prerequisite acceptance.

The separately installed Visual C++ Redistributable prerequisite of ADR-0009 remains unresolved against no-extra-prerequisite acceptance. Forbidden MSVC/UCRT DLLs must not be bundled. Headless byte inspection proves no installed-system or native-display result. No OS installation, GUI launch, release, tag, secret access or publication is performed here. Issue #98's workflows/provenance/publication remain separate.

The design follows [the selected P4KX reference](ci-release-reference.md): secret-free build/tool preparation, immutable verified prebuilt handoff, protected data-only insertion, final artefact validation and separate publication. No P4KX implementation was copied, no new upload service was added and no reverse-improvement issue is proposed without supported evidence.


## Verification topology correction — 2026-11-02

The initial full Windows check exposed a real self-hosting limitation: adding a Cargo integration-test target caused Cargo to rebuild the executing target/debug/xtask.exe during the tests and requirement-tag steps, which failed with access denied. That failed evidence is retained. The automated file/validation/archive contracts now call the public packaging library API and remain in the full check; no test or checker gate is excluded, and no binary is copied or renamed as a workaround. The actual thin CLI is verified separately through Cargo after the checker exits. Initial actual CLI contracts passed on Windows and Linux; the final source must receive the same standalone proof after its canonical check.

Linux network-denied/offline red/green contracts exposed and fixed Windows parent-path separators, case aliases, reserved device names, trailing-dot/space normalisation and illegal filename characters. Both writers share their filename policy, based on [Microsoft naming conventions](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file). The reviewed tool manifest retains its original byte encoding explicitly in .gitattributes; its Git blob and compiled identity must match exactly.
