use cube_core::{
    ciphers::mono::{
        self,
        alphabet::{Mapping, keyword_alphabet},
        checkerboard::{Alias, Definition as DigitDefinition, Profile, Table},
        grandpre,
        homophonic::{self, Counter},
        rotations,
    },
    errors::ErrorCode,
};
#[test]
fn exact_permutations_keywords_and_unmapped_collisions_are_explicit() {
    assert_eq!(
        keyword_alphabet("ABCDEFGHIJKLMNOPQRSTUVWXYZ", "BALLOON").unwrap(),
        "BALONCDEFGHIJKMPQRSTUVWXYZ"
    );
    let map = Mapping::keyword("ABCDEFGHIJKLMNOPQRSTUVWXYZ", "BALLOON").unwrap();
    assert_eq!(map.encode("ABCXYZ", "reject").unwrap(), "BALXYZ");
    assert_eq!(
        map.decode(
            &map.encode("ABCDEFGHIJKLMNOPQRSTUVWXYZ", "reject").unwrap(),
            "reject"
        )
        .unwrap(),
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    );
    let map = Mapping::new("Aé\0e\u{301}", "猫B\0\u{301}e").unwrap();
    assert_eq!(
        map.encode("Aé\0e\u{301}", "reject").unwrap(),
        "猫B\0\u{301}e"
    );
    assert_eq!(map.encode(" ! 世界", "preserve").unwrap(), " ! 世界");
    assert_eq!(map.encode("A!", "reject").unwrap_err().offset, Some(1));
    for (a, b) in [("", ""), ("AA", "AB"), ("AB", "AA"), ("A", "BC")] {
        assert!(Mapping::new(a, b).is_err());
    }
    assert!(keyword_alphabet("AB", "C").is_err());
    let map = Mapping::new("a", "x").unwrap();
    // Preserve keeps literal x unchanged; it is not an inverse outside the domain.
    assert_eq!(map.encode("ax", "preserve").unwrap(), "xx");
    assert_eq!(map.decode("xx", "preserve").unwrap(), "aa");
    assert_eq!(
        map.encode("a", "guess").unwrap_err().code,
        ErrorCode::InvalidParameter
    );
}
#[test]
fn alphabet_and_rotation_exact_boundaries_and_all_ascii_scalars() {
    let alphabet: String = (0..4096)
        .map(|i| char::from_u32(0x1000 + i).unwrap())
        .collect();
    assert!(Mapping::new(&alphabet, &alphabet).is_ok());
    assert_eq!(
        Mapping::new(&(alphabet.clone() + "A"), &(alphabet + "A"))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        keyword_alphabet("A", &"A".repeat(mono::MAX_DEFINITION - 1)).unwrap(),
        "A"
    );
    assert_eq!(
        keyword_alphabet("A", &"A".repeat(mono::MAX_DEFINITION))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let map = Mapping::new("a", "🦀").unwrap();
    assert_eq!(
        map.encode(&"a".repeat(mono::MAX_INPUT), "reject")
            .unwrap()
            .len(),
        mono::MAX_OUTPUT
    );
    assert_eq!(
        map.encode(&"a".repeat(mono::MAX_INPUT + 1), "reject")
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        rotations::rot5("0123456789 Aa!世界").unwrap(),
        "5678901234 Aa!世界"
    );
    assert_eq!(
        rotations::rot18("Hello 123!世界").unwrap(),
        "Uryyb 678!世界"
    );
    assert_eq!(rotations::rot47("Hello, World!").unwrap(), "w6==@[ (@C=5P");
    assert_eq!(
        rotations::rot47("\0 !P~\u{7f}猫").unwrap(),
        "\0 P!O\u{7f}猫"
    );
    assert_eq!(
        rotations::rot5(" /0495:ABaz\0\u{7f}猫").unwrap(),
        " /5940:ABaz\0\u{7f}猫"
    );
    let all: String = (0..=127)
        .map(char::from)
        .chain("é猫🦀\u{301}".chars())
        .collect();
    for function in [rotations::rot5, rotations::rot18, rotations::rot47] {
        assert_eq!(function("").unwrap(), "");
        assert_eq!(function(&function(&all).unwrap()).unwrap(), all);
        assert_eq!(function(&all).unwrap().len(), all.len());
        assert_eq!(
            function(&"A".repeat(mono::MAX_INPUT)).unwrap().len(),
            mono::MAX_INPUT
        );
        assert_eq!(
            function(&"A".repeat(mono::MAX_INPUT + 1)).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
    }
    assert_eq!(
        rotations::rot18(&all).unwrap(),
        rotations::rot5(&cube_core::ciphers::rot13(&all)).unwrap()
    );
}
fn homophones() -> homophonic::Mapping {
    homophonic::Mapping::parse(r#"{"ascii_uppercase":true,"entries":[{"symbol":"A","tokens":["1","22"]},{"symbol":"B","tokens":["3"]}]}"#).unwrap()
}
#[test]
fn homophonic_round_robin_is_local_and_transport_is_unambiguous() {
    let map = homophones();
    let expected = vec!["1", "22", "3", "1"];
    assert_eq!(map.encode("aabA", &[]).unwrap(), expected);
    assert_eq!(map.encode("aabA", &[]).unwrap(), expected);
    assert_eq!(
        map.decode(&expected.into_iter().map(str::to_owned).collect::<Vec<_>>())
            .unwrap(),
        "AABA"
    );
    assert_eq!(
        map.encode(
            "AAA",
            &[Counter {
                symbol: 'A',
                index: 65535
            }]
        )
        .unwrap(),
        ["22", "1", "22"]
    );
    assert!(
        map.encode(
            "A",
            &[Counter {
                symbol: 'X',
                index: 0
            }]
        )
        .is_err()
    );
    assert!(
        map.encode(
            "A",
            &[
                Counter {
                    symbol: 'A',
                    index: 0
                },
                Counter {
                    symbol: 'A',
                    index: 1
                }
            ]
        )
        .is_err()
    );
    assert!(map.decode(&["122".into()]).is_err());
    assert!(map.encode("a ", &[]).is_err());
    for bad in [
        r#"{"ascii_uppercase":true,"ascii_uppercase":false,"entries":[]}"#,
        r#"{"ascii_uppercase":true,"entries":[{"symbol":"a","tokens":["x"]}]}"#,
        r#"{"ascii_uppercase":false,"entries":[{"symbol":"A","tokens":["x"]},{"symbol":"B","tokens":["x"]}]}"#,
    ] {
        assert!(homophonic::Mapping::parse(bad).is_err());
    }
    assert_eq!(
        map.encode(&"A".repeat(mono::MAX_TOKENS), &[])
            .unwrap()
            .len(),
        mono::MAX_TOKENS
    );
    assert_eq!(
        map.encode(&"A".repeat(mono::MAX_TOKENS + 1), &[])
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        map.decode(&vec!["1".into(); mono::MAX_TOKENS + 1])
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let escaped = homophonic::Mapping::new(homophonic::Definition {
        ascii_uppercase: false,
        entries: vec![homophonic::Entry {
            symbol: 'A',
            tokens: vec!["\0".repeat(256)],
        }],
    })
    .unwrap();
    assert_eq!(
        escaped.encode(&"A".repeat(1000), &[]).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}
fn definition(digits: &str, prefixes: &str, alphabet: &str) -> DigitDefinition {
    DigitDefinition {
        digits: digits.into(),
        prefixes: prefixes.into(),
        alphabet: alphabet.into(),
        aliases: vec![],
        digit_escape: None,
    }
}
#[test]
fn digit_profiles_match_aca_and_retain_tridigital_ambiguity() {
    let mut d = definition("6318927054", "63", "NOTARIESBCDFGHKLMPQUVWXY");
    d.aliases = vec![Alias { from: 'J', to: 'I' }, Alias { from: 'Z', to: 'Y' }];
    let mono = Table::new(Profile::MonomeDinome, d).unwrap();
    assert_eq!(mono.encode("highf").unwrap(), "600676062");
    assert_eq!(mono.decode("600676062").unwrap(), "HIGHF");
    assert_eq!(mono.decode(&mono.encode("jz").unwrap()).unwrap(), "IY");
    for bad in ["6", "60 0", "66", "600a"] {
        assert!(mono.decode(bad).is_err());
    }
    let tri = Table::new(
        Profile::Tridigital,
        definition("6703528149", "", "DRAGONFLYBCEHIJKMPQSTUVWXZ"),
    )
    .unwrap();
    assert_eq!(tri.encode("the ides").unwrap(), "03095607");
    assert_eq!(
        tri.alternatives("09").unwrap(),
        vec![vec!['A', 'E', 'T'], vec![' ']]
    );
    assert!(tri.decode("0").is_err());
    assert!(tri.encode("a\tb").is_err());
    let mut d = definition("0123456789", "37", "ABCDEFGHIJKLMNOPQRSTUVWXYZ #");
    d.digit_escape = Some('#');
    let table = Table::new(Profile::Straddling, d).unwrap();
    assert_eq!(
        table.decode(&table.encode("ABC 120XYZ").unwrap()).unwrap(),
        "ABC 120XYZ"
    );
    assert!(table.encode("#").is_err());
    assert!(Table::new(Profile::Straddling, definition("0000000000", "37", "ABC")).is_err());
    assert!(Table::new(Profile::Straddling, definition("0123456789", "33", "ABC")).is_err());
    assert!(Table::new(Profile::Tridigital, definition("0123456789", "", "ABC")).is_err());
    assert_eq!(
        table.encode(&"A".repeat(mono::MAX_TOKENS)).unwrap().len(),
        mono::MAX_TOKENS
    );
    assert_eq!(
        table
            .encode(&"A".repeat(mono::MAX_TOKENS + 1))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
#[test]
fn grandpre_labels_repeated_cells_and_published_decode_are_exact() {
    let def = grandpre::Definition {
        rows: [
            "LADYBUGS", "AZIMUTHS", "CALFSKIN", "QUACKISH", "UNJOVIAL", "EVULSION", "ROWDYISM",
            "SEXTUPLY",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        row_labels: "12345678".into(),
        column_labels: "12345678".into(),
    };
    let square = grandpre::Square::new(def).unwrap();
    assert_eq!(
        square
            .decode(&[Some([8, 4]), Some([2, 7]), Some([8, 2])])
            .unwrap(),
        "THE"
    );
    assert_eq!(
        square.encode("THE").unwrap(),
        [Some([2, 6]), Some([2, 7]), Some([6, 1])]
    );
    let tokens = square.encode("the  keyword").unwrap();
    assert_eq!(square.decode(&tokens).unwrap(), "THE  KEYWORD");
    assert_eq!(square.encode("AA").unwrap(), [Some([1, 2]), Some([2, 1])]);
    assert!(square.decode(&[Some([0, 0])]).is_err());
    assert!(square.encode("é").is_err());
    let all = grandpre::Square::new(grandpre::Definition {
        rows: vec!["A".repeat(10); 10],
        row_labels: "1234567890".into(),
        column_labels: "1234567890".into(),
    })
    .unwrap();
    assert_eq!(all.encode(&"A".repeat(100)).unwrap()[99], Some([0, 0]));
    assert_eq!(all.decode(&[Some([0, 0]), None]).unwrap(), "A ");
    assert!(all.encode("B").is_err());
    assert_eq!(
        all.encode(&"A".repeat(mono::MAX_TOKENS)).unwrap().len(),
        mono::MAX_TOKENS
    );
    assert_eq!(
        all.encode(&"A".repeat(mono::MAX_TOKENS + 1))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert!(
        grandpre::Square::new(grandpre::Definition {
            rows: vec!["A".repeat(7); 7],
            row_labels: "1234567".into(),
            column_labels: "1234567".into()
        })
        .is_err()
    );
}

#[test]
fn aca_checkerboard_reuses_explicit_homophonic_tables_without_a_new_algorithm() {
    // ACA Checkerboard simple profile: caller supplies the complete table.
    let entries = ["KNIGH", "PQRST", "OYZUA", "MXWVB", "LFEDC"]
        .into_iter()
        .zip("BLACK".chars())
        .flat_map(|(row, label)| {
            row.chars()
                .zip("WHITE".chars())
                .map(move |(symbol, column)| homophonic::Entry {
                    symbol,
                    tokens: vec![format!("{label}{column}")],
                })
        })
        .collect();
    let map = homophonic::Mapping::new(homophonic::Definition {
        ascii_uppercase: true,
        entries,
    })
    .unwrap();
    let encoded = map.encode("NUMBERS", &[]).unwrap();
    assert_eq!(encoded, ["BH", "AT", "CW", "CE", "KI", "LI", "LT"]);
    assert_eq!(map.decode(&encoded).unwrap(), "NUMBERS");
    // Word/syllable source strings are a different, deliberately unsupported grammar.
    assert!(
        homophonic::Mapping::parse(
            r#"{"ascii_uppercase":false,"entries":[{"symbol":"OR","tokens":["10"]}]}"#
        )
        .is_err()
    );
}

#[test]
fn homophonic_definition_token_ownership_and_serialized_output_have_adjacent_limits() {
    let entries: Vec<_> = (b'A'..=b'P')
        .map(|b| homophonic::Entry {
            symbol: b as char,
            tokens: (0..256).map(|i| format!("{}-{i:03}", b as char)).collect(),
        })
        .collect();
    assert!(
        homophonic::Mapping::new(homophonic::Definition {
            ascii_uppercase: false,
            entries: entries.clone()
        })
        .is_ok()
    );
    let mut extra = entries;
    extra.push(homophonic::Entry {
        symbol: 'Q',
        tokens: vec!["extra".into()],
    });
    assert_eq!(
        homophonic::Mapping::new(homophonic::Definition {
            ascii_uppercase: false,
            entries: extra
        })
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
    let definition = r#"{"ascii_uppercase":false,"entries":[{"symbol":"A","tokens":["x"]}]}"#;
    let exact = definition.to_owned() + &" ".repeat(mono::MAX_DEFINITION - definition.len());
    assert!(homophonic::Mapping::parse(&exact).is_ok());
    assert_eq!(
        homophonic::Mapping::parse(&(exact + " ")).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let map = homophonic::Mapping::new(homophonic::Definition {
        ascii_uppercase: false,
        entries: vec![homophonic::Entry {
            symbol: 'A',
            tokens: vec!["x".repeat(22)],
        }],
    })
    .unwrap();
    // ["<22 ASCII bytes>",...] has 25*n+1 bytes: n=41943 fits exactly 1 MiB.
    let tokens = map.encode(&"A".repeat(41943), &[]).unwrap();
    assert_eq!(serde_json::to_vec(&tokens).unwrap().len(), mono::MAX_INPUT);
    assert_eq!(map.decode(&tokens).unwrap().len(), 41943);
    assert_eq!(
        map.encode(&"A".repeat(41944), &[]).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let mut over = tokens;
    over.push("x".repeat(22));
    assert_eq!(
        map.decode(&over).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}
