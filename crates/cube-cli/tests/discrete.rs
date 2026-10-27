use std::process::Command;
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .output()
        .unwrap()
}
fn json(args: &[&str]) -> serde_json::Value {
    let r = run(args);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    serde_json::from_slice(&r.stdout).unwrap()
}
#[test]
fn count_inventory_boolean_and_graph_commands_have_exact_json_contracts() {
    assert_eq!(
        run(&["number", "permutations", "--n", "5", "--k", "2"]).stdout,
        b"20\n"
    );
    assert_eq!(
        json(&["number", "binomial", "--n", "128", "--k", "64", "--json"])["value"],
        "23951146041928082866135587776380551750"
    );
    assert_eq!(
        json(&["number", "multinomial", "--groups", "2,1,1", "--json"])["value"],
        "12"
    );
    assert_eq!(
        json(&["number", "multinomial", "--groups", "", "--json"])["value"],
        "1"
    );
    assert_eq!(
        json(&["number", "pascal", "--rows", "4"])["rows"][3],
        serde_json::json!(["1", "3", "3", "1"])
    );
    let r = json(&[
        "text",
        "multiset",
        "compare",
        "--text",
        "listen",
        "--requested",
        "silent",
    ]);
    assert_eq!(r["equal"], true);
    let r = json(&[
        "text",
        "multiset",
        "subtract",
        "--text",
        "ab",
        "--requested",
        "aab",
    ]);
    assert_eq!(r["contains"], false);
    assert_eq!(r["deficits"], serde_json::json!([{"scalar":"a","count":1}]));
    let root = std::env::temp_dir().join(format!("cube-multiset-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let a = root.join("source 雪.txt");
    let b = root.join("requested 雪.txt");
    std::fs::write(&a, "ÉA!").unwrap();
    std::fs::write(&b, "aE\u{301}!").unwrap();
    let r = json(&[
        "text",
        "multiset",
        "compare",
        "--file",
        a.to_str().unwrap(),
        "--requested-file",
        b.to_str().unwrap(),
        "--normalization",
        "NFD",
        "--case",
        "ascii_lower",
    ]);
    assert_eq!(r["equal"], true);
    std::fs::remove_dir_all(root).unwrap();
    let model = r#"{"variables":["A","B"],"formula":{"op":"implies","left":{"op":"var","name":"A"},"right":{"op":"var","name":"B"}},"other":{"op":"var","name":"B"}}"#;
    let r = json(&[
        "logic",
        "boolean",
        "analyze",
        "--text",
        model,
        "--max-rows",
        "1",
    ]);
    assert_eq!(r["true_count"], 3);
    assert_eq!(r["listing_complete"], false);
    assert_eq!(r["relationship"]["implies"], false);
    let r = json(&[
        "logic",
        "boolean",
        "evaluate",
        "--text",
        model,
        "--assignment",
        r#"[{"name":"A","value":true},{"name":"B","value":false}]"#,
    ]);
    assert_eq!(r["value"], false);
    assert_eq!(r["other_value"], false);
    let graph = r#"{"vertices":["a","b","z"],"directed":false,"edges":[{"id":"e0","from":"a","to":"b"},{"id":"e1","from":"a","to":"b"}]}"#;
    assert_eq!(
        json(&["graph", "components", "--text", graph])["result"],
        serde_json::json!([["a", "b"], ["z"]])
    );
    assert_eq!(
        json(&[
            "graph",
            "shortest-path",
            "--text",
            graph,
            "--source",
            "a",
            "--target",
            "b"
        ])["result"]["edges"],
        serde_json::json!(["e0"])
    );
    assert_eq!(
        json(&["graph", "cycle", "--text", graph])["result"]["edges"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        json(&["graph", "euler-trail", "--text", graph, "--start", "b"])["result"]["circuit"],
        true
    );
}
#[test]
fn malformed_models_counts_and_exhausted_scans_never_emit_success() {
    for args in [
        vec!["number", "binomial", "--n", "1", "--k", "2"],
        vec!["number", "permutations", "--n", "128", "--k", "128"],
        vec![
            "number",
            "binomial",
            "--n",
            "5",
            "--k",
            "2",
            "--max-work",
            "0",
        ],
        vec!["number", "binomial", "--n", "-1", "--k", "0"],
        vec![
            "text",
            "multiset",
            "compare",
            "--text",
            "a",
            "--requested",
            "b",
            "--max-work",
            "0",
        ],
        vec![
            "logic",
            "boolean",
            "analyze",
            "--text",
            r#"{"variables":[],"formula":{"op":"const","value":true}}"#,
            "--max-work",
            "0",
        ],
        vec![
            "logic",
            "boolean",
            "analyze",
            "--text",
            r#"{"variables":[],"variables":[],"formula":{"op":"const","value":true}}"#,
        ],
        vec![
            "graph",
            "components",
            "--text",
            r#"{"vertices":["a","a"],"directed":false,"edges":[]}"#,
        ],
        vec![
            "graph",
            "components",
            "--text",
            r#"{"vertices":["a"],"directed":false,"edges":[]}"#,
            "--max-work",
            "0",
        ],
    ] {
        let mut argv = vec!["--json-errors"];
        argv.extend(args);
        let r = run(&argv);
        assert!(!r.status.success());
        assert!(r.stdout.is_empty());
        let v: serde_json::Value = serde_json::from_slice(&r.stderr).unwrap();
        assert!(v["error"]["code"].is_string());
    }
}
