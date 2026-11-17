use serde_json::{Value, json};
use std::process::Command;
fn bytes(v: &Value, name: &str) -> Vec<u8> {
    cube_core::encoding::hex_decode(v[name].as_str().unwrap()).unwrap()
}
#[test]
fn exact_aead_and_siv_files_detached_tags_and_no_failed_plaintext() {
    let directory = std::env::temp_dir().join(format!("cube-aead-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let settings = directory.join("private.json");
    let input = directory.join("input.bin");
    let context = directory.join("context.bin");
    let tag = directory.join("tag.bin");
    let invoke = |siv: bool, inverse: bool| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_cube"));
        command.args(["--json-errors", "cipher", "aead"]);
        if siv {
            command.arg("siv");
        }
        command
            .arg(if inverse { "decrypt" } else { "encrypt" })
            .arg("--parameters-file")
            .arg(&settings)
            .arg("--file")
            .arg(&input)
            .arg(if siv {
                "--aad-components-file"
            } else {
                "--aad-file"
            })
            .arg(&context);
        if inverse {
            command.arg("--tag-file").arg(&tag);
        }
        command.output().unwrap()
    };
    for siv in [false, true] {
        let vectors: Vec<Value> = serde_json::from_str(if siv {
            include_str!("../../cube-core/src/ciphers/modern_aead/siv_vectors.json")
        } else {
            include_str!("../../cube-core/src/ciphers/modern_aead/vectors.json")
        })
        .unwrap();
        for v in vectors {
            std::fs::write(&settings, v["parameters"].to_string()).unwrap();
            let plain = bytes(&v, "plaintext_hex");
            let ciphertext = bytes(&v, "ciphertext_hex");
            let expected_tag = bytes(&v, "tag_hex");
            if siv {
                let components: Vec<_> = v["aad_components_hex"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| cube_core::encoding::hex_decode(v.as_str().unwrap()).unwrap())
                    .collect();
                std::fs::write(&context, serde_json::to_vec(&components).unwrap()).unwrap();
            } else {
                std::fs::write(&context, bytes(&v, "aad_hex")).unwrap();
            }
            std::fs::write(&input, &plain).unwrap();
            let result = invoke(siv, false);
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            let result: Value = serde_json::from_slice(&result.stdout).unwrap();
            assert_eq!(
                result,
                json!({"ciphertext_hex":v["ciphertext_hex"],"tag_hex":v["tag_hex"]})
            );
            std::fs::write(&input, &ciphertext).unwrap();
            std::fs::write(&tag, &expected_tag).unwrap();
            let result = invoke(siv, true);
            assert!(result.status.success());
            assert_eq!(result.stdout, plain);
            let mut bad = expected_tag;
            bad[0] ^= 1;
            std::fs::write(&tag, bad).unwrap();
            let result = invoke(siv, true);
            assert!(!result.status.success());
            assert!(result.stdout.is_empty());
            let error: Value = serde_json::from_slice(&result.stderr).unwrap();
            assert_eq!(error["error"]["code"], "invalid_input");
        }
    }
    std::fs::write(&settings, b"{\"key_hex\":\"SECRET\"}").unwrap();
    let result = invoke(false, false);
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&result.stderr).contains("SECRET"));
    std::fs::remove_dir_all(directory).unwrap();
}
