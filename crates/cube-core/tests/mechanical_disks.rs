use cube_core::operations::execution::{self, Budget};
use cube_core::{
    ciphers::mechanical::{
        self,
        disks::{self, M94Parameters},
    },
    errors::ErrorCode,
};
use serde_json::json;

fn invoke(
    operation: &str,
    version: u32,
    inputs: &serde_json::Value,
    settings: &serde_json::Value,
    budget: Budget,
) -> execution::Result<serde_json::Value> {
    let mut inputs = inputs.clone();
    inputs["settings"] = settings.clone();
    execution::execute(operation, version, &inputs, &json!({}), budget)
}

fn settings() -> M94Parameters {
    serde_json::from_value(json!({"profile":"m94_v1","order":[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25],"row_offsets":[]})).unwrap()
}

#[test]
fn m94_manual_message_has_an_executable_core_contract() {
    let parameters = json!({"profile":"m94_v1","order":[3, 10, 12, 16, 6, 21, 7, 14, 1, 23, 17, 5, 20, 25, 4, 11, 13, 18, 8, 22, 9, 15, 2, 24, 19],"row_offsets":[20,10,9,4]});
    let result=invoke("cipher.m94.encode",1,&json!({"text":"HAVEJUSTREACHEDEASTERNEDGEOFWOODSALONGFIVEFIVETWODASHFIVENINETWOROADWILLREMAININOBSERVATION"}),&parameters,Budget::default()).unwrap();
    assert_eq!(
        result["text"],
        "JUKLDYKITZIIVCYCVUYVPYWHJYUYEZDHVUZDBQPOZMCFNBJJIXEAPTHYOWHKWWTNYGMPRZJIFADMEQRHBPOJTYUQNTW"
    );
}

#[test]
fn m94_all_wheels_and_rows_reverse_without_guessing_or_padding() {
    let mut p = settings();
    for wheel in disks::M94_WHEELS {
        let mut alphabet = wheel.bytes().collect::<Vec<_>>();
        alphabet.sort_unstable();
        assert_eq!(alphabet, b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    for offset in 1..=25 {
        let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".repeat(25);
        p.row_offsets = vec![offset; 26];
        let encoded = disks::m94_encode(&text, &p).unwrap();
        assert_eq!(disks::m94_decode(&encoded.text, &p).unwrap().text, text);
        assert_eq!(disks::m94_encode(&text, &p).unwrap().text, encoded.text);
    }
    p.row_offsets = vec![1, 25];
    let encoded = disks::m94_encode(&"A".repeat(26), &p).unwrap();
    assert_eq!(encoded.text, "BCDEFGHIJKLMNOPQRSTUVWXYZ N".replace(' ', ""));
    assert_eq!(
        disks::m94_decode(&encoded.text, &p).unwrap().text,
        "A".repeat(26)
    );
}

#[test]
fn m94_invalid_settings_text_and_missing_rows_return_safe_typed_errors() {
    let mut p = settings();
    assert_eq!(disks::m94_encode("", &p).unwrap().text, "");
    for id in [0, 26, 2] {
        p.order[0] = id;
        let e = disks::m94_encode("", &p).unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidParameter);
        assert_eq!(e.parameter, Some("order"));
        p = settings();
    }
    for offsets in [vec![], vec![0], vec![26], vec![1, 2]] {
        p.row_offsets = offsets;
        assert_eq!(
            disks::m94_encode("SECRET", &p).unwrap_err().parameter,
            Some("row_offsets")
        );
    }
    p.row_offsets = vec![1];
    for text in ["a", "A A", "é", "A\0"] {
        let e = disks::m94_encode(text, &p).unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidInput);
        assert!(e.offset.is_some());
        assert_eq!(e.operation, Some("cipher.m94.encode"));
    }
    for document in [
        r#"{"profile":"m94_v1","profile":"m94_v1"}"#,
        r#"{"profile":"m94_v2"}"#,
        r#"{}"#,
    ] {
        assert!(
            mechanical::parse_parameters::<M94Parameters>(document, "cipher.m94.encode").is_err()
        );
    }
}

#[test]
fn m94_exact_input_and_settings_transport_limits_are_enforced() {
    let mut p = settings();
    p.row_offsets = vec![1; mechanical::MAX_INPUT.div_ceil(25)];
    assert_eq!(
        disks::m94_encode(&"A".repeat(mechanical::MAX_INPUT), &p)
            .unwrap()
            .text
            .len(),
        mechanical::MAX_INPUT
    );
    assert_eq!(
        disks::m94_encode(&"A".repeat(mechanical::MAX_INPUT + 1), &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert!(matches!(
        invoke(
            "cipher.m94.encode",
            1,
            &json!({"text":"A".repeat(mechanical::MAX_INPUT+1)}),
            &serde_json::to_value(&p).unwrap(),
            Budget::default()
        ),
        Err(execution::Failure("resource_limit"))
    ));
    let mut document = serde_json::to_string(&settings()).unwrap();
    document.push_str(&" ".repeat(mechanical::MAX_PARAMETERS - document.len()));
    mechanical::parse_parameters::<M94Parameters>(&document, "cipher.m94.encode").unwrap();
    document.push(' ');
    assert_eq!(
        mechanical::parse_parameters::<M94Parameters>(&document, "cipher.m94.encode")
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn jefferson_configured_wheels_are_executable_with_explicit_row_selection() {
    // Three independently laid-out paper wheels; clockwise row1 is B/Z/K.
    let p = json!({"profile":"jefferson_configured_v1","wheels":["ABCDEFGHIJKLMNOPQRSTUVWXYZ","AZYXWVUTSRQPONMLKJIHGFEDCB","QWERTYUIOPASDFGHJKLZXCVBNM"],"order":[1,2,3],"row_offsets":[1,25]});
    let r = invoke(
        "cipher.jefferson.encode",
        1,
        &json!({"text":"AAJAAA"}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(r["text"], "BZKZBP");
    let d = invoke(
        "cipher.jefferson.decode",
        1,
        &json!({"text":r["text"]}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(d["text"], "AAJAAA");
}

#[test]
fn m138a_uses_selected_strip_identities_and_thirty_track_blocks() {
    // A deliberately disclosed qualification strip series, not a historical
    // key sheet: each track has a full alphabet and one26-letter displacement.
    let strips = (0..100)
        .map(|i| {
            if i % 2 == 0 {
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
            } else {
                "AZYXWVUTSRQPONMLKJIHGFEDCB"
            }
        })
        .collect::<Vec<_>>();
    let order = (1..=30).collect::<Vec<_>>();
    let p = json!({"profile":"m138a_configured_v1","strip_series":"qualification:alternating-v1","strips":strips,"order":order,"row_offsets":[1,2]});
    let r = invoke(
        "cipher.m138a.encode",
        1,
        &json!({"text":"A".repeat(31)}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(r["text"], "BZ".repeat(15) + "C");
    let d = invoke(
        "cipher.m138a.decode",
        1,
        &json!({"text":r["text"]}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(d["text"], "A".repeat(31));
}

#[test]
fn configured_wheels_reject_incomplete_alphabets_ids_and_rows() {
    let base = json!({"profile":"jefferson_configured_v1","wheels":["ABCDEFGHIJKLMNOPQRSTUVWXYZ","AZYXWVUTSRQPONMLKJIHGFEDCB"],"order":[1,2],"row_offsets":[1]});
    for (field, value) in [
        ("wheels", json!([])),
        ("wheels", json!(["A", "ABCDEFGHIJKLMNOPQRSTUVWXYZ"])),
        (
            "wheels",
            json!(["ABCDEFGHIJKLMNOPQRSTUVWXYA", "ABCDEFGHIJKLMNOPQRSTUVWXYZ"]),
        ),
        ("order", json!([1, 1])),
        ("order", json!([0, 2])),
        ("order", json!([1, 3])),
        ("order", json!([1])),
        ("row_offsets", json!([])),
        ("row_offsets", json!([0])),
        ("row_offsets", json!([26])),
    ] {
        let mut p = base.clone();
        p[field] = value;
        assert!(
            invoke(
                "cipher.jefferson.encode",
                1,
                &json!({"text":"AA"}),
                &p,
                Budget::default()
            )
            .is_err(),
            "{field}"
        );
    }
    let mut p: disks::JeffersonParameters = serde_json::from_value(base).unwrap();
    p.row_offsets = vec![1; 65536];
    let text = "A".repeat(131072);
    assert_eq!(
        disks::jefferson_encode(&text, &p).unwrap().text.len(),
        text.len()
    );
    p.row_offsets.push(1);
    assert_eq!(
        disks::jefferson_encode(&text, &p).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    p.wheels = vec!["ABCDEFGHIJKLMNOPQRSTUVWXYZ".into(); 64];
    p.order = (1..=64).collect();
    p.row_offsets = vec![1];
    assert_eq!(disks::jefferson_encode("A", &p).unwrap().text, "B");
    p.wheels.push("ABCDEFGHIJKLMNOPQRSTUVWXYZ".into());
    p.order.push(65);
    assert!(disks::jefferson_encode("A", &p).is_err());
}

#[test]
fn m138a_requires_the_complete_named_series_and_distinct_selected_strips() {
    let base = json!({"profile":"m138a_configured_v1","strip_series":"synthetic:qualification","strips":vec!["ABCDEFGHIJKLMNOPQRSTUVWXYZ";100],"order":(1..=30).collect::<Vec<_>>(),"row_offsets":[1]});
    for (field, value) in [
        ("profile", json!("m138")),
        ("strip_series", json!("")),
        ("strip_series", json!("unknown series")),
        ("strip_series", json!("é")),
        ("strip_series", json!("a".repeat(129))),
        ("strips", json!(vec!["ABCDEFGHIJKLMNOPQRSTUVWXYZ"; 99])),
        ("order", json!(vec![1; 30])),
        ("order", json!((0..30).collect::<Vec<_>>())),
        ("order", json!((72..102).collect::<Vec<_>>())),
    ] {
        let mut p = base.clone();
        p[field] = value;
        assert!(
            invoke(
                "cipher.m138a.encode",
                1,
                &json!({"text":"A"}),
                &p,
                Budget::default()
            )
            .is_err(),
            "{field}"
        );
    }
    let mut p: disks::M138aParameters = serde_json::from_value(base).unwrap();
    p.order = (71..=100).collect::<Vec<_>>().try_into().unwrap();
    p.strip_series = "a".repeat(128);
    p.row_offsets = vec![25; mechanical::MAX_INPUT.div_ceil(30)];
    assert_eq!(
        disks::m138a_encode(&"A".repeat(mechanical::MAX_INPUT), &p)
            .unwrap()
            .text,
        "Z".repeat(mechanical::MAX_INPUT)
    );
    assert_eq!(
        disks::m138a_encode(&"A".repeat(mechanical::MAX_INPUT + 1), &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
