use cube_core::operations::execution::{self, Budget};
use serde_json::json;

#[test]
fn exact_fictional_editions_execute_independent_known_answers_and_ambiguity() {
    for (profile, units, tokens) in [
        (
            "al_bhed_english_ffx_v1",
            vec!["Y", "O", "U"],
            vec!["O", "U", "I"],
        ),
        (
            "hylian_skyward_dcode_v1",
            vec!["D", "W", "E", "K"],
            vec!["hylian:r1c4", "hylian:r1c4", "hylian:r1c5", "hylian:r1c5"],
        ),
        (
            "sheikah_botw_dcode_v1",
            vec!["Z", "0", "9", "!", "-", ".", "?"],
            vec![
                "sheikah:r7c2",
                "sheikah:r7c3",
                "sheikah:r9c4",
                "sheikah:r10c1",
                "sheikah:r10c2",
                "sheikah:r10c3",
                "sheikah:r10c4",
            ],
        ),
        (
            "daedric_names_dcode_v1",
            vec!["N", "X", "Y", "Z"],
            vec!["Neht", "Xayah", "Yahkem", "Zyr"],
        ),
        (
            "futurama_alien1_dcode_v1",
            vec!["Y", "Z", "0", "9"],
            vec!["alien1:r5c1", "alien1:r5c2", "alien1:r5c3", "alien1:r6c6"],
        ),
        (
            "bill_cipher_dcode_v1",
            vec!["B", "I", "L", "L", "Z"],
            vec![
                "bill:r1c2",
                "bill:r2c2",
                "bill:r2c5",
                "bill:r2c5",
                "bill:r4c5",
            ],
        ),
        (
            "gnommish_disney_2020_v1",
            vec!["A", "M", "N", "Z"],
            vec![
                "gnommish:r1c1",
                "gnommish:r1c13",
                "gnommish:r2c1",
                "gnommish:r2c13",
            ],
        ),
    ] {
        let p = json!({"profile": profile});
        let encoded = execution::execute(
            "symbol.alphabet.encode",
            1,
            &json!({"units":units}),
            &p,
            Budget::default(),
        )
        .unwrap();
        assert_eq!(encoded["tokens"], json!(tokens), "{profile}");
        let decoded = execution::execute(
            "symbol.alphabet.decode",
            1,
            &json!({"tokens":tokens}),
            &p,
            Budget::default(),
        )
        .unwrap();
        assert_eq!(decoded["tokens"], json!(tokens));
        for (i, u) in units.iter().enumerate() {
            let expected = match (profile, *u) {
                ("hylian_skyward_dcode_v1", "D" | "W") => json!(["D", "W"]),
                ("hylian_skyward_dcode_v1", "E" | "K") => json!(["E", "K"]),
                _ => json!([u]),
            };
            assert_eq!(decoded["readings"][i]["alternatives"], expected);
            assert_eq!(decoded["readings"][i]["token_index"], json!(i));
        }
    }
    for (token, expected) in [
        ("hylian:r3c5", ["D", "W"]),
        ("hylian:r2c2", ["E", "K"]),
        ("hylian:r2c8", ["G", "Q"]),
        ("hylian:r3c6", ["I", "X"]),
        ("hylian:r3c8", ["O", "Z"]),
        ("hylian:r3c2", ["P", "T"]),
    ] {
        let d = execution::execute(
            "symbol.alphabet.decode",
            1,
            &json!({"tokens":[token]}),
            &json!({"profile":"hylian_skyward_dcode_v1"}),
            Budget::default(),
        )
        .unwrap();
        assert_eq!(d["tokens"], json!([token]));
        assert_eq!(d["readings"][0]["alternatives"], json!(expected));
    }
}

#[test]
fn fictional_profiles_reject_other_editions_and_implicit_normalization() {
    use cube_core::{errors::ErrorCode, symbols::alphabet as a};
    for profile in [
        "al_bhed_english_ffx_v1",
        "hylian_skyward_dcode_v1",
        "sheikah_botw_dcode_v1",
        "daedric_names_dcode_v1",
        "futurama_alien1_dcode_v1",
        "bill_cipher_dcode_v1",
        "gnommish_disney_2020_v1",
    ] {
        let p = serde_json::from_value(json!({"profile":profile})).unwrap();
        for u in ["a", " ", "AB", "é", "", "10"] {
            let e = a::encode(&[u.into()], &p).unwrap_err();
            assert_eq!(e.code, ErrorCode::InvalidInput, "{profile}: {u}");
            assert_eq!(e.parameter, Some("units"));
        }
    }
    for (profile, tokens) in [
        (
            "hylian_skyward_dcode_v1",
            vec!["hylian:r3c9", "hylian:r0c1", "sheikah:r1c1", "D"],
        ),
        (
            "sheikah_botw_dcode_v1",
            vec!["sheikah:r11c1", "sheikah:r10c5", "sheikah:r1c1:filled"],
        ),
        (
            "daedric_names_dcode_v1",
            vec!["Nehk", "ayem", "Ayem Bedt", "1"],
        ),
        (
            "futurama_alien1_dcode_v1",
            vec!["alien1:r6c7", "alien2:r1c1", "."],
        ),
        (
            "bill_cipher_dcode_v1",
            vec!["bill:r4c6", "author:r1c1", "journal3:r1c1"],
        ),
        (
            "gnommish_disney_2020_v1",
            vec!["gnommish:r1c14", "gnommish:r3c1", "1"],
        ),
        ("al_bhed_english_ffx_v1", vec!["y", "わ", "OUI", "!"]),
    ] {
        let p = serde_json::from_value(json!({"profile":profile})).unwrap();
        for t in tokens {
            assert_eq!(
                a::decode(&[t.into()], &p).unwrap_err().code,
                ErrorCode::InvalidInput,
                "{profile}: {t}"
            );
        }
    }
}
