use cube_core::{
    ciphers::{
        self,
        poly::{self, key_profiles::*},
    },
    errors::ErrorCode,
};

#[test]
fn published_aca_and_rs58356_answers_and_inverses() {
    let text = "THISCIPHERCANBEUSEDWITHANYOFTHEPERIODICS";
    let interrupted = InterruptedKey {
        key: "ORANGE".into(),
        resets: vec![4, 10, 12, 15, 19, 22, 23, 24, 29, 30, 32, 35],
    };
    let ciphertext = "HYIFQZPUKVQRBSEIJEQKZTVOBPOSZVSGSIICUIPY";
    assert_eq!(
        interrupted_key(text, &interrupted, false).unwrap(),
        ciphertext
    );
    assert_eq!(
        interrupted_key(ciphertext, &interrupted, true).unwrap(),
        text
    );
    let progressive = ProgressiveKey {
        key: "GRAPEFRUIT".into(),
        increment: 1,
    };
    let ciphertext = "ZYIHGNGBMKJSORJAKZMQQMJRTFHBDCNJHJPWXFNO";
    assert_eq!(
        progressive_key(text, &progressive, false).unwrap(),
        ciphertext
    );
    assert_eq!(
        progressive_key(ciphertext, &progressive, true).unwrap(),
        text
    );
    let mult = Multiplicative {
        key: "MOBILE".into(),
        multiplier: 7,
    };
    assert_eq!(
        multiplicative("IAMSTUDYING", &mult, false).unwrap(),
        "KUNACMBGLRP"
    );
    assert_eq!(
        multiplicative("KUNACMBGLRP", &mult, true).unwrap(),
        "IAMSTUDYING"
    );
    assert_eq!(
        multiplicative("I am studying. é\0", &mult, false).unwrap(),
        "K un acmbglrp. é\0"
    );
}

#[test]
fn interruption_offsets_count_only_ascii_letters_and_validate_before_output() {
    let mut p = InterruptedKey {
        key: "BC".into(),
        resets: vec![0, 2],
    };
    assert_eq!(
        interrupted_key("a éa\0 a-a", &p, false).unwrap(),
        "b éc\0 b-c"
    );
    assert_eq!(
        interrupted_key("b éc\0 b-c", &p, true).unwrap(),
        "a éa\0 a-a"
    );
    for resets in [vec![1, 1], vec![2, 1], vec![4], vec![u32::MAX]] {
        p.resets = resets;
        let e = interrupted_key("AAAA", &p, false).unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidParameter);
        assert_eq!(e.parameter, Some("resets"));
    }
    p.resets = vec![];
    assert_eq!(interrupted_key("", &p, false).unwrap(), "");
    assert_eq!(interrupted_key("é!", &p, false).unwrap(), "é!");
    p.resets = vec![0];
    assert!(interrupted_key("é!", &p, false).is_err());
    assert!(interrupted_key("", &p, false).is_err());
}

#[test]
fn progressive_groups_partial_tails_and_signed_extremes_match_wide_arithmetic() {
    let mut p = ProgressiveKey {
        key: "AB".into(),
        increment: -1,
    };
    assert_eq!(progressive_key("AAAAAA", &p, false).unwrap(), "ABZAYZ");
    p.key = "BC".into();
    p.increment = 1;
    assert_eq!(progressive_key("AAAAA", &p, false).unwrap(), "BCCDD");
    assert_eq!(
        progressive_key("a éa\0a-a", &p, false).unwrap(),
        "b éc\0c-d"
    );
    for increment in [i32::MIN, i32::MAX, -27, -26, 0, 26, 27] {
        p.increment = increment;
        let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let expected: String = text
            .bytes()
            .enumerate()
            .map(|(i, x)| {
                let key = i64::from(p.key.as_bytes()[i % 2] - b'A');
                char::from(
                    b'A' + (i64::from(x - b'A') + key + i64::from(increment) * (i / 2) as i64)
                        .rem_euclid(26) as u8,
                )
            })
            .collect();
        assert_eq!(progressive_key(text, &p, false).unwrap(), expected);
        assert_eq!(progressive_key(&expected, &p, true).unwrap(), text);
    }
}

#[test]
fn multiplicative_uses_independent_affine_oracle_for_every_key_letter_and_unit() {
    let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz é!\0";
    for a in 0..=255u8 {
        let mut p = Multiplicative {
            key: "A".into(),
            multiplier: a,
        };
        if ![1, 3, 5, 7, 9, 11, 15, 17, 19, 21, 23, 25].contains(&a) {
            let e = multiplicative("", &p, false).unwrap_err();
            assert_eq!(e.parameter, Some("multiplier"));
            continue;
        }
        for k in 0..26u8 {
            p.key = char::from(b'A' + k).to_string();
            let expected =
                ciphers::affine_encode(text, i32::from(a), (i32::from(a) * i32::from(k)) % 26)
                    .unwrap();
            assert_eq!(multiplicative(text, &p, false).unwrap(), expected);
            assert_eq!(multiplicative(&expected, &p, true).unwrap(), text);
        }
    }
}

#[test]
fn exact_and_one_over_input_key_and_reset_limits() {
    let mut p = InterruptedKey {
        key: "B".into(),
        resets: (0..MAX_RESETS as u32).collect(),
    };
    assert_eq!(
        interrupted_key(&"A".repeat(MAX_RESETS), &p, false).unwrap(),
        "B".repeat(MAX_RESETS)
    );
    p.resets.push(MAX_RESETS as u32);
    assert_eq!(
        interrupted_key(&"A".repeat(MAX_RESETS + 1), &p, false)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    p.resets.clear();
    p.key = "A".repeat(poly::MAX_KEY);
    assert_eq!(interrupted_key("", &p, false).unwrap(), "");
    let exact = "é".repeat(poly::MAX_INPUT / 2);
    assert_eq!(interrupted_key(&exact, &p, false).unwrap(), exact);
    assert_eq!(
        interrupted_key(&(exact + "A"), &p, false).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    for key in [
        String::new(),
        "A!".into(),
        "é".into(),
        "A".repeat(poly::MAX_KEY + 1),
    ] {
        p.key = key.clone();
        assert!(interrupted_key("", &p, false).is_err());
        assert!(
            progressive_key(
                "",
                &ProgressiveKey {
                    key: key.clone(),
                    increment: 0
                },
                false
            )
            .is_err()
        );
        assert!(multiplicative("", &Multiplicative { key, multiplier: 1 }, false).is_err());
    }
    let p = ProgressiveKey {
        key: "B".into(),
        increment: 0,
    };
    assert_eq!(
        progressive_key(&"A".repeat(poly::MAX_INPUT), &p, false).unwrap(),
        "B".repeat(poly::MAX_INPUT)
    );
    assert!(progressive_key(&"A".repeat(poly::MAX_INPUT + 1), &p, false).is_err());
    let p = Multiplicative {
        key: "A".into(),
        multiplier: 1,
    };
    assert_eq!(
        multiplicative(&"Z".repeat(poly::MAX_INPUT), &p, false).unwrap(),
        "Z".repeat(poly::MAX_INPUT)
    );
    assert!(multiplicative(&"Z".repeat(poly::MAX_INPUT + 1), &p, false).is_err());
}

#[test]
fn strict_parameters_reject_unknown_duplicate_missing_and_wrong_typed_fields() {
    for document in [
        r#"{"key":"A"}"#,
        r#"{"key":"A","resets":[],"unknown":true}"#,
        r#"{"key":"A","key":"B","resets":[]}"#,
        r#"{"key":"A","resets":[-1]}"#,
    ] {
        assert!(poly::parse_parameters::<InterruptedKey>(document).is_err());
    }
    assert!(poly::parse_parameters::<ProgressiveKey>(r#"{"key":"A"}"#).is_err());
    assert!(
        poly::parse_parameters::<ProgressiveKey>(r#"{"key":"A","increment":2147483648}"#).is_err()
    );
    assert!(poly::parse_parameters::<Multiplicative>(r#"{"key":"A"}"#).is_err());
    assert!(poly::parse_parameters::<Multiplicative>(r#"{"key":"A","multiplier":"7"}"#).is_err());
}

#[test]
fn versioned_contracts_validate_real_answers_and_reject_invalid_typed_parameters() {
    use cube_core::operations::{self, contracts};
    use serde_json::json;
    let fixtures = contracts::registered_fixtures().unwrap();
    let mut covered = 0;
    for (id, fixture) in fixtures {
        if ![
            "cipher.interrupted_key.",
            "cipher.progressive_key.",
            "cipher.vigenere_multiplicative.",
        ]
        .iter()
        .any(|prefix| id.starts_with(prefix))
        {
            continue;
        }
        let contract = operations::require(&id).unwrap().contract().unwrap();
        assert_eq!(contract.operation_version, 1);
        contract.validate_fixture(&fixture).unwrap();
        assert!(contract.validate_parameters(&json!({})).is_err());
        let mut extra = fixture.parameters.clone();
        extra["unknown"] = json!(true);
        assert!(contract.validate_parameters(&extra).is_err());
        let bad = if id.starts_with("cipher.interrupted_key.") {
            json!({"key":"A","resets":[1048576]})
        } else if id.starts_with("cipher.progressive_key.") {
            json!({"key":"A","increment":2147483648_i64})
        } else {
            json!({"key":"A","multiplier":2})
        };
        assert!(contract.validate_parameters(&bad).is_err(), "{id}");
        assert!(contract.validate_outputs(&json!({"text":true})).is_err());
        covered += 1;
    }
    assert_eq!(covered, 6);
}
