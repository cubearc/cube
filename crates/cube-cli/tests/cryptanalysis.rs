use serde_json::Value;
use std::{
    io::Write,
    process::{Command, Stdio},
};
#[test]
fn analysis_cli_executes_the_reviewed_core_fixtures() {
    let vectors: Vec<Value> = serde_json::from_str(include_str!(
        "../../cube-core/src/operations/contracts/conformance.json"
    ))
    .unwrap();
    for v in vectors
        .into_iter()
        .filter(|v| v["operation"].as_str().unwrap().starts_with("analysis."))
    {
        let op = cube_core::operations::require(v["operation"].as_str().unwrap()).unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(op.command.split_whitespace())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(
                serde_json::to_string(&v["inputs"]["request"])
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();
        let result = child.wait_with_output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&result.stdout).unwrap(),
            v["outputs"]["report"]
        );
    }
}
