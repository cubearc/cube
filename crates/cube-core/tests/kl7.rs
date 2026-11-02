//! Independent source checks through the public typed operation executor.
use cube_core::operations::execution::{self, Budget};
use serde_json::{Value, json};
fn vector() -> Value {
    serde_json::from_str(include_str!("../src/ciphers/mechanical/kl7_vectors.json")).unwrap()
}
fn run(direction: &str, text: &str, settings: Value) -> Value {
    execution::execute(
        &format!("cipher.kl7.{direction}"),
        1,
        &json!({"text":text,"settings":settings}),
        &json!({}),
        Budget::default(),
    )
    .unwrap()
}
#[test]
fn complete_independent_source_answer_and_final_state_match() {
    let v = vector();
    let r = run("encode", v["text"].as_str().unwrap(), v["settings"].clone());
    assert_eq!(r["text"], v["expected"]);
    assert_eq!(r["final_state"], v["expected_final_state"]);
    assert_eq!(
        run(
            "decode",
            v["expected"].as_str().unwrap(),
            v["settings"].clone()
        )["text"],
        v["text"]
    );
}

#[test]
fn published_preparatory_cycle_is_explicit_and_the_fourth_rotor_stays_fixed() {
    let mut p = vector()["settings"].clone();
    p["initial_state"] = json!({"core_positions":[33,0,0,16,0,0,0,0]});
    let one = run("encode", "A", p);
    assert_eq!(one["final_state"], vector()["settings"]["initial_state"]);
    let r = run("encode", &"A".repeat(676), vector()["settings"].clone());
    for row in r["trace"].as_array().unwrap() {
        assert_eq!(row["state"]["core_positions"][3], 16);
        assert!((1..=11).contains(&row["feedback_passes"].as_u64().unwrap()));
    }
}
#[test]
fn every_reference_rotor_and_ring_supports_bijections_inverse_and_all_chunk_splits() {
    use std::collections::BTreeSet;
    let v = vector();
    let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZ";
    for shift in 0..13 {
        let names: Vec<String> = (0..8)
            .map(|i| char::from(b'A' + ((i + shift) % 13) as u8).to_string())
            .collect();
        let mut p = v["settings"].clone();
        p["rotors"] = json!(names);
        p["notch_rings"] = json!((0..7).map(|i| ((i + shift) % 11) + 1).collect::<Vec<_>>());
        p["notch_offsets"] = json!([35, 0, 7, 14, 21, 28, 35]);
        p["initial_state"] = json!({"core_positions":[35,1,2,3,4,5,6,7]});
        let whole = run("encode", text, p.clone());
        assert_eq!(
            run("decode", whole["text"].as_str().unwrap(), p.clone())["text"],
            text
        );
        assert_eq!(whole["final_state"]["core_positions"][3], 3);
        assert_eq!(
            run("encode", "", p.clone())["final_state"],
            p["initial_state"]
        );
        let outputs: BTreeSet<String> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
            .chars()
            .map(|c| {
                run("encode", &c.to_string(), p.clone())["text"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
            .collect();
        assert_eq!(outputs.len(), 26);
        if shift == 3 {
            for direction in ["encode", "decode"] {
                let input = if direction == "encode" {
                    text
                } else {
                    whole["text"].as_str().unwrap()
                };
                let expected = if direction == "encode" {
                    whole["text"].as_str().unwrap()
                } else {
                    text
                };
                for split in 0..=input.len() {
                    let first = run(direction, &input[..split], p.clone());
                    let mut next = p.clone();
                    next["initial_state"] = first["final_state"].clone();
                    let second = run(direction, &input[split..], next);
                    assert_eq!(
                        format!(
                            "{}{}",
                            first["text"].as_str().unwrap(),
                            second["text"].as_str().unwrap()
                        ),
                        expected
                    );
                    assert_eq!(second["final_state"], whole["final_state"]);
                }
            }
        }
    }
}
#[test]
fn incompatible_wheels_duplicate_notches_outside_contacts_and_private_text_fail() {
    let v = vector();
    for change in 0..8 {
        let mut p = v["settings"].clone();
        match change {
            0 => p["profile"] = json!("kl7_guessed"),
            1 => p["rotors"][0] = p["rotors"][1].clone(),
            2 => p["notch_rings"][0] = json!(0),
            3 => p["notch_rings"][0] = json!(12),
            4 => p["notch_rings"][0] = p["notch_rings"][1].clone(),
            5 => p["notch_offsets"][0] = json!(36),
            6 => p["initial_state"]["core_positions"][3] = json!(36),
            _ => p["initial_state"]["windows"] = json!("PRIVATE_WINDOWS"),
        };
        let e = execution::execute(
            "cipher.kl7.encode",
            1,
            &json!({"text":"A","settings":p}),
            &json!({}),
            Budget::default(),
        )
        .unwrap_err();
        assert!(!format!("{e:?}").contains("PRIVATE_WINDOWS"));
    }
    for text in ["a", "A A", "é", "PRIVATE_TEXT"] {
        let e = execution::execute(
            "cipher.kl7.encode",
            1,
            &json!({"text":text,"settings":v["settings"]}),
            &json!({}),
            Budget::default(),
        )
        .unwrap_err();
        assert!(!format!("{e:?}").contains("PRIVATE_TEXT"));
    }
}
#[test]
fn full_native_input_and_independent_trace_report_ceilings_never_truncate() {
    use cube_core::{
        ciphers::mechanical::{MAX_INPUT, MAX_TRACE, kl7},
        errors::ErrorCode,
    };
    let mut p = vector()["settings"].clone();
    p["trace"] = json!(false);
    let mut p: kl7::Settings = serde_json::from_value(p).unwrap();
    assert_eq!(
        kl7::encode(&"A".repeat(MAX_INPUT), &p).unwrap().text.len(),
        MAX_INPUT
    );
    assert_eq!(
        kl7::encode(&"A".repeat(MAX_INPUT + 1), &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    p.trace = true;
    let r = kl7::encode(&"A".repeat(16384), &p).unwrap();
    assert_eq!(r.trace.len(), 16384);
    assert!(serde_json::to_vec(&r).unwrap().len() <= 4194304);
    let bytes = kl7::encode(&"A".repeat(MAX_TRACE), &p).unwrap_err();
    assert_eq!(bytes.code, ErrorCode::ResourceLimit);
    assert_eq!(bytes.parameter, Some("report"));
    assert_eq!(
        kl7::encode(&"A".repeat(MAX_TRACE + 1), &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
