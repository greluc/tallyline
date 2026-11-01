# Corpus inspection and annotation

The `ocr-eval` workspace package inspects schema-version-two corpus entries, image availability, expected values, conditions, measured bands and frozen reference metadata. Run it with `cargo run -p ocr-eval --locked -- inspect`; use `--public <directory>` to inspect another public fixture directory. Private inputs come only from `TALLYLINE_CORPUS_DIR`.

`cargo run -p ocr-eval --locked -- annotate <expected.json> <anchors.json> <proposal.json>` writes a separate, exclusive-create proposal. See [the contributor capture guide](../../docs/user/corpus-capture.md) for anchor coordinates, privacy rules and report interpretation. Recognition, snapshot replay and accuracy measurements remain separate roadmap work.
