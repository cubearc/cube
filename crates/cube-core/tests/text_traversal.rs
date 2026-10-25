use cube_core::text::{
    self,
    traversal::{self, Direction, Options},
};
fn oracle(
    input: &str,
    steps: &[u64],
    start: usize,
    reverse: bool,
    count: usize,
    remove: bool,
) -> (String, Vec<usize>) {
    let source: Vec<_> = input.chars().collect();
    let mut active: Vec<_> = (0..source.len()).collect();
    let mut anchor = start;
    let mut selected = vec![];
    for i in 0..count {
        let distance = ((steps[i % steps.len()] - 1) % active.len() as u64) as usize;
        let rank = if reverse {
            (anchor + active.len() - distance) % active.len()
        } else {
            (anchor + distance) % active.len()
        };
        selected.push(active[rank]);
        if remove {
            active.remove(rank);
            if active.is_empty() {
                break;
            }
            anchor = if reverse {
                (rank + active.len() - 1) % active.len()
            } else {
                rank % active.len()
            };
        } else {
            anchor = if reverse {
                (rank + active.len() - 1) % active.len()
            } else {
                (rank + 1) % active.len()
            };
        }
    }
    (selected.iter().map(|i| source[*i]).collect(), selected)
}
#[test]
fn retained_and_removed_vectors_match_an_independent_list_oracle() {
    assert_eq!(
        traversal::keyed("ABCDE", &[2], 5, &Options::default())
            .unwrap()
            .extracted,
        "BDACE"
    );
    assert_eq!(
        traversal::josephus("ABCDE", &[2], None, &Options::default())
            .unwrap()
            .extracted,
        "BDAEC"
    );
    for n in 1..=20 {
        let input: String = (0..n).map(|i| char::from(b'A' + i)).collect();
        for reverse in [false, true] {
            for start in 0..n as usize {
                for steps in [&[1][..], &[2, 3], &[u64::MAX, 1]] {
                    for remove in [false, true] {
                        let options = Options {
                            start: Some(start as u64),
                            direction: if reverse {
                                Direction::Reverse
                            } else {
                                Direction::Forward
                            },
                            ..Options::default()
                        };
                        let expected = oracle(&input, steps, start, reverse, n as usize, remove);
                        let result = if remove {
                            traversal::josephus(&input, steps, None, &options)
                        } else {
                            traversal::keyed(&input, steps, n as usize, &options)
                        }
                        .unwrap();
                        assert_eq!(result.extracted, expected.0);
                        assert_eq!(
                            result
                                .records
                                .iter()
                                .map(|r| r.scalar_index)
                                .collect::<Vec<_>>(),
                            expected.1
                        );
                    }
                }
            }
        }
    }
}
#[test]
fn traversal_origins_unicode_duplicates_and_fixed_stride_reuse() {
    let options = Options {
        index_base: 1,
        start: Some(2),
        ..Options::default()
    };
    assert_eq!(
        traversal::keyed("ABCDE", &[1], 3, &options)
            .unwrap()
            .extracted,
        "BCD"
    );
    let report = traversal::keyed("é🧊a\u{301}", &[2], 4, &Options::default()).unwrap();
    assert_eq!(report.extracted, "🧊\u{301}🧊\u{301}");
    assert_eq!(report.records[1].byte_offset, 7);
    let options = Options {
        wrap: false,
        ..Options::default()
    };
    let report = traversal::keyed("ABCDEFGH", &[2], 4, &options).unwrap();
    assert_eq!(
        report.extracted,
        text::extract("ABCDEFGH", None, Some(1), Some(2), None).unwrap()
    );
    assert!(traversal::keyed("ABCDE", &[2], 3, &options).is_err());
    assert!(traversal::josephus("ABCDE", &[2], None, &options).is_err());
    assert_eq!(
        traversal::josephus("ABCDE", &[2], Some(2), &Options::default())
            .unwrap()
            .remaining,
        3
    );
    assert!(
        traversal::keyed("", &[1], 0, &Options::default())
            .unwrap()
            .records
            .is_empty()
    );
    assert!(traversal::keyed("", &[1], 1, &Options::default()).is_err());
}
#[test]
fn traversal_budgets_and_invalid_requests_are_explicit() {
    for steps in [&[][..], &[0]] {
        assert!(traversal::keyed("A", steps, 1, &Options::default()).is_err());
    }
    for text in ["", "0", "-1", "1,", "1,,2", "1.0", "18446744073709551616"] {
        assert!(traversal::parse_steps(text).is_err());
    }
    assert_eq!(
        traversal::parse_steps("1,18446744073709551615").unwrap(),
        vec![1, u64::MAX]
    );
    let base = traversal::keyed("ABCDE", &[2], 5, &Options::default()).unwrap();
    let work = base.work_units;
    let memory = base.temporary_bytes;
    let bytes = serde_json::to_vec(&base).unwrap().len();
    assert!(
        traversal::keyed(
            "ABCDE",
            &[2],
            5,
            &Options {
                max_work: work,
                max_memory: memory,
                max_output: bytes,
                ..Options::default()
            }
        )
        .is_ok()
    );
    assert!(
        traversal::keyed(
            "ABCDE",
            &[2],
            5,
            &Options {
                max_work: work - 1,
                ..Options::default()
            }
        )
        .is_err()
    );
    assert!(
        traversal::keyed(
            "ABCDE",
            &[2],
            5,
            &Options {
                max_memory: memory - 1,
                ..Options::default()
            }
        )
        .is_err()
    );
    assert!(
        traversal::keyed(
            "ABCDE",
            &[2],
            5,
            &Options {
                max_output: bytes - 1,
                ..Options::default()
            }
        )
        .is_err()
    );
    assert!(traversal::keyed("A", &[1], 65536, &Options::default()).is_ok());
    assert!(traversal::keyed("A", &[1], 65537, &Options::default()).is_err());
    for options in [
        Options {
            index_base: 2,
            ..Options::default()
        },
        Options {
            index_base: 1,
            start: Some(0),
            ..Options::default()
        },
        Options {
            start: Some(u64::MAX),
            ..Options::default()
        },
        Options {
            max_work: traversal::MAX_WORK + 1,
            ..Options::default()
        },
    ] {
        assert!(traversal::keyed("A", &[1], 1, &options).is_err());
    }
    assert!(
        traversal::keyed(
            &"A".repeat(text::MAX_INPUT + 1),
            &[1],
            0,
            &Options::default()
        )
        .is_err()
    );
    assert!(traversal::josephus(&"A".repeat(700000), &[1], Some(1), &Options::default()).is_err());
}

#[test]
fn traversal_discovery_requires_count_only_for_retained_rings() {
    use cube_core::operations::{self, ParameterValue};
    let steps = [2];
    assert!(
        operations::require("text.skip.keyed")
            .unwrap()
            .validate(&[("steps", ParameterValue::UnsignedList(&steps))])
            .is_err()
    );
    assert!(
        operations::require("text.skip.keyed")
            .unwrap()
            .validate(&[
                ("steps", ParameterValue::UnsignedList(&steps)),
                ("count", ParameterValue::Unsigned(5))
            ])
            .is_ok()
    );
    assert!(
        operations::require("text.josephus.extract")
            .unwrap()
            .validate(&[("steps", ParameterValue::UnsignedList(&steps))])
            .is_ok()
    );
    let options = Options {
        direction: Direction::Reverse,
        ..Options::default()
    };
    assert_eq!(
        traversal::josephus("ABCDE", &[2], None, &options)
            .unwrap()
            .extracted,
        "ECABD"
    );
    assert_eq!(
        traversal::josephus("ABCDE", &[2, 3], None, &Options::default())
            .unwrap()
            .extracted,
        "BECDA"
    );
}
