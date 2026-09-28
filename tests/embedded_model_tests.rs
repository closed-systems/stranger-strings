use std::process::Command;
use stranger_strings::{AnalysisOptions, StrangerStrings};

#[test]
fn embedded_model_matches_file_scores() {
    let mut embedded = StrangerStrings::new();
    embedded.load_model(&AnalysisOptions::default()).unwrap();
    let mut external = StrangerStrings::new();
    external
        .load_model(&AnalysisOptions {
            model_path: Some(format!("{}/StringModel.sng", env!("CARGO_MANIFEST_DIR"))),
            ..Default::default()
        })
        .unwrap();
    for text in ["hello world", "initialize", "xqzfkj", "HELLO"] {
        assert_eq!(
            embedded.analyze_string(text).unwrap(),
            external.analyze_string(text).unwrap()
        );
    }
}

#[test]
fn cli_works_without_external_model_and_honors_overrides() {
    let dir = tempfile::tempdir().unwrap();
    let binary = dir
        .path()
        .join(format!("stranger-strings{}", std::env::consts::EXE_SUFFIX));
    std::fs::copy(env!("CARGO_BIN_EXE_stranger-strings"), &binary).unwrap();
    std::fs::write(dir.path().join("input.bin"), b"hello world\0initialize\0").unwrap();
    for args in [
        vec!["--info"],
        vec!["--test"],
        vec!["input.bin"],
        vec!["--auto-detect", "input.bin"],
    ] {
        let output = Command::new(&binary)
            .current_dir(dir.path())
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!output.stdout.is_empty());
    }
    std::fs::write(
        dir.path().join("custom.sng"),
        "# Model Type: mixed\nh\te\tl\t10\n",
    )
    .unwrap();
    let output = Command::new(&binary)
        .current_dir(dir.path())
        .args(["--model", "custom.sng", "--info"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Type: mixed"));
    let output = Command::new(&binary)
        .current_dir(dir.path())
        .args(["--model", "missing.sng", "--info"])
        .output()
        .unwrap();
    assert!(!output.status.success());
}
