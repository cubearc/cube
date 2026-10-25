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
fn validators_read_stdin_and_never_report_private_material() {
    let wif = "L5nJeqKmpHp4P7F8ZYyjwc5a7P4d8EabuGAzfGJk7yC1BJyzNaEd";
    let scalar = "ff778740f88ddcf102aeb81daee289c044c4a4571c4b6f287400f4b8e0b843f8";
    let result = run(
        &[
            "blockchain",
            "wif",
            "--expected-network",
            "mainnet",
            "--json",
        ],
        wif.as_bytes(),
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let info: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(info["compressed"], true);
    for result in [
        result,
        run(
            &["blockchain", "wif", "--expected-network", "testnet"],
            wif.as_bytes(),
        ),
        run(&["blockchain", "wif"], (wif.to_owned() + "\n").as_bytes()),
        run(
            &["blockchain", "scalar", "--format", "hex", "--json"],
            scalar.as_bytes(),
        ),
        run(
            &["--json-errors", "blockchain", "scalar", "--format", "hex"],
            &[b'g'; 64],
        ),
    ] {
        let output = format!(
            "{}{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(!output.contains(wif));
        assert!(!output.contains(scalar));
    }
    assert!(!run(&["blockchain", "wif"], &[255]).status.success());
    assert!(!run(&["blockchain", "scalar"], &[0; 32]).status.success());
}

#[test]
fn bitcoin_derivation_requires_explicit_network_and_encoding() {
    let scalar = "0000000000000000000000000000000000000000000000000000000000000001";
    let result = run(
        &[
            "blockchain",
            "bitcoin",
            "--format",
            "hex",
            "--network",
            "mainnet",
            "--compressed",
            "--json",
        ],
        scalar.as_bytes(),
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["address"], "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH");
    assert_eq!(report["public_key_hex"].as_str().unwrap().len(), 66);
    assert!(!String::from_utf8_lossy(&result.stdout).contains(scalar));
    assert_eq!(
        run(
            &[
                "blockchain",
                "bitcoin",
                "--format",
                "hex",
                "--network",
                "testnet",
                "--uncompressed"
            ],
            scalar.as_bytes()
        )
        .stdout,
        b"mtoKs9V381UAhUia3d7Vb9GNak8Qvmcsme\n"
    );
    assert!(
        !run(&["blockchain", "bitcoin", "--network", "mainnet"], &[1; 32])
            .status
            .success()
    );
    assert_eq!(
        run(&["hash", "ripemd160"], b"").stdout,
        b"9c1185a5c5e9fc54612808977ee8f548b2258d31\n"
    );
}
