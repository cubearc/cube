//! Independent source checks through the public typed operation executor.
use cube_core::operations::execution::{self, Budget};
use serde_json::{Value, json};
fn vector() -> Value {
    serde_json::from_str(include_str!(
        "../src/ciphers/mechanical/sigaba_vectors.json"
    ))
    .unwrap()
}
fn run(direction: &str, text: &str, settings: Value) -> Value {
    execution::execute(
        &format!("cipher.sigaba.{direction}"),
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
fn old_o_carries_and_reversed_windows_have_the_historical_control_period() {
    let v = vector();
    let p = v["settings"].clone();
    let r = run("encode", "AA", p.clone());
    assert_eq!(r["trace"][0]["state"]["control_windows"], "KNPNN");
    assert_eq!(r["trace"][1]["state"]["control_windows"], "KNQNN");
    let mut p = p;
    p["trace"] = json!(false);
    assert_eq!(
        run("encode", &"A".repeat(26), p.clone())["final_state"]["control_windows"],
        "KNONN"
    );
    assert_eq!(
        run("encode", &"A".repeat(676), p.clone())["final_state"]["control_windows"],
        "KNOON"
    );
    assert_eq!(
        run("encode", &"A".repeat(17576), p.clone())["final_state"]["control_windows"],
        p["initial_state"]["control_windows"]
    );
}
#[test]
fn all_source_splits_preserve_inverse_and_post_step_motion_without_double_motors() {
    let v = vector();
    let p = v["settings"].clone();
    let whole = run("encode", v["text"].as_str().unwrap(), p.clone());
    let mut previous = p["initial_state"]["cipher_windows"]
        .as_str()
        .unwrap()
        .to_owned();
    for row in whole["trace"].as_array().unwrap() {
        let mask = row["stepped_mask"].as_u64().unwrap();
        assert!((1..=4).contains(&mask.count_ones()));
        let next = row["state"]["cipher_windows"].as_str().unwrap();
        for (i, (old, new)) in previous.bytes().zip(next.bytes()).enumerate() {
            let reversed = p["cipher_rotors"][i]["reversed"].as_bool().unwrap();
            let expected = if mask & (1 << i) == 0 {
                old
            } else if reversed {
                b'A' + (old - b'A' + 1) % 26
            } else {
                b'A' + (old - b'A' + 25) % 26
            };
            assert_eq!(new, expected);
        }
        assert_eq!(
            row["state"]["index_windows"],
            p["initial_state"]["index_windows"]
        );
        previous = next.to_owned();
    }
    for direction in ["encode", "decode"] {
        let input = if direction == "encode" {
            v["text"].as_str().unwrap()
        } else {
            v["expected"].as_str().unwrap()
        };
        let expected = if direction == "encode" {
            &v["expected"]
        } else {
            &v["text"]
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
                expected.as_str().unwrap()
            );
            assert_eq!(second["final_state"], whole["final_state"]);
        }
    }
}
#[test]
fn every_individual_wheel_reversal_retains_bijections_empty_state_and_inverse() {
    use std::collections::BTreeSet;
    let v = vector();
    for changed in 0..=15 {
        let mut p = v["settings"].clone();
        p["trace"] = json!(false);
        if changed < 15 {
            let bank = ["cipher_rotors", "control_rotors", "index_rotors"][changed / 5];
            let i = changed % 5;
            p[bank][i]["reversed"] = json!(!p[bank][i]["reversed"].as_bool().unwrap());
        }
        let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let encoded = run("encode", text, p.clone());
        assert_eq!(
            run("decode", encoded["text"].as_str().unwrap(), p.clone())["text"],
            text
        );
        assert_eq!(
            run("encode", "", p.clone())["final_state"],
            p["initial_state"]
        );
        let outputs: BTreeSet<String> = text
            .chars()
            .map(|c| {
                run("encode", &c.to_string(), p.clone())["text"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
            .collect();
        assert_eq!(outputs.len(), 26);
    }
}
#[test]
fn incompatible_variants_duplicate_banks_and_private_malformed_windows_are_refused() {
    let v = vector();
    for change in 0..8 {
        let mut p = v["settings"].clone();
        match change {
            0 => p["profile"] = json!("sigaba_csp2900_v1"),
            1 => p["cipher_rotors"][0]["id"] = json!(10),
            2 => p["control_rotors"][0]["id"] = p["cipher_rotors"][0]["id"].clone(),
            3 => p["index_rotors"][0]["id"] = json!(5),
            4 => p["index_rotors"][0]["id"] = p["index_rotors"][1]["id"].clone(),
            5 => p["initial_state"]["index_windows"] = json!("0900X"),
            6 => p["initial_state"]["control_windows"] = json!("PRIVATE_STATE"),
            _ => p["reset_to_O"] = json!(true),
        };
        let e = execution::execute(
            "cipher.sigaba.encode",
            1,
            &json!({"text":"A","settings":p}),
            &json!({}),
            Budget::default(),
        )
        .unwrap_err();
        assert!(!format!("{e:?}").contains("PRIVATE_STATE"));
    }
    for text in ["a", "A A", "é", "PRIVATE_TEXT"] {
        let e = execution::execute(
            "cipher.sigaba.encode",
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
fn input_trace_cardinality_and_complete_report_byte_ceilings_are_independent() {
    use cube_core::{
        ciphers::mechanical::{MAX_INPUT, MAX_TRACE, sigaba},
        errors::ErrorCode,
    };
    let mut p = vector()["settings"].clone();
    p["trace"] = json!(false);
    let mut p: sigaba::Settings = serde_json::from_value(p).unwrap();
    assert_eq!(
        sigaba::encode(&"A".repeat(MAX_INPUT), &p)
            .unwrap()
            .text
            .len(),
        MAX_INPUT
    );
    assert_eq!(
        sigaba::encode(&"A".repeat(MAX_INPUT + 1), &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    p.trace = true;
    let report = sigaba::encode(&"A".repeat(16384), &p).unwrap();
    assert_eq!(report.trace.len(), 16384);
    assert!(serde_json::to_vec(&report).unwrap().len() <= 4194304);
    let bytes = sigaba::encode(&"A".repeat(MAX_TRACE), &p).unwrap_err();
    assert_eq!(bytes.code, ErrorCode::ResourceLimit);
    assert_eq!(bytes.parameter, Some("report"));
    assert_eq!(
        sigaba::encode(&"A".repeat(MAX_TRACE + 1), &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
