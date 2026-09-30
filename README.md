# stranger-strings

Rust implementation of Stranger Strings: extract candidate strings from binaries and score them using a Ghidra-compatible trigram model.

![Stranger Strings](./strangerstrings.png)

Stranger Strings helps analysts work through the noise produced by conventional `strings` extraction. It ranks candidate text using character patterns from a trained model, so common words and technical strings are more likely to survive than random printable bytes. The project began as a TypeScript implementation and is now written in Rust.

## What It Does

- Extracts strings from binaries (with offset tracking)
- Scores strings with trigram probabilities (`.sng` model format)
- Detects Base64 blocks and scores strings extracted from their decoded bodies
- Supports multiple extraction encodings: `ascii`, `utf8`, `utf16le`, `utf16be`, `latin1`, `latin9`
- Can use script-aware scoring for `chinese`, `arabic`, and `cyrillic`
- Outputs in `text`, `json`, or `csv`

## Background and Effectiveness Testing

The aim is to reduce manual review, not guarantee that every useful string is retained. Unusual identifiers and text poorly represented by the Ghidra trigram model can score badly. Use `-v` to inspect rejected candidates alongside their scores and thresholds; normal CLI output includes only accepted strings.

This is a quick sniff test based on a firmware file from my Downloads directory (via https://ssz.fr/brdl/A9-wifi/rtthread.bin).


Ghidra does a good job, but it misses domain specific terms from firmware like:   
```
     0009d3a5 43 43 4d     ds       "CCMP+TKIP"
              50 2b 54 
              4b 49 50
```
and
``` 
     0009d3bc 57 50 41     ds       "WPA2+WPA/IEEE 802.1X/EAP"
              32 2b 57 
              50 41 2f
```
More egregious was Ghidra's trigram model missing "HALT", "https://" and "http://". Stranger Strings misses them too so it is likely to do with the NSA's corpus rather than some specific exclusion.

### Stranger Strings
1281 unique strings identified.

The scoring cut-off is a little too aggressive on this binary, I need to add a more relaxed threshold option.
```
❯ ../target/release/stranger-strings -v ./rtthread.bin|grep -v ✓ |head -n 20 
Loading model: embedded StringModel.sng
Model type: lowercase, Lowercase: true
Analyzing file: ./rtthread.bin
Extracted 9231 candidate strings (min length: 4)
String               Adjusted Score Score        Threshold    Offset     Valid
-------------------------------------------------------------------------------------
"cuer"               -0.000         -2.710       -2.710       0x403B4    ✗
" %d 0x%x"           -0.000         -3.841       -3.840       0x95A3F    ✗
"http://183.193.243.90:9151/mp3/209713.mp3" -0.004         -6.026       -6.000       0xA085F    ✗
"%02d:%02d:%02d.%03d " -0.005         -5.445       -5.420       0xB0C23    ✗
"txu_cntrl_cfm"      -0.005         -5.054       -5.030       0xAC042    ✗
" LINK_UP"           -0.006         -3.865       -3.840       0xA805F    ✗
"[EVM]tx_exit"       -0.007         -4.912       -4.880       0xAB52F    ✗
"bick"               -0.007         -2.730       -2.710       0x4FF26    ✗
"stop"               -0.008         -2.731       -2.710       0x97EB6    ✗
"reg_isr_o1"         -0.008         -4.587       -4.550       0x947E2    ✗
"######input:%s"     -0.008         -5.101       -5.060       0x976F5    ✗
"--help"             -0.010         -3.555       -3.520       0xA5BA1    ✗
"SPK OFF!!"          -0.011         -4.541       -4.490       0x9FE5F    ✗
"http://183.193.243.90:9151/mp3/73865964.mp3" -0.012         -6.073       -6.000       0xA03BC    ✗
"=====TF OK!!===="   -0.014         -5.313       -5.240       0x977E9    ✗
"NO-EAP"             -0.014         -3.569       -3.520       0x9EF79    ✗
" ps :%d s"          -0.014         -4.291       -4.230       0x95A96    ✗
"rt_mq_recv"         -0.015         -4.617       -4.550       0xA39A9    ✗
```

### cctools-1030.6.3 strings (macos)
```
/Library/Developer/CommandLineTools/usr/bin/strings ./rtthread.bin| sort -u |wc -l
    8367
```

### GNU strings (GNU Binutils for Ubuntu) 2.46
```
/usr/bin/strings /tmp/rtthread.bin|sort -u |wc -l 
    7619
```
### Sysinternals Strings v2.54
```
c:\\sysinternals\\strings.exe -nobanner -n 4 rtthread.bin | sort -u | wc -l
    8172
```
## ELF test case (ubuntu coreutils ls)

-rwxr-xr-x 115 root root 11352352 Apr 16 22:41 /lib/cargo/bin/coreutils/ls
2d313ecc9fc058f6e0758abe00c0f6e8

String extractor | Unique strings
--- | ---
MacOS strings | 23026
Ubuntu strings | 22957
FLOSS | did not complete within 20 minutes 🤷‍♂️
Ghidra 12.04 (after a few rounds of auto-analysis) | 5030
stranger-strings | 7505

Stranger Strings is naive about the string extraction so Rust binaries are always going to be a bit less precise than a semantically aware parser (like Ghidra, or FLOSS... if it could finish).

My spot checks with very rudimentary MSA and Russian indicate those language models work better than I expected on the ubuntu ls binary. I used /usr/share/locale/zh_CN/LC_MESSAGES/apt.mo (the Chinese translation file for Ubuntu apt) to spot check that model and it seems sensible:

```
String               Adjusted Score Score        Threshold    Offset     Valid
-------------------------------------------------------------------------------------
"或者只能在其他发布源中找到"      3.141          6.423        -3.000       0x8607     ✓
"列出所有手动安装的软件包"       3.111          6.333        -3.000       0x897E     ✓
"列出所有手动安装的软件包"       3.111          6.333        -3.000       0x8980     ✓
"不是一个实包(虚包)"         3.083          6.250        -3.000       0xACBD     ✓
"不是一个实包(虚包)"         3.083          6.250        -3.000       0xACBF     ✓
"有些软件包不能通过验证"        3.076          6.227        -3.000       0x90A8     ✓
"有些软件包不能通过验证"        3.076          6.227        -3.000       0x90AC     ✓
"自动卸载所有不再使用的软件包"     3.071          6.214        -3.000       0xAA63     ✓
"参数不成对"              3.067          6.200        -3.000       0x6E56     ✓
"参数不成对"              3.067          6.200        -3.000       0x6E58     ✓
"列出所有自动安装的软件包"       3.056          6.167        -3.000       0x8959     ✓
...
```
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

Scored output includes the adjusted score (`(threshold - score) / threshold`), shown immediately after the string in text and CSV output and named `adjusted_score` in JSON/CSV. For negative thresholds, positive adjusted scores pass the threshold and higher values indicate stronger results. The default `--sort score` orders by adjusted score, largest first. Validity still uses `score > threshold`. Trigram candidates shorter than four normalised characters remain invalid and have no adjusted score: text shows `N/A`, JSON uses `null`, and CSV leaves the cell empty. These candidates sort after all scored results.

### Model path behaviour

If `--model` is omitted, the CLI uses `StringModel.sng` embedded at compile time. No external model file is needed at runtime. Use `--model PATH` to load a custom model instead.

### Encodings

```bash
# Multiple encodings
stranger-strings -e utf8,utf16le,latin1 ./sample.bin

# All supported encodings
stranger-strings -e all ./sample.bin
```

### Base64 blocks

Binary-file analysis also scans for standard and URL-safe Base64, with or without padding, including common LF/CRLF line wrapping. Decoded bodies use the selected encodings and minimum string length, and their strings pass through the same scoring and validity filters as ordinary strings. Decoding is limited to one layer.

JSON and CSV results include `base64_decoded_offset` for decoded strings. Their `offset` points to the encoded block in the original file; `base64_decoded_offset` points within the decoded body. Verbose text displays both offsets. Plain text includes accepted decoded strings alongside ordinary strings.

Candidates must contain at least six Base64 characters and decode successfully. Wrapped lines are joined when each preceding line has at least 16 characters, a length divisible by four, and no padding. Arbitrary space-separated Base64 and UTF-16-encoded Base64 containers are not detected. As with ordinary extraction, coincidental matches in binary data can occur; scores help filter them.

### Language-aware scoring

`-L arabic` explicitly selects the Arabic scorer (likewise `chinese` and `russian`). A comma-separated list detects the script and scores candidates whose detected script is in that list. `-L auto`, `-L all`, and `--auto-detect` enable detection across supported scripts.

Language-aware mode defaults to UTF-8 extraction and preserves printable Unicode candidates for scoring. An explicit `-e` overrides that default; use `-e utf16le` or `-e all` for other encodings. Language selection applies to file contents, decoded Base64 bodies, and stdin candidates. Without language-aware mode, extraction still defaults to ASCII.

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

# Sort: score (adjusted score) | alpha | offset
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
analyzer.load_model(&AnalysisOptions::default())?;

let result = analyzer.analyze_string("hello world")?;
println!("valid={} score={:.3}", result.is_valid, result.score);
```

### Batch analysis and custom models

`analyze_strings(&[String])` returns a result for every candidate; `extract_valid_strings(&[String])` returns only candidates that pass their thresholds. Results retain the original text alongside `normalized_string`, `score`, `threshold`, and `is_valid`.

To override the embedded model, pass `AnalysisOptions` with either `model_path: Some(path)` or `model_content: Some(content)` to `load_model`. If both are set, the file path takes precedence.

`extract_strings_from_binary(&bytes, min_length)` extracts raw ASCII strings with offsets without loading a model or scoring. The `analyze_binary_file` methods also extract from Base64 bodies and return both accepted and rejected results; filter on `is_valid` when using the library if you want only accepted strings.

### Binary analysis with multiple encodings

```rust
use stranger_strings::{BinaryAnalysisOptions, StrangerStrings, SupportedEncoding};

let mut analyzer = StrangerStrings::new();
analyzer.load_model(&stranger_strings::AnalysisOptions::default())?;

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

## How Trigram Scoring Works

The Latin-text scorer uses the `.sng` model's character-frequency data:

1. Lowercase text when the model specifies `lowercase`, replace non-ASCII characters with spaces, trim surrounding whitespace, and collapse repeated spaces and repeated tabs.
2. Look up character trigram probabilities, including beginning and end boundary terms. At model loading time, zero-count entries receive a count of one to avoid zero probabilities.
3. Sum the base-10 log probabilities used by the scorer and divide by the normalized string length.
4. Accept the string when its score is strictly greater than the threshold for that length. Higher (less negative) scores are better.

Selected thresholds from `src/constants.rs`:

| Normalized length | Threshold |
| --- | --- |
| 4 | -2.71 |
| 5 | -3.26 |
| 10 | -4.55 |
| 50 | -6.08 |
| 100 and above | -6.30 |

Normalized strings shorter than four characters cannot pass the threshold of `10.0`; strings shorter than three receive the default score of `-20.0`. The extraction minimum (`-l`, default 4) is separate from this scoring rule. Script-specific scorers use their own logic when language-aware scoring is enabled.

## Model Files

The embedded `StringModel.sng` is a lowercase model. Its header records training sources including word lists, proper names, contractions, and extracted strings. Those training choices affect which text scores well; changing the model changes the probabilities used for scoring.

Custom `.sng` files are tab-delimited text with a model-type comment and four fields per data row: three character tokens followed by a count. For example (the field separators below are literal tabs):

```text
# Model Type: lowercase
# Example counts for hello
[^]	h	e	1234
h	e	l	5678
e	l	l	4321
l	l	o	9012
l	o	[$]	3456
```

`[^]` marks the beginning of a string, `[$]` marks its end, `[SP]` represents a space, and `[HT]` a horizontal tab. A beginning row contains `[^]` followed by two characters; an ending row contains two characters followed by `[$]`. Do not add an uncommented column-header row.

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

Please don't send AI authored PRs unless you've read and understand what it did.

## License

Apache-2.0
