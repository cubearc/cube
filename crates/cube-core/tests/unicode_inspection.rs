use cube_core::text::{
    self,
    unicode::{self, Options},
};
#[test]
fn unicode_observations_are_versioned_exact_and_counts_survive_listing_limits() {
    let input = "Aaǅé中\0\u{200b}\u{200c}\u{200d}\u{a0}\u{202e}\u{fe0f}";
    let report = unicode::inspect(input, &Options::default()).unwrap();
    assert_eq!(report.counts.scalars, 12);
    assert_eq!(report.records[0].case, "uppercase");
    assert_eq!(report.records[1].case, "lowercase");
    assert_eq!(report.records[2].case, "titlecase");
    assert_eq!(report.records[4].case, "uncased");
    assert!(report.records[5].control);
    assert!(report.records[7].format);
    assert!(report.records[8].zero_width_candidate);
    assert!(report.records[9].whitespace);
    assert!(report.records[10].bidi_control);
    assert_eq!(report.category_unicode_version, (16, 0, 0));
    assert_eq!(
        report.derived_case_space_unicode_version,
        std::char::UNICODE_VERSION
    );
    assert_eq!(report.records[4].source.byte_offset, 6);
    let short = unicode::inspect(
        input,
        &Options {
            max_results: 1,
            ..Options::default()
        },
    )
    .unwrap();
    assert_eq!(short.counts.scalars, 12);
    assert_eq!(short.records.len(), 1);
    assert!(!short.complete);
    let zero = unicode::inspect(
        input,
        &Options {
            max_results: 0,
            ..Options::default()
        },
    )
    .unwrap();
    assert!(zero.records.is_empty());
    assert_eq!(zero.counts.format, 4);
    assert!(!zero.complete);
    assert!(
        unicode::inspect(
            "",
            &Options {
                max_results: 0,
                ..Options::default()
            }
        )
        .unwrap()
        .complete
    );
}
#[test]
fn exact_and_normalized_differences_keep_separate_offsets_and_missing_sides() {
    let report = unicode::compare("é", "e\u{301}", "NFC", &Options::default()).unwrap();
    assert!(!report.exact_equal);
    assert_eq!(report.normalized_equal, Some(true));
    assert_eq!(report.original.total_differences, 2);
    assert!(report.original.records[1].left.is_none());
    assert_eq!(
        report.original.records[1]
            .right
            .as_ref()
            .unwrap()
            .byte_offset,
        1
    );
    assert_eq!(
        report.normalized.as_ref().unwrap().coordinate_space,
        "normalized_scalar_index"
    );
    assert_eq!(report.normalized.unwrap().total_differences, 0);
    let report = unicode::compare("ﬀX", "ffY", "NFKC", &Options::default()).unwrap();
    assert_eq!(report.normalized_equal, Some(false));
    let record = &report.normalized.as_ref().unwrap().records[0];
    assert_eq!(record.scalar_index, 2);
    assert_eq!(record.left.as_ref().unwrap().byte_offset, 2);
    assert_eq!(
        report.original.records[1]
            .left
            .as_ref()
            .unwrap()
            .byte_offset,
        3
    );
    assert!(
        unicode::compare("", "", "none", &Options::default())
            .unwrap()
            .exact_equal
    );
    let report = unicode::compare(
        "AAA",
        "aaa",
        "NFC",
        &Options {
            max_results: 1,
            ..Options::default()
        },
    )
    .unwrap();
    assert_eq!(report.original.total_differences, 3);
    assert_eq!(report.normalized.as_ref().unwrap().total_differences, 3);
    assert_eq!(report.original.records.len(), 1);
    assert!(report.normalized.as_ref().unwrap().records.is_empty());
    assert!(!report.complete);
}
#[test]
fn unicode_input_work_report_and_normalized_expansion_bounds() {
    let report = unicode::inspect("A", &Options::default()).unwrap();
    let bytes = serde_json::to_vec(&report).unwrap().len();
    assert!(
        unicode::inspect(
            "A",
            &Options {
                max_work: report.work_units,
                max_output: bytes,
                ..Options::default()
            }
        )
        .is_ok()
    );
    assert!(
        unicode::inspect(
            "A",
            &Options {
                max_work: report.work_units - 1,
                ..Options::default()
            }
        )
        .is_err()
    );
    assert!(
        unicode::inspect(
            "A",
            &Options {
                max_output: bytes - 1,
                ..Options::default()
            }
        )
        .is_err()
    );
    let report = unicode::compare("A", "a", "NFC", &Options::default()).unwrap();
    assert!(
        unicode::compare(
            "A",
            "a",
            "NFC",
            &Options {
                max_work: report.work_units,
                ..Options::default()
            }
        )
        .is_ok()
    );
    assert!(
        unicode::compare(
            "A",
            "a",
            "NFC",
            &Options {
                max_work: report.work_units - 1,
                ..Options::default()
            }
        )
        .is_err()
    );
    assert!(unicode::compare("", "", "bad", &Options::default()).is_err());
    assert!(
        unicode::inspect(
            "",
            &Options {
                max_results: 65537,
                ..Options::default()
            }
        )
        .is_err()
    );
    assert!(
        unicode::inspect(
            &"A".repeat(text::MAX_INPUT),
            &Options {
                max_results: 0,
                ..Options::default()
            }
        )
        .is_ok()
    );
    assert!(unicode::inspect(&"A".repeat(text::MAX_INPUT + 1), &Options::default()).is_err());
    assert!(
        unicode::compare(
            &"\u{fdfa}".repeat(116509),
            "",
            "NFKD",
            &Options {
                max_results: 0,
                ..Options::default()
            }
        )
        .is_err()
    );
}
