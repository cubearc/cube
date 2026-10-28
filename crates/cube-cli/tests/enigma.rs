use serde_json::{Value, json};
use std::process::Command;

#[test]
fn cli_returns_typed_report_and_rejects_unsafe_or_ambiguous_settings() {
    let directory = std::env::temp_dir().join(format!("cube-enigma-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("settings.json");
    let p = json!({"profile":"enigma_i","rotors":["I","II","III"],"reflector":"B","rings":[1,1,1],"windows":"AAA","plugboard":[]});
    std::fs::write(&path, p.to_string()).unwrap();
    let call = |input: &str| {
        Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(["cipher", "enigma", "--parameters"])
            .arg(&path)
            .args(["--text", input])
            .output()
            .unwrap()
    };
    let r = call("AAAAA");
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    assert_eq!(
        serde_json::from_slice::<Value>(&r.stdout).unwrap(),
        json!({"text":"BDZGO","final_windows":"AAF","trace":[]})
    );
    assert!(!call("a").status.success());
    std::fs::write(&path, r#"{"profile":"m3","profile":"enigma_i"}"#).unwrap();
    assert!(!call("AAAAA").status.success());
    std::fs::remove_dir_all(directory).unwrap();
}
