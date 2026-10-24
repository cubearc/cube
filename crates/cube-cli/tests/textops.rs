use std::{
    io::Write,
    process::{Command, Output, Stdio},
};
fn run(args: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(input);
    child.wait_with_output().unwrap()
}
fn success(args: &[&str], input: &[u8], expected: &[u8]) {
    let result = run(args, input);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(result.stdout, expected);
    assert!(result.stderr.is_empty());
}
#[test]
fn classical_commands_use_shared_rules_and_errors() {
    success(
        &["cipher", "atbash", "--text", "ABC xyz"],
        b"",
        b"ZYX cba\n",
    );
    success(
        &[
            "cipher",
            "vigenere",
            "--key",
            "LEMON",
            "--text",
            "ATTACKATDAWN",
        ],
        b"",
        b"LXFOPVEFRNHR\n",
    );
    success(
        &["cipher", "vigenere", "--key", "LEMON", "--decode"],
        b"LXFOPVEFRNHR",
        b"ATTACKATDAWN\n",
    );
    success(
        &["cipher", "beaufort", "--key", "B", "--text", "ABC xyz"],
        b"",
        b"BAZ edc\n",
    );
    success(
        &[
            "cipher",
            "affine",
            "--a",
            "5",
            "--b",
            "8",
            "--text",
            "AFFINECIPHER",
        ],
        b"",
        b"IHHWVCSWFRCP\n",
    );
    success(
        &["cipher", "affine", "--decode", "--a", "5", "--b", "8"],
        b"IHHWVCSWFRCP",
        b"AFFINECIPHER\n",
    );
    success(
        &["cipher", "atbash"],
        "é🧊\0\r\n".as_bytes(),
        "é🧊\0\r\n\n".as_bytes(),
    );
    for (args, input, code) in [
        (vec!["cipher", "atbash"], vec![255], "invalid_input"),
        (
            vec!["cipher", "vigenere", "--key", ""],
            vec![],
            "invalid_parameter",
        ),
        (
            vec!["cipher", "affine", "--a", "13", "--b", "0"],
            vec![],
            "invalid_parameter",
        ),
        (
            vec!["cipher", "atbash"],
            vec![b'a'; 1024 * 1024 + 1],
            "resource_limit",
        ),
    ] {
        let mut arguments = vec!["--json-errors"];
        arguments.extend(args);
        let result = run(&arguments, &input);
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        let error: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
        assert_eq!(error["error"]["code"], code);
    }
}
