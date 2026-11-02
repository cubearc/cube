//! Independent published contact check through the public executor.
use cube_core::operations::execution::{self, Budget};
use serde_json::{Value, json};
fn vector() -> Value {
    serde_json::from_str(include_str!(
        "../src/ciphers/mechanical/fialka_vectors.json"
    ))
    .unwrap()
}
fn run(direction: &str, input: Value, settings: Value) -> Value {
    execution::execute(
        &format!("cipher.fialka.{direction}"),
        1,
        &json!({"codes":input,"settings":settings}),
        &json!({}),
        Budget::default(),
    )
    .unwrap()
}
#[test]
fn independent_published_contact_answer_and_inverse() {
    let v = vector();
    let r = run("encode", v["codes"].clone(), v["settings"].clone());
    assert_eq!(r["codes"], v["expected"]);
    assert_eq!(r["final_state"], v["expected_final_state"]);
    assert_eq!(
        run("decode", v["expected"].clone(), v["settings"].clone())["codes"],
        v["codes"]
    );
}

#[test]
fn continuation_at_every_split_and_empty_preserve_complete_state() {
    let v = vector();
    let length = v["codes"].as_array().unwrap().len();
    let whole = run("encode", v["codes"].clone(), v["settings"].clone());
    assert_eq!(
        run("encode", json!([]), v["settings"].clone())["final_state"],
        v["settings"]["initial_state"]
    );
    for direction in ["encode", "decode"] {
        let input = if direction == "encode" {
            v["codes"].clone()
        } else {
            v["expected"].clone()
        };
        let expected = if direction == "encode" {
            v["expected"].clone()
        } else {
            v["codes"].clone()
        };
        for split in 0..=length {
            let (left, right) = (
                json!(&input.as_array().unwrap()[..split]),
                json!(&input.as_array().unwrap()[split..]),
            );
            let first = run(direction, left, v["settings"].clone());
            let mut next = v["settings"].clone();
            next["initial_state"] = first["final_state"].clone();
            let second = run(direction, right, next);
            let joined = {
                let mut a = first["codes"].as_array().unwrap().clone();
                a.extend(second["codes"].as_array().unwrap().clone());
                json!(a)
            };
            assert_eq!(joined, expected, "split{split}");
            assert_eq!(second["final_state"], whole["final_state"]);
        }
    }
}
#[test]
fn complete_contact_bijection_with_nonzero_settings() {
    use std::collections::BTreeSet;
    let v = vector();
    let mut p = v["settings"].clone();
    p["ring_offsets"] = json!([29, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    p["rotor_order"] = json!([10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    p["punch_card"] = json!((0..30).map(|x| (x + 7) % 30).collect::<Vec<_>>());
    let mut outputs = BTreeSet::new();
    for x in 0..30 {
        let input = json!([x]);
        let r = run("encode", input.clone(), p.clone());
        outputs.insert(r["codes"].to_string());
        assert_eq!(run("decode", r["codes"].clone(), p.clone())["codes"], input);
    }
    assert_eq!(outputs.len(), 30);
}
#[test]
fn unsupported_profiles_unknown_private_fields_and_bad_state_fail_without_echo() {
    let v = vector();
    for change in 0..4 {
        let mut p = v["settings"].clone();
        match change {
            0 => p["profile"] = json!("PRIVATE_UNKNOWN"),
            1 => p["PRIVATE_UNKNOWN"] = json!("PRIVATE_codes"),
            2 => p["initial_state"]["rotor_positions"][0] = json!(30),
            _ => p["punch_card"][0] = p["punch_card"][1].clone(),
        };
        let e = execution::execute(
            "cipher.fialka.encode",
            1,
            &json!({"codes":v["codes"],"settings":p}),
            &json!({}),
            Budget::default(),
        )
        .unwrap_err();
        let rendered = format!("{e:?}");
        assert!(!rendered.contains("PRIVATE"));
    }
}

#[test]
fn full_input_and_independent_trace_count_and_report_budgets() {
    use cube_core::{
        ciphers::mechanical::{MAX_INPUT, MAX_TRACE, fialka},
        errors::ErrorCode,
    };
    let mut p: fialka::Settings = serde_json::from_value(vector()["settings"].clone()).unwrap();
    p.trace = false;
    assert_eq!(
        fialka::encode(&vec![0u8; MAX_INPUT], &p)
            .unwrap()
            .codes
            .len(),
        MAX_INPUT
    );
    assert_eq!(
        fialka::encode(&vec![0u8; MAX_INPUT + 1], &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    p.trace = true;
    let r = fialka::encode(&vec![0u8; 16384], &p).unwrap();
    assert_eq!(r.trace.len(), 16384);
    assert!(serde_json::to_vec(&r).unwrap().len() <= 4194304);
    let e = fialka::encode(&vec![0u8; MAX_TRACE], &p).unwrap_err();
    assert_eq!(e.code, ErrorCode::ResourceLimit);
    assert_eq!(e.parameter, Some("report"));
    assert_eq!(
        fialka::encode(&vec![0u8; MAX_TRACE + 1], &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn complete_chernov_full_contact_checks_cover_card_and_nonzero_rotors() {
    let cases: Vec<Value> = serde_json::from_str(include_str!(
        "../src/ciphers/mechanical/fialka_additional_vectors.json"
    ))
    .unwrap();
    for v in cases {
        let mut p = vector()["settings"].clone();
        p["initial_state"]["rotor_positions"] = v["positions"].clone();
        p["punch_card"] = v["card"].clone();
        let r = run("encode", v["codes"].clone(), p.clone());
        assert_eq!(r["codes"], v["expected"], "source case{}", v["case"]);
        assert_eq!(run("decode", v["expected"].clone(), p)["codes"], v["codes"]);
    }
}
