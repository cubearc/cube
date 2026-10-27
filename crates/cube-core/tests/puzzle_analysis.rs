use cube_core::{
    errors::ErrorCode,
    image::{Channel, ColorModel, Image, Region, spatial as s},
    probability as p,
    text::{
        anagram as a,
        multiset::{self, Profile},
        statistics::{CaseMode, Normalization, TextFilter},
        word_ladder as l,
    },
};
use std::collections::{BTreeMap, BTreeSet};
fn words(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).into()).collect()
}
#[test]
fn anagrams_match_exhaustive_unordered_multisets_and_shared_inventory() {
    let dictionary = words(&["ab", "a", "b", "ba", "aa"]);
    for source in ["", "ab", "aa", "aab", "abab", "bb", "x"] {
        for reuse in [false, true] {
            let o = a::Options {
                max_words: 3,
                reuse,
                ..Default::default()
            };
            let r = a::search(source, &dictionary, Profile::default(), o).unwrap();
            assert!(r.complete);
            let mut expected = BTreeSet::new();
            for i in 0..6 {
                for j in 0..6 {
                    for k in 0..6 {
                        let path: Vec<usize> = [i, j, k].into_iter().filter(|&v| v < 5).collect();
                        if path
                            .windows(2)
                            .any(|w| if reuse { w[0] > w[1] } else { w[0] >= w[1] })
                        {
                            continue;
                        }
                        let mut chars: Vec<char> =
                            path.iter().flat_map(|&v| dictionary[v].chars()).collect();
                        chars.sort_unstable();
                        let mut wanted: Vec<char> = source.chars().collect();
                        wanted.sort_unstable();
                        if chars == wanted {
                            expected.insert(path);
                        }
                    }
                }
            }
            assert_eq!(
                r.solutions.iter().cloned().collect::<BTreeSet<_>>(),
                expected,
                "{source}/{reuse}"
            );
            for path in &r.solutions {
                let text = path
                    .iter()
                    .map(|&i| r.dictionary[i].text.as_str())
                    .collect::<String>();
                assert!(
                    multiset::subtract(source, &text, Profile::default(), Default::default())
                        .unwrap()
                        .equal
                );
            }
        }
    }
}
#[test]
fn anagram_aliases_profiles_incomplete_prefixes_and_boundaries() {
    let p = Profile {
        normalization: Normalization::Nfc,
        case: CaseMode::AsciiLower,
        filter: TextFilter::All,
    };
    let d = words(&["E\u{301}", "É", "A", "a"]);
    let r = a::search("Éa", &d, p, Default::default()).unwrap();
    assert_eq!(r.dictionary[0].aliases, vec![0, 1]);
    assert_eq!(r.dictionary[1].aliases, vec![2, 3]);
    assert_eq!(r.solutions, vec![vec![0, 1]]);
    let d = words(&["ab", "a", "b"]);
    let full = a::search("ab", &d, Profile::default(), Default::default()).unwrap();
    assert_eq!(full.solutions, vec![vec![0], vec![1, 2]]);
    let o = a::Options {
        max_results: 1,
        ..Default::default()
    };
    let short = a::search("ab", &d, Profile::default(), o).unwrap();
    assert!(!short.complete);
    assert_eq!(short.stop_reason, Some("result_limit"));
    assert_eq!(short.solutions, full.solutions[..1]);
    let exact = a::Options {
        max_work: full.work,
        ..Default::default()
    };
    assert!(
        a::search("ab", &d, Profile::default(), exact)
            .unwrap()
            .complete
    );
    let lower = a::Options {
        max_work: full.work - 1,
        ..Default::default()
    };
    assert!(
        !a::search("ab", &d, Profile::default(), lower)
            .unwrap()
            .complete
    );
    for o in [
        a::Options {
            max_memory_bytes: full.memory_bytes_upper_bound - 1,
            ..Default::default()
        },
        a::Options {
            max_report_bytes: full.report_bytes_upper_bound - 1,
            ..Default::default()
        },
    ] {
        assert_eq!(
            a::search("ab", &d, Profile::default(), o).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
    }
    assert!(
        a::search("", &words(&["a"]), Profile::default(), Default::default())
            .unwrap()
            .solutions[0]
            .is_empty()
    );
    assert!(a::search("a", &words(&[""]), Profile::default(), Default::default()).is_err());
    assert!(
        a::search(
            "a",
            &vec!["a".into(); 10001],
            Profile::default(),
            Default::default()
        )
        .is_err()
    );
    assert!(
        a::search(
            "a",
            &["a".repeat(65)],
            Profile::default(),
            Default::default()
        )
        .is_err()
    );
    assert!(
        a::search(
            "a",
            &["a".repeat(64)],
            Profile::default(),
            Default::default()
        )
        .is_ok()
    );
    let profile = Profile {
        filter: TextFilter::AsciiLetters,
        ..Default::default()
    };
    assert!(a::search("a", &words(&["!"]), profile, Default::default()).is_err());
}
#[test]
fn ladders_use_lexicographic_ties_aliases_and_precise_failure_evidence() {
    let d = words(&["dog", "cog", "dot", "cot", "cat", "cot"]);
    let p = Profile::default();
    let r = l::shortest(&d, "cat", "dog", p, Default::default()).unwrap();
    let path = r.path.unwrap();
    let text: Vec<_> = path
        .iter()
        .map(|&i| r.dictionary[i].text.as_str())
        .collect();
    assert_eq!(text, vec!["cat", "cot", "cog", "dog"]);
    assert_eq!(r.edge_count, 5);
    assert_eq!(r.dictionary[2].aliases, vec![3, 5]);
    assert!(
        l::validate(
            &d,
            &words(&["cat", "cot", "cog", "dog"]),
            p,
            Default::default()
        )
        .unwrap()
        .valid
    );
    let r = l::validate(&d, &words(&["cat", "dog"]), p, Default::default()).unwrap();
    let fail = r.failure.unwrap();
    assert_eq!(fail.position, 1);
    assert_eq!(fail.step, Some(0));
    assert_eq!(fail.reason, "not_one_substitution");
    let r = l::validate(&d, &words(&["cat", "hat"]), p, Default::default()).unwrap();
    assert_eq!(r.failure.unwrap().reason, "not_in_dictionary");
    assert_eq!(
        l::shortest(&words(&["cat", "dog"]), "cat", "dog", p, Default::default())
            .unwrap()
            .status,
        "no_path"
    );
    assert_eq!(
        l::shortest(&d, "cat", "cat", p, Default::default())
            .unwrap()
            .path
            .unwrap()
            .len(),
        1
    );
    assert!(l::shortest(&d, "hat", "dog", p, Default::default()).is_err());
    assert!(l::shortest(&words(&["a", "ab"]), "a", "ab", p, Default::default()).is_err());
    let p = Profile {
        normalization: Normalization::Nfc,
        ..Default::default()
    };
    let d = words(&["e\u{301}x", "éx", "é!"]);
    assert_eq!(
        l::shortest(&d, "éx", "é!", p, Default::default())
            .unwrap()
            .path
            .unwrap()
            .len(),
        2
    );
}
#[test]
fn ladder_distances_match_independent_floyd_for_all_small_dictionaries() {
    let universe = ["aaa", "aab", "aba", "abb", "baa", "bab", "bba", "bbb"];
    for mask in 1u16..256 {
        let d: Vec<String> = universe
            .iter()
            .enumerate()
            .filter_map(|(i, w)| (mask & (1 << i) != 0).then_some(w.to_string()))
            .collect();
        let n = d.len();
        let mut dist = vec![vec![99; n]; n];
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    dist[i][j] = 0;
                } else if d[i]
                    .chars()
                    .zip(d[j].chars())
                    .filter(|(a, b)| a != b)
                    .count()
                    == 1
                {
                    dist[i][j] = 1;
                }
            }
        }
        for k in 0..n {
            for i in 0..n {
                for j in 0..n {
                    dist[i][j] = dist[i][j].min(dist[i][k] + dist[k][j]);
                }
            }
        }
        let r = l::shortest(&d, &d[0], &d[n - 1], Profile::default(), Default::default()).unwrap();
        assert_eq!(r.path.map(|p| p.len() - 1).unwrap_or(99), dist[0][n - 1]);
    }
}
#[test]
fn ladder_exact_budgets_and_dense_edge_caps() {
    let d = words(&["cat", "cot", "cog", "dog"]);
    let r = l::shortest(&d, "cat", "dog", Profile::default(), Default::default()).unwrap();
    let o = l::Options {
        max_work: r.work,
        ..Default::default()
    };
    assert!(l::shortest(&d, "cat", "dog", Profile::default(), o).is_ok());
    for o in [
        l::Options {
            max_work: r.work - 1,
            ..Default::default()
        },
        l::Options {
            max_report_bytes: r.report_bytes_upper_bound - 1,
            ..Default::default()
        },
        l::Options {
            max_memory_bytes: 1,
            ..Default::default()
        },
    ] {
        assert_eq!(
            l::shortest(&d, "cat", "dog", Profile::default(), o)
                .unwrap_err()
                .code,
            ErrorCode::ResourceLimit
        );
    }
    let d: Vec<String> = (0..258)
        .map(|i| char::from_u32(0x1000 + i).unwrap().to_string())
        .collect();
    assert_eq!(
        l::shortest(&d, &d[0], &d[1], Profile::default(), Default::default())
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert!(l::validate(&words(&["a"]), &[], Profile::default(), Default::default()).is_err());
}
fn ratio(f: &p::Fraction) -> (u128, u128) {
    (f.numerator, f.denominator)
}
#[test]
fn finite_conditioning_expectation_and_strict_models() {
    let m=p::parse_finite(r#"{"outcomes":[{"id":"00","weight":1,"value":-3},{"id":"01","weight":1,"value":1},{"id":"10","weight":1,"value":1},{"id":"11","weight":1,"value":1}],"event_a":["00","01"],"event_b":["00","10"]}"#).unwrap();
    let r = p::finite_analyze(&m, Default::default()).unwrap();
    assert_eq!(ratio(&r.probability_a), (1, 2));
    assert_eq!(ratio(r.intersection.as_ref().unwrap()), (1, 4));
    assert_eq!(ratio(r.conditional_a_given_b.as_ref().unwrap()), (1, 2));
    let e = r.expected_value.unwrap();
    assert!(!e.negative);
    assert_eq!((e.numerator, e.denominator), (0, 1));
    let mut m = m;
    m.outcomes[0].value = Some(-4);
    assert!(
        p::finite_analyze(&m, Default::default())
            .unwrap()
            .expected_value
            .unwrap()
            .negative
    );
    m.event_b = Some(vec![]);
    assert!(p::finite_analyze(&m, Default::default()).is_err());
    m.event_b = None;
    m.event_a.push("00".into());
    assert!(p::finite_analyze(&m, Default::default()).is_err());
    m.event_a = vec!["unknown".into()];
    assert!(p::finite_analyze(&m, Default::default()).is_err());
    for text in [
        r#"{"outcomes":[],"event_a":[],"event_a":[]}"#,
        r#"{"outcomes":[{"id":"x","weight":true}],"event_a":[]}"#,
        r#"{"outcomes":[{"id":"x","weight":1.5}],"event_a":[]}"#,
        r#"{"outcomes":[{"id":"x","weight":"01"}],"event_a":[]}"#,
    ] {
        assert!(p::parse_finite(text).is_err());
    }
}
fn dice(faces: &[&[i32]]) -> p::DiceModel {
    p::DiceModel {
        dice: faces
            .iter()
            .map(|d| {
                d.iter()
                    .map(|&v| p::Face {
                        value: v,
                        weight: 1,
                    })
                    .collect()
            })
            .collect(),
    }
}
fn cartesian(d: &p::DiceModel, i: usize, sum: i64, mass: u128, out: &mut BTreeMap<i64, u128>) {
    if i == d.dice.len() {
        *out.entry(sum).or_default() += mass;
        return;
    }
    for f in &d.dice[i] {
        cartesian(
            d,
            i + 1,
            sum + i64::from(f.value),
            mass * u128::from(f.weight),
            out,
        );
    }
}
#[test]
fn dice_convolution_matches_independent_cartesian_and_win_tie_loss() {
    let d6 = [1, 2, 3, 4, 5, 6];
    let r = p::dice_sum(&dice(&[&d6, &d6]), Default::default()).unwrap();
    assert_eq!(r.total_mass, 36);
    assert_eq!(
        r.distribution.iter().map(|p| p.mass).collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5, 6, 5, 4, 3, 2, 1]
    );
    for d in [
        dice(&[]),
        dice(&[&[-1, 0, 0, 3], &[-2, 2]]),
        p::DiceModel {
            dice: vec![vec![
                p::Face {
                    value: -1,
                    weight: 3,
                },
                p::Face {
                    value: 2,
                    weight: 2,
                },
                p::Face {
                    value: 99,
                    weight: 0,
                },
            ]],
        },
    ] {
        let r = p::dice_sum(&d, Default::default()).unwrap();
        let mut expected = BTreeMap::new();
        cartesian(&d, 0, 0, 1, &mut expected);
        expected.retain(|_, v| *v > 0);
        assert_eq!(
            r.distribution
                .into_iter()
                .map(|p| (p.value, p.mass))
                .collect::<BTreeMap<_, _>>(),
            expected
        );
    }
    let a = dice(&[&[2, 2, 4, 4, 9, 9]]);
    let b = dice(&[&[1, 1, 6, 6, 8, 8]]);
    let c = dice(&[&[3, 3, 5, 5, 7, 7]]);
    for (left, right) in [(a.clone(), b.clone()), (b.clone(), c.clone()), (c, a)] {
        let r = p::dice_compare(&p::DicePair { left, right }, Default::default()).unwrap();
        assert!(r.win.numerator * 2 > r.win.denominator);
        assert_eq!(r.win_mass + r.tie_mass + r.loss_mass, r.total_mass);
    }
    let r = p::dice_compare(
        &p::DicePair {
            left: b.clone(),
            right: b,
        },
        Default::default(),
    )
    .unwrap();
    assert_eq!(ratio(&r.tie), (1, 3));
}
#[test]
fn probability_overflow_support_exact_work_and_allocations() {
    let d = dice(&[&[1, 2, 3], &[2, 3]]);
    let r = p::dice_sum(&d, Default::default()).unwrap();
    let l = p::Limits {
        max_work: r.work,
        max_report_bytes: r.report_bytes_upper_bound,
        max_memory_bytes: r.memory_bytes_upper_bound,
    };
    assert!(p::dice_sum(&d, l).is_ok());
    for l in [
        p::Limits {
            max_work: r.work - 1,
            ..Default::default()
        },
        p::Limits {
            max_report_bytes: r.report_bytes_upper_bound - 1,
            ..Default::default()
        },
        p::Limits {
            max_memory_bytes: r.memory_bytes_upper_bound - 1,
            ..Default::default()
        },
    ] {
        assert_eq!(
            p::dice_sum(&d, l).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
    }
    let d = p::DiceModel {
        dice: vec![
            vec![p::Face {
                value: 1,
                weight: u64::MAX
            }];
            3
        ],
    };
    assert_eq!(
        p::dice_sum(&d, Default::default()).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let d = dice(&[&[0]]);
    let mut d = d;
    d.dice[0][0].weight = 0;
    assert!(p::dice_sum(&d, Default::default()).is_err());
    let d = dice(&[
        &(0..64).collect::<Vec<_>>(),
        &(0..64).map(|v| v * 64).collect::<Vec<_>>(),
        &[0, 4096],
    ]);
    assert_eq!(
        p::dice_sum(&d, Default::default()).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}
#[test]
fn spatial_moments_match_direct_sample_oracle_and_constant_semantics() {
    let data = vec![2, 17, 3, 19, 11, 5, 23, 7, 13, 29, 31, 37];
    let image = Image::from_samples(4, 3, ColorModel::Gray, data.clone(), vec![], vec![]).unwrap();
    let region = Region {
        x: 0,
        y: 0,
        width: 4,
        height: 3,
    };
    let lags = [
        s::Lag { dx: 0, dy: 0 },
        s::Lag { dx: 1, dy: -1 },
        s::Lag { dx: -2, dy: 1 },
        s::Lag { dx: 1, dy: -1 },
    ];
    let r = image
        .autocorrelation(region, Channel::Gray, &lags, Default::default())
        .unwrap();
    for m in &r.measurements {
        let mut pairs = Vec::new();
        for y in 0..3i32 {
            for x in 0..4i32 {
                let (tx, ty) = (x + m.lag.dx, y + m.lag.dy);
                if (0..4).contains(&tx) && (0..3).contains(&ty) {
                    pairs.push((
                        u64::from(data[(y * 4 + x) as usize]),
                        u64::from(data[(ty * 4 + tx) as usize]),
                    ));
                }
            }
        }
        assert_eq!(m.count, pairs.len() as u64);
        assert_eq!(m.sum_x, pairs.iter().map(|p| p.0).sum::<u64>());
        assert_eq!(m.sum_y, pairs.iter().map(|p| p.1).sum::<u64>());
        assert_eq!(m.sum_xy, pairs.iter().map(|p| p.0 * p.1).sum::<u64>());
        assert_eq!(
            m.squared_difference,
            pairs.iter().map(|p| p.0.abs_diff(p.1).pow(2)).sum::<u64>()
        );
    }
    assert_eq!(r.measurements[0].correlation, Some(1.0));
    let stripe =
        Image::from_samples(4, 1, ColorModel::Gray, vec![0, 255, 0, 255], vec![], vec![]).unwrap();
    let region = Region {
        x: 0,
        y: 0,
        width: 4,
        height: 1,
    };
    let r = stripe
        .autocorrelation(
            region,
            Channel::Gray,
            &[s::Lag { dx: 1, dy: 0 }],
            Default::default(),
        )
        .unwrap();
    assert_eq!(r.measurements[0].correlation, Some(-1.0));
    let constant = Image::from_samples(1, 1, ColorModel::Gray, vec![9], vec![], vec![]).unwrap();
    assert_eq!(
        constant
            .autocorrelation(
                Region {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1
                },
                Channel::Gray,
                &[s::Lag { dx: 0, dy: 0 }],
                Default::default()
            )
            .unwrap()
            .measurements[0]
            .correlation,
        None
    );
}
#[test]
fn spatial_preflight_limits_channels_and_palette_provenance() {
    let i = Image::from_samples(
        4,
        1,
        ColorModel::Rgba,
        vec![1, 2, 3, 0, 4, 5, 6, 255, 7, 8, 9, 0, 1, 2, 3, 255],
        vec![],
        vec![],
    )
    .unwrap();
    let region = Region {
        x: 0,
        y: 0,
        width: 4,
        height: 1,
    };
    let lags = [s::Lag { dx: 1, dy: 0 }];
    let r = i
        .autocorrelation(region, Channel::Alpha, &lags, Default::default())
        .unwrap();
    assert_eq!(r.measurements[0].sum_x, 255);
    let limits = s::Limits {
        max_work: r.work,
        max_memory_bytes: r.memory_bytes_upper_bound,
        max_report_bytes: r.report_bytes_upper_bound,
    };
    assert!(
        i.autocorrelation(region, Channel::Alpha, &lags, limits)
            .is_ok()
    );
    for limits in [
        s::Limits {
            max_work: r.work - 1,
            ..Default::default()
        },
        s::Limits {
            max_report_bytes: r.report_bytes_upper_bound - 1,
            ..Default::default()
        },
        s::Limits {
            max_memory_bytes: r.memory_bytes_upper_bound - 1,
            ..Default::default()
        },
    ] {
        assert_eq!(
            i.autocorrelation(region, Channel::Alpha, &lags, limits)
                .unwrap_err()
                .code,
            ErrorCode::ResourceLimit
        );
    }
    assert!(
        i.autocorrelation(region, Channel::Gray, &lags, Default::default())
            .is_err()
    );
    assert!(
        i.autocorrelation(
            region,
            Channel::Red,
            &[s::Lag {
                dx: i32::MIN,
                dy: 0
            }],
            Default::default()
        )
        .is_err()
    );
    assert!(
        i.autocorrelation(
            region,
            Channel::Red,
            &vec![s::Lag { dx: 0, dy: 0 }; 257],
            Default::default()
        )
        .is_err()
    );
    let indexed = Image::from_samples(
        2,
        1,
        ColorModel::Indexed,
        vec![0, 1],
        vec![[0, 1, 2], [255, 0, 0]],
        vec![],
    )
    .unwrap();
    let region = Region {
        x: 0,
        y: 0,
        width: 2,
        height: 1,
    };
    assert!(
        indexed
            .autocorrelation(region, Channel::Red, &lags, Default::default())
            .is_err()
    );
    assert!(
        indexed
            .expand_palette()
            .unwrap()
            .autocorrelation(region, Channel::Red, &lags, Default::default())
            .is_ok()
    );
    assert!(s::parse_lags(r#"[{"dx":1,"dx":2,"dy":0}]"#).is_err());
}

#[test]
fn lexical_and_finite_cardinality_ceilings_and_actual_json_bounds() {
    let d = vec!["a".to_string(); 10000];
    let r = a::search("a", &d, Profile::default(), Default::default()).unwrap();
    assert_eq!(r.dictionary[0].aliases.len(), 10000);
    assert!(serde_json::to_vec(&r).unwrap().len() <= r.report_bytes_upper_bound);
    let d: Vec<String> = (0..4096)
        .map(|i| {
            format!(
                "{}{}",
                char::from_u32(0x1000 + i).unwrap(),
                char::from_u32(0x5000 + i).unwrap()
            )
        })
        .collect();
    let r = l::shortest(&d, &d[0], &d[4095], Profile::default(), Default::default()).unwrap();
    assert_eq!(r.status, "no_path");
    assert!(serde_json::to_vec(&r).unwrap().len() <= r.report_bytes_upper_bound);
    let mut d = d;
    d.push("XY".into());
    assert!(l::shortest(&d, &d[0], &d[4095], Profile::default(), Default::default()).is_err());
    let model = p::FiniteModel {
        outcomes: (0..4096)
            .map(|i| p::Outcome {
                id: i.to_string(),
                weight: 1,
                value: None,
            })
            .collect(),
        event_a: vec![],
        event_b: None,
    };
    let r = p::finite_analyze(&model, Default::default()).unwrap();
    assert_eq!(r.total_mass, 4096);
    assert!(serde_json::to_vec(&r).unwrap().len() <= r.report_bytes_upper_bound);
    let mut model = model;
    model.outcomes.push(p::Outcome {
        id: "extra".into(),
        weight: 1,
        value: None,
    });
    assert!(p::finite_analyze(&model, Default::default()).is_err());
    let d = dice(&[
        &(0..64).collect::<Vec<_>>(),
        &(0..64).map(|v| v * 64).collect::<Vec<_>>(),
    ]);
    let r = p::dice_sum(&d, Default::default()).unwrap();
    assert_eq!(r.distribution.len(), 4096);
    assert!(serde_json::to_vec(&r).unwrap().len() <= r.report_bytes_upper_bound);
    let image = Image::from_samples(
        1024,
        1024,
        ColorModel::Gray,
        vec![255; 1024 * 1024],
        vec![],
        vec![],
    )
    .unwrap();
    let region = Region {
        x: 0,
        y: 0,
        width: 1024,
        height: 1024,
    };
    let r = image
        .autocorrelation(
            region,
            Channel::Gray,
            &[s::Lag { dx: 0, dy: 0 }; 16],
            Default::default(),
        )
        .unwrap();
    assert_eq!(r.work, s::MAX_WORK);
    assert!(serde_json::to_vec(&r).unwrap().len() <= r.report_bytes_upper_bound);
    assert!(
        image
            .autocorrelation(
                region,
                Channel::Gray,
                &[s::Lag { dx: 0, dy: 0 }; 17],
                Default::default()
            )
            .is_err()
    );
}
