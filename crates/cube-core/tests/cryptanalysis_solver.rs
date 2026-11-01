use cube_core::operations::execution::{self as e, Budget};
use serde_json::{Value, json};

fn invoke(op: &str, request: Value) -> Value {
    e::invoke(
        op,
        1,
        &json!({"request":request}),
        &json!({}),
        Budget::default(),
    )
    .unwrap()["report"]
        .clone()
}
#[test]
fn substitution_retains_dictionary_sources_and_unused_key_multiplicity() {
    let r = invoke(
        "analysis.substitution.solve",
        json!({
            "profile":"observed_bijection_v1", "alphabet":"ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            "ciphertext":"ABBA", "outside":"preserve",
            "dictionary":{"segmentation":"unicode_whitespace", "words":["NOON","DEED","NOON"]}
        }),
    );
    assert_eq!(r["status"]["complete"], true);
    assert_eq!(r["candidates"][0]["plaintext"], "NOON");
    assert_eq!(r["candidates"][1]["plaintext"], "DEED");
    assert_eq!(r["candidates"][0]["key_count"], "620448401733239439360000"); // 24!
    assert_eq!(r["dictionary_classes"][0]["input_indices"], json!([0, 2]));
    assert_eq!(r["contradiction"], false);
}
#[test]
fn family_evidence_keeps_plaintext_and_cipher_ambiguity() {
    let text = "THE READER CAN CHECK EVERY CLAIM AND COMPARE THE LETTER COUNTS WITH THE ORIGINAL MESSAGE. ".repeat(8);
    let r = invoke(
        "analysis.family.suggest",
        json!({"profile":"ascii_classical_evidence_v1", "text":text}),
    );
    assert_eq!(r["status"]["complete"], true);
    assert_eq!(r["suggestions"][0]["rule"], "nonuniform_coincidence_v1");
    assert_eq!(
        r["suggestions"][0]["compatible_families"],
        json!(["plaintext", "monoalphabetic_substitution", "transposition"])
    );
}

use cube_core::cryptanalysis::{self as a, family as f, substitution as s};
use std::sync::atomic::AtomicBool;
fn base() -> Value {
    json!({"profile":"observed_bijection_v1","alphabet":"ABCDEFGHIJKLMNOPQRSTUVWXYZ","ciphertext":"ABBA","outside":"preserve","dictionary":{"segmentation":"unicode_whitespace","words":["NOON","DEED","NOON"]}})
}
fn solve(v: Value) -> s::Report {
    s::solve(&a::parse_request(&v.to_string()).unwrap(), None).unwrap()
}
fn suggest(v: Value) -> f::Report {
    f::suggest(&a::parse_request(&v.to_string()).unwrap(), None).unwrap()
}
#[test]
fn permutation_oracle_and_full_key_roundtrip() {
    for free in 0..=3 {
        let mut v = base();
        v.as_object_mut().unwrap().remove("dictionary");
        v["ciphertext"] = json!("XYZ");
        let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        v["cribs"] = json!(
            alphabet
                .chars()
                .take(26 - free)
                .map(|c| json!({"cipher":c,"plain":c}))
                .collect::<Vec<_>>()
        );
        let r = solve(v.clone());
        let factorial = (1..=free).product::<usize>();
        assert!(r.status.complete);
        assert_eq!(r.candidates.len(), factorial);
        for c in &r.candidates {
            assert_eq!(c.key_count, "1");
            let mapping =
                cube_core::ciphers::mono::alphabet::Mapping::new(alphabet, &c.canonical_key)
                    .unwrap();
            assert_eq!(mapping.decode(&c.plaintext, "reject").unwrap(), "XYZ");
        }
        assert_eq!(r.plaintext_unique, Some(free < 2));
        assert_eq!(r.key_unique, Some(free < 2));
        let again = solve(v);
        assert_eq!(
            serde_json::to_value(r).unwrap(),
            serde_json::to_value(again).unwrap()
        );
    }
}
#[test]
fn unicode_64_alphabet_aliases_literals_and_exact_dictionary_tokens() {
    let alphabet: String = (0x400..0x440).map(|n| char::from_u32(n).unwrap()).collect();
    let chars: Vec<_> = alphabet.chars().collect();
    let token = format!("{}{}{}{}!", chars[0], chars[1], chars[1], chars[0]);
    let word = format!("{}{}{}{}!", chars[62], chars[63], chars[63], chars[62]);
    let v = json!({"profile":"observed_bijection_v1","alphabet":alphabet,"ciphertext":format!("{token}\u{2003}{token}"),"outside":"preserve","dictionary":{"segmentation":"unicode_whitespace","words":[word,word]}});
    let r = solve(v);
    assert!(r.status.complete);
    assert_eq!(r.candidates.len(), 1);
    assert_eq!(r.candidates[0].free_cipher.chars().count(), 62);
    assert_eq!(
        r.constraints[0].original_byte_offsets,
        vec![0, token.len() + 3]
    );
    assert_eq!(r.candidates[0].word_choices, vec![0]);
    assert_eq!(r.dictionary_classes[0].input_indices, vec![0, 1]);
    let mut v = base();
    v["ciphertext"] = json!("αBBα\nABBA");
    v["aliases"] = json!([{"alias":"α","symbol":"A"}]);
    let r = solve(v);
    assert_eq!(r.aliases[0].occurrences, 2);
    assert_eq!(r.candidates[0].plaintext, "NOON\nNOON");
    assert_eq!(r.constraints[0].original_byte_offsets, vec![0, 7]);
}
#[test]
fn contradictions_do_not_hide_invalid_cribs_or_incomplete_search() {
    let mut v = base();
    v["cribs"] = json!([{"cipher":"A","plain":"N"},{"cipher":"A","plain":"D"}]);
    assert!(solve(v.clone()).contradiction);
    v["cribs"] = json!([{"cipher":"A","plain":"N"},{"cipher":"B","plain":"N"}]);
    assert!(solve(v).contradiction);
    let mut v = base();
    v["dictionary"]["words"] = json!(["NONE"]);
    let r = solve(v);
    assert!(r.contradiction && r.status.complete);
    let mut v = base();
    v["limits"] = json!({"max_work":0});
    let r = solve(v);
    assert!(!r.contradiction);
    assert_eq!(r.plaintext_unique, None);
    assert_eq!(r.key_unique, None);
    assert_eq!(r.status.stop, a::Stop::WorkLimit);
    let request = a::parse_request(&base().to_string()).unwrap();
    let r = s::solve(&request, Some(&AtomicBool::new(true))).unwrap();
    assert_eq!(r.status.stop, a::Stop::Cancelled);
    assert!(!r.contradiction);
}
#[test]
fn exact_work_and_result_boundaries_preserve_completion_and_scores() {
    let mut v = base();
    let work = solve(v.clone()).status.work;
    v["limits"] = json!({"max_work":work});
    assert!(solve(v.clone()).status.complete);
    v["limits"]["max_work"] = json!(work - 1);
    assert_eq!(solve(v.clone()).status.stop, a::Stop::WorkLimit);
    v["limits"] = json!({"max_results":2});
    assert!(solve(v.clone()).status.complete);
    v["limits"]["max_results"] = json!(1);
    let r = solve(v.clone());
    assert_eq!(r.status.stop, a::Stop::ResultLimit);
    assert_eq!(r.plaintext_unique, None);
    assert_eq!(r.key_unique, Some(false));
    v["limits"] = json!({});
    v["model"] =
        json!({"kind":"dictionary","entries":[{"text":"DEED","weight":2.0}],"unseen_weight":-1.0});
    let r = solve(v);
    assert_eq!(r.ranking, vec![1, 0]);
    assert!(r.model_sha256.is_some());
}
#[test]
fn empty_and_fully_literal_inputs_have_exact_classes() {
    for text in ["", "!", " \n"] {
        let mut v = base();
        v.as_object_mut().unwrap().remove("dictionary");
        v["ciphertext"] = json!(text);
        let r = solve(v);
        assert!(r.status.complete);
        assert_eq!(r.candidates[0].plaintext, text);
        assert_eq!(r.candidates[0].key_count, "403291461126605635584000000");
        assert_eq!(r.plaintext_unique, Some(true));
        assert_eq!(r.key_unique, Some(false));
    }
    let mut v = base();
    v["ciphertext"] = json!("!!!");
    v["dictionary"]["words"] = json!(["!!!"]);
    assert_eq!(solve(v).candidates.len(), 1);
}
#[test]
fn invalid_profiles_models_and_structural_ceiling_are_typed() {
    for (key, value) in [
        ("alphabet", json!("A".repeat(26))),
        ("alphabet", json!("ABC")),
        ("alphabet", json!("ABCDEFGHIJKLMNOPQRSTUVWXYZ ")),
        ("alphabet", json!("A".repeat(65))),
        ("ciphertext", json!("A".repeat(65537))),
        ("cribs", json!([{"cipher":"!","plain":"A"}])),
        ("aliases", json!([{"alias":"A","symbol":"B"}])),
        ("aliases", json!([{"alias":"α","symbol":"!"}])),
        (
            "aliases",
            json!([{"alias":"α","symbol":"A"},{"alias":"α","symbol":"B"}]),
        ),
        (
            "dictionary",
            json!({"segmentation":"unicode_whitespace","words":[""]}),
        ),
        (
            "dictionary",
            json!({"segmentation":"unicode_whitespace","words":["A B"]}),
        ),
        (
            "dictionary",
            json!({"segmentation":"unicode_whitespace","words":["A".repeat(257)]}),
        ),
        (
            "dictionary",
            json!({"segmentation":"unicode_whitespace","words":vec!["A";4097]}),
        ),
        ("limits", json!({"max_work":1048577})),
        ("limits", json!({"max_results":0})),
        ("limits", json!({"max_memory_bytes":0})),
        ("limits", json!({"max_memory_bytes":67108865})),
        (
            "model",
            json!({"kind":"dictionary","entries":[{"text":"x x","weight":1.0}],"unseen_weight":0.0}),
        ),
    ] {
        let mut v = base();
        v[key] = value;
        let r = a::parse_request(&v.to_string()).unwrap();
        assert!(s::solve(&r, None).is_err(), "{key}");
    }
    let mut v = base();
    v["outside"] = json!("reject");
    v["ciphertext"] = json!("A!");
    assert!(s::solve(&a::parse_request(&v.to_string()).unwrap(), None).is_err());
    let mut v = base();
    let memory = solve(v.clone()).workspace_bytes_upper_bound;
    v["limits"] = json!({"max_memory_bytes":memory-1});
    assert!(s::solve(&a::parse_request(&v.to_string()).unwrap(), None).is_err());
}
#[test]
fn family_short_binary_unicode_transport_and_uniform_are_uncertain() {
    let inputs = [
        "".into(),
        "A".repeat(1000),
        "AB".repeat(1000),
        "48656C6C6F".repeat(20),
        "éTHE MESSAGE".repeat(100),
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ".repeat(100),
        "YWJjZGVmZ2hpamtsbW5vcHFyc3R1dnd4eXo=".repeat(20),
    ];
    for text in inputs {
        let r = suggest(json!({"profile":"ascii_classical_evidence_v1","text":text}));
        assert!(r.status.complete);
        assert!(r.suggestions.is_empty(), "{:?}", r.observations);
    }
    let r = suggest(
        json!({"profile":"ascii_classical_evidence_v1","bytes":(0..=255).collect::<Vec<_>>()}),
    );
    assert!(r.suggestions.is_empty());
    assert!(
        r.eligibility_failures
            .contains(&"binary_control_bytes".into())
    );
}
#[test]
fn family_repeating_key_has_qualified_column_evidence_and_limits() {
    let plain="THE READER CAN CHECK EVERY CLAIM AND COMPARE THE LETTER COUNTS WITH THE ORIGINAL MESSAGE. ".repeat(32);
    let cipher = cube_core::ciphers::vigenere_encode(&plain, "KEY").unwrap();
    let mut v = json!({"profile":"ascii_classical_evidence_v1","text":cipher,"periods":[3]});
    let r = suggest(v.clone());
    assert!(
        r.suggestions
            .iter()
            .any(|s| s.rule == f::Rule::PeriodicColumnsV1)
    );
    let work = r.status.work;
    v["limits"] = json!({"max_work":work});
    assert!(suggest(v.clone()).status.complete);
    v["limits"]["max_work"] = json!(work - 1);
    assert_eq!(suggest(v.clone()).status.stop, a::Stop::WorkLimit);
    v["limits"]["max_work"] = json!(0);
    let r = suggest(v.clone());
    assert!(r.observations.is_none());
    let r = f::suggest(
        &a::parse_request(&v.to_string()).unwrap(),
        Some(&AtomicBool::new(true)),
    )
    .unwrap();
    assert_eq!(r.status.stop, a::Stop::Cancelled);
    for invalid in [
        json!({"periods":[1]}),
        json!({"periods":[3,3]}),
        json!({"rules":["periodic_columns_v1","periodic_columns_v1"]}),
        json!({"bytes":[1,2]}),
    ] {
        let mut value = json!({"profile":"ascii_classical_evidence_v1","text":plain});
        for (k, x) in invalid.as_object().unwrap() {
            value[k] = x.clone();
        }
        assert!(f::suggest(&a::parse_request(&value.to_string()).unwrap(), None).is_err());
    }
}

#[test]
fn shared_dictionary_constraints_propagate_all_different_targets() {
    let mut v = base();
    v["ciphertext"] = json!("AB AC");
    v["dictionary"]["words"] = json!(["NO", "NP", "NO"]);
    let r = solve(v.clone());
    assert!(r.status.complete);
    assert_eq!(
        r.candidates
            .iter()
            .map(|c| c.plaintext.as_str())
            .collect::<Vec<_>>(),
        vec!["NO NP", "NP NO"]
    );
    v["cribs"] = json!([{"cipher":"A","plain":"O"}]);
    assert!(solve(v).contradiction);
}
#[test]
fn retained_report_limit_is_explicit_and_serializable() {
    let alphabet: String = (0x1f600..0x1f640)
        .map(|n| char::from_u32(n).unwrap())
        .collect();
    let text = alphabet.chars().next().unwrap().to_string().repeat(65536);
    let v = json!({"profile":"observed_bijection_v1","alphabet":alphabet,"ciphertext":text,"outside":"reject","limits":{"max_results":4096}});
    let r = solve(v);
    assert_eq!(r.status.stop, a::Stop::ResultLimit);
    assert!(!r.candidates.is_empty());
    assert!(r.candidates.len() < 64);
    assert!(!r.contradiction);
    assert!(serde_json::to_vec(&r).unwrap().len() < 4 << 20);
}
#[test]
fn family_retention_does_not_report_omitted_rules_as_complete() {
    let plain="THE READER CAN CHECK EVERY CLAIM AND COMPARE THE LETTER COUNTS WITH THE ORIGINAL MESSAGE. ".repeat(32);
    let cipher = cube_core::ciphers::vigenere_encode(&plain, "AB").unwrap();
    let mut v = json!({"profile":"ascii_classical_evidence_v1","text":cipher,"periods":[2]});
    let r = suggest(v.clone());
    assert_eq!(r.suggestions.len(), 2);
    v["limits"] = json!({"max_results":1});
    let r = suggest(v);
    assert_eq!(r.status.stop, a::Stop::ResultLimit);
    assert_eq!(r.suggestions.len(), 1);
}
