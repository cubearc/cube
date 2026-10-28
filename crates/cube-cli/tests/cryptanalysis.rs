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
    let mut expected: std::collections::BTreeSet<_> = cube_core::cryptanalysis::OPERATIONS
        .iter()
        .map(|d| d.id)
        .collect();
    for v in vectors
        .into_iter()
        .filter(|v| v["operation"].as_str().unwrap().starts_with("analysis."))
    {
        let op = cube_core::operations::require(v["operation"].as_str().unwrap()).unwrap();
        assert!(expected.remove(op.id));
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
    assert!(expected.is_empty());
}

#[test]
fn wide_scores_use_lossless_decimal_json_transport() {
    let request = serde_json::json!({"candidates":["A".repeat(32768)],"model":{"kind":"probabilities","n":1,"entries":[{"text":"A","weight":1e-300}],"unseen_probability":1.0,"normalization":"exact"}});
    let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(["cryptanalysis", "score"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(serde_json::to_string(&request).unwrap().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["scores"][0]["total_nanos"],
        (-690775527898_i64 * 32768).to_string()
    );
}
