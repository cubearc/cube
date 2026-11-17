//! Independent Wycheproof valid/adversarial answers through the public executor.
use cube_core::operations::execution::{self, Budget};
use serde_json::{Value, json};
fn port(v: &Value, key: &str) -> Value {
    let bytes = hex::decode(v[key].as_str().unwrap()).unwrap();
    json!({"base64":cube_core::encoding::base64_encode(&bytes)})
}
#[test]
fn wycheproof_selected_profiles_authenticate_and_refuse_adversarial_cases() {
    let cases: Vec<Value> = serde_json::from_str(include_str!(
        "../src/ciphers/modern_aead/wycheproof_vectors.json"
    ))
    .unwrap();
    let mut valid = 0;
    let mut invalid = 0;
    for v in cases {
        let result = execution::execute(
            "cipher.aead.decrypt",
            1,
            &json!({"input":port(&v,"ciphertext_hex"),"aad":port(&v,"aad_hex"),"tag":port(&v,"tag_hex")}),
            &v["parameters"],
            Budget::default(),
        );
        if v["valid"] == true {
            valid += 1;
            assert_eq!(result.unwrap(), json!({"output":port(&v,"plaintext_hex")}));
            let encrypted = execution::execute(
                "cipher.aead.encrypt",
                1,
                &json!({"input":port(&v,"plaintext_hex"),"aad":port(&v,"aad_hex")}),
                &v["parameters"],
                Budget::default(),
            )
            .unwrap();
            assert_eq!(
                encrypted,
                json!({"output":port(&v,"ciphertext_hex"),"tag":port(&v,"tag_hex")})
            );
        } else {
            invalid += 1;
            assert_eq!(result.unwrap_err().0, "invalid_input", "{}", v["tc_id"]);
        }
    }
    assert_eq!((valid, invalid), (79, 54));
}
