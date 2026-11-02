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


## Immutable prebuilt preparation — 2026-11-02

The preparation CLI now seals the native-platform prebuilt application, the running helper and the complete approved tool tree. Use package prepare windows|linux vMAJOR.MINOR.PATCH INPUT TOOLS FRESH-STAGE, retain the printed SHA-256 independently, then package verify STAGE SEAL. Tag spelling/MSI limits are checked; signed-tag authentication remains the release builder's separate obligation. These local spikes are not releases.

Preparation checks the x64 PE/ELF header without executing the app. It requires the existing pinned Small detector/recogniser/dictionary, all five unmodified fonts and their source-specific licences, and the approved sibling ONNX Runtime file. Only selected baseline assets are accepted; research Tiny/Medium profiles are not package defaults. The archive filename policy also governs staging paths. Mutable data is never copied, and an existing token file is refused. No token is read or inserted by preparation.

The fixed installation layout is tallyline.exe plus onnxruntime.dll on Windows, or tallyline plus libonnxruntime.so.1.31.0 on Linux, with assets/fonts/, assets/ocr/small/, legal/, LICENSE, NOTICE and README.md. The old copier's missing native runtime was caught by a real prebuilt-payload red check and corrected. The actual Windows app was freshly built under Rust 1.99.0 and inspected as x64 PE without launching it. A corrected 427-file prepared payload matches every independently recorded input byte and excludes its synthetic mutable-data sentinel. This is staging evidence, not protected MSI acceptance.

Package legal-verify PAYLOAD binds every physical notice to the catalogue text, checks exact registry index identities and rejects missing, extra, changed or linked material. Preparation additionally binds the project GPL/NOTICE and the two source-specific font licences to their delivered legal copies. The original IBM licence CRLF bytes were restored from the selected immutable upstream to its already declared hash; a real stale normalised legal copy first passed the old verifier and now fails closed. The generated attribution symbol was corrected to match the original notice. No font version, font bytes, terms or native activation changed.

A different helper, tool inventory, payload byte, permission or unlisted path invalidates the retained seal. Full source-duty map validation, protected fixture insertion and the MSI/Debian backends remain in progress. Final source/app/helper/legal artefacts must be frozen together again before the final fixture is created.

## Intrinsic legal identity and source evidence — 2026-11-02

Physical notice consistency alone does not identify the approved component graph. The helper now compiles exact target catalogue profiles (217 Windows components and 271 Linux components), bound to canonical project source inputs. package legal-verify windows|linux PAYLOAD checks both physical notice bytes and that intrinsic identity; package legal-verify PAYLOAD retains the general consistency-only diagnostic. Prebuilt preparation and sealed verification require the intrinsic profile. The actual CLI passed both current generated target inventories; a self-consistent synthetic catalogue is rejected. These counts are source-only evidence and do not prove a completed distribution.

package source-map PAYLOAD UNPACKED-SOURCE ARCHIVE FRESH-OUTPUT records exact component source files. The archive must match the current pinned deterministic tar.gz profile with its tallyline/ root; the helper streams a fresh archive identity without keeping a second archive. A real red check exposed acceptance of changed unpacked source against an older archive; it now rejects that mismatch before writing a map. An independent tar reader and the published SHA-256 vector for abc verify the selected source file. Current registry scopes are implemented; complete native scopes and protected map validation remain open. No final fixture is created by these commands.

All actual Linux source/native gates passed offline under Rust 1.99.0 before a release build from 319 hash-bound inputs. That active build uses the earlier schema-six source snapshot; the schema-seven integration requires its own fresh combined application build. Existing Windows app/stage evidence also predates the final source freeze. Neither is final package evidence.

## Source admission and fixture ordering — 2026-11-02

The source.tar.gz archive profile fixes every source file to mode 0644 and directories to 0755, independently of host checkout permissions. Native source scopes are compiled from supply-chain/package-source-scopes.json and match exact reviewed component identities. Registry scopes use locked versioned vendor directories. The project source lives under project/ so its REUSE annotations do not claim vendor, toolchain or native source siblings. Source-specific font, model, dictionary, artwork and native notice scopes remain distinct. These scope rules do not authenticate an arbitrary upstream archive or prove that every optional native dependency has been archived.

Package source-verify PAYLOAD ARCHIVE checks the map against the real compressed archive, including every mapped file hash, canonical USTAR metadata, safe paths, duplicate/case aliases, directory ancestry, padding and stream termination. It reads data without extraction or execution. Preparation accepts only source-capsule.tar.gz with its complete legal/source-map.json; it validates that archive before staging and seals its identity in manifest schema 2. The source archive remains a separate source-job artefact and is not copied into the protected stage. A diagnostic stage without a source reference may be verified but cannot admit a fixture.

Package inject STAGE SEAL FIXTURE and package verify-injected STAGE SEAL FIXTURE verify immutable helper, tool, payload and source-map bindings before fixture access. Insertion creates only payload/tallyline-app-token and scans for exact bare and wide bytes outside that path. The harmless fixture must fit the current application's ASCII-graphic, 1–1024-byte token envelope; this is not a verified claim about the real UEX token format. A bad seal is rejected before attempting to open even a missing fixture. These commands do not implement or prove MSI/Debian assembly.

The normal cargo xtask check now checks canonical Cargo.lock, NOTICE and supply-chain/notices.toml against the compiled legal source bindings before quality build/test work. Package source-bindings SOURCE-ROOT exposes that same bounded data-only check. A graph-neutral lockfile comment still invalidates the exact source binding. Rebinding requires regeneration and independent comparison of both target catalogue bytes; unchanged graphs alone do not justify retaining their hashes.

Corrected 2026-11-02: fresh schema-seven Windows PE and Linux ELF application builds are retained without launching either app. Their earlier source bases are recorded in external receipts; the pending final combined source/helper/app/legal freeze must rebuild or independently establish applicable unchanged inputs after integration. Neither those builds nor source-scope contracts complete #97, installed acceptance or #98 publication.
