use std::fs;
use std::process::Command;

#[test]
fn jsonl_matches_json_for_sorting_filtering_and_file_output() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("sample.bin");
    let destination = dir.path().join("results.jsonl");
    fs::write(
        &input,
        b"hello\0world\0hello\0Error: \"file\\path\"\0xqzfkj\0",
    )
    .unwrap();

    for sort in ["score", "alpha", "offset"] {
        for verbose in [false, true] {
            for unique in [false, true] {
                let run = |format: &str, to_file: bool| {
                    let mut command = Command::new(env!("CARGO_BIN_EXE_stranger-strings"));
                    command
                        .arg(&input)
                        .args(["--format", format, "--sort", sort]);
                    if verbose {
                        command.arg("-v");
                    }
                    if unique {
                        command.arg("--unique");
                    }
                    if to_file {
                        command.arg("--output").arg(&destination);
                    }
                    let output = command.output().unwrap();
                    assert!(
                        output.status.success(),
                        "{}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    output.stdout
                };
                let expected: Vec<serde_json::Value> =
                    serde_json::from_slice(&run("json", false)).unwrap();
                assert!(!expected.is_empty());
                let jsonl = String::from_utf8(run("jsonl", false)).unwrap();
                assert!(jsonl.ends_with('\n'));
                let actual: Vec<serde_json::Value> = jsonl
                    .lines()
                    .map(|line| serde_json::from_str(line).unwrap())
                    .collect();
                assert_eq!(actual, expected);
                assert!(run("jsonl", true).is_empty());
                assert_eq!(fs::read_to_string(&destination).unwrap(), jsonl);
            }
        }
    }
}

#[test]
fn jsonl_without_results_is_empty() {
    let input = tempfile::NamedTempFile::new().unwrap();
    fs::write(input.path(), b"\0\x01\x02").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_stranger-strings"))
        .arg(input.path())
        .args(["-f", "jsonl"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}
