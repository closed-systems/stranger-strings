use std::io::Write;
use std::process::{Command, Stdio};

fn run(format: &str, sort: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_stranger-strings"))
        .args(["-", "-v", "--format", format, "--sort", sort])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"hello initialize world function test xqzfkj a")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn score_sort_uses_difference_and_reports_it() {
    for sort in ["score", "offset"] {
        let rows: Vec<serde_json::Value> = serde_json::from_str(&run("json", sort)).unwrap();
        assert_eq!(rows.last().unwrap()["original_string"], "a");
        assert!(rows.last().unwrap()["adjusted_score"].is_null());
        assert_eq!(rows.last().unwrap()["is_valid"], false);
        let rows = &rows[..rows.len() - 1];
        for row in rows {
            let expected = row["score"].as_f64().unwrap() - row["threshold"].as_f64().unwrap();
            assert!((row["adjusted_score"].as_f64().unwrap() - expected).abs() < 1e-12);
        }
        assert!(rows
            .windows(2)
            .all(|pair| pair[0]["adjusted_score"].as_f64().unwrap()
                >= pair[1]["adjusted_score"].as_f64().unwrap()));
        // Ensure this fixture distinguishes difference sorting from raw score sorting.
        assert!(rows
            .windows(2)
            .any(|pair| pair[0]["score"].as_f64().unwrap() < pair[1]["score"].as_f64().unwrap()));
    }
}

#[test]
fn text_and_csv_include_adjusted_score() {
    assert!(run("text", "score")
        .lines()
        .next()
        .unwrap()
        .contains("Adjusted Score"));
    assert!(run("text", "score").lines().last().unwrap().contains("N/A"));
    let csv = run("csv", "score");
    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    assert_eq!(&reader.headers().unwrap()[1], "adjusted_score");
    for row in reader.records() {
        let row = row.unwrap();
        if &row[0] == "a" {
            assert!(row[1].is_empty());
            continue;
        }
        let expected = row[2].parse::<f64>().unwrap() - row[3].parse::<f64>().unwrap();
        assert!((row[1].parse::<f64>().unwrap() - expected).abs() < 1e-12);
    }
}

#[test]
fn eligibility_uses_normalized_length_in_both_scoring_paths() {
    use stranger_strings::{AnalysisOptions, StrangerStrings};
    let mut analyzer = StrangerStrings::new();
    analyzer.load_model(&AnalysisOptions::default()).unwrap();
    for language_scoring in [false, true] {
        for text in ["", "a", "abc", "   abc   "] {
            let result = analyzer.analyze_string_with_options(text, None, language_scoring, None).unwrap();
            assert!(!result.is_valid);
            assert_eq!(result.adjusted_score(), None);
        }
        let result = analyzer.analyze_string_with_options("hello", None, language_scoring, None).unwrap();
        assert!(result.adjusted_score().is_some());
    }
}

#[test]
fn adjusted_difference_handles_zero_and_positive_thresholds() {
    use stranger_strings::{AnalysisOptions, StrangerStrings};
    let mut analyzer = StrangerStrings::new();
    analyzer.load_model(&AnalysisOptions::default()).unwrap();
    let mut result = analyzer.analyze_string("hello").unwrap();
    for (score, threshold, expected) in [(2.0, 0.0, 2.0), (2.0, 3.0, -1.0), (3.0, 3.0, 0.0)] {
        result.score = score;
        result.threshold = threshold;
        assert_eq!(result.adjusted_score(), Some(expected));
    }
}
