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
