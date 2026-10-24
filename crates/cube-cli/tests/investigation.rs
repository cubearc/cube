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

#[test]
fn session_cli_has_explicit_private_saving_safe_reports_and_input_verified_replay() {
    let workspace = Workspace::new("session");
    let source = workspace.path("source.bin");
    let record = workspace.path("record.json");
    let secret = b"\x00\xffPRIVATE\n";
    std::fs::write(&source, secret).unwrap();
    let path = source.to_str().unwrap();
    let saved = record.to_str().unwrap();
    let steps = r#"[{"operation":"encoding.hex.encode","version":1},{"operation":"encoding.hex.decode","version":1}]"#;
    let result = run(&[
        "investigate",
        "pipeline",
        path,
        "--steps",
        steps,
        "--identify",
        "--save",
        saved,
        "--json",
        "--hypothesis",
        "Maybe \x1b[31mhidden",
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let info: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(info["status"], "complete");
    assert!(info["session"]["runs"][1]["retained"].is_null());
    assert_eq!(info["session"]["hypotheses"], serde_json::json!([]));
    assert!(!String::from_utf8_lossy(&std::fs::read(&record).unwrap()).contains("PRIVATE"));
    let original = std::fs::read(&record).unwrap();
    let result = run(&[
        "investigate",
        "pipeline",
        path,
        "--steps",
        steps,
        "--identify",
        "--save",
        saved,
    ]);
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(std::fs::read(&record).unwrap(), original);
    let result = run(&["investigate", "replay", saved, path, "--json"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let result = run(&[
        "investigate",
        "report",
        saved,
        "--hypothesis",
        "Maybe \x1b[31mhidden",
    ]);
    assert!(result.status.success());
    assert!(!result.stdout.contains(&27));
    assert!(String::from_utf8_lossy(&result.stdout).contains("User hypothesis"));
    let result = run(&[
        "investigate",
        "pipeline",
        path,
        "--steps",
        steps,
        "--identify",
        "--include-result",
        "2",
        "--json",
    ]);
    assert!(result.status.success());
    let selected: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(
        selected["session"]["runs"][1]["retained"]["base64"],
        "AP9QUklWQVRFCg=="
    );
    let result = run(&[
        "investigate",
        "pipeline",
        path,
        "--steps",
        steps,
        "--save",
        path,
    ]);
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(std::fs::read(path).unwrap(), secret);
    std::fs::write(&source, b"\x01\xffPRIVATE\n").unwrap();
    let result = run(&["--json-errors", "investigate", "replay", saved, path]);
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("identity changed"));
}

#[test]
fn profile_cli_dry_run_limits_and_saved_provenance_agree() {
    let workspace = Workspace::new("profile");
    let input = workspace.path("input.bin");
    let profile = workspace.path("profile.json");
    let record = workspace.path("session.json");
    std::fs::write(&input, b"abcdef").unwrap();
    let path = input.to_str().unwrap();
    let result = run(&["investigate", "plan", path, "--depth", "preview", "--json"]);
    assert!(result.status.success());
    let plan: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(plan["input_read_bytes"], 6);
    assert_eq!(plan["runs"].as_array().unwrap().len(), 3);
    let result = run(&[
        "investigate",
        "run",
        path,
        "--depth",
        "preview",
        "--identify",
        "--save",
        record.to_str().unwrap(),
        "--json",
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let session: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(session["status"], "complete");
    assert_eq!(session["session"]["profile"]["operation_units"], 4);
    assert_eq!(session["session"]["profile"]["input_read_bytes"], 6);
    let result = run(&["investigate", "replay", record.to_str().unwrap(), path]);
    assert!(result.status.success());
    let result = run(&[
        "investigate",
        "run",
        path,
        "--depth",
        "preview",
        "--budgets",
        r#"{"max_operations":1}"#,
        "--json",
    ]);
    assert!(result.status.success());
    let session: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(session["status"], "partial");
    assert_eq!(session["session"]["runs"][1]["state"], "skipped");
    std::fs::write(&profile,r#"{"format_version":1,"depth":"preview","budgets":{"max_input_bytes":5,"max_operations":2}}"#).unwrap();
    let result = run(&[
        "investigate",
        "plan",
        path,
        "--profile",
        profile.to_str().unwrap(),
        "--budgets",
        r#"{"max_operations":1}"#,
        "--json",
    ]);
    assert!(result.status.success());
    let plan: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(plan["budgets"]["max_input_bytes"], 5);
    assert_eq!(plan["budgets"]["max_operations"], 1);
    assert_eq!(plan["input_read_bytes"], 0);
    let result = run(&[
        "investigate",
        "run",
        path,
        "--depth",
        "preview",
        "--include-result",
        "2",
        "--json",
    ]);
    assert!(result.status.success());
    let session: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(
        session["session"]["runs"][1]["retained"][0]["hex"],
        "616263646566"
    );
    std::fs::write(&profile,r#"{"format_version":1,"pipeline":{"steps":[{"operation":"encoding.hex.encode","version":1},{"operation":"shell.exec","version":1}]}}"#).unwrap();
    let result = run(&[
        "investigate",
        "run",
        path,
        "--profile",
        profile.to_str().unwrap(),
        "--save",
        workspace.path("never.json").to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(1));
    assert!(!workspace.path("never.json").exists());
    assert_eq!(std::fs::read(&input).unwrap(), b"abcdef");
}
