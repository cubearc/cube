use std::{
    io::Write,
    process::{Command, Stdio},
};
#[test]
fn private_settings_files_execute_independent_stream_answers() {
    let rows: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../cube-core/src/ciphers/stream/reference/vectors.json"
    ))
    .unwrap();
    let root = std::env::temp_dir().join(format!("cube-stream-reference-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let settings = root.join("settings.json");
    for v in rows {
        std::fs::write(&settings, serde_json::to_vec(&v["parameters"]).unwrap()).unwrap();
        for (mode, input, expected) in [
            ("encrypt", "plaintext_hex", "ciphertext_hex"),
            ("decrypt", "ciphertext_hex", "plaintext_hex"),
        ] {
            let input = cube_core::encoding::hex_decode(v[input].as_str().unwrap()).unwrap();
            let expected = cube_core::encoding::hex_decode(v[expected].as_str().unwrap()).unwrap();
            let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
                .args(["cipher", "stream", mode, "--parameters-file"])
                .arg(&settings)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            child.stdin.take().unwrap().write_all(&input).unwrap();
            let actual = child.wait_with_output().unwrap();
            assert!(
                actual.status.success(),
                "{}",
                String::from_utf8_lossy(&actual.stderr)
            );
            assert_eq!(actual.stdout, expected);
        }
    }
    std::fs::write(
        &settings,
        br#"{"profile":"rc4_rfc6229_v1","key_hex":"SECRET","nonce_hex":""}"#,
    )
    .unwrap();
    let actual = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(["cipher", "stream", "encrypt", "--parameters-file"])
        .arg(&settings)
        .output()
        .unwrap();
    assert!(!actual.status.success());
    assert!(actual.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&actual.stderr).contains("SECRET"));
    std::fs::remove_dir_all(root).unwrap();
}
