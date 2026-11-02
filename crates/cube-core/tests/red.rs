//! Independent published contact check through the public executor.
use cube_core::operations::execution::{self, Budget};
use serde_json::{Value, json};
fn vector() -> Value {
    serde_json::from_str(include_str!("../src/ciphers/mechanical/red_vectors.json")).unwrap()
}
fn run(direction: &str, input: Value, settings: Value) -> Value {
    execution::execute(
        &format!("cipher.red.{direction}"),
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
fn paper_example_and_missing_pin_runs_wrap_in_both_directions() {
    let mut p = vector()["settings"].clone();
    p["alphabet"] = json!("TBZJNRSKYMPIHLDVWAOFQCXUEG");
    p["pins"] = json!("1".repeat(47));
    p["initial_state"] = json!({"breakwheel":0,"sixes":0,"twenties":0});
    assert_eq!(run("encode", json!("TOBEOR"), p.clone())["text"], "TARCDT");
    p["pins"] = json!(format!("1{}", "0".repeat(46)));
    p["initial_state"]["breakwheel"] = json!(1);
    let direct = run("encode", json!("A"), p.clone());
    assert_eq!(direct["trace"][0]["steps"], 47);
    assert_eq!(
        direct["final_state"],
        json!({"breakwheel":1,"sixes":5,"twenties":7})
    );
    p["direction"] = json!("reverse");
    p["initial_state"]["breakwheel"] = json!(46);
    let reverse = run("encode", json!("A"), p.clone());
    assert_eq!(reverse["trace"][0]["steps"], 47);
    assert_eq!(
        reverse["final_state"],
        json!({"breakwheel":46,"sixes":1,"twenties":13})
    );
    let text = json!("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    let r = run("encode", text.clone(), p.clone());
    assert_eq!(run("decode", r["text"].clone(), p)["text"], text);
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
            2 => p["initial_state"]["breakwheel"] = json!(47),
            _ => p["pins"] = json!("0".repeat(47)),
        };
        let e = execution::execute(
            "cipher.red.encode",
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
        ciphers::mechanical::{MAX_INPUT, MAX_TRACE, red},
        errors::ErrorCode,
    };
    let mut p: red::Settings = serde_json::from_value(vector()["settings"].clone()).unwrap();
    p.trace = false;
    assert_eq!(
        red::encode(&"A".repeat(MAX_INPUT), &p).unwrap().text.len(),
        MAX_INPUT
    );
    assert_eq!(
        red::encode(&"A".repeat(MAX_INPUT + 1), &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    p.trace = true;
    let r = red::encode(&"A".repeat(16384), &p).unwrap();
    assert_eq!(r.trace.len(), 16384);
    assert!(serde_json::to_vec(&r).unwrap().len() <= 4194304);
    let e = red::encode(&"A".repeat(MAX_TRACE), &p).unwrap_err();
    assert_eq!(e.code, ErrorCode::ResourceLimit);
    assert_eq!(e.parameter, Some("report"));
    assert_eq!(
        red::encode(&"A".repeat(MAX_TRACE + 1), &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn complete_january_1937_first_transmitted_part_matches_source() {
    let v: Value =
        serde_json::from_str(include_str!("../src/ciphers/mechanical/red_message.json")).unwrap();
    let p = vector()["settings"].clone();
    let r = run("encode", v["ciphertext"].clone(), p.clone());
    assert_eq!(r["text"], v["plaintext"]);
    assert_eq!(
        run("decode", v["plaintext"].clone(), p)["text"],
        v["ciphertext"]
    );
}
