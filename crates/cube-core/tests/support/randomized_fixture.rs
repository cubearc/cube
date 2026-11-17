//! Test-only assertions for randomized cryptographic outputs.
use cube_core::operations::{
    self,
    execution::{self, Budget},
};
use serde_json::{Value, json};

pub fn verify(case: &Value, actual: &Value) -> &'static str {
    let operation = case["operation"].as_str().unwrap();
    assert!(
        !operations::require(operation).unwrap().deterministic,
        "relational assertions are only for explicitly randomized operations"
    );
    let v = &case["verification"];
    match v["kind"].as_str() {
        Some("inverse") => {
            assert_inverse(case, actual);
            "verified-inverse"
        }
        Some("kem_key_pair" | "kem_encapsulation") => {
            assert_kem(case, actual);
            "verified-kem"
        }
        _ => panic!("randomized operations require a reviewed assertion"),
    }
}

fn assert_inverse(case: &Value, actual: &Value) {
    let operation = case["operation"].as_str().unwrap();
    let v = &case["verification"];
    let inverse = v["operation"].as_str().unwrap();
    assert_ne!(operation, inverse);
    let contract = operations::require(inverse).unwrap().contract().unwrap();
    assert!(operations::require(inverse).unwrap().deterministic);
    let output = v["output_port"].as_str().unwrap();
    let input = v["input_port"].as_str().unwrap();
    assert_eq!(actual.as_object().unwrap().len(), 1);
    assert_eq!(
        v["expected_outputs"],
        json!({output:case["inputs"][input]}),
        "inverse must recover the original known input"
    );
    for result in [actual, &case["outputs"]] {
        let value = result.get(output).expect("randomized result port missing");
        let mut inputs = v["inputs"].clone();
        inputs[input] = value.clone();
        contract.validate_inputs(&inputs).unwrap();
        let verified = execution::execute(
            inverse,
            v["operation_version"].as_u64().unwrap() as u32,
            &inputs,
            &v["parameters"],
            Budget::default(),
        )
        .expect("randomized ciphertext failed its reviewed inverse");
        assert_eq!(
            verified, v["expected_outputs"],
            "randomized output did not recover the independent expected plaintext"
        );
    }
}

fn assert_kem(case: &Value, actual: &Value) {
    let verification = &case["verification"];
    let kind = verification["kind"].as_str().unwrap();
    let suffix = match kind {
        "kem_key_pair" => ".keygen",
        _ => ".encapsulate",
    };
    let operation = case["operation"].as_str().unwrap();
    let family = operation
        .strip_suffix(suffix)
        .expect("KEM assertion matches its operation");
    let contract = operations::require(operation).unwrap().contract().unwrap();
    for output in [actual, &case["outputs"]] {
        contract
            .validate_outputs(output)
            .expect("KEM outputs match their declared ports");
        let (private_key, envelope) = if kind == "kem_key_pair" {
            let envelope = invoke(
                &format!("{family}.encapsulate"),
                &json!({"public_key":output["public_key"]}),
                &case["parameters"],
            );
            (&output["private_key"], envelope)
        } else {
            (&verification["private_key"], output.clone())
        };
        let recovered = invoke(
            &format!("{family}.decapsulate"),
            &json!({"private_key":private_key,"ciphertext":envelope["ciphertext"]}),
            &case["parameters"],
        );
        assert!(
            recovered == json!({"shared_secret":envelope["shared_secret"]}),
            "KEM peers derived different shared secrets"
        );
    }
}

fn invoke(operation: &str, inputs: &Value, parameters: &Value) -> Value {
    execution::execute(operation, 1, inputs, parameters, Budget::default())
        .expect("KEM conformance peer invocation failed")
}
