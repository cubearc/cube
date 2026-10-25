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
fn cli_base58check_is_independent_and_structured() {
    let data = cube_core::encoding::hex_decode("a31c06bd463e3923bc1aadbde48b16976c080717").unwrap();
    let encoded = run(&["encode", "base58check", "--version-byte", "0"], &data);
    assert!(
        encoded.status.success(),
        "{}",
        String::from_utf8_lossy(&encoded.stderr)
    );
    assert_eq!(encoded.stdout, b"1FsSia9rv4NeEwvJ2GvXrX7LyxYspbN2mo\n");
    let decoded = run(
        &["decode", "base58check", "--json"],
        encoded.stdout.strip_suffix(b"\n").unwrap(),
    );
    assert!(decoded.status.success());
    let report: serde_json::Value = serde_json::from_slice(&decoded.stdout).unwrap();
    assert_eq!(report["version"], 0);
    assert_eq!(
        report["payload_hex"],
        "a31c06bd463e3923bc1aadbde48b16976c080717"
    );
    assert!(
        !run(&["decode", "base58check"], &encoded.stdout)
            .status
            .success()
    );
    assert!(
        !run(
            &["encode", "base58check", "--version-byte", "0"],
            &vec![0; 1025]
        )
        .status
        .success()
    );
    assert!(!run(&["encode", "base58check"], b"").status.success());
}
