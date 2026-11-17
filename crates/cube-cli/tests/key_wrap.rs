use serde_json::Value;
use std::process::Command;
#[test]
fn binary_wrap_and_unwrap_never_write_failed_key_data() {
    let dir = std::env::temp_dir().join(format!("cube-key-wrap-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let settings = dir.join("private.json");
    let input = dir.join("key.bin");
    let invoke = |inverse: bool| {
        Command::new(env!("CARGO_BIN_EXE_cube"))
            .args([
                "--json-errors",
                "cipher",
                "aes",
                "key-wrap",
                if inverse { "unwrap" } else { "wrap" },
                "--parameters-file",
            ])
            .arg(&settings)
            .arg("--file")
            .arg(&input)
            .output()
            .unwrap()
    };
    for invalid in [false, true] {
        let vectors: Vec<Value> = serde_json::from_str(if invalid {
            include_str!("../../cube-core/src/ciphers/key_wrap/invalid_vectors.json")
        } else {
            include_str!("../../cube-core/src/ciphers/key_wrap/vectors.json")
        })
        .unwrap();
        for v in vectors {
            std::fs::write(&settings, v["parameters"].to_string()).unwrap();
            let wrapped =
                cube_core::encoding::hex_decode(v["wrapped_hex"].as_str().unwrap()).unwrap();
            if !invalid {
                let plain =
                    cube_core::encoding::hex_decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
                std::fs::write(&input, &plain).unwrap();
                let result = invoke(false);
                assert!(
                    result.status.success(),
                    "{}",
                    String::from_utf8_lossy(&result.stderr)
                );
                assert_eq!(result.stdout, wrapped);
                std::fs::write(&input, &wrapped).unwrap();
                let result = invoke(true);
                assert!(result.status.success());
                assert_eq!(result.stdout, plain);
            } else {
                std::fs::write(&input, &wrapped).unwrap();
                let result = invoke(true);
                assert!(!result.status.success());
                assert!(result.stdout.is_empty());
            }
        }
    }
    std::fs::remove_dir_all(dir).unwrap();
}
