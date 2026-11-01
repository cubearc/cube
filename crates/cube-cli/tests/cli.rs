use std::io::{self, Write};
use std::process::{Command, Output, Stdio};

fn cube(args: &[&str], stdin: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start cube");
    let write_result = child.stdin.take().expect("piped stdin").write_all(stdin);
    // Commands using --text or rejecting arguments may exit without reading stdin.
    if let Err(error) = write_result {
        assert_eq!(
            error.kind(),
            io::ErrorKind::BrokenPipe,
            "write stdin: {error}"
        );
    }
    child.wait_with_output().expect("wait for cube")
}

fn success(args: &[&str], stdin: &[u8], expected: &[u8]) {
    let output = cube(args, stdin);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, expected);
    assert!(output.stderr.is_empty());
}

#[test]
fn help_version_and_discovery() {
    for args in [
        vec!["--help"],
        vec!["encode", "base32", "--help"],
        vec!["decode", "base32", "--help"],
        vec!["encode", "base58", "--help"],
        vec!["decode", "base58", "--help"],
        vec!["decode", "base64", "--help"],
        vec!["cipher", "caesar", "--help"],
        vec!["hash", "crc32", "--help"],
        vec!["binary", "xor", "--help"],
    ] {
        let output = cube(&args, b"");
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
        assert!(output.stderr.is_empty());
    }
    success(
        &["--version"],
        b"",
        format!("cube {}\n", env!("CARGO_PKG_VERSION")).as_bytes(),
    );
    let output = cube(&["ops", "list"], b"");
    assert!(output.status.success());
    assert!(output.stdout.starts_with(b"encode base32\nencode base58\nencode base64\nencode hex\ndecode base32\ndecode base58\ndecode base64\ndecode hex\ncipher caesar\ncipher rot13\nhash crc32\nhash sha256\nhash sha512\nhash sha1\nhash md5\nhash crc32c\nhash crc16-ibm3740\nbinary xor\n"));
}
#[test]
fn semaphore_cli_uses_exact_anatomical_arm_contract() {
    success(
        &["encode", "semaphore", "--text", "A B"],
        b"",
        b"S,SW / S,W\n",
    );
    success(
        &["decode", "semaphore", "--text", "S,SW / S,W"],
        b"",
        b"A B\n",
    );
    assert!(
        !cube(&["decode", "semaphore", "--text", "SW,S"], b"")
            .status
            .success()
    );
}
#[test]
fn clock_cli_reports_ambiguity_and_exact_integer_angles() {
    let report = cube(
        &["number", "clock", "--time", "3:00", "--decode", "--json"],
        b"",
    );
    assert!(
        report.status.success(),
        "{}",
        String::from_utf8_lossy(&report.stderr)
    );
    let r: serde_json::Value = serde_json::from_slice(&report.stdout).unwrap();
    assert_eq!(r["decoded"], "J");
    let report = cube(
        &[
            "number",
            "clock",
            "--hour-millidegrees",
            "-45000",
            "--minute-millidegrees",
            "0",
            "--json",
        ],
        b"",
    );
    assert!(report.status.success());
    assert!(
        !cube(&["number", "clock", "--time", "12:45", "--decode"], b"")
            .status
            .success()
    );
    assert!(
        !cube(
            &[
                "number",
                "clock",
                "--time",
                "3:00",
                "--hour-millidegrees",
                "0",
                "--minute-millidegrees",
                "0"
            ],
            b""
        )
        .status
        .success()
    );
}

#[test]
fn caesar_known_answers_and_inverse() {
    success(
        &[
            "cipher",
            "caesar",
            "--shift",
            "3",
            "--text",
            "Attack at dawn!",
        ],
        b"ignored stdin",
        b"Dwwdfn dw gdzq!\n",
    );
    success(
        &[
            "cipher",
            "caesar",
            "--decode",
            "--shift",
            "3",
            "--text",
            "Dwwdfn dw gdzq!",
        ],
        b"ignored stdin",
        b"Attack at dawn!\n",
    );
    success(
        &["cipher", "caesar", "--shift", "-1", "--text", "AaZz"],
        b"",
        b"ZzYy\n",
    );
    success(
        &[
            "cipher",
            "caesar",
            "--decode",
            "--shift=-1",
            "--text",
            "ZzYy",
        ],
        b"",
        b"AaZz\n",
    );
}

#[test]
fn caesar_shift_limits_and_stdin_preserve_text() {
    for (shift, expected) in [("-2147483648", "CcBb"), ("2147483647", "XxWw")] {
        success(
            &["cipher", "caesar", "--shift", shift],
            b"AaZz",
            format!("{expected}\n").as_bytes(),
        );
        success(
            &["cipher", "caesar", "--decode", "--shift", shift],
            expected.as_bytes(),
            b"AaZz\n",
        );
    }
    for shift in ["0", "26", "-26", "52", "-52"] {
        success(&["cipher", "caesar", "--shift", shift], b"AaZz", b"AaZz\n");
        success(
            &["cipher", "caesar", "--decode", "--shift", shift],
            b"AaZz",
            b"AaZz\n",
        );
    }
    let text = "Olá, 世界 🧊 e\u{301}\0\t\r\n";
    let encoded = "Roá, 世界 🧊 h\u{301}\0\t\r\n";
    success(
        &["cipher", "caesar", "--shift", "3"],
        text.as_bytes(),
        format!("{encoded}\n").as_bytes(),
    );
    success(
        &["cipher", "caesar", "--decode", "--shift", "3"],
        encoded.as_bytes(),
        format!("{text}\n").as_bytes(),
    );
    for decode in [false, true] {
        let mut args = vec!["cipher", "caesar", "--shift", "13"];
        if decode {
            args.push("--decode");
        }
        let caesar = cube(&args, text.as_bytes());
        let rot13 = cube(&["cipher", "rot13"], text.as_bytes());
        assert!(caesar.status.success());
        assert!(rot13.status.success());
        assert!(caesar.stderr.is_empty());
        assert_eq!(caesar.stdout, rot13.stdout);
    }
}

#[test]
fn crc32_known_answers_and_binary_stdin() {
    success(
        &["hash", "crc32", "--text", "123456789"],
        b"ignored stdin",
        b"cbf43926\n",
    );
    success(&["hash", "crc32"], b"123456789", b"cbf43926\n");
    success(
        &["hash", "crc32", "--text", ""],
        b"ignored stdin",
        b"00000000\n",
    );
    success(&["hash", "crc32"], b"", b"00000000\n");
    success(&["hash", "crc32"], &[0, 255, 128], b"81dda740\n");
    success(&["hash", "crc32"], b"\0", b"d202ef8d\n");
    success(&["hash", "crc32", "--text", "\""], b"", b"0762ae69\n");
    success(&["hash", "crc32"], b"Hello\n", b"31963516\n");
    success(
        &["hash", "crc32", "--text", "Olá, 世界 🧊"],
        b"",
        b"08b240b4\n",
    );
    let bytes: Vec<u8> = (0..=255).collect();
    success(&["hash", "crc32"], &bytes, b"29058c73\n");
}

#[test]
fn base32_commands_and_binary_round_trip() {
    success(
        &["encode", "base32", "--text", "foo"],
        b"ignored",
        b"MZXW6===\n",
    );
    success(
        &["decode", "base32", "--text", "MZXW6==="],
        b"",
        b"666f6f\n",
    );
    success(&["encode", "base32"], &[0, 255], b"AD7Q====\n");
    success(&["decode", "base32"], b"AD7Q====", b"00ff\n");
    success(
        &["decode", "base32", "--output", "raw"],
        b"AD7Q====",
        &[0, 255],
    );
    success(&["encode", "base32"], b"foo\n", b"MZXW6CQ=\n");
    success(
        &["decode", "base32", "--text", "MZXW6CQ=", "--output", "raw"],
        b"",
        b"foo\n",
    );

    let bytes: Vec<u8> = (0..=255).collect();
    let encoded = cube(&["encode", "base32"], &bytes);
    assert!(encoded.status.success());
    assert!(encoded.stderr.is_empty());
    let encoded_input = encoded
        .stdout
        .strip_suffix(b"\n")
        .expect("encoded output newline");
    success(
        &["decode", "base32", "--output", "raw"],
        encoded_input,
        &bytes,
    );
}

#[test]
fn base58_commands_preserve_bytes_and_leading_zeroes() {
    success(
        &["encode", "base58", "--text", "Hello World"],
        b"ignored",
        b"JxF12TrwUP45BMd\n",
    );
    success(
        &["decode", "base58", "--text", "JxF12TrwUP45BMd"],
        b"",
        b"48656c6c6f20576f726c64\n",
    );
    success(
        &[
            "decode",
            "base58",
            "--text",
            "JxF12TrwUP45BMd",
            "--output",
            "raw",
        ],
        b"",
        b"Hello World",
    );
    success(&["encode", "base58"], &[0, 255], b"15Q\n");
    success(&["decode", "base58"], b"15Q", b"00ff\n");
    success(&["decode", "base58", "--output", "raw"], b"15Q", &[0, 255]);
    success(&["encode", "base58"], &[0, 0, 1], b"112\n");
    success(&["decode", "base58", "--output", "raw"], b"112", &[0, 0, 1]);
    success(&["encode", "base58"], &[0; 4], b"1111\n");
    success(&["decode", "base58", "--output", "raw"], b"1111", &[0; 4]);

    let bytes: Vec<u8> = (0..=255).collect();
    let encoded = cube(&["encode", "base58"], &bytes);
    assert!(encoded.status.success());
    assert!(encoded.stderr.is_empty());
    let encoded_input = encoded
        .stdout
        .strip_suffix(b"\n")
        .expect("encoded output newline");
    success(
        &["decode", "base58", "--output", "raw"],
        encoded_input,
        &bytes,
    );
}

#[test]
fn text_commands() {
    success(
        &["encode", "base64", "--text", "Hello"],
        b"ignored",
        b"SGVsbG8=\n",
    );
    success(&["encode", "hex", "--text", "Hello"], b"", b"48656c6c6f\n");
    success(
        &["decode", "base64", "--text", "SGVsbG8="],
        b"",
        b"48656c6c6f\n",
    );
    success(
        &["decode", "hex", "--text", "48656C6c6f", "--output", "raw"],
        b"",
        b"Hello",
    );
    success(
        &["cipher", "rot13", "--text", "Olá 世界"],
        b"",
        "Byá 世界\n".as_bytes(),
    );
    success(
        &["hash", "sha256", "--text", "Hello"],
        b"",
        b"185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969\n",
    );
    success(
        &["binary", "xor", "--text", "Hello", "--key", "key"],
        b"",
        b"230015070a\n",
    );
    success(
        &[
            "binary",
            "xor",
            "--text",
            "Hello",
            "--key-hex",
            "6b6579",
            "--output",
            "raw",
        ],
        b"",
        &[35, 0, 21, 7, 10],
    );
}

#[test]
fn stdin_binary_and_newlines_are_preserved() {
    success(&["encode", "base64"], &[0, 255], b"AP8=\n");
    success(&["encode", "hex"], b"Hello\n", b"48656c6c6f0a\n");
    success(&["decode", "base64", "--output", "raw"], b"AP8=", &[0, 255]);
    success(&["decode", "hex"], b"00FF", b"00ff\n");
    success(
        &["binary", "xor", "--key-hex", "ff", "--output", "raw"],
        &[0, 255],
        &[255, 0],
    );
    success(
        &["cipher", "rot13"],
        "Hello 🧊".as_bytes(),
        "Uryyb 🧊\n".as_bytes(),
    );
    success(
        &["hash", "sha256"],
        b"",
        b"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n",
    );
}

#[test]
fn empty_inputs_are_explicit_and_valid() {
    for args in [
        vec!["encode", "base58", "--text", ""],
        vec!["decode", "base58", "--text", ""],
        vec!["encode", "base32", "--text", ""],
        vec!["decode", "base32", "--text", ""],
        vec!["encode", "base64", "--text", ""],
        vec!["encode", "hex", "--text", ""],
        vec!["decode", "base64", "--text", ""],
        vec!["cipher", "rot13", "--text", ""],
        vec!["cipher", "caesar", "--shift", "3", "--text", ""],
        vec![
            "cipher",
            "caesar",
            "--decode",
            "--shift",
            "-2147483648",
            "--text",
            "",
        ],
        vec!["binary", "xor", "--text", "", "--key", "k"],
    ] {
        success(&args, b"must not read stdin", b"\n");
    }
    success(
        &["decode", "base58", "--text", "", "--output", "raw"],
        b"ignored",
        b"",
    );
    success(
        &["decode", "base32", "--text", "", "--output", "raw"],
        b"must not read stdin",
        b"",
    );
    success(
        &["decode", "hex", "--text", "", "--output", "raw"],
        b"",
        b"",
    );
}

#[test]
fn invalid_operation_input_uses_stderr_and_exit_one() {
    for (args, input, message) in [
        (
            vec!["decode", "base58", "--text", "0OIl"],
            &b""[..],
            "invalid Base58",
        ),
        (
            vec!["decode", "base58", "--text", "é"],
            &b""[..],
            "invalid Base58",
        ),
        (
            vec!["decode", "base58", "--text", "2g="],
            &b""[..],
            "invalid Base58",
        ),
        (vec!["decode", "base58"], &b"2g\n"[..], "invalid Base58"),
        (vec!["decode", "base58"], &[255][..], "utf-8"),
        (
            vec!["decode", "base32", "--text", "mzxw6==="],
            &b""[..],
            "invalid Base32",
        ),
        (
            vec!["decode", "base32", "--text", "MZXW6"],
            &b""[..],
            "invalid Base32",
        ),
        (
            vec!["decode", "base32", "--text", "MZXW7==="],
            &b""[..],
            "invalid Base32",
        ),
        (
            vec!["decode", "base32", "--text", "MY======MY======"],
            &b""[..],
            "invalid Base32",
        ),
        (
            vec!["decode", "base32", "--text", "0A======"],
            &b""[..],
            "invalid Base32",
        ),
        (
            vec!["decode", "base32"],
            &b"MZXW6===\n"[..],
            "invalid Base32",
        ),
        (vec!["decode", "base32"], &[255][..], "utf-8"),
        (
            vec!["decode", "base64", "--text", "!"],
            &b""[..],
            "invalid Base64",
        ),
        (vec!["decode", "base64"], &b"Zg==\n"[..], "invalid Base64"),
        (
            vec!["decode", "hex", "--text", "a"],
            &b""[..],
            "invalid hexadecimal",
        ),
        (
            vec!["binary", "xor", "--key", ""],
            &b""[..],
            "XOR key must not be empty",
        ),
        (
            vec!["binary", "xor", "--key-hex", ""],
            &b""[..],
            "XOR key must not be empty",
        ),
        (
            vec!["binary", "xor", "--key-hex", "zz"],
            &b""[..],
            "invalid hexadecimal",
        ),
        (vec!["cipher", "rot13"], &[255][..], "utf-8"),
        (
            vec!["cipher", "caesar", "--shift", "3"],
            &[255][..],
            "utf-8",
        ),
        (
            vec!["cipher", "caesar", "--decode", "--shift", "3"],
            &[255][..],
            "utf-8",
        ),
        (vec!["decode", "base64"], &[255][..], "utf-8"),
    ] {
        let output = cube(&args, input);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains(message));
    }
}

#[test]
fn invalid_cli_usage_exits_two() {
    for args in [
        vec![],
        vec!["unknown"],
        vec!["cipher", "caesar", "--text", "Hello"],
        vec!["cipher", "caesar", "--shift"],
        vec!["cipher", "caesar", "--shift", "three"],
        vec!["cipher", "caesar", "--shift", "1.5"],
        vec!["cipher", "caesar", "--shift", "-2147483649"],
        vec!["cipher", "caesar", "--shift", "2147483648"],
        vec!["binary", "xor"],
        vec!["binary", "xor", "--key", "k", "--key-hex", "ff"],
        vec!["decode", "base64", "--output", "lossy"],
        vec!["hash", "crc32", "--output", "raw"],
    ] {
        let output = cube(&args, b"");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn every_catalog_command_resolves_to_cli_help() {
    let commands: std::collections::BTreeSet<_> = cube_core::operations::catalog()
        .into_iter()
        .map(|descriptor| descriptor.command)
        .collect();
    for command in commands {
        let mut args: Vec<_> = command.split_whitespace().collect();
        assert!(!args.is_empty());
        args.push("--help");
        let result = cube(&args, b"");
        assert!(result.status.success(), "missing CLI command: {command}");
        assert!(String::from_utf8_lossy(&result.stdout).contains("Usage:"));
        assert!(result.stderr.is_empty());
    }
}

#[test]
fn structured_discovery_and_descriptions() {
    let result = cube(&["ops", "list", "--json"], b"");
    assert!(result.status.success());
    let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    let descriptors = value["operations"].as_array().unwrap();
    assert_eq!(descriptors.len(), cube_core::operations::catalog().len());
    let ids: Vec<_> = descriptors
        .iter()
        .map(|d| d["id"].as_str().unwrap())
        .collect();
    assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
    for id in ids {
        let result = cube(&["ops", "describe", id, "--json"], b"");
        assert!(result.status.success());
        let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(value["operation"]["id"], id);
        assert_eq!(
            value["operation"],
            *descriptors.iter().find(|d| d["id"] == id).unwrap()
        );
        assert_eq!(
            value["operation"],
            serde_json::to_value(cube_core::operations::describe(id).unwrap().discovery()).unwrap()
        );
        assert_eq!(
            value["operation"]["input_type"],
            cube_core::operations::describe(id).unwrap().input_type
        );
    }
    success(
        &["ops", "list", "--category", "binary"],
        b"",
        b"binary xor\nbinary and\nbinary or\nbinary not\nbinary reverse\nbinary shift-left\nbinary shift-right\nbinary swap-endian\nbinary rotate-left\nbinary rotate-right\nbinary xor-exact\nbinary add-wrapping\nbinary sub-wrapping\ncompose bytes\n",
    );
    for args in [
        vec!["ops", "list", "--category", "missing"],
        vec!["ops", "describe", "missing"],
    ] {
        let result = cube(&args, b"");
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("unknown"));
    }
    let result = cube(&["ops", "describe", "cipher.caesar.encode"], b"");
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("shift"));
    assert!(String::from_utf8_lossy(&result.stdout).contains("Pipeline: adapter version 1"));
    let result = cube(&["ops", "describe", "file.info.inspect"], b"");
    assert!(result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stdout)
            .contains("Pipeline: unavailable (no bounded adapter)")
    );
}

#[test]
fn optional_profile_discovery_preserves_json_and_enforces_complete_output_limits() {
    use cube_core::operations::{self as o, DiscoveryFormat as F, DiscoveryRequest as R};
    let result = cube(&["ops", "list", "--json", "--metadata"], b"");
    assert!(result.status.success());
    let expected = o::discovery_document(
        R::List { category: None },
        F::CliJson { metadata: true },
        o::MAX_DISCOVERY_BYTES,
    )
    .unwrap();
    assert_eq!(result.stdout, expected.as_bytes());
    let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(
        value["operations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| d.get("metadata").is_some())
            .count(),
        255
    );
    let args = [
        "ops",
        "describe",
        "blockchain.private.export",
        "--json",
        "--metadata",
    ];
    let result = cube(&args, b"");
    assert!(result.status.success());
    let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(
        value["operation"]["metadata"]["security"]["private_output"],
        true
    );
    assert!(value["operation"]["pipeline"].is_null());
    let exact = result.stdout.len().to_string();
    let mut bounded = args.to_vec();
    bounded.extend(["--max-output-bytes", &exact]);
    success(&bounded, b"", &result.stdout);
    let smaller = (result.stdout.len() - 1).to_string();
    *bounded.last_mut().unwrap() = &smaller;
    bounded.insert(0, "--json-errors");
    let failure = cube(&bounded, b"");
    assert!(!failure.status.success());
    assert!(failure.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&failure.stderr).unwrap();
    assert!(error.to_string().contains("resource_limit"));
    for args in [
        vec!["ops", "list", "--metadata"],
        vec![
            "ops",
            "describe",
            "hash.sha256",
            "--max-output-bytes",
            "1048577",
        ],
        vec![
            "ops",
            "describe",
            "Variant Beaufort",
            "--json",
            "--metadata",
        ],
        vec![
            "ops",
            "list",
            "--json",
            "--metadata",
            "--max-output-bytes",
            "0",
        ],
    ] {
        let failure = cube(&args, b"");
        assert!(!failure.status.success());
        assert!(failure.stdout.is_empty());
    }
    let result = cube(&["ops", "describe", "hash.sha256"], b"");
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("Canonical name: SHA-256"));
}

#[test]
fn structured_errors_keep_exit_codes_and_stdout_clean() {
    for (args, input, code, status) in [
        (
            vec!["--json-errors", "decode", "hex"],
            b"zz".as_slice(),
            "invalid_input",
            1,
        ),
        (
            vec!["cipher", "rot13", "--json-errors"],
            &[255][..],
            "invalid_input",
            1,
        ),
        (
            vec!["--json-errors", "ops", "describe", "missing"],
            b"".as_slice(),
            "invalid_parameter",
            1,
        ),
        (
            vec!["--json-errors", "cipher", "caesar"],
            b"".as_slice(),
            "invalid_usage",
            2,
        ),
    ] {
        let output = cube(&args, input);
        assert_eq!(output.status.code(), Some(status));
        assert!(output.stdout.is_empty());
        let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(error["schema_version"], 1);
        assert_eq!(error["error"]["code"], code);
    }
    let output = cube(&["--json-errors", "--help"], b"");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
}
