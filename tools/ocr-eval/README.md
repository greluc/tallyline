# Corpus inspection and annotation

The `ocr-eval` workspace package inspects schema-version-two corpus entries, image availability, expected values, conditions, measured bands and frozen reference metadata. Run it with `cargo run -p ocr-eval --locked -- inspect`; use `--public <directory>` to inspect another public fixture directory. Private inputs come only from `TALLYLINE_CORPUS_DIR`.

`cargo run -p ocr-eval --locked -- annotate <expected.json> <anchors.json> <proposal.json>` writes a separate, exclusive-create proposal. See [the contributor capture guide](../../docs/user/corpus-capture.md) for anchor coordinates, privacy rules and report interpretation. Recognition, snapshot replay and accuracy measurements remain separate roadmap work.

## Explicit Game.log witness inspection

`cargo run -p ocr-eval --locked -- witness --channel <directory>` explicitly opts into a local read-only inspection of the selected channel's manifest and Game.log. It prints sanitised facts separately from corpus accuracy output; it is not recognition, a Reader or a submission path. The output uses closed reasons for unavailable inputs and a finite build/mapping allowlist. It contains no raw lines, source path, player fields, prices, quantities or status values. See [the validation record](../../docs/game-log-validation.md) and [the desktop guide](../../docs/user/desktop.md).

## Isolated Linux complementary OCR research

The developer-only `tesseract-research` binary and public safe Rust `isolation-probe` belong to this package. They add no application backend or new dependency. See [the controller record](../../docs/tesseract-linux-research-controller.md) for exact approval hashes, CLI sequence, root-only native execution gates, private evidence, licence/native boundaries and resource limitations. `cargo run -p ocr-eval --bin tesseract-research --locked -- contract deadline` runs a recorded own-process contract without a model. Specifying `--bin ocr-eval` continues to select the existing corpus CLI when using `cargo run`.

## Explicit SCU policy sensitivity

`cargo run -p ocr-eval --locked -- scu-policy --public corpus/public` reads only expected metadata and prints input hashes, complete whole-SCU quantity counts and finite synthetic digit-edit comparisons for 30 explicit percentage/absolute candidates. It uses no images, inference, HTTP or private corpus variable, never writes annotations and never reports recognition accuracy. It separately counts repeated rows and distinct scalar quantities. Limits are 64 expected files, 4 MiB per file and 65,536 cards; unsupported quantity types are refused visibly. See [the provisional calibration note](../../docs/scu-policy-calibration.md) for the 20 % / 16 SCU recommendation, its worked cases and empirical limits. Production fractional quantities are unaffected by this whole-quantity research mode.
