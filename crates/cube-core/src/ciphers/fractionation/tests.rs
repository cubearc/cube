use super::*;
use serde::de::DeserializeOwned;
use serde_json::json;
fn p<T: DeserializeOwned>(v: serde_json::Value) -> T {
    serde_json::from_value(v).unwrap()
}
#[test]
fn coordinate_periods_aliases_and_exact_cube_symbols() {
    for n in [1, 2, 7, 4096] {
        let b = Bifid {
            square: IJ.into(),
            period: n,
        };
        for t in ["", "A", "ABCDEFGHIJKLMNO", "Z"] {
            let e = bifid(t, &b, false).unwrap();
            assert_eq!(bifid(&e, &b, true).unwrap(), t.replace('J', "I"));
        }
        let t = Trifid {
            alphabet: "😀βABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().take(27).collect(),
            period: n,
        };
        let e = trifid("A😀βZ".trim_end_matches('Z'), &t, false).unwrap();
        assert_eq!(trifid(&e, &t, true).unwrap(), "A😀β");
    }
    assert!(
        bifid(
            "a",
            &Bifid {
                square: IJ.into(),
                period: 1
            },
            false
        )
        .is_err()
    );
    for n in [0, 4097] {
        assert!(
            bifid(
                "",
                &Bifid {
                    square: IJ.into(),
                    period: n
                },
                false
            )
            .is_err()
        );
        assert!(
            trifid(
                "",
                &Trifid {
                    alphabet: (AZ.to_owned() + "#"),
                    period: n
                },
                false
            )
            .is_err()
        );
    }
    assert!(
        bifid(
            "",
            &Bifid {
                square: AZ.into(),
                period: 1
            },
            false
        )
        .is_err()
    );
    assert!(
        trifid(
            "",
            &Trifid {
                alphabet: "A".repeat(27),
                period: 1
            },
            false
        )
        .is_err()
    );
    assert!(
        trifid(
            "?",
            &Trifid {
                alphabet: AZ.to_owned() + "#",
                period: 1
            },
            false
        )
        .is_err()
    );
    let b = Bifid {
        square: IJ.into(),
        period: 4096,
    };
    assert_eq!(
        bifid(&"A".repeat(65536), &b, false).unwrap(),
        "A".repeat(65536)
    );
    assert!(bifid(&"A".repeat(65537), &b, false).is_err());
}
#[test]
fn coordinates_and_ordinary_addition_are_strict() {
    let p = Adfgx {
        square: IJ.into(),
        key: ColumnKey::Keyword {
            text: "BALLOON".into(),
        },
    };
    for text in ["", "JAZZ", "ATTACKATDAWN"] {
        let e = adfgx(text, &p, false).unwrap();
        assert_eq!(adfgx(&e, &p, true).unwrap(), text.replace('J', "I"));
    }
    for text in ["A", "AA?A", "aa"] {
        assert!(adfgx(text, &p, true).is_err());
    }
    assert!(adfgx("A A", &p, false).is_err());
    let p = Adfgvx {
        square: AZ.to_owned() + "0123456789",
        key: ColumnKey::Keyword {
            text: "BALLOON".into(),
        },
    };
    let e = adfgvx("J09", &p, false).unwrap();
    assert_eq!(adfgvx(&e, &p, true).unwrap(), "J09");
    let n = Nihilist {
        square: IJ.into(),
        key: "ZZ".into(),
    };
    assert_eq!(nihilist_encode("UZ", &n).unwrap(), [100, 110]);
    assert_eq!(nihilist_decode(&[100, 110], &n).unwrap(), "UZ");
    for v in [0, 55, 111, u16::MAX] {
        assert!(nihilist_decode(&[v], &n).is_err());
    }
    assert!(
        nihilist_encode(
            "",
            &Nihilist {
                square: IJ.into(),
                key: "".into()
            }
        )
        .is_err()
    );
    assert!(nihilist_decode(&vec![110; 65537], &n).is_err());
    let b = Bazeries {
        key: 111111,
        square: IJ.into(),
    };
    let e = bazeries("JAZZ", &b, false).unwrap();
    assert_eq!(bazeries(&e, &b, true).unwrap(), "IAZZ");
    for key in [0, 10, 1000000] {
        assert!(
            bazeries(
                "",
                &Bazeries {
                    key,
                    square: IJ.into()
                },
                false
            )
            .is_err()
        );
    }
}
#[test]
fn morse_padding_evidence_and_independent_digit_ownership() {
    use morse::*;
    let mut f = FractionatedMorse {
        alphabet: AZ.into(),
        padding_count: None,
    };
    for text in ["", "E", "T", "A B", "THE QUICK BROWN FOX 0123456789"] {
        let r = fractionated_morse(text, &f, false).unwrap();
        let unresolved = fractionated_morse(&r.ciphertext, &f, true).unwrap();
        assert!(unresolved.padding_ambiguous);
        assert!(unresolved.text.is_none());
        f.padding_count = Some(r.padding_count);
        assert_eq!(
            fractionated_morse(&r.ciphertext, &f, true)
                .unwrap()
                .text
                .as_deref(),
            Some(text)
        );
        f.padding_count = None;
    }
    f.padding_count = Some(3);
    assert!(fractionated_morse("", &f, true).is_err());
    f.padding_count = Some(1);
    assert!(fractionated_morse("A", &f, true).is_err());
    f.padding_count = None;
    assert!(fractionated_morse("?", &f, true).is_err());
    for t in [" e", "a", "A  B", "A ", ".", "A\n"] {
        assert!(fractionated_morse(t, &f, false).is_err());
    }
    let all_pairs = Morbit {
        digits: "123456789".into(),
        padding_count: None,
    };
    assert_eq!(
        morbit("123456789", &all_pairs, true).unwrap().morse,
        ["..", ".-", ".x", "-.", "--", "-x", "x.", "x-", "xx"].concat()
    );
    let all_triples = FractionatedMorse {
        alphabet: AZ.into(),
        padding_count: None,
    };
    assert_eq!(
        fractionated_morse(AZ, &all_triples, true).unwrap().morse,
        [
            "...", "..-", "..x", ".-.", ".--", ".-x", ".x.", ".x-", ".xx", "-..", "-.-", "-.x",
            "--.", "---", "--x", "-x.", "-x-", "-xx", "x..", "x.-", "x.x", "x-.", "x--", "x-x",
            "xx.", "xx-"
        ]
        .concat()
    );
    let published = Pollux {
        classes: ".x-..x.--x".into(),
    };
    assert_eq!(
        pollux("086393425702417685963041456234908745360", &published, true)
            .unwrap()
            .text
            .as_deref(),
        Some("LUCK HELPS")
    );
    let m = Morbit {
        digits: "123456789".into(),
        padding_count: Some(0),
    };
    assert!(morbit("9", &m, true).is_err());
    assert!(morbit("A", &m, true).is_err());
    assert!(morbit("A", &m, false).is_err());
    for classes in ["..........", ".-x", "abcdefghij"] {
        assert!(
            pollux(
                "",
                &Pollux {
                    classes: classes.into()
                },
                false
            )
            .is_err()
        );
    }
    let pollux_params = Pollux {
        classes: ".-x.......".into(),
    };
    let r = pollux("A A", &pollux_params, false).unwrap();
    assert_eq!(
        pollux(&r.ciphertext, &pollux_params, true)
            .unwrap()
            .text
            .as_deref(),
        Some("A A")
    );
    assert!(pollux("?", &pollux_params, true).is_err());
    assert!(pollux("2", &pollux_params, true).is_err());
    assert!(pollux(&"E".repeat(65536), &pollux_params, false).is_err());
    assert!(fractionated_morse(&"A".repeat(65537), &f, false).is_err());
    assert!(pollux(&"0".repeat(65537), &pollux_params, true).is_err());
}
#[test]
fn parameter_transport_rejects_duplicates_and_unknown_fields() {
    assert!(
        crate::ciphers::poly::parse_parameters::<Bifid>(
            "{\"square\":\"A\",\"period\":1,\"period\":2}"
        )
        .is_err()
    );
    assert!(
        crate::ciphers::poly::parse_parameters::<Bifid>(
            "{\"square\":\"A\",\"period\":1,\"extra\":0}"
        )
        .is_err()
    );
    let _: Nihilist = p(json!({"square":IJ,"key":"KEY"}));
}
