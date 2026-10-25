use cube_core::{
    blockchain::{Network, candidates::*, ethereum::Checksum, wif},
    encoding::base58check,
    errors::ErrorCode,
};
const WIF: &str = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";
fn scalar() -> String {
    format!("{:064x}", 1)
}
fn candidate(format: Format, value: &str) -> Candidate {
    Candidate::new(format, value).unwrap()
}
#[test]
fn mixed_entries_order_counts_and_safe_metadata_match_single_validators() {
    let hex = scalar();
    let bad_checksum = format!("{}x", &WIF[..WIF.len() - 1]);
    let zero = base58check::encode(0x80, &[0; 32]).unwrap();
    let wrong_version = base58check::encode(1, &[1; 32]).unwrap();
    let suffix = base58check::encode(0x80, &[1; 33]).unwrap(); // last=01 is valid
    let invalid_suffix = base58check::encode(0x80, &[2; 33]).unwrap();
    let mut batch = CandidateBatch::new(
        vec![
            candidate(Format::Wif, WIF),
            candidate(Format::Hex, &hex),
            candidate(Format::Wif, &bad_checksum),
            candidate(Format::Wif, &zero),
            candidate(Format::Wif, &wrong_version),
            candidate(Format::Wif, &invalid_suffix),
            candidate(Format::Wif, &suffix),
            candidate(Format::Unsupported, "secret-label"),
            candidate(Format::Hex, ""),
            candidate(Format::Hex, &hex),
        ],
        Derive::None,
    )
    .unwrap();
    let plan = batch.plan();
    assert_eq!(batch.len(), 10);
    assert!(!batch.progress().complete);
    let rows: Vec<_> = batch.by_ref().collect();
    assert_eq!(rows.len(), 10);
    for (i, row) in rows.iter().enumerate() {
        assert_eq!(row.index, i);
        assert!(row.public_address.is_none());
        assert!(row.address_error.is_none());
    }
    assert_eq!(
        rows.iter().map(|r| r.valid).collect::<Vec<_>>(),
        vec![
            true, true, false, false, false, false, true, false, false, true
        ]
    );
    assert_eq!(
        rows[0].wif.unwrap().compressed,
        wif::validate(WIF, None).unwrap().compressed
    );
    assert_eq!(
        rows[7].error.as_ref().unwrap().code,
        ErrorCode::UnsupportedFormat
    );
    let progress = batch.progress();
    assert!(progress.complete);
    assert_eq!(progress.processed, 10);
    assert_eq!(progress.valid, 4);
    assert_eq!(progress.invalid, 6);
    assert_eq!(progress.public_derived, 0);
    assert_eq!(batch.len(), 0);
    assert!(batch.next().is_none());
    let report = serde_json::to_string_pretty(
        &serde_json::json!({"results":rows,"progress":progress,"plan":plan}),
    )
    .unwrap();
    assert!(report.len() <= plan.max_output_bytes);
    for secret in [
        WIF,
        &hex,
        &bad_checksum,
        &zero,
        &wrong_version,
        &invalid_suffix,
        "secret-label",
    ] {
        assert!(!report.contains(secret));
    }
}
#[test]
fn public_derivation_is_explicit_and_intent_failure_preserves_validation() {
    let hex = scalar();
    let mut btc = CandidateBatch::new(
        vec![candidate(Format::Wif, WIF), candidate(Format::Hex, &hex)],
        Derive::Bitcoin {
            network: Network::Mainnet,
            compressed: true,
        },
    )
    .unwrap();
    for row in btc.by_ref() {
        assert!(row.valid);
        let PublicAddress::Bitcoin(public) = row.public_address.unwrap() else {
            panic!()
        };
        assert_eq!(public.address, "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH");
    }
    assert_eq!(btc.progress().public_derived, 2);
    let uncompressed = "5HpHagT65TZzG1PH3CSu63k8DbpvD8s5ip4nEB3kEsreAnchuDf";
    let row = CandidateBatch::new(
        vec![candidate(Format::Wif, uncompressed)],
        Derive::Bitcoin {
            network: Network::Mainnet,
            compressed: false,
        },
    )
    .unwrap()
    .next()
    .unwrap();
    assert!(!row.wif.unwrap().compressed);
    let PublicAddress::Bitcoin(public) = row.public_address.unwrap() else {
        panic!()
    };
    assert_eq!(public.address, "1EHNa6Q4Jz2uvNExL497mE43ikXhwF6kZm");
    let mut maximum = CandidateBatch::new(
        (0..32).map(|_| candidate(Format::Hex, &hex)).collect(),
        Derive::Bitcoin {
            network: Network::Testnet,
            compressed: false,
        },
    )
    .unwrap();
    let plan = maximum.plan();
    let rows: Vec<_> = maximum.by_ref().collect();
    let output = serde_json::to_vec_pretty(
        &serde_json::json!({"plan":plan,"results":rows,"progress":maximum.progress()}),
    )
    .unwrap();
    assert!(output.len() < plan.max_output_bytes);

    for mode in [
        Derive::Bitcoin {
            network: Network::Testnet,
            compressed: true,
        },
        Derive::Bitcoin {
            network: Network::Mainnet,
            compressed: false,
        },
        Derive::Ethereum {
            checksum: Checksum::Eip55,
        },
    ] {
        let row = CandidateBatch::new(vec![candidate(Format::Wif, WIF)], mode)
            .unwrap()
            .next()
            .unwrap();
        assert!(row.valid);
        assert!(row.error.is_none());
        assert!(row.address_error.is_some());
        assert!(row.public_address.is_none());
    }
    let row = CandidateBatch::new(
        vec![candidate(Format::Hex, &hex)],
        Derive::Ethereum {
            checksum: Checksum::Eip55,
        },
    )
    .unwrap()
    .next()
    .unwrap();
    let PublicAddress::Ethereum(public) = row.public_address.unwrap() else {
        panic!()
    };
    assert_eq!(public.address, "0x7E5F4552091A69125d5DfCb7b8C2659029395Bdf");
    assert!(derivation(None, Some(Network::Mainnet), None, None).is_err());
    assert!(derivation(Some("bitcoin"), None, Some(true), None).is_err());
    assert!(derivation(Some("bitcoin"), Some(Network::Mainnet), None, None).is_err());
    assert!(
        derivation(
            Some("bitcoin"),
            Some(Network::Mainnet),
            Some(true),
            Some("eip55")
        )
        .is_err()
    );
    assert!(derivation(Some("ethereum"), None, None, Some("bad")).is_err());
    assert!(derivation(Some("unknown"), None, None, None).is_err());
    assert!(matches!(
        derivation(None, None, None, None).unwrap(),
        Derive::None
    ));
    assert!(matches!(
        derivation(Some("ethereum"), None, None, None).unwrap(),
        Derive::Ethereum { .. }
    ));
    assert!(matches!(
        derivation(Some("bitcoin"), Some(Network::Mainnet), Some(true), None).unwrap(),
        Derive::Bitcoin { .. }
    ));
}
#[test]
fn preflight_limits_and_partial_progress_are_honest() {
    let make = |n, format, len| {
        (0..n)
            .map(|_| candidate(format, &"a".repeat(len)))
            .collect::<Vec<_>>()
    };
    assert!(Candidate::new(Format::Hex, &"a".repeat(129)).is_err());
    let boundary = CandidateBatch::new(make(256, Format::Hex, 128), Derive::None).unwrap();
    assert_eq!(boundary.plan().input_bytes, 32768);
    assert_eq!(boundary.len(), 256);
    assert!(CandidateBatch::new(make(257, Format::Hex, 1), Derive::None).is_err());
    assert!(CandidateBatch::new(make(256, Format::Wif, 128), Derive::None).is_err());
    assert_eq!(
        CandidateBatch::new(make(64, Format::Wif, 126), Derive::None)
            .unwrap()
            .plan()
            .work_units,
        MAX_WORK
    );
    assert!(CandidateBatch::new(make(65, Format::Wif, 126), Derive::None).is_err());
    let mode = Derive::Bitcoin {
        network: Network::Mainnet,
        compressed: true,
    };
    assert!(CandidateBatch::new(make(32, Format::Hex, 64), mode).is_ok());
    assert!(CandidateBatch::new(make(33, Format::Hex, 64), mode).is_err());
    let mut batch = CandidateBatch::new(make(2, Format::Hex, 64), Derive::None).unwrap();
    assert_eq!(batch.next().unwrap().index, 0);
    assert_eq!(batch.len(), 1);
    assert!(!batch.progress().complete);
    assert_eq!(batch.progress().processed, 1);
    assert!(!format!("{batch:?}").contains(&"a".repeat(64)));
    let empty = CandidateBatch::new(vec![], Derive::None).unwrap();
    assert!(empty.progress().complete);
    assert_eq!(empty.len(), 0);
    assert_eq!(Format::parse("wif"), Format::Wif);
    assert_eq!(Format::parse("hex"), Format::Hex);
    assert_eq!(Format::parse("private-label"), Format::Unsupported);
}
#[test]
fn strict_transport_rejects_schema_duplicate_keys_nesting_and_budget_without_causes() {
    use std::error::Error;
    let valid = format!(r#"[{{"format":"hex","value":"{}"}}]"#, scalar());
    let batch = CandidateBatch::new(parse_json(valid.as_bytes()).unwrap(), Derive::None).unwrap();
    assert_eq!(batch.len(), 1);
    for bad in [
        "{}".into(),
        "[1]".into(),
        r#"[{"format":"hex","format":"wif","value":"secret-material"}]"#.into(),
        r#"[{"format":"hex","value":1}]"#.into(),
        r#"[{"format":"hex","value":"secret-material","extra":0}]"#.into(),
        "[".repeat(18) + &"]".repeat(18),
        "[0,".to_owned() + &"0,".repeat(4096) + "0]",
        format!(r#"[{{"format":"hex","value":"{}"}}]"#, "a".repeat(129)),
        " ".repeat(MAX_DOCUMENT_BYTES + 1),
    ] {
        let e = parse_json(bad.as_bytes()).unwrap_err();
        assert!(e.source().is_none());
        assert!(!format!("{e:?}").contains("secret-material"));
    }
    let long_label =
        serde_json::to_vec(&serde_json::json!([{"format":"a".repeat(129),"value":""}])).unwrap();
    assert!(parse_json(&long_label).is_err());
    assert!(parse_json(&[255]).is_err());
    assert!(parse_json(&vec![b' '; MAX_DOCUMENT_BYTES]).is_err());
    let too_many =
        serde_json::to_vec(&vec![serde_json::json!({"format":"hex","value":""}); 257]).unwrap();
    assert!(parse_json(&too_many).is_err());
    let entries = serde_json::to_vec(&vec![
        serde_json::json!({"format":"hex","value":"a".repeat(128)});
        256
    ])
    .unwrap();
    assert!(entries.len() < MAX_DOCUMENT_BYTES);
    assert_eq!(parse_json(&entries).unwrap().len(), 256);
}
