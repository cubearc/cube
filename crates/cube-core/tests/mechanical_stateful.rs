//! Published configurations, not a cipher followed only by its own inverse.
use cube_core::operations::execution::{self, Budget};
use serde_json::{Value, json};
fn run(operation: &str, text: &str, settings: Value) -> Value {
    execution::execute(
        operation,
        1,
        &json!({"text":text,"settings":settings}),
        &json!({}),
        Budget::default(),
    )
    .unwrap()
}
#[test]
fn wheatstones_original_france_example_is_executable() {
    // Wheatstone1879, reproduced/analyzed by Friedman1918, printed pp7–10.
    let settings = json!({"profile":"wheatstone_1879_v1","cipher_alphabet":"FBKSYRDLTZAGMUNHOVCIPWEJQX","initial_state":{"plain_position":0,"cipher_position":0}});
    let r = run("cipher.wheatstone.encode", "THE+QUARTER+", settings.clone());
    assert_eq!(r["text"], "PZLSPQREQAJD");
    assert_eq!(
        r["final_state"],
        json!({"plain_position":0,"cipher_position":6})
    );
    assert_eq!(
        run("cipher.wheatstone.decode", "PZLSPQREQAJD", settings)["text"],
        "THE+QUARTER+"
    );
}
#[test]
fn kryha_published_hydraulic_key_and_seventeen_sector_steps_are_executable() {
    // Callimahos1973, Q.E.D., pp20–21: exact17-sector effective step table.
    let settings = json!({"profile":"kryha_callimahos_1973_v1","plain_alphabet":"ABCDEFGHIJKLMNOPQRSTUVWXYZ","cipher_alphabet":"HYDRAULICBEFGJKMNOPQSTVWXZ","initial_state":{"rotation":0,"selector":1},"trace":true});
    let r = run("cipher.kryha.encode", &"A".repeat(17), settings.clone());
    assert_eq!(r["text"], "HIJSZUGPHLNTYLJQX");
    assert_eq!(r["final_state"], json!({"rotation":7,"selector":1}));
    assert_eq!(r["trace"][0]["state"], json!({"rotation":7,"selector":2}));
    assert_eq!(
        run("cipher.kryha.decode", "HIJSZUGPHLNTYLJQX", settings)["text"],
        "A".repeat(17)
    );
}
#[test]
fn alberti_museum_digit_realignments_match_both_published_examples() {
    // DiPalma2009, primary reconstruction pp11–13, lower-case u / upper-case V.
    let settings = json!({"profile":"alberti_di_palma_2009_v1","initial_index":"c","trace":true});
    let r = run(
        "cipher.alberti.encode",
        "ALE4ISA1BEAVTI3FVL2CITI",
        settings.clone(),
    );
    assert_eq!(r["text"], "czdqpoqlhqluybeix&okapa");
    assert_eq!(r["final_index"], "o");
    assert_eq!(
        run("cipher.alberti.decode", "sfzefqqfxkmohihuy", settings)["text"],
        "SILVI4ALOVES2CATS"
    );
}

#[test]
fn stateful_profiles_resume_from_the_returned_state_in_both_directions() {
    let cases = [
        (
            "wheatstone",
            "THE+QUARTER+",
            json!({"profile":"wheatstone_1879_v1","cipher_alphabet":"FBKSYRDLTZAGMUNHOVCIPWEJQX","initial_state":{"plain_position":0,"cipher_position":0}}),
        ),
        (
            "kryha",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            json!({"profile":"kryha_callimahos_1973_v1","plain_alphabet":"QWERTYUIOPASDFGHJKLZXCVBNM","cipher_alphabet":"HYDRAULICBEFGJKMNOPQSTVWXZ","initial_state":{"rotation":25,"selector":17}}),
        ),
        (
            "alberti",
            "ALE4ISA1BEAVTI3FVL2CITI",
            json!({"profile":"alberti_di_palma_2009_v1","initial_index":"&"}),
        ),
    ];
    for (machine, plain, settings) in cases {
        let encoded = run(&format!("cipher.{machine}.encode"), plain, settings.clone());
        for direction in ["encode", "decode"] {
            let op = format!("cipher.{machine}.{direction}");
            let text = if direction == "encode" {
                plain
            } else {
                encoded["text"].as_str().unwrap()
            };
            let whole = run(&op, text, settings.clone());
            for split in 0..=text.len() {
                let first = run(&op, &text[..split], settings.clone());
                let mut resumed = settings.clone();
                let (initial, final_key) = if machine == "alberti" {
                    ("initial_index", "final_index")
                } else {
                    ("initial_state", "final_state")
                };
                resumed[initial] = first[final_key].clone();
                let second = run(&op, &text[split..], resumed);
                assert_eq!(
                    format!(
                        "{}{}",
                        first["text"].as_str().unwrap(),
                        second["text"].as_str().unwrap()
                    ),
                    whole["text"],
                    "{op} at {split}"
                );
                assert_eq!(second[final_key], whole[final_key]);
                assert_eq!(second["trace"], json!([]));
            }
            if direction == "decode" {
                assert_eq!(whole["text"], plain);
            }
        }
    }
}

#[test]
fn kryha_selector_and_rotation_return_after_442_letters() {
    let p = json!({"profile":"kryha_callimahos_1973_v1","plain_alphabet":"ABCDEFGHIJKLMNOPQRSTUVWXYZ","cipher_alphabet":"HYDRAULICBEFGJKMNOPQSTVWXZ","initial_state":{"rotation":0,"selector":1},"trace":true});
    let r = run("cipher.kryha.encode", &"A".repeat(884), p);
    assert_eq!(r["final_state"], json!({"rotation":0,"selector":1}));
    let text = r["text"].as_str().unwrap();
    assert_eq!(&text[..442], &text[442..]);
    assert!(
        r["trace"].as_array().unwrap()[..441]
            .iter()
            .all(|t| t["state"] != r["final_state"])
    );
}

#[test]
fn exact_trace_and_text_limits_fail_without_truncation_or_private_input_echo() {
    use cube_core::{
        ciphers::mechanical::{self, alberti, kryha, wheatstone},
        errors::ErrorCode,
    };
    let mut a: alberti::Settings = serde_json::from_value(
        json!({"profile":"alberti_di_palma_2009_v1","initial_index":"c","trace":true}),
    )
    .unwrap();
    let mut k: kryha::Settings = serde_json::from_value(json!({"profile":"kryha_callimahos_1973_v1","plain_alphabet":"ABCDEFGHIJKLMNOPQRSTUVWXYZ","cipher_alphabet":"HYDRAULICBEFGJKMNOPQSTVWXZ","initial_state":{"rotation":0,"selector":1},"trace":true})).unwrap();
    let mut w: wheatstone::Settings = serde_json::from_value(json!({"profile":"wheatstone_1879_v1","cipher_alphabet":"ABCDEFGHIJKLMNOPQRSTUVWXYZ","initial_state":{"plain_position":0,"cipher_position":0},"trace":true})).unwrap();
    let at = "A".repeat(mechanical::MAX_TRACE);
    assert_eq!(
        alberti::encode(&at, &a).unwrap().trace.len(),
        mechanical::MAX_TRACE
    );
    assert_eq!(
        kryha::encode(&at, &k).unwrap().trace.len(),
        mechanical::MAX_TRACE
    );
    let wt = "AB".repeat(mechanical::MAX_TRACE / 2);
    assert_eq!(
        wheatstone::encode(&wt[..48000], &w).unwrap().trace.len(),
        48000
    );
    // The independently bounded full JSON report can hit 4 MiB before the trace count.
    assert_eq!(
        wheatstone::encode(&wt, &w).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    for e in [
        alberti::encode(&(at.clone() + "A"), &a).unwrap_err(),
        kryha::encode(&(at + "A"), &k).unwrap_err(),
        wheatstone::encode(&(wt + "A"), &w).unwrap_err(),
    ] {
        assert_eq!(e.code, ErrorCode::ResourceLimit);
    }
    a.trace = false;
    k.trace = false;
    w.trace = false;
    let at = "A".repeat(mechanical::MAX_INPUT);
    assert_eq!(
        alberti::encode(&at, &a).unwrap().text.len(),
        mechanical::MAX_INPUT
    );
    assert_eq!(
        kryha::encode(&at, &k).unwrap().text.len(),
        mechanical::MAX_INPUT
    );
    assert_eq!(
        wheatstone::encode(&"AB".repeat(mechanical::MAX_INPUT / 2), &w)
            .unwrap()
            .text
            .len(),
        mechanical::MAX_INPUT
    );
    for e in [
        alberti::encode(&(at.clone() + "A"), &a).unwrap_err(),
        kryha::encode(&(at.clone() + "A"), &k).unwrap_err(),
        wheatstone::encode(&(at + "A"), &w).unwrap_err(),
    ] {
        assert_eq!(e.code, ErrorCode::ResourceLimit);
    }
    for e in [
        alberti::encode("A秘密", &a).unwrap_err(),
        kryha::encode("A秘密", &k).unwrap_err(),
        wheatstone::encode("A秘密", &w).unwrap_err(),
    ] {
        assert_eq!(e.code, ErrorCode::InvalidInput);
        assert!(!e.to_string().contains("秘密"));
    }
    assert_eq!(
        wheatstone::encode("AA", &w).unwrap_err().code,
        ErrorCode::InvalidInput
    );
    assert_eq!(
        wheatstone::encode("+", &w).unwrap_err().code,
        ErrorCode::InvalidInput
    );
    assert_eq!(
        alberti::encode("U", &a).unwrap_err().code,
        ErrorCode::InvalidInput
    );
    assert_eq!(
        alberti::decode("v", &a).unwrap_err().code,
        ErrorCode::InvalidInput
    );
    a.initial_index = "cc".into();
    k.initial_state.selector = 0;
    w.initial_state.plain_position = 27;
    assert_eq!(
        alberti::encode("", &a).unwrap_err().code,
        ErrorCode::InvalidParameter
    );
    assert_eq!(
        kryha::encode("", &k).unwrap_err().code,
        ErrorCode::InvalidParameter
    );
    assert_eq!(
        wheatstone::encode("", &w).unwrap_err().code,
        ErrorCode::InvalidParameter
    );
    k.initial_state.selector = 1;
    k.plain_alphabet = "A".repeat(26);
    assert_eq!(
        kryha::encode("", &k).unwrap_err().code,
        ErrorCode::InvalidParameter
    );
    w.initial_state.plain_position = 0;
    w.cipher_alphabet = "a".repeat(26);
    assert_eq!(
        wheatstone::encode("", &w).unwrap_err().code,
        ErrorCode::InvalidParameter
    );
}
