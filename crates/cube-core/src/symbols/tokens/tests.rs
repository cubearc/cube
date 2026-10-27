use super::*;
fn params() -> Parameters {
    Parameters {
        table: Table {
            source_id: "original".into(),
            profile_id: "scalar".into(),
            license_id: "original-fixture".into(),
            entries: vec![
                Entry {
                    token: "△".into(),
                    value: "A".into(),
                    aliases: vec![],
                },
                Entry {
                    token: "○".into(),
                    value: "B".into(),
                    aliases: vec![],
                },
            ],
        },
        boundaries: Boundaries::Concatenated,
        output_separator: "".into(),
    }
}
#[test]
fn scalar_backend_and_explicit_multiscalar_boundaries_preserve_positions() {
    let p = params();
    let r = transform("△○△", &p, false).unwrap();
    assert_eq!(r.text, "ABA");
    assert!(r.bijective);
    assert_eq!(transform(&r.text, &p, true).unwrap().text, "△○△");
    assert_eq!(r.records[1].byte_offset, 3);
    assert_eq!(r.records[1].scalar_offset, 1);
    assert_eq!(transform("", &p, false).unwrap().text, "");
    let mut p = p;
    p.table.entries[0].token = "e\u{301}".into();
    p.table.entries[1].token = "é".into();
    p.boundaries = Boundaries::Separated {
        separator: " | ".into(),
    };
    p.output_separator = " | ".into();
    let r = transform("e\u{301} | é", &p, false).unwrap();
    assert_eq!(r.text, "A | B");
    assert_eq!(r.records[1].byte_offset, 6);
    assert_eq!(r.records[1].scalar_offset, 5);
    assert_eq!(transform(&r.text, &p, true).unwrap().text, "e\u{301} | é");
}
#[test]
fn aliases_ambiguous_inverse_and_prefix_grammars_are_not_guessed() {
    let mut p = params();
    p.table.entries[0].aliases = vec!["triangle".into()];
    let r = transform("triangle", &p, false).unwrap();
    assert!(r.records[0].alias_used);
    assert!(!r.bijective);
    assert_eq!(r.records[0].canonical_token, "△");
    assert!(transform("A", &p, true).is_err());
    p.table.entries[0].aliases = vec!["△".into()];
    assert!(transform("", &p, false).is_err());
    p.table.entries[0].aliases.clear();
    p.table.entries[1].value = "A".into();
    assert!(transform("A", &p, true).is_err());
    assert_eq!(transform("△○", &p, false).unwrap().text, "AA");
    let mut p = params();
    p.table.entries[0].token = "a".into();
    p.table.entries[1].token = "ab".into();
    assert!(transform("", &p, false).is_err());
    p.boundaries = Boundaries::Separated {
        separator: "|".into(),
    };
    p.output_separator = "|".into();
    assert_eq!(transform("a|ab", &p, false).unwrap().text, "A|B");
    for text in ["a|", "|a", "a||ab", "?"] {
        assert!(transform(text, &p, false).is_err());
    }
    p.boundaries = Boundaries::Separated {
        separator: "a".into(),
    };
    assert!(transform("", &p, false).is_err());
}
#[test]
fn definitions_token_counts_output_and_missing_scalars_are_bounded() {
    let p = params();
    assert!(transform("?", &p, false).is_err());
    assert!(transform("△", &p, true).is_err());
    assert!(transform(&"a".repeat(1048577), &p, false).is_err());
    assert!(transform(&"△".repeat(65537), &p, false).is_err());
    let mut p = p;
    p.table.source_id = "x".repeat(1025);
    assert!(transform("", &p, false).is_err());
    p.table.source_id = "original".into();
    p.table.entries[0].value = "A".repeat(256);
    assert!(transform(&"△".repeat(65536), &p, false).is_err());
    p.table.entries[0].value = "A".repeat(257);
    assert!(transform("", &p, false).is_err());
    let mut p = params();
    p.table.entries = vec![p.table.entries[0].clone(); 4097];
    assert!(transform("", &p, false).is_err());
}

#[test]
fn serialized_table_accepts_exactly_64_kib_and_rejects_one_more_byte() {
    let mut p = params();
    p.boundaries = Boundaries::Separated {
        separator: "|".into(),
    };
    p.output_separator = "|".into();
    p.table.entries = (0..128)
        .map(|n| Entry {
            token: format!("t{n}"),
            value: format!("v{n}"),
            aliases: vec![],
        })
        .collect();
    let mut remaining = 65536 - serde_json::to_vec(&p.table).unwrap().len();
    for e in &mut p.table.entries {
        for field in [&mut e.token, &mut e.value] {
            let extra = remaining.min(256 - field.len());
            field.push_str(&"x".repeat(extra));
            remaining -= extra;
        }
    }
    assert_eq!(remaining, 0);
    assert_eq!(serde_json::to_vec(&p.table).unwrap().len(), 65536);
    assert!(transform("", &p, false).is_ok());
    p.table.source_id.push('x');
    assert_eq!(serde_json::to_vec(&p.table).unwrap().len(), 65537);
    assert!(transform("", &p, false).is_err());
}
