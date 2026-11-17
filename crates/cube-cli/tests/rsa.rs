use serde_json::Value;
use std::process::Command;
#[test]
fn rsa_key_files_preserve_independent_bytes_and_never_write_failed_oaep() {
    let dir = std::env::temp_dir().join(format!("cube-rsa-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let settings = dir.join("settings.json");
    let input = dir.join("input.bin");
    let key = dir.join("key.bin");
    let invoke = |scheme: &str, direction: &str| {
        Command::new(env!("CARGO_BIN_EXE_cube"))
            .args([
                "--json-errors",
                "cipher",
                "rsa",
                scheme,
                direction,
                "--parameters-file",
            ])
            .arg(&settings)
            .arg("--key-file")
            .arg(&key)
            .arg("--file")
            .arg(&input)
            .output()
            .unwrap()
    };
    let vectors: Vec<Value> = serde_json::from_str(include_str!(
        "../../cube-core/src/asymmetric/rsa/interoperability.json"
    ))
    .unwrap();
    for v in vectors {
        let plain = bytes(&v, "plaintext_hex");
        let ct = bytes(&v, "ciphertext_hex");
        std::fs::write(&settings, v["private_parameters"].to_string()).unwrap();
        std::fs::write(&key, bytes(&v, "private_key_hex")).unwrap();
        std::fs::write(&input, &ct).unwrap();
        let result = invoke("oaep", "decrypt");
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(result.stdout, plain);
        std::fs::write(&settings, v["public_parameters"].to_string()).unwrap();
        std::fs::write(&key, bytes(&v, "public_key_hex")).unwrap();
        std::fs::write(&input, &plain).unwrap();
        let a = invoke("oaep", "encrypt");
        let b = invoke("oaep", "encrypt");
        assert!(a.status.success());
        assert!(b.status.success());
        assert_ne!(a.stdout, b.stdout);
        std::fs::write(&settings, v["private_parameters"].to_string()).unwrap();
        std::fs::write(&key, bytes(&v, "private_key_hex")).unwrap();
        std::fs::write(&input, &a.stdout).unwrap();
        let result = invoke("oaep", "decrypt");
        assert!(result.status.success());
        assert_eq!(result.stdout, plain);
        let mut bad = ct;
        let n = bad.len();
        bad[n - 1] ^= 1;
        std::fs::write(&input, bad).unwrap();
        let failure = invoke("oaep", "decrypt");
        assert!(!failure.status.success());
        assert!(failure.stdout.is_empty());
        assert!(
            !String::from_utf8_lossy(&failure.stderr)
                .contains(v["private_key_hex"].as_str().unwrap())
        );
    }
    let vectors: Value = serde_json::from_str(include_str!(
        "../../cube-core/src/asymmetric/rsa/implicit_rejection_vectors.json"
    ))
    .unwrap();
    for group in vectors["groups"].as_array().unwrap() {
        std::fs::write(&settings, group["parameters"].to_string()).unwrap();
        std::fs::write(&key, bytes(group, "key_hex")).unwrap();
        for v in group["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v["expect"] != "public_error")
        {
            std::fs::write(&input, bytes(v, "ciphertext_hex")).unwrap();
            let result = invoke("pkcs1-v1-5", "decrypt");
            assert!(result.status.success());
            assert_eq!(result.stdout, bytes(v, "output_hex"));
        }
    }
    std::fs::remove_dir_all(dir).unwrap();
}
fn bytes(v: &Value, name: &str) -> Vec<u8> {
    cube_core::encoding::hex_decode(v[name].as_str().unwrap()).unwrap()
}
