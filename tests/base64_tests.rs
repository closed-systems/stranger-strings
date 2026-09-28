use base64::{
    engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD},
    Engine,
};
use stranger_strings::{
    AnalysisOptions, BinaryAnalysisOptions, StrangerStrings, SupportedEncoding,
};

fn analyzer() -> StrangerStrings {
    let mut analyzer = StrangerStrings::new();
    analyzer.load_model(&AnalysisOptions::default()).unwrap();
    analyzer
}

#[test]
fn decoded_binary_strings_have_scores_and_source_offsets() {
    let analyzer = analyzer();
    for engine in [STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD] {
        let encoded = engine.encode(b"\xfb\xff\0hello world\0initialize\0");
        let data = format!("\0\0\"{}\"", encoded);
        let results = analyzer
            .analyze_binary_file(data.as_bytes(), &BinaryAnalysisOptions::default())
            .unwrap();
        for (text, offset) in [("hello world", 3), ("initialize", 15)] {
            let result = results
                .iter()
                .find(|r| r.original_string == text && r.base64_decoded_offset.is_some())
                .unwrap();
            assert_eq!(result.offset, Some(3));
            assert_eq!(result.base64_decoded_offset, Some(offset));
            assert_eq!(result.score, analyzer.analyze_string(text).unwrap().score);
            assert!(result.is_valid);
        }
    }
}

#[test]
fn wrapped_blocks_and_selected_encodings() {
    let analyzer = analyzer();
    let body: Vec<u8> = "hello world"
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    let encoded = STANDARD.encode(body);
    let wrapped = format!("{}\r\n{}", &encoded[..16], &encoded[16..]);
    let options = BinaryAnalysisOptions {
        encodings: Some(vec![SupportedEncoding::Utf16Le]),
        ..Default::default()
    };
    let results = analyzer
        .analyze_binary_file_multi_encoding(wrapped.as_bytes(), &options)
        .unwrap();
    let (result, encoding) = results
        .iter()
        .find(|(r, _)| r.original_string == "hello world")
        .unwrap();
    assert_eq!(*encoding, SupportedEncoding::Utf16Le);
    assert_eq!(result.base64_decoded_offset, Some(0));
    assert_eq!(result.offset, Some(0));
}

#[test]
fn invalid_blocks_and_minimum_length_are_respected() {
    let analyzer = analyzer();
    for data in ["aGVsbG8===", "aGVsbG8=AAAA", "aGVsbG9", "a", ""] {
        let results = analyzer
            .analyze_binary_file(data.as_bytes(), &BinaryAnalysisOptions::default())
            .unwrap();
        assert!(
            results.iter().all(|r| r.base64_decoded_offset.is_none()),
            "{data}"
        );
    }
    let encoded = STANDARD.encode(b"hello\0initialize");
    let options = BinaryAnalysisOptions {
        min_length: Some(6),
        ..Default::default()
    };
    let results = analyzer
        .analyze_binary_file(encoded.as_bytes(), &options)
        .unwrap();
    assert!(!results.iter().any(|r| r.original_string == "hello"));
    assert!(results.iter().any(|r| r.original_string == "initialize"));
}

#[test]
fn cli_reports_decoded_provenance_in_json() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sample.bin");
    std::fs::write(&path, STANDARD.encode(b"hello world")).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_stranger-strings"))
        .args(["--format", "json"])
        .arg(path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let results: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(results
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["original_string"] == "hello world" && r["base64_decoded_offset"] == 0));
}

#[test]
fn unpadded_short_strings_and_unicode_offsets() {
    let analyzer = analyzer();
    for (body, text, offset, encoding) in [
        ("test", "test", 0, SupportedEncoding::Ascii),
        (
            "café menu\0hello world",
            "hello world",
            11,
            SupportedEncoding::Utf8,
        ),
    ] {
        let encoded = STANDARD_NO_PAD.encode(body.as_bytes());
        let options = BinaryAnalysisOptions {
            encodings: Some(vec![encoding]),
            ..Default::default()
        };
        let results = analyzer
            .analyze_binary_file(encoded.as_bytes(), &options)
            .unwrap();
        assert!(results
            .iter()
            .any(|r| r.original_string == text && r.base64_decoded_offset == Some(offset)));
    }
}
