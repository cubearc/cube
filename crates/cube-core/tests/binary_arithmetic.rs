use cube_core::{
    binary::{self, transform as t},
    errors::ErrorCode,
    operations,
    pipeline::{self, Kind, Limits, Parameter, Plan, Step, Value, recipe::Recipe},
};
use std::collections::BTreeMap;
#[test]
fn rotations_and_arithmetic_match_independent_wide_integer_oracle() {
    for width in [1, 2, 4, 8] {
        let bits = width * 8;
        let mask = (1u128 << bits) - 1;
        for n in [
            0u128,
            1,
            0x55,
            0xaa,
            mask,
            1 << (bits - 1),
            (1 << (bits - 1)) | 1,
        ] {
            let data = &n.to_be_bytes()[16 - width..];
            for count in [
                0,
                1,
                bits as u64 - 1,
                bits as u64,
                bits as u64 + 1,
                u64::MAX,
            ] {
                let k = count % bits as u64;
                let expected = if k == 0 {
                    n
                } else {
                    ((n << k) | (n >> (bits as u64 - k))) & mask
                };
                let left = t::rotate_left(data, count, width).unwrap();
                assert_eq!(left, &expected.to_be_bytes()[16 - width..]);
                assert_eq!(t::rotate_right(&left, count, width).unwrap(), data);
            }
            for rhs in [0u128, 1, mask] {
                let key = &rhs.to_be_bytes()[16 - width..];
                assert_eq!(
                    t::add_wrapping(data, key, width).unwrap(),
                    &((n + rhs) & mask).to_be_bytes()[16 - width..]
                );
                assert_eq!(
                    t::sub_wrapping(data, key, width).unwrap(),
                    &((n + (1u128 << bits) - rhs) & mask).to_be_bytes()[16 - width..]
                );
            }
        }
    }
    assert_eq!(t::rotate_left(&[0x81], 1, 1).unwrap(), [3]);
    assert_eq!(t::rotate_left(&[0x80, 1], 1, 2).unwrap(), [0, 3]);
    assert_eq!(
        t::add_wrapping(&[255, 255, 0, 1], &[0, 1, 0, 1], 2).unwrap(),
        [0, 0, 0, 2]
    );
    assert_eq!(
        t::sub_wrapping(&[0, 0, 0, 1], &[0, 1, 0, 1], 2).unwrap(),
        [255, 255, 0, 0]
    );
    assert_eq!(t::xor_exact(&[0, 255], &[255, 15]).unwrap(), [255, 240]);
    assert_eq!(
        t::xor_exact(&t::xor_exact(&[0, 128], &[255, 0]).unwrap(), &[255, 0]).unwrap(),
        [0, 128]
    );
    assert_eq!(binary::xor(&[0, 0], &[255]).unwrap(), [255, 255]);
    assert!(binary::xor(&[], &[]).is_err());
}
#[test]
fn empty_validation_and_exact_operand_ceilings() {
    for width in [1, 2, 4, 8] {
        assert!(t::rotate_left(&[], u64::MAX, width).unwrap().is_empty());
        assert!(t::rotate_right(&[], 0, width).unwrap().is_empty());
        assert!(t::add_wrapping(&[], &[], width).unwrap().is_empty());
        assert!(t::sub_wrapping(&[], &[], width).unwrap().is_empty());
    }
    assert!(t::xor_exact(&[], &[]).unwrap().is_empty());
    for width in [0, 3, usize::MAX] {
        assert!(t::rotate_left(&[], 0, width).is_err());
        assert!(t::add_wrapping(&[], &[], width).is_err());
        assert!(t::sub_wrapping(&[], &[], width).is_err());
    }
    assert!(t::rotate_right(&[0], 1, 2).is_err());
    assert!(t::add_wrapping(&[0], &[0], 2).is_err());
    for r in [
        t::xor_exact(&[], &[0]),
        t::add_wrapping(&[0], &[], 1),
        t::sub_wrapping(&[], &[0], 1),
    ] {
        assert_eq!(r.unwrap_err().code, ErrorCode::InvalidParameter);
    }
    let data = vec![255; t::MAX_INPUT + 1];
    let exact = &data[..t::MAX_INPUT];
    assert_eq!(t::rotate_left(exact, 1, 8).unwrap(), exact);
    assert_eq!(t::rotate_right(exact, 1, 8).unwrap(), exact);
    assert_eq!(t::xor_exact(exact, exact).unwrap(), vec![0; t::MAX_INPUT]);
    assert_eq!(
        t::sub_wrapping(exact, exact, 8).unwrap(),
        vec![0; t::MAX_INPUT]
    );
    assert_eq!(t::add_wrapping(exact, exact, 8).unwrap()[7], 254);
    for r in [
        t::rotate_left(&data, 0, 1),
        t::rotate_right(&data, 0, 1),
        t::xor_exact(&data, &[]),
        t::xor_exact(&[], &data),
        t::add_wrapping(&data, &[], 1),
        t::add_wrapping(&[], &data, 1),
        t::sub_wrapping(&[], &data, 1),
    ] {
        assert_eq!(r.unwrap_err().code, ErrorCode::ResourceLimit);
    }
}
fn step(id: &str, params: &[(&str, Parameter)]) -> Step {
    let mut s = Step::new(id);
    s.parameters = params
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();
    s
}
#[test]
fn versioned_pipeline_recipe_composition_and_budgets() {
    let steps = vec![
        step(
            "binary.rotate_left",
            &[
                ("count", Parameter::Integer(17)),
                ("unit_bytes", Parameter::Integer(2)),
            ],
        ),
        step(
            "binary.rotate_right",
            &[
                ("count", Parameter::Integer(1)),
                ("unit_bytes", Parameter::Integer(2)),
            ],
        ),
        step(
            "binary.add_wrapping",
            &[
                ("key", Parameter::Bytes { data: vec![0, 1] }),
                ("unit_bytes", Parameter::Integer(2)),
            ],
        ),
        step(
            "binary.sub_wrapping",
            &[
                ("key", Parameter::Bytes { data: vec![0, 1] }),
                ("unit_bytes", Parameter::Integer(2)),
            ],
        ),
        step(
            "binary.xor_exact",
            &[(
                "key",
                Parameter::Bytes {
                    data: vec![255, 255],
                },
            )],
        ),
    ];
    let data = Value::Bytes { data: vec![128, 1] };
    let limits = Limits {
        max_work_bytes: 22,
        ..Limits::default()
    };
    let p = Plan::validate(Kind::Bytes, steps.clone(), limits.clone()).unwrap();
    let r = p.execute(data.clone(), &[], None).unwrap();
    assert_eq!(
        r.output,
        Value::Bytes {
            data: vec![127, 254]
        }
    );
    assert_eq!(r.work_bytes_upper_bound, 22);
    assert!(
        Plan::validate(
            Kind::Bytes,
            steps.clone(),
            Limits {
                max_work_bytes: 21,
                ..limits
            }
        )
        .unwrap()
        .execute(data.clone(), &[], None)
        .is_err()
    );
    let definition = serde_json::json!({"format_version":1,"engine_api":1,"input":{"source":"slot","name":"payload","type":"bytes"},"output":{"type":"bytes"},"steps":steps});
    let recipe = Recipe::parse(&definition.to_string()).unwrap();
    assert_eq!(
        recipe
            .execute(&BTreeMap::from([("payload".into(), data)]), &[], None)
            .unwrap()
            .output,
        r.output
    );
    assert_eq!(pipeline::ENGINE_API, 1);
    for id in [
        "binary.rotate_left",
        "binary.rotate_right",
        "binary.xor_exact",
        "binary.add_wrapping",
        "binary.sub_wrapping",
    ] {
        assert_eq!(
            operations::require(id)
                .unwrap()
                .discovery()
                .pipeline
                .unwrap()
                .version,
            1
        );
    }
    for params in [
        vec![("count", Parameter::Integer(-1))],
        vec![("count", Parameter::Text("1".into()))],
        vec![
            ("count", Parameter::Integer(1)),
            ("unit_bytes", Parameter::Integer(3)),
        ],
    ] {
        assert!(
            Plan::validate(
                Kind::Bytes,
                vec![step("binary.rotate_left", &params)],
                Limits::default()
            )
            .is_err()
        );
    }
    for len in [65536, 65537] {
        let s = step(
            "binary.xor_exact",
            &[(
                "key",
                Parameter::Bytes {
                    data: vec![255; len],
                },
            )],
        );
        let p = Plan::validate(Kind::Bytes, vec![s], Limits::default());
        if len == 65536 {
            assert!(
                p.unwrap()
                    .execute(Value::Bytes { data: vec![0; len] }, &[], None)
                    .is_ok()
            );
        } else {
            assert!(p.is_err());
        }
    }
    let p = Plan::validate(
        Kind::Bytes,
        vec![step(
            "binary.add_wrapping",
            &[
                ("key", Parameter::Bytes { data: vec![0, 1] }),
                ("unit_bytes", Parameter::Integer(2)),
            ],
        )],
        Limits::default(),
    )
    .unwrap();
    assert!(
        p.execute(Value::Bytes { data: vec![0] }, &[], None)
            .is_err()
    );
    assert!(
        Plan::validate(
            Kind::Bytes,
            vec![step(
                "binary.sub_wrapping",
                &[
                    ("key", Parameter::Bytes { data: vec![0] }),
                    ("unit_bytes", Parameter::Integer(2))
                ]
            )],
            Limits::default()
        )
        .is_err()
    );
}
