use cube_core::{
    ciphers,
    errors::ErrorCode,
    operations, pipeline,
    text::{
        self,
        positions::{IndexBase, Selector},
        transform::{self, Config, Override, Transformation},
    },
};
fn config(selector: Selector, index_base: IndexBase) -> Config {
    Config {
        selector,
        index_base,
        selected: Transformation::Atbash,
        otherwise: Transformation::Caesar { shift: 3 },
        overrides: vec![],
    }
}
#[test]
fn fibonacci_known_answer_exclusions_and_determinism() {
    let input = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let config = config(
        Selector::Fibonacci {
            exclude: vec![13, 21, 13],
        },
        IndexBase::One,
    );
    let result = transform::masked(input, &config).unwrap();
    assert_eq!(result.selected_positions, [1, 2, 3, 5, 8]);
    assert_eq!(result.deterministic_text, "ZYXGVIJSLMNOPQRSTUVWXYZABC");
    assert_eq!(result.final_text, result.deterministic_text);
    assert_eq!(result.final_text.chars().nth(12), Some('P'));
    assert_eq!(result.final_text.chars().nth(20), Some('X'));
    assert_eq!(result.original_input, input);
    assert!(result.overrides.is_empty());
    assert_eq!(result, transform::masked(input, &config).unwrap());
}
#[test]
fn fibonacci_short_inputs_duplicates_and_index_bases() {
    let fibonacci = Selector::Fibonacci { exclude: vec![] };
    for (length, zero, one) in [
        (0, vec![], vec![]),
        (1, vec![0], vec![1]),
        (2, vec![0, 1], vec![1, 2]),
        (3, vec![0, 1, 2], vec![1, 2, 3]),
        (5, vec![0, 1, 2, 3], vec![1, 2, 3, 5]),
        (8, vec![0, 1, 2, 3, 5], vec![1, 2, 3, 5, 8]),
    ] {
        assert_eq!(fibonacci.positions(length, IndexBase::Zero).unwrap(), zero);
        assert_eq!(fibonacci.positions(length, IndexBase::One).unwrap(), one);
    }
    // Exclude numeric value 3, not the third Fibonacci term (value 1).
    let selector = Selector::Fibonacci {
        exclude: vec![0, 3, 21],
    };
    assert_eq!(
        selector.positions(9, IndexBase::Zero).unwrap(),
        [1, 2, 5, 8]
    );
    assert_eq!(selector.positions(9, IndexBase::One).unwrap(), [1, 2, 5, 8]);
    let cfg = config(fibonacci, IndexBase::One);
    assert_eq!(transform::masked("", &cfg).unwrap().final_text, "");
}
#[test]
fn explicit_and_every_nth_are_sets_in_original_scalar_order() {
    let indices = Selector::Indices {
        indices: vec![5, 1, 1, 2],
    };
    assert_eq!(indices.positions(6, IndexBase::Zero).unwrap(), [1, 2, 5]);
    assert_eq!(indices.positions(6, IndexBase::One).unwrap(), [1, 2, 5]);
    assert_eq!(
        transform::masked("AbCdEf", &config(indices, IndexBase::Zero))
            .unwrap()
            .final_text,
        "DyXgHu"
    );
    assert_eq!(
        text::extract("ABCDEF", Some(&[5, 1, 1, 2]), None, None, None).unwrap(),
        "FBBC"
    );
    for (base, expected) in [
        (IndexBase::Zero, vec![1, 3, 5]),
        (IndexBase::One, vec![2, 4, 6]),
    ] {
        let cfg = config(Selector::EveryNth { n: 2 }, base);
        assert_eq!(cfg.selector.positions(6, base).unwrap(), expected);
        assert_eq!(
            transform::masked("ABCDEF", &cfg).unwrap().final_text,
            "DYFWHU"
        );
    }
    assert!(
        Selector::EveryNth { n: usize::MAX }
            .positions(6, IndexBase::One)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        transform::masked(
            "ABC",
            &config(Selector::Indices { indices: vec![] }, IndexBase::Zero)
        )
        .unwrap()
        .final_text,
        "DEF"
    );
}
#[test]
fn unicode_whitespace_punctuation_and_all_cipher_modes_match_existing_apis() {
    let input = "AaZz é世界🧊e\u{301}\0\r\n!?";
    for transformation in [
        Transformation::Atbash,
        Transformation::Caesar { shift: -1 },
        Transformation::Caesar { shift: i32::MIN },
        Transformation::Caesar { shift: i32::MAX },
        Transformation::Rot13,
        Transformation::Identity,
    ] {
        let expected = match transformation {
            Transformation::Atbash => ciphers::atbash(input).unwrap(),
            Transformation::Caesar { shift } => ciphers::caesar_encode(input, shift),
            Transformation::Rot13 => ciphers::rot13(input),
            Transformation::Identity => input.into(),
        };
        for selector in [
            Selector::EveryNth { n: 1 },
            Selector::Indices { indices: vec![] },
        ] {
            let cfg = Config {
                selected: transformation,
                otherwise: transformation,
                ..config(selector, IndexBase::Zero)
            };
            assert_eq!(transform::masked(input, &cfg).unwrap().final_text, expected);
        }
    }
    let input = "A !éB e\u{301}Z";
    let result =
        transform::masked(input, &config(Selector::EveryNth { n: 2 }, IndexBase::One)).unwrap();
    assert_eq!(result.selected_positions, [2, 4, 6, 8]);
    assert_eq!(result.final_text, "D !éE h\u{301}C");
    assert_eq!(result.final_text.chars().count(), input.chars().count());
    assert_eq!(
        text::extract(input, Some(&[3, 6, 7]), None, None, None).unwrap(),
        "ée\u{301}"
    );
}
#[test]
fn overrides_preserve_deterministic_output_and_record_each_explicit_edit() {
    for base in [IndexBase::Zero, IndexBase::One] {
        let position = u8::from(base) as usize + 1;
        let mut cfg = config(Selector::EveryNth { n: 1 }, base);
        cfg.overrides = vec![
            Override::new(position, "雪").unwrap(),
            Override::new(position, "🧊").unwrap(),
        ];
        let result = transform::masked("ABC", &cfg).unwrap();
        assert_eq!(result.deterministic_text, "ZYX");
        assert_eq!(result.final_text, "Z🧊X");
        assert_eq!(result.overrides[0].previous, 'Y');
        assert_eq!(result.overrides[1].previous, '雪');
        assert_eq!(result.overrides[1].replacement, '🧊');
        assert_eq!(result.config, cfg);
    }
    assert_eq!(Override::parse("0==").unwrap().replacement, '=');
    assert_eq!(Override::new(0, "\u{301}").unwrap().replacement, '\u{301}');
    for value in ["", "0", "-1=x", "0=", "0=ab", "0=e\u{301}"] {
        assert!(Override::parse(value).is_err());
    }
    assert!(Override::new(0, "").is_err());
    let mut cfg = config(Selector::Fibonacci { exclude: vec![] }, IndexBase::One);
    for position in [0, 4, usize::MAX] {
        cfg.overrides = vec![Override {
            position,
            replacement: 'x',
        }];
        assert_eq!(
            transform::masked("ABC", &cfg).unwrap_err().parameter,
            Some("overrides")
        );
    }
    cfg.overrides = vec![Override {
        position: 1,
        replacement: 'x',
    }];
    assert!(transform::masked("", &cfg).is_err());
}
#[test]
fn reject_invalid_selectors_transforms_and_conflicting_options() {
    assert!(IndexBase::try_from(2).is_err());
    assert!(Selector::EveryNth { n: 0 }.validate().is_err());
    assert!(Selector::Fibonacci { exclude: vec![4] }.validate().is_err());
    for (name, indices, nth, exclude) in [
        ("guess", None, None, None),
        ("indices", None, None, None),
        ("indices", Some(vec![]), Some(1), None),
        ("every_nth", None, None, None),
        ("every_nth", None, Some(1), Some(vec![])),
        ("fibonacci", Some(vec![]), None, None),
    ] {
        assert!(Selector::from_options(name, indices, nth, exclude).is_err());
    }
    for (name, indices, nth, exclude) in [
        ("indices", Some(vec![]), None, None),
        ("every_nth", None, Some(2), None),
        ("fibonacci", None, None, Some(vec![13])),
    ] {
        assert!(Selector::from_options(name, indices, nth, exclude).is_ok());
    }
    for name in ["atbash", "rot13", "identity"] {
        assert!(Transformation::parse(name, None).is_ok());
        assert!(Transformation::parse(name, Some(3)).is_err());
    }
    assert!(Transformation::parse("caesar", None).is_err());
    assert!(Transformation::parse("caesar", Some(i32::MIN)).is_ok());
    assert!(Transformation::parse("guess", None).is_err());
    for (base, index) in [
        (IndexBase::Zero, 3),
        (IndexBase::One, 0),
        (IndexBase::One, 4),
        (IndexBase::Zero, usize::MAX),
    ] {
        let error = transform::masked(
            "ABC",
            &config(
                Selector::Indices {
                    indices: vec![index],
                },
                base,
            ),
        )
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidParameter);
        assert_eq!(error.operation, Some("text.transform.masked"));
    }
}
#[test]
fn exact_resource_limits_and_overflow_do_not_truncate() {
    let max = text::MAX_INDICES;
    let mut cfg = config(Selector::EveryNth { n: 1 }, IndexBase::Zero);
    assert_eq!(
        transform::masked(&"A".repeat(max), &cfg)
            .unwrap()
            .selected_positions
            .len(),
        max
    );
    assert_eq!(
        transform::masked(&"A".repeat(max + 1), &cfg)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    cfg.selector = Selector::Indices {
        indices: vec![0; max],
    };
    assert_eq!(transform::masked("A", &cfg).unwrap().final_text, "Z");
    cfg.selector = Selector::Indices {
        indices: vec![0; max + 1],
    };
    assert_eq!(
        transform::masked("A", &cfg).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    cfg.selector = Selector::Fibonacci {
        exclude: vec![1; max],
    };
    assert!(transform::masked("A", &cfg).is_ok());
    cfg.selector = Selector::Fibonacci {
        exclude: vec![1; max + 1],
    };
    assert_eq!(
        transform::masked("A", &cfg).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    cfg.selector = Selector::Fibonacci { exclude: vec![] };
    cfg.overrides = vec![
        Override {
            position: 0,
            replacement: '🧊'
        };
        max
    ];
    assert_eq!(transform::masked("A", &cfg).unwrap().overrides.len(), max);
    cfg.overrides.push(Override {
        position: 0,
        replacement: 'x',
    });
    assert_eq!(
        transform::masked("A", &cfg).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    cfg.overrides.clear();
    assert!(transform::masked(&"A".repeat(text::MAX_INPUT), &cfg).is_ok());
    assert_eq!(
        transform::masked(&"A".repeat(text::MAX_INPUT + 1), &cfg)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert!(cfg.selector.positions(usize::MAX, IndexBase::One).is_err());
    assert!(
        Selector::Fibonacci {
            exclude: vec![usize::MAX]
        }
        .validate()
        .is_err()
    );
    // Last representable Fibonacci values exercise checked-add termination.
    let last = if usize::BITS == 64 {
        12_200_160_415_121_876_738u64
    } else {
        2_971_215_073u64
    };
    assert!(
        Selector::Fibonacci {
            exclude: vec![last as usize]
        }
        .validate()
        .is_ok()
    );
}
#[test]
fn metadata_exposes_direct_contract_and_versioned_pipeline_support() {
    let d = operations::require("text.transform.masked").unwrap();
    assert_eq!((d.input_type, d.output_type), ("text", "text"));
    assert!(d.deterministic);
    assert_eq!(d.discovery().pipeline.unwrap().version, 1);
    assert!(
        pipeline::Plan::validate(
            pipeline::Kind::Text,
            vec![pipeline::Step::new(d.id)],
            pipeline::Limits::default()
        )
        .is_err()
    );
    d.validate(&[
        ("selector", operations::ParameterValue::Text("fibonacci")),
        ("selected", operations::ParameterValue::Text("atbash")),
        (
            "exclude",
            operations::ParameterValue::UnsignedList(&[13, 21]),
        ),
    ])
    .unwrap();
}
