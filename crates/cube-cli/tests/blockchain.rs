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

#[test]
fn ethereum_and_keccak_cli_preserve_public_bytes_and_errors() {
    let key = format!("{:064x}", 1);
    let result = run(
        &["blockchain", "ethereum", "--format", "hex", "--json"],
        key.as_bytes(),
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(
        json["address"],
        "0x7E5F4552091A69125d5DfCb7b8C2659029395Bdf"
    );
    assert_eq!(json["public_key_xy_hex"].as_str().unwrap().len(), 128);
    assert!(!String::from_utf8_lossy(&result.stdout).contains(&key));
    let lower = run(
        &[
            "blockchain",
            "ethereum",
            "--format",
            "hex",
            "--checksum",
            "lowercase",
        ],
        key.as_bytes(),
    );
    assert_eq!(
        lower.stdout,
        b"0x7e5f4552091a69125d5dfcb7b8c2659029395bdf\n"
    );
    let invalid = "invalid-private-input";
    let result = run(
        &["--json-errors", "blockchain", "ethereum", "--format", "hex"],
        invalid.as_bytes(),
    );
    assert!(!result.status.success());
    assert!(!String::from_utf8_lossy(&result.stderr).contains(invalid));
    let digest = run(&["hash", "keccak256", "--text", "abc"], b"");
    assert!(digest.status.success());
    assert_eq!(
        digest.stdout,
        b"4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45\n"
    );
}
#[test]
fn candidate_cli_has_safe_defaults_explicit_public_output_and_strict_transport() {
    let hex = format!("{:064x}", 1);
    let wif = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";
    let entries=serde_json::to_vec(&serde_json::json!([{"format":"wif","value":wif},{"format":"hex","value":hex},{"format":"wif","value":"invalid-private-material"}])).unwrap();
    let result = run(&["blockchain", "candidates", "--json"], &entries);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(json["progress"]["valid"], 2);
    assert_eq!(json["progress"]["invalid"], 1);
    assert_eq!(json["progress"]["complete"], true);
    assert!(json["results"][0].get("public_address").is_none());
    for secret in [wif, &hex, "invalid-private-material"] {
        assert!(!String::from_utf8_lossy(&result.stdout).contains(secret));
    }
    let human = run(&["blockchain", "candidates"], &entries);
    assert!(human.status.success());
    assert!(String::from_utf8_lossy(&human.stdout).contains("Complete: true"));
    let public = run(
        &[
            "blockchain",
            "candidates",
            "--derive",
            "bitcoin",
            "--network",
            "mainnet",
            "--compressed",
            "--json",
        ],
        &entries,
    );
    assert!(public.status.success());
    let json: serde_json::Value = serde_json::from_slice(&public.stdout).unwrap();
    assert_eq!(
        json["results"][0]["public_address"]["value"]["address"],
        "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH"
    );
    assert_eq!(json["progress"]["public_derived"], 2);
    let public = run(
        &["blockchain", "candidates", "--derive", "ethereum"],
        &entries,
    );
    assert!(public.status.success());
    assert!(
        String::from_utf8_lossy(&public.stdout)
            .contains("0x7E5F4552091A69125d5DfCb7b8C2659029395Bdf")
    );
    for args in [
        vec!["blockchain", "candidates", "--network", "mainnet"],
        vec!["blockchain", "candidates", "--derive", "bitcoin"],
        vec![
            "blockchain",
            "candidates",
            "--derive",
            "ethereum",
            "--compressed",
        ],
    ] {
        assert!(!run(&args, &entries).status.success());
    }
    for bad in [
        br#"[{"format":"hex","format":"wif","value":"private-data"}]"#.to_vec(),
        vec![b' '; 65537],
    ] {
        let failed = run(
            &["--json-errors", "blockchain", "candidates", "--json"],
            &bad,
        );
        assert!(!failed.status.success());
        assert!(!String::from_utf8_lossy(&failed.stderr).contains("private-data"));
        assert!(failed.stdout.is_empty());
    }
}
