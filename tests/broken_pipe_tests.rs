use std::process::{Command, Stdio};

#[test]
fn closed_stdout_exits_successfully_without_panicking() {
    for args in [
        vec!["--info"],
        vec!["--test"],
        vec!["--test", "-v"],
        vec!["StringModel.sng"],
        vec!["StringModel.sng", "-v"],
    ] {
        // Close the reader before starting the CLI, avoiding races with small output.
        let mut pipe = Command::new(env!("CARGO_BIN_EXE_stranger-strings"))
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        let writer = pipe.stdin.take().unwrap();
        assert!(pipe.wait().unwrap().success());
        let output = Command::new(env!("CARGO_BIN_EXE_stranger-strings"))
            .args(&args)
            .stdout(Stdio::from(writer))
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if args == ["StringModel.sng", "-v"] {
            let stderr = String::from_utf8(output.stderr).unwrap();
            assert!(stderr.contains("Summary:"));
            assert!(stderr.contains("  Accepted:"));
            assert!(stderr.contains("  Rejected:"));
            assert!(stderr.contains("  Total:"));
            assert!(stderr.contains("  Acceptance rate:"));
            assert!(!stderr.contains("Broken pipe"));
            assert!(!stderr.contains("panicked"));
        } else {
            assert!(
                output.stderr.is_empty(),
                "{args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
