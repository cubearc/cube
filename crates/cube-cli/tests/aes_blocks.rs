use std::process::Command;

#[test]
fn aes_raw_key_file_binary_output_and_no_secret_cli_arguments_or_capture() {
    let directory = std::env::temp_dir().join(format!("cube-aes-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let key = directory.join("key.bin");
    let input = directory.join("blocks.bin");
    std::fs::write(&key, (0u8..16).collect::<Vec<_>>()).unwrap();
    let plain = hex_bytes("00112233445566778899aabbccddeeff");
    std::fs::write(&input, &plain).unwrap();
    let invoke = |mode: &str| {
        Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(["--json-errors", "cipher", "aes", mode, "--key-file"])
            .arg(&key)
            .arg("--file")
            .arg(&input)
            .output()
            .unwrap()
    };
    let encrypted = invoke("encrypt");
    assert!(
        encrypted.status.success(),
        "{}",
        String::from_utf8_lossy(&encrypted.stderr)
    );
    assert_eq!(
        encrypted.stdout,
        hex_bytes("69c4e0d86a7b0430d8cdb78070b4c55a")
    );
    std::fs::write(&input, encrypted.stdout).unwrap();
    assert_eq!(invoke("decrypt").stdout, plain);
    std::fs::write(&key, b"SECRET").unwrap();
    let bad = invoke("encrypt");
    assert!(!bad.status.success());
    assert!(!String::from_utf8_lossy(&bad.stderr).contains("SECRET"));
    let bad: serde_json::Value = serde_json::from_slice(&bad.stderr).unwrap();
    assert_eq!(bad["error"]["code"], "invalid_parameter");
    std::fs::write(&key, (0u8..16).collect::<Vec<_>>()).unwrap();
    for option in ["--key", "--key-hex", "--text"] {
        assert!(
            !Command::new(env!("CARGO_BIN_EXE_cube"))
                .args(["--json-errors", "cipher", "aes", "encrypt", "--key-file"])
                .arg(&key)
                .arg("--file")
                .arg(&input)
                .args([option, "SECRET"])
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    let capture = directory.join("capture.json");
    assert!(
        !Command::new(env!("CARGO_BIN_EXE_cube"))
            .arg(format!("--lab-capture={}", capture.display()))
            .args(["cipher", "aes", "encrypt", "--key-file"])
            .arg(&key)
            .arg("--file")
            .arg(&input)
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(!capture.exists());
    std::fs::remove_dir_all(directory).unwrap();
}
fn hex_bytes(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
