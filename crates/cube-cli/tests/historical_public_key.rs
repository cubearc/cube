use serde_json::Value;
use std::process::Command;
#[test]
fn exact_reference_token_arrays_and_root_sets_use_real_cli_without_partial_errors() {
    let dir = std::env::temp_dir().join(format!("cube-historical-pk-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let params = dir.join("settings.json");
    let key = dir.join("key.json");
    let input = dir.join("tokens.json");
    let invoke = |family: &str, action: &str| {
        Command::new(env!("CARGO_BIN_EXE_cube"))
            .args([
                "--json-errors",
                "cipher",
                family,
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
        "../../cube-core/src/asymmetric/historical_public_key/vectors.json"
    ))
    .unwrap();
    for v in vectors {
        let family = v["scheme"].as_str().unwrap().replace('_', "-");
        std::fs::write(&params, v["parameters"].to_string()).unwrap();
        std::fs::write(&key, v["private_key"].to_string()).unwrap();
        std::fs::write(&input, v["ciphertext_tokens"].to_string()).unwrap();
        let result = invoke(&family, "decrypt");
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let expected = v.get("candidate_tokens").unwrap_or(&v["tokens"]);
        assert_eq!(
            &serde_json::from_slice::<Value>(&result.stdout).unwrap(),
            expected
        );
        std::fs::write(&key, v["public_key"].to_string()).unwrap();
        std::fs::write(&input, v["tokens"].to_string()).unwrap();
        let fresh = invoke(&family, "encrypt");
        assert!(
            fresh.status.success(),
            "{}",
            String::from_utf8_lossy(&fresh.stderr)
        );
        std::fs::write(&key, v["private_key"].to_string()).unwrap();
        std::fs::write(&input, fresh.stdout).unwrap();
        assert_eq!(
            &serde_json::from_slice::<Value>(&invoke(&family, "decrypt").stdout).unwrap(),
            expected
        );
        let mut bad = v["parameters"].clone();
        bad["max_work"] = serde_json::json!(1);
        std::fs::write(&params, bad.to_string()).unwrap();
        let result = invoke(&family, "decrypt");
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
    }
    std::fs::remove_dir_all(dir).unwrap();
}
