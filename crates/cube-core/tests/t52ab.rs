//! Siemens T52a/b five-bit XOR and the actual pluggable two-contact network.
use cube_core::operations::execution::{self, Budget};
use serde_json::{Value, json};
fn settings() -> Value {
    let lengths = [47, 53, 59, 61, 64, 65, 67, 69, 71, 73];
    let mut pins: Vec<String> = lengths.iter().map(|n| "0".repeat(*n)).collect();
    pins[0] = "1".repeat(lengths[0]);
    // Permutation unit1 inactive; units2..5 active. All contacts stay constant
    // for this hand-laid circuit check, not an asserted historical factory set.
    for wheel in 6..10 {
        pins[wheel] = "1".repeat(lengths[wheel]);
    }
    json!({"profile":"siemens_t52ab_configured_v1","wheel_set_id":"synthetic-figure8-hand-check","cam_patterns":pins,"wheel_order":[0,1,2,3,4,5,6,7,8,9],"socket_pairs":[[1,3],[2,10],[4,5],[6,7],[8,9]],"initial_state":[0,0,0,0,0,0,0,0,0,0],"trace":true})
}
fn run(direction: &str, codes: Value, p: Value) -> Value {
    execution::execute(
        &format!("cipher.t52ab.{direction}"),
        1,
        &json!({"codes":codes,"settings":p}),
        &json!({}),
        Budget::default(),
    )
    .unwrap()
}
#[test]
fn figure8_one_three_inactive_swaps_impulses_one_five_after_xor() {
    // Weierud,Figure8,p21: inactive1–3 routes impulse1 to5. The other plug
    // choices complete the reverse path5→1. XOR1 must precede this exchange.
    let input = json!([0, 1, 2, 4, 8, 16, 31]);
    let expected = json!([16, 0, 18, 20, 24, 17, 15]);
    let result = run("encode", input.clone(), settings());
    assert_eq!(result["codes"], expected);
    assert_eq!(result["trace"][0]["permutation"], json!([4, 1, 2, 3, 0]));
    assert_eq!(result["final_state"], json!(vec![7; 10]));
    assert_eq!(run("decode", expected, settings())["codes"], input);
}
fn matchings(remaining: &[u8], current: &mut Vec<[u8; 2]>, result: &mut Vec<Vec<[u8; 2]>>) {
    if remaining.is_empty() {
        result.push(current.clone());
        return;
    }
    for index in 1..remaining.len() {
        current.push([remaining[0], remaining[index]]);
        let next: Vec<u8> = remaining
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != 0 && *i != index)
            .map(|(_, v)| *v)
            .collect();
        matchings(&next, current, result);
        current.pop();
    }
}
#[test]
fn all_945_physical_pairings_reproduce_the_published_switch_statistics() {
    use cube_core::ciphers::mechanical::t52ab;
    use std::collections::{BTreeMap, BTreeSet};
    let mut variants = Vec::new();
    matchings(
        &(1..=10).collect::<Vec<_>>(),
        &mut Vec::new(),
        &mut variants,
    );
    assert_eq!(variants.len(), 945);
    let mut p = settings();
    for (wheel, length) in t52ab::WHEEL_LENGTHS.iter().enumerate() {
        p["cam_patterns"][wheel] = json!(
            (0..*length)
                .map(|i| if wheel >= 5 && i < 32 && i & (1 << (wheel - 5)) != 0 {
                    '1'
                } else {
                    '0'
                })
                .collect::<String>()
        );
    }
    let mut counts = BTreeMap::new();
    for pairs in variants {
        p["socket_pairs"] = json!(pairs);
        let typed: t52ab::Settings = serde_json::from_value(p.clone()).unwrap();
        let result = t52ab::encode(&[0; 32], &typed).unwrap();
        let maps: BTreeSet<[u8; 5]> = result.trace.iter().map(|r| r.permutation).collect();
        for map in &maps {
            assert_eq!(
                map.iter().copied().collect::<BTreeSet<_>>(),
                BTreeSet::from([0, 1, 2, 3, 4])
            );
        }
        *counts.entry(maps.len()).or_insert(0usize) += 1;
    }
    // Weierud pp22–23/Figure10, independently enumerated from the source
    // physical circuit rather than a sequential five-swap approximation.
    assert_eq!(counts[&27], 24);
    assert_eq!(counts[&30], 240);
    assert_eq!(counts[&32], 120);
    assert_eq!(
        counts
            .iter()
            .filter(|(n, _)| **n <= 16)
            .map(|(_, count)| count)
            .sum::<usize>(),
        561
    );
    assert_eq!(counts[&1], 1);
}
#[test]
fn all_codes_have_exact_inverse_empty_identity_wraps_and_every_chunk_split() {
    let mut p = settings();
    let lengths = [47, 53, 59, 61, 64, 65, 67, 69, 71, 73];
    for (wheel, length) in lengths.iter().enumerate() {
        p["cam_patterns"][wheel] = json!(
            (0..*length)
                .map(|i| if (i + wheel) % 3 == 0 { '1' } else { '0' })
                .collect::<String>()
        );
        p["initial_state"][wheel] = json!(length - 1);
    }
    p["wheel_order"] = json!([9, 0, 7, 2, 5, 4, 3, 6, 1, 8]);
    let codes: Vec<u8> = (0..96).map(|i| i % 32).collect();
    let result = run("encode", json!(codes), p.clone());
    let decoded = run("decode", result["codes"].clone(), p.clone());
    assert_eq!(decoded["codes"], json!(codes));
    assert_eq!(decoded["final_state"], result["final_state"]);
    assert_eq!(
        run("encode", json!([]), p.clone())["final_state"],
        p["initial_state"]
    );
    for split in 0..=codes.len() {
        let first = run("encode", json!(codes[..split]), p.clone());
        let mut next = p.clone();
        next["initial_state"] = first["final_state"].clone();
        let second = run("encode", json!(codes[split..]), next);
        let joined: Vec<Value> = first["codes"]
            .as_array()
            .unwrap()
            .iter()
            .chain(second["codes"].as_array().unwrap())
            .cloned()
            .collect();
        assert_eq!(json!(joined), result["codes"]);
        assert_eq!(second["final_state"], result["final_state"]);
    }
}
#[test]
fn duplicate_plugs_unknown_modes_and_malformed_contacts_fail_without_echo() {
    use cube_core::{ciphers::mechanical::t52ab, errors::ErrorCode};
    let changes = [
        ("wheel_set_id", json!("")),
        ("wheel_set_id", json!("PRIVATE KEY")),
        ("profile", json!("siemens_t52d")),
        ("kt", json!(true)),
        ("wheel_order", json!(vec![0; 10])),
        ("wheel_order", json!([255, 1, 2, 3, 4, 5, 6, 7, 8, 9])),
        (
            "socket_pairs",
            json!([[1, 1], [2, 3], [4, 5], [6, 7], [8, 9]]),
        ),
        (
            "socket_pairs",
            json!([[1, 11], [2, 3], [4, 5], [6, 7], [8, 9]]),
        ),
        ("initial_state", json!([47, 0, 0, 0, 0, 0, 0, 0, 0, 0])),
    ];
    for (field, value) in changes {
        let mut p = settings();
        p[field] = value;
        let e = execution::execute(
            "cipher.t52ab.encode",
            1,
            &json!({"codes":[0],"settings":p}),
            &json!({}),
            Budget::default(),
        )
        .unwrap_err();
        assert!(!format!("{e:?}").contains("PRIVATE KEY"));
    }
    for codes in [
        json!([32]),
        json!([-1]),
        json!([true]),
        json!(["PRIVATE_CODE"]),
    ] {
        let e = execution::execute(
            "cipher.t52ab.encode",
            1,
            &json!({"codes":codes,"settings":settings()}),
            &json!({}),
            Budget::default(),
        )
        .unwrap_err();
        assert!(!format!("{e:?}").contains("PRIVATE_CODE"));
    }
    for (field, value) in [
        (
            "cam_patterns",
            json!([
                "0".repeat(47),
                "x".repeat(53),
                "0".repeat(59),
                "0".repeat(61),
                "0".repeat(64),
                "0".repeat(65),
                "0".repeat(67),
                "0".repeat(69),
                "0".repeat(71),
                "0".repeat(73)
            ]),
        ),
        ("wheel_set_id", json!("bad space")),
        ("wheel_set_id", json!("A".repeat(129))),
        ("initial_state", json!([47, 0, 0, 0, 0, 0, 0, 0, 0, 0])),
    ] {
        let mut p = settings();
        p[field] = value;
        let p: t52ab::Settings = serde_json::from_value(p).unwrap();
        assert_eq!(
            t52ab::encode(&[0], &p).unwrap_err().code,
            ErrorCode::InvalidParameter
        );
    }
}
#[test]
fn direct_code_trace_and_report_budgets_remain_independent() {
    use cube_core::{
        ciphers::mechanical::{MAX_INPUT, MAX_TRACE, t52ab},
        errors::ErrorCode,
    };
    let mut p = settings();
    p["trace"] = json!(false);
    let plain: t52ab::Settings = serde_json::from_value(p.clone()).unwrap();
    assert_eq!(
        t52ab::encode(&vec![31; MAX_INPUT], &plain)
            .unwrap()
            .codes
            .len(),
        MAX_INPUT
    );
    assert_eq!(
        t52ab::encode(&vec![31; MAX_INPUT + 1], &plain)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        t52ab::encode(&[32], &plain).unwrap_err().code,
        ErrorCode::InvalidInput
    );
    p["trace"] = json!(true);
    let traced = serde_json::from_value(p).unwrap();
    assert_eq!(
        t52ab::encode(&vec![0; 10000], &traced).unwrap().trace.len(),
        10000
    );
    assert_eq!(
        t52ab::encode(&vec![0; MAX_TRACE + 1], &traced)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        t52ab::encode(&vec![0; MAX_TRACE], &traced)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
