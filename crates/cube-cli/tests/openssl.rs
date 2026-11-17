use std::process::Command;
#[test]
fn explicit_legacy_files_decrypt_without_partial_failed_output() {
    let root = std::env::temp_dir().join(format!("cube-openssl-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let p = root.join("settings.json");
    let f = root.join("input.bin");
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../cube-core/src/ciphers/openssl/vectors.json"
    ))
    .unwrap();
    for v in cases {
        std::fs::write(&p, v["parameters"].to_string()).unwrap();
        let data = cube_core::encoding::hex_decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
        std::fs::write(&f, &data).unwrap();
        let run = || {
            Command::new(env!("CARGO_BIN_EXE_cube"))
                .args(["cipher", "openssl", "decrypt", "--parameters-file"])
                .arg(&p)
                .arg("--file")
                .arg(&f)
                .output()
                .unwrap()
        };
        let result = run();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            result.stdout,
            cube_core::encoding::hex_decode(v["plaintext_hex"].as_str().unwrap()).unwrap()
        );
        std::fs::write(&f, &data[..data.len() - 1]).unwrap();
        let failed = run();
        assert!(!failed.status.success());
        assert!(failed.stdout.is_empty());
    }
    std::fs::remove_dir_all(root).unwrap();
}
