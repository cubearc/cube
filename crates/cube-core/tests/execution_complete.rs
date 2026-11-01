//! Every portable named-port executor belongs to Core, rather than an SDK host.
use cube_core::operations::{self, contracts, execution};
use serde_json::Value;
use std::collections::BTreeSet;

#[test]
fn diagnostic_interpreters_cannot_raise_the_invocation_instruction_budget() {
    let operation = "esolang.brainfuck.report";
    let budget = execution::Budget {
        instructions: 1,
        ..Default::default()
    };
    let version = operations::require(operation)
        .unwrap()
        .contract()
        .unwrap()
        .operation_version;
    let mut parameters = execution::defaults(operation, budget).unwrap();
    parameters["max_instructions"] = serde_json::json!(2);
    assert_eq!(
        execution::validate_parameters(operation, version, &parameters, budget),
        Err(execution::Failure("resource_limit"))
    );
}

#[test]
fn complete_catalog_has_named_core_executors_with_exact_reviewed_contracts() {
    let budget = execution::Budget::default();
    let executors = execution::catalog(budget).unwrap();
    let actual: BTreeSet<_> = executors.iter().map(|e| e.operation).collect();
    let expected: BTreeSet<_> = operations::catalog().iter().map(|e| e.id).collect();
    assert_eq!(actual, expected);
    assert_eq!(actual.len(), executors.len());
    for executor in executors {
        let contract = operations::require(executor.operation)
            .unwrap()
            .contract()
            .unwrap();
        assert_eq!(
            serde_json::to_value(executor.contract).unwrap(),
            serde_json::to_value(contract).unwrap()
        );
    }
}

#[test]
fn every_reviewed_known_answer_executes_through_the_same_core_boundary() {
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("../src/operations/contracts/conformance.json")).unwrap();
    assert_eq!(fixtures.len(), operations::catalog().len());
    let mut covered = BTreeSet::new();
    for fixture in fixtures {
        let operation = fixture["operation"].as_str().unwrap();
        assert!(
            covered.insert(operation.to_owned()),
            "duplicate {operation}"
        );
        let version = u32::try_from(fixture["operation_version"].as_u64().unwrap()).unwrap();
        let contract = operations::require(operation).unwrap().contract().unwrap();
        assert_eq!(version, contract.operation_version);
        assert_eq!(
            fixture["contract_sha256"],
            cube_core::hashing::sha256(&serde_json::to_vec(&contract).unwrap()),
            "{operation}"
        );
        let output = execution::execute(
            operation,
            version,
            &fixture["inputs"],
            &fixture["parameters"],
            execution::Budget::default(),
        )
        .unwrap_or_else(|failure| panic!("{operation}: {failure}"));
        assert_eq!(
            contracts::canonical_value(output),
            contracts::canonical_value(fixture["outputs"].clone()),
            "{operation}"
        );
    }
    assert_eq!(
        covered,
        operations::catalog()
            .iter()
            .map(|d| d.id.to_owned())
            .collect()
    );
}

#[test]
fn only_clock_effects_sample_the_explicit_host_clock() {
    let mut calls = 0;
    let mut clock = || {
        calls += 1;
        Ok(std::time::Duration::ZERO)
    };
    assert_eq!(
        execution::invoke_with_clock(
            "text.reverse",
            1,
            &serde_json::json!({"input":"Cube"}),
            &serde_json::json!({}),
            Default::default(),
            &mut clock
        )
        .unwrap(),
        serde_json::json!({"output":"ebuC"})
    );
    assert_eq!(calls, 0);
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("../src/operations/contracts/conformance.json")).unwrap();
    let fixture = fixtures
        .iter()
        .find(|f| f["operation"] == "investigation.profile.run")
        .unwrap();
    let mut clock = || {
        calls += 1;
        Ok(std::time::Duration::ZERO)
    };
    execution::invoke_with_clock(
        "investigation.profile.run",
        fixture["operation_version"].as_u64().unwrap() as u32,
        &fixture["inputs"],
        &fixture["parameters"],
        Default::default(),
        &mut clock,
    )
    .unwrap();
    assert!(calls > 0);
}

#[test]
fn compact_diagnostics_preserve_internal_limits_and_enforce_final_output_size() {
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("../src/operations/contracts/conformance.json")).unwrap();
    for operation in ["esolang.brainfuck.report", "esolang.brainfuck.compare"] {
        let fixture = fixtures
            .iter()
            .find(|f| f["operation"] == operation)
            .unwrap();
        let budget = execution::Budget {
            output_bytes: 16 * 1024,
            ..Default::default()
        };
        let output = execution::execute(
            operation,
            fixture["operation_version"].as_u64().unwrap() as u32,
            &fixture["inputs"],
            &fixture["parameters"],
            budget,
        )
        .unwrap();
        assert_eq!(output, fixture["outputs"]);
        let budget = execution::Budget {
            output_bytes: 1,
            ..Default::default()
        };
        assert_eq!(
            execution::execute(
                operation,
                fixture["operation_version"].as_u64().unwrap() as u32,
                &fixture["inputs"],
                &fixture["parameters"],
                budget
            ),
            Err(execution::Failure("resource_limit"))
        );
    }
}
