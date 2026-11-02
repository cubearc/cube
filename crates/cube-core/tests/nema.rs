//! Model45 physical windows and independent simulator/stepping source answers.
use cube_core::operations::execution::{self, Budget};
use serde_json::{Value, json};
fn vector() -> Value {
    serde_json::from_str(include_str!("../src/ciphers/mechanical/nema_vectors.json")).unwrap()
}
fn run(direction: &str, text: &str, settings: Value) -> Value {
    execution::execute(
        &format!("cipher.nema.{direction}"),
        1,
        &json!({"text":text,"settings":settings}),
        &json!({}),
        Budget::default(),
    )
    .unwrap()
}
#[test]
fn complete_sullivan_reference_and_final_windows_match_real_contact_phase() {
    let v = vector();
    let p = v["settings"].clone();
    let result = run("encode", v["text"].as_str().unwrap(), p.clone());
    assert_eq!(result["text"], v["expected"]);
    assert_eq!(result["final_state"], v["expected_final_state"]);
    assert_eq!(
        run("decode", v["expected"].as_str().unwrap(), p)["text"],
        v["text"]
    );
}
#[test]
fn all_twenty_six_published_steps_use_old_notches_and_decrement_real_windows() {
    let v = vector();
    let mut p = v["settings"].clone();
    p["initial_state"] = json!({"windows":"AAAAAAAAAA"});
    let result = run("encode", &"A".repeat(26), p);
    let states: Vec<Value> = result["trace"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["state"]["windows"].clone())
        .collect();
    assert_eq!(
        json!(states),
        json!(v["step_windows"].as_array().unwrap()[1..].to_vec())
    );
    assert_eq!(result["final_state"]["windows"], "FAWVJAWVLA");
}
#[test]
fn both_models_and_all_contact_wheels_are_reciprocal_and_continue_at_every_split() {
    let v = vector();
    let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZ";
    for (profile, rotors, drives) in [
        (
            "nema_model45_operational_v1",
            json!(["A", "B", "C", "D"]),
            json!([12, 13, 14, 15]),
        ),
        (
            "nema_model45_operational_v1",
            json!(["F", "E", "A", "D"]),
            json!([18, 17, 15, 12]),
        ),
        (
            "nema_model45_training_v1",
            json!(["D", "C", "B", "A"]),
            json!([21, 20, 19, 16]),
        ),
    ] {
        let mut p = v["settings"].clone();
        p["profile"] = json!(profile);
        p["rotors"] = rotors;
        p["drive_wheels"] = drives;
        p["initial_state"] = json!({"windows":"HFYQKFAQMF"});
        let whole = run("encode", text, p.clone());
        let decoded = run("decode", whole["text"].as_str().unwrap(), p.clone());
        assert_eq!(decoded["text"], text);
        assert_eq!(decoded["final_state"], whole["final_state"]);
        assert!(
            text.bytes()
                .zip(whole["text"].as_str().unwrap().bytes())
                .all(|(a, b)| a != b)
        );
        let empty = run("encode", "", p.clone());
        assert_eq!(empty["final_state"], p["initial_state"]);
        assert_eq!(empty["trace"], json!([]));
        for split in 0..=text.len() {
            let first = run("encode", &text[..split], p.clone());
            let mut next = p.clone();
            next["initial_state"] = first["final_state"].clone();
            let second = run("encode", &text[split..], next);
            assert_eq!(
                format!(
                    "{}{}",
                    first["text"].as_str().unwrap(),
                    second["text"].as_str().unwrap()
                ),
                whole["text"]
            );
            assert_eq!(second["final_state"], whole["final_state"]);
        }
    }
}
#[test]
fn operational_reference_state_returns_after_17576_steps() {
    let v = vector();
    let mut p = v["settings"].clone();
    p["initial_state"] = json!({"windows":"AAAAAAAAAA"});
    p["trace"] = json!(false);
    let result = run("encode", &"A".repeat(17576), p.clone());
    assert_eq!(result["final_state"], p["initial_state"]);
}
#[test]
fn wrong_wheel_families_duplicate_components_and_invalid_windows_are_refused() {
    let v = vector();
    let changes = [
        ("profile", json!("nema_foreign_office")),
        ("rotors", json!(["A", "A", "C", "D"])),
        ("drive_wheels", json!([12, 12, 14, 15])),
        ("drive_wheels", json!([16, 13, 14, 15])),
        ("drive_wheels", json!([255, 13, 14, 15])),
        ("initial_state", json!({"windows":"aaaaaaaaaa"})),
        ("initial_state", json!({"windows":"PRIVATE_WINDOW"})),
        ("initial_state", json!({"windows":"AAAAA"})),
        ("plugboard", json!("AB")),
    ];
    for (field, value) in changes {
        let mut p = v["settings"].clone();
        p[field] = value;
        let e = execution::execute(
            "cipher.nema.encode",
            1,
            &json!({"text":"A","settings":p}),
            &json!({}),
            Budget::default(),
        )
        .unwrap_err();
        assert!(!format!("{e:?}").contains("PRIVATE_WINDOW"));
    }
    let mut training = v["settings"].clone();
    training["profile"] = json!("nema_model45_training_v1");
    training["drive_wheels"] = json!([16, 19, 20, 21]);
    training["rotors"] = json!(["A", "B", "C", "F"]);
    assert!(
        execution::execute(
            "cipher.nema.encode",
            1,
            &json!({"text":"A","settings":training}),
            &json!({}),
            Budget::default()
        )
        .is_err()
    );
    for text in ["a", "A A", "é", "PRIVATE_TEXT"] {
        let e = execution::execute(
            "cipher.nema.encode",
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
fn full_native_input_and_trace_boundaries_preserve_every_character() {
    use cube_core::{
        ciphers::mechanical::{MAX_INPUT, MAX_TRACE, nema},
        errors::ErrorCode,
    };
    let v = vector();
    let mut p = v["settings"].clone();
    p["trace"] = json!(false);
    let typed: nema::Settings = serde_json::from_value(p).unwrap();
    assert_eq!(
        nema::encode(&"A".repeat(MAX_INPUT), &typed)
            .unwrap()
            .text
            .len(),
        MAX_INPUT
    );
    assert_eq!(
        nema::encode(&"A".repeat(MAX_INPUT + 1), &typed)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let traced = serde_json::from_value(v["settings"].clone()).unwrap();
    let result = nema::encode(&"A".repeat(MAX_TRACE), &traced).unwrap();
    assert_eq!(result.trace.len(), MAX_TRACE);
    assert!(serde_json::to_vec(&result).unwrap().len() <= 4 * 1024 * 1024);
    assert_eq!(
        nema::encode(&"A".repeat(MAX_TRACE + 1), &traced)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
