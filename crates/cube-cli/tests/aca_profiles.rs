//! Actual CLI transports must agree with the reviewed independent Core fixtures.
use serde_json::Value;
use std::{
    io::Write,
    process::{Command, Stdio},
};
#[test]
fn aca_profile_cli_invocations_match_the_published_answers() {
    let cases: Vec<Value> = serde_json::from_str(include_str!(
        "../../cube-core/src/operations/contracts/conformance.json"
    ))
    .unwrap();
    let directory = std::env::temp_dir().join(format!("cube-aca-profiles-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let parameters = directory.join("parameters.json");
    let mut checked = 0;
    for case in cases.iter().filter(|c| {
        c["operation"].as_str().is_some_and(|op| {
            [
                "numbered_key",
                "headlines",
                "key_phrase",
                "arnold",
                "solresol",
                "prime_number",
                "prime_multiplication",
                "modulo",
                "triliteral",
            ]
            .contains(&op.split('.').nth(1).unwrap_or(""))
        })
    }) {
        std::fs::write(
            &parameters,
            serde_json::to_vec(&case["parameters"]).unwrap(),
        )
        .unwrap();
        let operation = case["operation"].as_str().unwrap();
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_cube"));
        let command = operation.split('.').nth(1).unwrap().replace('_', "-");
        let symbolic = operation.starts_with("symbol.solresol.");
        let mode = if symbolic {
            if operation.ends_with("decode") {
                "decode"
            } else {
                "encode"
            }
        } else {
            "cipher"
        };
        cmd.args([mode, &command, "--parameters"]).arg(&parameters);
        if operation.ends_with("decode") && !symbolic {
            cmd.arg("--decode");
        }
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let input = if operation.starts_with("cipher.arnold.") {
            serde_json::to_string(&case["inputs"]).unwrap()
        } else if let Some(text) = case["inputs"].get("text").and_then(Value::as_str) {
            text.into()
        } else {
            serde_json::to_string(case["inputs"].as_object().unwrap().values().next().unwrap())
                .unwrap()
        };
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{operation}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
            case["outputs"],
            "{operation}"
        );
        checked += 1;
    }
    assert_eq!(checked, 18);
    std::fs::remove_dir_all(directory).unwrap();
}
