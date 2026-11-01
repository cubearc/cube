//! Independently transcribed Gajewski monosyllable meanings, not musical OCR.
use cube_core::operations::execution::{self, Budget};
use serde_json::json;
#[test]
fn gajewski_profile_executes_negative_affirmative_and_accented_glosses() {
    let p = json!({"profile":"gajewski_1902_monosyllables_v1"});
    let result = execution::execute(
        "symbol.solresol.encode",
        1,
        &json!({"terms":["non","oui","à","si (condition)"]}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(result["tokens"], json!(["do", "si", "fa", "sol"]));
    let result = execution::execute(
        "symbol.solresol.decode",
        1,
        &json!({"tokens":["d","si","r","ré","do","do"]}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(result["tokens"], json!(["d", "si", "r", "ré", "do", "do"]));
    assert_eq!(
        result["readings"][0]["alternatives"],
        json!(["non", "pas", "point", "ne", "ni"])
    );
    assert_eq!(
        result["readings"][1]["alternatives"],
        json!(["oui", "soit", "volontiers", "d'accord"])
    );
    assert_eq!(
        result["readings"][2]["alternatives"],
        json!(["et", "ainsi que"])
    );
    assert_eq!(result["readings"][5]["token_index"], 5);
    assert_eq!(
        result["provenance"]["profile_id"],
        "gajewski_1902_monosyllables_v1"
    );
}
fn parameters() -> cube_core::symbols::solresol::Parameters {
    serde_json::from_value(json!({"profile":"gajewski_1902_monosyllables_v1"})).unwrap()
}
#[test]
fn all_seven_primary_entries_and_abbreviations_keep_every_gloss() {
    use cube_core::symbols::solresol as s;
    let rows = [
        ("do", "d", vec!["non", "pas", "point", "ne", "ni"]),
        ("ré", "r", vec!["et", "ainsi que"]),
        ("mi", "m", vec!["ou", "ou bien"]),
        ("fa", "f", vec!["à", "au"]),
        ("sol", "so", vec!["si (condition)"]),
        ("la", "l", vec!["le"]),
        ("si", "s", vec!["oui", "soit", "volontiers", "d'accord"]),
    ];
    let p = parameters();
    let mut checked = 0;
    for (form, alias, glosses) in rows {
        for code in [form, alias] {
            let result = s::decode(&[code.into()], &p).unwrap();
            assert_eq!(result.tokens.0, [code]);
            assert_eq!(result.readings[0].alternatives, glosses);
            assert_eq!(result.readings[0].token_index, 0);
            assert_eq!(
                result.provenance.source_sha256,
                "c5dda1a678112e54c2de790fe3b7612ba18fef6483ee43d42f8b6f74699d6a34"
            );
        }
        for term in glosses {
            assert_eq!(s::encode(&[term.into()], &p).unwrap().tokens.0, [form]);
            checked += 1;
        }
    }
    assert_eq!(checked, 17);
}
#[test]
fn unknown_forms_accents_marks_and_nonsegmented_words_reject() {
    use cube_core::{errors::ErrorCode, symbols::solresol as s};
    let p = parameters();
    for token in [
        "re",
        "re\u{301}",
        "Ré",
        "rr",
        "réré",
        "do-ré",
        "do ré",
        "do'",
        "C4",
    ] {
        let failure = s::decode(&[token.into()], &p).unwrap_err();
        assert_eq!(failure.code, ErrorCode::InvalidInput, "{token}");
        assert_eq!(failure.operation, Some("symbol.solresol.decode"));
    }
    for term in ["NON", "si", "non oui", "oui "] {
        assert!(s::encode(&[term.into()], &p).is_err());
    }
    for p in [
        json!({}),
        json!({"profile":"gajewski_1902"}),
        json!({"profile":"gajewski_1902_monosyllables_v1","pitch":"C4"}),
    ] {
        assert!(
            execution::execute(
                "symbol.solresol.encode",
                1,
                &json!({"terms":[]}),
                &p,
                Budget::default()
            )
            .is_err()
        );
    }
}
#[test]
fn empty_repeated_inputs_and_effective_definition_hash_are_stable() {
    use cube_core::symbols::solresol as s;
    let p = parameters();
    let encoded = s::encode(&[], &p).unwrap();
    let decoded = s::decode(&[], &p).unwrap();
    assert!(encoded.tokens.0.is_empty() && decoded.readings.is_empty());
    assert_eq!(
        encoded.provenance.definition_sha256,
        decoded.provenance.definition_sha256
    );
    assert_eq!(encoded.provenance.definition_sha256.len(), 64);
    let result = s::decode(&["ré".into(), "ré".into()], &p).unwrap();
    assert_eq!(result.readings.len(), 2);
    assert_eq!(result.readings[1].token_index, 1);
    assert_eq!(result.tokens.0, ["ré", "ré"]);
}
#[test]
fn token_and_report_limits_are_real_without_truncation() {
    use cube_core::{errors::ErrorCode, symbols::solresol as s};
    let p = parameters();
    assert_eq!(
        s::encode(&vec!["le".into(); 65536], &p)
            .unwrap()
            .tokens
            .0
            .len(),
        65536
    );
    assert_eq!(
        s::encode(&vec!["le".into(); 65537], &p).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        s::decode(&vec!["do".into(); 5000], &p)
            .unwrap()
            .readings
            .len(),
        5000
    );
    assert_eq!(
        s::decode(&vec!["do".into(); 30000], &p).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        s::decode(&["x".repeat(257)], &p).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        s::decode(&["x".repeat(1048576)], &p).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}
