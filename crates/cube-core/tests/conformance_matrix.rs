//! Exact catalog-to-real-result coverage, including legacy direct signatures.
#[path = "support/conformance/mod.rs"]
mod calls;
use cube_core::operations::{
    self,
    contracts::{self, Fixture},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn same(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) if a.is_f64() || b.is_f64() => {
            let a = a.as_f64().unwrap();
            let b = b.as_f64().unwrap();
            (a - b).abs() <= 1e-12 * a.abs().max(b.abs()).max(1.0)
        }
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same(a, b))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len() && a.iter().all(|(k, a)| b.get(k).is_some_and(|b| same(a, b)))
        }
        _ => actual == expected,
    }
}
fn file_info() -> Value {
    use std::{fs::OpenOptions, io::Write};
    let name = format!(
        "cube-contract-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let path = std::env::temp_dir().join(name);
    struct Remove(std::path::PathBuf);
    impl Drop for Remove {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let _remove = Remove(path.clone());
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).unwrap();
    file.write_all(&[0]).unwrap();
    json!({"output":cube_core::forensics::inspect::file_info_reader(&mut file).unwrap()})
}
#[test]
fn every_catalog_id_executes_a_real_fixture_and_matches_its_complete_contract() {
    let vectors: Vec<Value> =
        serde_json::from_str(include_str!("../src/operations/contracts/conformance.json")).unwrap();
    let mut owned: BTreeMap<_, _> = contracts::registered_fixtures()
        .unwrap()
        .into_iter()
        .collect();
    let expected: BTreeSet<_> = operations::catalog()
        .into_iter()
        .map(|d| d.id.to_owned())
        .collect();
    let mut covered = BTreeSet::new();
    let mut report = Vec::new();
    for v in vectors {
        let operation = v["operation"].as_str().unwrap();
        assert!(
            covered.insert(operation.to_owned()),
            "duplicate fixture {operation}"
        );
        let contract = operations::require(operation).unwrap().contract().unwrap();
        assert_eq!(
            v["operation_version"], contract.operation_version,
            "{operation}"
        );
        let hash = cube_core::hashing::sha256(&serde_json::to_vec(&contract).unwrap());
        assert_eq!(
            v["contract_sha256"], hash,
            "review and update the fixture contract identity for {operation}"
        );
        let inputs = v["inputs"].clone();
        let parameters = v["parameters"].clone();
        contract
            .validate_inputs(&inputs)
            .unwrap_or_else(|e| panic!("{operation} inputs: {e}"));
        contract
            .validate_parameters(&parameters)
            .unwrap_or_else(|e| panic!("{operation} parameters: {e}"));
        let actual = if let Some(fixture) = owned.remove(operation) {
            assert_eq!(
                contracts::canonical_value(fixture.inputs),
                inputs,
                "{operation} owning input"
            );
            assert_eq!(
                contracts::canonical_value(fixture.parameters),
                parameters,
                "{operation} owning parameters"
            );
            contracts::canonical_value(fixture.outputs)
        } else if operation == "file.info.inspect" {
            file_info()
        } else {
            contracts::canonical_value(
                calls::run_fixed_fixture(operation, inputs.clone(), parameters.clone())
                    .unwrap_or_else(|e| panic!("{operation}: {e}")),
            )
        };
        contract
            .validate_fixture(&Fixture {
                inputs,
                parameters,
                outputs: actual.clone(),
            })
            .unwrap_or_else(|e| panic!("{operation} result: {e}"));
        let mut comparable = actual;
        if operation == "file.info.inspect" {
            comparable["output"]["modified"] = Value::Null;
            comparable["output"]["created"] = Value::Null;
        }
        assert!(
            same(&comparable, &v["outputs"]),
            "{operation}: actual {comparable} expected {}",
            v["outputs"]
        );
        // Every contract rejects an unknown input, parameter and output field.
        for (section, mut value) in [
            ("input", v["inputs"].clone()),
            ("parameter", v["parameters"].clone()),
            ("output", v["outputs"].clone()),
        ] {
            value["__unregistered_field__"] = json!(true);
            let outcome = match section {
                "input" => contract.validate_inputs(&value),
                "parameter" => contract.validate_parameters(&value),
                _ => contract.validate_outputs(&value),
            };
            assert!(
                outcome.is_err(),
                "{operation}: unexpected {section} accepted"
            );
        }
        report.push(json!({"operation":operation,"operation_version":contract.operation_version,"contract_sha256":hash,"real_core_result":true,"input_schema":true,"parameter_schema":true,"output_schema":true,"rejects_unknown_fields":true}));
    }
    assert_eq!(
        covered, expected,
        "every implemented operation needs an executed conformance fixture"
    );
    assert!(owned.is_empty());
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("core-operation-conformance.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
