use super::*;
use crate::operations::contracts::Fixture;
use serde_json::{Value, json};
fn fixture<P: Serialize, O: Serialize>(inputs: Value, p: P, output: O) -> Fixture {
    Fixture {
        inputs,
        parameters: serde_json::to_value(p).expect("parameters"),
        outputs: serde_json::to_value(output).expect("report"),
    }
}
fn mismatch() -> OperationError {
    error(
        "numeric",
        ErrorCode::InvalidInput,
        "fixture",
        "known numeric result mismatch",
    )
}
pub(crate) fn prime_encode() -> Result<Fixture, OperationError> {
    let p:prime::Parameters=serde_json::from_value(json!({"profile":"indexed_primes_v1","index":{"alphabet":"ABC","first_index":1,"max_prime":5}})).expect("profile");
    let input = "ABCBA";
    let result = prime::encode(input, &p)?;
    if result.tokens.0 != ["2", "3", "5", "3", "2"] {
        return Err(mismatch());
    }
    Ok(fixture(json!({"text":input}), p, result))
}
pub(crate) fn prime_decode() -> Result<Fixture, OperationError> {
    let p:prime::Parameters=serde_json::from_value(json!({"profile":"indexed_primes_v1","index":{"alphabet":"ABC","first_index":1,"max_prime":5}})).expect("profile");
    let input = vec!["2".into(), "3".into(), "5".into(), "3".into(), "2".into()];
    let result = prime::decode(&input, &p)?;
    if result.text != "ABCBA" {
        return Err(mismatch());
    }
    Ok(fixture(json!({"tokens":input}), p, result))
}
pub(crate) fn modulo_encode() -> Result<Fixture, OperationError> {
    let p:modulo::Parameters=serde_json::from_value(json!({"profile":"affine_residue_v1","alphabet":"ABCDEFGHIJKLMNOPQRSTUVWXYZ","modulus":26,"multiplier":"5","offset":"8","token_origin":0})).expect("profile");
    let input = "AFFINECIPHER";
    let result = modulo::encode(input, &p)?;
    if result.tokens.0
        != [
            "8", "7", "7", "22", "21", "2", "18", "22", "5", "17", "2", "15",
        ]
    {
        return Err(mismatch());
    }
    Ok(fixture(json!({"text":input}), p, result))
}
pub(crate) fn modulo_decode() -> Result<Fixture, OperationError> {
    let p:modulo::Parameters=serde_json::from_value(json!({"profile":"affine_residue_v1","alphabet":"ABCDEFGHIJKLMNOPQRSTUVWXYZ","modulus":26,"multiplier":"5","offset":"8","token_origin":0})).expect("profile");
    let input = vec![
        "8".into(),
        "7".into(),
        "7".into(),
        "22".into(),
        "21".into(),
        "2".into(),
        "18".into(),
        "22".into(),
        "5".into(),
        "17".into(),
        "2".into(),
        "15".into(),
    ];
    let result = modulo::decode(&input, &p)?;
    if result.text != "AFFINECIPHER" {
        return Err(mismatch());
    }
    Ok(fixture(json!({"tokens":input}), p, result))
}
pub(crate) fn triliteral_encode() -> Result<Fixture, OperationError> {
    let p:triliteral::Parameters=serde_json::from_value(json!({"profile":"ordered_ternary_v1","alphabet":"ROUNDTABLECFGHIJKMPQSVWXYZ","trits":".-x"})).expect("profile");
    let input = "ROUZ";
    let result = triliteral::encode(input, &p)?;
    if result.tokens.0 != ["...", "..-", "..x", "xx-"] {
        return Err(mismatch());
    }
    Ok(fixture(json!({"text":input}), p, result))
}
pub(crate) fn triliteral_decode() -> Result<Fixture, OperationError> {
    let p:triliteral::Parameters=serde_json::from_value(json!({"profile":"ordered_ternary_v1","alphabet":"ROUNDTABLECFGHIJKMPQSVWXYZ","trits":".-x"})).expect("profile");
    let input = vec!["...".into(), "..-".into(), "..x".into(), "xx-".into()];
    let result = triliteral::decode(&input, &p)?;
    if result.text != "ROUZ" {
        return Err(mismatch());
    }
    Ok(fixture(json!({"tokens":input}), p, result))
}
pub(crate) fn product_encode() -> Result<Fixture, OperationError> {
    let p:product::Parameters=serde_json::from_value(json!({"profile":"prime_multiset_v1","index":{"alphabet":"AB","first_index":1,"max_prime":3},"max_factor_checks":100})).expect("profile");
    let input: Vec<String> = ["AB", "BA", "", "AAA"]
        .into_iter()
        .map(String::from)
        .collect();
    let result = product::encode(&input, &p)?;
    if result
        .groups
        .iter()
        .map(|g| g.product.as_str())
        .collect::<Vec<_>>()
        != ["6", "6", "1", "8"]
        || result.groups[1].canonical_text != "AB"
        || result.factor_checks != 0
    {
        return Err(mismatch());
    }
    Ok(fixture(json!({"groups":input}), p, result))
}
pub(crate) fn product_decode() -> Result<Fixture, OperationError> {
    let p:product::Parameters=serde_json::from_value(json!({"profile":"prime_multiset_v1","index":{"alphabet":"AB","first_index":1,"max_prime":3},"max_factor_checks":100})).expect("profile");
    let input: Vec<String> = ["6", "6", "1", "8"].into_iter().map(String::from).collect();
    let result = product::decode(&input, &p)?;
    if result
        .groups
        .iter()
        .map(|g| g.product.as_str())
        .collect::<Vec<_>>()
        != ["6", "6", "1", "8"]
        || result.groups[1].canonical_text != "AB"
        || result.factor_checks != 13
    {
        return Err(mismatch());
    }
    Ok(fixture(json!({"products":input}), p, result))
}
