use std::process::{Command, Stdio};

#[test]
fn closed_stdout_exits_successfully_without_panicking() {
    for args in [
        vec!["--info"],
        vec!["--test"],
        vec!["--test", "-v"],
        vec!["StringModel.sng"],
    ] {
        // Close the reader before starting the CLI, avoiding races with small output.
        let mut pipe = Command::new("sh")
            .args(["-c", "exit 0"])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        let writer = pipe.stdin.take().unwrap();
        pipe.wait().unwrap();
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
        assert!(
            output.stderr.is_empty(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
