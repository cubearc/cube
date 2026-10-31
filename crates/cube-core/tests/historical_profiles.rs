use cube_core::operations::execution::{Budget, execute};
use serde_json::{Value, json};
fn call(operation: &str, inputs: Value, parameters: Value) -> Value {
    execute(operation, 1, &inputs, &parameters, Budget::default()).unwrap()
}
#[test]
fn named_paired_profiles_execute_their_exact_conventions() {
    assert_eq!(
        call(
            "cipher.malespin",
            json!({"text":"bueno Amigo áéíóú"}),
            json!({})
        ),
        json!({"text":"tuani Epofi áéíóú"})
    );
    assert_eq!(
        call(
            "cipher.vatsyayana",
            json!({"text":"QOEU"}),
            json!({"pairs":[["Q","W"],["O","I"],["E","S"],["U","H"]]})
        ),
        json!({"text":"WISH"})
    );
}
#[test]
fn nomenclator_preserves_segmentation_nulls_and_canceled_group_provenance() {
    let p = json!({"definition":{"entries":[{"alternatives":["OR"],"codes":["10"]},{"alternatives":["D"],"codes":["05"]}],"null_codes":["99"],"cancel_codes":["27"]}});
    assert_eq!(
        call(
            "cipher.nomenclator.encode",
            json!({"units":["OR","D"]}),
            p.clone()
        ),
        json!({"tokens":["10","05"]})
    );
    assert_eq!(
        call(
            "cipher.nomenclator.decode",
            json!({"tokens":["10","05","27","99"]}),
            p
        ),
        json!({"report":{"units":[{"token_index":0,"alternatives":["OR"]}],"null_indices":[3],"canceled_indices":[1],"cancel_indices":[2]}})
    );
}

#[test]
fn malformed_tables_pairs_cancellation_and_unknown_groups_fail() {
    use cube_core::ciphers::mono::historical as h;
    use cube_core::errors::ErrorCode;
    for pairs in [
        json!([]),
        json!([["A", "A"]]),
        json!([["A", "B"], ["B", "C"]]),
    ] {
        let p: h::Pairs = h::parse(&json!({"pairs":pairs}).to_string()).unwrap();
        assert_eq!(
            h::vatsyayana("A", &p).unwrap_err().operation,
            Some("cipher.vatsyayana")
        );
    }
    let p: h::Pairs = h::parse(r#"{"pairs":[["é","猫"]],"unmapped":"preserve"}"#).unwrap();
    assert_eq!(h::vatsyayana("é\0e\u{301}", &p).unwrap(), "猫\0e\u{301}");
    assert_eq!(
        h::vatsyayana(&h::vatsyayana("é猫", &p).unwrap(), &p).unwrap(),
        "é猫"
    );
    let definition = json!({"entries":[{"alternatives":["OR"],"codes":["10"]}],"null_codes":["99"],"cancel_codes":["27"]});
    let book = h::Book::new(h::parse(&definition.to_string()).unwrap()).unwrap();
    for tokens in [vec!["27"], vec!["unknown"], vec!["10", "27", "27"]] {
        assert_eq!(
            book.decode(&tokens.into_iter().map(str::to_owned).collect::<Vec<_>>())
                .unwrap_err()
                .code,
            ErrorCode::InvalidInput
        );
    }
    // Cancellation erases one whole previous group; even an unknown group can
    // be canceled without an invented reading, and a null is still a group.
    assert!(
        book.decode(&["unknown".into(), "27".into()])
            .unwrap()
            .units
            .is_empty()
    );
    assert_eq!(
        book.decode(&["10".into(), "99".into(), "27".into()])
            .unwrap()
            .units[0]
            .alternatives,
        vec!["OR"]
    );
    for bad in [
        json!({"entries":[]}),
        json!({"entries":[{"alternatives":["A"],"codes":["x","x"]}]}),
        json!({"entries":[{"alternatives":["A"],"codes":["x"]}],"null_codes":["x"]}),
        json!({"entries":[{"alternatives":[""],"codes":["x"]}]}),
    ] {
        assert!(h::Book::new(h::parse(&bad.to_string()).unwrap()).is_err());
    }
    assert!(h::parse::<h::Definition>(r#"{"entries":[],"entries":[]}"#).is_err());
    assert!(h::parse::<h::Pairs>(r#"{"pairs":[["\ud800","A"]]}"#).is_err());
    let letters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ áéíóú\0";
    assert_eq!(
        h::malespin(&h::malespin(letters).unwrap()).unwrap(),
        letters
    );
    assert!(
        h::execute(
            "cipher.malespin",
            &json!({"text":"A","extra":true}),
            &json!({})
        )
        .is_err()
    );
    assert!(
        h::execute(
            "cipher.great_cipher.decode",
            &json!({"tokens":[]}),
            &json!({"extra":true})
        )
        .is_err()
    );
    let fixed = h::great_cipher().unwrap();
    assert_eq!(
        fixed.decode(&["164".into(), "215".into()]).unwrap().units[0].alternatives,
        vec!["Ie", "Je"]
    );
    assert_eq!(
        fixed.decode(&["215".into()]).unwrap().units[0].alternatives,
        vec!["Iu", "Ju"]
    );
    assert!(fixed.decode(&["6".into()]).is_err());
    assert!(fixed.decode(&["0587".into()]).is_err());
    assert_eq!(
        fixed.decode(&["502".into()]).unwrap().units[0].alternatives,
        vec!["Bien"]
    );
    assert_eq!(
        fixed.decode(&["587".into()]).unwrap().units[0].alternatives,
        vec!["U", "V"]
    );
}

#[test]
fn codebook_limits_are_enforced_on_direct_rust_callers_before_output_growth() {
    use cube_core::ciphers::mono::historical as h;
    use cube_core::errors::ErrorCode;
    let book = h::Book::new(h::Definition {
        entries: vec![h::Entry {
            alternatives: vec!["A".repeat(256)],
            codes: vec!["x".into()],
        }],
        null_codes: vec![],
        cancel_codes: vec![],
    })
    .unwrap();
    assert_eq!(book.encode(&["A".repeat(256)]).unwrap(), vec!["x"]);
    assert_eq!(
        book.encode(&vec!["A".repeat(256); h::MAX_ITEMS + 1])
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        book.decode(&vec!["x".into(); h::MAX_ITEMS])
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert!(
        h::Book::new(h::Definition {
            entries: vec![h::Entry {
                alternatives: vec!["A".repeat(257)],
                codes: vec!["x".into()]
            }],
            null_codes: vec![],
            cancel_codes: vec![]
        })
        .is_err()
    );
    let tiny = h::Book::new(h::Definition {
        entries: vec![h::Entry {
            alternatives: vec!["A".into()],
            codes: vec!["x".into()],
        }],
        null_codes: vec![],
        cancel_codes: vec![],
    })
    .unwrap();
    assert_eq!(
        tiny.encode(&vec!["A".into(); h::MAX_ITEMS]).unwrap().len(),
        h::MAX_ITEMS
    );
    assert_eq!(tiny.encode(&[]).unwrap(), Vec::<String>::new());
    assert!(tiny.decode(&[]).unwrap().units.is_empty());
    let many = h::Definition {
        entries: (0..h::MAX_ENTRIES)
            .map(|i| h::Entry {
                alternatives: vec![format!("A{i}")],
                codes: vec![i.to_string()],
            })
            .collect(),
        null_codes: vec![],
        cancel_codes: vec![],
    };
    assert!(h::Book::new(many.clone()).is_ok());
    let mut over = many;
    over.null_codes.push("too-many".into());
    assert!(h::Book::new(over).is_err());
}

#[test]
fn syllabary_rejects_non_permutations_and_incomplete_or_invalid_cells() {
    use cube_core::ciphers::mono::historical as h;
    let mut definition =
        json!({"cells":vec!["A";100],"row_labels":"0123456789","column_labels":"0123456789"});
    for (field, value) in [
        ("row_labels", json!("0012345678")),
        ("row_labels", json!("012345678x")),
        ("column_labels", json!("012345678")),
        ("cells", json!(vec!["A"; 99])),
        ("cells", json!(vec!["A"; 101])),
    ] {
        let original = definition[field].clone();
        definition[field] = value;
        assert!(
            h::parse::<h::Syllabary>(&definition.to_string())
                .unwrap()
                .book()
                .is_err()
        );
        definition[field] = original;
    }
    definition["cells"][0] = json!("");
    assert!(
        h::parse::<h::Syllabary>(&definition.to_string())
            .unwrap()
            .book()
            .is_err()
    );
}

#[test]
fn codebook_serialized_table_and_output_accept_exactly_one_mib_and_reject_one_more_byte() {
    use cube_core::ciphers::mono::historical as h;
    use cube_core::errors::ErrorCode;
    let mut definition = h::Definition {
        entries: (0..h::MAX_ENTRIES)
            .map(|i| h::Entry {
                alternatives: vec!["X".into()],
                codes: vec![i.to_string()],
            })
            .collect(),
        null_codes: vec![],
        cancel_codes: vec![],
    };
    let mut remaining = h::MAX_BYTES - serde_json::to_vec(&definition).unwrap().len();
    for entry in &mut definition.entries {
        let count = remaining.min(255);
        entry.alternatives[0].push_str(&"A".repeat(count));
        remaining -= count;
    }
    assert_eq!(remaining, 0);
    assert_eq!(serde_json::to_vec(&definition).unwrap().len(), h::MAX_BYTES);
    assert!(h::Book::new(definition.clone()).is_ok());
    definition.entries.last_mut().unwrap().alternatives[0].push('A');
    assert_eq!(
        serde_json::to_vec(&definition).unwrap().len(),
        h::MAX_BYTES + 1
    );
    assert_eq!(
        h::Book::new(definition).unwrap_err().code,
        ErrorCode::ResourceLimit
    );

    let mut definition = h::Definition {
        entries: vec![
            h::Entry {
                alternatives: vec!["A".into()],
                codes: vec!["x".repeat(256)],
            },
            h::Entry {
                alternatives: vec!["B".into()],
                codes: vec!["y".repeat(140)],
            },
        ],
        null_codes: vec![],
        cancel_codes: vec![],
    };
    let mut units = vec!["A".to_owned(); 4048];
    units.push("B".into());
    let output = h::Book::new(definition.clone())
        .unwrap()
        .encode(&units)
        .unwrap();
    assert_eq!(serde_json::to_vec(&output).unwrap().len(), h::MAX_BYTES);
    definition.entries[1].codes[0].push('y');
    assert_eq!(
        h::Book::new(definition)
            .unwrap()
            .encode(&units)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
