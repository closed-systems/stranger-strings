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
- Outputs in `text`, `json`, `jsonl`, or `csv`

## Install

### Prebuilt binaries

Builds for Linux, macOS (x86_64 + aarch64), and Windows are published from Git tags in GitHub Actions.

### Using Cargo

Requires Rust 1.90 or newer.

Once published to crates.io, install with:

```bash
cargo install stranger-strings --locked
```

Until then, or to install the latest repository version:

```bash
cargo install --git https://github.com/closed-systems/stranger-strings --locked
```

Or install from a local checkout:

```bash
cargo install --path . --locked
```

Cargo installs `stranger-strings` into `$CARGO_HOME/bin` (by default `~/.cargo/bin`). Ensure that directory is on your `PATH`, then run:

```bash
stranger-strings --help
```

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
$ stranger-strings ./sample.bin

# Verbose output 
$ stranger-strings -v ./sample.bin

# to see the borderline matches use grep, or pipe to less and use / to search for 'N$' to go to the first rejected strings to see what just passes and fails.
$ stranger-strings -v ./sample.bin | grep N\$

Loading model: embedded StringModel.sng
Model type: lowercase, Lowercase: true
Analyzing file: ./rtthread.bin
Extracted 9231 candidate strings (min length: 4)
"cuer"               -0.000         -2.710       -2.710       0x403B4    N
" %d 0x%x"           -0.001         -3.841       -3.840       0x95A3F    N
"bick"               -0.020         -2.730       -2.710       0x4FF26    N
"stop"               -0.021         -2.731       -2.710       0x97EB6    N
"txu_cntrl_cfm"      -0.024         -5.054       -5.030       0xAC042    N
" LINK_UP"           -0.025         -3.865       -3.840       0xA805F    N
"%02d:%02d:%02d.%03d " -0.025         -5.445       -5.420       0xB0C23    N
"http://183.193.243.90:9151/mp3/209713.mp3" -0.026         -6.026       -6.000       0xA085F    N
"[EVM]tx_exit"       -0.032         -4.912       -4.880       0xAB52F    N
"--help"             -0.035         -3.555       -3.520       0xA5BA1    N

Summary:
  Accepted: 3814 strings
  Rejected: 5442 strings
  Total: 9256 strings
  Acceptance rate: 41.2%

# JSON Lines output (one JSON object per line)
$ stranger-strings -f jsonl -o result.jsonl ./sample.bin

# JSON output
$ stranger-strings -f json -o result.json ./sample.bin

# Use explicit model path
$ stranger-strings -m ./StringModel.sng ./sample.bin

# Minimum extracted string length
$ stranger-strings -l 6 ./sample.bin

# Keep only unique strings
$ stranger-strings -u ./sample.bin

# Sort: score (adjusted score) | alpha | offset
$ stranger-strings -s offset ./sample.bin

# Do clever filtering with jq
$ stranger-strings ./rtthread.bin -f json |jq '.[] | select(.offset < 100000)'

{
  "original_string": "tItH",
  "score": -2.6803513051595993,
  "threshold": -2.71,
  "is_valid": true,
  "normalized_string": "tith",
  "offset": 54418,
  "detected_script": null,
  "scorer_name": null,
  "adjusted_score": 0.010940477800885856
}
```

Use `--relaxed` to retain more borderline candidates, for example `stranger-strings binary.exe --relaxed`. This lowers all eligible scoring thresholds by 1.0 (length 4: -2.71 to -3.71; length 100+: -6.30 to -7.30), including language-specific thresholds.
This accepts more noise as well as potentially useful strings. Verbose, JSON, JSONL, CSV, and `--info` output report the active thresholds.
Use `--threshold <number>` for a custom adjustment: `--threshold 1` is equivalent to `--relaxed`, `--threshold 2` lowers thresholds by 2.0, and negative values make scoring stricter. `--threshold` and `--relaxed` cannot be combined. Library callers can use `analyzer.set_threshold_adjustment(2.0)?`.

Scored output includes the adjusted score (`score - threshold`), shown immediately after the string in text and CSV output and named `adjusted_score` in JSON/JSONL/CSV. Positive adjusted scores pass the threshold and higher values indicate stronger results. The default `--sort score` orders by adjusted score, largest first. Trigram candidates shorter than four normalised characters are invalid and have no adjusted score: verbose text shows `N/A`, JSON and JSONL use `null`, and CSV leaves the cell empty. These candidates sort after all scored results.

JSONL (`--format jsonl`) emits one compact JSON object per line with the same fields as JSON output, including `adjusted_score`. Each record ends with a newline; no results produce empty output. Filtering, sorting, and `--unique` work the same as for other formats.



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

``` bash
$ stranger-strings rtthread.bin  -v |head        
Loading model: embedded StringModel.sng
Model type: lowercase, Lowercase: true
Analyzing file: rtthread.bin
Extracted 9231 candidate strings (min length: 4)
String               Adjusted Score Score        Threshold    Offset     Valid
-------------------------------------------------------------------------------------
"The sector size of device is greater than the sector size of FAT." 2.782          -3.378       -6.160       0xA4630    Y
"Print the name of the current working directory." 2.777          -3.283       -6.060       0xA5905    Y
"acceptmbox must be deallocated before calling this function" 2.725          -3.405       -6.130       0xA64F5    Y
"Association request to the driver failed" 2.707          -3.283       -5.990       0x9EF98    Y
"Initialize failed! Don't found the partition table." 2.696          -3.404       -6.100       0xA2ED9    Y
"Mathematics argument out of domain of function" 2.695          -3.325       -6.020       0xB17CC    Y
"list the information of network interfaces" 2.694          -3.306       -6.000       0xA584F    Y
"Save failed. The partition %s was not found." 2.691          -3.329       -6.020       0xAED52    Y

Summary:
  Accepted: 3814 strings
  Rejected: 5442 strings
  Total: 9256 strings
  Acceptance rate: 41.2%
```


```
❯ ../target/release/stranger-strings -v ./rtthread.bin|grep -v ' Y$' |head -n 20
Loading model: embedded StringModel.sng
Model type: lowercase, Lowercase: true
Analyzing file: ./rtthread.bin
Extracted 9231 candidate strings (min length: 4)
String               Adjusted Score Score        Threshold    Offset     Valid
-------------------------------------------------------------------------------------
"cuer"               -0.000         -2.710       -2.710       0x403B4    N
" %d 0x%x"           -0.000         -3.841       -3.840       0x95A3F    N
"http://183.193.243.90:9151/mp3/209713.mp3" -0.004         -6.026       -6.000       0xA085F    N
"%02d:%02d:%02d.%03d " -0.005         -5.445       -5.420       0xB0C23    N
"txu_cntrl_cfm"      -0.005         -5.054       -5.030       0xAC042    N
" LINK_UP"           -0.006         -3.865       -3.840       0xA805F    N
"[EVM]tx_exit"       -0.007         -4.912       -4.880       0xAB52F    N
"bick"               -0.007         -2.730       -2.710       0x4FF26    N
"stop"               -0.008         -2.731       -2.710       0x97EB6    N
"reg_isr_o1"         -0.008         -4.587       -4.550       0x947E2    N
"######input:%s"     -0.008         -5.101       -5.060       0x976F5    N
"--help"             -0.010         -3.555       -3.520       0xA5BA1    N
"SPK OFF!!"          -0.011         -4.541       -4.490       0x9FE5F    N
"http://183.193.243.90:9151/mp3/73865964.mp3" -0.012         -6.073       -6.000       0xA03BC    N
"=====TF OK!!===="   -0.014         -5.313       -5.240       0x977E9    N
"NO-EAP"             -0.014         -3.569       -3.520       0x9EF79    N
" ps :%d s"          -0.014         -4.291       -4.230       0x95A96    N
"rt_mq_recv"         -0.015         -4.617       -4.550       0xA39A9    N
```
The scoring cut-off is a little too aggressive on this binary, I need to add a more relaxed threshold option:
```
../target/release/stranger-strings -v ./rtthread.bin --relaxed |grep -v ' Y$' |head -n 30
Loading model: embedded StringModel.sng
Model type: lowercase, Lowercase: true
Analyzing file: ./rtthread.bin
Extracted 9231 candidate strings (min length: 4)
String               Adjusted Score Score        Threshold    Offset     Valid
-------------------------------------------------------------------------------------
"ssl://"             -0.000         -4.520       -4.520       0xA1E01    N
"MAC: "              -0.000         -3.711       -3.710       0xA8047    N
"& z0@ABAA"          -0.001         -5.493       -5.490       0x5950D    N
"'hyl"               -0.001         -3.714       -3.710       0x4772C    N
"%hu%n:%hu%n:%hu%n"  -0.001         -6.297       -6.290       0xB1D7C    N
"'HIZZ"              -0.001         -4.266       -4.260       0x5175F    N
"'h3bkm3ach"         -0.002         -5.559       -5.550       0x20C58    N
"VMFh)hhi"           -0.002         -5.242       -5.230       0x5D17A    N
"FcFSC"              -0.003         -4.271       -4.260       0x9000D    N
"VFMFDF_F"           -0.003         -5.244       -5.230       0x86D36    N
"VFMFDF_F"           -0.003         -5.244       -5.230       0x894EE    N
".pcm"               -0.003         -3.720       -3.710       0xAE3C9    N
".PCM"               -0.003         -3.720       -3.710       0xAE3CE    N
"\iLa"               -0.003         -3.721       -3.710       0x3B1FA    N
"aCakF"              -0.003         -4.274       -4.260       0x48757    N
"FDFVFMF[FSC"        -0.004         -5.762       -5.740       0x9153D    N
"C+`LLMKMJNIY"       -0.005         -5.907       -5.880       0x79E9     N
"pS@ap"              -0.005         -4.280       -4.260       0xCFD9     N
"VF_FDFMF"           -0.005         -5.255       -5.230       0x8AC86    N
"%s%u"               -0.005         -3.729       -3.710       0x9E917    N
"FYBYAm"             -0.005         -4.544       -4.520       0x8CA25    N
"kF5pupLN"           -0.006         -5.259       -5.230       0xC53E     N
"jFch"               -0.008         -3.739       -3.710       0x21AB4    N
"C fu"               -0.008         -3.739       -3.710       0x3E076    N
"0BBBAPB"            -0.008         -4.878       -4.840       0x51C77    N
"38&0?3,]"5e7;)=O=97nez" -0.008         -6.583       -6.530       0x93EC4    N
""ckeh"              -0.009         -4.299       -4.260       0x54D27    N
"&0;YBYAd"           -0.009         -5.279       -5.230       0x42DE9    N
```
```
❯ ../target/release/stranger-strings ./rtthread.bin --relaxed |wc -l                   
    4231
```

Let's compare this to the tools you might regularly use:

### cctools-1030.6.3 strings (macOS)
```
/Library/Developer/CommandLineTools/usr/bin/strings ./rtthread.bin| sort -u |wc -l
    8367
```
A lot of visual scanning is required to find any stand-out strings in here.
```
/Library/Developer/CommandLineTools/usr/bin/strings ./rtthread.bin |head

P!#4
P!%T
@ #!
!1C "
0C 

!1C ,
OFFF
FcF{C

eCcFcC
DKFCCJC6
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
## ELF test case - Ubuntu coreutils ls - a Rust binary (2d313ecc9fc058f6e0758abe00c0f6e8)
```
-rwxr-xr-x 115 root root 11352352 Apr 16 22:41 /lib/cargo/bin/coreutils/ls
```

String extractor | Unique strings
--- | ---
macOS strings | 23026
Ubuntu strings | 22957
FLOSS | did not complete within 20 minutes 🤷‍♂️
Ghidra 12.04 (after a few rounds of auto-analysis) | 5030
stranger-strings | 7505

Stranger Strings is naive about the string extraction so Rust binaries are always going to be a bit less precise than a structurally aware parser (like Ghidra, or FLOSS... if it could finish).

My spot checks with very rudimentary MSA and Russian indicate those language models work better than I expected on the Ubuntu ls binary. I used `/usr/share/locale/zh_CN/LC_MESSAGES/apt.mo` (the Chinese translation file for Ubuntu APT) to spot check that model - it seems sensible:

```
String               Adjusted Score Score        Threshold    Offset     Valid
-------------------------------------------------------------------------------------
"或者只能在其他发布源中找到"      3.141          6.423        -3.000       0x8607     Y
"列出所有手动安装的软件包"       3.111          6.333        -3.000       0x897E     Y
"列出所有手动安装的软件包"       3.111          6.333        -3.000       0x8980     Y
"不是一个实包(虚包)"         3.083          6.250        -3.000       0xACBD     Y
"不是一个实包(虚包)"         3.083          6.250        -3.000       0xACBF     Y
"有些软件包不能通过验证"        3.076          6.227        -3.000       0x90A8     Y
"有些软件包不能通过验证"        3.076          6.227        -3.000       0x90AC     Y
"自动卸载所有不再使用的软件包"     3.071          6.214        -3.000       0xAA63     Y
"参数不成对"              3.067          6.200        -3.000       0x6E56     Y
"参数不成对"              3.067          6.200        -3.000       0x6E58     Y
"列出所有自动安装的软件包"       3.056          6.167        -3.000       0x8959     Y
...
```

### Model path behaviour

If `--model` is omitted, the CLI uses `StringModel.sng` embedded at compile time. No external model file is needed at runtime. Use `--model PATH` to load a custom model instead. If you make one please share it back!

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
3. Sum the base-10 log probabilities used by the scorer and divide by the normalised string length.
4. Accept the string when its score is strictly greater than the threshold for that length. Higher (less negative) scores are better.

Selected thresholds from `src/constants.rs`:

| Normalised length | Threshold |
| --- | --- |
| 4 | -2.71 |
| 5 | -3.26 |
| 10 | -4.55 |
| 50 | -6.08 |
| 100 and above | -6.30 |

Normalised strings shorter than four characters cannot pass the threshold of `10.0`; strings shorter than three receive the default score of `-20.0`. The extraction minimum (`-l`, default 4) is separate from this scoring rule. Script-specific scorers use their own logic when language-aware scoring is enabled.

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

For Latin text with a loaded model, scoring is intended to match the original TypeScript implementation and `.sng` model behaviour.

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

PRs are welcome. Keep changes focused, add/adjust tests with behaviour changes, and include CLI/library docs updates when flags or API behaviour change.

Suggested further enhancements include looking at the radare2 string finder/scorers and stress testing unicode matching methods against binaries from weird and wonderful compilers.

Please don't send AI authored PRs unless you've read and understand what it did.

## License

Apache-2.0

StringModel.sng and original approach via NSA's [Ghidra](https://github.com/NationalSecurityAgency/ghidra/).