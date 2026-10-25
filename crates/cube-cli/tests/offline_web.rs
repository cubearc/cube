use std::process::Command;
#[test]
fn offline_web_cli_reads_only_selected_sources_and_reports_inert_data() {
    let path = std::env::temp_dir().join(format!("cube-offline-web-{}.html", std::process::id()));
    let original = "<script>fetch('https://invalid.example')</script><a href='../secret'>Text</a>";
    std::fs::write(&path, original).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(["web", "html", path.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(result.status.success(), "{:?}", result.stderr);
    let r: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(r["references"][0]["value"], "../secret");
    assert_eq!(r["text"], "Text");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    let har = serde_json::json!({"log":{"version":"1.2","entries":[{"request":{"method":"GET","url":"file:///not-opened"},"response":{"status":0}}]}});
    std::fs::write(&path, har.to_string()).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(["web", "har", path.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(result.status.success(), "{:?}", result.stderr);
    let r: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(r["entries"][0]["status"], 0);
    assert!(r["entries"][0]["mime_type"].is_null());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn selected_web_assets_use_explicit_binary_outputs_and_never_overwrite() {
    let directory = std::env::temp_dir().join(format!("cube-web-assets-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let source = directory.join("saved.html");
    let destination = directory.join("selected.bin");
    let original = "<img src='data:application/octet-stream;base64,AP8='>";
    std::fs::write(&source, original).unwrap();
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(args)
            .output()
            .unwrap()
    };
    let prefix = [
        "web",
        "asset",
        "html",
        source.to_str().unwrap(),
        "--index",
        "0",
        "--selector",
        "data_url",
    ];
    let result = run(&[prefix.as_slice(), &["--stdout"]].concat());
    assert!(result.status.success(), "{:?}", result.stderr);
    assert_eq!(result.stdout, [0, 255]);
    let result = run(&[prefix.as_slice(), &["--json"]].concat());
    assert!(result.status.success(), "{:?}", result.stderr);
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["data_hex"], "00ff");
    assert_eq!(report["provenance"]["bytes"], 2);
    let result = run(&[
        prefix.as_slice(),
        &["--output", destination.to_str().unwrap()],
    ]
    .concat());
    assert!(result.status.success(), "{:?}", result.stderr);
    assert_eq!(std::fs::read(&destination).unwrap(), [0, 255]);
    let result = run(&[
        prefix.as_slice(),
        &["--output", destination.to_str().unwrap()],
    ]
    .concat());
    assert!(!result.status.success());
    assert_eq!(std::fs::read(&destination).unwrap(), [0, 255]);
    let result = run(&[prefix.as_slice(), &["--output", source.to_str().unwrap()]].concat());
    assert!(!result.status.success());
    assert_eq!(std::fs::read_to_string(&source).unwrap(), original);
    assert!(!run(&prefix).status.success());
    assert!(
        !run(&[prefix.as_slice(), &["--json", "--stdout"]].concat())
            .status
            .success()
    );
    let steps = r#"[{"operation":"web.html.asset","version":1,"parameters":{"index":0,"selector":"data_url"}},{"operation":"encoding.hex.encode","version":1}]"#;
    let result = run(&[
        "pipeline", "run", "--text", original, "--steps", steps, "--format", "raw",
    ]);
    assert!(result.status.success(), "{:?}", result.stderr);
    assert_eq!(result.stdout, b"00ff");
    let har = serde_json::json!({"log":{"version":"1.2","entries":[{"request":{"method":"GET","url":"file:///never-opened"},"response":{"status":200,"content":{"text":"AP8=","encoding":"base64"}}}]}});
    std::fs::write(&source, har.to_string()).unwrap();
    let result = run(&[
        "web",
        "asset",
        "har",
        source.to_str().unwrap(),
        "--index",
        "0",
        "--stdout",
    ]);
    assert!(result.status.success(), "{:?}", result.stderr);
    assert_eq!(result.stdout, [0, 255]);
    std::fs::remove_dir_all(directory).unwrap();
}
