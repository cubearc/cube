use std::{path::PathBuf, process::Command};
struct Workspace(PathBuf);
impl Workspace {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("cube-investigate-{}-{name}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Workspace {
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
fn triage_cli_preserves_inputs_and_reports_only_evidence_based_suggestions() {
    let workspace = Workspace::new("triage");
    let path = workspace.path("sample.bin");
    let bytes = b"\x89PNG\r\n\x1a\n";
    std::fs::write(&path, bytes).unwrap();
    let path = path.to_str().unwrap();
    let result = run(&["investigate", "triage", path, "--json"]);
    assert!(result.status.success());
    let info: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(info["inspected_bytes"], 8);
    assert_eq!(info["suggestions"][1]["operation"], "image.png.inspect");
    let result = run(&["investigate", "triage", path]);
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("triage.png_chunks.v1"));
    let result = run(&[
        "--json-errors",
        "investigate",
        "triage",
        path,
        "--max-read-bytes",
        "7",
    ]);
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(error["error"]["code"], "resource_limit");
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}
