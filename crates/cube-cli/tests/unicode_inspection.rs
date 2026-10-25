use std::process::Command;
#[test]
fn unicode_cli_retains_original_and_normalized_evidence() {
    let r = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "text",
            "unicode-compare",
            "--normalization",
            "NFC",
            "--text",
            "é",
            "--other",
            "e\u{301}",
        ])
        .output()
        .unwrap();
    assert!(r.status.success(), "{:?}", r.stderr);
    let report: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(report["exact_equal"], false);
    assert_eq!(report["normalized_equal"], true);
    assert_eq!(report["original"]["records"][1]["right"]["byte_offset"], 1);
    assert_eq!(
        report["normalized"]["coordinate_space"],
        "normalized_scalar_index"
    );
    let r = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "text",
            "unicode-inspect",
            "--text",
            "A\u{200d}",
            "--max-results",
            "0",
        ])
        .output()
        .unwrap();
    assert!(r.status.success());
    let report: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(report["counts"]["format"], 1);
    assert_eq!(report["records"], serde_json::json!([]));
    assert_eq!(report["complete"], false);
    let r = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "--json-errors",
            "text",
            "unicode-inspect",
            "--text",
            "A",
            "--max-work",
            "0",
        ])
        .output()
        .unwrap();
    assert!(!r.status.success());
    assert!(r.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&r.stderr).unwrap();
    assert_eq!(error["error"]["code"], "resource_limit");
}
