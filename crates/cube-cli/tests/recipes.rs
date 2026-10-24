use std::{path::PathBuf, process::Command};
struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("cube-recipes-{}", std::process::id()));
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
fn recipe_cli_uses_caller_bindings_exact_outputs_and_independent_contract_checks() {
    let workspace = Workspace::new();
    let recipe = workspace.path("recipe.json");
    let input = workspace.path("input.bin");
    let output = workspace.path("output.bin");
    let definition = r#"{"format_version":1,"engine_api":1,"input":{"source":"slot","name":"payload","type":"bytes"},"output":{"type":"bytes"},"steps":[{"operation":"encoding.hex.encode","version":1},{"operation":"encoding.hex.decode","version":1}]}"#;
    std::fs::write(&recipe, definition).unwrap();
    std::fs::write(&input, [0, 255, 10]).unwrap();
    let path = recipe.to_str().unwrap();
    let result = run(&["recipe", "validate", path]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let info: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(info["input_slot"], "payload");
    let binding = format!("payload={}", input.display());
    let result = run(&[
        "recipe", "run", path, "--input", &binding, "--format", "raw",
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(result.stdout, [0, 255, 10]);
    let result = run(&[
        "recipe",
        "run",
        path,
        "--hex",
        "payload=00ff",
        "--format",
        "json",
        "--capture",
        "0",
    ]);
    assert!(result.status.success());
    let info: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(info["intermediates"]["0"]["data"], "00ff");
    let result = run(&[
        "recipe",
        "run",
        path,
        "--hex",
        "payload=00ff",
        "--format",
        "raw",
        "--output",
        output.to_str().unwrap(),
    ]);
    assert!(result.status.success());
    assert!(result.stdout.is_empty());
    assert_eq!(std::fs::read(&output).unwrap(), [0, 255]);
    let result = run(&[
        "recipe",
        "run",
        path,
        "--hex",
        "payload=01",
        "--output",
        output.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(std::fs::read(&output).unwrap(), [0, 255]);
    let result = run(&[
        "recipe",
        "run",
        path,
        "--hex",
        "payload=01",
        "--output",
        input.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(std::fs::read(&input).unwrap(), [0, 255, 10]);
    let result = run(&[
        "recipe",
        "check",
        path,
        "--hex",
        "payload=00ff",
        "--expect",
        r#"{"type":"bytes","base64":"AP8="}"#,
    ]);
    assert!(result.status.success());
    let info: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(info["passed"], true);
    for args in [
        vec![
            "recipe",
            "check",
            path,
            "--hex",
            "payload=00ff",
            "--expect",
            r#"{"type":"bytes","base64":""}"#,
        ],
        vec!["recipe", "run", path],
        vec!["recipe", "run", path, "--hex", "wrong=00ff"],
        vec![
            "recipe",
            "run",
            path,
            "--hex",
            "payload=00ff",
            "--text",
            "payload=00ff",
        ],
        vec!["recipe", "run", path, "--text", "payload=00ff"],
        vec![
            "recipe",
            "run",
            path,
            "--value",
            r#"payload={"type":"bytes","base64":"AB=="}"#,
        ],
    ] {
        let result = run(&args);
        assert_eq!(result.status.code(), Some(1), "{args:?}");
        assert!(result.stdout.is_empty());
    }
    let result = run(&[
        "recipe",
        "run",
        path,
        "--value",
        r#"payload={"type":"bytes","base64":"AP8="}"#,
    ]);
    assert!(result.status.success());
    assert_eq!(result.stdout, b"00ff\n");
    let invalid = definition.replace("\"version\":1}", "\"version\":2}");
    std::fs::write(&recipe, invalid).unwrap();
    let result = run(&["--json-errors", "recipe", "validate", path]);
    assert_eq!(result.status.code(), Some(1));
    let error: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(error["error"]["step"], 0);
    std::fs::write(&recipe,r#"{"format_version":1,"engine_api":1,"input":{"source":"inline","value":{"type":"text","data":"$HOME/secret\n"}},"output":{"type":"text"},"steps":[]}"#).unwrap();
    let result = run(&["recipe", "run", path, "--format", "raw"]);
    assert!(result.status.success());
    assert_eq!(result.stdout, b"$HOME/secret\n");
    std::fs::write(&recipe, vec![b' '; 256 * 1024 + 1]).unwrap();
    assert_eq!(run(&["recipe", "validate", path]).status.code(), Some(1));
}
