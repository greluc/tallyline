# Review of the plan (2026-10-08)

> **Doc type:** Historical record — corrections are new dated sections. Last reviewed: 2026-11-01.

All documents under `docs/plan/` were reviewed, plus `CLAUDE.md` and `README.md`. The criteria were completeness, factual errors, logic gaps and contradictions between the documents. The findings have been corrected directly in the documents. This list records **what** was wrong or incomplete, so that the changes remain traceable.

## A. Factual errors (corrected)

| # | Finding | Correction |
|---|---|---|
| A-1 | `sealed interface Reader permits OcrReader, VlmReader` in `domain`, but the implementations in the modules `ocr` and `vlm`. **Does not compile:** in named JPMS modules, permitted subtypes must be in the same module. | `Reader` is a normal interface; `ReaderKind` as an enum (02 §4b, CLAUDE.md rule) |
| A-2 | The price regex in 07 §2.4 rejected exactly the case it was intended for (`96705/SCU`), and all numbers without thousands separators | Procedural parser with candidate generation (07 §2.4) |
| A-3 | ADR: "iced has no mature table widget" – **wrong**. `iced_widget` 0.14 contains `table.rs` (checked in the crate source code). | Statement corrected; the assessments of egui and slint are marked as unverified. The decision does not change. |
| A-4 | ADR: basetool "proves" that the OCR approach works – **overstated**. There, PP-OCR is only a digit second reader; the primary reader is a VLM. | Toned down; suitability as the primary reader is measured in M2 |
| A-5 | `InventoryStatus` checked hard for 1–7. That contradicts our own rule "status levels from the API, not from constants". | Check against `ReferenceSnapshot` (02 §3) |
| A-6 | Thread pool `min(2, cores/2)` yields 0 threads on single-core systems | `max(1, min(2, cores / 2))` |
| A-7 | R-NF-1 claimed Wayland operation via XWayland as a given | Marked as an assumption or test point; native Wayland support of JavaFX is not proven |
| A-8 | Rust release date: the build date (2026-09-28) was given as the release date | Clarified |

## B. Logic gaps and contradictions (closed)

| # | Finding | Solution | Location |
|---|---|---|---|
| B-1 | `pipeline → refdata → uex-api` and `pipeline → ocr`: the "pure" pipeline depended on modules with network and file I/O, contradicting guiding principle 3 | `pipeline` now depends only on `domain`. It is passed `ReferenceSnapshot`, `TextDetector` and `ReaderResult`; `app` wires everything. | 02 §2, §4 |
| B-2 | Fusion order contradictory (diagram: fusion → validation; text: validation → fusion) | Defined: per reader parse and resolve → fusion → validate **once** → stitching | 02 §4b, 07 §2.7 |
| B-3 | **Send threshold** not defined anywhere; confidence values without meaning for the submission gate | Send threshold 0.80 as a term and in the confidence table | 01 Terms, 07 §2.6 |
| B-4 | Prior-based repair could **silently "repair" real price changes back to old values**, and that with 0.85, i.e. sendable | Repairs automatic only with an independent witness, otherwise 0.75 → confirm | R-VAL-2b, 07 §2.5 |
| B-5 | No behaviour defined if **no prior** exists (new commodity at the terminal) | Fallback to the commodity average, otherwise no repair | R-VAL-2a |
| B-6 | Manual capture **prefilled UEX values**. This would allow outdated values to be sent as a fresh report by clicking through; a data quality problem for UEX. | Values are shown as a reference, not prefilled; only actively accepted rows are sent | R-MAN-2 |
| B-7 | Manual reports have no screenshot, but new DataRunners need one (`screenshot_required`) | Screenshot can be attached, the requirement is shown in advance | R-MAN-5, A11 |
| B-8 | A report consists of several scroll captures, but the API only accepts **one** screenshot | Compose the shop crops vertically (assumption A11) | R-SUB-7 |
| B-9 | "Time window" for reports was not defined | Grouping rule: same terminal, same side and environment, ≤ 10 min apart; splitting and merging in the UI | R-OCR-13 |
| B-10 | UEX knows only `live`/`ptu`; the mapping of HOTFIX, EPTU and TECH-PREVIEW was missing | Default mapping plus assumption A9 | R-CAP-3a |
| B-11 | **Patch day:** screenshot before the patch, sending afterwards → wrong game version | Record the version at capture time, warn on change | R-CAP-3b |
| B-12 | `data_parameters` was cached for 1 day, but the acceptance flags change on patch day | Before sending, a state at most 15 min old | R-SUB-5 |
| B-13 | The send queue was not persistent: offline reports were lost on exit | Persistent queue; after a restart send only after release | R-SUB-2, 02 §6 |
| B-14 | Cooldown key inconsistent (01: with side; 02: with side and environment); the UEX rule is unknown | Uniform and conservative (terminal, commodity, environment); `duplicated_report` is treated as cooldown; assumption A12 | R-SUB-4, 02 §6 |
| B-15 | If the original is deleted (cleanup function or user), the image was missing for review, AI and upload | Working copies of the panel crops with a retention period | R-CAP-7 |
| B-16 | "manual" as an environment value per folder was unclear | Options: automatic from path / fixed / ask for every image | R-CAP-1 |
| B-17 | Diagram: manual reports bypassed validation (contradiction to R-MAN-4) | Arrow added | 02 §1 |
| B-18 | Locate needs coarse OCR, but the pipeline must not know `ocr` | Interface `TextDetector` is injected | 02 §4 |
| B-19 | M5 acceptance criteria "≤ 5 s abort" could not be reliably reached with a 5 s check interval | 2 s interval during an AI run | 02 §4b, 04 M5 |
| B-20 | References to non-existent requirements (`R-SCOPE`, tray without a requirement) | `R-SCOPE-2`, new `R-UI-9` | 05 F28/F29 |
| B-21 | CLAUDE.md mentioned "assumptions A1–A6", meanwhile there are A1–A12 | Updated | CLAUDE.md |

## C. Missing parts (added)

| # | Was missing | Added in |
|---|---|---|
| C-1 | Risk register (among others app token as a possible project blocker, UEX terms of use, OCR accuracy, layout patches) | 04 "Risks" |
| C-2 | Check whether the toolchain (Error Prone, NullAway, google-java-format, jlink plugin, jpackage/WiX) runs with **JDK 27** | 04 M0, ADR §4 |
| C-3 | Open API questions: sell SCU field (`scu_sell` vs. `scu_sell_stock`), `container_sizes` per commodity vs. per report, `is_missing` mandatory fields, screenshot composite | 06 Open points 8–14, A10–A12 |
| C-4 | Processing status of own reports at UEX (`data_info`) in the history | R-SUB-4 |

## D. Deliberately left open (decision or verification needed)

- **To be decided by the project owner:** project and package name, licence, JDK 27 vs. 25 LTS.
- **To be clarified against the live API or with UEX:** A1, A2, A9–A12 as well as the open points in 06. This applies especially to `container_sizes`: the sizes differ per commodity in the game. If the field exists only per report, we should rather omit it than falsify it.
- **To be measured on the corpus:** all thresholds (send threshold, confidence levels, tolerances, grouping window), runtime and RAM targets, VLM suitability (A8).
- **To be tested on real systems:** Linux/Wine paths (A3), game detection (A7), XWayland, Secret Service. For libsecret via FFM, note that the store API is variadic; the FFM API supports this, but it increases the effort. An alternative would be D-Bus directly.

## E. Overall assessment (honestly)

- The plan is complete enough to start M0. The biggest uncertainties lie **outside** our control: the app token and UEX's terms of use as well as the exact semantics of `data_submit`. These points should be clarified before any larger implementation, because in the worst case they block the project.
- The second-biggest uncertainty is the **real OCR accuracy** of PP-OCR on terminal screenshots. That is why M2 (measurement) comes before building the complete pipeline.
- The numbers from basetool (VLM speed, Markdown vs. JSON accuracy, recognition rates) come from a different domain (refinery). They are to be understood as starting values, not as a promise.

## F. Addendum: modularisation, maintainability and deviation marking

A second review focusing on modularisation, clean code and best practices yielded the following (module names in sections A–E still refer to the old split):

| # | Finding | Change |
|---|---|---|
| F-1 | `app` combined UI, secret store, AI control and wiring; `submission` and `capture` mixed domain logic (queue, cooldown, policy) with technology (HTTP, SQLite, file system). Use cases were therefore only testable with UI or technology. | New split following **Ports & Adapters**: core `domain`/`pipeline`/`application`, adapters `adapter-*`, presentation `ui`, composition root `app` (02 §2) |
| F-2 | Several modules would have had their own SQLite access (duplication, scattered SQL) | One module `adapter-storage` with repositories; SQL only there (09 §6) |
| F-3 | OS integration (secret store, process monitor, installation detection, truststore) was spread across `app` and `capture` | Bundled in `adapter-platform` |
| F-4 | Best practices were named but not **enforced** | 09: rules with enforcement (JPMS, ArchUnit, Error Prone/NullAway, JaCoCo gate, Dependabot, PR checklist, Definition of Done); M0 sets up the gates before the first domain code |
| F-5 | Time-dependent logic (cooldown, hysteresis, grouping) was not deterministically testable | `java.time.Clock` as an injected port |
| F-6 | Deviations from the previous UEX value were only indirectly visible via the confidence. A **cleanly read but wrong** value (transposed digits in the source, wrong row) would have remained unmarked. | Separate dimension "deviation" with colour marking, Δ display, age of the reference and mandatory confirmation for strong deviation (R-UI-10..12, 07 §2.6, 02 §7) |

## G. Addendum: module cut by bounded context

The project owner asked whether the modules can follow DDD. Assessment and decision are in [ADR-002](../adr/0002-modules-by-bounded-context.md).

In short:

- The core is now cut by bounded context (`capture`, `recognition`, `reporting`, `submission`, `reference-data`, `game`) plus `shared-kernel` and `workflows`. Adapters stay cut by technology.
- `adapter-refdata` was dissolved: fuzzy matching and indexes are domain logic. `adapter-capture` became `adapter-files`.
- Module names in sections A–F above refer to the earlier cuts.

Disadvantages accepted, with handling documented in the ADR:

- more modules
- explicit translation between contexts
- more expensive re-cuts
- the risk that the shared kernel grows

## H. Second full review (2026-10-08)

All documents were re-read after the switch to English and the module re-cut. Findings and fixes:

### Contradictions (fixed)

| # | Finding | Fix |
|---|---|---|
| H-1 | R-VAL-2 said deviations beyond tolerance only warn ("not blocking"), while R-UI-10 requires confirmation for major deviations | R-VAL-2 now: never auto-rejected, but a major deviation must be confirmed |
| H-2 | The goal said manual input is "prefilled with current UEX data", contradicting R-MAN-2 (reference only) | Goal text corrected |
| H-3 | The core must not touch the file system, but `WatchedFolder` used `java.nio.file.Path` and the folder scanner/watcher were placed in `capture` | `FolderLocation` value object in the core; scanner, watcher and stable-file gate live in `adapter-files` |
| H-4 | `recognition` worked on `BufferedImage` (AWT, module `java.desktop`) although the core is supposed to be technology-free | Own `ImageRaster` type in the core; ImageIO only in adapters; ArchUnit forbids `java.awt..`/`javax.imageio..` in the core |
| H-5 | The game process monitor published a "JavaFX property" from an adapter | Publishes via the `GameStateProbe` port; only `ui` maps it to JavaFX |
| H-6 | Retries were assigned to both `submission` and `adapter-uex` | `submission` decides retries; the gateway makes one attempt per call |
| H-7 | `GameEnvironment` was listed in `shared-kernel` (11) and in `game` (ADR-002) | Owned by `game`, used as published language |
| H-8 | Confidence was also lowered for out-of-tolerance values, i.e. the deviation was counted twice | Confidence is independent of the prior band; deviation is its own dimension |
| H-9 | The traceability rule required tests for all M requirements from M3 on, although M requirements are implemented in M1–M5 | Requirement-to-milestone table in 04; the rule applies per completed milestone |

### Logic gaps (closed)

| # | Finding | Fix |
|---|---|---|
| H-10 | The AI re-read could change reports that were already released or queued, and could collide with a concurrent user edit | Only `Draft` reports; `Report.version` for optimistic concurrency |
| H-11 | Catch-up import after a patch: the "version at capture time" would have been the new version | Local history of observed version changes; uncertain versions must be confirmed (R-CAP-3b) |
| H-12 | Manual reports had no defined environment or game version | R-MAN-6 |
| H-13 | Screenshot attachment for manual reports in M1 would have uploaded the full screenshot including the balance, because automatic locate only arrives in M3 | In M1 the user selects the shop-panel region; only that region is uploaded |
| H-14 | M2 should measure raw OCR accuracy per field, but layout and field parsers were only planned for M3 | `recognition` is split into a first slice (M2) and a second slice (M3) |
| H-15 | UEX counts every price row as a report (1000 per 30 min) – not enforced | Rate budget in `submission`; open point 16 in 06 |
| H-16 | Non-terminal screenshots had no defined handling (the predecessor's "failed to process" error) | R-OCR-16 |
| H-17 | Test-mode default unclear for end users | R-SUB-8: on in dev/CI, off in release builds after onboarding, always visible |
| H-18 | No defined home for user settings, upload encoding and update check | Ports `*SettingsStore`, `ImageEncoder`, `UpdateCheck` (02 §2) |
| H-19 | Units of `price_variation`/`scu_variation` unconfirmed but used as thresholds | Assumption A15; configurable conservative defaults |
| H-20 | `data_remove`/`data_info` parameters unknown although withdrawal (I4) depends on them | Open point 15 in 06 |
| H-21 | No "delete all local data" action for a tool that stores secrets and screenshots | R-NF-9 |
| H-22 | Smaller errors: corpus structure in 07 did not match the real corpus; prompt path pointed to an old module name; requirement order within tables; German mermaid ID | Fixed |

### Still open (unchanged, outside the plan's control)

- UEX app token and terms of use (A2) remain the biggest risk.
- The exact `data_submit` semantics remain unverified: header, `container_sizes`, sell-side SCU, duplicate lock, environments (A1, A9–A12, A15).
- All thresholds are start values until measured on the corpus.

## I. Third review (2026-10-08) – three independent reviewers

Three reviewers ran in parallel, without knowledge of the earlier reviews: cross-document consistency, technical correctness (claims checked against JDK, Gradle, GitHub, ONNX Runtime, sqlite-jdbc and Ollama sources), and completeness along end-to-end scenarios. They produced about 75 findings. A sample of the technical claims was re-checked against primary sources before changing anything:

- `actions/attest` README: `artifact-metadata: write`, and public repositories only on small plans.
- `gradle/actions` dependency-submission docs: `contents: write`.
- ONNX Runtime 1.30.0 source: `onnxruntime.native.path`.

The most important fixes:

| Area | Finding | Fix |
|---|---|---|
| JPMS | Repository ports in `api` named aggregates hidden in `internal`, so adapters could not implement them | Export `api` + `api.model`; aggregates protected by the compact constructor plus an ArchUnit constructor rule (02 §2, ADR-002, 09, 11) |
| Ownership | Recognition produced a Reporting aggregate (`Report`) although the context map forbids it | `Stitcher` → `StitchedScan` (recognition); `ReportGrouper` builds the `Report` (reporting) |
| Fusion | The prior alone could decide between OCR and VLM, contradicting R-VAL-2b | The prior alone gives at most 0.75 (confirm); glyph-topology role defined once (witness when agreeing, veto when contradicting) |
| Submission lifecycle | Unknown outcome → possible double submission; partial `ids_reports`; rejected and withdrawn reports were dead ends; no error classes; cooldown vs. multi-row reports; no correction path | R-SUB-9…12, extended R-SUB-4, report states and transitions in 11 §A3 |
| Data quality for UEX | Old captures sent as fresh data; PTU compared against LIVE priors; patch-day layout drift could produce confident wrong reads; stale snapshot at release | R-VAL-6, R-API-6, R-OCR-19, R-VAL-7 |
| Security/platform | Linux jlink runtime ignores the system CA store; `Windows-ROOT` missing from a minimal jlink image; composite trust manager could silently skip hostname verification; native libraries extracted into shared temp (incl. JavaFX); JEP 472 native-access warnings; locking did not cover buildscript/settings/build-logic; plugins mostly unsigned; dependency-submission and attestation permissions | 02 §8, S-3, S-4, S-14, S-16, S-24, S-26 |
| Robustness | Game detection on Linux via `ProcessHandle` misses Wine (argv[0] dropped); HttpClient cancel is best effort and Ollama unloads only after the request ends; in-process events lost on crash; MDC vs. `ScopedValue`; pyramid scaling wrong | `/proc` primary; cancel + verify via `/api/ps`; transactional outbox; MDC exception; target-height downscale |
| Operations | No release, upgrade/downgrade or patch-day process; no user documentation; no single instance; "portable" undefined; diagnostics redaction unspecified; no path for real misreads into the corpus | `docs/release-process.md`, R-NF-10/11, R-NF-2, R-NF-6, R-DOC-1/2, R-QA-5 |
| Consistency | About 20 smaller mismatches, among them: `Deviation` naming and stale rule; "same PR" vs "separate PR"; R-VLM-3 vs mode "Always"; requirement-to-milestone table gaps; wrong citations (R-UI-4, `data_parameters`, R-SUB-3 tag); annotation count; A6 scope; §-reference; `tools/ocr-eval` dependencies | Fixed in place |

Verified as correct by the technical reviewer:

- ArchUnit 1.5.1 and JaCoCo 0.8.15 read Java 27 class files.
- ScopedValue is final in JDK 25.
- The Gradle option names used are real.
- Dependabot `cooldown`.
- OSV-Scanner supports Gradle lockfiles.
- The ONNX Runtime and sqlite-jdbc artifacts are as described.

Still unverified, and listed in M0: jqwik with JUnit Platform 6; TestFX headless with JavaFX 27; `java-test-fixtures` with JPMS; Gradle 9.8.1 toolchain 27; Wine argv[0] in practice; EAC and `ProcessHandle` on Windows.

## J. Resolution and HDR review (2026-10-08) – three lenses, cross-checked

Three reviewers looked at the earlier resolution/HDR plan (algorithm, measurement, user experience). Every proposal was then checked by a second agent against sources and against the rest of the plan; only proposals that survived were applied. Overlapping proposals were merged into one version per requirement.

| Area | Finding | Fix |
|---|---|---|
| Scale gate | The gate used anchor height with a 10 px start value. A preliminary downscale experiment (third-party PP-OCRv6 small export, one unverified entry) showed that the price digits are the limiting glyph and that the limit lies lower | R-OCR-17: gate on the **price-digit cap height** (smallest fully visible card), start values < 6 px `TextTooSmall`, 6–8 px `SmallText`, ≥ 8 px normal; calibrated in M2 with the official export |
| Anchor search | A fixed factor or a detector `max_side_limit` (PaddleOCR default 4000) can shrink anchors below legibility on ultrawide or distant captures | 07 §2.1 item 1: factor min(1, 1080/height), native-resolution retry, tiling instead of a size limit |
| Large inputs | 7680×4320 DSR captures could exceed the heap budget | Pixel budget with decode-time box reduction and `Downscaled` finding; resource test |
| Recogniser input | Recognition crops came from the resampled normalised panel; preprocessing parity with PaddleOCR training was not specified | Crops from the original raster; `INTER_LINEAR`, BGR, normalisation and padding pinned by a golden test (07 §2.1 item 4) |
| Colour decoding | `ImageIO.read` skips colour chunks; the JDK does not apply `iCCP`/`gAMA` and does not know `cICP`, so PQ PNGs would be read as washed-out sRGB | R-CAP-8: magic-byte detection, explicit colour metadata, PQ/HLG never read as sRGB, JXR/AVIF/EXR always listed with tool-specific hints; decode fixtures (07 §4) |
| Tone handling | One "washed out" finding mixed recoverable low contrast with lost detail; the stretched panel was also fed to the recogniser | Tone classes `NORMAL`/`LOW_CONTRAST`/`PQ_SUSPECTED`/`CLIPPED`; findings `LowContrastCapture` and `ClippedHighlights`; recogniser gets the un-stretched crop by default; colour decisions relative within the panel |
| Confidence | A line-mean OCR score hides one uncertain digit; letters like B/O/S inside numbers were not handled | Per-digit minimum probability (07 §2.6); letter–digit confusables (07 §2.4) |
| Measurement | Corpus classes had no minimum sample sizes; "0 silently wrong" on a few fields looked like proof; HDR and resolution could not be tested without the hardware | R-QA-2 ladders and text-height bands; R-QA-3 class status and rule-of-three bound; synthetic class variants, metamorphic gate, capture protocol, schema version 2 (07 §4) |
| User guidance | HDR and framing advice was one phrase; hints could nag on every capture | R-DOC-1 capture tips (each marked "verified on game version X"); non-modal per-source hints (R-OCR-18); "Check a screenshot" (R-CAP-9) |
| Smaller fixes | Invariant count I1–I5 vs. I6 in four documents; R-OCR-2 and F7 still named colour anchors | Fixed |

Rejected: a proposal on folder-name casing (out of scope for this review).

**Still unverified** (marked in the documents): what the SC screenshot key stores with HDR on (A16); the Game Bar PNG tone curve; the NVIDIA overlay output; all start values. The planning experiments are not reproducible from the repository yet; M2 repeats them with the official, SHA-pinned model export.

## K. Fourth review (2026-10-08) – six dimensions, adversarial verification, completeness round

Six reviewers read the whole plan, each along one dimension: cross-document consistency, domain logic, end-to-end scenarios, OCR logic, technical correctness, and security and privacy. A completeness critic then looked for areas that no finding had examined and started a second round on three of them: the design-system prompt (`docs/prompts/design-system.md`) against the plan; settings, persistence and the data lifecycle; and resource use and attention next to the running game. Three independent skeptics checked every finding against the text. **127 findings were confirmed** (102 by all three skeptics, 25 by two; 14 high, 75 medium, 38 low; 96 from the first round, 31 from the second) and **11 were rejected**. Before each fix, the finding was re-read against the current text, because the resolution/HDR changes (section J) and the invariant sync had landed in the meantime. Fixes that touched the same requirement row or section were merged, so that each was rewritten once. Technical claims the fixes rely on were checked at the source, among them the OpenJDK 27 and OpenJFX 27 sources, the Gradle 9.8.1 docs, ONNX Runtime 1.30.0, the Ollama source and API docs, the `setup-java`, `actions/attest` and OSV-Scanner releases, JUnit's tag rules, and third-party recordings of UEX responses (marked [3P] in 06). 117 findings were applied in full and 5 in part; 2 concern mainly the design-system prompt (with one small plan change each), and 3 were deferred to the licence and brand review.

| Area | Finding | Fix |
|---|---|---|
| Game version | `game_versions` was cached for a day and R-CAP-3b relied on "the app was not running in between", so captures after a patch got the old, already validated version and the R-OCR-19 cap did not fire, although release-process.md said it would; the observed-version history had no persistence port | Extra fetches at startup, before an import batch is versioned, on game start and at most 15 min before every send attempt; a persisted history per UEX environment (first/last seen, only from successful fetches) with the classes certain, provisional, uncertain and unknown and the choice keep/switch/discard; port `GameObservationRepository`; R-OCR-19 is re-evaluated on every version change, applies to the fused value and covers PTU; the UEX publication lag is a recorded gap (R-CAP-3b, R-SUB-5, R-OCR-19, new A17, 06 `game_versions` row, 02 §2, release-process.md patch day) |
| Game version | I2 required the version to match the UEX acceptance state while the version is fixed at capture, so a confirmed pre-patch report looped release → `invalid_game_version` → Draft | The version is the newest observed one or resolved by the R-CAP-3b choice; a report is never relabelled; `invalid_game_version` is permanent with "discard" as the only action; whether UEX accepts an older version is open (I2, I5, R-SUB-11, 11 §A3, new A18, new 06 open point 21) |
| Game version | Validating a new game version was vacuous: new-version captures could stay unverified, and the only corpus entry has `gameVersion: null` | "Validated" is defined: verified captures of the profile's theme on that version, buy and sell, no silently wrong field, accuracy not below the frozen baseline, evaluated with the cap disabled; unverified and `null` entries never validate a version (R-OCR-19, release-process.md patch day steps 2–3, 04 M3, corpus/README.md) |
| Environment | A fixed per-folder environment or the manual default LIVE could label PTU data as LIVE without any independent check | The process monitor records the running game's channel; a mismatch sends the capture to "environment pending" with a suggestion that is never preselected; manual reports preselect the last explicitly chosen environment; the editor shows the environment with its source (new R-CAP-10, S priority, M4; R-MAN-6, R-UI-2) |
| Priors and deviation | The stale-reference rule lowered `MAJOR` to `MINOR` for I2, so a cleanly read factor-10 misread against an old prior could be sent unchecked | `MAJOR` is never lowered; an outdated reference only displays `MINOR` like `EQUAL`, with "reference outdated"; I2, the gate and R-VAL-7 use the unlowered level; decimal prices only where the layout profile allows them (`decimalPrices`) (07 §2.6, I2, R-VAL-2, 05) |
| Priors and deviation | "Confirmations stay valid unless the deviation level gets worse" (R-VAL-7) could not be evaluated: a confirmation stored only the value, and no order involving `NO_REFERENCE` existed | A confirmation records the deviation level it was given against; `Deviation.isWorseThan` is defined once; release recomputes only the deviation part and refuses with `DeviationWorsened` (R-VAL-7, I3, I6, 07 §2.6) |
| Priors and deviation | Repair scoring used "last value or `avg_week`" while the deviation used the latest value; the no-prior substitute versus `NO_REFERENCE` was unresolved; LIVE averages and `is_available_live` were applied to PTU; container sizes had no deviation rule | One `PricePrior` (latest UEX value with its age) for repair and deviation, with the same effective tolerance; the commodity average is a coarse reference (beyond twice the tolerance `MAJOR`), taken only from the report's environment; container sizes compare as a set and are never `MAJOR`; terminal candidates come from the capture's environment (R-VAL-2a, R-UI-10, R-API-6, 07 §2.5/§2.6, 11 §A1, new 06 open point 23) |
| Priors and deviation | Tolerance sources contradicted each other (UEX value, settings, conservative defaults, "never from constants") | One **effective tolerance**, defined in R-VAL-2: the UEX value only with verified units, otherwise a documented conservative default; user overrides can only tighten it; diagnostics shows the source (R-VAL-2, R-UI-10, A15, CLAUDE.md, 09, 02 stage 5, 05 F1) |
| Priors and deviation | The local prior was updated after any submission, including test submissions and rejected rows | Only production submissions, only the rows UEX accepted, only the report's environment; a withdrawal drops the local update (R-VAL-7, R-SUB-8) |
| UEX API facts | Requirements relied on unverified UEX facts not listed in 06: per-row report counting, `commodity.is_accepted`, `/commodities` `price_buy`/`price_sell` | The [3P] structure of `data_parameters` (`global.*`, one object per report type) and of `/commodities` is recorded as unverified; `commodity.is_accepted` means acceptance of commodity reports as a whole; rows are counted conservatively (R-SUB-2, R-SUB-5, R-CAP-3a, 05 F14, new 06 open points 22 and 23) |
| Evaluation corpus | Golden metrics depended on the live UEX cache: entries carried no frozen reference data, UEX IDs or capture times, so prior-dependent metrics could drift without a code change | Schema 2 adds expected UEX IDs, capture times and a `reference/` directory of recorded responses, replayed through adapter-uex's own mapping with a fixed eval clock; priors recorded after the capture are reported, not gated; `reference/` stays private until UEX allows publication (R-QA-3, R-QA-5, 07 §4, corpus/README.md, 02 §2 `tools/ocr-eval`, 06 open point 7) |
| Submission lifecycle | A job interrupted while sending (cancel, shutdown, crash) could be sent again, and automatic retries on 5xx or I/O errors could double-submit, because `java.net.http` cannot tell whether the body was sent | R-SUB-9 defines the only retryable set (connect-phase failures, 429, `requests_limit_reached`); every other failure is an unknown outcome; a `sending` marker is written before the request; cancel is refused while sending; at startup an interrupted send becomes an unknown outcome; after the retry limit the job is held for the user (R-SUB-9, R-SUB-2, R-SUB-11, 02 §6, 11 §A3, new 06 open point 25) |
| Submission lifecycle | `OutcomeUnknown → Draft` without a cooldown, plus "`duplicated_report` counts as cooldown", re-sent reports that UEX had already stored | Rows of an unknown-outcome job get a provisional cooldown; `duplicated_report` after an unknown outcome returns the new report with `PossiblyAlreadyReceived` and never re-sends it automatically; the user's "not received" keeps `OutcomeUnknown` and only then allows a duplicate; `OutcomeUnknown → PartiallyAccepted` added (R-SUB-4, R-SUB-9, 11 §A1/§A3, 02 §6, 06 open point 15) |
| Submission lifecycle | The cooldown key ignored the UEX user and test versus production mode | `CooldownKey(user, mode, terminal, commodity)` with `SubmissionMode {TEST, PRODUCTION}` in shared-kernel; the environment stays out of the key until A12 is verified, which blocks more, not less (R-SUB-4, 02 §3/§6, 11, 05 F19) |
| Submission lifecycle | The error classes did not cover `duplicated_report`, `user_not_found`, `missing_secret_key`, HTTP 401/403 and other known codes | One code-to-class table in 06; R-SUB-11 keeps only the class definitions and gains the class *cooldown*; 401/403 without a code counts as *account* until verified (R-SUB-11, 06 `data_submit`, 06 open point 6) |
| Submission lifecycle | Job phases had no Report states or events; cancel from `WaitingForCooldown`, "send the others now" and restart were undefined; R-VAL-6 at a send attempt had no target state; discarded reports could not be represented; withdrawal changed the Report before `data_remove` answered | A job-phase → Report-state → event table with the new events `SubmissionDeferred`, `SubmissionReturned`, `SubmissionCancelled`, `WithdrawalSucceeded` and `WithdrawalFailed`; "send the others now" splits the report into two released reports; a restart returns unfinished jobs to Draft; report-level `AgeConfirmation`; final state `Discarded(reason)`, reached only by the user; `withdraw()` only after `WithdrawalSucceeded` (02 §3/§6, 11 §A3, I4, I6, R-SUB-4, R-VAL-6, R-CAP-7) |
| Submission lifecycle | `PartiallyAccepted` assumed a positional `ids_reports` mapping that a shorter list cannot provide | Positional mapping only when the counts match; otherwise every row is unmapped, keeps its cooldown and is resolved via `data_info` or by the user; rows resolved as not accepted can be duplicated (R-SUB-9, 02 §6, new 06 open point 24) |
| Submission lifecycle | Release while offline or while UEX acceptance is closed was undefined | Release uses only local state (newest cached snapshot with its age, cached `game_versions`, last account check); closed acceptance is a warning at release and the job phase `held` at send time (I2, I6, R-API-7, R-UI-8, R-SUB-5) |
| Submission lifecycle | An older observation could reach UEX after a newer one of the same key and become the latest value | Such rows get `NewerObservationSent` and block or return the report until the user removes or confirms them; queued jobs of one key are sent oldest first; a [3P] indication of a `date_added` parameter is recorded (new R-VAL-8, I6, 06 open point 18) |
| Submission lifecycle | Manual reports had no `observedAt`; the rate budgets reset on restart; M1 shipped the lifecycle without a queue view; manual entry could neither add a commodity UEX does not list for the terminal nor show "screenshot required" in advance | `observedAt` of a manual report is the earlier of the Draft creation and its attachment and can only be moved earlier; the row budget is a sliding window over persisted attempt records; R-UI-1 is split (report and job states M1, capture states M3); "Add commodity" from the global vocabulary; the screenshot requirement is shown once known (R-VAL-6, 11 §A1, R-SUB-2, 02 §6, 04 M1/M3, R-UI-1, R-MAN-2, R-MAN-5, A11, 06 open point 10) |
| Capture time | A capture time taken from the file time or the clipboard let old copied or pasted screenshots pass the R-VAL-6 age gate and the R-CAP-3b version check; DST overlaps and gaps were undefined | `CaptureTime` with its source (file name, file time seen live by the watcher, clipboard event, paste time, user); all other cases get `CaptureTimeUncertain`, which is confirmed or moved earlier once per report and keeps stitching and grouping from deciding by time; DST rules; the cooldown end counts from local receipt time (R-CAP-5, R-VAL-6, R-OCR-11, R-OCR-13, new A19, 02 §3, 11 §A1) |
| Image intake | The first import or catch-up scan took over the whole screenshot backlog | Default cutoff for a new folder: the time of adding minus the R-VAL-6 hard limit; older files only by explicit choice; skipped files are not registered (R-CAP-1, R-CAP-1a, 02 §4a, CLAUDE.md) |
| Image intake | Bulk imports queued decoded images, and decoding had no bounded executor; clipboard monitoring had no interval, change detection or thread; Re-import contradicted "Capture identity = content hash"; 01 and 05 sent Wine prefixes to polling while 02 said inotify works there | The queue holds file references; decoding runs on a bounded intake executor and again through `CaptureImageLoader`; pasted images are first written as PNG; clipboard monitoring is opt-in and detects a change before reading (Linux: new A23); Re-import calls `rescan()` and is offered only for captures without an active report; polling only for FUSE and network mounts (R-CAP-1b, R-CAP-1d, R-CAP-2, 02 §4a/§7, CLAUDE.md, 05 F16) |
| Domain model | I6 was missing from the "I1–I5" references and the `release()` contract; "mandatory field" was undefined; user-entered and corrected values had no status under I2 and no origin, so a new capture could overwrite them | Invariant table rewritten once; mandatory fields defined (price, SCU and status of every sent row; `container_sizes` optional); `FieldOrigin {RECOGNIZED, USER}`; new invariant I7: new captures and merges never silently change a user value; every range in the plan now reads I1–I7 (11 §A1/§A3, 07 §2.6, R-OCR-11, CLAUDE.md, 02, ADR-002, 04 M0) |
| Domain model | The Report had no commands or report-level state for terminal, side, environment, game version, observation age and attachment preview; an ambiguous commodity could not be represented; an `is_missing` row could not be built; the gate returned only one reason | 02 §3 code block rewritten: `RowId`, `Field<CommodityId>`, `ObservedRow`/`MissingRow`, `Coverage`, `VersionAtCapture`, `ManualAttachment`, `ReportFinding`, `FixableReason` and key-change commands; `SubmissionGate.evaluate()` returns every reason from a closed `BlockReason` set; account standing enters as `ReleaseContext` (02 §3, 11 §A3/§A5, R-UI-2, R-VAL-5) |
| Domain model | `Field<T>` mixed recognition output with reporting concepts; the model could not drive the four confidence levels or "reference outdated", and the pseudo-class set was incomplete | Recognition outputs `FieldReading<T>`; reporting's `Field<T>` carries the origin, the `FieldAssessment` (with `ConfidenceLevel` and `referenceOutdated`) and the confirmation; the complete pseudo-class set is defined in 02 §7, with `:needs-confirmation` meaning "blocks release under I2" (02 §3/§7, ADR-002, 11 §A2) |
| Domain model | Stitching needed groups that grouping formed only from stitched scans; the Capture states contradicted "environment pending"; AI jobs had no lifecycle | `ReportGrouper` assigns scans to groups first, then `workflows` calls `StitchingService` on the stored scans (`ScanRepository`); Capture states `Imported`, `EnvironmentPending`, `Ready`, `Scanned`, `Failed` with the event `CaptureReady`; AI job states `Pending`, `Running`, `Done`, `Failed`, `Obsolete` and `Cancelled`, and a queue pause is not a failure (02 §2/§4/§4b, 11 §A2/§A3, R-VLM-3, R-VLM-10) |
| Domain model | The transactional outbox was unreachable from the context modules, and 11 §A6 described store-then-publish | Repository ports take the events together with the new state; adapter-storage writes both in one transaction; `EventOutbox` (workflows) only reads, marks delivered and notifies (11 §A6, 02 §2) |
| Module rules | `ImageRaster` sat in `recognition`, which `capture` cannot depend on; the working-copy store had no port; `adapter-files` was needed in M1 but planned for M3; ScopedValue bindings do not reach executor tasks without structured concurrency (preview); CLAUDE.md contradicted itself on exports; 09's port rule forbade upstream types; the ArchUnit list left file and socket I/O, process and native access open; ADR-002 named a nonexistent port | `ImageRaster`, `FolderLocation` and `LogContext` move to shared-kernel; `WorkingCopyStore` (workflows); first slice of `adapter-files` in M1; `app` wraps every executor and re-binds the `LogContext`, and only `app` creates executors; export and port rules aligned with the context map; ArchUnit list extended, including the no-argument clock calls; ADR-002 corrected (CLAUDE.md, 02 §2/§3, 09, 04 M1, ADR-002) |
| Settings and storage | Safety thresholds could be lowered without bounds; many settings had no owner; nothing serialised writes to the config file | Safety thresholds can only be made stricter (send and digit threshold, staleness limit, tolerances, `maxObservationAge`); calibrated values are not editable; invalid values are replaced by defaults and reported; settings catalogue with `GameSettingsStore` and `AppSettingsStore`; one `ConfigFile` writer (R-NF-5, 02 §2, 09, 04 M1) |
| Settings and storage | The storage layout was contradictory (%APPDATA% versus %LOCALAPPDATA%, one database or two, portable versus installed) | 02 §8 is the single source: config directory; one SQLite database in the data directory, never in a cache directory; logs in the state directory; only regenerable files in the cache; portable mode under `<launcher dir>/data/`; one instance per OS user across installed and portable (R-NF-2, R-NF-5, R-NF-10, 02 §8) |
| Settings and storage | The DB backup copied the raw file, and a restore could re-send jobs; "delete all local data" covered only part of the data; retention left out most report states and the processed-file register; data at rest had no permission rules | Backup through SQLite with `integrity_check`, config file included, `backup_meta` marker and "Restore backup"; R-NF-9 with full scope, a precondition and the history export offered first; retention per state, open reports never purged, import pauses on low disk space; Linux directories 0700 and a Linux-only 0600 fallback secret file (release-process.md, R-NF-11, R-NF-9, R-CAP-7, R-CAP-1d, R-NF-2, R-NF-4, 02 §8) |
| User interface | The rule against unsolicited dialogs was an aside in R-CAP-1 while several requirements "ask", "warn" or "offer" on background events; 02 §7 had no Session overview or global status area; no accessibility requirement existed; nothing limited animation next to the game | Background events never open dialogs, take focus or change the window state; they show as persistent state (`DialogPresenter` plus ArchUnit); app shell with a global status area and the Session overview; R-UI-1 lists every needs-action state; adopted WCAG 2.2 AA subset; no indeterminate progress or repeating animations (new R-UI-16 and R-UI-17; R-UI-1, R-UI-7, R-UI-8, R-NF-3, 02 §7, 09) |
| Recognition: numbers | Confusable variants for every 0/6/8/9 made almost every price `Ambiguous`; hole counting cannot separate 0, 6 and 9 | Variants only at positions below the per-digit threshold, topology vetoes applied first; topology returns the set of fitting digits and acts as witness, as veto or abstains (07 §2.5, R-VAL-2a, R-VAL-2b, 05 F2) |
| Recognition: terminal and side | A foreign location in YOUR INVENTORIES resolved confidently to the wrong terminal; Jaccard was the wrong metric; "assortment first" was circular; the side had no field, finding or second witness | Global commodity pass first; containment against the side-specific assortment for every candidate (`TerminalResolutionSettings`, `LocationAssortmentMismatch`); the assortment pass runs only after the terminal is resolved (`UnexpectedCommodity`); the side is a `Field<TradeSide>` with shop-panel text as witness (07 §2.1/§2.5, R-OCR-7, R-OCR-13, R-UI-2, R-UI-12, R-SUB-3, 05 F11/F12, new 06 open point 26) |
| Recognition: parsing and layout | VLM answers could not pass the OCR parser; the SCU regex failed on the plan's own example; the currency-glyph rule could strip a real leading 8 or 9; cards cut at the top edge passed; the section per card was undefined; WrongScreen and NotLocated had no criterion; the number parser sat in the wrong stage | Shared number parser with OCR-only unit and glyph handling; procedural SCU parser; glyph strip only with alignment evidence; a card is complete only with both borders inside the viewport; section headers read only in the shop panel, `SectionUnknown`, SELLABLE CARGO not sent until verified; decision rule for WrongScreen with a language hint; stage table with "4b Fusion" (07 §1/§2.1/§2.3/§2.4, R-OCR-11, R-OCR-17, R-VAL-5, new A21, 02 §4, 11 §A3) |
| Recognition: structure | Structural cues seen in the corpus were unused (contiguous cargo-size runs, `max_container_size`, A14 dimming, MAX INVENTORY) | Each one only raises `Inconsistent` (confirm), never repairs, and is active only once verified (new A22, 07 §1b/§2.4, 04 M2, 06 open point 26) |
| Recognition: terms | The upload screenshot was shop panel plus location field in 01 but only the shop crop in 02; "second reader" meant both the VLM and the status-bar classifier | The location field appears once above the stacked shop crops, taken from the working copies; "status-bar witness" versus "second reader" (VLM only), both in the glossary (R-SUB-7, 02 §6/§9, R-OCR-10, 07, 05 F10, 11 §A1) |
| Fusion | Fusion compared single values although the parsers yield candidate sets (OCR {96705, 6705} against VLM 6705); a VLM-only row turned partial cards into sendable values | Fusion over per-reader candidate sets: a VLM value equal to exactly one non-raw OCR candidate wins as repaired, with the VLM as witness; only cards that Layout marked complete can be filled, other VLM rows are discarded (07 §2.7, 02 §4b, R-VLM-5, R-VAL-2b, 04 M5) |
| Optional AI | Cloud models were detected only by ":cloud" and run through localhost; "localhost" was undefined, and S-26 forbade the default `http://localhost:11434`; "models only with a fixed SHA-256" conflicted with pulling by tag; Ollama cannot report the download size before a pull | Remote models detected by the `remote_host`/`remote_model` metadata, fail closed, checked before every job and on every response; literal-only loopback check with `NO_PROXY`; S-26 exception for plain HTTP to loopback and to confirmed hosts; evaluated-model list with digests, other models capped at confirm (`UnevaluatedModel`); approximate size shipped, live total from the pull stream (R-VLM-6, R-VLM-7, R-VLM-11, S-26, S-27, new A20, CLAUDE.md, 02 §4b/§8/§9, 04 M5) |
| Privacy | Balance redaction existed only in the upload composition, with no rule for locating the region and no fail-closed path; review, VLM and exports saw the unredacted panel | Geometric crop extent, applied before anything is persisted; every consumer gets only the redacted crops; unconfirmed geometry (manual crop, fallback corners, unvalidated game version) needs a full-size preview confirmation (R-SUB-7, R-CAP-7, 05 F30, 07 §2.1, CLAUDE.md, 04 M3) |
| Security | The secret key went to any configured host; untrusted input (image files, folder links, UEX and Ollama responses) had no rules; the diagnostics bundle for public issues removed only secrets and the user name; the single-instance mechanism was undefined | Fallback list empty until UEX confirms the mirror, a custom host only as an expert setting with https, no redirects; "Untrusted input" rules (size caps, no link following, crash-loop protection, bounded `Retry-After`); allowlist-based export with a preview; single instance via a lock plus a Unix-domain socket (R-API-3, R-API-5, R-CAP-1c, R-CAP-1d, R-NF-6, R-NF-10, 02 §4a/§6/§8/§9, 06 open point 4) |
| Supply chain | R-SEC-2 required PGP for every artifact while S-4 accepted checksums for unsigned plugins; CODEOWNERS and CI guards left out files that can switch off verification; detached configurations were neither locked nor verified; `setup-java` did not enforce signatures; the OCR dictionary was not pinned | SHA-256 for every artifact plus PGP where signed, bootstrapped in two runs with `help check`; armored keyring; `--dependency-verification strict` with a guard; extended CODEOWNERS and wrapper check; S-3 exception for google-java-format; `verify-signature: true`; all OCR assets committed with SHA-256 (R-SEC-2, R-SEC-7, S-3, S-4, S-5, S-10, S-11, S-22, CLAUDE.md, 04 M0) |
| CI | The OSV scan named the wrong action, needed permissions S-14 did not grant and ran a mutable image tag; OSV has no severity filter; the release job held write tokens while running the build; the live-API job exposed the key to the whole build; the attestation action differed between documents; "live tests only manually" contradicted the weekly job | OSV-Scanner CLI v2.6.0 with a committed SHA-256 and a checked-in gate script (CVSS ≥ 7.0 in runtime dependencies, fail closed, exceptions with expiry); read-only build matrix plus a separate publish job with the write tokens; live job with a prebuilt CLI and a dedicated test account; `actions/attest` v4.2.2 with a public-repository precondition (S-13, S-14, S-16, S-19, CLAUDE.md, 02 §10, 04 M1) |
| Packaging | "JavaFX from jmods" bypassed Maven Central and verification; ONNX Runtime lands in jlink's merged module; Gradle 9 archives drop execute bits; the WiX toolset was outside every measure | JavaFX natives copied from the verified Maven Central JARs, `javafx.cachedir` set by the bootstrap; explicit `mergedModuleName` named by `--enable-native-access`; packaged-app smoke test with `--illegal-native-access=deny`; `useFileSystemPermissions()` on the Linux tar task; WiX 3.14.1 from the runner image as a recorded trust anchor (02 §8/§10, 03, 04 M0/M1, S-11, S-24, S-30, release-process.md) |
| Releases | There was no path for security releases; PIT and OSV were listed inside `./gradlew check`; the Java 27 status of JaCoCo and Gradle was stale | "Security release" section (quarterly JDK updates, end of support of the JDK line, OSV findings against the release tag, `SECURITY.md` reports; a PATCH release with the full checklist); JDK pin file and scheduled check; checklist corrected; JaCoCo's Java 27 support marked experimental, Gradle 9.8.1 toolchain 27 verified (release-process.md, 03, S-11, S-18, S-19, 09 §12, 04 M0, CLAUDE.md) |
| Resources | Only peak heap was a metric; ONNX Runtime threading and off-heap memory were unspecified; OCR, decoding and hashing had no CPU priority; the targets had no reference machine and no pass/fail criterion before release 0.5 | R-NF-3 rewritten and raised to M (process memory with explicit `-Xmx` and `-XX:MaxDirectMemorySize`, CPU ceiling, workers below normal priority, UI rendering); ONNX Runtime with one intra-op thread per session, sequential, no spinning; reference machine named in M2, R-OCR-12 as p95 per resolution class; launcher limits in M3; pass/fail checks in M4 (R-NF-3, R-OCR-12, 07 §2.2, 02 §7, 04 M2/M3/M4) |
| Traceability | The traceability check could not pass for non-code and split M requirements, and R-UI-4 and R-VAL-2b were mapped to M1; smaller reference errors: 06 open point 17, missing F20/F7/F2 back-links, the roadmap missing from the CLAUDE.md reading list | List "Verified outside JUnit" and qualified tags for split requirements; coverage table corrected; references fixed; 04 added to the CLAUDE.md table with the rule to keep the coverage table current (04 "Requirement coverage", 11 §B4, CLAUDE.md, 05, 06) |

**Applied in part:** the per-row handling of `commodity.is_accepted` was not adopted, because the [3P] sources show a flag per report type, not per commodity; the side witness from the panel text already existed after section J; the I1–I6 ranges were already synced; for untrusted input, the archive (zip-slip) item, numeric-only `ids_reports` links and strict `global.ini` parsing were dropped (the app extracts no archives; the verifiers disputed the rest); the plan side of the game-detection finding is applied, and its claim about section I is corrected below.

**Rejected** (11; the current text already covered them, or the quote was stale): duplicating a submitted report (withdraw first, then duplicate, was already consistent); an AI re-read hitting a released report (R-VLM-4 already allows only Drafts); privacy rules for corpus intake (present in `corpus/README.md`); the S-8 remark on Dependabot (it refers to updating lockfile and verification metadata in one PR); the app token in storage and logs (R-NF-4, R-NF-6); the price-order witness for near-duplicate names (R-OCR-14 already requires an exact read); "orange frame lines" (the frame search is already theme-agnostic); two concurrent OCR jobs in a 512 MB heap (a provisional start value; the limits are set from measurement, R-NF-3); the game state before M5 (the rule against dialogs does not depend on it, and an undeterminable state counts as running); settings changes and existing Drafts (the gate uses the settings in force at release); no channel outside the app window in 1.0 (tray notifications are deliberately priority C).

**Corrections to earlier sections** (sections A–J stay as the historical log):

- §I "Submission lifecycle": the unknown-outcome fix was incomplete. An interrupted `sending` job, retries on 5xx or I/O errors and `OutcomeUnknown → Draft` without a cooldown could still double-submit; closed in this review (R-SUB-9, R-SUB-4, 11 §A3).
- §I "Data quality for UEX": R-VAL-6 did not stop old copied or pasted screenshots, because their capture time came from the file time or the paste; closed by `CaptureTime` with its source (R-CAP-5).
- §I, last paragraph: "Wine argv[0] in practice" and "EAC and `ProcessHandle` on Windows" were not listed in M0. Both are now verified with the game process monitor in M4 (A7; on Windows the image name comes from a Toolhelp snapshot, which needs no process handle). The Gradle 9.8.1 toolchain for Java 27 is verified (Gradle compatibility matrix), and the JaCoCo 0.8.15 changelog calls its Java 27 support experimental.
- §H H-18: the `*SettingsStore` ports existed, but many settings had no owner and the config file had no single writer; completed by the settings catalogue and `ConfigFile` (02 §2).
- §B B-14 and §J "Invariant count": superseded. The cooldown key is now (UEX user, submission mode, terminal, commodity) (R-SUB-4), and the invariants are I1–I7.

### Deferred

- **Licence and brand review** (runs in parallel; the CLAUDE.md licence rule is unchanged until then):
  - The licence rule by category: the stack already contains GPLv2 with Classpath Exception (bundled OpenJDK runtime, OpenJFX) and EUPL-1.1 (TestFX, test scope), which "Apache/MIT/BSD/EPL/LGPL fine" does not cover; mirror it in S-29 and 09 §9.
  - NOTICE, the project licence and the third-party licence texts in every distribution, assembled from the SBOM licence data, plus an About view; this extends the M1 packaging bullets, release step 7 and S-18/S-22 as rewritten here.
  - Brand, trademark and fan-content guardrails: a risk row for CIG/RSI and UEX names, marks and in-game imagery; naming criteria for the M0 decision (product name, package root, `UEXDR_` prefix, directory, installer and secret-store names); UEX consent for the name; a corpus rule for in-game imagery; a disclaimer requirement.
- **Design-system prompt rewrite** (`docs/prompts/design-system.md` is rewritten in parallel and was not changed here; notes were collected from 40 findings):
  - The spike downloads (`.sha1` check, optional PGP without a key check, no JDK source, JavaFX SDK) follow the new 10 §2.3 rule "Downloads outside the build" and use only verified Maven Central JARs.
  - The state inventory follows the current model: report states including `Discarded`, job phases without "failed", the Capture states, the AI job states, the `BlockReason` set and the `ReportFinding` list of 02 §3, the pseudo-class set of 02 §7, the watched-folder state enabled/disabled (R-CAP-1e). The prompt still says I1–I6.
  - R-UI-16 replaces the R-CAP-1 aside on dialogs, and R-UI-17 replaces the untraced accessibility criteria; automatic high contrast stays an owner decision.
- **Owner questions** (each with the default applied in the plan):
  1. PTU reports keep the `UnvalidatedGameVersion` cap until verified PTU captures exist, so every recognised field needs a confirmation or the per-session lift. Acceptable for PTU DataRunners?
  2. A major deviation against a prior older than 7 days always needs a confirmation; only minor deviations are visually suppressed. Acceptable?
  3. The commodity-wide average is a coarse deviation reference for commodities new at a terminal (beyond twice the tolerance: major deviation, confirmation). Keep it, or use the average only as a repair aid?
  4. May recorded UEX reference data be published in the public corpus? Until UEX clears it (06 open point 7), `reference/` stays private, so CI cannot gate prior-dependent metrics on public entries.
  5. Manual reports preselect the environment of the last manual report (shown prominently) instead of LIVE, and the running-channel cross-check (R-CAP-10) comes in M4. Agreed?
  6. "Send the others now" keeps the release of the rows still in cooldown; they are sent automatically when the cooldown ends, after the age check. Should they become a Draft that needs a new release instead?
  7. An older observation of a key that already has a newer sent one is blocked until the user removes or confirms the row. Should such rows be dropped by default with a notice instead? Once `date_added` is verified, should the client send `observedAt` in it?
  8. Files without a recognised file-name pattern (Import, drag & drop, file dialog) and pasted images need one capture-time confirmation per report. Acceptable, or should the file time in the configured screenshot folders count as certain?
  9. Should manual entry offer "mark as missing" (`is_missing`) on the user's statement alone, without scan coverage? Default: not offered.
  10. Side and environment are editable in a Draft (re-running resolution and validation and offering a merge). Or should they be display-only, so that a wrong value can only be fixed by discarding and re-importing?
  11. May an installed and a portable instance run at the same time for one OS user? Default: no, a per-user lock refuses the second instance.
  12. With fewer than 3 recognised commodities, a single location candidate is proposed at "confirm" (one key press). Should it be accepted automatically instead?
  13. A model not on the evaluated list stays usable, but its readings always need confirmation. Or should Automatic mode refuse such a model?
  14. A later capture (60 s or more later) that differs from a confident earlier reading is proposed and needs a confirmation (before, the later capture won automatically). Acceptable?
  15. A VLM reading from an evaluated model counts as an independent witness for an OCR repair. Or should only glyph topology be a witness?
  16. One recognised commodity missing from the terminal's UEX assortment prevents automatic terminal selection (0 misses allowed, configurable). Keep, given that the assortment can lag after patches?
  17. A commodity outside the terminal's UEX assortment for this side (`UnexpectedCommodity`) always needs a confirmation. Keep?
  18. Until the SELLABLE CARGO section is verified, its cards produce no report rows. Acceptable, or offer them after a confirmation?
  19. A non-loopback Ollama host over plain HTTP is allowed after a warning. Or accept only HTTPS?
  20. Can the project commit to a PATCH release within 7 days (start value) of every quarterly JDK security update, and to releasing the JDK 28 move before the April 2027 update? If not, that argues for JDK 25 LTS in the M0 decision.
  21. Build the MSI with the WiX Toolset 3.14.1 preinstalled on `windows-latest` (default), or with a pinned WiX ≥ 4 from NuGet (a documented S-1 exception that needs a licence check)? The support status of WiX 3 was not verified.
  22. Does the repository stay public? On Free/Pro/Team plans this is the precondition for artifact attestations (R-SEC-6).
  23. Can a dedicated UEX test account, separate from any maintainer's account, be obtained for the scheduled live-API job (asked in the M0 UEX clarification)?
  24. "Delete all local data" first offers the history export, because without `ids_reports` bad data can no longer be withdrawn. Or is the warning enough?
  25. The default cutoff for a new folder is the time of adding minus the R-VAL-6 hard limit. Prefer exactly the time of adding? On a first run, every capture older than the first `game_versions` fetch needs a version choice. Acceptable?
  26. After a patch, until the layout profile is validated for the new version, every OCR report needs the full-size preview confirmation before release. Acceptable, or only when the located geometry deviates from the profile?
  27. Accessibility: should the high-contrast theme switch on automatically with Windows high contrast (default: manual only)? Which minimum window size (e.g. 1280×720) is the reference for the 200 % text test? Is a documented non-conformance acceptable if 24 px targets conflict with the compact table density?
  28. R-NF-3 is raised from S to M, because R-OCR-17 relies on its memory budget and release 0.5 ships automatic OCR. Confirm?
  29. Which PC is the reference machine (CPU model, cores, RAM), and which frame-time tolerance (p95 delta of the game with the app importing against the app closed) is acceptable for 1.0?
  30. Re-import is offered only for captures that belong to no active report. Should a capture of a Draft also be re-readable in place, keeping confirmations like the AI re-read?
  31. Until the first successful `game_versions` fetch (e.g. an offline first start), new captures wait in `Imported` and are neither versioned nor scanned; that fetch moves them to `Ready` or `EnvironmentPending` (R-CAP-3b, R-UI-13, 02 §4a). Acceptable, or should OCR run earlier and the version be assigned later?
  32. The R-CAP-10 confirmation of a manual report is stored on the Report (`EnvironmentConfirmation`: observed running channel and time; revoked by an environment change). The release use case passes the running channel's environment from `game` in `ReleaseContext`, and the gate refuses with `EnvironmentUnconfirmed` while they differ and no confirmation for that channel exists (02 §3, 11 §A3 I2). Agreed?

  One further question from the review (should an outdated reference weaken only the display marking, so that a `MAJOR` deviation still needs a confirmation?) is answered by the stale-reference rule above and is therefore not listed.

**Still unverified** (marked in the documents): the new assumptions A17–A23 and 06 open points 21–26 (game-version lag and acceptance of older versions, `data_parameters` structure, prior and average semantics, partial acceptance, retry safety of `data_submit`, fields used by recognition checks), together with the earlier open points on per-row counting (16) and the observation time (18). All [3P] facts in 06 stay unverified until the M0 API spike.

## L. Governance and licence decisions (2026-10-08)

- The owner decided the open governance points; [ADR-0003](../adr/0003-licence-and-contributions.md) records them. Open points live only in [docs/adr/0000-open-points.md](../adr/0000-open-points.md).
- §D "licence" is answered: GPL-3.0-or-later (O-1, O-2). Porting from basetool-sc-extractor and basetool follows the porting rules in CLAUDE.md, and 03 §5 carries a status line.
- Contributions: a DCO 1.1 sign-off on every commit plus the CLA with a public roster; Conventional Commits; no squash merges (O-4, O-5, O-29).
- §K "Deferred", licence and brand review:
  - The licence rule by category is the CLAUDE.md "Stack rules" (S-29, S-34, 09 §9).
  - Notices, About view and source archives are R-UI-18, R-DOC-3 and R-DOC-4, S-34 and S-35.
  - The Star Citizen Fan Kit unit is adopted (R-UI-19, O-17). The corpus screenshots are marked `LicenseRef-Game-Screenshots` (O-18, O-28).
  - Naming criteria and UEX consent stay open (O-8).
  - ONNX Runtime telemetry is R-NF-12 (O-13).
- The other §D points are now O-8, O-9 and O-44 to O-46. The 32 owner questions of §K are O-47 to O-78.
- The sections above stay as written. This section supersedes their statements that the licence is undecided and that the CLAUDE.md licence rule is unchanged.

## M. Design-system prompt rewritten for Claude Design (2026-10-08)

- The prompt that §K "Deferred" left open is replaced. @greluc asked for a brief for Claude Design instead of a Claude Code prompt, written within Claude Design's limits. `docs/prompts/` now holds the brief (`design-system.md`), the stage messages, the owner checklist and the JavaFX handoff for Claude Code. The previous Claude Code prompt stays in the history (commit `1263435` (`07f7356` before the history was re-signed on 2026-10-09)).
- The §K prompt notes are applied:
  - The state inventory follows 02 §3, §4b and §7 and 11 §A3: report states with `Discarded`, job phases without "failed", the Capture and AI job states, the `BlockReason` set and the `ReportFinding` list.
  - The stale-reference rule of 07 §2.6 is applied: a major deviation is never lowered.
  - R-UI-16 and R-UI-17 are applied.
  - The download rule of 10 §2.3 is in the handoff.
- The governance decisions are applied:
  - The Fan Kit unit (R-UI-19) replaces the earlier disclaimer and the neutral trademark line. Both notices are verbatim; Claude Design draws the logo only as a placeholder.
  - The About dialog follows R-UI-18.
  - The design-system ADR takes the next free number (ADR-0004 as of 2026-10-08).
- An independent review compared the brief with the plan before the commit. Its corrections are in, among them:
  - Observation age unconfirmed vs. too old.
  - Set-aside files are not Captures.
  - The second Superseded variant.
  - The game version "to be checked" after a patch.
  - The settings that are not user-editable.
- One §K note was checked against 01 and corrected rather than applied as written: after `invalid_game_version` the report stays `Rejected` without "duplicate as new draft" (R-SUB-11); "discard" applies to Drafts only (11 §A3).
- The design decisions are open points O-83 (Claude Design checkpoints) and O-84 (D10 of the handoff).

## N. UEX API verification (2026-10-09)

The plan was written in an environment that could not reach UEX, so 06 rested on search snippets and third-party clients. On 2026-10-09 the plan was checked against the official documentation (all 90 endpoint pages), the UEX dev notes, the terms of use and read-only GET calls on both API hosts, without any user key or app token. No write call was made. Three independent passes compared every UEX statement in 01, 02, 04, 05, 07, 08, 09, 10, 11, the open-points register, the release process, the corpus and the design-system prompt with the verified facts; their figures were recomputed from the live data before anything was written down.

### Corrected or confirmed (applied)

- 06 is rewritten on the verified facts, with source tags, the full `data_submit` code list, the new `data_info`/`data_edit`/`data_remove` section, the reference-data fields as they are served, the terms, and the status of every open point (points 1–26 keep their numbers; 27–29 are new).
- The header is `secret-key`, not `secret_key` (A1, 10 S-26).
- The app token is required for `data_submit`, `data_edit`, `data_remove` and `data_info` (A2; 04 risk row). The 403 answers were checked on `data_info`.
- The sell-side number is sent as `scu_sell`; the matching prior is `scu_sell_stock`, because `scu_sell` in `commodities_prices` is a forecast demand (A10, 07 §1b). Comparing against `scu_sell` would have raised false major deviations.
- `price_variation` is a percentage and `ttl` is in days; `scu_variation` stays without a unit (A15).
- The `data_parameters` nesting is as the plan assumed, and `is_accepted` is per report type (06 point 22).
- The API sends numbers, not numeric strings (11 §A7).
- The global `is_buyable`/`is_sellable` flags contradict real terminal rows, and the MAX INVENTORY check has no data source; both are marked in 07 and wait for O-94.
- The status names differ per side, and the in-game texts match neither UEX name field (07 §2.2, A5).
- The design-system prompt's sample data named a terminal and three commodities that do not exist as written or are not sold at that terminal; they are replaced by real ones.
- Confirmed as planned: the 120/min and 500-row limits, the 1000-reports-per-30-min budget counted per row, the 5-minute duplicate block on terminal and commodity, one side per row, the `commodities_prices` limit of 10 terminal IDs, the per-side status levels and colours, `/game_versions` with `live` and `ptu`, the corpus terminal (Admin - Pyro Gateway (Stanton), ID 252) and every corpus commodity resolving to exactly one UEX ID.

### Findings that change the plan's design (owner decisions)

Each is an open point in the register; the plan's normative text is unchanged until @greluc decides.

- **Credentials:** the app token is a second user credential with its own error codes (O-85); the `X-Client-Version` lock (O-86).
- **Error classes:** about 40 documented codes are not classified by R-SUB-11; `requests_limit_reached` is not documented as HTTP 200 and no `Retry-After` header was seen (O-87).
- **One UEX report is one row** with its own ID and lifecycle (`pending` … `consolidated`, `declined`, `expired`). The history, R-SUB-9 "accepted" and the R-VAL-7 prior update do not model this (O-88). `ids_reports` is documented as a string, not an array, so the per-row mapping of R-SUB-9 waits for a write check (06 point 24). IDs repeat across UEX partitions.
- **Withdrawal** is one call per row, only before consolidation, without a test mode; I4 and the release smoke test assume otherwise (O-89). `data_edit` exists (O-90).
- **New report fields:** `date_added` (≤ 30 days old; O-91), `faction_affinity` (O-93); `container_sizes` is per report only (O-92).
- **Validation and recognition:** O-94 (contradicted checks), O-95 (staleness 7 days vs. UEX `ttl` 15 days), O-96 (`displayname` is not unique, refinery terminals, commodity names that are prefixes of others), O-99 (version strings are not semver).
- **Privacy:** `GET /user` returns e-mail and Discord name with the secret key (O-97).
- **Screenshot:** the 10 MB limit is probably on the base64 length (O-98).
- **Staff notification and `is_datacenter_enabled`** in `data_parameters` (O-100); the current commodity notice says to report the "Local Market" value, which bears on A21.

### Still open after this check

- Needs a write call with `is_production=0`, each approved by @greluc: the `ids_reports` format and partial acceptance (06 points 24), the duplicate-block scope (11), `date_added` semantics (18), acceptance of an older game version (21), `container_sizes` per row (9), `is_missing` fields (12), whether test submissions are stored and can be removed (29), whether UEX stores a reported sell SCU of 0 (8).
- Needs UEX: token distribution (O-85), primary host and writes on both hosts (4), `scu_variation` unit and `is_datacenter_enabled` (6), terms for recorded responses (7, O-50), HOTFIX/EPTU/TECH-PREVIEW (13), budget scope (16), server-error codes before or after processing (25), `faction_affinity` (27).

## O. Predecessor and third-party clients (2026-10-09)

@greluc asked what the predecessor and other clients can teach us. Three read-only passes:

- **The predecessor.** The installed copy of SC-Datarunner-UEX (v0.12.0) was examined only through what it writes and ships as plain data: its logs from one session on 2026-10-08, its data maps, the key list of its locale file and its working images. Its README, 19 release notes and 37 issues with comments were read as well. Nothing was decompiled; `CLAUDE.md` forbids it.
- **Screen-reading tools:** oovz/uex-datarunner (MIT), EtienneLamoureux/sc-trade-companion (GPL-3.0 text, no version statement), Zamotic/UexCorpDataRunnerClient (same), BlueMystical/Courrier-UEX (MIT), dolejska-daniel/starcitizen-kioskprobe (no licence), ArkanisOverlay (PolyForm-Noncommercial), and smaller ones.
- **API clients:** dolejska-daniel/uexcorp-openapi, Hybris95/UEX-Trader, ByteCollectiveIO/sc-nav, Garulf/uex-lib, the ShipBit Wingman AI skill, Sammmy1036/Quantum (MIT, the most complete open-source datarunner client), SC-Ravaxx/Bedrock (GPL-3.0).

No code was copied and none is ported. Every finding is recorded in our own words, with its source.

### Applied

- 06 has a new section "Observed by third parties" with the production traffic the API documentation does not show:
  - `ids_reports` is an array of numeric strings, one per row, and IDs interleave across parallel submissions.
  - `date_added` is accepted in Unix seconds and echoed as a string.
  - `duplicated_report` is HTTP 400 with an empty message.
  - A 15 s timeout did not stop processing.
  - `.space` is the proven write host.
  - Some strings are HTML-escaped.
  - Open points 4, 8, 9, 18, 24 and 25 carry the evidence.
- 05 has F31–F39 from the logs and the issue comments. The most serious is F31, a prior-driven repair that cut 6,868 down to 8 at 88–98 % confidence. Next are F32 (a commodity name that is a prefix of another resolving to the shorter one, 29 cards in one session) and F34 (timeouts shown as failures, whose resends were all duplicates). F30 is upgraded from unknown to very likely.
- The register gains O-101 to O-104, and O-85, O-87 and O-92 carry the new evidence.

### What the plan already does better

- No open-source tool has per-field confidence, an independent witness for repairs, a balance-free screenshot, or an unknown-outcome rule.
- Three tools upload the full screenshot including the balance.
- Two retry `data_submit` on any error.
- Several take silent defaults: side becomes sell, an undetected status becomes 1, a decimal point is inserted until a price fits.
- These confirm R-SUB-7, R-SUB-9 and "never guess silently"; they become negative test cases (O-104).

### New ideas for @greluc's decision

- **Game.log as a local, independent witness** for terminal, assortment, per-commodity container sizes and game version (O-101). This is the largest gap the survey found.
- Never touch files in watched folders (O-102).
- Robustness of submissions (O-103): longer timeout, fewer parallel sends, row-to-ID mapping verified through `data_info`.
- Parser edge cases (O-104): cSCU/µSCU, `¤` read as "1", status labels in game languages.
- **Mapping UEX IDs to the game's localisation keys** in `global.ini`, instead of matching names, as the predecessor does. It also yields the UI labels as anchors in every language. This fits R-L10N-2 and is worth a spike in M2.
- **Removing the known UI labels before name matching**; their OCR variants seen in the logs go into the corpus confusion list.
- **Onboarding:**
  - trim pasted credentials;
  - check the 40-character key shape as a hint;
  - validate the token and the key before saving them (Quantum);
  - link to the DataRunner sign-up (R-API-4).
- **A post-alignment similarity check** before the crop geometry counts as established (sc-trade-companion). This is a candidate gate for R-SUB-7, implemented on `ImageRaster` without OpenCV.

## P. Owner decisions on the UEX findings (2026-10-09)

@greluc decided O-85 to O-104 on 2026-10-09; the register ([docs/adr/0000-open-points.md](../adr/0000-open-points.md)) holds each outcome. The sections above stay as written; where they call these points open, this section supersedes them.

- **O-85** (own app token per user, validated before saving; already applied): consistency checked; A2, R-NF-4, R-API-4, 06 open point 2, 04 M0 and risk row, 05 F39 now say it is decided.
- **O-86** (`X-Client-Version` with the build version on every request): R-API-7; 06 "Basics", open point 20; 02 §6, §9 N1; release process "Versioning".
- **O-87** (every documented `data_submit` code classified; HTML 404 on a write = "API changed"; `GET /user` re-check before asking for a new key; "HTTP 200" of `requests_limit_reached` corrected): R-SUB-11, R-SUB-9, R-SUB-2, R-UI-1, R-MAN-1; 06 class table; 02 §3 `FixableReason`, §6, §7.
- **O-88** (UEX lifecycle status per row, polled from `data_info`; prior update on `ok`, reverted on `declined`/`expired`; `date_added` with every ID): R-SUB-4, R-VAL-7; 02 §6; 06 "Reading, correcting and withdrawing reports"; 11 §A1 ("UEX report ID", "UEX report status"), §A4, §A6 (`UexReportStatusChanged`).
- **O-89** (withdrawal per row, only while `is_editable` = 1, per-row outcome, cooldown lifted; smoke-test withdrawal removed): R-SUB-4, R-SUB-10; 02 §3 (`RowWithdrawal`, `applyWithdrawal`), §6; 11 §A3 I4 and states, §A6; release process step 3; `docs/prompts/design-system.md` history and withdrawal lines.
- **O-90** (no `data_edit`; withdraw plus duplicate): R-SUB-10; 11 I4; 06 open point 17.
- **O-91** (`observedAt` sent as `date_added`; hard limit ≤ 30 days; R-VAL-8 kept until our own write check): R-SUB-1, R-VAL-6, R-VAL-8; 02 §6, §9 N1; 06 "Submitting", open point 18; O-53 note.
- **O-92** (one report per distinct container-size set): R-OCR-9, R-SUB-1, R-SUB-7; 02 §3 "Container-size split" (`splitByContainerSizes`); 07 §2.4; 11 §A1 "Report", "Mandatory field", §A3; 05 F27; 06 open point 9.
- **O-93** (optional user-entered faction affinity): new **R-SUB-13**; R-SUB-1; 02 §3, §6, §9 N1; 11 §A1, §A4; 04 coverage (M1); 06 open point 27.
- **O-94** (side from `commodities_prices` instead of global flags; MAX INVENTORY check removed; "Out of Stock" ⇒ SCU 0 as a screen rule): R-VAL-3; 07 §1b, §2.5 item 4; 06 `/commodities` row.
- **O-95** (staleness default from `commodity.ttl`, fallback 15 days after the follow-up decision): R-UI-11; 02 §2, §5; 07 §2.6; 06 `data_parameters`; O-48 note; design-system brief.
- **O-96** (`displayname` in matching and search; repeats `Ambiguous`; refinery screens unsupported; prefix names): R-MAN-1, R-OCR-7, R-OCR-8; 02 §5; 07 §2.5 items 1–2; 06 `/terminals` row; 05 F32.
- **O-97** (`email`, `discord_username` dropped at the ACL): R-SUB-12, R-NF-6; 02 §9 N1; 06 `/user` row; 11 §A7.
- **O-98** (base64 length ≤ 10,000,000 bytes): R-SUB-7; 02 §6; 06 "Submitting", open point 10.
- **O-99** (version order only from observation; `/game_versions_all` only for A17): R-CAP-3b, R-OCR-19, A17; 06 `game_versions_all`.
- **O-100** (staff notification in the status area; `is_datacenter_enabled` = 0 is acceptance closed; A21 note): R-UI-8, R-SUB-5, R-SUB-11, A21; 02 §7; 06 `data_parameters`, open point 28.
- **O-101** (Game.log as an opt-in, read-only local witness): new **R-OCR-20** and **A24**; R-VAL-2b, R-NF-6; 02 §2 (`GameLogSource`, settings), §9 G1 and L3; 07 §2.5 item 6, §2.7; 11 §A1, §A7; 04 M2 spike and implementation (moved from M4 by a follow-up decision), coverage.
- **O-102** (no extra requirement; R-CAP-6's optional cleanup stays unchanged): register and 05 F36 only.
- **O-103** (strictly serial sending, 90 s send timeout, one shared `HttpClient`, row mapping verified through `data_info`): R-SUB-2, R-SUB-9; 02 §6; 11 §A3 `SubmissionJob`; 05 F21 (dated note), F34; 06 open point 24.
- **O-104** (parser edge cases, each verified on the corpus): R-OCR-5, R-OCR-7, R-L10N-2, R-API-5; 07 §2.4, §2.5 item 1; 11 §A7; 04 M2; 06 "Observed by third parties".
- **Follow-up decisions (same day):** the staleness fallback is 15 days, never laxer than the last verified UEX value (O-95); R-CAP-6's optional cleanup stays unchanged (O-102); "Add commodity" in R-MAN-2 no longer filters by the global `is_buyable`/`is_sellable` flags; R-OCR-20 is implemented in M2 instead of M4; R-SUB-13 stays in M1. `CLAUDE.md` follows O-103 (row-to-ID mapping verified through `data_info`).

## Q. Game.log spike (2026-10-09)

@greluc provided his own LIVE `Game.log` and its backups (45 builds, 4.4 to 4.10) for the M2 spike of R-OCR-20. The logs stay outside the repository; the results are in A24, R-OCR-20, 07 §2.5 item 6, 02 §9 G1 and 06 open point 9:

- The kiosk lines list only the buy side. The sell side is not in the log, so the witness covers the sell side's location only.
- Their container sizes equalled UEX's per-row sets.
- The line template did not change from 4.4 to 4.10.
- Shop names are templates shared by several stations, so the terminal follows from the location code.
- Location codes and internal commodity names need maintained mapping tables, because `global.ini` resolves only part of them.
- The build version (`4.10.193.11644`, also in `build_manifest.id`) gives only the major and minor version, not the UEX patch string.
- Every capture of one session was taken 7–170 s after the latest kiosk load at its location; the matching window starts at 5 minutes.
- Open: the two mapping tables and their per-version validation.

## Addendum 2026-10-12: language and toolkit decision superseded by ADR-0005

[ADR-0005](../adr/0005-rust-and-egui.md) replaced the implementation stack that this review and its sections A to Q assumed: Java 27 with JavaFX 27 and Gradle (ADR-0001, [03](03-language-decision.md)) became Rust with egui, with native code only by registered exception. The sections above are left as written; they are a historical record and name Java technology (JDK, JavaFX, jlink, jpackage, Gradle, Error Prone, ArchUnit, TestFX, `HttpClient`) where it was current on their dates.

Reading guide for the findings above:

- Findings about the domain, the UEX API, recognition, validation, the submission lifecycle and the corpus are unaffected.
- Findings about the Java toolchain, packaging and licence duties (JDK 27 versus 25, the jlink image and the ONNX Runtime merge, JavaFX natives, WiX through jpackage, OpenJDK and OpenJFX source archives, the Licensee gate, Gradle dependency verification) are overtaken. The matching open points are marked moot or amended in the [register](../adr/0000-open-points.md) (O-9, O-11, O-12, O-14 to O-16, O-25, O-35, O-66, O-67), and O-108 to O-124 record the new decisions.
- The accessibility question O-73 is narrowed: R-UI-17 no longer contains the screen-reader requirement, which became R-UI-17a (Could).
- Section K question 20 (JDK security-release rhythm) and question 21 (WiX) are moot; the Rust toolchain rhythm is O-119 and the installer decision is O-116.

## R. UEX answers and owner decisions (2026-10-26)

UEX answered the 22 questions of issue #19 by e-mail on 2026-10-25, and a follow-up the same day reported server fixes made because of them (06 tag [UEX]). The sections above stay as written; where they rest on the superseded facts, this section and the documents it names supersede them.

- **Facts that changed the plan:** `api.uexcorp.uk` is the primary host and `api.uexcorp.space` must not be a desktop client's fallback; the quota is daily per app token, with no per-minute limit, and runs out as HTTP 429 `exceeded_daily_quota`; `container_sizes` is read per row; a request stops at its first failing row and keeps the rows before it; `service_unavailable` and the image errors come before any row is stored; only the current game version is accepted (A18 refuted); the live price follows submission order, not `date_added`; dry runs are never stored; `duplicated_report` and `too_many_reports` count only the user's own reports since the follow-up; UEX resizes stored screenshots to fit 1920×1080.
- **O-85** reopened and decided: one Tallyline app token in the release artefacts, a user's own token only in a build without it or through an expert setting. A2, R-API-4, R-NF-4, 02 §2, §6, §9 N1 and L1, 05 F39, 06 "Basics", 10 S-14, `CLAUDE.md` "Working rules". How the token reaches the artefacts is O-139.
- **O-69** decided: no test account exists; the scheduled drift job is GET-only without credentials, and dry-run writes run locally with the owner's key and approval per call. 04 M0, M1; 10 S-14; `CLAUDE.md` "Tests".
- **O-136** (per-row `container_sizes`, the split of O-92 withdrawn): R-OCR-9, R-SUB-1; 02 §3, §6; 05 F27; 07 §2.4; 11 §A1, §A3.
- **O-137** (one submission per capture instead of a stitched screenshot): R-SUB-7, A11; 02 §2, §3 "Capture split" (`splitByCapture`), §6; 11 §A1, §A3; release process step 3.
- **O-138** (rows stored before an error, default applied) and the new assumption **A25**: R-SUB-9, R-SUB-11; 02 §6; 06 "Rows stored before an error".
- **O-140** (`scu_variation` is a percentage; the SCU tolerance keeps its settings default for now): R-VAL-2, A15.
- **Requirements that followed their own condition:** R-SUB-9 and R-SUB-11 move `service_unavailable`, `image_upload_error` and `image_storage_error` to the not-processed set, as R-SUB-11 had foreseen; R-API-3 keeps an empty fallback list; R-CAP-3b drops the "keep the capture-time version" choice once a newer version exists, as A18 had foreseen; R-SUB-4 and the cooldown key keep their shape (A12 verified); R-VAL-8 keeps its rule, now confirmed; R-SUB-2 loses the 120/min bucket.
- **Answered without plan change:** O-19, O-50 (new default), O-106 (UEX may be named, with an independence statement; README updated), O-53, O-55, O-89, O-98.
- **Asked again:** 06 open points 30 to 36.
- **Decided the same day after the options were laid out:** O-138 (D): a report-level error answer without a row takes effect only after one `data_info` check, so A25 is no longer relied on (R-SUB-9, 02 §6, 06). O-140 (D): the SCU tolerance is a percentage plus an absolute minimum, both to be exceeded, calibrated in M1; UEX's `scu_variation` never loosens it (R-VAL-2, A15, 02 §6).

## S. UEX answers to the follow-up questions (2026-10-31)

UEX answered the eight follow-up questions of 2026-10-26 (06 open points 30 to 36) by e-mail on 2026-10-31 (06 tag [UEX]). The sections above stay as written; where they call these points open, this section and the documents it names supersede them.

- **Facts:** rows are checked and stored one at a time, and UEX listed every code that can only come before the first row is stored, which verifies A25; `database_error` carries `data.row`, while `invalid_prices_array_format` and `reference_key_not_supplied` carry no `data`; a dry run can meet every error but `database_error`. An extractable Tallyline token is fine with UEX; a token is replaced only by re-creating the app, which stops it at once for every installed release; UEX can disable a misused token. The version lock takes one exact version. The reference endpoints may go without the token; `data_info` polls count on its quota, and 288 a day per active player are fine. A reported SCU of 0 equals no SCU; sell-side "OUT OF STOCK" is `status_sell` 1 with the SCU shown. `is_missing` goes as a bare row inside a normal request. `too_many_reports` is still 1000 in 30 minutes of the user's own reports, checked once per request. `date_added` tolerates no clock difference. One request per capture is welcome. `data_info` lists every datarunner's reports, with a `username` filter and an `is_owner` flag. Three undocumented codes were named: `missing_client_version`, `upgrade_required`, `invalid_screenshot`.
- **Requirements that followed their own condition:** R-VAL-5 offers `is_missing` for scanned reports, since 06 open point 12 is verified (02 §6); A25 is verified.
- **Confirmed without plan change:** O-85 (shared token, unlocked, reference requests without it), O-131 (sell-side "OUT OF STOCK" with stock), O-137 (one submission per capture), R-SUB-2 (row budget), R-SUB-4 (the `username` filter is enough).
- **Defaults applied:** the two per-row codes without a row index leave every row unmapped, like `database_error` without one (R-SUB-9, O-142).
- **For @greluc's decision:** O-139 gains the rotation question; new points O-141 (classes of the three new codes), O-142 (keep or drop the `data_info` check of O-138), O-143 (`data_info` poll backoff), O-144 (a `date_added` ahead of the server clock), O-145 (an SCU of 0 sent without `scu_*`). The release process gains the token-misuse step ("UEX API change", step 6).
- **Decided 2026-11-01 (@greluc), each as recommended:** O-141: `upgrade_required` pauses the whole queue with "update the app", `missing_client_version` is permanent, `invalid_screenshot` is report-fixable (R-SUB-11, 02 §6, 06). O-142: the `data_info` check is dropped for the codes UEX listed as before any row and kept for every other code without a row; the default for the two codes without a row index stands (R-SUB-9, A25, 02 §6, 06). O-143: `data_info` is polled every 5 minutes in the first hour, then every 30 minutes (R-SUB-4, 02 §6, 06). O-144: `date_added` is moved back to the server-time estimate from the latest `Date` header when `observedAt` is later, with `invalid_date` as the fallback (R-VAL-6, R-SUB-11, 02 §6, 06). O-145: a read SCU of 0 is sent without `scu_*` (R-SUB-1, 02 §6, 06). O-139 stays open.
## T. Release token delivery (2026-11-01)

@greluc decided O-139: secret-free build jobs produce binaries and separate protected packaging jobs add the token as a data file without running project build code. A PATCH release delivers a replacement; an invalid token pauses the queue, and the user-token override remains available. [ADR-0010](../adr/0010-release-token-delivery.md) records the decision and the M1 verification; app creation and protected-secret setup remain owner actions.
