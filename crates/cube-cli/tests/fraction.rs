use serde_json::Value;
use std::{
    io::Write,
    process::{Command, Stdio},
};
#[test]
fn all_transposition_and_fractionation_cli_results_match_core() {
    let cases: Vec<Value> = [
        include_str!("../../cube-core/src/ciphers/transposition/vectors.json"),
        include_str!("../../cube-core/src/ciphers/fractionation/vectors.json"),
    ]
    .into_iter()
    .flat_map(|s| serde_json::from_str::<Vec<Value>>(s).unwrap())
    .collect();
    let dir = std::env::temp_dir().join(format!("cube-fraction-vectors-{}", std::process::id()));
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
            .write_all(
                case["inputs"]["text"]
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| serde_json::to_string(&case["inputs"]["tokens"]).unwrap())
                    .as_bytes(),
            )
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
        if expected.get("tokens").is_some() {
            assert_eq!(
                serde_json::from_str::<Value>(&text).unwrap(),
                expected["tokens"]
            );
        } else if expected.as_object().unwrap().len() == 1 {
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
