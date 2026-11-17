use std::process::Command;
#[test]
fn exact_mode_settings_files_binary_io_and_no_failed_plaintext() {
    let dir = std::env::temp_dir().join(format!("cube-aes-mode-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let settings = dir.join("private.json");
    let input = dir.join("input.bin");
    let invoke = |direction: &str| {
        Command::new(env!("CARGO_BIN_EXE_cube"))
            .args([
                "--json-errors",
                "cipher",
                "aes",
                "mode",
                direction,
                "--parameters-file",
            ])
            .arg(&settings)
            .arg("--file")
            .arg(&input)
            .output()
            .unwrap()
    };
    let vectors: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../cube-core/src/ciphers/aes_modes/vectors.json"
    ))
    .unwrap();
    for v in vectors {
        std::fs::write(&settings, v["parameters"].to_string()).unwrap();
        let plain = cube_core::encoding::hex_decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
        let expected =
            cube_core::encoding::hex_decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
        std::fs::write(&input, &plain).unwrap();
        let encrypted = invoke("encrypt");
        assert!(
            encrypted.status.success(),
            "{}",
            String::from_utf8_lossy(&encrypted.stderr)
        );
        assert_eq!(encrypted.stdout, expected);
        std::fs::write(&input, &expected).unwrap();
        let decrypted = invoke("decrypt");
        assert!(decrypted.status.success());
        assert_eq!(decrypted.stdout, plain);
    }
    std::fs::write(&settings,serde_json::json!({"mode":"cbc","padding":"pkcs7","key_hex":"00".repeat(16),"iv_hex":"00".repeat(16)}).to_string()).unwrap();
    for len in [0, 1, 15, 16, 17] {
        std::fs::write(&input, vec![0; len]).unwrap();
        let failed = invoke("decrypt");
        assert!(!failed.status.success());
        assert!(failed.stdout.is_empty());
    }
    std::fs::write(&settings, b"{\"key_hex\":\"SECRET\"}").unwrap();
    let failed = invoke("encrypt");
    assert!(!failed.status.success());
    assert!(failed.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&failed.stderr).contains("SECRET"));
    std::fs::remove_dir_all(dir).unwrap();
}
