//! Logical contact positions and independently laid transition examples.
use cube_core::operations::execution::{self, Budget};
use serde_json::{Value, json};
fn settings(profile: &str) -> Value {
    let mut chi: Vec<String> = [41, 31, 29, 26, 23].map(|n| "0".repeat(n)).into();
    chi[0].replace_range(..4, "1010");
    let mut psi: Vec<String> = [43, 47, 51, 53, 59].map(|n| "0".repeat(n)).into();
    psi[0].replace_range(..4, "0110");
    json!({"profile":profile,"chi":chi,"psi":psi,"mu61":"1".repeat(61),"mu37":"0".repeat(37),"plaintext_limitation":false,"initial_state":{"chi":[0,0,0,0,0],"psi":[0,0,0,0,0],"mu61":0,"mu37":0,"chi2_previous":false,"psi1_previous":false,"plain5_previous":[false,false]},"trace":true})
}
fn run(direction: &str, codes: Value, settings: Value) -> Value {
    execution::execute(
        &format!("cipher.lorenz.{direction}"),
        1,
        &json!({"codes":codes,"settings":settings}),
        &json!({}),
        Budget::default(),
    )
    .unwrap()
}
#[test]
fn sz40_sz42a_and_sz42b_have_distinct_executable_motor_rules() {
    // Chi1=1010, Psi1=0110; M37 dots and M61 crosses. Other signal pins are dots.
    // SZ40 stops all Psi motion. Chi2 limitation releases SZ42a at every place.
    // SZ42b stops when the previous Psi1 cross makes the limitation a cross.
    for (profile, expected, psi) in [
        ("lorenz_sz40_v1", json!([1, 0, 17, 0]), 0),
        ("lorenz_sz42a_v1", json!([1, 1, 16, 0]), 4),
        ("lorenz_sz42b_v1", json!([1, 1, 16, 1]), 2),
    ] {
        let p = settings(profile);
        let r = run("encode", json!([0, 0, 16, 0]), p.clone());
        assert_eq!(r["codes"], expected, "{profile}");
        assert_eq!(r["final_state"]["psi"], json!([psi, psi, psi, psi, psi]));
        assert_eq!(r["final_state"]["mu37"], 4);
        assert_eq!(run("decode", expected, p)["codes"], json!([0, 0, 16, 0]));
    }
}

#[test]
fn complete_bream_pin_transcription_matches_hand_read_contact_and_motor_states() {
    let v: Value = serde_json::from_str(include_str!(
        "../src/ciphers/mechanical/lorenz_vectors.json"
    ))
    .unwrap();
    let r = run("encode", v["codes"].clone(), v["settings"].clone());
    assert_eq!(r["codes"], v["expected"]);
    assert_eq!(r["final_state"]["mu61"], 4);
    assert_eq!(r["final_state"]["mu37"], 3);
    assert_eq!(r["final_state"]["psi"], json!([3, 3, 3, 3, 3]));
    let keys: Vec<Value> = r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["key_code"].clone())
        .collect();
    assert_eq!(keys, vec![json!(14), json!(2), json!(28), json!(2)]);
    assert_eq!(
        run("decode", v["expected"].clone(), v["settings"].clone())["codes"],
        v["codes"]
    );
}

#[test]
fn limitation_truth_tables_and_basic_motor_override_match_the_documented_contacts() {
    // Rows are previous Chi2/Psi1/P5 two-back, in binary order000..111.
    let tables = [
        (
            "lorenz_sz40_v1",
            false,
            [false, false, false, false, false, false, false, false],
        ),
        (
            "lorenz_sz42a_v1",
            false,
            [true, true, true, true, false, false, false, false],
        ),
        (
            "lorenz_sz42a_v1",
            true,
            [true, false, true, false, false, true, false, true],
        ),
        (
            "lorenz_sz42b_v1",
            false,
            [true, true, false, false, false, false, true, true],
        ),
        (
            "lorenz_sz42b_v1",
            true,
            [true, false, false, true, false, true, true, false],
        ),
    ];
    for (profile, kt, table) in tables {
        for (row, expected) in table.into_iter().enumerate() {
            let mut p = settings(profile);
            p["plaintext_limitation"] = json!(kt);
            p["initial_state"]["chi2_previous"] = json!(row & 4 != 0);
            p["initial_state"]["psi1_previous"] = json!(row & 2 != 0);
            p["initial_state"]["plain5_previous"] = json!([row & 1 != 0, false]);
            let r = run("encode", json!([0]), p.clone());
            assert_eq!(
                r["trace"][0]["psi_step"], expected,
                "{profile}, KT={kt}, row={row}"
            );
            p["mu37"] = json!("1".repeat(37));
            assert_eq!(run("encode", json!([0]), p)["trace"][0]["psi_step"], true);
        }
    }
}

#[test]
fn kt_uses_recovered_plaintext_two_back_and_continues_across_every_split() {
    use cube_core::ciphers::mechanical::lorenz;
    let plain: Vec<u8> = (0..160).map(|i| ((i * 7 + 3) % 32) as u8).collect();
    for profile in ["lorenz_sz40_v1", "lorenz_sz42a_v1", "lorenz_sz42b_v1"] {
        for kt in [false, true] {
            if profile == "lorenz_sz40_v1" && kt {
                continue;
            }
            let mut value = settings(profile);
            value["plaintext_limitation"] = json!(kt);
            value["trace"] = json!(false);
            value["initial_state"]["plain5_previous"] = json!([true, false]);
            let p: lorenz::Settings = serde_json::from_value(value).unwrap();
            let encoded = lorenz::encode(&plain, &p).unwrap();
            let decoded = lorenz::decode(&encoded.codes, &p).unwrap();
            assert_eq!(decoded.codes, plain);
            assert_eq!(decoded.final_state, encoded.final_state);
            for (input, transform, whole) in [
                (
                    &plain,
                    lorenz::encode
                        as fn(
                            &[u8],
                            &lorenz::Settings,
                        )
                            -> Result<lorenz::Report, cube_core::errors::OperationError>,
                    &encoded,
                ),
                (
                    &encoded.codes,
                    lorenz::decode
                        as fn(
                            &[u8],
                            &lorenz::Settings,
                        )
                            -> Result<lorenz::Report, cube_core::errors::OperationError>,
                    &decoded,
                ),
            ] {
                for split in 0..=input.len() {
                    let first: lorenz::Report = transform(&input[..split], &p).unwrap();
                    let mut continuation = p.clone();
                    continuation.initial_state = first.final_state;
                    let second: lorenz::Report = transform(&input[split..], &continuation).unwrap();
                    assert_eq!([first.codes, second.codes].concat(), whole.codes);
                    assert_eq!(second.final_state, whole.final_state);
                    assert!(second.trace.is_empty());
                }
            }
        }
    }
    let mut p = settings("lorenz_sz42a_v1");
    p["plaintext_limitation"] = json!(true);
    let r = run("encode", json!([0, 0, 16, 0, 0, 0]), p);
    let movement: Vec<Value> = r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["psi_step"].clone())
        .collect();
    assert_eq!(
        movement,
        json!([true, true, true, true, false, true])
            .as_array()
            .unwrap()
            .to_vec()
    );
}

#[test]
fn motor_hold_wrap_and_all_five_impulses_are_explicit() {
    use cube_core::ciphers::mechanical::lorenz;
    let mut value = settings("lorenz_sz40_v1");
    value["mu61"] = json!("0".repeat(61));
    value["mu37"] = json!("1".repeat(37));
    value["initial_state"]["chi"] = json!([40, 30, 28, 25, 22]);
    value["initial_state"]["psi"] = json!([42, 46, 50, 52, 58]);
    value["initial_state"]["mu61"] = json!(60);
    value["initial_state"]["mu37"] = json!(36);
    let r = run("encode", json!([0]), value.clone());
    assert_eq!(r["final_state"]["chi"], json!([0, 0, 0, 0, 0]));
    assert_eq!(r["final_state"]["psi"], json!([0, 0, 0, 0, 0]));
    assert_eq!(r["final_state"]["mu61"], 0);
    assert_eq!(r["final_state"]["mu37"], 36);
    value["mu61"] = json!("1".repeat(61));
    assert_eq!(run("encode", json!([0]), value)["final_state"]["mu37"], 0);
    for bit in 0..5 {
        let mut value = settings("lorenz_sz40_v1");
        value["chi"] = json!(lorenz::CHI_LENGTHS.map(|n| "0".repeat(n)));
        value["psi"] = json!(lorenz::PSI_LENGTHS.map(|n| "0".repeat(n)));
        value["chi"][bit] = json!("1".repeat(lorenz::CHI_LENGTHS[bit]));
        assert_eq!(run("encode", json!([0]), value)["codes"], json!([1 << bit]));
    }
}

#[test]
fn code_trace_and_serialized_report_limits_fail_without_partial_results() {
    use cube_core::{
        ciphers::mechanical::{self, lorenz},
        errors::ErrorCode,
    };
    let mut p: lorenz::Settings = serde_json::from_value(settings("lorenz_sz40_v1")).unwrap();
    p.trace = false;
    let input = vec![31; mechanical::MAX_INPUT];
    assert_eq!(
        lorenz::encode(&input, &p).unwrap().codes.len(),
        mechanical::MAX_INPUT
    );
    assert_eq!(
        lorenz::encode(&vec![0; mechanical::MAX_INPUT + 1], &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    p.trace = true;
    assert_eq!(
        lorenz::encode(&vec![0; 4096], &p).unwrap().trace.len(),
        4096
    );
    for count in [mechanical::MAX_TRACE, mechanical::MAX_TRACE + 1] {
        assert_eq!(
            lorenz::encode(&vec![0; count], &p).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
    }
    for code in [32, 255] {
        let e = lorenz::encode(&[0, code], &p).unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidInput);
        assert_eq!(e.offset, Some(1));
        assert_eq!(e.parameter, Some("codes"));
    }
    let mut value = settings("lorenz_sz40_v1");
    value["trace"] = json!(false);
    let e = execution::execute(
        "cipher.lorenz.encode",
        1,
        &json!({"codes":vec![0;mechanical::MAX_INPUT+1],"settings":value}),
        &json!({}),
        Budget::default(),
    )
    .unwrap_err();
    assert_eq!(e.to_string(), "resource_limit");
}

#[test]
fn invalid_complete_settings_and_unknown_profiles_are_rejected() {
    use cube_core::{
        ciphers::mechanical::{self, lorenz},
        errors::ErrorCode,
    };
    let base: lorenz::Settings = serde_json::from_value(settings("lorenz_sz40_v1")).unwrap();
    for field in ["chi", "psi"] {
        for i in 0..5 {
            let mut p = base.clone();
            if field == "chi" {
                p.chi[i].push('0');
            } else {
                p.psi[i].push('0');
            }
            assert_eq!(
                lorenz::encode(&[], &p).unwrap_err().code,
                ErrorCode::InvalidParameter
            );
        }
    }
    let mut bad = Vec::new();
    let mut p = base.clone();
    p.chi[0] = "x".repeat(41);
    bad.push(p);
    let mut p = base.clone();
    p.psi[0] = "猫".repeat(43);
    bad.push(p);
    let mut p = base.clone();
    p.mu61 = "0".repeat(60);
    bad.push(p);
    let mut p = base.clone();
    p.mu37 = "2".repeat(37);
    bad.push(p);
    let mut p = base.clone();
    p.initial_state.chi[4] = 23;
    bad.push(p);
    let mut p = base.clone();
    p.initial_state.psi[4] = 59;
    bad.push(p);
    let mut p = base.clone();
    p.initial_state.mu61 = 61;
    bad.push(p);
    let mut p = base.clone();
    p.initial_state.mu37 = 37;
    bad.push(p);
    let mut p = base.clone();
    p.plaintext_limitation = true;
    bad.push(p);
    for p in bad {
        assert_eq!(
            lorenz::encode(&[], &p).unwrap_err().code,
            ErrorCode::InvalidParameter
        );
    }
    for document in [
        r#"{"profile":"lorenz_sz42_v1"}"#,
        r#"{"profile":"lorenz_sz40_v1","profile":"lorenz_sz42a_v1"}"#,
    ] {
        assert!(
            mechanical::parse_parameters::<lorenz::Settings>(document, "cipher.lorenz.encode")
                .is_err()
        );
    }
    for key in ["initial_state", "plaintext_limitation"] {
        let mut value = settings("lorenz_sz42a_v1");
        value.as_object_mut().unwrap().remove(key);
        assert!(
            mechanical::parse_parameters::<lorenz::Settings>(
                &value.to_string(),
                "cipher.lorenz.encode"
            )
            .is_err()
        );
    }
    let mut value = settings("lorenz_sz42a_v1");
    value["guess"] = json!(true);
    assert!(
        mechanical::parse_parameters::<lorenz::Settings>(
            &value.to_string(),
            "cipher.lorenz.encode"
        )
        .is_err()
    );
}

#[test]
fn bounded_json_transport_preserves_empty_arrays_and_refuses_malformed_values() {
    use cube_core::{
        ciphers::mechanical::{self, lorenz},
        errors::ErrorCode,
    };
    let op = "cipher.lorenz.encode";
    let at = format!("{}[]", " ".repeat(mechanical::MAX_OUTPUT - 2));
    assert!(lorenz::parse_codes(&at, op).unwrap().is_empty());
    assert_eq!(
        lorenz::parse_codes(&(at + " "), op).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    for document in [
        "[true]",
        "[-1]",
        "[256]",
        "[1.5]",
        "[\"SECRET\"]",
        "[[0]]",
        "{}",
        "[0] [0]",
    ] {
        let e = lorenz::parse_codes(document, op).unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidInput);
        assert!(!e.to_string().contains("SECRET"));
    }
}
