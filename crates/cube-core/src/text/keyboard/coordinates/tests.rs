use super::*;
fn p() -> Options {
    Options {
        layout: Geometry::preset("qwerty").unwrap(),
        index_base: 0,
    }
}
#[test]
fn original_layouts_and_ragged_coordinates_remain_compatible() {
    for preset in ["qwerty", "dvorak", "qwertz"] {
        let g = Geometry::preset(preset).unwrap();
        assert_eq!(g.entries.len(), 94);
        let old = Layout::preset(preset).unwrap();
        assert_eq!(
            super::super::remap("qQ!", &old, &old, "error").unwrap(),
            "qQ!"
        );
        for index_base in [0, 1] {
            let p = Options {
                layout: g.clone(),
                index_base,
            };
            let text: String = p.layout.entries.iter().map(|e| e.symbol).collect();
            let r = encode(&text, &p).unwrap();
            assert_eq!(decode(&r.coordinates, &p).unwrap().text, text);
            assert_eq!(
                r.coordinates[0],
                Coordinate(u16::from(index_base), u16::from(index_base), "base".into())
            );
        }
    }
    let mut p = p();
    p.index_base = 1;
    let r = encode("qA?", &p).unwrap();
    assert_eq!(
        r.coordinates,
        [
            Coordinate(2, 1, "base".into()),
            Coordinate(3, 1, "shift".into()),
            Coordinate(4, 10, "shift".into())
        ]
    );
    assert_eq!(r.positions, [[0, 0], [1, 1], [2, 2]]);
    assert!(encode("😀", &p).is_err());
    assert!(decode(&[Coordinate(0, 0, "base".into())], &p).is_err());
}
#[test]
fn displacement_edges_and_bijections_are_explicit() {
    let layout = Geometry::preset("qwerty").unwrap();
    let p = Shift {
        layout: layout.clone(),
        dx: 1,
        dy: 0,
        edge: EdgePolicy::Reject,
    };
    let r = shift("qQ", &p, false).unwrap();
    assert_eq!(r.text, "wW");
    assert!(!r.bijective);
    assert!(shift("/", &p, false).is_err());
    assert!(shift("wW", &p, true).is_err());
    for dx in [i64::MIN, -1, 0, 1, i64::MAX] {
        let p = Shift {
            layout: layout.clone(),
            dx,
            dy: 0,
            edge: EdgePolicy::Wrap,
        };
        let text = "qQ!?/zZ1";
        let r = shift(text, &p, false).unwrap();
        assert!(r.bijective);
        assert_eq!(shift(&r.text, &p, true).unwrap().text, text);
    }
    let p = Shift {
        layout: layout.clone(),
        dx: 0,
        dy: 1,
        edge: EdgePolicy::Wrap,
    };
    let r = shift("q", &p, false).unwrap();
    assert!(!r.bijective);
    assert!(shift(&r.text, &p, true).is_err());
    let p = Shift {
        layout,
        dx: i64::MIN,
        dy: i64::MAX,
        edge: EdgePolicy::Reject,
    };
    assert!(shift("a", &p, false).is_err());
}
#[test]
fn malformed_geometry_duplicates_sparse_wrap_and_limits_reject() {
    let mut g = Geometry::preset("qwerty").unwrap();
    g.entries[0].row = 700;
    assert!(
        encode(
            "",
            &Options {
                layout: g,
                index_base: 0
            }
        )
        .is_err()
    );
    let mut g = Geometry::preset("qwerty").unwrap();
    g.entries[1].column = 0;
    assert!(
        encode(
            "",
            &Options {
                layout: g,
                index_base: 0
            }
        )
        .is_err()
    );
    let mut g = Geometry::preset("qwerty").unwrap();
    g.entries[47].column = 1;
    assert!(
        encode(
            "",
            &Options {
                layout: g,
                index_base: 0
            }
        )
        .is_err()
    );
    let mut g = Geometry::preset("qwerty").unwrap();
    for e in &mut g.entries {
        e.row += 1;
    }
    assert!(
        shift(
            "",
            &Shift {
                layout: g,
                dx: 0,
                dy: 0,
                edge: EdgePolicy::Wrap
            },
            false
        )
        .is_err()
    );
    let p = p();
    assert_eq!(
        encode(&"q".repeat(65536), &p).unwrap().coordinates.len(),
        65536
    );
    assert!(encode(&"q".repeat(65537), &p).is_err());
    assert!(encode(&"q".repeat(1048577), &p).is_err());
    assert!(decode(&vec![Coordinate(0, 0, "base".into()); 65537], &p).is_err());
    let bad = Options { index_base: 2, ..p };
    assert!(encode("", &bad).is_err());
}

#[test]
fn serialized_geometry_accepts_exactly_64_kib_and_rejects_one_more_byte() {
    let mut layout = Geometry {
        geometry: "original-boundary-fixture".into(),
        entries: (0..256)
            .map(|n| Key {
                key: format!("k{n}"),
                layer: format!("l{n}"),
                symbol: char::from_u32(0x1000 + n).unwrap(),
                row: n as u16,
                column: 0,
            })
            .collect(),
    };
    let mut remaining = 65536 - serde_json::to_vec(&layout).unwrap().len();
    for e in &mut layout.entries {
        for field in [&mut e.key, &mut e.layer] {
            let extra = remaining.min(128 - field.len());
            field.push_str(&"x".repeat(extra));
            remaining -= extra;
        }
    }
    assert_eq!(remaining, 0);
    assert_eq!(serde_json::to_vec(&layout).unwrap().len(), 65536);
    let mut p = Options {
        layout,
        index_base: 0,
    };
    assert!(encode("", &p).is_ok());
    p.layout.geometry.push('x');
    assert_eq!(serde_json::to_vec(&p.layout).unwrap().len(), 65537);
    assert!(encode("", &p).is_err());
}
