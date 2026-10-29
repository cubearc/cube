use std::process::Command;

#[test]
fn ordered_text_and_byte_joins_use_explicit_inputs_and_exact_output() {
    for (args, expected) in [
        (
            vec![
                "compose",
                "text",
                "--left",
                "é",
                "--right",
                "世界",
                "--separator",
                " | ",
            ],
            "é | 世界".as_bytes(),
        ),
        (
            vec![
                "compose",
                "bytes",
                "--left-hex",
                "00ff",
                "--right-hex",
                "800a",
                "--output",
                "raw",
            ],
            &[0, 255, 128, 10][..],
        ),
        (
            vec!["compose", "text", "--left", "", "--right", ""],
            &[][..],
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
        assert_eq!(result.stdout, expected);
    }
    let result = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(["compose", "bytes", "--left-hex", "ff0", "--right-hex", "00"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
}
