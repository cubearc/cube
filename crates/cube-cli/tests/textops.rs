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

#[test]
fn unicode_text_commands_are_explicit_and_bounded() {
    success(
        &["text", "normalize", "--form", "NFC", "--text", "e\u{301}"],
        b"",
        "é\n".as_bytes(),
    );
    success(
        &["text", "reverse"],
        "e\u{301}\r\n".as_bytes(),
        "\n\r\u{301}e\n".as_bytes(),
    );
    success(
        &[
            "text",
            "extract",
            "--indices",
            "5,1,1,2",
            "--text",
            "Aé🧊e\u{301}Z",
        ],
        b"",
        "Zéé🧊\n".as_bytes(),
    );
    success(
        &["text", "extract", "--indices", "", "--text", ""],
        b"",
        b"\n",
    );
    success(
        &[
            "text", "extract", "--start", "1", "--stride", "2", "--text", "abcdef",
        ],
        b"",
        b"bdf\n",
    );
    for args in [
        vec!["text", "normalize", "--form", "other"],
        vec!["text", "extract", "--indices", "-1"],
        vec!["text", "extract", "--stride", "0"],
        vec!["text", "extract", "--indices", "2", "--text", "é"],
    ] {
        let result = run(&args, b"");
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
    }
    for args in [
        vec!["text", "normalize", "--form", "NFC"],
        vec!["text", "reverse"],
        vec!["text", "extract", "--stride", "1"],
    ] {
        let result = run(&args, &[255]);
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
    }
    assert_eq!(
        run(
            &["text", "extract", "--indices", "0", "--stride", "1"],
            b"a"
        )
        .status
        .code(),
        Some(2)
    );
    assert_eq!(run(&["text", "extract"], b"a").status.code(), Some(2));
}
