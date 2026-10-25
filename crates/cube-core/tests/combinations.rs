use cube_core::text::combinations::{Combinations, parse_groups};
fn groups(data: &[&[&str]]) -> Vec<Vec<String>> {
    data.iter()
        .map(|g| g.iter().map(|s| s.to_string()).collect())
        .collect()
}
#[test]
fn product_order_empty_semantics_and_exact_jsonl_bounds() {
    let mut c = Combinations::new(groups(&[&["a", "b"], &["1", "2"]]), "-".into(), 4).unwrap();
    assert_eq!(c.plan().total_results, 4);
    assert_eq!(c.next().unwrap(), "a-1");
    assert_eq!(c.len(), 3);
    assert_eq!(c.collect::<Vec<_>>(), ["a-2", "b-1", "b-2"]);
    assert_eq!(
        Combinations::new(vec![], "ignored".into(), 1)
            .unwrap()
            .collect::<Vec<_>>(),
        [""]
    );
    assert_eq!(
        Combinations::new(groups(&[&["a"], &[]]), "".into(), 1)
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        Combinations::new(groups(&[&["", ""], &[""]]), "|".into(), 2)
            .unwrap()
            .collect::<Vec<_>>(),
        ["|", "|"]
    );
    let c = Combinations::new(groups(&[&["🧊", "\n", "\0"], &["é", ""]]), "\"".into(), 6).unwrap();
    let raw = c.plan().aggregate_output_bytes;
    let json = c.plan().jsonl_output_bytes;
    let values: Vec<_> = c.collect();
    assert_eq!(raw, values.iter().map(String::len).sum::<usize>());
    assert_eq!(
        json,
        values
            .iter()
            .map(|s| serde_json::to_string(s).unwrap().len() + 1)
            .sum::<usize>()
    );
    assert!(parse_groups("[[\"a\"],[\"b\"]]").is_ok());
    for bad in ["{}", "[1]", "[[false]]"] {
        assert!(parse_groups(bad).is_err());
    }
}
#[test]
fn count_overflow_and_independent_limits_prevent_generation() {
    assert!(Combinations::new(vec![vec![String::new(); 256]; 16], "".into(), 10000).is_err());
    assert!(Combinations::new(groups(&[&["a", "b"], &["1", "2"]]), "".into(), 3).is_err());
    assert!(Combinations::new(vec![vec![]; 17], "".into(), 1).is_err());
    assert!(Combinations::new(vec![vec!["a".repeat(65537)]], "".into(), 1).is_err());
    assert!(
        Combinations::new(
            vec![vec!["a".repeat(40000)], vec!["b".repeat(40000)]],
            "".into(),
            1
        )
        .is_err()
    );
    assert!(Combinations::new(vec![vec!["a".repeat(2048); 64]; 2], "".into(), 10000).is_err());
    let work = Combinations::new(vec![vec!["a".repeat(2000); 64]; 2], "".into(), 10000)
        .err()
        .unwrap();
    assert!(work.message.contains("work"));
    assert!(Combinations::new(vec![vec!["a".repeat(65536); 17]], "".into(), 100).is_err());
    assert!(Combinations::new(vec![vec![String::new(); 4097]], "".into(), 10000).is_err());
    assert!(Combinations::new(vec![vec!["\0".repeat(2048); 64]; 2], "".into(), 10000).is_err());
    assert!(Combinations::new(vec![], "x".repeat(4097), 1).is_err());
    assert!(Combinations::new(vec![], "".into(), 0).is_err());
    let mut c = Combinations::new(vec![vec!["x".into(); 100]; 2], "".into(), 10000).unwrap();
    assert_eq!(c.next().unwrap(), "xx");
    assert_eq!(c.len(), 9999);
    drop(c);
    let c = Combinations::new(vec![vec!["x".repeat(65536)]], "".into(), 1).unwrap();
    assert_eq!(c.plan().max_result_bytes, 65536);
}
