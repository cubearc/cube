use std::process::Command;
#[test]
fn real_cli_selects_typed_fields_and_composes_exact_integers() {
    for (args, expected) in [
        (
            vec!["compose", "parse-json", "--text", "{\"dimensions\":[2,3]}"],
            "{\"dimensions\":[2,3]}\n",
        ),
        (
            vec![
                "compose",
                "select",
                "--json",
                "{\"a/b\":[null,7]}",
                "--pointer",
                "/a~1b/1",
            ],
            "7\n",
        ),
        (
            vec!["compose", "json-text", "--json", "\"é\\u0000世界\\r\\n\""],
            "é\0世界\r\n",
        ),
        (vec!["compose", "integer", "--text=-7"], "-7\n"),
        (vec!["compose", "maximum", "--values", "[-9,3,2]"], "3\n"),
        (
            vec!["compose", "add", "--left", "3", "--right", "7"],
            "10\n",
        ),
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(args)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(result.stdout, expected.as_bytes());
    }
    for args in [
        vec![
            "compose",
            "select",
            "--json",
            "{\"x\":null}",
            "--pointer",
            "/missing",
        ],
        vec!["compose", "maximum", "--values", "[]"],
        vec!["compose", "maximum", "--values", "[1,\"2\"]"],
        vec!["compose", "maximum", "--values", "[1,2.0]"],
        vec![
            "compose",
            "add",
            "--left",
            "9007199254740991",
            "--right",
            "1",
        ],
        vec!["compose", "integer", "--text", "1e3"],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(args)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
    }
}
