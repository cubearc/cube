use super::*;
use serde::de::DeserializeOwned;
use serde_json::json;
fn p<T: DeserializeOwned>(value: serde_json::Value) -> T {
    serde_json::from_value(value).unwrap()
}
fn key(s: &str) -> ColumnKey {
    ColumnKey::Keyword { text: s.into() }
}
#[test]
fn columnar_ragged_unicode_ties_and_independent_passes() {
    for width in 1..=12 {
        for n in 0..=50 {
            let source: String = "a😀β !\0e\u{301}".chars().cycle().take(n).collect();
            let k = ColumnKey::Permutation {
                order: (0..width).rev().collect(),
            };
            let params = Columnar { key: k };
            let encoded = columnar(&source, &params, false).unwrap();
            assert_eq!(encoded.len(), source.len());
            assert_eq!(columnar(&encoded, &params, true).unwrap(), source);
            let double = DoubleColumnar {
                first: key("BALLOON"),
                second: key("KEY"),
            };
            let encoded = double_columnar(&source, &double, false).unwrap();
            assert_eq!(double_columnar(&encoded, &double, true).unwrap(), source);
            for pattern in [CellPattern::Aca, CellPattern::Continuous] {
                for initial_cell in [1, 2] {
                    let a = Amsco {
                        key: ColumnKey::Permutation {
                            order: (0..width).rev().collect(),
                        },
                        initial_cell,
                        pattern,
                    };
                    let e = amsco(&source, &a, false).unwrap();
                    assert_eq!(amsco(&e, &a, true).unwrap(), source);
                }
            }
        }
    }
    assert_eq!(key("BALLOON").order("test").unwrap(), [1, 0, 2, 3, 6, 4, 5]);
    let m = Myszkowski {
        keyword: "AAAA".into(),
    };
    assert_eq!(myszkowski("a😀β !", &m, false).unwrap(), "a😀β !");
    let m = Myszkowski {
        keyword: "CAB".into(),
    };
    let c = Columnar { key: key("CAB") };
    assert_eq!(
        myszkowski("ABCDEFGHIJ", &m, false).unwrap(),
        columnar("ABCDEFGHIJ", &c, false).unwrap()
    );
}
#[test]
fn rails_offsets_readout_and_scalar_limits() {
    for rails in [1, 2, 3, 20, 4096] {
        for initial_rail in [0, rails - 1] {
            for direction in [Direction::Up, Direction::Down] {
                let p = Rails {
                    rails,
                    initial_rail,
                    direction,
                };
                for text in ["", "a", "ab😀é e\u{301}!\0"] {
                    let encoded = rail_fence(text, &p, false).unwrap();
                    assert_eq!(rail_fence(&encoded, &p, true).unwrap(), text);
                }
            }
        }
    }
    for parameters in [
        json!({"rails":0,"initial_rail":0,"direction":"up"}),
        json!({"rails":2,"initial_rail":2,"direction":"down"}),
        json!({"rails":4097,"initial_rail":0,"direction":"down"}),
    ] {
        assert!(rail_fence("", &p(parameters), false).is_err());
    }
    assert!(
        redefence(
            "",
            &p(json!({"rails":2,"initial_rail":0,"direction":"up","order":[0,0]})),
            false
        )
        .is_err()
    );
    assert!(scytale("", &Scytale { columns: 0 }, false).is_err());
    assert!(scytale("", &Scytale { columns: 4097 }, false).is_err());
    let source = "😀".repeat(MAX_SYMBOLS);
    assert_eq!(
        scytale(&source, &Scytale { columns: 4096 }, false).unwrap(),
        source
    );
    assert!(scytale(&(source + "x"), &Scytale { columns: 1 }, false).is_err());
    assert!(scytale(&"x".repeat(MAX_INPUT + 1), &Scytale { columns: 1 }, false).is_err());
}
#[test]
fn bijections_and_minimum_signed_skip() {
    for start in [i64::MIN, -1, 0, i64::MAX] {
        for step in [i64::MIN, -2, -1, 1, 2, i64::MAX] {
            let p = Skip { start, step };
            let e = skip("ABCDE", &p, false).unwrap();
            assert_eq!(skip(&e, &p, true).unwrap(), "ABCDE");
            assert_eq!(skip("", &p, false).unwrap(), "");
            assert_eq!(skip("😀", &p, true).unwrap(), "😀");
        }
    }
    for step in [0, 2, 4] {
        assert!(skip("ABCD", &Skip { start: 0, step }, false).is_err());
    }
    for order in [vec![], vec![0, 0], vec![1, 2], vec![0, 1, 2]] {
        assert!(sequence("AB", &Sequence { order }, false).is_err());
    }
    assert_eq!(sequence("", &Sequence { order: vec![] }, true).unwrap(), "");
    assert!(
        ColumnKey::Permutation { order: vec![] }
            .order("test")
            .is_err()
    );
    assert!(key("").order("test").is_err());
    assert!(key(&"é".repeat(4097)).order("test").is_err());
    assert!(
        amsco(
            "",
            &Amsco {
                key: key("KEY"),
                initial_cell: 3,
                pattern: CellPattern::Aca
            },
            false
        )
        .is_err()
    );
}
#[test]
fn complete_padding_requires_evidence_and_handles_collisions() {
    let mut p = CompleteColumnar {
        key: key("CAB"),
        padding: Some('X'),
        original_length: Some(4),
    };
    let r = complete_columnar("AXXX", &p, false).unwrap();
    assert_eq!(r.padding_count, 2);
    assert_eq!(complete_columnar(&r.text, &p, true).unwrap().text, "AXXX");
    p.original_length = None;
    assert_eq!(complete_columnar(&r.text, &p, true).unwrap().text, "AXXXXX");
    p.padding = None;
    assert!(complete_columnar("A", &p, false).is_err());
    assert!(complete_columnar("A", &p, true).is_err());
    p.original_length = Some(0);
    assert!(complete_columnar(&r.text, &p, true).is_err());
    p.original_length = Some(7);
    assert!(complete_columnar(&r.text, &p, true).is_err());
    p.original_length = Some(4);
    assert!(complete_columnar(&r.text, &p, true).is_err());
    p.padding = Some('Q');
    assert!(complete_columnar(&r.text, &p, true).is_err());
    assert!(complete_columnar("AAAAA", &p, false).is_err());
    p.original_length = None;
    p.padding = Some('😀');
    assert!(complete_columnar(&"x".repeat(65536), &p, false).is_err());
}
#[test]
fn every_route_corner_shape_and_direction_is_a_bijection() {
    use routes::*;
    for (rows, columns) in [(1, 1), (1, 7), (7, 1), (2, 3), (3, 2), (4, 4)] {
        let source: String = (0..rows * columns)
            .map(|i| char::from_u32(0x400 + i as u32).unwrap())
            .collect();
        for corner in [
            Corner::TopLeft,
            Corner::TopRight,
            Corner::BottomRight,
            Corner::BottomLeft,
        ] {
            for kind in [
                RouteKind::Rows,
                RouteKind::Columns,
                RouteKind::Clockwise,
                RouteKind::Counterclockwise,
            ] {
                let p = Routes {
                    rows,
                    columns,
                    fill: Route { kind, corner },
                    read: Route {
                        kind: RouteKind::Counterclockwise,
                        corner: Corner::BottomRight,
                    },
                };
                let e = route(&source, &p, false).unwrap();
                assert_eq!(route(&e, &p, true).unwrap(), source);
            }
        }
    }
    let mut p = Routes {
        rows: 2,
        columns: 3,
        fill: Route {
            kind: RouteKind::Rows,
            corner: Corner::TopLeft,
        },
        read: Route {
            kind: RouteKind::Columns,
            corner: Corner::TopLeft,
        },
    };
    assert!(spiral("ABCDEF", &p, false).is_err());
    p.rows = usize::MAX;
    assert!(route("A", &p, false).is_err());
    p.rows = 0;
    assert!(route("", &p, false).is_err());
    assert!(caesar_box("ABC", &CaesarBox { side: 2 }, false).is_err());
}
#[test]
fn grilles_validate_coverage_and_preserve_source_provenance() {
    use grille::*;
    for orientations in [vec![0, 1, 2, 3], vec![3, 1, 0, 2]] {
        let p = TurningGrille {
            side: 4,
            holes: vec![0, 7, 9, 11],
            orientations,
        };
        let text = "A😀β !\x001234567890xy😀β !\0ABCDEF12345";
        let text: String = text.chars().cycle().take(32).collect();
        let e = turning_grille(&text, &p, false).unwrap();
        assert_eq!(turning_grille(&e, &p, true).unwrap(), text);
        assert!(turning_grille("x", &p, false).is_err());
        assert_eq!(turning_grille("", &p, false).unwrap(), "");
    }
    for value in [
        json!({"side":3,"holes":[0],"orientations":[0,1,2,3]}),
        json!({"side":2,"holes":[4],"orientations":[0,1,2,3]}),
        json!({"side":4,"holes":[0,3,12,15],"orientations":[0,1,2,3]}),
        json!({"side":2,"holes":[0],"orientations":[0,0,2,3]}),
    ] {
        assert!(turning_grille("", &p(value), false).is_err());
    }
    let mask = Cardan {
        rows: 2,
        columns: 2,
        holes: vec![0, 3],
    };
    assert_eq!(cardan_extract("a😀β!", &mask).unwrap(), "a!");
    let r = cardan_overlay(
        "a😀β!",
        &Overlay {
            mask,
            replacements: "XY".into(),
        },
    )
    .unwrap();
    assert_eq!(r.text, "X😀βY");
    assert_eq!(r.original_selected, "a!");
    assert_eq!(r.positions, [0, 3]);
    for mask in [
        Cardan {
            rows: 0,
            columns: 2,
            holes: vec![],
        },
        Cardan {
            rows: 2,
            columns: 2,
            holes: vec![0, 0],
        },
        Cardan {
            rows: 2,
            columns: 2,
            holes: vec![4],
        },
    ] {
        assert!(cardan_extract("ABCD", &mask).is_err());
    }
    assert!(
        cardan_overlay(
            "ABCD",
            &Overlay {
                mask: Cardan {
                    rows: 2,
                    columns: 2,
                    holes: vec![0]
                },
                replacements: "XY".into()
            }
        )
        .is_err()
    );
}

#[test]
fn canonical_decimal_integer_transport_is_exact_for_scalar_lists_and_optionals() {
    use crate::ciphers::{fractionation, poly::parse_parameters};
    let p: Skip = parse_parameters(r#"{"start":"0","step":"-9223372036854775808"}"#).unwrap();
    assert_eq!(skip("ABCDE", &p, false).unwrap(), "ACEBD");
    for step in [
        "-0",
        "00",
        "+1",
        " 1",
        "9223372036854775808",
        "-9223372036854775809",
    ] {
        assert!(
            parse_parameters::<Skip>(&serde_json::json!({"start":0,"step":step}).to_string())
                .is_err()
        );
    }
    assert!(parse_parameters::<Skip>(r#"{"start":0,"step":1.5}"#).is_err());
    let s: Sequence = parse_parameters(r#"{"order":["1","0"]}"#).unwrap();
    assert_eq!(sequence("AB", &s, false).unwrap(), "BA");
    let r: Rails =
        parse_parameters(r#"{"rails":"1","initial_rail":"0","direction":"up"}"#).unwrap();
    assert_eq!(rail_fence("😀", &r, false).unwrap(), "😀");
    let c: CompleteColumnar =
        parse_parameters(r#"{"key":{"kind":"permutation","order":["0"]},"original_length":"1"}"#)
            .unwrap();
    assert_eq!(complete_columnar("A", &c, true).unwrap().text, "A");
    assert_eq!(
        fractionation::parse_nihilist_tokens(r#"["110",22]"#).unwrap(),
        [110, 22]
    );
    assert!(fractionation::parse_nihilist_tokens(r#"["01"]"#).is_err());
    let m: fractionation::morse::Morbit =
        parse_parameters(r#"{"digits":"123456789","padding_count":"0"}"#).unwrap();
    assert_eq!(
        fractionation::morse::morbit("1", &m, true)
            .unwrap()
            .text
            .as_deref(),
        Some("I")
    );
}
