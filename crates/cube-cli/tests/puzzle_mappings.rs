use std::process::{Command, Output};
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .output()
        .unwrap()
}
fn success(args: &[&str]) -> String {
    let result = run(args);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout).unwrap()
}
#[test]
fn explicit_puzzle_cli_vectors() {
    assert_eq!(
        success(&[
            "cipher",
            "polybius",
            "--alphabet",
            "ABCD",
            "--side",
            "2",
            "--alias",
            "==A",
            "--text",
            "="
        ]),
        "1,1\n"
    );
    assert_eq!(
        success(&["cipher", "bacon", "--variant", "extended26", "--text", "AB"]),
        "AAAAA AAAAB\n"
    );
    assert_eq!(
        success(&[
            "cipher",
            "bacon",
            "--variant",
            "classical24",
            "--decode",
            "--text",
            "ABAAA BAABB"
        ]),
        "IU\n"
    );
    assert_eq!(
        success(&[
            "cipher",
            "bacon",
            "--variant",
            "extended26",
            "--notation",
            "binary_inverted",
            "--text",
            "B"
        ]),
        "11110\n"
    );
    assert_eq!(
        success(&["cipher", "polybius", "--preset", "tap", "--text", "CK J"]),
        "1,3 1,3 / 2,5\n"
    );
    assert_eq!(
        success(&["cipher", "polybius", "--decode", "--text", "1,1 / 2,4"]),
        "A I\n"
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&success(&[
            "cipher",
            "polybius",
            "--alphabet",
            "αβγδ",
            "--side",
            "2",
            "--index-base",
            "0",
            "--text",
            "δ α",
            "--json"
        ]))
        .unwrap(),
        serde_json::json!([[1, 1], null, [0, 0]])
    );
    assert_eq!(
        success(&[
            "cipher",
            "polybius",
            "--alphabet",
            "αβγδ",
            "--side",
            "2",
            "--alias",
            "a=α",
            "--text",
            "a"
        ]),
        "1,1\n"
    );
    assert_eq!(
        success(&[
            "text", "keyboard", "--source", "qwerty", "--target", "dvorak", "--text", "qwerty"
        ]),
        "',.pyf\n"
    );
    assert_eq!(
        success(&["encode", "keypad", "--mode", "telephone", "--text", "HELLO"]),
        "43556\n"
    );
    assert_eq!(
        success(&["encode", "keypad", "--mode", "multi_tap", "--text", "HELLO"]),
        "44|33|555|555|666\n"
    );
    let report: serde_json::Value = serde_json::from_str(&success(&[
        "decode",
        "keypad",
        "--mode",
        "telephone",
        "--text",
        "4",
    ]))
    .unwrap();
    assert_eq!(
        report,
        serde_json::json!({"text":null,"alternatives":["GHI"],"reversible":false})
    );
    let report: serde_json::Value = serde_json::from_str(&success(&[
        "text",
        "class",
        "--unmapped",
        "filter",
        "--text",
        "éa!B",
    ]))
    .unwrap();
    assert_eq!(report["mapped_text"], "01");
    assert_eq!(report["records"][0]["byte_offset"], 2);
    assert_eq!(report["filtered_count"], 2);
}
#[test]
fn custom_mapping_cli_and_structured_failures() {
    let layout = r#"{"geometry":"one","entries":[{"key":"k","layer":"base","symbol":"α"}]}"#;
    let target = r#"{"geometry":"one","entries":[{"key":"k","layer":"base","symbol":"🧊"}]}"#;
    assert_eq!(
        success(&[
            "text", "keyboard", "--source", layout, "--target", target, "--text", "α"
        ]),
        "🧊\n"
    );
    let mapping = r#"{"ascii_fold":false,"keys":[{"digit":"0","symbols":" 0"}]}"#;
    assert_eq!(
        success(&[
            "encode",
            "keypad",
            "--mapping",
            mapping,
            "--mode",
            "multi_tap",
            "--text",
            "0 "
        ]),
        "00|0\n"
    );
    let classes = r#"{"ascii_fold":false,"classes":[{"name":"x","characters":"αβ","symbol":"!"}]}"#;
    let report: serde_json::Value = serde_json::from_str(&success(&[
        "text",
        "class",
        "--definition",
        classes,
        "--text",
        "α",
    ]))
    .unwrap();
    assert_eq!(report["mapped_text"], "!");
    for args in [
        vec![
            "cipher",
            "bacon",
            "--variant",
            "extended26",
            "--decode",
            "--text",
            "AAAA",
        ],
        vec!["cipher", "polybius", "--decode", "--text", "0,1"],
        vec!["decode", "keypad", "--mode", "multi_tap", "--text", "4433"],
        vec!["text", "class", "--text", "Y"],
        vec![
            "text",
            "keyboard",
            "--source",
            "qwerty",
            "--target",
            "dvorak",
            "--unmapped",
            "error",
            "--text",
            "é",
        ],
    ] {
        let mut all = vec!["--json-errors"];
        all.extend(args);
        let output = run(&all);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert!(error["error"]["code"].as_str().is_some());
    }
}
