//! Independent ACA Numbered Key construction and worksheet checks.
use cube_core::operations::execution::{self, Budget};
use serde_json::json;

#[test]
fn aca_shifted_numbered_key_preserves_repeated_positions() {
    let parameters = json!({"alphabet":"abcdefghijklmnopqrstuvwxyz","key":"I like ciphers.","preparation":"ascii_lowercase_letters","rotation":18,"origin":"0","width":2,"choice":"round_robin"});
    let result = execution::execute(
        "cipher.numbered_key.encode",
        1,
        &json!({"text":"A eee iii"}),
        &parameters,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(
        result["tokens"],
        json!(["23", "15", "20", "15", "11", "13", "17"])
    );
    assert_eq!(result["text"], "aeeeiii");
    let tokens = "04 19 20 21 02 23 25 04 02 22 05 16 16 15 22 22 11 22 23 12 07 23 09 22 05 01 25 20 21 16 02 01 22 04 21 05 16 04 17 02 01".split_whitespace().collect::<Vec<_>>();
    let result = execution::execute(
        "cipher.numbered_key.decode",
        1,
        &json!({"tokens":tokens}),
        &parameters,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(result["text"], "theroadtosuccessisalwaysunderconstruction");
}

fn parameters() -> serde_json::Value {
    json!({"alphabet":"AB","key":"ABA","preparation":"exact","rotation":0,"origin":"0","width":1,"choice":"round_robin"})
}
fn call(
    op: &str,
    input: serde_json::Value,
    p: serde_json::Value,
) -> execution::Result<serde_json::Value> {
    execution::execute(
        op,
        1,
        &input,
        &p,
        Budget {
            output_bytes: 4 * 1048576,
            ..Default::default()
        },
    )
}
#[test]
fn first_choice_and_round_robin_reset_without_losing_the_decode_table() {
    let p = parameters();
    assert_eq!(
        call(
            "cipher.numbered_key.encode",
            json!({"text":"AAAA"}),
            p.clone()
        )
        .unwrap()["tokens"],
        json!(["0", "2", "0", "2"])
    );
    let mut first = p;
    first["choice"] = json!("first");
    let result = call(
        "cipher.numbered_key.encode",
        json!({"text":"AAAA"}),
        first.clone(),
    )
    .unwrap();
    assert_eq!(result["tokens"], json!(["0", "0", "0", "0"]));
    assert_eq!(result["table"][0]["tokens"], json!(["0", "2"]));
    assert_eq!(
        call(
            "cipher.numbered_key.decode",
            json!({"tokens":["2","1"]}),
            first
        )
        .unwrap()["text"],
        "AB"
    );
}
#[test]
fn exact_unicode_and_empty_inputs_keep_explicit_semantics() {
    let mut p = parameters();
    p["alphabet"] = json!("🐈\0e\u{301}");
    p["key"] = json!("🐈🐈");
    let result = call(
        "cipher.numbered_key.encode",
        json!({"text":"🐈\0e\u{301}🐈"}),
        p.clone(),
    )
    .unwrap();
    assert_eq!(result["tokens"], json!(["0", "2", "3", "4", "1"]));
    assert_eq!(
        call(
            "cipher.numbered_key.decode",
            json!({"tokens":result["tokens"]}),
            p.clone()
        )
        .unwrap()["text"],
        "🐈\0e\u{301}🐈"
    );
    assert_eq!(
        call("cipher.numbered_key.encode", json!({"text":""}), p).unwrap()["tokens"],
        json!([])
    );
}
#[test]
fn strict_tokens_and_invalid_definitions_are_rejected() {
    for patch in [
        json!({"alphabet":"AA"}),
        json!({"key":"C"}),
        json!({"rotation":3}),
        json!({"width":0}),
        json!({"origin":"01"}),
        json!({"origin":"340282366920938463463374607431768211455"}),
        json!({"unexpected":true}),
    ] {
        let mut p = parameters();
        p.as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        assert!(
            call("cipher.numbered_key.encode", json!({"text":"A"}), p).is_err(),
            "{patch}"
        );
    }
    for input in [
        json!({"tokens":["00"]}),
        json!({"tokens":["9"]}),
        json!({"tokens":[0]}),
        json!({"tokens":[],"extra":true}),
    ] {
        assert!(call("cipher.numbered_key.decode", input, parameters()).is_err());
    }
}
#[test]
fn exact_token_and_homophone_limits_and_checked_u128_origin() {
    let mut p = parameters();
    p["key"] = json!("A".repeat(4095));
    p["width"] = json!(4);
    assert!(
        call(
            "cipher.numbered_key.encode",
            json!({"text":"A".repeat(65536)}),
            p.clone()
        )
        .is_ok()
    );
    assert!(
        call(
            "cipher.numbered_key.encode",
            json!({"text":"A".repeat(65537)}),
            p.clone()
        )
        .is_err()
    );
    p["key"] = json!("A".repeat(4096));
    assert!(call("cipher.numbered_key.encode", json!({"text":""}), p).is_err());
    let p = json!({"alphabet":"A","key":"","preparation":"exact","rotation":0,"origin":"340282366920938463463374607431768211455","width":39,"choice":"first"});
    assert_eq!(
        call("cipher.numbered_key.encode", json!({"text":"A"}), p).unwrap()["tokens"],
        json!(["340282366920938463463374607431768211455"])
    );
}
