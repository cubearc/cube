//! Exact segmented arrays pass through the CLI without a second mapping.
use serde_json::{Value, json};
use std::process::Command;
#[test]
fn every_dated_alphabet_table_executes_in_both_cli_directions() {
    let rows: Vec<Value> = serde_json::from_str(include_str!(
        "../../cube-core/src/symbols/alphabet/vectors.json"
    ))
    .unwrap();
    let directory = std::env::temp_dir().join(format!("cube-alphabet-cli-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let parameters = directory.join("parameters.json");
    for row in rows {
        std::fs::write(
            &parameters,
            serde_json::to_vec(&json!({"profile":row["profile"]})).unwrap(),
        )
        .unwrap();
        for (mode, field) in [("encode", "units"), ("decode", "tokens")] {
            let output = Command::new(env!("CARGO_BIN_EXE_cube"))
                .args([mode, "alphabet", "--parameters"])
                .arg(&parameters)
                .args(["--text", &row[field].to_string()])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{} {mode}: {}",
                row["profile"],
                String::from_utf8_lossy(&output.stderr)
            );
            let actual: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(actual["tokens"], row["tokens"]);
            if mode == "decode" {
                for (index, unit) in row["units"].as_array().unwrap().iter().enumerate() {
                    assert_eq!(
                        actual["readings"][index],
                        json!({"token_index":index,"alternatives":row.get("alternatives").and_then(|a|a.get(index)).cloned().unwrap_or_else(||json!([unit]))})
                    );
                }
            }
        }
    }
    std::fs::remove_dir_all(directory).unwrap();
}
