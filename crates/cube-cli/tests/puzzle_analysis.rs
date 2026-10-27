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
fn lexical_and_exact_probability_json_commands() {
    let r = json(&[
        "text",
        "anagram",
        "search",
        "ab",
        "--dictionary",
        r#"["ab","a","b"]"#,
    ]);
    assert_eq!(r["solutions"], serde_json::json!([[0], [1, 2]]));
    let r = json(&[
        "text",
        "anagram",
        "search",
        "aa",
        "--dictionary",
        r#"["a"]"#,
        "--reuse",
    ]);
    assert_eq!(r["solutions"], serde_json::json!([[0, 0]]));
    let r = json(&[
        "text",
        "word-ladder",
        "shortest",
        "--dictionary",
        r#"["cat","cot","cog","dog"]"#,
        "--start",
        "cat",
        "--end",
        "dog",
    ]);
    assert_eq!(r["status"], "found");
    assert_eq!(r["path"], serde_json::json!([0, 2, 1, 3]));
    let r = json(&[
        "text",
        "word-ladder",
        "validate",
        "--dictionary",
        r#"["cat","cot"]"#,
        "--path",
        r#"["cat","cat"]"#,
    ]);
    assert_eq!(r["failure"]["position"], 1);
    let r = json(&[
        "probability",
        "finite",
        "analyze",
        r#"{"outcomes":[{"id":"H","weight":"1"},{"id":"T","weight":1}],"event_a":["H"]}"#,
    ]);
    assert_eq!(
        r["probability_a"],
        serde_json::json!({"numerator":"1","denominator":"2"})
    );
    let r = json(&[
        "probability",
        "dice",
        "sum",
        r#"{"dice":[[{"value":1,"weight":1},{"value":2,"weight":1}]]}"#,
    ]);
    assert_eq!(r["total_mass"], "2");
    let r = json(&[
        "probability",
        "dice",
        "compare",
        r#"{"left":{"dice":[]},"right":{"dice":[]}}"#,
    ]);
    assert_eq!(r["tie_mass"], "1");
}
#[test]
fn explicit_local_dictionary_files_and_spatial_image_measurements() {
    let root = std::env::temp_dir().join(format!("cube-puzzle-analysis-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let dictionary = root.join("dictionary 雪.json");
    std::fs::write(&dictionary, r#"["cat","cot","cog","dog"]"#).unwrap();
    let r = json(&[
        "text",
        "word-ladder",
        "shortest",
        "--dictionary-file",
        dictionary.to_str().unwrap(),
        "--start",
        "cat",
        "--end",
        "dog",
    ]);
    assert_eq!(r["status"], "found");
    assert!(
        !run(&[
            "text",
            "anagram",
            "search",
            "cat",
            "--dictionary",
            "[]",
            "--dictionary-file",
            dictionary.to_str().unwrap()
        ])
        .status
        .success()
    );
    let image = root.join("stripes.png");
    cube_core::image::Image::from_samples(
        4,
        1,
        cube_core::image::ColorModel::Gray,
        vec![0, 255, 0, 255],
        vec![],
        vec![],
    )
    .unwrap()
    .save_png(&image)
    .unwrap();
    let r = json(&[
        "image",
        "autocorrelation",
        "--input",
        image.to_str().unwrap(),
        "--region",
        "0,0,4,1",
        "--channel",
        "gray",
        "--lags",
        r#"[{"dx":1,"dy":0}]"#,
    ]);
    assert_eq!(r["measurements"][0]["correlation"], -1.0);
    assert_eq!(r["measurements"][0]["squared_difference"], 195075);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn errors_and_resource_exhaustion_are_never_no_path_or_empty_proofs() {
    for args in [
        vec![
            "text",
            "word-ladder",
            "shortest",
            "--dictionary",
            r#"["cat","dog"]"#,
            "--start",
            "cat",
            "--end",
            "dog",
            "--max-work",
            "0",
        ],
        vec![
            "probability",
            "dice",
            "sum",
            r#"{"dice":[[{"value":0,"weight":0}]]}"#,
        ],
        vec!["text", "anagram", "search", "ab", "--dictionary", r#"[""]"#],
    ] {
        let r = run(&args);
        assert!(!r.status.success());
        assert!(!r.stderr.is_empty());
    }
}
