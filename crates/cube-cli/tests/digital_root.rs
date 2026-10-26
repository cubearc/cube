use std::io::Write;
use std::process::{Command, Stdio};
#[test]
fn decimal_text_signs_limits_json_and_strict_stdin() {
    for (input, expected) in [
        ("0", 0),
        ("9", 9),
        ("10", 1),
        ("38", 2),
        ("999", 9),
        ("-38", 2),
        ("+00038", 2),
        ("-340282366920938463463374607431768211455", 3),
    ] {
        let r = Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(["number", "digital-root", "--json"])
            .arg(format!("--text={input}"))
            .output()
            .unwrap();
        assert!(r.status.success(), "{:?}", r.stderr);
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&r.stdout).unwrap(),
            serde_json::json!({"value":expected})
        );
    }
    for (input, code) in [
        ("1.0", "invalid_input"),
        ("0x10", "invalid_input"),
        ("1\n", "invalid_input"),
        ("340282366920938463463374607431768211456", "resource_limit"),
    ] {
        let r = Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(["--json-errors", "number", "digital-root", "--text", input])
            .output()
            .unwrap();
        assert!(!r.status.success());
        assert!(r.stdout.is_empty());
        let e: serde_json::Value = serde_json::from_slice(&r.stderr).unwrap();
        assert_eq!(e["error"]["operation"], "number.digital_root");
        assert_eq!(e["error"]["code"], code);
    }
    let r = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "--json-errors",
            "number",
            "digital-root",
            "--text",
            "38",
            "--radix",
            "2",
        ])
        .output()
        .unwrap();
    assert!(!r.status.success());
    let e: serde_json::Value = serde_json::from_slice(&r.stderr).unwrap();
    assert_eq!(e["error"]["parameter"], "radix");
    for (input, success) in [
        ("0".repeat(4096), true),
        ("0".repeat(4097), false),
        ("0".repeat(4098), false),
        ("38\n".into(), false),
    ] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(["--json-errors", "number", "digital-root"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let r = child.wait_with_output().unwrap();
        assert_eq!(r.status.success(), success);
    }
}
