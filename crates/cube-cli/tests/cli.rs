use std::io::{self, Write};
use std::process::{Command, Output, Stdio};

fn cube(args: &[&str], stdin: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start cube");
    let write_result = child.stdin.take().expect("piped stdin").write_all(stdin);
    // Commands using --text or rejecting arguments may exit without reading stdin.
    if let Err(error) = write_result {
        assert_eq!(
            error.kind(),
            io::ErrorKind::BrokenPipe,
            "write stdin: {error}"
        );
    }
    child.wait_with_output().expect("wait for cube")
}

fn success(args: &[&str], stdin: &[u8], expected: &[u8]) {
    let output = cube(args, stdin);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, expected);
    assert!(output.stderr.is_empty());
}

#[test]
fn help_version_and_discovery() {
    for args in [
        vec!["--help"],
        vec!["decode", "base64", "--help"],
        vec!["binary", "xor", "--help"],
    ] {
        let output = cube(&args, b"");
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
        assert!(output.stderr.is_empty());
    }
    success(
        &["--version"],
        b"",
        format!("cube {}\n", env!("CARGO_PKG_VERSION")).as_bytes(),
    );
    success(&["ops", "list"], b"", b"encode base64\nencode hex\ndecode base64\ndecode hex\ncipher rot13\nhash sha256\nbinary xor\n");
}

#[test]
fn text_commands() {
    success(
        &["encode", "base64", "--text", "Hello"],
        b"ignored",
        b"SGVsbG8=\n",
    );
    success(&["encode", "hex", "--text", "Hello"], b"", b"48656c6c6f\n");
    success(
        &["decode", "base64", "--text", "SGVsbG8="],
        b"",
        b"48656c6c6f\n",
    );
    success(
        &["decode", "hex", "--text", "48656C6c6f", "--output", "raw"],
        b"",
        b"Hello",
    );
    success(
        &["cipher", "rot13", "--text", "Olá 世界"],
        b"",
        "Byá 世界\n".as_bytes(),
    );
    success(
        &["hash", "sha256", "--text", "Hello"],
        b"",
        b"185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969\n",
    );
    success(
        &["binary", "xor", "--text", "Hello", "--key", "key"],
        b"",
        b"230015070a\n",
    );
    success(
        &[
            "binary",
            "xor",
            "--text",
            "Hello",
            "--key-hex",
            "6b6579",
            "--output",
            "raw",
        ],
        b"",
        &[35, 0, 21, 7, 10],
    );
}

#[test]
fn stdin_binary_and_newlines_are_preserved() {
    success(&["encode", "base64"], &[0, 255], b"AP8=\n");
    success(&["encode", "hex"], b"Hello\n", b"48656c6c6f0a\n");
    success(&["decode", "base64", "--output", "raw"], b"AP8=", &[0, 255]);
    success(&["decode", "hex"], b"00FF", b"00ff\n");
    success(
        &["binary", "xor", "--key-hex", "ff", "--output", "raw"],
        &[0, 255],
        &[255, 0],
    );
    success(
        &["cipher", "rot13"],
        "Hello 🧊".as_bytes(),
        "Uryyb 🧊\n".as_bytes(),
    );
    success(
        &["hash", "sha256"],
        b"",
        b"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n",
    );
}

#[test]
fn empty_inputs_are_explicit_and_valid() {
    for args in [
        vec!["encode", "base64", "--text", ""],
        vec!["encode", "hex", "--text", ""],
        vec!["decode", "base64", "--text", ""],
        vec!["cipher", "rot13", "--text", ""],
        vec!["binary", "xor", "--text", "", "--key", "k"],
    ] {
        success(&args, b"must not read stdin", b"\n");
    }
    success(
        &["decode", "hex", "--text", "", "--output", "raw"],
        b"",
        b"",
    );
}

#[test]
fn invalid_operation_input_uses_stderr_and_exit_one() {
    for (args, input, message) in [
        (
            vec!["decode", "base64", "--text", "!"],
            &b""[..],
            "invalid Base64",
        ),
        (vec!["decode", "base64"], &b"Zg==\n"[..], "invalid Base64"),
        (
            vec!["decode", "hex", "--text", "a"],
            &b""[..],
            "invalid hexadecimal",
        ),
        (
            vec!["binary", "xor", "--key", ""],
            &b""[..],
            "XOR key must not be empty",
        ),
        (
            vec!["binary", "xor", "--key-hex", ""],
            &b""[..],
            "XOR key must not be empty",
        ),
        (
            vec!["binary", "xor", "--key-hex", "zz"],
            &b""[..],
            "invalid hexadecimal",
        ),
        (vec!["cipher", "rot13"], &[255][..], "utf-8"),
        (vec!["decode", "base64"], &[255][..], "utf-8"),
    ] {
        let output = cube(&args, input);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains(message));
    }
}

#[test]
fn invalid_cli_usage_exits_two() {
    for args in [
        vec![],
        vec!["unknown"],
        vec!["binary", "xor"],
        vec!["binary", "xor", "--key", "k", "--key-hex", "ff"],
        vec!["decode", "base64", "--output", "text"],
    ] {
        let output = cube(&args, b"");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}
