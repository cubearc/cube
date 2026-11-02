//! Configured Mk22 contact symbols and independently published simulator checks.
use cube_core::operations::execution::{self, Budget};
use serde_json::{Value, json};
fn vectors() -> Vec<Value> {
    serde_json::from_str(include_str!("../src/ciphers/mechanical/typex_vectors.json")).unwrap()
}
fn run(direction: &str, text: &str, settings: Value) -> Value {
    execution::execute(
        &format!("cipher.typex.{direction}"),
        1,
        &json!({"text":text,"settings":settings}),
        &json!({}),
        Budget::default(),
    )
    .unwrap()
}
#[test]
fn complete_configured_source_answer_and_reversed_fast_core_match() {
    for v in vectors() {
        let p = v["settings"].clone();
        let r = run("encode", v["text"].as_str().unwrap(), p.clone());
        assert_eq!(r["text"], v["expected"]);
        assert_eq!(r["final_state"], v["expected_final_state"]);
        assert_eq!(
            run("decode", v["expected"].as_str().unwrap(), p)["text"],
            v["text"]
        );
    }
}

#[test]
fn middle_double_steps_from_old_notches_and_stators_never_move() {
    let mut p = vectors()[0]["settings"].clone();
    p["turnover_windows"] = json!(["B", "B"]);
    p["initial_state"] = json!({"windows":"AAADA"});
    let r = run("encode", "AAAA", p.clone());
    let states: Vec<Value> = r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["state"]["windows"].clone())
        .collect();
    assert_eq!(json!(states), json!(["AABDA", "ABCDA", "BCDDA", "BCEDA"]));
    p["ring_settings"] = json!("ZYXWV");
    let shifted = run("encode", "AAAA", p);
    assert_eq!(shifted["trace"], r["trace"]);
    assert_ne!(shifted["text"], r["text"]);
}
#[test]
fn reversal_of_each_core_arbitrary_plugs_and_all_chunk_splits_are_reciprocal() {
    let v = &vectors()[0];
    let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZ";
    for mask in 0..32 {
        let mut p = v["settings"].clone();
        p["initial_state"] = json!({"windows":"ZMSDF"});
        p["ring_settings"] = json!("CBXYZ");
        p["plugboard"] = json!("BCDEFGHIJKLMNOPQRSTUVWXYZA");
        for i in 0..5 {
            p["rotors"][i]["reversed"] = json!(mask & (1 << i) != 0);
        }
        let whole = run("encode", text, p.clone());
        assert_eq!(
            run("decode", whole["text"].as_str().unwrap(), p.clone())["text"],
            text
        );
        assert!(
            text.bytes()
                .zip(whole["text"].as_str().unwrap().bytes())
                .all(|(a, b)| a != b)
        );
        assert_eq!(
            run("encode", "", p.clone())["final_state"],
            p["initial_state"]
        );
        if mask == 5 {
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
                    let second = run(direction, &input[split..], {
                        let mut next = p.clone();
                        next["initial_state"] = first["final_state"].clone();
                        next
                    });
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
fn incomplete_non_permutation_and_nonreciprocal_settings_are_private_errors() {
    let v = &vectors()[0];
    let settings = &v["settings"];
    for (field, value) in [
        ("profile", json!("typex_guess")),
        ("wheel_set_id", json!("PRIVATE_ID/KEY")),
        ("ring_settings", json!("aaaaa")),
        ("initial_state", json!({"windows":"PRIVATE_WINDOWS"})),
        ("turnover_windows", json!(["AA", "B"])),
        ("turnover_windows", json!(["", "B"])),
        ("turnover_windows", json!(["A", "é"])),
        ("entry_wiring", json!("A".repeat(26))),
        ("plugboard", json!("A".repeat(26))),
        ("reflector", json!("ABCDEFGHIJKLMNOPQRSTUVWXYZ")),
        ("reflector", json!("BCDEFGHIJKLMNOPQRSTUVWXYZA")),
        ("stator_motion", json!(true)),
    ] {
        let mut p = settings.clone();
        p[field] = value;
        let e = execution::execute(
            "cipher.typex.encode",
            1,
            &json!({"text":"A","settings":p}),
            &json!({}),
            Budget::default(),
        )
        .unwrap_err();
        assert!(!format!("{e:?}").contains("PRIVATE_"));
    }
    for change in 0..3 {
        let mut p = settings.clone();
        if change == 0 {
            p["rotors"][4]["id"] = p["rotors"][0]["id"].clone();
        } else if change == 1 {
            p["rotors"][0]["wiring"] = json!("ABCDEFGHIJKLMNOPQRSTUVWXYA");
        } else {
            p.as_object_mut().unwrap().remove("turnover_windows");
        }
        assert!(
            execution::execute(
                "cipher.typex.encode",
                1,
                &json!({"text":"A","settings":p}),
                &json!({}),
                Budget::default()
            )
            .is_err()
        );
    }
    for text in ["a", "A A", "秘密", "PRIVATE_TEXT"] {
        let e = execution::execute(
            "cipher.typex.encode",
            1,
            &json!({"text":text,"settings":settings}),
            &json!({}),
            Budget::default(),
        )
        .unwrap_err();
        assert!(!format!("{e:?}").contains("PRIVATE_TEXT"));
    }
}
#[test]
fn exact_native_input_and_trace_boundaries_return_complete_results() {
    use cube_core::{
        ciphers::mechanical::{MAX_INPUT, MAX_TRACE, typex},
        errors::ErrorCode,
    };
    let mut p = vectors()[0]["settings"].clone();
    p["trace"] = json!(false);
    let mut p: typex::Settings = serde_json::from_value(p).unwrap();
    assert_eq!(
        typex::encode(&"A".repeat(MAX_INPUT), &p)
            .unwrap()
            .text
            .len(),
        MAX_INPUT
    );
    assert_eq!(
        typex::encode(&"A".repeat(MAX_INPUT + 1), &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    p.trace = true;
    let report = typex::encode(&"A".repeat(MAX_TRACE), &p).unwrap();
    assert_eq!(report.trace.len(), MAX_TRACE);
    assert!(serde_json::to_vec(&report).unwrap().len() <= 4194304);
    assert_eq!(
        typex::encode(&"A".repeat(MAX_TRACE + 1), &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
