use std::process::{Command, Output};
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn exact_decimal_commands_and_json_do_not_round_full_width_integers() {
    for (args, expected) in [
        (vec!["gcd", "--a", "48", "--b", "18"], "6\n"),
        (vec!["mod-inverse", "--a", "3", "--modulus", "11"], "4\n"),
        (
            vec![
                "mod-pow",
                "--base",
                "4",
                "--exponent",
                "13",
                "--modulus",
                "497",
            ],
            "445\n",
        ),
        (
            vec!["gcd", "--a", "00000000000000000000", "--b", "0"],
            "0\n",
        ),
    ] {
        let mut full = vec!["number"];
        full.extend(args);
        let r = run(&full);
        assert!(r.status.success(), "{:?}", r.stderr);
        assert_eq!(r.stdout, expected.as_bytes());
    }
    let r = run(&[
        "number",
        "extended-gcd",
        "--a",
        "240",
        "--b",
        "46",
        "--json",
    ]);
    assert!(r.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&r.stdout).unwrap(),
        serde_json::json!({"gcd":"2","x":"-9","y":"47"})
    );
    let m = u64::MAX.to_string();
    let r = run(&["number", "gcd", "--a", &m, "--b", "0", "--json"]);
    assert!(r.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&r.stdout).unwrap(),
        serde_json::json!({"value":m})
    );
}
#[test]
fn rejected_decimal_inputs_and_moduli_keep_context_and_no_stdout() {
    for (value, code) in [
        ("-1", "invalid_parameter"),
        ("+1", "invalid_parameter"),
        ("1.0", "invalid_parameter"),
        (" 1", "invalid_parameter"),
        ("0x10", "invalid_parameter"),
        ("18446744073709551616", "resource_limit"),
        ("000000000000000000000", "resource_limit"),
    ] {
        let r = run(&["--json-errors", "number", "gcd", "--a", value, "--b", "1"]);
        assert!(!r.status.success());
        assert!(r.stdout.is_empty());
        let e: serde_json::Value = serde_json::from_slice(&r.stderr).unwrap();
        assert_eq!(e["error"]["operation"], "number.gcd");
        assert_eq!(e["error"]["parameter"], "a");
        assert_eq!(e["error"]["code"], code);
    }
    let r = run(&[
        "--json-errors",
        "number",
        "mod-inverse",
        "--a",
        "6",
        "--modulus",
        "9",
    ]);
    assert!(!r.status.success());
    assert!(r.stdout.is_empty());
    let e: serde_json::Value = serde_json::from_slice(&r.stderr).unwrap();
    assert_eq!(e["error"]["code"], "invalid_input");
    let r = run(&[
        "--json-errors",
        "number",
        "mod-pow",
        "--base",
        "1",
        "--exponent",
        "2",
        "--modulus",
        "0",
    ]);
    assert!(!r.status.success());
    assert!(r.stdout.is_empty());
    let e: serde_json::Value = serde_json::from_slice(&r.stderr).unwrap();
    assert_eq!(e["error"]["operation"], "number.mod_pow");
    assert_eq!(e["error"]["parameter"], "modulus");
}
