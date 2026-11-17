use cube_core::encoding::base64_decode;
use serde_json::Value;
use std::process::Command;

fn raw(result: &Value, port: &str) -> Vec<u8> {
    base64_decode(result[port]["base64"].as_str().unwrap()).unwrap()
}

#[test]
fn ml_kem_cli_derives_the_same_secret_from_both_ends() {
    let root = std::env::temp_dir().join(format!("cube-ml-kem-{}", std::process::id()));
    let mut directory = std::fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        directory.mode(0o700);
    }
    directory.create(&root).unwrap();
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Scratch(root.clone());
    std::fs::write(
        root.join("settings.json"),
        r#"{"profile":"ml_kem_768_fips203"}"#,
    )
    .unwrap();
    let invoke = |verb, options: &[&str]| {
        let result = Command::new(env!("CARGO_BIN_EXE_cube"))
            .current_dir(&root)
            .args([
                "cipher",
                "ml-kem",
                verb,
                "--parameters-file",
                "settings.json",
            ])
            .args(options)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "ML-KEM {verb} failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        serde_json::from_slice::<Value>(&result.stdout).unwrap()
    };
    let pair = invoke("keygen", &[]);
    std::fs::write(root.join("public.bin"), raw(&pair, "public_key")).unwrap();
    std::fs::write(root.join("private.bin"), raw(&pair, "private_key")).unwrap();
    let envelope = invoke("encapsulate", &["--key-file", "public.bin"]);
    std::fs::write(root.join("ciphertext.bin"), raw(&envelope, "ciphertext")).unwrap();
    let recovered = invoke(
        "decapsulate",
        &["--key-file", "private.bin", "--file", "ciphertext.bin"],
    );
    assert!(raw(&envelope, "shared_secret") == raw(&recovered, "shared_secret"));
}
