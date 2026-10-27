use cube_core::{
    errors::ErrorCode,
    text::{self, multiset as m, statistics as s},
};
fn scalars(v: &[m::ScalarCount]) -> String {
    v.iter()
        .flat_map(|c| std::iter::repeat_n(c.scalar, c.count as usize))
        .collect()
}
#[test]
fn canonical_inventories_match_independent_sorted_scalar_oracle() {
    for (a, b) in [
        ("listen", "silent"),
        ("aab", "ab"),
        ("ab", "aab"),
        ("ab", "aa"),
        ("", ""),
        ("a", ""),
        ("", "a"),
        ("ée\u{301}\0雪🧊", "eé\u{301}\0🧊雪"),
    ] {
        let r = m::compare(a, b, m::Profile::default(), m::Limits::default()).unwrap();
        let mut av: Vec<_> = a.chars().collect();
        av.sort_unstable();
        let mut bv: Vec<_> = b.chars().collect();
        bv.sort_unstable();
        assert_eq!(scalars(&r.source), av.into_iter().collect::<String>());
        assert_eq!(scalars(&r.requested), bv.into_iter().collect::<String>());
        assert_eq!(r.equal, r.source == r.requested);
        assert_eq!(r.contains, r.deficits.is_empty());
        assert!(r.source.windows(2).all(|w| w[0].scalar < w[1].scalar));
        for _ in 0..2 {
            let next = m::subtract(a, b, m::Profile::default(), m::Limits::default()).unwrap();
            assert_eq!(
                serde_json::to_value(&r).unwrap(),
                serde_json::to_value(&next).unwrap()
            );
        }
        assert!(serde_json::to_vec(&r).unwrap().len() <= r.report_bytes_upper_bound);
    }
    assert_eq!(
        scalars(
            &m::subtract("aab", "ab", m::Profile::default(), m::Limits::default())
                .unwrap()
                .residual
        ),
        "a"
    );
    assert_eq!(
        scalars(
            &m::subtract("ab", "aab", m::Profile::default(), m::Limits::default())
                .unwrap()
                .deficits
        ),
        "a"
    );
}
#[test]
fn preparation_is_shared_with_unchanged_statistics() {
    for normalization in [
        s::Normalization::None,
        s::Normalization::Nfc,
        s::Normalization::Nfd,
        s::Normalization::Nfkc,
        s::Normalization::Nfkd,
    ] {
        for case in [s::CaseMode::Preserve, s::CaseMode::AsciiLower] {
            for filter in [s::TextFilter::All, s::TextFilter::AsciiLetters] {
                let input = "É e\u{301} Aa! ①\0";
                let profile = m::Profile {
                    normalization,
                    case,
                    filter,
                };
                let r = m::compare(input, "", profile, m::Limits::default()).unwrap();
                let stats = s::analyze(
                    input,
                    s::Options {
                        ngram_size: 1,
                        normalization,
                        case,
                        filter,
                        max_results: 10000,
                    },
                )
                .unwrap();
                assert!(stats.complete);
                assert_eq!(
                    r.source.iter().map(|r| r.count as usize).sum::<usize>(),
                    stats.scalar_count
                );
                for c in r.source {
                    assert_eq!(
                        stats
                            .frequencies
                            .iter()
                            .find(|f| f.ngram == c.scalar.to_string())
                            .unwrap()
                            .count,
                        c.count
                    );
                }
            }
        }
    }
    let r = m::compare(
        "É",
        "E\u{301}",
        m::Profile {
            normalization: s::Normalization::Nfc,
            ..m::Profile::default()
        },
        m::Limits::default(),
    )
    .unwrap();
    assert!(r.equal);
    let r = s::analyze(
        "ABAB",
        s::Options {
            ngram_size: 2,
            ..s::Options::default()
        },
    )
    .unwrap();
    assert_eq!(r.scalar_count, 4);
    assert_eq!(r.frequencies[0].ngram, "AB");
    assert_eq!(r.frequencies[0].count, 2);
    assert_eq!(r.index_of_coincidence, Some(1.0 / 3.0));
    assert_eq!(r.shannon_entropy_bits, Some(1.0));
}
#[test]
fn raw_distinct_work_memory_and_report_boundaries_fail_explicitly() {
    let p = m::Profile::default();
    let l = m::Limits::default();
    let r = m::compare("aab", "ab", p, l).unwrap();
    let exact = m::Limits {
        max_work: r.work,
        max_memory_bytes: r.memory_bytes_upper_bound,
        max_report_bytes: r.report_bytes_upper_bound,
    };
    assert!(m::compare("aab", "ab", p, exact).is_ok());
    for limits in [
        m::Limits {
            max_work: exact.max_work - 1,
            ..exact
        },
        m::Limits {
            max_memory_bytes: exact.max_memory_bytes - 1,
            ..exact
        },
        m::Limits {
            max_report_bytes: exact.max_report_bytes - 1,
            ..exact
        },
        m::Limits {
            max_work: m::MAX_WORK + 1,
            ..l
        },
        m::Limits {
            max_memory_bytes: usize::MAX,
            ..l
        },
        m::Limits {
            max_report_bytes: usize::MAX,
            ..l
        },
    ] {
        assert!(m::compare("aab", "ab", p, limits).is_err());
    }
    let max = "a".repeat(text::MAX_INPUT);
    assert!(m::compare(&max, &max, p, l).unwrap().equal);
    assert_eq!(
        m::compare(&(max + "a"), "", p, l).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let distinct: String = (0..).filter_map(char::from_u32).take(10000).collect();
    assert_eq!(m::compare(&distinct, "", p, l).unwrap().source.len(), 10000);
    assert!(m::compare(&(distinct + "🧊"), "", p, l).is_err());
    let r = m::compare(
        "\u{fb03}",
        "ffi",
        m::Profile {
            normalization: s::Normalization::Nfkd,
            ..p
        },
        l,
    )
    .unwrap();
    assert!(r.equal);
}
