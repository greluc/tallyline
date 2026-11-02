# Prebuilt package verification

> **Doc type:** Historical record — completed development experiments; current release and installation acceptance pending. Last reviewed: 2026-11-02.

## Historical Development fixture

The first complete application-package experiment uses the actual Windows and Linux Development `0.0.0` applications built at source revision `10ec53bcf6a479eee4d0a0ce4ec25c37f993e167`. These are not current-head applications, authenticated releases or signed-tag evidence. The separately verified source capsule has SHA-256 `17eb58c11357cf5c09399a8acce6cfabbbfaddd4afb85caa1860146c7630797b`; its project, locked vendor and reviewed native/source-duty inventory was verified before fixture insertion.

| Artefact | Bytes | SHA-256 |
| --- | ---: | --- |
| Windows MSI | 40565365 | `278e387f4936a188736cb46b6311b41b3fde6771599e80280e1827c24a69473f` |
| Windows ZIP | 83414056 | `d4d245c24f7cbbad3c9cb76a7a694f2379f6d6c3ee252f3b0fc647ca5d0dbd9f` |
| Linux deb | 52302710 | `c389eff19a9c9e8f7c55e0afa308c54b508240edc81ed674a2850079a3c4b5bd` |
| Linux tar.gz | 51998080 | `d2971405b9620c1844ed7e40895a1358759033d5fdb5cde6d3fe51a2efcc56df` |

Both native assembly jobs exited successfully using verified prebuilt helpers and approved tools. Fresh harmless fixtures were created after compilation and inserted only as the separate token data file. No real credential or UEX token was read. The fixture satisfies the application's graphic-ASCII envelope; the real UEX lexical format remains unverified.

The Linux protected container used the existing runtime image `sha256:c84a0d691a0b1f80e3523e0c450c3ad45ec53f00946847a924d5488dc96331c2`, network mode `none`, a read-only root, dropped capabilities and no-new-privileges. Its only mounts were the sealed stage read-only, fixture read-only and owned output writable. No project source, source capsule, Cargo cache, Cargo or compiler was mounted or available in the audited runtime image. The same prebuilt helper performed insertion and final assembly; hooks were null and packaging tools were sealed before fixture access.

The Windows helper ran outside the checkout with a cleared environment and empty outer PATH; its handoff contained only approved data and prebuilt code. Its checked WiX child used the private runtime and fixed system-tool path. This does not prove operating-system denial of host source/compiler access or networking. That Windows isolation gap remains open.

## Independent inspection

The ZIP reader checked all 428 regular payload files against the sealed inventory, including exact bytes, safe unique paths, normalised times and modes. A separate read-only Windows Installer query found 428 files/components, x64 component flags, zero custom actions, zero embedded storages and only the cabinet and summary-information streams. An independent cabinet parser verified every member name and size before the built-in cabinet reader extracted them; all extracted hashes matched. The private fixture occurred only in its designated data file. Retained text/logs were clean, and intermediate cabinets matched the independently checked cabinet.

The actual MSI's 362 launch conditions include the 361 authored CRT rows and standard downgrade refusal. Restricted read-only engine evaluation passed 467 canonical positive/negative cases, totalling 169054 actual condition evaluations, with zero mismatches. Stored messages include the official Microsoft download page. This did not execute AppSearch, installer action/sequence APIs, installation or UI. A forward-slash controller path was refused; independently normalising the Windows path fixed the controller, and both attempts remain in the external evidence.

Independent ar/tar readers checked the Linux payloads, root ownership, fixed timestamps, data/control modes, sole fixed `usr/bin/tallyline` symlink, every control checksum and the 114292 KiB installed-size accounting. The Debian control archive contains only control metadata and checksums, with no maintainer scripts. The standard `dpkg-deb --info` reader also accepted it. All 271 component notice sets and their source scopes survived DEP-5 folding; original delivered legal-file hashes also matched. This is not Debian archive-maintainer or clean-OS installation acceptance.

Actual prebuilt scanner calls accepted independent ZIP/cabinet extractions and refused controlled bare, UTF-16LE and UTF-16BE leaks without disclosing fixture bytes. Repeated plain ZIP and tar.gz assembly from the same payload and fixture produced byte-identical archives. This establishes neither compiler nor MSI reproducibility. No application/model execution or installation occurred.

## Metadata corrections

The historical MSI build exposed unnecessary `DefaultLanguage=0` warnings for unversioned files. Removing that fallback preserves WiX's actual file metadata and makes the real corrected build quiet. Its separately recorded corrected helper used the historical application/source payload; it is not a new current-source application build.

The historical Debian desktop entry referenced `Icon=tallyline` although no such icon was packaged. The corrected adapter writes a fixed desktop data file without that optional reference and disables the packager's default template. A real protected Debian/tar rebuild and independent inspection passed: launcher target, existing executable/resources, modes, checksums, source bindings and full legal notices remained intact. Both corrections have authentic artefact-level red/green evidence rather than comparisons against their own generators. Initial independent-auditor COM null-row and empty-log assumptions were rejected and corrected; failed controllers/receipts remain retained.

## Remaining gates

A new combined-source application/helper/source-capsule freeze and rebuild must precede the final current fixture and artefact set; historical or mixed-helper fixture receipts must not be relabelled. Authenticated signed-tag/CI provenance and publication belong to issue #98. Real UEX format confirmation, Windows OS isolation, clean-OS install/upgrade/uninstall, native Windows/X11/Wayland display and applicable permissions/prerequisite acceptance remain open. Windows OCR still requires the separately installed Visual C++ Redistributable; forbidden Microsoft runtime DLLs are not bundled. The unresolved no-extra-prerequisite and legal System Library review gates remain unchanged. No release, tag, upload or workflow dispatch follows from this record.
