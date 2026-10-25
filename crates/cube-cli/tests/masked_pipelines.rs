use serde_json::{Value, json};
use std::process::Command;
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .output()
        .unwrap()
}
fn steps() -> Value {
    json!([{"operation":"text.transform.masked","version":1,"parameters":{
    "selector":"fibonacci","exclude":{"unsigned_list":[13,21]},"index_base":1,
    "selected":"atbash","otherwise":"caesar","otherwise_shift":3}}])
}
#[test]
fn cli_requires_json_and_preserves_mandatory_audits_in_pipelines_and_recipe_checks() {
    let original = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let definition = steps().to_string();
    let result = run(&[
        "pipeline",
        "run",
        "--steps",
        &definition,
        "--text",
        original,
        "--format",
        "json",
    ]);
    assert!(result.status.success(), "{:?}", result.stderr);
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["output"]["data"], "ZYXGVIJSLMNOPQRSTUVWXYZABC");
    assert_eq!(
        report["masked_audits"]["0"]["selected_positions"],
        json!([1, 2, 3, 5, 8])
    );
    for format in ["raw", "hex"] {
        let result = run(&[
            "pipeline",
            "run",
            "--steps",
            &definition,
            "--text",
            original,
            "--format",
            format,
        ]);
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("preserve step audits"));
    }
    let mut definition = steps();
    definition[0]["parameters"]["overrides"] = json!({"text_list":["1=雪","1=🧊"]});
    let recipe =
        std::env::temp_dir().join(format!("cube-masked-recipe-{}.json", std::process::id()));
    std::fs::write(&recipe,json!({"format_version":1,"engine_api":1,"input":{"source":"slot","name":"text","type":"text"},"output":{"type":"text"},"steps":definition}).to_string()).unwrap();
    let path = recipe.to_str().unwrap();
    let expected = json!({"type":"text","data":"🧊YXGVIJSLMNOPQRSTUVWXYZABC"}).to_string();
    let result = run(&[
        "recipe",
        "check",
        path,
        "--text",
        &format!("text={original}"),
        "--expect",
        &expected,
    ]);
    assert!(result.status.success(), "{:?}", result.stderr);
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(
        report["masked_audits"]["0"]["deterministic_text"],
        "ZYXGVIJSLMNOPQRSTUVWXYZABC"
    );
    assert_eq!(
        report["masked_audits"]["0"]["overrides"][1]["previous"],
        "雪"
    );
    let result = run(&[
        "recipe",
        "run",
        path,
        "--text",
        &format!("text={original}"),
        "--format",
        "json",
    ]);
    assert!(result.status.success());
    let result = run(&["recipe", "run", path, "--text", &format!("text={original}")]);
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    std::fs::remove_file(recipe).unwrap();
}
#[test]
fn typed_list_errors_have_step_context_and_do_not_emit_partial_output() {
    let mut definition = steps();
    definition[0]["parameters"]["exclude"] = json!({"unsigned_list":[-1]});
    let result = run(&[
        "--json-errors",
        "pipeline",
        "run",
        "--steps",
        &definition.to_string(),
        "--text",
        "ABC",
        "--format",
        "json",
    ]);
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    let info: Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(info["error"]["step"], 0);
    let mut definition = steps();
    definition[0]["parameters"]["exclude"] = json!({"unsigned_list":[4]});
    assert!(
        !run(&[
            "pipeline",
            "validate",
            "--steps",
            &definition.to_string(),
            "--input-type",
            "text"
        ])
        .status
        .success()
    );
}
