use serde_json::Value;
use std::{path::PathBuf, process::Command};
struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn hex_bytes(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
#[test]
fn reference_blocks_use_raw_key_files_binary_stdout_and_explicit_settings() {
    let vectors: Vec<Value> = serde_json::from_str(include_str!(
        "../../cube-core/src/ciphers/block_profiles/vectors.json"
    ))
    .unwrap();
    let directory =
        Temporary(std::env::temp_dir().join(format!("cube-block-profiles-{}", std::process::id())));
    std::fs::create_dir(&directory.0).unwrap();
    let key = directory.0.join("key.bin");
    let input = directory.0.join("input.bin");
    for v in vectors {
        std::fs::write(&key, hex_bytes(v["key_hex"].as_str().unwrap())).unwrap();
        for (mode, source, expected) in [
            ("encrypt", &v["plaintext_hex"], &v["ciphertext_hex"]),
            ("decrypt", &v["ciphertext_hex"], &v["plaintext_hex"]),
        ] {
            std::fs::write(&input, hex_bytes(source.as_str().unwrap())).unwrap();
            let mut command = Command::new(env!("CARGO_BIN_EXE_cube"));
            command
                .args([
                    "--json-errors",
                    "cipher",
                    "block",
                    mode,
                    "--profile",
                    v["profile"].as_str().unwrap(),
                    "--key-file",
                ])
                .arg(&key)
                .arg("--file")
                .arg(&input);
            if let Some(bits) = v["effective_key_bits"].as_u64() {
                command.arg("--effective-key-bits").arg(bits.to_string());
            }
            let result = command.output().unwrap();
            assert!(
                result.status.success(),
                "{} {mode}: {}",
                v["profile"],
                String::from_utf8_lossy(&result.stderr)
            );
            assert_eq!(result.stdout, hex_bytes(expected.as_str().unwrap()));
        }
    }
    std::fs::write(&input, [0; 8]).unwrap();
    std::fs::write(&key, b"SECRET").unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_cube"));
    command
        .args([
            "--json-errors",
            "cipher",
            "block",
            "encrypt",
            "--profile",
            "des_fips46_v1",
            "--key-file",
        ])
        .arg(&key)
        .arg("--file")
        .arg(&input);
    let result = command.output().unwrap();
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&result.stderr).contains("SECRET"));
    let error: Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(error["error"]["code"], "invalid_parameter");
    std::fs::write(&key, [0; 8]).unwrap();
    for option in ["--key", "--key-hex", "--text"] {
        assert!(
            !Command::new(env!("CARGO_BIN_EXE_cube"))
                .args([
                    "--json-errors",
                    "cipher",
                    "block",
                    "encrypt",
                    "--profile",
                    "des_fips46_v1",
                    "--key-file"
                ])
                .arg(&key)
                .arg("--file")
                .arg(&input)
                .args([option, "SECRET"])
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    let capture = directory.0.join("capture.json");
    assert!(
        !Command::new(env!("CARGO_BIN_EXE_cube"))
            .arg(format!("--lab-capture={}", capture.display()))
            .args([
                "cipher",
                "block",
                "encrypt",
                "--profile",
                "des_fips46_v1",
                "--key-file"
            ])
            .arg(&key)
            .arg("--file")
            .arg(&input)
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(!capture.exists());
}
