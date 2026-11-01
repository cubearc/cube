use cube_core::ciphers::{
    fractionation::{
        self,
        cm::{self, CmBifid},
    },
    transposition::tables::{self, Cadenus, Convention, Nihilist, Readout, Swagman},
};
use cube_core::errors::ErrorCode;
use cube_core::operations::execution::{Budget, execute};
use serde_json::{Value, json};

fn parameters<T: serde::de::DeserializeOwned>(v: Value) -> T {
    serde_json::from_value(v).unwrap()
}

fn round_trip(name: &str, plain: &str, cipher: &str, parameters: Value, report: bool) {
    let expected = |text: &str| {
        if report {
            json!({"text":text,"original_length":plain.chars().count(),"padding_count":0})
        } else {
            json!({"text":text})
        }
    };
    for (direction, input, output) in [("encode", plain, cipher), ("decode", cipher, plain)] {
        let operation = format!("cipher.{name}.{direction}");
        let actual = execute(
            &operation,
            1,
            &json!({"text":input}),
            &parameters,
            Budget::default(),
        )
        .unwrap_or_else(|error| panic!("{operation}: {error:?}"));
        assert_eq!(actual, expected(output), "{operation}");
    }
}

#[test]
fn aca_cadenus_printed_easy_block_and_inverse() {
    round_trip(
        "cadenus",
        "ASEVERELIMITATIONONTHEUSEFULNESSOFTHECADENUSISTHATEVERYMESSAGEMUSTBEAMULTIPLEOFTWENTYFIVELETTERSLONG",
        "SYSTRETOMTATTLUSOATLEEESFIYHEASDFNMSCHBHNEUVSNPMTOFARENUSEIEEIELTARLMENTIEETOGEVESITFAISLTNGEEUVOWUL",
        json!({"keyword":"EASY"}),
        true,
    );
}

#[test]
fn aca_swagman_printed_latin_square_and_inverse() {
    round_trip(
        "swagman",
        "DONTBEAFRAIDTOTAKEABIGLEAPIFONEISINDICATEDYOUCANNOTCROSSARIVERORACHASMINTWOSMALLJUMPS",
        "ENDSCMORDANIBOISICTNASTGBLTEWAOAREEFSAIDVPYRMOEAIAFUILRLDOCOTJNRAAENOUNCMITSOAPHSKATI",
        json!({"key":[[3,2,1,4,5],[1,5,3,2,4],[2,4,5,3,1],[5,3,4,1,2],[4,1,2,5,3]]}),
        true,
    );
}

#[test]
fn aca_nihilist_transposition_printed_square_both_readouts_and_inverse() {
    for convention in ["writing_in", "taking_out"] {
        for (readout, cipher) in [
            ("columns", "EQDERSEHNUEREADE"),
            ("rows", "ERNEQSUADEEDEHRE"),
        ] {
            round_trip(
                "nihilist_transposition",
                "SQUARENEEDEDHERE",
                cipher,
                json!({"key":[2,1,3,4],"convention":convention,"readout":readout}),
                true,
            );
        }
    }
}

#[test]
fn aca_cm_bifid_unequal_squares_printed_periods_and_inverse() {
    round_trip(
        "cm_bifid",
        "ODDPERIODSAREPOPULAR",
        "FANXZEXFENUKKRBYNKAK",
        json!({"input_square":"EXTRAKLMPOHWZQDGVUSIFCBYN","output_square":"NCDRSOBFQUVAGPWEYHMXLTIKZ","period":7}),
        false,
    );
}

#[test]
fn non_self_inverse_nihilist_keys_distinguish_the_two_published_conventions() {
    for (convention, expected) in [
        (Convention::WritingIn, "IGHCABFDE"),
        (Convention::TakingOut, "EFDHIGBCA"),
    ] {
        let p = Nihilist {
            key: vec![2, 3, 1],
            convention,
            readout: Readout::Rows,
            padding: None,
            original_length: None,
        };
        assert_eq!(
            tables::nihilist("ABCDEFGHI", &p, false).unwrap().text,
            expected
        );
        assert_eq!(
            tables::nihilist(expected, &p, true).unwrap().text,
            "ABCDEFGHI"
        );
    }
}

#[test]
fn exact_unicode_and_explicit_padding_evidence_never_guess_nulls() {
    let plain = "X🐈β\0e\u{301} X";
    let mut p: Cadenus = parameters(json!({"keyword":"VWVW","padding":"X"}));
    let encoded = tables::cadenus(plain, &p, false).unwrap();
    assert_eq!(encoded.original_length, plain.chars().count());
    assert_eq!(encoded.padding_count, 100 - plain.chars().count());
    let full = tables::cadenus(&encoded.text, &p, true).unwrap();
    assert_eq!(full.text.chars().count(), 100);
    assert!(full.text.starts_with(plain));
    p.original_length = Some(encoded.original_length);
    assert_eq!(
        tables::cadenus(&encoded.text, &p, true).unwrap().text,
        plain
    );
    p.padding = Some('Y');
    assert_eq!(
        tables::cadenus(&encoded.text, &p, true).unwrap_err().code,
        ErrorCode::InvalidParameter
    );
    p.original_length = Some(101);
    assert!(tables::cadenus(&encoded.text, &p, true).is_err());
    for h in 4..=8 {
        let key = (0..h)
            .map(|r| (0..h).map(|c| ((r + c) % h + 1) as u8).collect())
            .collect();
        let mut p = Swagman {
            key,
            padding: Some('🐈'),
            original_length: None,
        };
        let out = tables::swagman(plain, &p, false).unwrap();
        p.original_length = Some(plain.chars().count());
        assert_eq!(tables::swagman(&out.text, &p, true).unwrap().text, plain);
    }
    for side in 1..=10 {
        for readout in [Readout::Rows, Readout::Columns] {
            let mut p = Nihilist {
                key: (1..=side).rev().collect(),
                convention: Convention::TakingOut,
                readout,
                padding: Some('X'),
                original_length: None,
            };
            let out = tables::nihilist(plain, &p, false).unwrap();
            p.original_length = Some(plain.chars().count());
            assert_eq!(tables::nihilist(&out.text, &p, true).unwrap().text, plain);
        }
    }
}

#[test]
fn malformed_labels_latin_squares_permutations_and_ragged_ciphertexts_reject() {
    for keyword in ["", "easy", "É", "EASY "] {
        assert!(tables::cadenus("", &parameters(json!({"keyword":keyword})), false).is_err());
    }
    let c: Cadenus = parameters(json!({"keyword":"EASY"}));
    assert!(tables::cadenus("A", &c, false).is_err());
    assert!(tables::cadenus("A", &c, true).is_err());
    for key in [
        json!([]),
        json!([[1, 2], [2, 1]]),
        json!(vec![vec![1, 2, 3, 4]; 4]),
        json!([[1, 2, 3, 4], [2, 3, 4, 1], [3, 4, 1, 2], [0, 1, 2, 3]]),
        json!([[1, 2, 3, 4], [2, 3, 4, 1], [3, 4, 1, 2], [4, 1, 2]]),
    ] {
        assert!(tables::swagman("", &parameters(json!({"key":key})), false).is_err());
    }
    for key in [vec![], vec![0], vec![2, 2], vec![1, 2, 4], vec![1; 11]] {
        let p = Nihilist {
            key,
            convention: Convention::WritingIn,
            readout: Readout::Rows,
            padding: None,
            original_length: None,
        };
        assert!(tables::nihilist("", &p, false).is_err());
    }
    let p: Swagman = parameters(json!({"key":[[1,2,3,4],[2,3,4,1],[3,4,1,2],[4,1,2,3]]}));
    assert!(tables::swagman("A", &p, true).is_err());
    assert_eq!(tables::swagman("", &p, false).unwrap().text, "");
    let bad = json!({"keyword":"A","unknown":true});
    assert!(
        execute(
            "cipher.cadenus.encode",
            1,
            &json!({"text":""}),
            &bad,
            Budget::default()
        )
        .is_err()
    );
}

#[test]
fn table_resource_boundaries_include_padding_and_full_cadenus_geometry() {
    let p: Cadenus = parameters(json!({"keyword":"A".repeat(2621)}));
    assert_eq!(
        tables::cadenus(&"A".repeat(65525), &p, false)
            .unwrap()
            .text
            .len(),
        65525
    );
    let too_wide: Cadenus = parameters(json!({"keyword":"A".repeat(2622)}));
    assert_eq!(
        tables::cadenus("", &too_wide, false).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let key = (0..8)
        .map(|r| (0..8).map(|c| ((r + c) % 8 + 1) as u8).collect())
        .collect();
    let p = Swagman {
        key,
        padding: Some('X'),
        original_length: None,
    };
    let exact = "🐈".repeat(65536);
    assert_eq!(tables::swagman(&exact, &p, false).unwrap().text, exact);
    assert_eq!(
        tables::swagman(&"A".repeat(65537), &p, false)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let p: Nihilist = parameters(
        json!({"key":[1,2,3,4,5,6,7,8,9,10],"convention":"taking_out","readout":"columns","padding":"X"}),
    );
    assert_eq!(
        tables::nihilist(&"A".repeat(65535), &p, false)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn cm_bifid_retains_basic_bifid_and_canonical_aliases_and_short_periods() {
    let plain = "ODDPERIODSAREPOPULAR";
    let first = "EXTRAKLMPOHWZQDGVUSIFCBYN";
    assert_eq!(
        fractionation::bifid(
            plain,
            &parameters(json!({"square":first,"period":7})),
            false
        )
        .unwrap(),
        "MWEINGIMGEOYYRLVEYWY"
    );
    for period in [1, 2, 7, 20, 4096] {
        let params: CmBifid = parameters(
            json!({"input_square":first,"output_square":"NCDRSOBFQUVAGPWEYHMXLTIKZ","period":period}),
        );
        let out = cm::cm_bifid(plain, &params, false).unwrap();
        assert_eq!(cm::cm_bifid(&out, &params, true).unwrap(), plain);
        assert_eq!(cm::cm_bifid("", &params, false).unwrap(), "");
        assert_eq!(
            cm::cm_bifid(&cm::cm_bifid("J", &params, false).unwrap(), &params, true).unwrap(),
            "I"
        );
    }
    let mut params: CmBifid =
        parameters(json!({"input_square":first,"output_square":first,"period":7}));
    assert_eq!(
        cm::cm_bifid(plain, &params, false).unwrap(),
        "MWEINGIMGEOYYRLVEYWY"
    );
    assert_eq!(
        cm::cm_bifid(&"A".repeat(65536), &params, false)
            .unwrap()
            .len(),
        65536
    );
    assert_eq!(
        cm::cm_bifid(&"A".repeat(65537), &params, false)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert!(cm::cm_bifid("é", &params, false).is_err());
    params.period = 0;
    assert!(cm::cm_bifid("", &params, false).is_err());
    params.period = 4097;
    assert!(cm::cm_bifid("", &params, false).is_err());
    params.period = 7;
    params.output_square = "A".repeat(25);
    assert!(cm::cm_bifid("", &params, false).is_err());
}
