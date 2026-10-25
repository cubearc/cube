use std::process::Command;
#[test]
fn word_cli_shared_per_word_and_errors_preserve_evidence() {
    for (args, expected) in [
        (
            vec![
                "--positions",
                "[1,3]",
                "--text",
                "Cube Uses Binary Evidence",
            ],
            "CbUeBnEi",
        ),
        (
            vec!["--per-word", "[[2,1,2],[-1]]", "--text", "ab cd"],
            "babd",
        ),
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(["text", "word-extract"])
            .args(args)
            .output()
            .unwrap();
        assert!(result.status.success(), "{:?}", result.stderr);
        let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(report["extracted"], expected);
    }
    let result = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "--json-errors",
            "text",
            "word-extract",
            "--positions",
            "[0]",
            "--text",
            "A",
        ])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    let e: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(e["error"]["operation"], "text.word.extract");
}
