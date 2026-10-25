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

#[test]
fn private_exports_require_intent_and_exclusive_restricted_destinations() {
    let root = std::env::temp_dir().join(format!("cube-private-export-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let output = root.join("key.wif");
    let scalar = "0000000000000000000000000000000000000000000000000000000000000001";
    let wif = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";
    let args = [
        "blockchain",
        "export",
        "--format",
        "hex",
        "--to",
        "wif",
        "--network",
        "mainnet",
        "--compressed",
        "--private-output",
        "--output",
        output.to_str().unwrap(),
    ];
    let result = run(&args, scalar.as_bytes());
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!String::from_utf8_lossy(&result.stdout).contains(wif));
    assert_eq!(std::fs::read(&output).unwrap(), wif.as_bytes());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&output).unwrap().permissions().mode() & 0o077,
            0
        );
    }
    assert!(!run(&args, scalar.as_bytes()).status.success());
    assert_eq!(std::fs::read(&output).unwrap(), wif.as_bytes());
    let raw = run(
        &[
            "blockchain",
            "export",
            "--format",
            "wif",
            "--to",
            "raw",
            "--private-output",
            "--stdout",
        ],
        wif.as_bytes(),
    );
    assert!(raw.status.success());
    assert_eq!(raw.stdout, [vec![0; 31], vec![1]].concat());
    assert_eq!(
        run(
            &[
                "blockchain",
                "export",
                "--format",
                "wif",
                "--to",
                "hex",
                "--private-output",
                "--stdout"
            ],
            wif.as_bytes()
        )
        .stdout,
        scalar.as_bytes()
    );
    let refused = run(
        &[
            "blockchain",
            "export",
            "--format",
            "hex",
            "--to",
            "wif",
            "--network",
            "mainnet",
            "--compressed",
            "--stdout",
        ],
        scalar.as_bytes(),
    );
    assert!(!refused.status.success());
    assert!(refused.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&refused.stderr).contains(scalar));
    assert!(
        !run(
            &["blockchain", "export", "--to", "raw", "--private-output"],
            &[1; 32]
        )
        .status
        .success()
    );
    assert!(
        !run(
            &[
                "blockchain",
                "export",
                "--to",
                "wif",
                "--private-output",
                "--stdout"
            ],
            &[1; 32]
        )
        .status
        .success()
    );
    std::fs::remove_dir_all(&root).unwrap();
}
