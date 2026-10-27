use b::Formula as F;
use cube_core::{errors::ErrorCode, logic::boolean as b};
fn var(s: &str) -> F {
    F::Var { name: s.into() }
}
fn not(f: F) -> F {
    F::Not {
        operand: Box::new(f),
    }
}
fn pair(op: u8, a: F, c: F) -> F {
    let (left, right) = (Box::new(a), Box::new(c));
    match op {
        0 => F::And { left, right },
        1 => F::Or { left, right },
        2 => F::Xor { left, right },
        3 => F::Implies { left, right },
        _ => F::Iff { left, right },
    }
}
fn model(f: F, other: Option<F>) -> b::Model {
    b::Model {
        variables: vec!["A".into(), "B".into()],
        formula: f,
        other,
    }
}
fn table(f: &F) -> u8 {
    match f {
        F::Const { value } => {
            if *value {
                15
            } else {
                0
            }
        }
        F::Var { name } => {
            if name == "A" {
                12
            } else {
                10
            }
        }
        F::Not { operand } => !table(operand) & 15,
        F::And { left, right } => table(left) & table(right),
        F::Or { left, right } => table(left) | table(right),
        F::Xor { left, right } => table(left) ^ table(right),
        F::Implies { left, right } => (!table(left) | table(right)) & 15,
        F::Iff { left, right } => !(table(left) ^ table(right)) & 15,
    }
}
#[test]
fn full_operator_tables_exact_assignments_and_counterexamples() {
    for (op, bits) in [(0, 8u8), (1, 14), (2, 6), (3, 11), (4, 9)] {
        let m = model(pair(op, var("A"), var("B")), Some(var("B")));
        let r = b::analyze(&m, b::Limits::default()).unwrap();
        assert_eq!(r.true_count, bits.count_ones() as usize);
        assert_eq!(r.assignments, 4);
        assert!(r.listing_complete);
        for (i, row) in r.rows.iter().enumerate() {
            assert_eq!(row.value, bits & (1 << i) != 0);
            let bindings = vec![
                b::Binding {
                    name: "B".into(),
                    value: i & 1 != 0,
                },
                b::Binding {
                    name: "A".into(),
                    value: i & 2 != 0,
                },
            ];
            let e = b::evaluate(&m, &bindings, b::Limits::default()).unwrap();
            assert_eq!(e.value, row.value);
            assert_eq!(e.other_value, row.other_value);
        }
        let relation = r.relationship.unwrap();
        assert_eq!(relation.implies, bits & !10 == 0);
        assert_eq!(relation.equivalent, bits == 10);
        if let Some(c) = relation.implication_counterexample {
            let e = b::evaluate(&m, &c, b::Limits::default()).unwrap();
            assert!(e.value);
            assert_eq!(e.other_value, Some(false));
        }
    }
    let demorgan = model(
        not(pair(0, var("A"), var("B"))),
        Some(pair(1, not(var("A")), not(var("B")))),
    );
    assert!(
        b::analyze(&demorgan, b::Limits::default())
            .unwrap()
            .relationship
            .unwrap()
            .equivalent
    );
    for (value, true_count) in [(true, 1), (false, 0)] {
        let m = b::Model {
            variables: vec![],
            formula: F::Const { value },
            other: None,
        };
        let r = b::analyze(&m, b::Limits::default()).unwrap();
        assert_eq!(r.assignments, 1);
        assert_eq!(r.true_count, true_count);
        assert!(r.rows[0].assignment.is_empty());
    }
    let knight = model(
        pair(
            0,
            pair(4, var("A"), pair(0, not(var("A")), not(var("B")))),
            pair(4, var("B"), not(var("A"))),
        ),
        None,
    );
    let r = b::analyze(&knight, b::Limits::default()).unwrap();
    assert_eq!(r.true_count, 1);
    let values = r.first_satisfying.unwrap();
    assert!(!values[0].value);
    assert!(values[1].value);
}
fn generated(seed: &mut u64, depth: u8) -> F {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    let op = (*seed >> 32) % 9;
    if depth == 0 || op < 3 {
        return match op % 3 {
            0 => var("A"),
            1 => var("B"),
            _ => F::Const {
                value: *seed & 1 != 0,
            },
        };
    }
    if op == 3 {
        return not(generated(seed, depth - 1));
    }
    let a = generated(seed, depth - 1);
    let c = generated(seed, depth - 1);
    pair((op - 4) as u8, a, c)
}
#[test]
fn independent_bitset_oracle_for_original_generated_trees() {
    let mut seed = 20142;
    for _ in 0..256 {
        let f = generated(&mut seed, 5);
        let expected = table(&f);
        let r = b::analyze(&model(f, None), b::Limits::default()).unwrap();
        assert_eq!(r.true_count, expected.count_ones() as usize);
        for (i, row) in r.rows.iter().enumerate() {
            assert_eq!(row.value, expected & (1 << i) != 0);
        }
    }
}
#[test]
fn listing_and_exact_work_memory_report_boundaries() {
    let m = model(pair(3, var("A"), var("B")), None);
    let r = b::analyze(
        &m,
        b::Limits {
            max_rows: 1,
            ..b::Limits::default()
        },
    )
    .unwrap();
    assert_eq!(r.true_count, 3);
    assert!(!r.listing_complete);
    assert_eq!(r.rows.len(), 1);
    assert_eq!(r.work, 12);
    let l = b::Limits {
        max_work: r.work,
        max_rows: 1,
        max_memory_bytes: r.memory_bytes_upper_bound,
        max_report_bytes: r.report_bytes_upper_bound,
    };
    assert!(b::analyze(&m, l).is_ok());
    assert!(serde_json::to_vec(&r).unwrap().len() <= r.report_bytes_upper_bound);
    for l in [
        b::Limits {
            max_work: l.max_work - 1,
            ..l
        },
        b::Limits {
            max_memory_bytes: l.max_memory_bytes - 1,
            ..l
        },
        b::Limits {
            max_report_bytes: l.max_report_bytes - 1,
            ..l
        },
        b::Limits {
            max_work: b::MAX_WORK + 1,
            ..l
        },
        b::Limits {
            max_rows: 4097,
            ..l
        },
        b::Limits {
            max_report_bytes: usize::MAX,
            ..l
        },
        b::Limits {
            max_memory_bytes: usize::MAX,
            ..l
        },
    ] {
        assert!(b::analyze(&m, l).is_err());
    }
    assert!(
        b::analyze(
            &m,
            b::Limits {
                max_rows: 0,
                ..b::Limits::default()
            }
        )
        .unwrap()
        .rows
        .is_empty()
    );
    let m = b::Model {
        variables: (0..12).map(|i| format!("v{i}")).collect(),
        formula: F::Const { value: true },
        other: None,
    };
    let r = b::analyze(
        &m,
        b::Limits {
            max_rows: 4096,
            ..b::Limits::default()
        },
    )
    .unwrap();
    assert_eq!(r.rows.len(), 4096);
    assert_eq!(r.true_count, 4096);
    assert!(r.listing_complete);
    assert!(serde_json::to_vec(&r).unwrap().len() <= r.report_bytes_upper_bound);
}
fn full(depth: u8) -> F {
    if depth == 0 {
        F::Const { value: true }
    } else {
        pair(0, full(depth - 1), full(depth - 1))
    }
}
#[test]
fn nodes_depth_variables_schemas_and_binding_errors() {
    let mut m = b::Model {
        variables: vec![],
        formula: not(full(7)),
        other: None,
    };
    assert!(b::analyze(&m, b::Limits::default()).is_ok());
    m.other = Some(F::Const { value: false });
    assert_eq!(
        b::analyze(&m, b::Limits::default()).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let mut f = F::Const { value: true };
    for _ in 0..31 {
        f = not(f);
    }
    m.formula = f;
    m.other = None;
    assert!(b::analyze(&m, b::Limits::default()).is_ok());
    m.formula = not(m.formula);
    assert!(b::analyze(&m, b::Limits::default()).is_err());
    for s in [
        r#"{"variables":[],"formula":{"op":"const","value":true,"extra":0}}"#,
        r#"{"variables":[],"formula":{"op":"const","value":1}}"#,
        r#"{"variables":[],"formula":{"op":"var","name":"missing"}}"#,
        r#"{"variables":[],"variables":[],"formula":{"op":"const","value":true}}"#,
        r#"{"variables":[],"formula":{"op":"and","left":{"op":"const","value":true}}}"#,
        r#"{"variables":["A","A"],"formula":{"op":"var","name":"A"}}"#,
    ] {
        assert!(b::parse(s).is_err(), "{s}");
    }
    assert_eq!(
        b::parse(&" ".repeat(b::MAX_INPUT + 1)).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let m = model(var("A"), None);
    for bindings in [
        vec![],
        vec![
            b::Binding {
                name: "A".into(),
                value: true
            };
            2
        ],
        vec![
            b::Binding {
                name: "A".into(),
                value: false,
            },
            b::Binding {
                name: "C".into(),
                value: true,
            },
        ],
    ] {
        assert!(b::evaluate(&m, &bindings, b::Limits::default()).is_err());
    }
    assert!(b::parse_assignment(r#"[{"name":"A","value":1}]"#).is_err());
    assert!(b::parse_assignment(r#"[{"name":"A","value":false,"value":true}]"#).is_err());
    assert!(
        b::parse_assignment(&format!(
            "[{}]",
            std::iter::repeat_n(r#"{"name":"A","value":true}"#, 13)
                .collect::<Vec<_>>()
                .join(",")
        ))
        .is_err()
    );
    let mut m = m;
    m.variables = (0..13).map(|i| i.to_string()).collect();
    assert!(b::analyze(&m, b::Limits::default()).is_err());
    m.variables = vec!["".into()];
    assert!(b::analyze(&m, b::Limits::default()).is_err());
    m.variables = vec!["a".repeat(65)];
    assert!(b::analyze(&m, b::Limits::default()).is_err());
}
