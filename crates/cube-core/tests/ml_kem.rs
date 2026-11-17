use cube_core::{
    encoding,
    operations::execution::{self, Budget},
};
use serde_json::{Value, json};

fn bytes(value: &Value) -> Value {
    json!({"base64": encoding::base64_encode(&hex::decode(value.as_str().unwrap()).unwrap())})
}

#[test]
fn acvp_ciphertext_and_implicit_rejection_return_exact_shared_keys() {
    let vectors: Value =
        serde_json::from_str(include_str!("../src/asymmetric/ml_kem/vectors.json"))
            .expect("reviewed ACVP vectors");
    for profile in vectors["profiles"].as_array().unwrap() {
        for vector in profile["decapsulation"].as_array().unwrap() {
            let output = execution::execute(
                "crypto.ml_kem.decapsulate",
                1,
                &json!({"ciphertext": bytes(&vector["c"]), "private_key": bytes(&vector["dk"])}),
                &json!({"profile": profile["profile"]}),
                Budget::default(),
            )
            .expect("correct-width ACVP ciphertexts always produce shared bytes");
            assert!(
                output == json!({"shared_secret": bytes(&vector["k"])}),
                "ACVP case {}",
                vector["tcId"]
            );
        }
    }
}

#[test]
fn key_generation_honors_the_invocation_memory_and_work_budget() {
    let parameters = json!({"profile": "ml_kem_512_fips203"});
    let memory_bytes = 1024 * 1024;
    let budget = Budget {
        memory_bytes,
        instructions: 1,
        ..Budget::default()
    };
    let run =
        |budget| execution::execute("crypto.ml_kem.keygen", 1, &json!({}), &parameters, budget);
    let keys = run(budget).unwrap();
    assert_eq!(bytes_len(&keys["public_key"]), 800);
    assert_eq!(bytes_len(&keys["private_key"]), 1632);
    assert!(matches!(
        run(Budget {
            memory_bytes: memory_bytes - 1,
            ..budget
        }),
        Err(execution::Failure("resource_limit"))
    ));
    assert!(matches!(
        run(Budget {
            instructions: 0,
            ..budget
        }),
        Err(execution::Failure("resource_limit"))
    ));
}

fn bytes_len(value: &Value) -> usize {
    encoding::base64_decode(value["base64"].as_str().unwrap())
        .unwrap()
        .len()
}
