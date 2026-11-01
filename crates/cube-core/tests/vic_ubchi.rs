use cube_core::ciphers::{
    fractionation::vic::{self, VicEnglish2010},
    transposition::ubchi::{self, Ubchi},
};
use cube_core::errors::ErrorCode;
use cube_core::operations::execution::{Budget, execute};
use serde_json::json;

fn parameters() -> VicEnglish2010 {
    serde_json::from_value(json!({"phrase":"ALLTHEPEOPLEAREDEADB","mnemonic":"ASINTOER","date_digits":"391752","personal_number":15,"message_indicator":"60115","split_at":14,"padding":""})).unwrap()
}

#[test]
fn lanaki_1997_ubchi_full_message_and_exact_between_pass_null_inverse() {
    let p = json!({"key_phrase":"HERRSCHAFT","nulls":"Z"});
    let plain = "FIRSTARMYXPLANFIVEACTIVATEDXCROSSMARNEATSETHOUR";
    let cipher = "RARHZIADAREIOSAUFRTMLTVTECNMTXSSTOEERSXAYVNCIPAF";
    for (direction, input, expected) in [("encode", plain, cipher), ("decode", cipher, plain)] {
        assert_eq!(
            execute(
                &format!("cipher.ubchi.{direction}"),
                1,
                &json!({"text":input}),
                &p,
                Budget::default()
            )
            .unwrap(),
            json!({"text":expected,"original_length":47,"null_count":1})
        );
    }
}

#[test]
fn published_vic_english_2010_complete_message_and_inverse() {
    let p = json!({"phrase":"ALLTHEPEOPLEAREDEADB","mnemonic":"ASINTOER","date_digits":"391752","personal_number":15,"message_indicator":"60115","split_at":14,"padding":""});
    let plain = "ASSIGNEDOBJECTIVESINVALIDATED.REPORTIMMEDIATELYTOSAFEHOUSE.AWAITEXTRACTIONINSTRUCTIONSWITHINWEEK";
    let cipher = "439836529332548692543593224039193759123412656674021687356133550753351136926066081424184749148634617711450653261143330085374956011529149";
    for (direction, input, expected) in [("encode", plain, cipher), ("decode", cipher, plain)] {
        assert_eq!(
            execute(
                &format!("cipher.vic.{direction}"),
                1,
                &json!({"text":input}),
                &p,
                Budget::default()
            )
            .unwrap(),
            json!({"text":expected})
        );
    }
}

#[test]
fn vic_explicit_split_and_pad_evidence_preserves_literal_period_runs_and_empty_text() {
    for (plain, padding) in [
        ("", "0"),
        ("A", ""),
        ("B", "1234"),
        ("A..", "0"),
        ("A...B", "01"),
        ("HELLO.WORLD", "123"),
    ] {
        for split_at in 0..=plain.len() {
            let mut p = parameters();
            p.split_at = split_at;
            p.padding = padding.into();
            let cipher = vic::transform(plain, &p, false).unwrap();
            assert_eq!(vic::transform(&cipher, &p, true).unwrap(), plain);
        }
    }
    let mut p = parameters();
    p.split_at = 0;
    p.padding = "1234".into();
    let cipher = vic::transform("B", &p, false).unwrap();
    p.padding = "1235".into();
    assert!(vic::transform(&cipher, &p, true).is_err());
    p.padding = "1234".into();
    p.message_indicator = "00000".into();
    assert!(vic::transform(&cipher, &p, true).is_err());
}

#[test]
fn vic_zero_date_digit_is_explicitly_tenth_group_and_short_frames_reject() {
    let mut p = parameters();
    p.date_digits = "391750".into();
    p.split_at = 0;
    let source = "A".repeat(41);
    let cipher = vic::transform(&source, &p, false).unwrap();
    assert!(cipher.starts_with("60115"));
    assert_eq!(vic::transform(&cipher, &p, true).unwrap(), source);
    assert!(vic::transform("A", &p, false).is_err());
    for cipher in ["", "00000", "000000", "00000 00000", "１２３４５"] {
        assert!(vic::transform(cipher, &p, true).is_err());
    }
}

#[test]
fn vic_exact_source_and_cipher_ceiling_and_redacted_invalid_keys_are_enforced() {
    let mut p = parameters();
    p.split_at = 0;
    p.padding = "123".into();
    let source = "B".repeat(65534);
    let cipher = vic::transform(&source, &p, false).unwrap();
    assert_eq!(cipher.len(), 131080);
    assert_eq!(vic::transform(&cipher, &p, true).unwrap(), source);
    assert_eq!(
        vic::transform(&"B".repeat(65535), &p, false)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        vic::transform(&"0".repeat(131081), &p, true)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    p.padding = "".into();
    for plain in ["lowercase", "A A", "123", "#", "é", "🐈"] {
        assert!(vic::transform(plain, &p, false).is_err());
    }
    let mut bad = p.clone();
    bad.phrase = "A PRIVATE SECRET".into();
    let error = vic::transform("A", &bad, false).unwrap_err();
    assert!(!error.to_string().contains(&bad.phrase));
    bad = p.clone();
    bad.mnemonic = "AAAAAAAA".into();
    assert!(vic::transform("A", &bad, false).is_err());
    bad = p.clone();
    bad.date_digits = "39175X".into();
    assert!(vic::transform("A", &bad, false).is_err());
    bad = p.clone();
    bad.message_indicator = "1234".into();
    assert!(vic::transform("A", &bad, false).is_err());
    for number in [0, 21, 25] {
        bad = p.clone();
        bad.personal_number = number;
        assert!(vic::transform("A", &bad, false).is_err());
    }
    bad = p.clone();
    bad.padding = "12345".into();
    assert!(vic::transform("A", &bad, false).is_err());
    bad = p.clone();
    bad.split_at = 65535;
    assert!(vic::transform("A", &bad, false).is_err());
    let mut unknown = serde_json::to_value(p).unwrap();
    unknown["profile"] = json!("russian");
    assert!(
        execute(
            "cipher.vic.encode",
            1,
            &json!({"text":"A"}),
            &unknown,
            Budget::default()
        )
        .is_err()
    );
}

#[test]
fn ubchi_counts_actual_phrase_words_and_validates_only_the_between_pass_suffix() {
    for source in ["", "X🐈e\u{301}\0Z", "NULLSAREDATAZZ"] {
        let p = Ubchi {
            key_phrase: "AB AB".into(),
            nulls: "🐈Z".into(),
        };
        let out = ubchi::transform(source, &p, false).unwrap();
        assert_eq!(out.null_count, 2);
        assert_eq!(ubchi::transform(&out.text, &p, true).unwrap().text, source);
        let wrong = Ubchi {
            key_phrase: p.key_phrase,
            nulls: "🐈Y".into(),
        };
        assert!(ubchi::transform(&out.text, &wrong, true).is_err());
    }
    for phrase in ["", "a", " A", "A ", "A  B", "É"] {
        assert!(
            ubchi::transform(
                "",
                &Ubchi {
                    key_phrase: phrase.into(),
                    nulls: "Z".into()
                },
                false
            )
            .is_err()
        );
    }
    for nulls in ["", "ZZ", "e\u{301}"] {
        assert!(
            ubchi::transform(
                "",
                &Ubchi {
                    key_phrase: "A".into(),
                    nulls: nulls.into()
                },
                false
            )
            .is_err()
        );
    }
    assert!(
        ubchi::transform(
            "",
            &Ubchi {
                key_phrase: "A".into(),
                nulls: "Z".into()
            },
            true
        )
        .is_err()
    );
}

#[test]
fn ubchi_limits_charge_inserted_nulls_and_preserve_exact_unicode_scalars() {
    let p = Ubchi {
        key_phrase: "A".into(),
        nulls: "🐈".into(),
    };
    let source = "B".repeat(65535);
    let out = ubchi::transform(&source, &p, false).unwrap();
    assert_eq!(out.text.chars().count(), 65536);
    assert_eq!(ubchi::transform(&out.text, &p, true).unwrap().text, source);
    assert_eq!(
        ubchi::transform(&"B".repeat(65536), &p, false)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let p = Ubchi {
        key_phrase: vec!["A"; 4096].join(" "),
        nulls: "🐈".repeat(4096),
    };
    let out = ubchi::transform("", &p, false).unwrap();
    assert_eq!(out.null_count, 4096);
    assert_eq!(ubchi::transform(&out.text, &p, true).unwrap().text, "");
    let too_large = Ubchi {
        key_phrase: vec!["A"; 4097].join(" "),
        nulls: "Z".repeat(4097),
    };
    assert!(ubchi::transform("", &too_large, false).is_err());
}
