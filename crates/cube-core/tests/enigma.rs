use cube_core::{
    ciphers::enigma::{self, Parameters},
    errors::ErrorCode,
};
use serde_json::json;

fn settings() -> Parameters {
    serde_json::from_value(json!({"profile":"enigma_i","rotors":["I","II","III"],"reflector":"B","rings":[1,1,1],"windows":"AAA","plugboard":[]})).unwrap()
}

#[test]
fn known_answer_reciprocity_and_continuation() {
    let p = settings();
    let r = enigma::transform("AAAAA", &p).unwrap();
    assert_eq!(r.text, "BDZGO");
    assert_eq!(r.final_windows, "AAF");
    assert!(r.trace.is_empty());
    assert_eq!(enigma::transform(&r.text, &p).unwrap().text, "AAAAA");
    let first = enigma::transform("AA", &p).unwrap();
    let mut next = p;
    next.windows = first.final_windows;
    assert_eq!(
        first.text + &enigma::transform("AAA", &next).unwrap().text,
        r.text
    );
}

#[test]
fn hamers_independent_double_step_trace_and_ring_independence() {
    let mut p = settings();
    p.rotors.swap(0, 2);
    p.windows = "ADO".into();
    p.trace = true;
    for rings in [[1, 1, 1], [2, 26, 13]] {
        p.rings = rings;
        let r = enigma::transform("AAAAAA", &p).unwrap();
        assert_eq!(
            r.trace
                .iter()
                .map(|s| s.windows.as_str())
                .collect::<Vec<_>>(),
            ["ADP", "ADQ", "AER", "BFS", "BFT", "BFU"]
        );
        assert_eq!(r.final_windows, "BFU");
    }
}

#[test]
fn scharnhorst_message_with_naval_rotors_rings_and_plugs() {
    let p: Parameters = serde_json::from_value(json!({"profile":"m3","rotors":["III","VI","VIII"],"reflector":"B","rings":[1,8,13],"windows":"UZV","plugboard":["AN","EZ","HK","IJ","LR","MQ","OT","PV","SW","UX"]})).unwrap();
    let ciphertext =
        "YKAENZAPMSCHZBFOCUVMRMDPYCOFHADZIZMEFXTHFLOLPZLFGGBOTGOXGRETDWTJIQHLMXVJWKZUASTR";
    let plaintext =
        "STEUEREJTANAFJORDJANSTANDORTQUAAACCCVIERNEUNNEUNZWOF AHRTZWONULSMXXSCHARNHORSTHCO"
            .replace(' ', "");
    assert_eq!(enigma::transform(ciphertext, &p).unwrap().text, plaintext);
    assert_eq!(enigma::transform(&plaintext, &p).unwrap().text, ciphertext);
}

#[test]
fn strict_and_explicit_preservation_do_not_guess_or_normalize() {
    let mut p = settings();
    for input in ["a", "A A", "é", "\0"] {
        assert_eq!(
            enigma::transform(input, &p).unwrap_err().code,
            ErrorCode::InvalidInput
        );
    }
    p = serde_json::from_value(json!({"profile":"enigma_i","rotors":["I","II","III"],"reflector":"B","rings":[1,1,1],"windows":"AAA","plugboard":[],"text_mode":"preserve","trace":true})).unwrap();
    let r = enigma::transform("a éA\0a\naA", &p).unwrap();
    assert_eq!(r.text, "b éD\0z\ngO");
    assert_eq!(r.final_windows, "AAF");
    assert_eq!(
        r.trace.iter().map(|s| s.offset).collect::<Vec<_>>(),
        [0, 4, 6, 8, 9]
    );
    assert_eq!(enigma::transform(&r.text, &p).unwrap().text, "a éA\0a\naA");
}

#[test]
fn invalid_settings_are_rejected_before_processing() {
    let base = serde_json::to_value(settings()).unwrap();
    for (key, value) in [
        ("rotors", json!(["I", "I", "III"])),
        ("rotors", json!(["I", "II", "VI"])),
        ("rings", json!([0, 1, 1])),
        ("rings", json!([1, 27, 1])),
        ("windows", json!("aaA")),
        ("windows", json!("AAAA")),
        ("plugboard", json!(["AA"])),
        ("plugboard", json!(["AB", "AC"])),
        ("plugboard", json!(["ab"])),
        ("plugboard", json!(["ABC"])),
    ] {
        let mut value_map = base.clone();
        value_map[key] = value;
        let p: Parameters = serde_json::from_value(value_map).unwrap();
        let e = enigma::transform("", &p).unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidParameter, "{key}");
        assert_eq!(e.parameter, Some(key));
        assert_eq!(e.operation, Some("cipher.enigma"));
    }
    for (key, value) in [
        ("profile", json!("m4")),
        ("reflector", json!("B-thin")),
        ("rotors", json!(["I", "II", "IX"])),
        ("unknown", json!(true)),
    ] {
        let mut p = base.clone();
        p[key] = value;
        assert!(serde_json::from_value::<Parameters>(p).is_err());
    }
}

#[test]
fn exact_input_and_trace_budgets_fail_without_partial_results() {
    let mut p = settings();
    let empty = enigma::transform("", &p).unwrap();
    assert_eq!(empty.text, "");
    assert_eq!(empty.final_windows, "AAA");
    assert_eq!(
        enigma::transform(&"A".repeat(enigma::MAX_INPUT), &p)
            .unwrap()
            .text
            .len(),
        enigma::MAX_INPUT
    );
    assert_eq!(
        enigma::transform(&"A".repeat(enigma::MAX_INPUT + 1), &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    p.trace = true;
    let r = enigma::transform(&"A".repeat(enigma::MAX_TRACE), &p).unwrap();
    assert_eq!(r.trace.len(), enigma::MAX_TRACE);
    assert!(serde_json::to_vec(&r.trace).unwrap().len() <= 4 * 1024 * 1024);
    assert_eq!(
        enigma::transform(&"A".repeat(enigma::MAX_TRACE + 1), &p)
            .unwrap_err()
            .parameter,
        Some("trace")
    );
}

#[test]
fn both_reflectors_all_rotors_and_both_naval_turnovers_are_reciprocal() {
    let mut c = settings();
    c.reflector = enigma::Reflector::C;
    // Independent first-key electrical path: A -> C -> D -> F -> A -> U -> H -> P.
    assert_eq!(enigma::transform("A", &c).unwrap().text, "P");
    for reflector in ["B", "C"] {
        for rotor in ["I", "II", "III", "IV", "V", "VI", "VII", "VIII"] {
            let other = if rotor == "I" || rotor == "II" {
                ["IV", "V"]
            } else {
                ["I", "II"]
            };
            let p: Parameters = serde_json::from_value(json!({"profile":"m3","rotors":[other[0],other[1],rotor],"reflector":reflector,"rings":[26,13,2],"windows":"ZZZ","plugboard":["AZ","BY","CX","DW","EV","FU","GT","HS","IR","JQ"]})).unwrap();
            let input = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".repeat(30);
            let encrypted = enigma::transform(&input, &p).unwrap();
            assert!(
                input
                    .bytes()
                    .zip(encrypted.text.bytes())
                    .all(|(a, b)| a != b)
            );
            assert_eq!(enigma::transform(&encrypted.text, &p).unwrap().text, input);
            assert_eq!(enigma::transform(&input, &p).unwrap().text, encrypted.text);
        }
    }
    for window in ["AAM", "AAZ"] {
        let p: Parameters = serde_json::from_value(json!({"profile":"m3","rotors":["I","II","VI"],"reflector":"C","rings":[26,26,26],"windows":window,"plugboard":[]})).unwrap();
        assert_eq!(
            enigma::transform("A", &p).unwrap().final_windows,
            if window == "AAM" { "ABN" } else { "ABA" }
        );
    }
}

#[test]
fn settings_transport_is_strict_and_exactly_bounded() {
    let document = serde_json::to_string(&settings()).unwrap();
    let mut padded = document.clone();
    padded.push_str(&" ".repeat(enigma::MAX_PARAMETERS - document.len()));
    enigma::parse_parameters(&padded).unwrap();
    padded.push(' ');
    assert_eq!(
        enigma::parse_parameters(&padded).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    assert!(enigma::parse_parameters(r#"{"profile":"m3","profile":"enigma_i"}"#).is_err());
    for field in [
        "profile",
        "rotors",
        "rings",
        "windows",
        "reflector",
        "plugboard",
    ] {
        let mut p = serde_json::to_value(settings()).unwrap();
        p.as_object_mut().unwrap().remove(field);
        assert!(enigma::parse_parameters(&p.to_string()).is_err(), "{field}");
    }
    let mut p = settings();
    p.plugboard = (0..11).map(|_| "AB".to_owned()).collect();
    assert_eq!(
        enigma::transform("", &p).unwrap_err().parameter,
        Some("plugboard")
    );
}
