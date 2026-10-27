use serde_json::Value;
use std::{
    io::Write,
    process::{Command, Stdio},
};
#[test]
fn every_cli_report_matches_real_core_native_and_play_vectors() {
    let cases: Vec<Value> = [
        include_str!("../../cube-core/src/ciphers/book/encoding/vectors.json"),
        include_str!("../../cube-core/src/text/keyboard/coordinates/vectors.json"),
        include_str!("../../cube-core/src/symbols/tokens/vectors.json"),
    ]
    .into_iter()
    .flat_map(|s| serde_json::from_str::<Vec<Value>>(s).unwrap())
    .collect();
    let dir = std::env::temp_dir().join(format!("cube-puzzle-code-vectors-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let parameter_path = dir.join("parameters.json");
    let source_path = dir.join("source.txt");
    for case in cases {
        std::fs::write(
            &parameter_path,
            serde_json::to_vec(&case["parameters"]).unwrap(),
        )
        .unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_cube"));
        for arg in case["command"].as_str().unwrap().split(' ') {
            command.arg(arg);
        }
        command.arg("--parameters").arg(&parameter_path);
        if case["op"].as_str().unwrap().starts_with("text.keyboard.")
            && case["op"].as_str().unwrap().ends_with(".decode")
        {
            command.arg("--decode");
        }
        if let Some(source) = case["inputs"]["source"].as_str() {
            std::fs::write(&source_path, source).unwrap();
            command.arg("--source").arg(&source_path);
        }
        let input = case["inputs"]["text"]
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| serde_json::to_string(&case["inputs"]["coordinates"]).unwrap());
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
            .write_all(input.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}:{}",
            case["op"],
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
            case["outputs"],
            "{}",
            case["op"]
        );
    }
    std::fs::remove_dir_all(dir).unwrap();
}
