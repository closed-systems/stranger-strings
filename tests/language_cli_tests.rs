use base64::{engine::general_purpose::STANDARD, Engine};
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn selected_languages_reach_scorers_for_files_base64_and_stdin() {
    let dir = tempfile::tempdir().unwrap();
    for (language, text, scorer) in [
        ("arabic", "مرحبا بالعالم", "Arabic"),
        ("chinese", "你好世界", "Chinese"),
        ("russian", "Привет мир", "Cyrillic"),
    ] {
        let path = dir.path().join("input.bin");
        let body = format!("{}\0{}\0", text, STANDARD.encode(text));
        std::fs::write(&path, body).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_stranger-strings"))
            .args(["-L", language, "-v", "-f", "json"])
            .arg(&path)
            .output()
            .unwrap();
        assert!(output.status.success());
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
        let matches: Vec<_> = rows
            .iter()
            .filter(|r| r["original_string"] == text)
            .collect();
        assert_eq!(matches.len(), 2, "{language}: {rows:?}");
        assert!(matches.iter().all(|r| r["scorer_name"] == scorer));
        assert!(matches.iter().any(|r| r["base64_decoded_offset"] == 0));

        let mut child = Command::new(env!("CARGO_BIN_EXE_stranger-strings"))
            .args(["-L", language, "-v", "-f", "json", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(text.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
        assert!(!rows.is_empty());
        assert!(rows.iter().all(|r| r["scorer_name"] == scorer));
    }
}

#[test]
fn multiple_languages_restrict_detected_scripts() {
    use stranger_strings::{AnalysisOptions, BinaryAnalysisOptions, ScriptType, StrangerStrings};
    let mut analyzer = StrangerStrings::new();
    analyzer.load_model(&AnalysisOptions::default()).unwrap();
    let options = BinaryAnalysisOptions {
        target_languages: Some(vec![ScriptType::Arabic, ScriptType::Cyrillic]),
        use_language_scoring: true,
        ..Default::default()
    };
    let results = analyzer
        .analyze_binary_file(
            "مرحبا بالعالم\0Привет мир\0hello world\0".as_bytes(),
            &options,
        )
        .unwrap();
    assert!(results
        .iter()
        .any(|r| r.scorer_name.as_deref() == Some("Arabic")));
    assert!(results
        .iter()
        .any(|r| r.scorer_name.as_deref() == Some("Cyrillic")));
    assert!(!results.iter().any(|r| r.original_string == "hello world"));
}
