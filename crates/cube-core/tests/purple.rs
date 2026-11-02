//! Independent published contact check through the public executor.
use cube_core::operations::execution::{self, Budget};
use serde_json::{Value, json};
fn vector() -> Value {
    serde_json::from_str(include_str!(
        "../src/ciphers/mechanical/purple_vectors.json"
    ))
    .unwrap()
}
fn run(direction: &str, input: Value, settings: Value) -> Value {
    execution::execute(
        &format!("cipher.purple.{direction}"),
        1,
        &json!({"text":input,"settings":settings}),
        &json!({}),
        Budget::default(),
    )
    .unwrap()
}
#[test]
fn independent_published_contact_answer_and_inverse() {
    let v = vector();
    let r = run("encode", v["text"].clone(), v["settings"].clone());
    assert_eq!(r["text"], v["expected"]);
    assert_eq!(r["final_state"], v["expected_final_state"]);
    assert_eq!(
        run("decode", v["expected"].clone(), v["settings"].clone())["text"],
        v["text"]
    );
}

#[test]
fn latched_carries_select_exactly_one_twenty_for_all_six_speed_assignments() {
    for fast in 1..=3 {
        for middle in 1..=3 {
            if fast == middle {
                continue;
            }
            let slow = 6 - fast - middle;
            let mut p = vector()["settings"].clone();
            p["fast_switch"] = json!(fast);
            p["middle_switch"] = json!(middle);
            let mut positions = [0u8; 4];
            positions[0] = 23;
            positions[middle] = 24;
            p["initial_state"] = json!({"switch_positions":positions});
            let r = run("encode", json!("AAA"), p);
            positions[0] = 24;
            positions[slow] = 1;
            assert_eq!(r["trace"][0]["state"]["switch_positions"], json!(positions));
            assert_eq!(r["trace"][0]["advanced_switch"], slow);
            positions[0] = 0;
            positions[middle] = 0;
            assert_eq!(r["trace"][1]["state"]["switch_positions"], json!(positions));
            assert_eq!(r["trace"][1]["advanced_switch"], middle);
            positions[0] = 1;
            positions[fast] = 1;
            assert_eq!(r["final_state"]["switch_positions"], json!(positions));
            assert_eq!(r["trace"][2]["advanced_switch"], fast);
        }
    }
}

#[test]
fn continuation_at_every_split_and_empty_preserve_complete_state() {
    let v = vector();
    let length = v["text"].as_str().unwrap().len();
    let whole = run("encode", v["text"].clone(), v["settings"].clone());
    assert_eq!(
        run("encode", json!(""), v["settings"].clone())["final_state"],
        v["settings"]["initial_state"]
    );
    for direction in ["encode", "decode"] {
        let input = if direction == "encode" {
            v["text"].clone()
        } else {
            v["expected"].clone()
        };
        let expected = if direction == "encode" {
            v["expected"].clone()
        } else {
            v["text"].clone()
        };
        for split in 0..=length {
            let (left, right) = (
                json!(&input.as_str().unwrap()[..split]),
                json!(&input.as_str().unwrap()[split..]),
            );
            let first = run(direction, left, v["settings"].clone());
            let mut next = v["settings"].clone();
            next["initial_state"] = first["final_state"].clone();
            let second = run(direction, right, next);
            let joined = json!(format!(
                "{}{}",
                first["text"].as_str().unwrap(),
                second["text"].as_str().unwrap()
            ));
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
    p["alphabet"] = json!("ZYXWVUTSRQPONMLKJIHGFEDCBA");
    let mut outputs = BTreeSet::new();
    for x in 0..26 {
        let input = json!(char::from(b'A' + x as u8).to_string());
        let r = run("encode", input.clone(), p.clone());
        outputs.insert(r["text"].to_string());
        assert_eq!(run("decode", r["text"].clone(), p.clone())["text"], input);
    }
    assert_eq!(outputs.len(), 26);
}
#[test]
fn unsupported_profiles_unknown_private_fields_and_bad_state_fail_without_echo() {
    let v = vector();
    for change in 0..4 {
        let mut p = v["settings"].clone();
        match change {
            0 => p["profile"] = json!("PRIVATE_UNKNOWN"),
            1 => p["PRIVATE_UNKNOWN"] = json!("PRIVATE_text"),
            2 => p["initial_state"]["switch_positions"][0] = json!(25),
            _ => p["fast_switch"] = p["middle_switch"].clone(),
        };
        let e = execution::execute(
            "cipher.purple.encode",
            1,
            &json!({"text":v["text"],"settings":p}),
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
        ciphers::mechanical::{MAX_INPUT, MAX_TRACE, purple},
        errors::ErrorCode,
    };
    let mut p: purple::Settings = serde_json::from_value(vector()["settings"].clone()).unwrap();
    p.trace = false;
    assert_eq!(
        purple::encode(&"A".repeat(MAX_INPUT), &p)
            .unwrap()
            .text
            .len(),
        MAX_INPUT
    );
    assert_eq!(
        purple::encode(&"A".repeat(MAX_INPUT + 1), &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    p.trace = true;
    let r = purple::encode(&"A".repeat(16384), &p).unwrap();
    assert_eq!(r.trace.len(), 16384);
    assert!(serde_json::to_vec(&r).unwrap().len() <= 4194304);
    let e = purple::encode(&"A".repeat(MAX_TRACE), &p).unwrap_err();
    assert_eq!(e.code, ErrorCode::ResourceLimit);
    assert_eq!(e.parameter, Some("report"));
    assert_eq!(
        purple::encode(&"A".repeat(MAX_TRACE + 1), &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn complete_fourteen_part_message_first_part_matches_every_known_contact() {
    let v: Value = serde_json::from_str(include_str!(
        "../src/ciphers/mechanical/purple_message.json"
    ))
    .unwrap();
    // Explicit X placeholders preserve motion at documented unreadable contacts.
    for (direction, input, expected) in [
        ("encode", "plaintext", "ciphertext"),
        ("decode", "ciphertext", "plaintext"),
    ] {
        let input = v[input].as_str().unwrap().replace('-', "X");
        let r = run(direction, json!(input), vector()["settings"].clone());
        for (offset, (actual, expected)) in r["text"]
            .as_str()
            .unwrap()
            .bytes()
            .zip(v[expected].as_str().unwrap().bytes())
            .enumerate()
        {
            if expected != b'-' {
                assert_eq!(actual, expected, "source contact{offset}");
            }
        }
    }
}
