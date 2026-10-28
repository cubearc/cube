use cube_core::cryptanalysis as a;
use serde_json::json;
fn model() -> serde_json::Value {
    json!({"kind":"counts","n":1,"entries":[{"text":"A","weight":20.0},{"text":"B","weight":1.0}],"smoothing":1.0,"normalization":"ascii_letters"})
}
#[test]
fn caesar_scores_decryptions_and_retains_all_duplicate_keys() {
    let r =
        serde_json::from_value(json!({"text":"Bb!é","shifts":[0,1,1],"model":model()})).unwrap();
    let result = a::classical::caesar(&r, None).unwrap();
    assert_eq!(result.candidates[0].plaintext, "Aa!é");
    assert_eq!(result.candidates[0].aliases.len(), 2);
    assert_eq!(result.attempted, 3);
    assert_eq!(result.status.work, 24);
    assert!(result.status.complete);
    let empty = serde_json::from_value(json!({"text":"","model":model()})).unwrap();
    let result = a::classical::caesar(&empty, None).unwrap();
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.candidates[0].aliases.len(), 26);
}
#[test]
fn affine_failures_are_charged_and_explicit_keys_recover_known_plaintext() {
    let r = serde_json::from_value(
        json!({"text":"IHHWVCSWFRCP","keys":[{"a":2,"b":0},{"a":5,"b":8}],"model":model()}),
    )
    .unwrap();
    let result = a::classical::affine(&r, None).unwrap();
    assert_eq!(result.rejected.len(), 1);
    assert_eq!(result.candidates[0].plaintext, "AFFINECIPHER");
    assert_eq!(result.attempted, 2);
    let r = serde_json::from_value(json!({"text":"B","model":model(),"limits":{"max_work":1}}))
        .unwrap();
    let result = a::classical::caesar(&r, None).unwrap();
    assert!(!result.status.complete);
    assert!(result.candidates.is_empty());
}
#[test]
fn vigenere_beam_recovers_explicit_period_and_never_claims_pruned_exhaustion() {
    let r = serde_json::from_value(
        json!({"text":"B-éC!BCBC","lengths":[2],"beam_width":1,"model":model()}),
    )
    .unwrap();
    let result = a::vigenere::rank(&r, None).unwrap();
    assert_eq!(result.ranking.candidates[0].plaintext, "A-éA!AAAA");
    assert_eq!(result.original_byte_offsets, [0, 4, 6, 7, 8, 9]);
    assert!(!result.ranking.status.complete);
    assert_eq!(result.ranking.status.stop, a::Stop::Pruned);
    assert_eq!(
        serde_json::to_value(result).unwrap()["status"]["complete"],
        false
    );
}
#[test]
fn transpositions_share_core_inverse_and_return_exact_scalar_provenance() {
    let r=serde_json::from_value(json!({"text":"ACEBDF","configurations":[{"kind":"columnar","order":[0,1]},{"kind":"columnar","order":[0,1]}],"model":model()})).unwrap();
    let result = a::transposition::rank(&r, None).unwrap();
    assert_eq!(result.candidates[0].plaintext, "ABCDEF");
    assert_eq!(result.candidates[0].aliases.len(), 2);
    assert_eq!(
        result.candidates[0].aliases[0].source_positions,
        [0, 3, 1, 4, 2, 5]
    );
}

#[test]
fn finite_search_limits_empty_aliases_and_cancelled_work_are_observable() {
    let empty = serde_json::from_value(json!({"text":"!","model":model()})).unwrap();
    let all = a::classical::affine(&empty, None).unwrap();
    assert_eq!(all.attempted, 312);
    assert_eq!(all.candidates[0].aliases.len(), 312);
    let flag = std::sync::atomic::AtomicBool::new(true);
    let r = serde_json::from_value(
        json!({"text":"B","shifts":[0,1],"model":model(),"limits":{"max_results":1}}),
    )
    .unwrap();
    assert_eq!(
        a::classical::caesar(&r, None).unwrap().status.stop,
        a::Stop::ResultLimit
    );
    assert_eq!(
        a::classical::caesar(&r, Some(&flag)).unwrap().status.stop,
        a::Stop::Cancelled
    );
    let too_many =
        serde_json::from_value(json!({"text":"B","shifts":vec![0;1025],"model":model()})).unwrap();
    assert!(a::classical::caesar(&too_many, None).is_err());
    let too_long =
        serde_json::from_value(json!({"text":"A".repeat(65537),"model":model()})).unwrap();
    assert!(a::classical::caesar(&too_long, None).is_err());
}

#[test]
fn vigenere_single_column_can_be_exhaustive_but_limits_and_invalid_periods_are_explicit() {
    let r =
        serde_json::from_value(json!({"text":"B","lengths":[1],"beam_width":26,"model":model()}))
            .unwrap();
    let result = a::vigenere::rank(&r, None).unwrap();
    assert_eq!(result.ranking.candidates.len(), 26);
    assert!(result.ranking.status.complete);
    let r=serde_json::from_value(json!({"text":"BCBC","lengths":[2],"beam_width":1,"model":model(),"limits":{"max_work":10}})).unwrap();
    assert_eq!(
        a::vigenere::rank(&r, None).unwrap().ranking.status.stop,
        a::Stop::WorkLimit
    );
    for (length, width) in [(0, 1), (65, 1), (1, 0), (1, 257)] {
        let r = serde_json::from_value(
            json!({"text":"B","lengths":[length],"beam_width":width,"model":model()}),
        )
        .unwrap();
        assert!(a::vigenere::rank(&r, None).is_err());
    }
}

#[test]
fn rail_and_full_grid_candidates_follow_existing_conventions() {
    for (text, configuration, expected) in [
        (
            "WECRLTEERDSOEEFEAOCAIVDEN",
            json!({"kind":"rail_fence","rails":3,"initial_rail":0,"direction":"down","readout":[0,1,2]}),
            "WEAREDISCOVEREDFLEEATONCE",
        ),
        (
            "ADBECF",
            json!({"kind":"route","rows":2,"columns":3,"fill":{"kind":"rows","corner":"top_left"},"read":{"kind":"columns","corner":"top_left"}}),
            "ABCDEF",
        ),
    ] {
        let r = serde_json::from_value(
            json!({"text":text,"configurations":[configuration],"model":model()}),
        )
        .unwrap();
        assert_eq!(
            a::transposition::rank(&r, None).unwrap().candidates[0].plaintext,
            expected
        );
    }
    let r = serde_json::from_value(
        json!({"text":"ABC","configurations":[{"kind":"columnar","order":[0,0]}],"model":model()}),
    )
    .unwrap();
    assert_eq!(a::transposition::rank(&r, None).unwrap().rejected.len(), 1);
}
