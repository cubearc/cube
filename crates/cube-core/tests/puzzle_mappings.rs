use cube_core::{
    ciphers::{
        bacon,
        polybius::{self, Square},
    },
    errors::ErrorCode,
    symbols::keypad::{Key, Mapping},
    text::{
        self,
        classes::{self, Class, Definition},
        keyboard::{self, Entry, Layout},
    },
};
#[test]
fn bacon_independent_alphabets_and_notations() {
    let extended = "AAAAA AAAAB AAABA AAABB AABAA AABAB AABBA AABBB ABAAA ABAAB ABABA ABABB ABBAA ABBAB ABBBA ABBBB BAAAA BAAAB BAABA BAABB BABAA BABAB BABBA BABBB BBAAA BBAAB";
    let classical = "AAAAA AAAAB AAABA AAABB AABAA AABAB AABBA AABBB ABAAA ABAAB ABABA ABABB ABBAA ABBAB ABBBA ABBBB BAAAA BAAAB BAABA BAABB BABAA BABAB BABBA BABBB";
    for (variant, alphabet, expected) in [
        ("extended26", "ABCDEFGHIJKLMNOPQRSTUVWXYZ", extended),
        ("classical24", "ABCDEFGHIKLMNOPQRSTUWXYZ", classical),
    ] {
        assert_eq!(bacon::encode(alphabet, variant, "ab").unwrap(), expected);
        assert_eq!(bacon::decode(expected, variant, "ab").unwrap(), alphabet);
        for notation in ["binary", "binary_inverted"] {
            let encoded = bacon::encode(alphabet, variant, notation).unwrap();
            assert_eq!(
                bacon::decode(&encoded, variant, notation).unwrap(),
                alphabet
            );
        }
    }
    assert_eq!(
        bacon::encode("ab", "extended26", "binary").unwrap(),
        "00000 00001"
    );
    assert_eq!(
        bacon::encode("ab", "extended26", "binary_inverted").unwrap(),
        "11111 11110"
    );
    let encoded = bacon::encode("  ij  uv  ", "classical24", "ab").unwrap();
    assert_eq!(
        bacon::decode(&encoded, "classical24", "ab").unwrap(),
        "II UU"
    );
    for variant in ["extended26", "classical24"] {
        assert_eq!(bacon::encode("", variant, "ab").unwrap(), "");
        assert_eq!(bacon::decode("", variant, "ab").unwrap(), "");
    }
}
#[test]
fn bacon_errors_and_byte_boundaries() {
    for input in ["é", "A\nB", "A\tB", "!", "Α"] {
        assert!(bacon::encode(input, "extended26", "ab").is_err());
    }
    for input in [
        "AAAA",
        "AAAAAA",
        "AAAAA  AAAAB",
        " AAAAA",
        "AAAAA ",
        "/ AAAAA",
        "AAAAA /",
        "AAAAA / / AAAAB",
        "BBBBB",
        "00000",
    ] {
        assert!(bacon::decode(input, "extended26", "ab").is_err(), "{input}");
    }
    assert!(bacon::decode("BBAAA", "classical24", "ab").is_err());
    assert!(bacon::encode("", "bad", "ab").is_err());
    assert!(bacon::decode("", "extended26", "bad").is_err());
    assert_eq!(
        bacon::encode("Aé", "extended26", "ab").unwrap_err().offset,
        Some(1)
    );
    assert!(bacon::encode(&"A".repeat(text::MAX_INPUT), "extended26", "ab").is_ok());
    assert_eq!(
        bacon::encode(&"A".repeat(text::MAX_INPUT + 1), "extended26", "ab")
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let input = "AAAAA ".repeat(text::MAX_INPUT / 6);
    let input = input.trim_end();
    assert!(bacon::decode(input, "extended26", "ab").is_ok());
}
#[test]
fn square_presets_coordinates_and_custom_roundtrips() {
    let p = Square::preset("polybius").unwrap();
    let t = Square::preset("tap").unwrap();
    assert_eq!(
        p.encode("AIJZ", 1).unwrap(),
        vec![Some([1, 1]), Some([2, 4]), Some([2, 4]), Some([5, 5])]
    );
    assert_eq!(
        t.encode("ACKJZ", 1).unwrap(),
        vec![
            Some([1, 1]),
            Some([1, 3]),
            Some([1, 3]),
            Some([2, 5]),
            Some([5, 5])
        ]
    );
    for (square, alphabet) in [
        (&p, "ABCDEFGHIKLMNOPQRSTUVWXYZ"),
        (&t, "ABCDEFGHIJLMNOPQRSTUVWXYZ"),
    ] {
        for base in [0, 1] {
            let tokens = square.encode(alphabet, base).unwrap();
            assert_eq!(square.decode(&tokens, base).unwrap(), alphabet);
        }
    }
    let custom = Square::new(2, "αβγδ", &[('a', 'α')]).unwrap();
    let tokens = custom.encode(" δβα  γ ", 0).unwrap();
    assert_eq!(custom.decode(&tokens, 0).unwrap(), " δβα  γ ");
    assert_eq!(custom.encode("a", 1).unwrap(), vec![Some([1, 1])]);
    assert_eq!(
        polybius::parse_tokens(&polybius::format_tokens(&tokens).unwrap()).unwrap(),
        tokens
    );
    assert_eq!(
        p.decode(&p.encode(" ij  ", 1).unwrap(), 1).unwrap(),
        " II  "
    );
    assert!(p.encode("", 1).unwrap().is_empty());
    assert_eq!(p.decode(&[], 0).unwrap(), "");
}
#[test]
fn square_invalid_and_limits() {
    for (side, alphabet, aliases) in [
        (0, "", vec![]),
        (65, "A", vec![]),
        (2, "abc", vec![]),
        (2, "abca", vec![]),
        (2, "ab c", vec![]),
        (1, "A", vec![('A', 'A')]),
        (1, "A", vec![('B', 'C')]),
        (1, "A", vec![('B', 'A'), ('B', 'A')]),
    ] {
        assert!(Square::new(side, alphabet, &aliases).is_err());
    }
    let square = Square::preset("polybius").unwrap();
    assert!(square.encode("A!", 1).is_err());
    assert!(square.encode("", 2).is_err());
    assert!(square.decode(&[], 2).is_err());
    for pair in [[0, 1], [1, 0], [6, 1], [1, 6], [255, 255]] {
        assert!(square.decode(&[Some(pair)], 1).is_err());
    }
    for text in [
        "1", "11", "1,2,3", "1,", "+1,2", "-1,2", "1,2 ", "1,2  /", "256,1", "1,é",
    ] {
        assert!(polybius::parse_tokens(text).is_err(), "{text}");
    }
    assert!(Square::preset("bad").is_err());
    assert_eq!(square.encode(&"A".repeat(65536), 1).unwrap().len(), 65536);
    assert!(square.encode(&"A".repeat(65537), 1).is_err());
    assert!(square.decode(&vec![None; 65537], 1).is_err());
    assert!(polybius::parse_tokens(&"/ ".repeat(65537)).is_err());
    assert!(polybius::format_tokens(&vec![None; 65537]).is_err());
    assert!(square.encode(&"A".repeat(text::MAX_INPUT + 1), 1).is_err());
    let alphabet: String = (0x1000..0x2000)
        .map(|c| char::from_u32(c).unwrap())
        .collect();
    assert!(Square::new(64, &alphabet, &[]).is_ok());
}
fn custom_layout(symbols: &str) -> Layout {
    Layout {
        geometry: "two".into(),
        entries: symbols
            .chars()
            .enumerate()
            .map(|(i, symbol)| Entry {
                key: format!("k{i}"),
                layer: "base".into(),
                symbol,
            })
            .collect(),
    }
}
#[test]
fn keyboard_presets_layers_and_geometry() {
    let q = Layout::preset("qwerty").unwrap();
    let d = Layout::preset("dvorak").unwrap();
    let z = Layout::preset("qwertz").unwrap();
    assert_eq!(q.entries.len(), 94);
    assert_eq!(d.entries.len(), 94);
    assert_eq!(
        keyboard::remap("qwerty QWERTY []{};:'\"", &q, &d, "preserve").unwrap(),
        "',.pyf \"<>PYF /=?+sS-_"
    );
    assert_eq!(keyboard::remap("yYzZ", &q, &z, "error").unwrap(), "zZyY");
    for target in [&d, &z, &q] {
        let source: String = q.entries.iter().map(|e| e.symbol).collect();
        let mapped = keyboard::remap(&source, &q, target, "error").unwrap();
        assert_eq!(
            keyboard::remap(&mapped, target, &q, "error").unwrap(),
            source
        );
    }
    let source = custom_layout("αβ");
    let target = custom_layout("🧊x");
    assert_eq!(
        keyboard::remap("α!β", &source, &target, "preserve").unwrap(),
        "🧊!x"
    );
    assert_eq!(
        keyboard::remap("α!", &source, &target, "error")
            .unwrap_err()
            .offset,
        Some(2)
    );
    assert_eq!(keyboard::remap("", &q, &d, "error").unwrap(), "");
    assert!(keyboard::remap("", &q, &d, "bad").is_err());
    let mut bad = target.clone();
    bad.geometry = "other".into();
    assert!(keyboard::remap("", &source, &bad, "error").is_err());
    bad = target.clone();
    bad.entries.pop();
    assert!(keyboard::remap("", &source, &bad, "error").is_err());
    bad = target.clone();
    bad.entries[1].symbol = bad.entries[0].symbol;
    assert!(keyboard::remap("", &source, &bad, "error").is_err());
    bad = target.clone();
    bad.entries[1].key = bad.entries[0].key.clone();
    assert!(keyboard::remap("", &source, &bad, "error").is_err());
    assert!(Layout::parse("{\"geometry\":\"a\",\"geometry\":\"b\",\"entries\":[]}").is_err());
    assert!(Layout::parse(&format!("{{{}", " ".repeat(65536))).is_err());
    assert!(Layout::preset("de").is_err());
    assert!(keyboard::remap(&"q".repeat(text::MAX_INPUT), &q, &d, "error").is_ok());
    assert!(keyboard::remap(&"q".repeat(text::MAX_INPUT + 1), &q, &d, "error").is_err());
}
#[test]
fn keypad_independent_vectors_and_ambiguity() {
    let m = Mapping::standard();
    assert_eq!(m.encode("HELLO", "telephone").unwrap(), "43556");
    assert_eq!(m.encode("hello", "multi_tap").unwrap(), "44|33|555|555|666");
    let result = m.decode("43556", "telephone").unwrap();
    assert_eq!(result.alternatives, ["GHI", "DEF", "JKL", "JKL", "MNO"]);
    assert_eq!(result.text, None);
    assert!(!result.reversible);
    assert_eq!(
        m.decode("44|33|555|555|666", "multi_tap")
            .unwrap()
            .text
            .as_deref(),
        Some("HELLO")
    );
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    assert_eq!(
        m.decode(&m.encode(alphabet, "multi_tap").unwrap(), "multi_tap")
            .unwrap()
            .text
            .as_deref(),
        Some(alphabet)
    );
    let custom = Mapping {
        ascii_fold: false,
        keys: vec![Key {
            digit: '0',
            symbols: " 0🧊".into(),
        }],
    };
    assert_eq!(custom.encode("0 🧊", "multi_tap").unwrap(), "00|0|000");
    assert_eq!(
        custom
            .decode("00|0|000", "multi_tap")
            .unwrap()
            .text
            .as_deref(),
        Some("0 🧊")
    );
    for mode in ["telephone", "multi_tap"] {
        assert_eq!(m.encode("", mode).unwrap(), "");
        assert!(m.decode("", mode).is_ok());
    }
}
#[test]
fn keypad_errors_and_bounds() {
    let m = Mapping::standard();
    for text in ["Hello world", "1", "é"] {
        assert!(m.encode(text, "telephone").is_err());
    }
    for text in ["1", "0", "44|", "|44", "44||33", "4444", "4433", "é"] {
        assert!(m.decode(text, "multi_tap").is_err(), "{text}");
    }
    for text in ["1", "4|3", "é"] {
        assert!(m.decode(text, "telephone").is_err());
    }
    assert!(m.encode("", "bad").is_err());
    assert!(m.decode("", "bad").is_err());
    assert!(m.encode(&"A".repeat(65536), "telephone").is_ok());
    assert!(m.encode(&"A".repeat(65537), "telephone").is_err());
    assert!(m.decode(&"2".repeat(65537), "telephone").is_err());
    for keys in [
        vec![],
        vec![Key {
            digit: 'a',
            symbols: "A".into(),
        }],
        vec![Key {
            digit: '2',
            symbols: "Aa".into(),
        }],
        vec![
            Key {
                digit: '2',
                symbols: "A".into(),
            },
            Key {
                digit: '2',
                symbols: "B".into(),
            },
        ],
        vec![Key {
            digit: '2',
            symbols: "".into(),
        }],
    ] {
        assert!(
            Mapping {
                ascii_fold: true,
                keys
            }
            .encode("", "telephone")
            .is_err()
        );
    }
    assert!(Mapping::parse("{\"ascii_fold\":true,\"keys\":[],\"unknown\":1}").is_err());
    assert!(Mapping::parse(&" ".repeat(65537)).is_err());
    let chars: String = (0x1000..0x1100)
        .map(|c| char::from_u32(c).unwrap())
        .collect();
    let mut custom = Mapping {
        ascii_fold: false,
        keys: vec![Key {
            digit: '0',
            symbols: chars,
        }],
    };
    assert!(custom.encode("", "telephone").is_ok());
    custom.keys[0].symbols.push('x');
    assert!(custom.encode("", "telephone").is_err());
}
#[test]
fn classes_provenance_policies_and_custom_sets() {
    assert_eq!(
        classes::map("AEIOU", None, "error").unwrap().mapped_text,
        "00000"
    );
    assert_eq!(
        classes::map("bCd", None, "error").unwrap().mapped_text,
        "111"
    );
    assert!(classes::map("Y", None, "filter").is_err());
    for (y, out) in [("vowel", "0"), ("consonant", "1"), ("unmapped", "")] {
        assert_eq!(
            classes::map("y", Some(y), "filter").unwrap().mapped_text,
            out
        );
    }
    let report = classes::map("éa!B", None, "filter").unwrap();
    assert_eq!(report.mapped_text, "01");
    assert_eq!(report.filtered_count, 2);
    assert_eq!(
        (
            report.records[0].scalar_index,
            report.records[0].byte_offset
        ),
        (1, 2)
    );
    assert_eq!(
        (
            report.records[1].scalar_index,
            report.records[1].byte_offset
        ),
        (3, 4)
    );
    assert_eq!(
        (
            report.filtered[1].scalar_index,
            report.filtered[1].byte_offset
        ),
        (2, 3)
    );
    let report = classes::map("A🧊e\u{301}", None, "preserve").unwrap();
    assert_eq!(report.mapped_text, "0🧊0\u{301}");
    assert_eq!(report.preserved.len(), 2);
    assert!(classes::map("1", None, "error").is_err());
    assert!(classes::map("", Some("bad"), "error").is_err());
    assert!(classes::map("", None, "bad").is_err());
    let definition = Definition {
        ascii_fold: false,
        classes: vec![Class {
            name: "greek".into(),
            characters: "αβ".into(),
            symbol: "🧊".into(),
        }],
    };
    assert_eq!(definition.map("βα", "error").unwrap().mapped_text, "🧊🧊");
    assert!(definition.map("Α", "error").is_err());
    assert_eq!(classes::map("", None, "error").unwrap().records.len(), 0);
}
#[test]
fn classes_validation_and_budgets() {
    let base = Class {
        name: "letter".into(),
        characters: "a".into(),
        symbol: "x".into(),
    };
    for classes in [
        vec![],
        vec![Class {
            characters: "aA".into(),
            ..base.clone()
        }],
        vec![
            base.clone(),
            Class {
                name: "other".into(),
                ..base.clone()
            },
        ],
        vec![Class {
            symbol: "".into(),
            ..base.clone()
        }],
        vec![Class {
            name: "".into(),
            ..base.clone()
        }],
        vec![Class {
            characters: "".into(),
            ..base.clone()
        }],
    ] {
        assert!(
            Definition {
                ascii_fold: true,
                classes
            }
            .map("", "error")
            .is_err()
        );
    }
    assert!(Definition::parse("{\"ascii_fold\":false,\"classes\":[],\"bad\":0}").is_err());
    assert!(Definition::parse(&" ".repeat(65537)).is_err());
    assert!(classes::map(&"A".repeat(65536), None, "error").is_ok());
    assert!(classes::map(&"A".repeat(65537), None, "error").is_err());
    assert!(classes::map(&"!".repeat(65536), None, "filter").is_ok());
    let expansion = Definition {
        ascii_fold: false,
        classes: vec![Class {
            symbol: "x".repeat(1024),
            ..base
        }],
    };
    assert!(expansion.map(&"a".repeat(8192), "error").is_err());
}

#[test]
fn mapping_definition_count_and_serialization_boundaries() {
    let entries: Vec<_> = (0..512)
        .map(|i| Entry {
            key: format!("k{i}"),
            layer: "base".into(),
            symbol: char::from_u32(0x1000 + i).unwrap(),
        })
        .collect();
    let mut layout = Layout {
        geometry: "bounded".into(),
        entries,
    };
    assert!(keyboard::remap("", &layout, &layout, "error").is_ok());
    layout.entries.push(Entry {
        key: "extra".into(),
        layer: "base".into(),
        symbol: 'x',
    });
    assert!(keyboard::remap("", &layout, &layout, "error").is_err());
    let mut layout = custom_layout("a");
    layout.entries[0].layer = "x".repeat(129);
    assert!(keyboard::remap("", &layout, &layout, "error").is_err());
    let characters: String = (0x1000..0x2000)
        .map(|c| char::from_u32(c).unwrap())
        .collect();
    let mut definition = Definition {
        ascii_fold: false,
        classes: vec![Class {
            name: "x".into(),
            characters,
            symbol: "0".into(),
        }],
    };
    assert!(definition.map("", "error").is_ok());
    definition.classes[0].characters.push('A');
    assert!(definition.map("", "error").is_err());
    definition.classes[0].characters = "A".into();
    let initial = serde_json::to_string(&definition).unwrap().len();
    definition.classes[0]
        .name
        .push_str(&"x".repeat(65536 - initial));
    assert_eq!(serde_json::to_string(&definition).unwrap().len(), 65536);
    assert!(definition.map("", "error").is_ok());
    definition.classes[0].name.push('x');
    assert!(definition.map("", "error").is_err());
    let mut mapping = Mapping {
        ascii_fold: false,
        keys: vec![Key {
            digit: '2',
            symbols: "A".into(),
        }],
    };
    mapping.keys.extend((0..10).map(|i| Key {
        digit: char::from(b'0' + i),
        symbols: format!("{i}"),
    }));
    assert!(mapping.encode("", "telephone").is_err());
    let mut duplicate = Definition {
        ascii_fold: false,
        classes: vec![
            Class {
                name: "x".into(),
                characters: "A".into(),
                symbol: "0".into(),
            },
            Class {
                name: "y".into(),
                characters: "B".into(),
                symbol: "0".into(),
            },
        ],
    };
    assert!(duplicate.map("", "error").is_err());
    duplicate.classes[1].symbol = "1".into();
    duplicate.classes[1].name = "x".into();
    assert!(duplicate.map("", "error").is_err());
    let aliases = vec![('x', 'A'); 4097];
    assert!(Square::new(1, "A", &aliases).is_err());
    assert!(Square::new(1, &"A".repeat(65537), &[]).is_err());
    assert!(polybius::parse_tokens(&"/".repeat(text::MAX_INPUT + 1)).is_err());
}

#[test]
fn class_output_symbols_reject_prefix_ambiguity() {
    let definition = Definition {
        ascii_fold: false,
        classes: vec![
            Class {
                name: "one".into(),
                characters: "a".into(),
                symbol: "0".into(),
            },
            Class {
                name: "two".into(),
                characters: "b".into(),
                symbol: "01".into(),
            },
        ],
    };
    assert!(definition.map("ab", "error").is_err());
}
