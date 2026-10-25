use cube_core::{
    errors::ErrorCode,
    text::{
        self,
        statistics::{
            CaseMode, MAX_DISTINCT, MAX_WORK, Normalization, Options, TextFilter, analyze,
        },
    },
};
#[test]
fn exact_distributions_overlapping_windows_and_numeric_conventions() {
    let report = analyze(
        "ABAB",
        Options {
            ngram_size: 2,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        (report.scalar_count, report.windows, report.distinct_ngrams),
        (4, 3, 2)
    );
    assert_eq!(
        (&*report.frequencies[0].ngram, report.frequencies[0].count),
        ("AB", 2)
    );
    assert!((report.index_of_coincidence.unwrap() - 1.0 / 3.0).abs() < 1e-12);
    assert_eq!(report.shannon_entropy_bits, Some(1.0));
    let constant = analyze("AAAA", Options::default()).unwrap();
    assert_eq!(constant.index_of_coincidence, Some(1.0));
    assert_eq!(constant.shannon_entropy_bits, Some(0.0));
    let empty = analyze("", Options::default()).unwrap();
    assert!(empty.frequencies.is_empty());
    assert_eq!(empty.index_of_coincidence, None);
    assert_eq!(empty.shannon_entropy_bits, None);
    let one = analyze("🧊", Options::default()).unwrap();
    assert_eq!(one.scalar_count, 1);
    assert_eq!(one.index_of_coincidence, None);
    assert_eq!(one.shannon_entropy_bits, Some(0.0));
    let short = analyze(
        "ABC",
        Options {
            ngram_size: 4,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(short.windows, 0);
    assert!(short.frequencies.is_empty());
    assert!((short.shannon_entropy_bits.unwrap() - 3f64.log2()).abs() < 1e-12);
}
#[test]
fn normalization_filtering_scalar_units_and_top_counts_are_explicit() {
    let options = Options {
        normalization: Normalization::Nfc,
        ..Default::default()
    };
    let report = analyze("ée\u{301}", options).unwrap();
    assert_eq!(report.scalar_count, 2);
    assert_eq!(report.frequencies[0].ngram, "é");
    assert_eq!(report.frequencies[0].count, 2);
    let report = analyze(
        "B!Aa?雪",
        Options {
            case: CaseMode::AsciiLower,
            filter: TextFilter::AsciiLetters,
            max_results: 1,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(report.scalar_count, 3);
    assert_eq!(report.frequencies[0].ngram, "a");
    assert_eq!(report.frequencies[0].count, 2);
    assert!(!report.complete);
    assert_eq!(report.distinct_ngrams, 2);
    let tied = analyze("BA", Options::default()).unwrap();
    assert_eq!(tied.frequencies[0].ngram, "A");
    for name in ["none", "NFC", "NFD", "NFKC", "NFKD"] {
        assert!(Normalization::parse(name).is_ok());
    }
    assert!(Normalization::parse("guess").is_err());
    assert!(CaseMode::parse("fold").is_err());
    assert!(TextFilter::parse("guess").is_err());
    assert!(CaseMode::parse("ascii_lower").is_ok());
    assert!(TextFilter::parse("ascii_letters").is_ok());
}
#[test]
fn distinct_input_work_and_result_budgets_are_enforced() {
    for options in [
        Options {
            ngram_size: 0,
            ..Default::default()
        },
        Options {
            ngram_size: 17,
            ..Default::default()
        },
        Options {
            max_results: 0,
            ..Default::default()
        },
        Options {
            max_results: MAX_DISTINCT + 1,
            ..Default::default()
        },
    ] {
        assert!(analyze("text", options).is_err());
    }
    assert_eq!(
        analyze(&"a".repeat(text::MAX_INPUT + 1), Options::default())
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        analyze(
            &"A".repeat(MAX_WORK / 16 + 1),
            Options {
                ngram_size: 16,
                ..Default::default()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
    let distinct: String = (0x1000..0x1000 + MAX_DISTINCT as u32 + 1)
        .map(|n| char::from_u32(n).unwrap())
        .collect();
    assert!(analyze(&distinct, Options::default()).is_err());
    let combinations: String = (0..10020).map(|n| format!("{n:05},")).collect();
    assert!(
        analyze(
            &combinations,
            Options {
                ngram_size: 7,
                ..Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(
        analyze(
            &"\u{fdfa}".repeat(65536),
            Options {
                normalization: Normalization::Nfkd,
                ..Default::default()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
}
