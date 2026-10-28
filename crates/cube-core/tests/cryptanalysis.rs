use cube_core::cryptanalysis::{self as a, scoring};
use serde_json::json;

fn model() -> serde_json::Value {
    json!({"kind":"counts","n":1,"entries":[{"text":"A","weight":3.0},{"text":"B","weight":1.0}],"smoothing":1.0,"normalization":"ascii_letters"})
}

#[test]
fn caller_model_scores_are_fixed_point_stable_and_empty_is_undefined() {
    let request =
        serde_json::from_value(json!({"candidates":["B","a!","","A"],"model":model()})).unwrap();
    let report = scoring::score(&request, None).unwrap();
    assert!(report.status.complete);
    assert_eq!(report.ranking, [1, 3, 0, 2]);
    assert_eq!(
        report.scores[1].total_nanos,
        Some((4.0_f64 / 7.0).ln().mul_add(1e9, 0.0).round() as i64)
    );
    assert_eq!(report.scores[1].samples, 1);
    assert_eq!(report.scores[2].total_nanos, None);
    assert_eq!(report.model_sha256.len(), 64);
    assert_eq!(
        serde_json::to_value(report).unwrap()["normalization"],
        "ascii_letters"
    );
}

#[test]
fn scoring_rejects_ambiguous_models_and_honors_exact_work_and_cancellation() {
    let mut v = json!({"candidates":["AA","B"],"model":model(),"limits":{"max_work":2}});
    let r = scoring::score(&serde_json::from_value(v.clone()).unwrap(), None).unwrap();
    assert_eq!(r.scores.len(), 1);
    assert_eq!(r.status.work, 2);
    assert!(!r.status.complete);
    v["model"]["entries"][1]["text"] = json!("A");
    assert!(scoring::score(&serde_json::from_value(v).unwrap(), None).is_err());
    let request = serde_json::from_value(json!({"candidates":["AAA"],"model":model()})).unwrap();
    let flag = std::sync::atomic::AtomicBool::new(true);
    assert_eq!(
        scoring::score(&request, Some(&flag)).unwrap().status.stop,
        a::Stop::Cancelled
    );
}

#[test]
fn probability_and_dictionary_models_keep_units_and_unicode_explicit() {
    let request = serde_json::from_value(json!({"candidates":["éé","é!"],"model":{"kind":"probabilities","n":2,"entries":[{"text":"éé","weight":0.75}],"unseen_probability":0.25,"normalization":"exact"}})).unwrap();
    let r = scoring::score(&request, None).unwrap();
    assert_eq!(r.ranking, [0, 1]);
    assert_eq!(r.scores[0].samples, 1);
    assert_eq!(r.scores[1].total_nanos, Some(-1_386_294_361));
    let request=serde_json::from_value(json!({"candidates":["cat\u{2003}cat!"],"model":{"kind":"dictionary","entries":[{"text":"cat","weight":2.0}],"unseen_weight":-1.0}})).unwrap();
    assert_eq!(
        scoring::score(&request, None).unwrap().scores[0].total_nanos,
        Some(1_000_000_000)
    );
    assert!(a::parse_request::<scoring::Request>("{\"candidates\":[],\"candidates\":[]}").is_err());
}

#[test]
fn period_evidence_uses_ascii_positions_exact_counts_and_bounded_pairs() {
    let request = serde_json::from_value(
        json!({"text":"é AB!ABAB","periods":[2,1],"ngram":2,"expected_ic":0.5}),
    )
    .unwrap();
    let r = a::period::estimate(&request, None).unwrap();
    assert!(r.status.complete);
    assert_eq!(r.letters, 6);
    assert_eq!(r.ic, Some(0.4));
    assert_eq!(r.periods[0].mean_column_ic, Some(1.0));
    assert_eq!(r.periods[0].divisor_hits, 4);
    assert_eq!(r.repeats[0].positions, [0, 2, 4]);
    assert_eq!(r.repeats[0].original_byte_offsets, [3, 6, 8]);
    let tiny =
        serde_json::from_value(json!({"text":"A","periods":[1],"ngram":2,"expected_ic":0.065}))
            .unwrap();
    let r = a::period::estimate(&tiny, None).unwrap();
    assert_eq!(r.ic, None);
    assert_eq!(r.friedman, None);
    let limited=serde_json::from_value(json!({"text":"AAAAAAAAAA","periods":[1],"ngram":2,"expected_ic":0.065,"limits":{"max_work":20}})).unwrap();
    assert!(!a::period::estimate(&limited, None).unwrap().status.complete);
}

#[test]
fn a_uniform_language_model_cannot_supply_a_friedman_length() {
    let r = serde_json::from_value(
        json!({"text":"AAAA","periods":[1],"ngram":2,"expected_ic":1.0/26.0}),
    )
    .unwrap();
    assert_eq!(a::period::estimate(&r, None).unwrap().friedman, None);
}

#[test]
fn canonical_decimal_integer_inputs_match_the_published_schema() {
    let r:a::period::Request=a::parse_request(r#"{"text":"ABAB","periods":["2"],"ngram":"2","expected_ic":0.5,"limits":{"max_work":"100","max_results":"2"}}"#).unwrap();
    assert_eq!(a::period::estimate(&r, None).unwrap().periods[0].period, 2);
    for value in ["01", "+1", "1.0"] {
        let doc = json!({"text":"AB","periods":[value],"ngram":2,"expected_ic":0.5}).to_string();
        assert!(a::parse_request::<a::period::Request>(&doc).is_err());
    }
}

#[test]
fn wide_fixed_point_scores_validate_in_the_lossless_contract_wire_format() {
    let request=serde_json::from_value(json!({"candidates":["A".repeat(32768)],"model":{"kind":"probabilities","n":1,"entries":[{"text":"A","weight":1e-300}],"unseen_probability":1.0,"normalization":"exact"}})).unwrap();
    let r = scoring::score(&request, None).unwrap();
    assert_eq!(r.scores[0].total_nanos, Some(-690775527898_i64 * 32768));
    let output = cube_core::operations::contracts::canonical_value(json!({"report":r}));
    cube_core::operations::require("analysis.language.score")
        .unwrap()
        .contract()
        .unwrap()
        .validate_outputs(&output)
        .unwrap();
}
