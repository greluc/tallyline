# Mutation evidence admission

> **Doc type:** Living reference — implemented guard, replacement measurements pending. Last reviewed: 2026-11-02.

A successful mutation-tool exit and its `caught` or `unviable` counters do not establish a usable score. Each completed selection needs its original cause logs as well as its exact source, selected mutant identities, baseline and outcomes. An unrelated workspace test failure invalidates the affected measurement.

`cargo xtask mutation` and `cargo xtask mutation-aggregate` check the retained logs before accepting counts for scoring. Every caught or unviable outcome must name a readable UTF-8 regular file directly below its `mutants.out/log/` directory, at most 16 MiB. Missing files, escaping paths and visible symlinks are refused. This is a local evidence boundary, not a claim of race-free access to an adversarially mutable directory.

The guard admits a caught outcome only with a named failed test and a failed test-run result. A test returning a domain error is valid evidence; a panic is not required. An unviable outcome requires a coded Rust compiler diagnostic. ANSI colour sequences are removed before checking; unsupported escapes are refused.

Known independent infrastructure diagnostics invalidate either class, even when a genuine Rust diagnostic is also present: executable-file busy, exhausted disk space, linker failures including LNK1104, and missing or incompatible Rust crate artefacts. The log scanner is a conservative admission check for these known failures. It cannot establish that every possible failed test was caused by the selected mutation; suspicious failures still require source and cause review.

The exact selection, all locked workspace variant tests, successful baseline, outcome completeness, timeout rejection and frozen 100% whole-selection floor remain binding. Invalid evidence is not repaired by dropping affected mutants, calling them unviable, stitching retries into a run or changing the denominator. Retain the failed run and repeat the complete relevant selection after fixing its cause. An all-unviable whole selection has no measured score and fails.

## Empty selections

Pinned cargo-mutants 27.1.0 reports a successful diff-filtered empty selection with zero stdout bytes and exactly one of three stderr messages: `INFO No mutants to filter`, `INFO Diff changes no Rust source files` or `INFO Diff file is empty`. These are the three successful early exits in the [pinned diff-filter source](https://github.com/sourcefrog/cargo-mutants/blob/v27.1.0/src/in_diff.rs), emitted before JSON listing by [the pinned entry point](https://github.com/sourcefrog/cargo-mutants/blob/v27.1.0/src/main.rs). The wrapper accepts those exact normalised protocols as `[]`, retains both empty selection files and visibly skips without a baseline or score. Other empty output, malformed JSON, non-array JSON and an unsuccessful listing still fail. A valid JSON array is preserved.

## Withdrawn historical claims

Corrected 2026-11-02: earlier full hosted runs reported 100%, but an independent review found unrelated OCR loader fixture failures among their reported catches. Those full-run scores are withdrawn. The Rust-unviable counts below were reviewed separately and do not absorb the infrastructure failures.

| Source head | Selected | Reported caught | Genuine test failures | Independent fixture failures | Rust-unviable |
|---|---:|---:|---:|---:|---:|
| `12f6ca99187c3cf214fb5b2d5931746296aea82b` | 757 | 567 | 548 | 19 | 190 |
| `208d09282f04e21219bff8ac29183993f0344f1a` | 700 | 524 | 506 | 18 | 176 |
| `93c06e12d5eb5c892a9a747f8d4c97b9566ee951` | 588 | 438 | 418 | 20 | 150 |
| `d9a172f2861deef59e1b118d2b40ec0978ff32dc` | 619 | 462 | 450 | 12 | 157 |
| `d78f5b7b5e2332a9f52151141c74e4de7edd0dbb` | 757 | 567 | 544 | 23 | 190 |

The first four runs contain 69 independent fixture failures; the separate d78 run contains 23. Each affected log stops at the native-free OCR fixture with `ExecutableFileBusy` / `Text file busy`. Their raw counts remain evidence of what ran, not accepted mutation scores. The first four runs' complete cause audit has SHA-256 `867bfb8b8afea292d0e7764eebe7e091151745ac58c41ed01037ddf8e8c9199d`; the separate d78 audit has SHA-256 `813a89503ad667e93fe12db73ba4e006e810c07bb0ab6a2f20112dac8a35f9ad`. Both evidence packets remain outside the repository.

The review covered every selected outcome, retained cause log and complete partition. Genuine domain-error-returning tests were retained as genuine catches. It checked available archive identities without inventing missing provenance: the first two runs had their archived ZIP bytes and server digests; the third had extracted logs without the ZIPs or metadata; the fourth had metadata without ZIP bytes. The d78 run's eight downloaded archives were verified against their server digests before bounded text-only extraction.

A separate local diagnostics selection of 51 mutants also reported 100% on its repeat, with 46 catches and five reported unviable outcomes. One of those five is an unrelated LNK1104 failure opening a test executable, not a Rust type error. That measurement is also invalid. Its cause is unproved; antivirus activity or leaked fixture handles must not be presented as established explanations.

The initial eight-mutant workflow baseline documented in the engineering principles is a distinct historical measurement. The frozen floor is unchanged. No replacement full current-source measurement is established by this guard or by the historical recount.

## Verification boundaries

File-backed contracts reject the reproduced executable-busy catch, a linker failure, mixed compiler/linker output, missing or escaping logs and unrecognised empty-list output. They retain genuine assertion failures, domain-error test failures and coloured Rust type diagnostics. Aggregate contracts exercise the public file-backed admission seam with exact shard evidence. The actual d78 artifact packet is also refused by the public aggregate command after this correction.

These checks run without a native OCR library or model. They do not establish a fixture fix, native runtime acceptance, model accuracy or a new whole-core mutation score.
