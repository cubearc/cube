use cube_core::{
    encoding, image,
    operations::{self, contracts},
};
use serde_json::json;
use std::collections::BTreeSet;

#[test]
fn complete_catalog_has_concrete_resolved_contracts() {
    let document = contracts::document().unwrap();
    let actual = document
        .operations
        .iter()
        .map(|c| c.operation.as_str())
        .collect::<BTreeSet<_>>();
    let expected = operations::catalog()
        .iter()
        .map(|d| d.id)
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
    assert_eq!(actual.len(), document.operations.len());
    assert!(document.operations.iter().all(|c| !c.outputs.is_empty()));
}
#[test]
fn actual_typed_results_and_parameters_conform() {
    let image =
        image::Image::from_bytes(1, 1, image::ColorModel::Rgb, &[12, 34, 56], &[], &[]).unwrap();
    let histogram = image.histogram(None, None, 256).unwrap();
    let contract = operations::require("image.histogram")
        .unwrap()
        .contract()
        .unwrap();
    contracts::validate_value(
        &contract.outputs[0].schema,
        &serde_json::to_value(histogram).unwrap(),
    )
    .unwrap();
    assert!(
        contracts::validate_value(&contract.outputs[0].schema, &json!({"colors":"invented"}))
            .is_err()
    );
    let decoded =
        encoding::base58check::decode(&encoding::base58check::encode(0, &[0, 255]).unwrap())
            .unwrap();
    let contract = operations::require("encoding.base58check.decode")
        .unwrap()
        .contract()
        .unwrap();
    contracts::validate_value(
        &contract.outputs[0].schema,
        &serde_json::to_value(decoded).unwrap(),
    )
    .unwrap();
    let contract = operations::require("cipher.caesar.encode")
        .unwrap()
        .contract()
        .unwrap();
    assert!(contracts::validate_value(&contract.parameters, &json!({"shift":3})).is_ok());
    assert!(contracts::validate_value(&contract.parameters, &json!({"shift":"three"})).is_err());
    assert!(
        contracts::validate_value(&contract.parameters, &json!({"shift":3,"script":"execute"}))
            .is_err()
    );
}
#[test]
fn no_remote_schemas_or_implicit_capabilities() {
    let bad = schemars::Schema::try_from(json!({"$ref":"https://invalid.example/schema"})).unwrap();
    assert!(contracts::validate_schema(&bad).is_err());
    let bad = schemars::Schema::try_from(json!({"$ref":"file:///private/data"})).unwrap();
    assert!(contracts::validate_schema(&bad).is_err());
    let contract = operations::require("blockchain.private.export")
        .unwrap()
        .contract()
        .unwrap();
    assert!(contract.policy.private_output);
    assert!(!contract.policy.automatic_capture);
    assert!(!contract.policy.grants_filesystem_access);
    assert!(!contract.policy.grants_network_access);
}

#[test]
fn full_width_integers_round_trip_without_javascript_rounding() {
    let schema = contracts::schema::<u64>();
    let exact = contracts::canonical_value(json!(u64::MAX));
    assert_eq!(exact, json!("18446744073709551615"));
    contracts::validate_value(&schema, &exact).unwrap();
    assert!(contracts::validate_value(&schema, &json!("18446744073709551616")).is_err());
    assert!(contracts::validate_value(&schema, &json!("01")).is_err());
    assert!(contracts::validate_value(&schema, &json!(u64::MAX)).is_err());
    let signed = contracts::schema::<i64>();
    contracts::validate_value(&signed, &contracts::canonical_value(json!(i64::MIN))).unwrap();
    assert!(contracts::validate_value(&signed, &json!("-9223372036854775809")).is_err());
}
#[test]
fn module_owned_fixtures_execute_real_operations() {
    assert!(contracts::validate_registered_fixtures().unwrap() >= 2);
}

#[test]
fn promoted_inputs_retain_owning_parameter_bounds_and_list_constraints() {
    for (id, valid, invalid) in [
        (
            "number.binomial",
            json!({"n":128,"k":2}),
            json!({"n":129,"k":2}),
        ),
        (
            "number.permutations",
            json!({"n":3,"k":2}),
            json!({"n":3,"k":129}),
        ),
        ("number.pascal", json!({"rows":1}), json!({"rows":130})),
        (
            "number.multinomial",
            json!({"groups":[]}),
            json!({"groups":[129]}),
        ),
    ] {
        let descriptor = operations::require(id).unwrap();
        let contract = descriptor.contract().unwrap();
        contract.validate_inputs(&valid).unwrap();
        assert!(contract.validate_inputs(&invalid).is_err(), "{id}");
        for input in &contract.inputs {
            assert!(
                contract.parameters.as_value()["properties"]
                    .get(&input.name)
                    .is_none()
            );
        }
    }
}

#[test]
fn unregistered_or_modified_descriptors_cannot_publish_canonical_contracts() {
    let descriptor = operations::require("text.reverse").unwrap();
    let mut changed = *descriptor;
    changed.id = "text.invented";
    assert!(changed.contract().is_err());
    changed = *descriptor;
    changed.description = "Unregistered changed semantics";
    assert!(changed.contract().is_err());
}
