use std::process::Command;
#[test]
fn numeric_reports_preserve_integers_and_formatting() {
    let r = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(["text", "codepoints", "--format", "unicode", "--text", "A é"])
        .output()
        .unwrap();
    assert!(r.status.success());
    let v: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(v[2]["value"], 233);
    assert_eq!(v[2]["presentation"], "U+00E9");
    let r = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "text",
            "word-values",
            "--sums",
            "--table",
            "[[\"A\",18446744073709551615]]",
            "--text",
            "A",
        ])
        .output()
        .unwrap();
    assert!(r.status.success(), "{:?}", r.stderr);
    let v: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(v[0]["sum"].as_u64(), Some(u64::MAX));
    let r = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "--json-errors",
            "text",
            "word-values",
            "--sums",
            "--max-sum",
            "64",
            "--text",
            "A",
        ])
        .output()
        .unwrap();
    assert!(!r.status.success());
    assert!(r.stdout.is_empty());
}
