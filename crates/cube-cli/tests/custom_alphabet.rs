use std::process::Command;
#[test]
fn custom_alphabet_cli_preserves_exact_text_and_errors() {
    let args = [
        "number",
        "alphabet",
        "--source-alphabet",
        "01",
        "--target-alphabet",
        "abcdefghij",
        "--from-base",
        "2",
        "--to-base",
        "10",
        "--text",
    ];
    let output = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .arg("1010")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"ba\n");
    let output = Command::new(env!("CARGO_BIN_EXE_cube"))
        .arg("--json-errors")
        .args(args)
        .arg("2")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["operation"], "number.alphabet.convert");
}
