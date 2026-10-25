use std::process::Command;
#[test]
fn traversal_cli_keeps_evidence_and_finite_failures() {
    for (command, expected) in [("keyed-skip", "BDACE"), ("josephus", "BDAEC")] {
        let output = Command::new(env!("CARGO_BIN_EXE_cube"))
            .args([
                "text", command, "--steps", "2", "--count", "5", "--text", "ABCDE",
            ])
            .output()
            .unwrap();
        assert!(output.status.success(), "{:?}", output.stderr);
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["extracted"], expected);
        assert_eq!(report["records"][0]["scalar_index"], 1);
    }
    let output = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "--json-errors",
            "text",
            "keyed-skip",
            "--steps",
            "2",
            "--count",
            "3",
            "--text",
            "ABCDE",
            "--no-wrap",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["operation"], "text.skip.keyed");
}
