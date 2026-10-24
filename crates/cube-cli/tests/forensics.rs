use std::{path::PathBuf, process::Command};
struct Files(PathBuf);
impl Files {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("cube-cli-{}-{name}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        Self(dir)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn file_inspection_and_safe_raw_extraction() {
    let files = Files::new("basic");
    let source = files.path("misleading.png");
    let target = files.path("target");
    let bytes = b"%PDF-1.7\0\xffPK\x03\x04";
    std::fs::write(&source, bytes).unwrap();
    let path = source.to_str().unwrap();
    for args in [
        vec!["inspect", path, "--json"],
        vec!["file", "signature", path, "--json"],
        vec!["file", "embedded", path, "--json"],
    ] {
        let output = run(&args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let _: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(output.stderr.is_empty());
    }
    let output = run(&[
        "file", "extract", path, "--offset", "8", "--length", "2", "--stdout",
    ]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"\0\xff");
    let output = run(&[
        "file",
        "extract",
        path,
        "--offset",
        "8",
        "--length",
        "2",
        "--output",
        target.to_str().unwrap(),
    ]);
    assert!(output.status.success());
    assert_eq!(std::fs::read(&target).unwrap(), b"\0\xff");
    let output = run(&[
        "--json-errors",
        "file",
        "extract",
        path,
        "--offset",
        "0",
        "--length",
        "1",
        "--output",
        target.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["code"], "io");
    assert_eq!(std::fs::read(&source).unwrap(), bytes);
    let output = run(&["file", "extract", path, "--offset", "0", "--length", "1"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn hex_strings_and_compare_have_structured_and_human_output() {
    let files = Files::new("ranges");
    let first = files.path("first");
    let second = files.path("second");
    std::fs::write(&first, b"\0hello\xfftail").unwrap();
    std::fs::write(&second, b"\0Hello").unwrap();
    let a = first.to_str().unwrap();
    let b = second.to_str().unwrap();
    for args in [
        vec!["file", "hex", a, "--json"],
        vec!["file", "strings", a, "--json"],
        vec!["file", "compare", a, b, "--json"],
    ] {
        let output = run(&args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let _: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    }
    let output = run(&[
        "file", "hex", a, "--offset", "5", "--length", "2", "--width", "2",
    ]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"00000005  6f ff  |o.|\n");
    let output = run(&["file", "strings", a]);
    assert!(String::from_utf8_lossy(&output.stdout).contains("00000001  5  hello"));
    let output = run(&["file", "compare", a, b]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("EOF"));
}
