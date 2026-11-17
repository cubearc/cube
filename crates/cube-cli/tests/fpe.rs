use std::process::Command;
#[test]
fn numeric_json_and_secret_settings_files_execute_all_known_profiles() {
    let root = std::env::temp_dir().join(format!("cube-fpe-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let vectors: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../cube-core/src/ciphers/fpe/vectors.json")).unwrap();
    for v in vectors {
        let p = root.join("settings.json");
        let f = root.join("digits.json");
        std::fs::write(&p, v["parameters"].to_string()).unwrap();
        for (direction, input, expected) in [
            ("encrypt", &v["plaintext"], &v["ciphertext"]),
            ("decrypt", &v["ciphertext"], &v["plaintext"]),
        ] {
            std::fs::write(&f, input.to_string()).unwrap();
            let o = Command::new(env!("CARGO_BIN_EXE_cube"))
                .args(["cipher", "fpe", direction, "--parameters-file"])
                .arg(&p)
                .arg("--file")
                .arg(&f)
                .output()
                .unwrap();
            assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&o.stdout).unwrap(),
                *expected
            );
        }
        std::fs::write(&f, "[0]").unwrap();
        let o = Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(["cipher", "fpe", "encrypt", "--parameters-file"])
            .arg(&p)
            .arg("--file")
            .arg(&f)
            .output()
            .unwrap();
        assert!(!o.status.success());
        assert!(o.stdout.is_empty());
    }
    std::fs::remove_dir_all(root).unwrap();
}
