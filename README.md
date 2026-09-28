# stranger-strings

Rust implementation of Stranger Strings: extract candidate strings from binaries and score them using a Ghidra-compatible trigram model.

![Stranger Strings](./strangerstrings.png)

## What It Does

- Extracts strings from binaries (with offset tracking)
- Scores strings with trigram probabilities (`.sng` model format)
- Supports multiple extraction encodings: `ascii`, `utf8`, `utf16le`, `utf16be`, `latin1`, `latin9`
- Can use script-aware scoring for `han`, `arabic`, and `cyrillic`
- Outputs in `text`, `json`, or `csv`

## Install

### Prebuilt binaries

Builds for Linux, macOS (x86_64 + aarch64), and Windows are published from Git tags in GitHub Actions.

### From source

```bash
git clone https://github.com/closed-systems/stranger-strings
cd stranger-strings
cargo build --release
```

Binary path:

```bash
target/release/stranger-strings
```

## CLI

```bash
stranger-strings [OPTIONS] <input>
```

`input` can be a file path or `-` for stdin.

### Quick start

```bash
# Analyze a binary with default settings (ASCII extraction)
stranger-strings ./sample.bin

# Verbose output
stranger-strings -v ./sample.bin

# JSON output
stranger-strings -f json -o result.json ./sample.bin

# Use explicit model path
stranger-strings -m ./StringModel.sng ./sample.bin
```

### Model path behavior

If `--model` is omitted, the CLI looks for `StringModel.sng` **next to the executable** (not in the current working directory).

### Encodings

```bash
# Multiple encodings
stranger-strings -e utf8,utf16le,latin1 ./sample.bin

# All supported encodings
stranger-strings -e all ./sample.bin
```

### Language-aware scoring

```bash
# Auto-detect script and score with script-specific scorer
stranger-strings --auto-detect -e utf8 ./sample.bin

# Restrict to specific scripts
stranger-strings -L chinese,russian,arabic -e utf8 ./sample.bin
```

Important note:
- Latin and unknown-script scoring use the trigram scorer.
- If no trigram model is loaded and Latin/unknown text is scored, analysis will fail with `ModelNotLoaded`.

### Other useful flags

```bash
# Minimum extracted string length
stranger-strings -l 6 ./sample.bin

# Keep only unique strings
stranger-strings -u ./sample.bin

# Sort: score | alpha | offset
stranger-strings -s offset ./sample.bin

# Show model metadata and exit
stranger-strings --info

# Run built-in test strings
stranger-strings --test
```

## Library Usage

### Basic trigram scoring

```rust
use stranger_strings::{AnalysisOptions, StrangerStrings};

let mut analyzer = StrangerStrings::new();
analyzer.load_model(&AnalysisOptions {
    model_path: Some("./StringModel.sng".to_string()),
    ..Default::default()
})?;

let result = analyzer.analyze_string("hello world")?;
println!("valid={} score={:.3}", result.is_valid, result.score);
```

### Binary analysis with multiple encodings

```rust
use stranger_strings::{BinaryAnalysisOptions, StrangerStrings, SupportedEncoding};

let mut analyzer = StrangerStrings::new();
analyzer.load_model(&stranger_strings::AnalysisOptions {
    model_path: Some("./StringModel.sng".to_string()),
    ..Default::default()
})?;

let bytes = std::fs::read("./sample.bin")?;
let results = analyzer.analyze_binary_file(
    &bytes,
    &BinaryAnalysisOptions {
        min_length: Some(4),
        encodings: Some(vec![SupportedEncoding::Ascii, SupportedEncoding::Utf16le]),
        use_language_scoring: false,
        ..Default::default()
    },
)?;

println!("{} strings analyzed", results.len());
```

### Script detection only

```rust
use stranger_strings::StrangerStrings;

let mut analyzer = StrangerStrings::new();
analyzer.enable_language_detection()?;

let detection = analyzer.detect_language("Привет мир")?;
println!("script={:?} confidence={:.2}", detection.primary_script, detection.confidence);
```

## Compatibility

For Latin text with a loaded model, scoring is intended to match the original TypeScript implementation and `.sng` model behavior.

Current tests include compatibility checks and language-scoring checks:

```bash
cargo test
```

## Release Process

A release workflow is included at:

- `.github/workflows/release.yml`

On tag push (for example `v0.1.0`), it builds and publishes artifacts for:

- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

## Development

```bash
# Format + lint (if installed)
cargo fmt
cargo clippy --all-targets --all-features

# Test
cargo test

# Run CLI locally
cargo run -- --help
```

## Contributing

PRs are welcome. Keep changes focused, add/adjust tests with behavior changes, and include CLI/library docs updates when flags or API behavior change.

## License

Apache-2.0
