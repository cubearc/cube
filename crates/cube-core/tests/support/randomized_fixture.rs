//! Test-only inverse assertions for randomized encryption; never a runtime API.
use cube_core::operations::{
    self,
    execution::{self, Budget},
};
use serde_json::{Value, json};

pub fn assert_inverse(case: &Value, actual: &Value) {
    let operation = case["operation"].as_str().unwrap();
    assert!(
        !operations::require(operation).unwrap().deterministic,
        "inverse assertions are only for explicitly randomized operations"
    );
    let v = &case["verification"];
    assert_eq!(
        v["kind"], "inverse",
        "randomized operations require a reviewed inverse assertion"
    );
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
