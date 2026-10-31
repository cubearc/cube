use std::{
    path::PathBuf,
    process::{Command, Output},
};
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .output()
        .unwrap()
}
fn success(args: &[&str]) -> String {
    let result = run(args);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(result.stderr.is_empty());
    String::from_utf8(result.stdout).unwrap()
}
fn definition(profile: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../cube-core/tests/fixtures/mono")
        .join(format!("{profile}.json"))
}
#[test]
fn substitutions_rotations_and_default_rejection_have_exact_transport() {
    assert_eq!(
        success(&[
            "cipher",
            "substitution",
            "--source",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            "--keyword",
            "BALLOON",
            "--text",
            "ABCXYZ"
        ]),
        "BALXYZ\n"
    );
    assert_eq!(
        success(&[
            "cipher",
            "substitution",
            "--source",
            "ABC",
            "--target",
            "CAB",
            "--decode",
            "--text",
            "CAB"
        ]),
        "ABC\n"
    );
    assert_eq!(
        success(&[
            "cipher",
            "substitution",
            "--source",
            "a",
            "--target",
            "x",
            "--unmapped",
            "preserve",
            "--text",
            "ax!"
        ]),
        "xx!\n"
    );
    for (profile, input, expected) in [
        ("rot5", "A123", "A678\n"),
        ("rot18", "Hello 123!", "Uryyb 678!\n"),
        ("rot47", "Hello, World!", "w6==@[ (@C=5P\n"),
        ("rot8000", "A 😀", "籊 😀\n"),
    ] {
        assert_eq!(success(&["cipher", profile, "--text", input]), expected);
    }
    for args in [
        vec!["cipher", "substitution", "--source", "AB", "--text", "A"],
        vec![
            "cipher",
            "substitution",
            "--source",
            "AB",
            "--target",
            "AB",
            "--keyword",
            "A",
            "--text",
            "A",
        ],
        vec![
            "cipher",
            "substitution",
            "--source",
            "A",
            "--target",
            "B",
            "--text",
            "!",
        ],
    ] {
        let error = run(&args);
        assert!(!error.status.success());
        assert!(error.stdout.is_empty());
    }
}
#[test]
fn token_and_digit_profiles_use_bounded_caller_owned_definitions() {
    for (profile, text, encoded, decoded) in [
        (
            "homophonic",
            "AABA",
            "[\"1\",\"2\",\"3\",\"1\"]\n",
            "AABA\n",
        ),
        ("monome-dinome", "HIGHF", "600676062\n", "HIGHF\n"),
        ("straddling", "AB", "01\n", "AB\n"),
        ("grandpre", "A A", "[[1,1],null,[1,2]]\n", "A A\n"),
    ] {
        let path = definition(&profile.replace('-', "_"));
        let path = path.to_str().unwrap();
        assert_eq!(
            success(&["cipher", profile, "--definition", path, "--text", text]),
            encoded
        );
        assert_eq!(
            success(&[
                "cipher",
                profile,
                "--definition",
                path,
                "--decode",
                "--text",
                encoded.trim_end()
            ]),
            decoded
        );
    }
    let path = definition("tridigital");
    let path = path.to_str().unwrap();
    assert_eq!(
        success(&[
            "cipher",
            "tridigital",
            "--definition",
            path,
            "--text",
            "THE"
        ]),
        "030\n"
    );
    assert_eq!(
        success(&[
            "cipher",
            "tridigital",
            "--definition",
            path,
            "--decode",
            "--text",
            "0"
        ]),
        "[[\"A\",\"E\",\"T\"]]\n"
    );
    let path = definition("homophonic");
    let path = path.to_str().unwrap();
    assert_eq!(
        success(&[
            "cipher",
            "homophonic",
            "--definition",
            path,
            "--counters",
            r#"[{"symbol":"A","index":65535}]"#,
            "--text",
            "AA"
        ]),
        "[\"2\",\"1\"]\n"
    );
    for (profile, value) in [
        ("homophonic", "[true]"),
        ("grandpre", "[[true,1]]"),
        ("monome-dinome", "6"),
    ] {
        let path = definition(&profile.replace('-', "_"));
        let result = run(&[
            "--json-errors",
            "cipher",
            profile,
            "--definition",
            path.to_str().unwrap(),
            "--decode",
            "--text",
            value,
        ]);
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        let _: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
    }
}

#[test]
fn definition_file_and_exact_stdin_support_nul_alphabets() {
    use std::io::Write;
    use std::process::Stdio;
    let path = definition("substitution_nul");
    let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "cipher",
            "substitution",
            "--definition",
            path.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"a\0").unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(result.stdout, b"x\0\n");
    assert!(result.stderr.is_empty());
}

#[test]
fn historical_pairs_and_fixed_codebook_use_exact_cli_transport() {
    assert_eq!(
        success(&["cipher", "malespin", "bueno Amigo á"]),
        "tuani Epofi á\n"
    );
    assert_eq!(
        success(&[
            "cipher",
            "vatsyayana",
            "QOEU",
            "--pairs",
            r#"[["Q","W"],["O","I"],["E","S"],["U","H"]]"#
        ]),
        "WISH\n"
    );
    assert_eq!(
        success(&["cipher", "great-cipher", r#"["Les","En","Ne","Mi","S"]"#]),
        "[\"124\",\"22\",\"125\",\"46\",\"120\"]\n"
    );
    let report: serde_json::Value = serde_json::from_str(&success(&[
        "cipher",
        "great-cipher",
        "--decode",
        r#"["55","502"]"#,
    ]))
    .unwrap();
    assert_eq!(
        report["units"][0]["alternatives"],
        serde_json::json!(["I", "J"])
    );
    assert_eq!(
        report["units"][1]["alternatives"],
        serde_json::json!(["Bien"])
    );
    let output = run(&[
        "--json-errors",
        "cipher",
        "great-cipher",
        "--decode",
        r#"["6"]"#,
    ]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["code"], "invalid_input");
}
