use std::process::Command;
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn exact_factor_and_grid_cli_contracts() {
    let report = run(&["text", "grid", "--text", "ABCDEF", "--json"]);
    assert!(
        report.status.success(),
        "{}",
        String::from_utf8_lossy(&report.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&report.stdout).unwrap();
    assert_eq!(json["candidates"].as_array().unwrap().len(), 4);
    let report = run(&["text", "grid", "--text", "ABCDEF", "--width", "3"]);
    assert!(report.status.success());
    assert_eq!(report.stdout, b"\"A\"\t\"B\"\t\"C\"\n\"D\"\t\"E\"\t\"F\"\n");
    assert!(
        !run(&["text", "grid", "--text", "ABCDEF", "--width", "4"])
            .status
            .success()
    );
    let dir = std::env::temp_dir().join(format!("cube-grid-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let output = dir.join("grid.png");
    let args = [
        "text",
        "grid",
        "--text",
        "01",
        "--width",
        "2",
        "--colors",
        "[[\"0\",\"000000ff\"],[\"1\",\"ffffffff\"]]",
        "--output",
        output.to_str().unwrap(),
    ];
    assert!(run(&args).status.success());
    assert_eq!(
        cube_core::image::Image::open(&output).unwrap().samples(),
        [0, 0, 0, 255, 255, 255, 255, 255]
    );
    assert!(!run(&args).status.success());
    std::fs::remove_dir_all(dir).unwrap();
}
