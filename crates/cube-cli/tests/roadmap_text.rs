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

#[test]
fn structured_statistics_keep_counts_and_qualified_numbers() {
    let result = run(
        &[
            "text",
            "statistics",
            "--ngram-size",
            "2",
            "--text",
            "ABAB",
            "--json",
        ],
        b"",
    );
    assert!(result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["frequencies"][0]["count"], 2);
    assert_eq!(report["shannon_entropy_bits"], 1.0);
    let result = run(
        &[
            "text",
            "statistics",
            "--filter",
            "ascii_letters",
            "--case",
            "ascii_lower",
        ],
        b"A a!",
    );
    assert!(result.status.success());
    assert!(
        String::from_utf8(result.stdout)
            .unwrap()
            .contains("Scalars: 2")
    );
    assert!(
        !run(&["text", "statistics", "--ngram-size", "0"], b"text")
            .status
            .success()
    );
}

#[test]
fn scalar_selection_has_known_acrostics_and_crlf_behavior() {
    let result = run(
        &[
            "text",
            "select",
            "--selector",
            "word_first",
            "--text",
            "Cube Uses Binary Evidence",
        ],
        b"",
    );
    assert!(result.status.success());
    assert_eq!(result.stdout, b"CUBE\n");
    let result = run(
        &["text", "select", "--selector", "line_last"],
        "Alpha!\r\n\n雪🧊\n".as_bytes(),
    );
    assert!(result.status.success());
    assert_eq!(result.stdout, "!🧊\n".as_bytes());
    assert!(
        !run(
            &["text", "select", "--selector", "guess", "--text", "text"],
            b""
        )
        .status
        .success()
    );
}
