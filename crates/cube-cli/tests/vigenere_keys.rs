use std::process::Command;

#[test]
fn invalid_key_profile_files_fail_without_partial_output_or_source_changes() {
    let dir = std::env::temp_dir().join(format!("cube-key-profiles-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("parameters.json");
    for (command, document) in [
        ("interrupted-key", r#"{"key":"KEY","resets":[2,1]}"#),
        ("interrupted-key", r#"{"key":"KEY","resets":[3]}"#),
        ("interrupted-key", r#"{"key":"KEY","key":"A","resets":[]}"#),
        ("progressive-key", r#"{"key":"KEY"}"#),
        (
            "progressive-key",
            r#"{"key":"KEY","increment":1,"unknown":true}"#,
        ),
        (
            "vigenere-multiplicative",
            r#"{"key":"MOBILE","multiplier":2}"#,
        ),
        (
            "vigenere-multiplicative",
            r#"{"key":"MOBILE","multiplier":0}"#,
        ),
    ] {
        std::fs::write(&path, document).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(["cipher", command, "--parameters"])
            .arg(&path)
            .args(["--text", "ABC"])
            .output()
            .unwrap();
        assert!(!output.status.success(), "{command}");
        assert!(output.stdout.is_empty(), "{command}");
        assert!(!output.stderr.is_empty(), "{command}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), document);
    }
    std::fs::remove_dir_all(dir).unwrap();
}
