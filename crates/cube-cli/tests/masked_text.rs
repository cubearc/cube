use std::{
    io::Write,
    process::{Command, Stdio},
};
fn run(args: &[&str], input: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
#[test]
fn fibonacci_cli_known_answer_plain_and_json() {
    let args = [
        "text",
        "transform",
        "masked",
        "--selector",
        "fibonacci",
        "--exclude",
        "13,21",
        "--selected",
        "atbash",
        "--otherwise",
        "caesar",
        "--otherwise-shift",
        "3",
        "--index-base",
        "1",
    ];
    let input = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let result = run(&args, input);
    assert!(result.status.success(), "{:?}", result.stderr);
    assert_eq!(result.stdout, b"ZYXGVIJSLMNOPQRSTUVWXYZABC\n");
    let result = run(
        &[
            args.as_slice(),
            &[
                "--text",
                input,
                "--json",
                "--override",
                "1=雪",
                "--override",
                "1=🧊",
            ],
        ]
        .concat(),
        "",
    );
    assert!(result.status.success(), "{:?}", result.stderr);
    let actual: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    use cube_core::text::{
        positions::{IndexBase, Selector},
        transform::{self, Config, Override, Transformation},
    };
    let expected = transform::masked(
        input,
        &Config {
            index_base: IndexBase::One,
            selector: Selector::Fibonacci {
                exclude: vec![13, 21],
            },
            selected: Transformation::Atbash,
            otherwise: Transformation::Caesar { shift: 3 },
            overrides: vec![
                Override::new(1, "雪").unwrap(),
                Override::new(1, "🧊").unwrap(),
            ],
        },
    )
    .unwrap();
    assert_eq!(actual, serde_json::to_value(expected).unwrap());
}
#[test]
fn other_selectors_signed_shifts_and_invalid_parameters() {
    for (args, input, expected) in [
        (
            vec![
                "--selector",
                "indices",
                "--indices",
                "5,1,1,2",
                "--selected",
                "atbash",
            ],
            "ABCDEF",
            "AYXDEU\n",
        ),
        (
            vec![
                "--selector",
                "every_nth",
                "--nth",
                "2",
                "--selected",
                "caesar",
                "--selected-shift",
                "-2147483648",
            ],
            "A A",
            "A A\n",
        ),
        (
            vec!["--selector", "fibonacci", "--selected", "rot13"],
            "",
            "\n",
        ),
        (
            vec![
                "--selector",
                "indices",
                "--indices",
                "",
                "--selected",
                "identity",
            ],
            "e\u{301} !雪\r\n",
            "e\u{301} !雪\r\n\n",
        ),
    ] {
        let result = run(
            &[&["text", "transform", "masked"], args.as_slice()].concat(),
            input,
        );
        assert!(result.status.success(), "{:?}", result.stderr);
        assert_eq!(result.stdout, expected.as_bytes());
    }
    for extra in [
        vec!["--exclude", "4"],
        vec!["--index-base", "2"],
        vec!["--nth", "2"],
        vec!["--override", "3=x"],
        vec!["--override", "0=ab"],
        vec!["--selected-shift", "3"],
    ] {
        let result = run(
            &[
                &[
                    "--json-errors",
                    "text",
                    "transform",
                    "masked",
                    "--selector",
                    "fibonacci",
                    "--selected",
                    "atbash",
                ],
                extra.as_slice(),
            ]
            .concat(),
            "ABC",
        );
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        let diagnostic: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
        assert_eq!(diagnostic["error"]["operation"], "text.transform.masked");
    }
    let result = run(&["ops", "describe", "text.transform.masked", "--json"], "");
    assert!(result.status.success());
    let metadata: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert!(metadata.to_string().contains("text transform masked"));
}
