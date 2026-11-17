use serde_json::Value;
use std::process::Command;
#[test]
fn ies_private_file_transport_matches_external_vectors_and_refuses_tampering_without_stdout() {
    let dir = std::env::temp_dir().join(format!("cube-ies-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let key = dir.join("key.bin");
    let params = dir.join("parameters.json");
    let input = dir.join("input.bin");
    let run = |action: &str| {
        Command::new(env!("CARGO_BIN_EXE_cube"))
            .args([
                "--json-errors",
                "cipher",
                "ies",
                action,
                "--parameters-file",
            ])
            .arg(&params)
            .arg("--key-file")
            .arg(&key)
            .arg("--file")
            .arg(&input)
            .output()
            .unwrap()
    };
    let vectors: Vec<Value> = serde_json::from_str(include_str!(
        "../../cube-core/src/asymmetric/ies/interoperability.json"
    ))
    .unwrap();
    for v in vectors {
        std::fs::write(&params, v["parameters"].to_string()).unwrap();
        let private =
            cube_core::encoding::hex_decode(v["private_key_hex"].as_str().unwrap()).unwrap();
        let public =
            cube_core::encoding::hex_decode(v["public_key_hex"].as_str().unwrap()).unwrap();
        let plain = cube_core::encoding::hex_decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
        let mut cipher =
            cube_core::encoding::hex_decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
        std::fs::write(&key, &private).unwrap();
        std::fs::write(&input, &cipher).unwrap();
        let result = run("decrypt");
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(result.stdout, plain);
        *cipher.last_mut().unwrap() ^= 1;
        std::fs::write(&input, &cipher).unwrap();
        let bad = run("decrypt");
        assert!(!bad.status.success());
        assert!(bad.stdout.is_empty());
        std::fs::write(&key, &public).unwrap();
        std::fs::write(&input, &plain).unwrap();
        let result = run("encrypt");
        assert!(result.status.success());
        std::fs::write(&key, &private).unwrap();
        std::fs::write(&input, &result.stdout).unwrap();
        assert_eq!(run("decrypt").stdout, plain);
    }
    std::fs::remove_dir_all(dir).unwrap();
}
