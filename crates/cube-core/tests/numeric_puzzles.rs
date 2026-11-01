//! Exact qualified numerical puzzle profiles; no inferred alphabets or order.
use cube_core::ciphers::mono::{
    homophonic,
    numeric::{modulo, prime, product, triliteral},
};
use cube_core::operations::execution::{self, Budget};
use serde_json::json;
fn parse<T: serde::de::DeserializeOwned>(v: serde_json::Value) -> T {
    serde_json::from_value(v).unwrap()
}

#[test]
fn sieve_matches_independent_trial_division_and_excludes_pseudoprimes() {
    let expected: Vec<String> = (2u32..=10000)
        .filter(|n| !(2..).take_while(|d| d <= &(n / d)).any(|d| n % d == 0))
        .map(|n| n.to_string())
        .collect();
    assert_eq!(expected.len(), 1229);
    let alphabet: String = (0x1000..)
        .filter_map(char::from_u32)
        .take(expected.len())
        .collect();
    let p: prime::Parameters = parse(
        json!({"profile":"indexed_primes_v1","index":{"alphabet":alphabet,"first_index":1,"max_prime":10000}}),
    );
    let table = prime::encode("", &p).unwrap().table;
    assert_eq!(
        table
            .iter()
            .map(|e| e.tokens[0].clone())
            .collect::<Vec<_>>(),
        expected
    );
    for composite in [49, 561, 1105, 1729, 2047] {
        assert!(!expected.contains(&composite.to_string()));
    }
}
#[test]
fn finite_index_bounds_are_atomic_and_support_the_complete_maximum_table() {
    let mut p: prime::Parameters = parse(
        json!({"profile":"indexed_primes_v1","index":{"alphabet":"ABC","first_index":5,"max_prime":17}}),
    );
    assert_eq!(
        prime::encode("ABC", &p).unwrap().tokens.0,
        ["11", "13", "17"]
    );
    p.index.max_prime = 16;
    assert!(prime::encode("A", &p).is_err());
    for first in [0, 4097] {
        p.index.first_index = first;
        assert!(prime::decode(&[], &p).is_err());
    }
    p.index.first_index = 4096;
    p.index.alphabet = (0x1000..).filter_map(char::from_u32).take(4096).collect();
    p.index.max_prime = 1_000_000;
    let table = prime::encode("", &p).unwrap().table;
    assert_eq!(table.len(), 4096);
    // A derived table does not weaken the public caller-table64-KiB policy.
    assert!(
        homophonic::Mapping::new(homophonic::Definition {
            ascii_uppercase: false,
            entries: table
        })
        .is_err()
    );
    p.index.max_prime = 1_000_001;
    assert!(prime::encode("", &p).is_err());
}
#[test]
fn product_checked_arithmetic_and_trial_budget_are_exact() {
    let mut p: product::Parameters = parse(
        json!({"profile":"prime_multiset_v1","index":{"alphabet":"AB","first_index":1,"max_prime":3},"max_factor_checks":13}),
    );
    let input = ["6", "6", "1", "8"].map(String::from);
    assert_eq!(product::decode(&input, &p).unwrap().factor_checks, 13);
    p.max_factor_checks = 12;
    assert_eq!(
        product::decode(&input, &p).unwrap_err().code.as_str(),
        "resource_limit"
    );
    p.max_factor_checks = 1000;
    let r = product::encode(&["A".repeat(127)], &p).unwrap();
    assert_eq!(r.groups[0].product, (1u128 << 127).to_string());
    assert_eq!(
        product::decode(&[r.groups[0].product.clone()], &p)
            .unwrap()
            .groups[0]
            .scalar_count,
        127
    );
    assert_eq!(
        product::encode(&["A".repeat(128)], &p)
            .unwrap_err()
            .code
            .as_str(),
        "resource_limit"
    );
    for token in [
        "0",
        "00",
        "01",
        "-1",
        "+1",
        " 6",
        "6.0",
        "5",
        "340282366920938463463374607431768211456",
    ] {
        assert_eq!(
            product::decode(&[token.into()], &p)
                .unwrap_err()
                .code
                .as_str(),
            "invalid_input",
            "{token}"
        );
    }
    assert!(product::encode(&["C".into()], &p).is_err());
}
#[test]
fn numeric_profiles_preserve_exact_unicode_and_report_requested_operation() {
    let p: modulo::Parameters = parse(
        json!({"profile":"affine_residue_v1","alphabet":"é\u{0000}🦀é","modulus":5,"multiplier":"18446744073709551614","offset":"18446744073709551615","token_origin":1}),
    );
    let text = "🦀é\u{0000}é";
    let encoded = modulo::encode(text, &p).unwrap();
    assert_eq!(modulo::decode(&encoded.tokens.0, &p).unwrap().text, text);
    assert_eq!(
        modulo::decode(&["01".into()], &p).unwrap_err().operation,
        Some("cipher.modulo.decode")
    );
    let p: product::Parameters = parse(
        json!({"profile":"prime_multiset_v1","index":{"alphabet":"🦀é\u{0000}","first_index":1,"max_prime":5},"max_factor_checks":100}),
    );
    let r = product::encode(&["\u{0000}é🦀é".into()], &p).unwrap();
    assert_eq!(r.groups[0].canonical_text, "🦀éé\u{0000}");
    assert_eq!(r.groups[0].scalar_count, 4);
}
#[test]
fn modulo_rejects_non_bijections_and_noncanonical_parameters() {
    let base = json!({"profile":"affine_residue_v1","alphabet":"ABCDEFGHIJKLMNOPQRSTUVWXYZ","modulus":26,"multiplier":"5","offset":"8","token_origin":0});
    for (field, value) in [
        ("multiplier", json!("4")),
        ("multiplier", json!("05")),
        ("offset", json!("18446744073709551616")),
        ("token_origin", json!(2)),
        ("modulus", json!(25)),
        ("modulus", json!(1)),
    ] {
        let mut p = base.clone();
        p[field] = value;
        let p: modulo::Parameters = parse(p);
        assert!(modulo::encode("", &p).is_err());
        assert!(modulo::decode(&[], &p).is_err());
    }
}
#[test]
fn full_triliteral_alphabet_and_strict_trit_words() {
    let mut p: triliteral::Parameters = parse(
        json!({"profile":"ordered_ternary_v1","alphabet":"ABCDEFGHIJKLMNOPQRSTUVWXYZ\u{0000}","trits":"0🦀é"}),
    );
    assert_eq!(
        triliteral::encode("A\u{0000}", &p).unwrap().tokens.0,
        ["000", "ééé"]
    );
    assert_eq!(
        triliteral::decode(&["ééé".into()], &p).unwrap().text,
        "\u{0000}"
    );
    for token in ["0", "00", "0000", "222"] {
        assert!(triliteral::decode(&[token.into()], &p).is_err());
    }
    for trits in ["00é", "01", "0123"] {
        p.trits = trits.into();
        assert!(triliteral::encode("", &p).is_err());
    }
}
#[test]
fn numeric_messages_are_bounded_before_preprocessing() {
    let p: prime::Parameters = parse(
        json!({"profile":"indexed_primes_v1","index":{"alphabet":"A","first_index":1,"max_prime":2}}),
    );
    assert_eq!(
        prime::encode(&"A".repeat(65536), &p)
            .unwrap()
            .tokens
            .0
            .len(),
        65536
    );
    assert_eq!(
        prime::encode(&"A".repeat(65537), &p)
            .unwrap_err()
            .code
            .as_str(),
        "resource_limit"
    );
    assert!(prime::decode(&vec!["2".into(); 65537], &p).is_err());
    let p: product::Parameters = parse(
        json!({"profile":"prime_multiset_v1","index":{"alphabet":"A","first_index":1,"max_prime":2},"max_factor_checks":1000000}),
    );
    assert!(product::encode(&vec!["AA".into(); 32769], &p).is_err());
}
#[test]
fn indexed_primes_generate_the_complete_requested_table() {
    let p = json!({"profile":"indexed_primes_v1","index":{"alphabet":"ABC","first_index":1,"max_prime":5}});
    let result = execution::execute(
        "cipher.prime_number.encode",
        1,
        &json!({"text":"ABCBA"}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(result["tokens"], json!(["2", "3", "5", "3", "2"]));
    assert_eq!(
        execution::execute(
            "cipher.prime_number.decode",
            1,
            &json!({"tokens":result["tokens"]}),
            &p,
            Budget::default()
        )
        .unwrap()["text"],
        "ABCBA"
    );
}
#[test]
fn prime_products_decode_multisets_without_claiming_original_order() {
    let p = json!({"profile":"prime_multiset_v1","index":{"alphabet":"AB","first_index":1,"max_prime":3},"max_factor_checks":100});
    let result = execution::execute(
        "cipher.prime_multiplication.encode",
        1,
        &json!({"groups":["AB","BA","","AAA"]}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(result["groups"][0]["product"], "6");
    assert_eq!(result["groups"][0], result["groups"][1]);
    assert_eq!(result["groups"][2]["product"], "1");
    assert_eq!(result["groups"][3]["product"], "8");
    let decoded = execution::execute(
        "cipher.prime_multiplication.decode",
        1,
        &json!({"products":["6","6","1","8"]}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(decoded["groups"], result["groups"]);
    assert_eq!(decoded["factor_checks"], 13);
}
#[test]
fn affine_residues_have_exact_inverse_and_explicit_wire_origin() {
    let p = json!({"profile":"affine_residue_v1","alphabet":"ABCDEFGHIJKLMNOPQRSTUVWXYZ","modulus":26,"multiplier":"5","offset":"8","token_origin":0});
    let result = execution::execute(
        "cipher.modulo.encode",
        1,
        &json!({"text":"AFFINECIPHER"}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(
        result["tokens"],
        json!([
            "8", "7", "7", "22", "21", "2", "18", "22", "5", "17", "2", "15"
        ])
    );
    assert_eq!(
        execution::execute(
            "cipher.modulo.decode",
            1,
            &json!({"tokens":result["tokens"]}),
            &p,
            Budget::default()
        )
        .unwrap()["text"],
        "AFFINECIPHER"
    );
}
#[test]
fn triliteral_preserves_published_trigraph_order_and_leading_zero_trits() {
    let p = json!({"profile":"ordered_ternary_v1","alphabet":"ROUNDTABLECFGHIJKMPQSVWXYZ","trits":".-x"});
    let result = execution::execute(
        "cipher.triliteral.encode",
        1,
        &json!({"text":"ROUZ"}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(result["tokens"], json!(["...", "..-", "..x", "xx-"]));
    assert_eq!(
        execution::execute(
            "cipher.triliteral.decode",
            1,
            &json!({"tokens":result["tokens"]}),
            &p,
            Budget::default()
        )
        .unwrap()["text"],
        "ROUZ"
    );
    assert!(
        execution::execute(
            "cipher.triliteral.decode",
            1,
            &json!({"tokens":["xxx"]}),
            &p,
            Budget::default()
        )
        .is_err()
    );
}
