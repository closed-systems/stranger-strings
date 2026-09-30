use std::io::Write;
use std::process::{Command, Stdio};

fn run(extra: &[&str]) -> Vec<serde_json::Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_stranger-strings"))
        .args(["-", "--format", "json", "--sort", "alpha"])
        .args(extra)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"hello world function initialize main sizeof free XML abc xqzfkj")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn relaxed_cli_changes_thresholds_and_filtering() {
    for language in [vec![], vec!["--language", "latin"]] {
        let mut strict_args = language.clone();
        strict_args.push("-v");
        let strict = run(&strict_args);
        strict_args.push("--relaxed");
        let relaxed = run(&strict_args);
        let mut newly_accepted = 0;
        for (before, after) in strict.iter().zip(&relaxed) {
            assert_eq!(before["original_string"], after["original_string"]);
            assert_eq!(before["score"], after["score"]);
            let threshold = before["threshold"].as_f64().unwrap();
            assert_eq!(
                after["threshold"].as_f64().unwrap(),
                if threshold < 0.0 {
                    threshold - 1.0
                } else {
                    threshold
                }
            );
            if before["is_valid"] == false && after["is_valid"] == true {
                newly_accepted += 1;
            }
        }
        assert!(newly_accepted > 0);
        let mut args = language;
        args.push("--relaxed");
        assert_eq!(
            run(&args),
            relaxed
                .into_iter()
                .filter(|row| row["is_valid"] == true)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn relaxed_preserves_script_eligibility_and_short_string_rejection() {
    use stranger_strings::{AnalysisOptions, ScriptType, StrangerStrings};
    let mut analyzer = StrangerStrings::new();
    analyzer.load_model(&AnalysisOptions::default()).unwrap();
    analyzer.set_relaxed(true);
    for text in ["", "a", "abc", "  abc  "] {
        assert!(!analyzer.analyze_string(text).unwrap().is_valid);
    }
    for (script, text) in [
        (ScriptType::Han, "你好世界"),
        (ScriptType::Arabic, "مرحبا"),
        (ScriptType::Cyrillic, "привет"),
    ] {
        let result = analyzer
            .analyze_string_with_options(text, None, true, Some(script))
            .unwrap();
        assert_eq!(result.threshold, -4.0);
        let wrong_script = analyzer
            .analyze_string_with_options("hello", None, true, Some(script))
            .unwrap();
        assert_eq!(wrong_script.threshold, 10.0);
        assert!(!wrong_script.is_valid);
    }
}

#[test]
fn custom_threshold_adjustments_and_alias() {
    assert_eq!(run(&["-v", "--relaxed"]), run(&["-v", "--threshold", "1"]));
    let defaults = run(&["-v"]);
    for value in ["0", "2", "0.5", "-1"] {
        let adjustment: f64 = value.parse().unwrap();
        let adjusted = run(&["-v", "--threshold", value]);
        for (before, after) in defaults.iter().zip(adjusted) {
            let threshold = before["threshold"].as_f64().unwrap();
            let expected = if threshold < 0.0 {
                threshold - adjustment
            } else {
                threshold
            };
            assert_eq!(after["score"], before["score"]);
            assert_eq!(after["threshold"].as_f64().unwrap(), expected);
            assert_eq!(
                after["is_valid"],
                after["score"].as_f64().unwrap() > expected
            );
        }
    }
}

#[test]
fn rejects_invalid_or_conflicting_threshold_options() {
    for args in [
        vec!["--threshold", "NaN"],
        vec!["--threshold", "inf"],
        vec!["--threshold", "-inf"],
        vec!["--threshold", "abc"],
        vec!["--threshold"],
        vec!["--threshold", "2", "--relaxed"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_stranger-strings"))
            .arg("--info")
            .args(&args)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{args:?}");
    }
}
