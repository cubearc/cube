use std::{
    io::Write,
    process::{Command, Stdio},
};
fn run(name: &str, bytes: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(["hash", name])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Resource-limit failures may close stdin early.
    let _ = child.stdin.take().unwrap().write_all(bytes);
    child.wait_with_output().unwrap()
}
#[test]
fn exact_hash_commands_use_binary_stdin_and_limit_new_io() {
    for (name, expected) in [
        ("sha512", cube_core::hashing::sha512(b"abc")),
        ("sha1", cube_core::hashing::sha1(b"abc")),
        ("md5", cube_core::hashing::md5(b"abc")),
        ("crc32c", "e3069283".into()),
        ("crc16-ibm3740", "29b1".into()),
    ] {
        let data = if name.starts_with("crc") {
            &b"123456789"[..]
        } else {
            &b"abc"[..]
        };
        let result = run(name, data);
        assert!(result.status.success(), "{:?}", result.stderr);
        assert_eq!(result.stdout, format!("{expected}\n").as_bytes());
    }
    let binary: Vec<u8> = (0..=255).collect();
    assert_eq!(
        run("sha1", &binary).stdout,
        format!("{}\n", cube_core::hashing::sha1(&binary)).as_bytes()
    );
    let result = run(
        "sha512",
        &vec![0; cube_core::binary::transform::MAX_INPUT + 1],
    );
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    let help = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(["hash", "--help"])
        .output()
        .unwrap();
    assert!(
        String::from_utf8(help.stdout)
            .unwrap()
            .contains("collision-resistant security")
    );
}
