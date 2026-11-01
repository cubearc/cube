//! Known assignments independently checked against the dated UCSUR proposals.
use cube_core::operations::execution::{self, Budget};
use serde_json::json;

#[test]
fn aurebesh_digraphs_and_standard_galactic_are_distinct_executable_profiles() {
    for (profile, units, expected) in [
        (
            "aurebesh_ucsur_2016_v1",
            vec!["A", "CH", "C", "H", "Z", "."],
            vec![
                "\u{e890}", "\u{e8aa}", "\u{e892}", "\u{e897}", "\u{e8a9}", "\u{e8c1}",
            ],
        ),
        (
            "standard_galactic_ucsur_2019_v1",
            vec!["K", "E", "E", "N", "."],
            vec!["\u{eb4a}", "\u{eb44}", "\u{eb44}", "\u{eb4d}", "\u{eb5f}"],
        ),
    ] {
        let parameters = json!({"profile":profile});
        let encoded = execution::execute(
            "symbol.alphabet.encode",
            1,
            &json!({"units":units}),
            &parameters,
            Budget::default(),
        )
        .unwrap();
        assert_eq!(encoded["tokens"], json!(expected));
        let decoded = execution::execute(
            "symbol.alphabet.decode",
            1,
            &json!({"tokens":expected}),
            &parameters,
            Budget::default(),
        )
        .unwrap();
        for (index, unit) in units.iter().enumerate() {
            assert_eq!(decoded["readings"][index]["alternatives"], json!([unit]));
            assert_eq!(decoded["readings"][index]["token_index"], index);
        }
        assert_eq!(decoded["tokens"], json!(expected));
        assert_eq!(encoded["provenance"], decoded["provenance"]);
    }
}

#[test]
fn every_published_assignment_executes_in_both_directions() {
    let vectors: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/symbols/alphabet/vectors.json")).unwrap();
    for row in vectors {
        let p = json!({"profile":row["profile"]});
        let encoded = execution::execute(
            "symbol.alphabet.encode",
            1,
            &json!({"units":row["units"]}),
            &p,
            Budget::default(),
        )
        .unwrap();
        assert_eq!(encoded["tokens"], row["tokens"]);
        let decoded = execution::execute(
            "symbol.alphabet.decode",
            1,
            &json!({"tokens":row["tokens"]}),
            &p,
            Budget::default(),
        )
        .unwrap();
        for (index, unit) in row["units"].as_array().unwrap().iter().enumerate() {
            let alternatives = row
                .get("alternatives")
                .and_then(|a| a.get(index))
                .cloned()
                .unwrap_or_else(|| json!([unit]));
            assert_eq!(decoded["readings"][index]["alternatives"], alternatives);
            assert_eq!(decoded["readings"][index]["token_index"], index);
        }
    }
}
fn parameters() -> cube_core::symbols::alphabet::Parameters {
    serde_json::from_value(json!({"profile":"aurebesh_ucsur_2016_v1"})).unwrap()
}
#[test]
fn unsupported_units_reserved_positions_and_implicit_profiles_reject() {
    use cube_core::{errors::ErrorCode, symbols::alphabet as a};
    let p = parameters();
    for unit in ["a", "Ch", "CHAT", "0", "é", "e\u{301}", ""] {
        let e = a::encode(&[unit.into()], &p).unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidInput);
        assert_eq!(e.operation, Some("symbol.alphabet.encode"));
    }
    for (profile, tokens) in [
        (
            "aurebesh_ucsur_2016_v1",
            vec!["\u{e8b2}", "\u{e8cf}", "\u{e8d0}", "\u{eb40}", "A"],
        ),
        (
            "standard_galactic_ucsur_2019_v1",
            vec!["\u{eb5a}", "\u{eb5b}", "\u{eb5c}", "\u{e890}", "?", "0"],
        ),
    ] {
        let p = serde_json::from_value(json!({"profile":profile})).unwrap();
        for token in tokens {
            let e = a::decode(&[token.into()], &p).unwrap_err();
            assert_eq!(e.code, ErrorCode::InvalidInput);
            assert_eq!(e.operation, Some("symbol.alphabet.decode"));
        }
    }
    for p in [
        json!({}),
        json!({"profile":"aurebesh"}),
        json!({"profile":"aurebesh_ucsur_2016_v1","font":"guess"}),
    ] {
        assert!(
            execution::execute(
                "symbol.alphabet.encode",
                1,
                &json!({"units":[]}),
                &p,
                Budget::default()
            )
            .is_err()
        );
    }
    for inputs in [
        json!({"units":[],"extra":[]}),
        json!({"units":[0]}),
        json!({"tokens":[]}),
    ] {
        assert!(
            execution::execute(
                "symbol.alphabet.encode",
                1,
                &inputs,
                &json!({"profile":"aurebesh_ucsur_2016_v1"}),
                Budget::default()
            )
            .is_err()
        );
    }
}
#[test]
fn exact_empty_repeated_data_and_limits_never_truncate() {
    use cube_core::{errors::ErrorCode, symbols::alphabet as a};
    let p = parameters();
    let encoded = a::encode(&[], &p).unwrap();
    let decoded = a::decode(&[], &p).unwrap();
    assert!(encoded.tokens.0.is_empty() && decoded.readings.is_empty());
    assert_eq!(
        encoded.provenance.definition_sha256,
        decoded.provenance.definition_sha256
    );
    assert_eq!(
        encoded.provenance.source_sha256,
        "d83267b3d7ca4bf82558b3927a9f86d900c729c04bf312601f91696452b5922b"
    );
    let repeated = a::decode(&["\u{e890}".into(), " ".into(), "\u{e890}".into()], &p).unwrap();
    assert_eq!(repeated.readings[2].token_index, 2);
    assert_eq!(
        a::encode(&vec!["A".into(); 65536], &p)
            .unwrap()
            .tokens
            .0
            .len(),
        65536
    );
    assert_eq!(
        a::encode(&vec!["A".into(); 65537], &p).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        a::decode(&vec!["\u{e890}".into(); 30000], &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        a::decode(&["x".repeat(257)], &p).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        a::decode(&["x".repeat(1048576)], &p).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn geometric_profiles_use_source_orientations_markers_and_clock_origin() {
    for (profile, units, tokens) in [
        (
            "pigpen_haverhill_pairs_v1",
            vec!["A", "B", "S", "T", "W"],
            vec![
                "grid:ES",
                "grid:ES:dot",
                "wedge:north",
                "wedge:north:dot",
                "wedge:east",
            ],
        ),
        (
            "tic_tac_toe_dcode_v1",
            vec!["A", "J", "S", "Z", " "],
            vec![
                "grid:ES",
                "grid:ES:cross",
                "grid:ES:circle",
                "grid:NEW:circle",
                "grid:NW:circle",
            ],
        ),
        (
            "templars_bothwell_gosse_v1",
            vec!["A", "O", "E", "V", "K", "Y", "N"],
            vec![
                "angle:south",
                "angle:south:dot",
                "triangle:east",
                "triangle:east:dot",
                "lozenge:east-arm",
                "lozenge:east-arm:dot",
                "saltire",
            ],
        ),
        (
            "betamaze_cliff_2006_v1",
            vec!["a", "m", "z", " "],
            vec!["#CB/..C/C.#", "ADB/B.C/CAD", "D.C/.../B.A", "A.B/.../C.D"],
        ),
        (
            "clock_quarter_hour_zero_v1",
            vec!["A", "B", "M", "Y", "Z"],
            vec!["00:00", "00:15", "03:00", "06:00", "06:15"],
        ),
        (
            "birds_nrtech_four_features_v1",
            vec!["H", "E", "L", "L", "O"],
            vec![
                "upright:left:flat:raised",
                "upright:right:up:raised",
                "forward:left:up:raised",
                "forward:left:up:raised",
                "forward:right:flat:raised",
            ],
        ),
    ] {
        let p = json!({"profile":profile});
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
        for (index, unit) in units.iter().enumerate() {
            assert_eq!(decoded["readings"][index]["alternatives"], json!([unit]));
        }
    }
}

#[test]
fn source_listed_aliases_preserve_received_forms_and_all_readings() {
    let aliases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/symbols/alphabet/aliases.json")).unwrap();
    assert_eq!(aliases.len(), 181); // Betamaze121 + birds26 + Enochian26 + Samech1 + Kaldeorum7.
    for row in aliases {
        let p = json!({"profile":row["profile"]});
        let decoded = execution::execute(
            "symbol.alphabet.decode",
            1,
            &json!({"tokens":[row["token"]]}),
            &p,
            Budget::default(),
        )
        .unwrap();
        assert_eq!(decoded["tokens"], json!([row["token"]]));
        assert_eq!(decoded["readings"][0]["alternatives"], row["alternatives"]);
    }
}
#[test]
fn unrelated_dot_cross_circle_and_clock_variants_are_not_guessed() {
    for (profile, tokens) in [
        (
            "pigpen_haverhill_pairs_v1",
            vec![
                "grid:ES:circle",
                "grid:ES:cross",
                "grid:SE",
                "wedge:east:cross",
            ],
        ),
        (
            "tic_tac_toe_dcode_v1",
            vec!["grid:ES:dot", "wedge:north", "grid:ES:Circle"],
        ),
        (
            "templars_bothwell_gosse_v1",
            vec![
                "angle:north:cross",
                "triangle:north:circle",
                "lozenge:north",
                "cross",
            ],
        ),
        (
            "betamaze_cliff_2006_v1",
            vec![".../.../...", "#CB/..C/C.# ", "ABC/DEF/GHI", "#CB/..C/C.."],
        ),
        (
            "clock_quarter_hour_zero_v1",
            vec!["12:00", "00:0", "0:00", "06:30", "00:01", " "],
        ),
        (
            "birds_nrtech_four_features_v1",
            vec![
                "B00",
                "B27",
                "B1",
                "b01",
                "back:left:up:raised",
                "left:back:up:folded",
            ],
        ),
    ] {
        for token in tokens {
            assert!(
                execution::execute(
                    "symbol.alphabet.decode",
                    1,
                    &json!({"tokens":[token]}),
                    &json!({"profile":profile}),
                    Budget::default()
                )
                .is_err(),
                "{profile}: {token}"
            );
        }
    }
    let p: cube_core::symbols::alphabet::Parameters =
        serde_json::from_value(json!({"profile":"templars_bothwell_gosse_v1"})).unwrap();
    assert!(cube_core::symbols::alphabet::encode(&["J".into()], &p).is_err());
    let p = json!({"profile":"pigpen_haverhill_pairs_v1"});
    let r=execution::execute("symbol.alphabet.decode",1,&json!({"tokens":["grid:NSW:dot","grid:NESW","wedge:north","wedge:north:dot","grid:SW","grid:NE:dot"]}),&p,Budget::default()).unwrap();
    assert_eq!(
        r["readings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["alternatives"][0].as_str().unwrap())
            .collect::<String>(),
        "LISTEN"
    );
    let p = json!({"profile":"birds_nrtech_four_features_v1"});
    let r = execution::execute(
        "symbol.alphabet.decode",
        1,
        &json!({"tokens":["B08","B05","B12","B12","B15"]}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(
        r["readings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["alternatives"][0].as_str().unwrap())
            .collect::<String>(),
        "HELLO"
    );
}

#[test]
fn historical_tables_use_actual_repertoires_and_preserve_flags_and_ambiguity() {
    for (profile, units, tokens) in [
        (
            "theban_trithemius_1518_v1",
            vec!["A", "I", "V", "OMEGA"],
            vec!["theban:r1c1", "theban:r2c1", "theban:r3c4", "theban:r3c8"],
        ),
        (
            "celestial_agrippa_ch30_v1",
            vec!["א", "ט", "ק", "ת"],
            vec![
                "celestial:r1c9",
                "celestial:r1c1",
                "celestial:r3c4",
                "celestial:r3c1",
            ],
        ),
        (
            "malachim_agrippa_ch30_v1",
            vec!["א", "ס", "ת"],
            vec!["malachim:r1c7", "malachim:r3c6", "malachim:r3c9"],
        ),
        (
            "magi_ambrosius_1539_v1",
            vec!["א", "ל", "ת", "מ"],
            vec!["magi:r1c12", "magi:r1c1", "magi:r2c1", "magi:r2c10"],
        ),
        (
            "enochian_peterson_2022_v1",
            vec!["A", "I", "J", "U", "V", "Y"],
            vec![
                "enochian:r1c1",
                "enochian:r1c9",
                "enochian:r1c9",
                "enochian:r3c1",
                "enochian:r3c1",
                "enochian:r3c5",
            ],
        ),
        (
            "moon_english_1877_v1",
            vec!["A", "G", "H", "U", "V", "&", " "],
            vec![
                "moon:r1c1",
                "moon:r1c7",
                "moon:r2c1",
                "moon:r3c7",
                "moon:r4c1",
                "moon:r4c6",
                " ",
            ],
        ),
        (
            "dancing_men_doyle_1905_v1",
            vec!["A", "M ", "P", "V"],
            vec![
                "doyle:f01p01",
                "doyle:f01p02:flag",
                "doyle:f05p06",
                "doyle:f04p03",
            ],
        ),
        (
            "alphabetum_kaldeorum_hahn_2004_v1",
            vec!["G", "H", "U", "V"],
            vec![
                "kaldeorum:r1c7v1",
                "kaldeorum:r1c8v1",
                "kaldeorum:r3c4v1",
                "kaldeorum:r3c4v1",
            ],
        ),
    ] {
        let p = json!({"profile":profile});
        let actual = execution::execute(
            "symbol.alphabet.encode",
            1,
            &json!({"units":units}),
            &p,
            Budget::default(),
        )
        .unwrap();
        assert_eq!(actual["tokens"], json!(tokens), "{profile}");
        let decoded = execution::execute(
            "symbol.alphabet.decode",
            1,
            &json!({"tokens":tokens}),
            &p,
            Budget::default(),
        )
        .unwrap();
        if profile == "enochian_peterson_2022_v1" {
            assert_eq!(decoded["readings"][1]["alternatives"], json!(["I", "J"]));
            assert_eq!(decoded["readings"][3]["alternatives"], json!(["U", "V"]));
        } else if profile == "alphabetum_kaldeorum_hahn_2004_v1" {
            assert_eq!(decoded["readings"][2]["alternatives"], json!(["U", "V"]));
        } else {
            for (index, unit) in units.iter().enumerate() {
                assert_eq!(decoded["readings"][index]["alternatives"], json!([unit]));
            }
        }
    }
}

#[test]
fn long_glyph_reports_fail_as_output_limits_without_relabelling_them_as_input() {
    use cube_core::{errors::ErrorCode, symbols::alphabet as a};
    let p = serde_json::from_value(json!({"profile":"birds_nrtech_four_features_v1"})).unwrap();
    let error = a::encode(&vec!["H".into(); 65536], &p).unwrap_err();
    assert_eq!(error.code, ErrorCode::ResourceLimit);
    assert_eq!(error.parameter, Some("output"));
    assert_eq!(error.operation, Some("symbol.alphabet.encode"));
    // The bare token array fits; the complete report, including provenance, does not.
    let count = (a::MAX_REPORT - 1) / ("upright:left:flat:raised".len() + 3);
    let error = a::encode(&vec!["H".into(); count], &p).unwrap_err();
    assert_eq!(error.code, ErrorCode::ResourceLimit);
    assert_eq!(error.parameter, Some("output"));
}

#[test]
fn historical_gaps_marks_and_source_variants_never_infer_modern_text() {
    use cube_core::{errors::ErrorCode, symbols::alphabet as a};
    for (profile, units) in [
        (
            "theban_trithemius_1518_v1",
            vec!["J", "U", "W", "omega", "a", " "],
        ),
        (
            "celestial_agrippa_ch30_v1",
            vec!["A", "ך", "ם", "ן", "ף", "ץ", "אָ", " "],
        ),
        (
            "malachim_agrippa_ch30_v1",
            vec!["A", "ך", "ם", "ן", "ף", "ץ", " "],
        ),
        (
            "magi_ambrosius_1539_v1",
            vec!["A", "ך", "ם", "ן", "ף", "ץ", " "],
        ),
        ("enochian_peterson_2022_v1", vec!["K", "W", "a", " "]),
        ("moon_english_1877_v1", vec!["1", "THE", "TH", "é"]),
        (
            "dancing_men_doyle_1905_v1",
            vec!["F", "J", "K", "Q", "U", "W", "X", "Z", " ", "M  ", "AM"],
        ),
        (
            "alphabetum_kaldeorum_hahn_2004_v1",
            vec!["J", "W", "VV", "NULL", "a"],
        ),
    ] {
        let p = serde_json::from_value(json!({"profile":profile})).unwrap();
        for unit in units {
            let error = a::encode(&[unit.into()], &p).unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidInput, "{profile}: {unit}");
        }
    }
    for (profile, token, expected) in [
        ("malachim_agrippa_ch30_v1", "malachim:r3c7", vec!["ס"]),
        (
            "enochian_peterson_2022_v1",
            "enochian:r4c10",
            vec!["I", "J"],
        ),
        ("enochian_peterson_2022_v1", "enochian:r6c2", vec!["U", "V"]),
        ("enochian_peterson_2022_v1", "enochian:r6c5", vec!["Y"]),
        ("dancing_men_doyle_1905_v1", "doyle:f01p02:flag", vec!["M "]),
        (
            "alphabetum_kaldeorum_hahn_2004_v1",
            "kaldeorum:r1c8v3",
            vec!["H"],
        ),
        (
            "alphabetum_kaldeorum_hahn_2004_v1",
            "kaldeorum:r3c4v1",
            vec!["U", "V"],
        ),
    ] {
        let p = serde_json::from_value(json!({"profile":profile})).unwrap();
        let result = a::decode(&[token.into()], &p).unwrap();
        assert_eq!(result.tokens.0, [token]);
        assert_eq!(result.readings[0].alternatives, expected);
    }
    for (profile, token) in [
        ("theban_trithemius_1518_v1", "theban:r4c1"),
        ("celestial_agrippa_ch30_v1", "celestial:r1c10"),
        ("malachim_agrippa_ch30_v1", "malachim:r3c10"),
        ("magi_ambrosius_1539_v1", "magi:r2c11"),
        ("enochian_peterson_2022_v1", "enochian:r2c1"),
        ("moon_english_1877_v1", "moon:r1c8"),
        ("dancing_men_doyle_1905_v1", "doyle:f01p02:flag:flag"),
        ("alphabetum_kaldeorum_hahn_2004_v1", "kaldeorum:r1c8v4"),
    ] {
        let p = serde_json::from_value(json!({"profile":profile})).unwrap();
        assert_eq!(
            a::decode(&[token.into()], &p).unwrap_err().code,
            ErrorCode::InvalidInput
        );
    }
}
