use serde_json::Value;
use std::{
    io::Write,
    process::{Command, Stdio},
};
#[test]
fn every_new_cipher_matches_core_and_native_vectors() {
    let cases: Vec<Value> = serde_json::from_str(include_str!(
        "../../cube-core/src/ciphers/poly/vectors.json"
    ))
    .unwrap();
    let dir = std::env::temp_dir().join(format!("cube-poly-vectors-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("parameters.json");
    for case in cases {
        std::fs::write(&path, serde_json::to_vec(&case["parameters"]).unwrap()).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_cube"));
        command
            .args(["cipher", case["command"].as_str().unwrap(), "--parameters"])
            .arg(&path);
        if case["operation"].as_str().unwrap().ends_with(".decode") {
            command.arg("--decode");
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(case["inputs"]["text"].as_str().unwrap().as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}: {}",
            case["operation"],
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        let expected = &case["outputs"];
        if expected.as_object().unwrap().len() == 1 {
            assert_eq!(
                text.trim_end_matches('\n'),
                expected["text"].as_str().unwrap(),
                "{}",
                case["operation"]
            );
        } else {
            assert_eq!(
                serde_json::from_str::<Value>(&text).unwrap(),
                *expected,
                "{}",
                case["operation"]
            );
        }
    }
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn malformed_parameter_transport_fails_without_echoing_source() {
    let dir = std::env::temp_dir().join(format!("cube-poly-invalid-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("parameters.json");
    for document in [r#"{"key":"1","key":"2"}"#, r#"{"key":"123","extra":true}"#] {
        std::fs::write(&path, document).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(["cipher", "gronsfeld", "--parameters"])
            .arg(&path)
            .args(["--text", "PRIVATE_SOURCE"])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("PRIVATE_SOURCE"));
    }
    std::fs::remove_dir_all(dir).unwrap();
}
