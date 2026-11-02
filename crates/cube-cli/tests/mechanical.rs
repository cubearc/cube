use serde_json::{Value, json};
use std::process::Command;
#[test]
fn m94_cli_uses_explicit_rows_and_bounded_typed_settings() {
    let dir = std::env::temp_dir().join(format!("cube-mechanical-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("settings.json");
    let p = json!({"profile":"m94_v1","order":[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25],"row_offsets":[1]});
    std::fs::write(&path, p.to_string()).unwrap();
    let call = |direction: &str, text: &str| {
        Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(["cipher", "m94", direction, "--parameters"])
            .arg(&path)
            .args(["--text", text])
            .output()
            .unwrap()
    };
    let r = call("encode", "AAAAA");
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    assert_eq!(
        serde_json::from_slice::<Value>(&r.stdout).unwrap(),
        json!({"text":"BCDEF"})
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&call("decode", "BCDEF").stdout).unwrap(),
        json!({"text":"AAAAA"})
    );
    assert!(!call("encode", "a").status.success());
    std::fs::write(&path, r#"{"profile":"m94_v1","profile":"m94_v1"}"#).unwrap();
    assert!(!call("encode", "AAAAA").status.success());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn every_mechanical_fixture_executes_through_the_real_cli() {
    let fixtures: Vec<Value> = serde_json::from_str(include_str!(
        "../../cube-core/src/operations/contracts/conformance.json"
    ))
    .unwrap();
    let dir = std::env::temp_dir().join(format!("cube-mechanical-matrix-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("settings.json");
    let mut tested = 0;
    for row in fixtures {
        let op = row["operation"].as_str().unwrap();
        let parts = op.split('.').collect::<Vec<_>>();
        if parts.len() != 3
            || ![
                "m94",
                "jefferson",
                "m138a",
                "wheatstone",
                "kryha",
                "alberti",
                "lorenz",
                "t52ab",
                "nema",
                "typex",
                "sigaba",
                "kl7",
                "hagelin",
            ]
            .contains(&parts[1])
        {
            continue;
        }
        std::fs::write(&path, row["inputs"]["settings"].to_string()).unwrap();
        let input = if matches!(parts[1], "lorenz" | "t52ab") {
            row["inputs"]["codes"].to_string()
        } else {
            row["inputs"]["text"].as_str().unwrap().to_owned()
        };
        let result = Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(["cipher", parts[1], parts[2], "--parameters"])
            .arg(&path)
            .args(["--text", &input])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{op}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&result.stdout).unwrap(),
            row["outputs"],
            "{op}"
        );
        tested += 1;
    }
    assert_eq!(tested, 26);
    std::fs::remove_dir_all(dir).unwrap();
}
