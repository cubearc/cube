use std::process::Command;
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn cli_override_is_explicit_and_json_retains_certification() {
    assert_eq!(
        run(&["number", "digits", "pi"]).stdout,
        b"1415926535897932\n"
    );
    let failed = run(&["number", "digits", "pi", "--start", "256", "--count", "1"]);
    assert!(!failed.status.success());
    assert!(failed.stdout.is_empty());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("max_digits"));
    let result = run(&[
        "number",
        "digits",
        "pi",
        "--start",
        "256",
        "--count",
        "1",
        "--max-digits",
        "512",
        "--json",
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(json["integer_part"], "3");
    assert_eq!(json["max_digits"], 512);
    assert_eq!(json["digits"].as_str().unwrap().len(), 1);
    assert_eq!(json["prefix_sha256"].as_str().unwrap().len(), 64);
    assert_eq!(json["fractional_zero_based"], true);
    assert_eq!(
        run(&["number", "digits", "pi", "--indices", "2,0,2"]).stdout,
        b"111\n"
    );
    assert_eq!(
        run(&["number", "digits", "e", "--indices", ""]).stdout,
        b"\n"
    );
    for flags in [
        vec!["--max-digits", "4097"],
        vec!["--indices", "-1"],
        vec!["--indices", "1.5"],
        vec!["--indices", "0", "--start", "0"],
    ] {
        let mut args = vec!["number", "digits", "pi"];
        args.extend(flags);
        let result = run(&args);
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
    }
}
