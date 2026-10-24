use std::{
    io::Write,
    process::{Command, Stdio},
};
fn run(args: &[&str], data: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(data);
    child.wait_with_output().unwrap()
}
#[test]
fn pipeline_cli_preserves_binary_stdout_validates_and_reports_steps() {
    let steps = r#"[{"operation":"encoding.hex.encode","version":1},{"operation":"encoding.hex.decode","version":1}]"#;
    let output = run(
        &[
            "pipeline",
            "validate",
            "--steps",
            steps,
            "--input-type",
            "bytes",
        ],
        b"",
    );
    assert!(output.status.success());
    let info: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(info["output_type"], "bytes");
    let output = run(
        &["pipeline", "run", "--steps", steps, "--format", "raw"],
        &[0, 255, 10],
    );
    assert!(output.status.success());
    assert_eq!(output.stdout, [0, 255, 10]);
    let output = run(
        &[
            "pipeline",
            "run",
            "--steps",
            steps,
            "--hex",
            "00ff",
            "--capture",
            "0",
            "--format",
            "json",
        ],
        b"",
    );
    assert!(output.status.success());
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["output"]["base64"], "AP8=");
    assert_eq!(result["intermediates"]["0"]["data"], "00ff");
    let output = run(
        &[
            "--json-errors",
            "pipeline",
            "validate",
            "--steps",
            r#"[{"operation":"encoding.hex.decode","version":1},{"operation":"file.range.extract","version":1}]"#,
            "--input-type",
            "text",
        ],
        b"",
    );
    assert_eq!(output.status.code(), Some(1));
    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["step"], 1);
    let output = run(&["text", "to-bytes", "--text", "é", "--output", "raw"], b"");
    assert_eq!(output.stdout, "é".as_bytes());
    let output = run(&["text", "from-bytes"], &[255]);
    assert_eq!(output.status.code(), Some(1));
}
