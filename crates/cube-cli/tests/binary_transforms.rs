use std::{
    io::Write,
    process::{Command, Stdio},
};
fn run(args: &[&str], data: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(data).unwrap();
    child.wait_with_output().unwrap()
}
#[test]
fn new_transforms_preserve_raw_bytes_and_hex_conventions() {
    for (args, input, expected) in [
        (
            vec!["binary", "and", "--key-hex", "ff0f"],
            vec![0, 255],
            b"000f\n".to_vec(),
        ),
        (
            vec!["binary", "or", "--key-hex", "00ff", "--output", "raw"],
            vec![128, 0],
            vec![128, 255],
        ),
        (
            vec!["binary", "not", "--output", "raw"],
            vec![0, 255],
            vec![255, 0],
        ),
        (
            vec!["binary", "reverse", "--text", "AB"],
            vec![],
            b"4241\n".to_vec(),
        ),
        (
            vec![
                "binary",
                "shift-left",
                "--bits",
                "1",
                "--unit-bytes",
                "2",
                "--output",
                "raw",
            ],
            vec![1, 255],
            vec![3, 254],
        ),
        (
            vec!["binary", "shift-right", "--bits", "1", "--unit-bytes", "2"],
            vec![1, 0],
            b"0080\n".to_vec(),
        ),
        (
            vec![
                "binary",
                "swap-endian",
                "--unit-bytes",
                "2",
                "--text",
                "ABCD",
            ],
            vec![],
            b"42414443\n".to_vec(),
        ),
    ] {
        let result = run(&args, &input);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(result.stdout, expected);
    }
    for args in [
        vec!["binary", "and", "--key-hex", "00", "--text", "AB"],
        vec!["binary", "shift-left", "--bits", "9", "--text", "A"],
        vec!["binary", "swap-endian", "--unit-bytes", "2", "--text", "A"],
    ] {
        let result = run(&args, b"");
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
    }
}
