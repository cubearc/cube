use serde_json::Value;
use std::process::Command;
#[test]
fn cli_contracts_are_opt_in_complete_and_bounded() {
    let output = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(["ops", "list", "--json", "--contracts"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        value["operations"].as_array().unwrap().len(),
        cube_core::operations::catalog().len()
    );
    assert!(
        value["operations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|op| op["contract"]["outputs"].is_array())
    );
    let maximum = output.stdout.len().to_string();
    let exact = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "ops",
            "list",
            "--json",
            "--contracts",
            "--max-output-bytes",
            &maximum,
        ])
        .output()
        .unwrap();
    assert!(exact.status.success());
    assert_eq!(exact.stdout, output.stdout);
    let below = (output.stdout.len() - 1).to_string();
    let rejected = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "ops",
            "list",
            "--json",
            "--contracts",
            "--max-output-bytes",
            &below,
        ])
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    assert!(rejected.stdout.is_empty());
    let legacy = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(["ops", "describe", "image.histogram", "--json"])
        .output()
        .unwrap();
    let legacy: Value = serde_json::from_slice(&legacy.stdout).unwrap();
    assert!(legacy["operation"].get("contract").is_none());
}
