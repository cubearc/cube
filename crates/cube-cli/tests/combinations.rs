use std::process::Command;
#[test]
fn combination_cli_streams_unambiguous_records_and_rejects_excessive_plans() {
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(args)
            .output()
            .unwrap()
    };
    let input = "[[\"a\",\"b\"],[\"1\",\"2\"]]";
    let output = run(&["text", "combinations", "--text", input]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"\"a1\"\n\"a2\"\n\"b1\"\n\"b2\"\n");
    let plan = run(&["text", "combinations", "--text", input, "--plan"]);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&plan.stdout).unwrap()["total_results"],
        4
    );
    let failed = run(&[
        "text",
        "combinations",
        "--text",
        input,
        "--max-results",
        "3",
    ]);
    assert!(!failed.status.success());
    assert!(failed.stdout.is_empty());
    let output = run(&["text", "combinations", "--text", "[[\"a\\nb\",\"🧊\"]]"]);
    let values: Vec<String> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(values, ["a\nb", "🧊"]);
}
