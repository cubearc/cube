use serde_json::Value;
use std::process::Command;
#[test]
fn raw_shared_secrets_use_private_settings_files_and_fail_without_stdout() {
    let dir = std::env::temp_dir().join(format!("cube-dh-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let settings = dir.join("private.json");
    let input = dir.join("peer.bin");
    let key = dir.join("secret.bin");
    let invoke = || {
        Command::new(env!("CARGO_BIN_EXE_cube"))
            .args([
                "--json-errors",
                "cipher",
                "dh",
                "agree",
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
        "../../cube-core/src/asymmetric/dh/vectors.json"
    ))
    .unwrap();
    for v in vectors {
        std::fs::write(&settings, v["parameters"].to_string()).unwrap();
        std::fs::write(&input, hex_bytes(&v, "peer_hex")).unwrap();
        std::fs::write(&key, hex_bytes(&v, "private_key_hex")).unwrap();
        let result = invoke();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(result.stdout, hex_bytes(&v, "shared_hex"));
        std::fs::write(&input, [0u8; 32]).unwrap();
        let result = invoke();
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        assert!(
            !String::from_utf8_lossy(&result.stderr)
                .contains(v["private_key_hex"].as_str().unwrap())
        );
    }
    std::fs::remove_dir_all(dir).unwrap();
}
fn hex_bytes(v: &Value, field: &str) -> Vec<u8> {
    cube_core::encoding::hex_decode(v[field].as_str().unwrap()).unwrap()
}
