//! Real CLI private transport and verify-before-output behavior.
use serde_json::Value;
use std::{
    path::Path,
    process::{Command, Output},
};
fn run(mode: &str, root: &Path, capture: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cube"));
    command.env_remove("CUBE_PLAY_CAPTURE_DIR");
    if capture {
        command.arg(format!(
            "--lab-capture={}",
            root.join("capture.json").display()
        ));
    }
    command
        .args(["cipher", "stream", "auth", mode, "--parameters-file"])
        .arg(root.join("settings.json"))
        .arg("--file")
        .arg(root.join("input.bits"));
    if mode == "decrypt" {
        command.arg("--tag-file").arg(root.join("tag.bits"));
    }
    command.output().unwrap()
}
#[test]
fn detached_tags_execute_independent_answers_and_reject_without_stdout() {
    let rows: Vec<Value> = serde_json::from_str(include_str!(
        "../../cube-core/src/ciphers/stream/reference/authentication/vectors.json"
    ))
    .unwrap();
    let root =
        std::env::temp_dir().join(format!("cube-authenticated-stream-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    for v in rows {
        std::fs::write(
            root.join("settings.json"),
            serde_json::to_vec(&v["parameters"]).unwrap(),
        )
        .unwrap();
        let plain = cube_core::encoding::hex_decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
        let ciphertext =
            cube_core::encoding::hex_decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
        let mut tag = cube_core::encoding::hex_decode(v["tag_hex"].as_str().unwrap()).unwrap();
        std::fs::write(root.join("input.bits"), &plain).unwrap();
        let encrypted = run("encrypt", &root, false);
        assert!(
            encrypted.status.success(),
            "{}",
            String::from_utf8_lossy(&encrypted.stderr)
        );
        let result: Value = serde_json::from_slice(&encrypted.stdout).unwrap();
        assert_eq!(result["ciphertext_hex"], v["ciphertext_hex"]);
        assert_eq!(result["tag_hex"], v["tag_hex"]);
        std::fs::write(root.join("input.bits"), &ciphertext).unwrap();
        std::fs::write(root.join("tag.bits"), &tag).unwrap();
        let decrypted = run("decrypt", &root, false);
        assert!(
            decrypted.status.success(),
            "{}",
            String::from_utf8_lossy(&decrypted.stderr)
        );
        assert_eq!(decrypted.stdout, plain);
        tag[0] ^= 128;
        std::fs::write(root.join("tag.bits"), &tag).unwrap();
        let forged = run("decrypt", &root, false);
        assert!(!forged.status.success());
        assert!(forged.stdout.is_empty());
        assert!(String::from_utf8_lossy(&forged.stderr).contains("authentication failed"));
    }
    for mode in ["encrypt", "decrypt"] {
        let capture = run(mode, &root, true);
        assert!(!capture.status.success());
        assert!(capture.stdout.is_empty());
        assert!(!root.join("capture.json").exists());
    }
    std::fs::write(
        root.join("settings.json"),
        br#"{"profile":"grain128a_skew2011_msb_v1","key_hex":"SECRET"}"#,
    )
    .unwrap();
    let invalid = run("encrypt", &root, false);
    assert!(!invalid.status.success());
    assert!(invalid.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&invalid.stderr).contains("SECRET"));
    std::fs::remove_dir_all(root).unwrap();
}
