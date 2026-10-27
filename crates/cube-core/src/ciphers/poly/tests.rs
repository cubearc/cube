use super::*;
use serde::de::DeserializeOwned;
use serde_json::json;
fn p<T: DeserializeOwned>(value: serde_json::Value) -> T {
    serde_json::from_value(value).unwrap()
}
macro_rules! round {
    ($function:path,$params:expr,$text:expr) => {{
        let params = $params;
        let source: &str = $text;
        let encoded = $function(source, &params, false).unwrap();
        assert_eq!($function(&encoded, &params, true).unwrap(), source);
        assert_eq!($function("", &params, false).unwrap(), "");
        assert_eq!($function("", &params, true).unwrap(), "");
    }};
}
#[test]
fn letter_schedules_preserve_every_nonletter_and_handle_signed_endpoints() {
    let text = "AaZz !\0é e\u{301} 世界";
    round!(
        schedules::gronsfeld,
        p::<schedules::Gronsfeld>(json!({"key":"90"})),
        text
    );
    for start in [i32::MIN, -26, 0, 26, i32::MAX] {
        for step in [i32::MIN, -1, 0, 25, i32::MAX] {
            let params = schedules::Trithemius { start, step };
            round!(schedules::trithemius, params, text);
        }
    }
    for feedback in ["plaintext", "ciphertext"] {
        round!(
            schedules::autokey,
            p::<schedules::Autokey>(json!({"seed":"KEY","feedback":feedback})),
            text
        );
    }
    let params = p::<schedules::RunningKey>(json!({"key":"ABCDEF"}));
    round!(schedules::running_key, params, text);
    assert!(schedules::running_key("A", &p(json!({"key":""})), false).is_err());
    assert_eq!(
        schedules::running_key("世界", &p(json!({"key":""})), false).unwrap(),
        "世界"
    );
    let key = p::<schedules::Key>(json!({"key":"PORTA"}));
    let cipher = schedules::porta(text, &key).unwrap();
    assert_eq!(schedules::porta(&cipher, &key).unwrap(), text);
    for key in ["", "é", "1A"] {
        assert!(
            schedules::autokey("A", &p(json!({"seed":key,"feedback":"plaintext"})), false).is_err()
        );
    }
    for key in ["", "A", "1 2"] {
        assert!(schedules::gronsfeld("A", &p(json!({"key":key})), false).is_err());
    }
    assert!(schedules::gronsfeld("", &p(json!({"key":"1".repeat(MAX_KEY+1)})), false).is_err());
    assert_eq!(
        schedules::trithemius(
            &"A".repeat(MAX_INPUT),
            &p(json!({"start":0,"step":0})),
            false
        )
        .unwrap()
        .len(),
        MAX_INPUT
    );
    assert_eq!(
        schedules::trithemius(
            &"A".repeat(MAX_INPUT + 1),
            &p(json!({"start":0,"step":0})),
            false
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
}
#[test]
fn stateful_streams_reset_and_validate_definitions() {
    round!(
        stateful::gromark,
        p::<stateful::Gromark>(json!({"alphabet":AZ,"primer":"99999"})),
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    );
    round!(
        stateful::periodic_gromark,
        p::<stateful::PeriodicGromark>(json!({"keyword":"REPEATED","period":6})),
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    );
    round!(
        stateful::condi,
        p::<stateful::Condi>(json!({"alphabet":AZ,"shift":25})),
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    );
    round!(
        stateful::chaocipher,
        p::<stateful::Chaocipher>(json!({"left":AZ,"right":AZ})),
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    );
    let rag =
        p::<stateful::Ragbaby>(json!({"alphabet":"ABCDEFGHIKLMNOPQRSTUVWYZ","merge_aliases":true}));
    round!(stateful::ragbaby, rag, "CAN'T STOP-THE CLOCK! \0");
    let rag =
        p::<stateful::Ragbaby>(json!({"alphabet":"ABCDEFGHIKLMNOPQRSTUVWYZ","merge_aliases":true}));
    assert_eq!(
        stateful::ragbaby(&stateful::ragbaby("JX", &rag, false).unwrap(), &rag, true).unwrap(),
        "IW"
    );
    assert!(stateful::ragbaby("lower", &rag, false).is_err());
    assert!(stateful::gromark("A", &p(json!({"alphabet":AZ,"primer":"1234"})), false).is_err());
    assert!(
        stateful::gromark(
            "A",
            &p(json!({"alphabet":"A".repeat(26),"primer":"12345"})),
            false
        )
        .is_err()
    );
    assert!(stateful::condi("A", &p(json!({"alphabet":AZ,"shift":26})), false).is_err());
    assert!(
        stateful::periodic_gromark("A", &p(json!({"keyword":"REPEATED","period":8})), false)
            .is_err()
    );
    assert!(stateful::chaocipher("J!", &p(json!({"left":AZ,"right":AZ})), false).is_err());
}
#[test]
fn solitaire_original_keying_joker_outputs_and_work_budget() {
    let deck: Vec<u8> = (1..=54).collect();
    for (phrase, plain, expected) in [
        ("", "AAAAAAAAAA", "EXKYIZSGEH"),
        ("FOO", "AAAAAAAAAAAAAAA", "ITHZUJIWGRFARMW"),
        ("CRYPTONOMICON", "SOLITAIREX", "KIRAKSFJAN"),
    ] {
        let params = p::<stateful::Solitaire>(json!({"deck":deck,"passphrase":phrase}));
        assert_eq!(
            stateful::solitaire(plain, &params, false).unwrap(),
            expected
        );
        assert_eq!(stateful::solitaire(expected, &params, true).unwrap(), plain);
    }
    assert!(stateful::solitaire("A", &p(json!({"deck":deck,"max_steps":0})), false).is_err());
    assert_eq!(
        stateful::solitaire("", &p(json!({"deck":deck,"max_steps":0})), false).unwrap(),
        ""
    );
    let mut rotated = deck.clone();
    rotated.rotate_left(1);
    assert!(stateful::solitaire("", &p(json!({"deck":rotated,"passphrase":"A"})), false).is_err());
    let mut duplicate = deck;
    duplicate[0] = 2;
    assert!(stateful::solitaire("", &p(json!({"deck":duplicate})), false).is_err());
}
#[test]
fn slidefair_all_rectangles_and_equal_column_rules() {
    for k in 0..26 {
        let params = p::<schedules::Key>(json!({"key":char::from(b'A'+k).to_string()}));
        for a in 0..26 {
            for b in 0..26 {
                let text = format!("{}{}", char::from(b'A' + a), char::from(b'A' + b));
                round!(tableaux::slidefair, &params, &text);
            }
        }
    }
    assert!(tableaux::slidefair("A", &p(json!({"key":"B"})), false).is_err());
}
#[test]
fn portax_reciprocal_for_all_pairs_and_short_groups() {
    for k in 0..26 {
        let key = p::<schedules::Key>(json!({"key":char::from(b'A'+k).to_string()}));
        for a in 0..26 {
            for b in 0..26 {
                let text = format!("{}{}", char::from(b'A' + a), char::from(b'A' + b));
                let cipher = tableaux::portax(&text, &key).unwrap();
                assert_eq!(tableaux::portax(&cipher, &key).unwrap(), text);
            }
        }
    }
    let key = p::<schedules::Key>(json!({"key":"TEST"}));
    for text in ["", "AB", "ABCDEFGH", "ABCDEFGHIJ"] {
        assert_eq!(
            tableaux::portax(&tableaux::portax(text, &key).unwrap(), &key).unwrap(),
            text
        );
    }
    assert!(tableaux::portax("A", &key).is_err());
}
#[test]
fn nicodemus_ties_depth_and_incomplete_columns() {
    for key in ["A", "BANANA", "CAT"] {
        for depth in [1, 5, 64] {
            round!(
                tableaux::nicodemus,
                p::<tableaux::Nicodemus>(json!({"key":key,"depth":depth})),
                "ABCDEFGHIJKLMNOPQRSTUVWXYZABC"
            );
        }
    }
    assert!(tableaux::nicodemus("", &p(json!({"key":"A","depth":0})), false).is_err());
}
#[test]
fn quagmire_all_four_profiles_and_mismatch_rejection() {
    let mixed = "SPRINGFEVABCDHJKLMOQTUWXYZ";
    for (profile, a, b) in [
        ("i", mixed, AZ),
        ("ii", AZ, mixed),
        ("iii", mixed, mixed),
        ("iv", AZ, mixed),
    ] {
        round!(
            tableaux::quagmire,
            p::<tableaux::Quagmire>(
                json!({"profile":profile,"plain_alphabet":a,"cipher_alphabet":b,"indicator":"A","key":"FLOWER"})
            ),
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        );
    }
    for (profile, a, b) in [("i", AZ, mixed), ("ii", mixed, AZ), ("iii", mixed, AZ)] {
        assert!(tableaux::quagmire("",&p(json!({"profile":profile,"plain_alphabet":a,"cipher_alphabet":b,"indicator":"A","key":"FLOWER"})),false).is_err());
    }
}
#[test]
fn phillips_eight_states_and_transposed_column_profile() {
    let square = "DIAGOCBSLNEFHKMUTRQPVWXYZ";
    for profile in ["rows", "columns", "rows_columns"] {
        round!(
            tableaux::phillips,
            p::<tableaux::Phillips>(json!({"square":square,"profile":profile})),
            "ABCDEFGHIKLMNOPQRSTUVWXYZABCDEFGHIKLMNOPQRSTUVWXYZ"
        );
    }
    let rc = p::<tableaux::Phillips>(json!({"square":square,"profile":"rows_columns"}));
    let plain = "SQUARESONEANDFIVEAREACTUALLYTHESAMEASARESQUARESTWOANDEIGHTTHEOVERALLPERIODISFORTY";
    // Source's step-by-step rows resolve two typos in its final combined line.
    assert_eq!(
        tableaux::phillips(plain, &rc, false).unwrap(),
        "KZWLYRGFIRUFQARNPYGPLFXWLPPOYBRYKURKGKYRKZWLYRGVCFUFQGRVQAAQTCITYFPPVSLMEHMGUFYVO"
    );
    assert!(tableaux::phillips("J", &rc, false).is_err());
    assert!(tableaux::phillips(&"A".repeat(327681), &rc, false).is_err());
}
#[test]
fn explicit_square_geometries_reject_j_odd_and_wrong_alphabets() {
    for first in [IJ, "LOGARITHMBCDEFKNPQSUVWXYZ"] {
        let params = p::<geometry::Squares>(json!({"first":first,"second":IJ}));
        round!(geometry::two_square, params, "ABCDEFGHIKLMNOPQRSTUVWXYZA");
        let params = p::<geometry::Squares>(json!({"first":first,"second":IJ}));
        round!(geometry::four_square, params, "ABCDEFGHIKLMNOPQRSTUVWXYZA");
    }
    assert!(geometry::two_square("A", &p(json!({"first":IJ,"second":IJ})), false).is_err());
    assert!(geometry::four_square("JA", &p(json!({"first":IJ,"second":IJ})), false).is_err());
}
#[test]
fn playfair_padding_is_evidence_and_never_guessed_away() {
    let params = p::<geometry::Playfair>(json!({"square":IJ}));
    for text in ["", "A", "XX", "BALLOON", "XXXXX", "QUICK"] {
        let enc = geometry::playfair(text, &params, false).unwrap();
        let dec = geometry::playfair(&enc.text, &params, true).unwrap();
        assert_eq!(dec.text, enc.prepared);
        assert!(dec.fillers.is_empty());
        let restored: String = enc
            .prepared
            .chars()
            .enumerate()
            .filter(|(i, _)| !enc.fillers.contains(&(*i as u32)))
            .map(|(_, c)| c)
            .collect();
        assert_eq!(restored, text);
    }
    let enc = geometry::playfair("XX", &params, false).unwrap();
    assert_eq!(enc.prepared, "XQXQ");
    assert_eq!(enc.fillers, [1, 3]);
    assert!(geometry::playfair("AA", &params, true).is_err());
    assert!(geometry::playfair("A", &params, true).is_err());
    assert!(geometry::playfair("J", &params, false).is_err());
    assert!(
        geometry::playfair(
            "",
            &p(json!({"square":IJ,"filler":"X","alternate":"X"})),
            false
        )
        .is_err()
    );
}
#[test]
fn seriated_preparation_survives_short_and_adversarial_tails() {
    for period in [1, 2, 3, 6, 64, 4096] {
        let params = p::<geometry::SeriatedPlayfair>(json!({"square":IJ,"period":period}));
        for text in [
            "",
            "A",
            "XX",
            "BALLOON",
            "AAAAAAAAAAAAA",
            "ABABABABABA",
            "COMEQUICKLYWENEEDHELPIMMEDIATELYTOM",
        ] {
            let enc = geometry::seriated_playfair(text, &params, false).unwrap();
            let dec = geometry::seriated_playfair(&enc.text, &params, true).unwrap();
            assert_eq!(dec.text, enc.prepared);
            let restored: String = enc
                .prepared
                .chars()
                .enumerate()
                .filter(|(i, _)| !enc.fillers.contains(&(*i as u32)))
                .map(|(_, c)| c)
                .collect();
            assert_eq!(restored, text);
        }
    }
    assert!(geometry::seriated_playfair("", &p(json!({"square":IJ,"period":0})), false).is_err());
}
#[test]
fn trisquare_choices_and_published_inverse() {
    let params = p::<geometry::TriSquare>(json!({"first":IJ,"second":IJ,"middle":IJ}));
    for text in ["", "AA", "ABCDEFGHIKLMNOPQRSTUVWXYZA"] {
        round!(geometry::tri_square, &params, text);
    }
    assert!(geometry::tri_square("A", &params, false).is_err());
    assert!(geometry::tri_square("AA", &params, true).is_err());
    assert!(
        geometry::tri_square(
            "AB",
            &p(json!({"first":IJ,"second":IJ,"middle":IJ,"rows":[5]})),
            false
        )
        .is_err()
    );
}
#[test]
fn digrafid_periods_corner_symbols_and_explicit_padding() {
    for period in [1, 3, 4, 4096] {
        let params = p::<geometry::Digrafid>(
            json!({"horizontal":"KEYWORDABCFGHIJLMNPQSTUVXZ#","vertical":"VDPEFQRGSTHUIJWCKXAMYLNZBO#","extra":"#","period":period,"padding":"#"}),
        );
        for text in ["", "A", "##", "ABCDEFGHIJKLMNOPQRSTUVWXYZ#"] {
            let enc = geometry::digrafid(text, &params, false).unwrap();
            assert_eq!(
                geometry::digrafid(&enc.text, &params, true).unwrap().text,
                enc.prepared
            );
        }
    }
    let params = p::<geometry::Digrafid>(
        json!({"horizontal":format!("{AZ}#"),"vertical":format!("{AZ}#"),"extra":"#","period":1,"padding":null}),
    );
    assert!(geometry::digrafid("A", &params, false).is_err());
    assert!(geometry::digrafid("AA?A", &params, false).is_err());
}
#[test]
fn hill_exact_inverses_composite_moduli_negative_keys_and_unicode() {
    for alphabet in ["ABCDEFGHIJKLMNOPQRSTUVWXYZ", "Aé\0世界"] {
        let params = p::<hill::Hill>(
            json!({"alphabet":alphabet,"dimension":2,"matrix":[1,-1,0,1],"padding":null}),
        );
        let text: String = alphabet.chars().cycle().take(10).collect();
        let enc = hill::hill(&text, &params, false).unwrap();
        assert_eq!(hill::hill(&enc.text, &params, true).unwrap().text, text);
    }
    let params = p::<hill::Hill>(
        json!({"alphabet":AZ,"dimension":3,"matrix":[6,24,1,13,16,10,20,17,15],"padding":null}),
    );
    assert_eq!(hill::hill("ACT", &params, false).unwrap().text, "POH");
    assert_eq!(hill::hill("POH", &params, true).unwrap().text, "ACT");
    let params = p::<hill::Hill>(
        json!({"alphabet":AZ,"dimension":2,"matrix":[0,1,1,0],"padding":{"symbol":"X","count":1}}),
    );
    let enc = hill::hill("A", &params, false).unwrap();
    assert_eq!(enc.padding_count, 1);
    assert_eq!(hill::hill(&enc.text, &params, true).unwrap().text, "AX");
    for matrix in [vec![0, 0, 0, 0], vec![2, 0, 0, 1], vec![1, 1, 1, 1]] {
        assert!(
            hill::hill(
                "",
                &p(json!({"alphabet":AZ,"dimension":2,"matrix":matrix,"padding":null})),
                false
            )
            .is_err()
        );
    }
    assert!(
        hill::hill(
            "A",
            &p(json!({"alphabet":AZ,"dimension":2,"matrix":[1,0,0,1],"padding":null})),
            false
        )
        .is_err()
    );
    assert!(
        hill::hill(
            "",
            &p(json!({"alphabet":"AA","dimension":2,"matrix":[1,0,0,1],"padding":null})),
            false
        )
        .is_err()
    );
    let matrix: Vec<i32> = (0..64).map(|i| i32::from(i / 8 == i % 8)).collect();
    let params =
        p::<hill::Hill>(json!({"alphabet":AZ,"dimension":8,"matrix":matrix,"padding":null}));
    assert_eq!(
        hill::hill("ABCDEFGH", &params, true).unwrap().text,
        "ABCDEFGH"
    );
}
#[test]
fn parameter_transport_rejects_duplicates_nesting_types_and_unknowns() {
    for document in [
        r#"{"key":"A","key":"B"}"#,
        r#"{"key":3}"#,
        r#"{"key":"A","unknown":true}"#,
    ] {
        assert!(parse_parameters::<schedules::Key>(document).is_err());
    }
    assert!(parse_parameters::<schedules::Key>(&" ".repeat(MAX_PARAMETERS + 1)).is_err());
}

#[test]
fn every_family_rejects_one_over_its_direct_input_ceiling() {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!("vectors.json")).unwrap();
    macro_rules! over {
        ($name:literal,$ty:ty,$f:path,$limit:expr) => {{
            let case = cases
                .iter()
                .find(|c| c["function"] == concat!($name, "_encode"))
                .unwrap();
            let params: $ty = p(case["parameters"].clone());
            assert_eq!(
                $f(&"A".repeat($limit + 1), &params, false)
                    .unwrap_err()
                    .code,
                ErrorCode::ResourceLimit,
                $name
            );
        }};
    }
    over!(
        "gronsfeld",
        schedules::Gronsfeld,
        schedules::gronsfeld,
        MAX_INPUT
    );
    over!(
        "trithemius",
        schedules::Trithemius,
        schedules::trithemius,
        MAX_INPUT
    );
    over!("autokey", schedules::Autokey, schedules::autokey, MAX_INPUT);
    over!(
        "running_key",
        schedules::RunningKey,
        schedules::running_key,
        MAX_INPUT
    );
    over!("gromark", stateful::Gromark, stateful::gromark, MAX_INPUT);
    over!(
        "periodic_gromark",
        stateful::PeriodicGromark,
        stateful::periodic_gromark,
        MAX_INPUT
    );
    over!(
        "solitaire",
        stateful::Solitaire,
        stateful::solitaire,
        MAX_SYMBOLS
    );
    over!("slidefair", schedules::Key, tableaux::slidefair, MAX_INPUT);
    over!("ragbaby", stateful::Ragbaby, stateful::ragbaby, MAX_INPUT);
    over!("condi", stateful::Condi, stateful::condi, MAX_INPUT);
    over!(
        "nicodemus",
        tableaux::Nicodemus,
        tableaux::nicodemus,
        MAX_INPUT
    );
    over!(
        "quagmire",
        tableaux::Quagmire,
        tableaux::quagmire,
        MAX_INPUT
    );
    over!("phillips", tableaux::Phillips, tableaux::phillips, 327680);
    over!(
        "chaocipher",
        stateful::Chaocipher,
        stateful::chaocipher,
        MAX_INPUT
    );
    over!(
        "playfair",
        geometry::Playfair,
        geometry::playfair,
        MAX_INPUT
    );
    over!(
        "seriated_playfair",
        geometry::SeriatedPlayfair,
        geometry::seriated_playfair,
        MAX_INPUT
    );
    over!(
        "two_square",
        geometry::Squares,
        geometry::two_square,
        MAX_INPUT
    );
    over!(
        "four_square",
        geometry::Squares,
        geometry::four_square,
        MAX_INPUT
    );
    over!("hill", hill::Hill, hill::hill, MAX_SYMBOLS);
    over!(
        "digrafid",
        geometry::Digrafid,
        geometry::digrafid,
        MAX_SYMBOLS
    );
    over!(
        "tri_square",
        geometry::TriSquare,
        geometry::tri_square,
        MAX_SYMBOLS
    );
    let key = p::<schedules::Key>(json!({"key":"A"}));
    assert_eq!(
        schedules::porta(&"A".repeat(MAX_INPUT + 1), &key)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        tableaux::portax(&"A".repeat(MAX_INPUT + 1), &key)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let rag = p::<stateful::Ragbaby>(
        json!({"alphabet":"ABCDEFGHIKLMNOPQRSTUVWYZ","merge_aliases":false}),
    );
    assert!(stateful::ragbaby(&"A ".repeat(MAX_SYMBOLS), &rag, false).is_ok());
    assert_eq!(
        stateful::ragbaby(&"A ".repeat(MAX_SYMBOLS + 1), &rag, false)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn phillips_column_profile_is_exactly_the_transposed_row_profile() {
    let source = "DIAGOCBSLNEFHKMUTRQPVWXYZ";
    let transposed: String = (0..5)
        .flat_map(|c| (0..5).map(move |r| char::from(source.as_bytes()[5 * r + c])))
        .collect();
    let columns = p::<tableaux::Phillips>(json!({"square":source,"profile":"columns"}));
    let rows = p::<tableaux::Phillips>(json!({"square":transposed,"profile":"rows"}));
    let text = IJ.repeat(4);
    assert_eq!(
        tableaux::phillips(&text, &columns, false).unwrap(),
        tableaux::phillips(&text, &rows, false).unwrap()
    );
}

#[test]
fn published_alternate_profiles_and_feedback_schedules_have_independent_answers() {
    // ACA Quagmire I/II/III sheets; IV is the module's contract fixture.
    // https://www.cryptogram.org/downloads/aca.info/ciphers/QuagmireI.pdf
    // https://www.cryptogram.org/downloads/aca.info/ciphers/QuagmireII.pdf
    // https://www.cryptogram.org/downloads/aca.info/ciphers/QuagmireIII.pdf
    let spring = "SPRINGFEVABCDHJKLMOQTUWXYZ";
    for (profile, plain_alphabet, cipher_alphabet, key, plain, cipher) in [
        (
            "i",
            spring,
            AZ,
            "FLOWER",
            "THEQUAGONEISAPERIODICCIPHERWITHAKEYEDPLAINALPHABETRUNAGAINSTASTRAIGHTCIPHERALPHABET",
            "QPMGQRBUJUYIFDMPYAIFQYYJJJHJYCJLUUTPIDVWYMFSGAESDWHIZRBLIRVCFCZPELBPZYYJJJHWLJJLPUP",
        ),
        (
            "ii",
            AZ,
            spring,
            "FLOWER",
            "INTHEQUAGTWOASTRAIGHTPLAINALPHABETISRUNAGAINSTAKEYEDCIPHERALPHABET",
            "JICICOSLYKILFVCHEBDXCCORJIOEWAFMWKKTXBGWHRJIBKEDBJWZABUXWHEHUXOXCU",
        ),
        (
            "iii",
            "AUTOMBILECDFGHJKNPQRSVWXYZ",
            "AUTOMBILECDFGHJKNPQRSVWXYZ",
            "HIGHWAY",
            "THESAMEKEYEDALPHABETISUSEDFORPLAINANDCIPHERALPHABETS",
            "KRSLWMITJDVIABMRGQMTMLLIVIFUIXRHTNYONVRHHIIIRMCAOVEI",
        ),
    ] {
        let params = p::<tableaux::Quagmire>(
            json!({"profile":profile,"plain_alphabet":plain_alphabet,"cipher_alphabet":cipher_alphabet,"indicator":"A","key":key}),
        );
        assert_eq!(
            tableaux::quagmire(plain, &params, false).unwrap(),
            cipher,
            "{profile}"
        );
        assert_eq!(tableaux::quagmire(cipher, &params, true).unwrap(), plain);
    }
    // ACA Running Key exposes the two halves as explicit caller-owned material.
    let running = p::<schedules::RunningKey>(json!({"key":"THISCIPHERCANBEUSEDW"}));
    assert_eq!(
        schedules::running_key("ITHANYOFTHEPERIODICS", &running, false).unwrap(),
        "BAPSPGDMXYGPRSMIVMFO"
    );
    let plain = p::<schedules::Autokey>(json!({"seed":"B","feedback":"plaintext"}));
    let cipher = p::<schedules::Autokey>(json!({"seed":"B","feedback":"ciphertext"}));
    assert_eq!(schedules::autokey("AAAA", &plain, false).unwrap(), "BAAA");
    assert_eq!(schedules::autokey("AAAA", &cipher, false).unwrap(), "BBBB");
    // ACA Slidefair's equal-column Vigenere example.
    let slide = p::<schedules::Key>(json!({"key":"B"}));
    assert_eq!(tableaux::slidefair("DE", &slide, false).unwrap(), "EF");
    let ties = p::<tableaux::Nicodemus>(json!({"key":"BBA","depth":1}));
    assert_eq!(
        tableaux::nicodemus("ABCDEF", &ties, false).unwrap(),
        "CBCFEF"
    );
}
