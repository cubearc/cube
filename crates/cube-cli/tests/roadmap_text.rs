use std::{
    io::Write,
    process::{Command, Stdio},
};
fn run(args: &[&str], input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}
#[test]
fn a1z26_uses_text_outputs_stdin_and_explicit_errors() {
    assert_eq!(
        run(&["encode", "a1z26", "--text", "Hi Cube"], b"").stdout,
        b"8-9 3-21-2-5\n"
    );
    assert_eq!(run(&["decode", "a1z26"], b"1-26\r\n").stdout, b"AZ\r\n\n");
    let result = run(&["--json-errors", "decode", "a1z26", "--text", "1--2"], b"");
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    let _: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
}
