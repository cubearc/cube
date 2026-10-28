use cube_core::cryptanalysis as a;
use serde_json::json;
#[test]
fn lowered_homophonic_retention_limit_is_honored() {
    let r = serde_json::from_value(json!({"tokens":["A","B"],"limits":{"max_results":1}})).unwrap();
    let result = a::homophonic::inspect(&r, None).unwrap();
    assert!(!result.status.complete);
    assert_eq!(result.observed.len(), 1);
}
#[test]
fn homophonic_tokens_are_not_unicode_characters_and_groups_are_hypotheses() {
    let r = serde_json::from_value(
        json!({"tokens":["12","雪","12","éé"],"groups":[{"symbol":"A","tokens":["12","雪"]}]}),
    )
    .unwrap();
    let result = a::homophonic::inspect(&r, None).unwrap();
    assert_eq!(result.observed[0].token, "12");
    assert_eq!(result.observed[0].count, 2);
    assert_eq!(result.token_ic.unwrap().numerator, 2);
    assert_eq!(result.hypotheses[0].count, 3);
    assert_eq!(result.unmapped, 1);
    assert_eq!(result.adjacency.len(), 3);
    assert!(result.status.complete);
    let invalid = serde_json::from_value(
        json!({"tokens":[],"groups":[{"symbol":"A","tokens":["x"]},{"symbol":"B","tokens":["x"]}]}),
    )
    .unwrap();
    assert!(a::homophonic::inspect(&invalid, None).is_err());
}
#[test]
fn crib_offsets_are_explicit_and_reencoding_checks_only_supplied_keys() {
    let r=serde_json::from_value(json!({"candidates":[{"data":{"kind":"text","text":"é Aa!"},"key":{"kind":"caesar","shift":1}}],"ciphertext":{"kind":"text","text":"é Bb!"},"crib":{"kind":"text","text":"aa"},"offset":0,"view":"ascii_letters"})).unwrap();
    let result = a::crib::verify(&r, None).unwrap();
    assert_eq!(result.candidates[0].all_match, Some(true));
    assert_eq!(result.candidates[0].reencode_matches, Some(true));
    assert_eq!(
        result.candidates[0].positions[0].original_byte_offset,
        Some(3)
    );
    let r=serde_json::from_value(json!({"candidates":[{"data":{"kind":"bytes","bytes":[0,255,2]}}],"crib":{"kind":"bytes","bytes":[255,3]},"offset":1,"view":"bytes"})).unwrap();
    let result = a::crib::verify(&r, None).unwrap();
    assert_eq!(result.candidates[0].all_match, Some(false));
    assert_eq!(
        result.candidates[0]
            .positions
            .iter()
            .map(|p| p.matched)
            .collect::<Vec<_>>(),
        [true, false]
    );
}
#[test]
fn crib_partial_out_of_range_empty_and_cancellation_never_claim_complete_matches() {
    for (crib, offset, expected) in [("", 0, None), ("ABC", 1, Some(false))] {
        let r=serde_json::from_value(json!({"candidates":[{"data":{"kind":"text","text":"ABC"}}],"crib":{"kind":"text","text":crib},"offset":offset,"view":"scalars"})).unwrap();
        assert_eq!(
            a::crib::verify(&r, None).unwrap().candidates[0].all_match,
            expected
        );
    }
    let r=serde_json::from_value(json!({"candidates":[{"data":{"kind":"text","text":"AAA"}}],"crib":{"kind":"text","text":"AAA"},"offset":0,"view":"scalars","limits":{"max_work":1}})).unwrap();
    let report = a::crib::verify(&r, None).unwrap();
    assert!(!report.status.complete);
    assert_eq!(report.candidates[0].all_match, None);
}
