use std::{
    io::Write,
    process::{Command, Stdio},
};
#[test]
fn rfc_stream_cli_requires_explicit_state_and_keeps_secrets_out_of_errors() {
    let root = std::env::temp_dir().join(format!("cube-stream-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let key = root.join("key.bin");
    std::fs::write(&key, (0..32).collect::<Vec<u8>>()).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(["cipher", "chacha20", "--key-file"])
        .arg(&key)
        .args([
            "--nonce-hex",
            "000000090000004a00000000",
            "--counter",
            "1",
            "--stdout",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&[0; 64]).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        cube_core::encoding::hex_encode(&output.stdout),
        "10f1e7e4d13b5915500fdd1fa32071c4c7d1f4c733c068030422aa9ac3d46c4ed2826446079faa0914c2d705d98b02a2b5129cd1de164eb9cbd083e8a2503c4e"
    );
    for args in [
        vec!["cipher", "chacha20", "--key-hex", "SECRET"],
        vec!["cipher", "chacha20", "--nonce-hex", "SECRET"],
        vec![
            "--lab-capture=invalid.json",
            "cipher",
            "vernam",
            "--key-file",
            "secret.bin",
            "--stdout",
        ],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(args)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
    }
    std::fs::remove_dir_all(root).unwrap();
}
