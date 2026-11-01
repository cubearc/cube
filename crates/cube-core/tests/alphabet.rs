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
            assert_eq!(decoded["readings"][index]["alternatives"], json!([unit]));
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
fn source_listed_geometric_aliases_preserve_the_received_orientation() {
    let aliases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/symbols/alphabet/aliases.json")).unwrap();
    assert_eq!(aliases.len(), 147); // 121 additional Betamaze orientations +26 bird codes.
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
