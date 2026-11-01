//! Thin ABI qualification over actual Core APIs; algorithms remain in Core.
mod bindings;
use cube_core::{
    errors::{ErrorCode, OperationError},
    operations::{contracts, execution},
    pipeline::{self, recipe::Recipe},
};
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

const MAX_CODEC_INPUT: usize = 1024 * 1024;
const DIRECT_DOCUMENT_BYTES: usize = 4 * 1024 * 1024;
const DIRECT_DOCUMENT_DEPTH: u8 = 32;
const DIRECT_DOCUMENT_NODES: usize = 65536;

#[wasm_bindgen]
pub fn runtime_limits_json() -> String {
    let direct = budget();
    serde_json::json!({
        "linearMemoryBytes": env!("CUBE_WASM_LINEAR_MEMORY_BYTES").parse::<u64>().expect("build-time memory bound"),
        "direct": {
            "inputBytes": direct.input_bytes, "outputBytes": direct.output_bytes,
            "imageSampleBytes": direct.image_sample_bytes, "interpreterMemoryBytes": direct.memory_bytes,
            "interpreterInstructions": direct.instructions,
            "documentBytes": DIRECT_DOCUMENT_BYTES, "documentDepth": DIRECT_DOCUMENT_DEPTH, "documentNodes": DIRECT_DOCUMENT_NODES,
        },
        "recipe": {
            "documentBytes": pipeline::json::MAX_DOCUMENT_BYTES, "documentDepth": pipeline::json::MAX_DEPTH,
            "documentNodes": pipeline::json::MAX_NODES, "maxSteps": pipeline::MAX_STEPS,
            "valueBytes": pipeline::MAX_VALUE_BYTES, "workBytes": pipeline::MAX_WORK_BYTES,
            "captureBytes": pipeline::MAX_CAPTURE_BYTES, "parameterBytes": pipeline::MAX_PARAMETER_BYTES,
        },
    }).to_string()
}

fn budget() -> execution::Budget {
    execution::Budget {
        input_bytes: MAX_CODEC_INPUT,
        output_bytes: MAX_CODEC_INPUT,
        image_sample_bytes: MAX_CODEC_INPUT,
        memory_bytes: cube_core::esolang::brainfuck::MAX_MEMORY_BYTES,
        instructions: 1_000_000,
    }
}

fn execution_error(value: execution::Failure) -> String {
    execution_error_at(value, None)
}

fn execution_error_at(value: execution::Failure, operation: Option<&'static str>) -> String {
    let code = match value.0 {
        "unsupported_operation" => "unsupported_operation",
        "invalid_parameter" | "invalid_parameters" | "invalid_pointer" => "invalid_parameter",
        "resource_limit" | "input_limit" | "output_limit" | "numeric_overflow" => "resource_limit",
        "invalid_registration" | "invalid_response" => "runtime_failure",
        "operation_failed" => "operation_failed",
        _ => "invalid_input",
    };
    serde_json::json!({
        "code": code,
        "message": format!("Core executor rejected the invocation ({})", value.0),
        "reason": value.0,
        "operation": operation,
    })
    .to_string()
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Invocation {
    operation: String,
    version: u32,
    inputs: serde_json::Value,
    #[serde(default = "empty_parameters")]
    parameters: serde_json::Value,
}

fn empty_parameters() -> serde_json::Value {
    serde_json::json!({})
}

#[wasm_bindgen]
pub fn executors_json() -> Result<String, String> {
    let entries = execution::catalog(budget()).map_err(execution_error)?;
    let mut values = entries
        .into_iter()
        .map(|entry| {
            serde_json::json!({
                "operation": entry.operation,
                "version": entry.contract.operation_version,
                "effects": entry.effects,
                "inputs": entry.inputs,
                "outputs": entry.outputs,
                "parameters": entry.parameters,
            })
        })
        .collect::<Vec<_>>();
    for contract in contracts::document().map_err(error)?.operations {
        if bindings::supports(&contract.operation)
            && !values
                .iter()
                .any(|entry| entry["operation"] == contract.operation)
        {
            values.push(
                bindings::describe(
                    &serde_json::to_value(contract).expect("canonical contract serializes"),
                )
                .map_err(error)?,
            );
        }
    }
    Ok(contracts::canonical_value(serde_json::json!(values)).to_string())
}

#[wasm_bindgen]
pub fn execute_json(document: &str) -> Result<String, String> {
    let value = pipeline::json::parse_with_limits(
        document,
        DIRECT_DOCUMENT_BYTES,
        DIRECT_DOCUMENT_DEPTH,
        DIRECT_DOCUMENT_NODES,
    )
    .map_err(|value| value.diagnostic().to_string())?;
    let request: Invocation = serde_json::from_value(value).map_err(|_| {
        error(OperationError::new(
            ErrorCode::InvalidInput,
            "use an exact named operation invocation",
        ))
    })?;
    let entry = match execution::describe(&request.operation, budget()) {
        Ok(entry) => entry,
        Err(_) if bindings::supports(&request.operation) => {
            let contract = cube_core::operations::require(&request.operation)
                .map_err(error)?
                .contract()
                .map_err(error)?;
            if request.version != contract.operation_version {
                return Err(serde_json::json!({"code":"incompatible_version", "message":"operation version differs from the installed executor", "operation":request.operation}).to_string());
            }
            return bindings::execute(
                &request.operation,
                request.version,
                request.inputs,
                request.parameters,
                MAX_CODEC_INPUT,
            )
            .map(|value| value.to_string())
            .map_err(error);
        }
        Err(failure) => return Err(execution_error(failure)),
    };
    if request.version != entry.contract.operation_version {
        return Err(serde_json::json!({
            "code": "incompatible_version",
            "message": "operation version differs from the installed executor",
            "operation": entry.operation,
        })
        .to_string());
    }
    let output = execution::execute(
        &request.operation,
        request.version,
        &request.inputs,
        &request.parameters,
        budget(),
    )
    .map_err(|value| execution_error_at(value, Some(entry.operation)))?;
    Ok(contracts::canonical_value(output).to_string())
}

fn error(value: OperationError) -> String {
    serde_json::to_string(&value).expect("operation errors have a JSON representation")
}

fn limit(size: usize, operation: &'static str) -> Result<(), String> {
    if size > MAX_CODEC_INPUT {
        return Err(error(
            OperationError::new(ErrorCode::ResourceLimit, "SDK input exceeds one MiB")
                .operation(operation),
        ));
    }
    Ok(())
}

#[wasm_bindgen]
pub fn encode_base64(input: &[u8]) -> Result<String, String> {
    limit(input.len(), "encoding.base64.encode")?;
    Ok(cube_core::encoding::base64_encode(input))
}

#[wasm_bindgen]
pub fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    limit(input.len(), "encoding.base64.decode")?;
    cube_core::encoding::base64_decode(input).map_err(|_| {
        error(
            OperationError::new(ErrorCode::InvalidInput, "input is not canonical Base64")
                .operation("encoding.base64.decode"),
        )
    })
}

fn serialize_catalog(document: &contracts::Document) -> Result<String, String> {
    let output = serde_json::to_string_pretty(document).expect("canonical catalog is serializable");
    if output.len() > contracts::MAX_DOCUMENT_BYTES {
        return Err(error(OperationError::new(
            ErrorCode::ResourceLimit,
            "catalog exceeds its serialized budget",
        )));
    }
    Ok(output)
}

#[wasm_bindgen]
pub fn catalog_json() -> Result<String, String> {
    serialize_catalog(&contracts::document().map_err(error)?)
}
#[wasm_bindgen]
pub fn catalog_snapshot_json() -> Result<String, String> {
    let document = contracts::document().map_err(error)?;
    let json = serialize_catalog(&document)?;
    let sha256 = cube_core::hashing::sha256(json.trim().as_bytes());
    let identities = document
        .operations
        .iter()
        .map(|contract| {
            let bytes = serde_json::to_vec(contract).expect("canonical contract is serializable");
            serde_json::json!({
                "operation": contract.operation,
                "operation_version": contract.operation_version,
                "contract_sha256": cube_core::hashing::sha256(&bytes),
            })
        })
        .collect::<Vec<_>>();
    Ok(serde_json::json!({ "json": json, "sha256": sha256, "contracts": identities }).to_string())
}

/// Pipeline capabilities are independent of direct-operation bindings.
#[wasm_bindgen]
pub fn pipeline_adapters_json() -> Result<String, String> {
    let adapters = pipeline::supported_adapters()
        .into_iter()
        .map(|adapter| {
            let (input, output) = adapter
                .kinds()
                .map_err(|value| value.diagnostic().to_string())?;
            Ok(serde_json::json!({
                "operation": adapter.operation,
                "version": adapter.version,
                "input_type": input,
                "output_type": output,
                "parameters": adapter.descriptor.parameters,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(contracts::canonical_value(serde_json::Value::Array(adapters)).to_string())
}

/// Return only Core's redacted plan information; validation executes no steps.
#[wasm_bindgen]
pub fn validate_recipe_json(definition: &str) -> Result<String, String> {
    let recipe = Recipe::parse(definition).map_err(|value| value.diagnostic().to_string())?;
    let value = serde_json::to_value(recipe.info()).expect("typed plan is serializable");
    Ok(contracts::canonical_value(value).to_string())
}

#[wasm_bindgen]
pub fn run_recipe_json(
    definition: &str,
    bindings: &str,
    captures: &[u32],
) -> Result<String, String> {
    if captures.len() > pipeline::MAX_STEPS {
        return Err(error(OperationError::new(
            ErrorCode::ResourceLimit,
            "capture count exceeds the step budget",
        )));
    }
    let recipe = Recipe::parse(definition).map_err(|value| value.diagnostic().to_string())?;
    let bindings =
        pipeline::json::parse(bindings).map_err(|value| value.diagnostic().to_string())?;
    let bindings: BTreeMap<String, pipeline::Value> =
        serde_json::from_value(bindings).map_err(|_| {
            error(OperationError::new(
                ErrorCode::InvalidInput,
                "use exact typed recipe bindings",
            ))
        })?;
    let captures = captures
        .iter()
        .map(|value| *value as usize)
        .collect::<Vec<_>>();
    let result = recipe
        .execute(&bindings, &captures, None)
        .map_err(|value| value.diagnostic().to_string())?;
    let value = serde_json::to_value(result).expect("typed execution is serializable");
    Ok(contracts::canonical_value(value).to_string())
}

#[cfg(test)]
mod binding_tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn historical_native_executors_preserve_typed_error_categories() {
        for (operation, inputs, parameters, code) in [
            (
                "cipher.nomenclator.encode",
                json!({"units":vec!["A";4050]}),
                json!({"definition":{"entries":[{"alternatives":["A"],"codes":["x".repeat(256)]}]}}),
                "resource_limit",
            ),
            (
                "cipher.great_cipher.decode",
                json!({"tokens":["6"]}),
                json!({}),
                "invalid_input",
            ),
            (
                "cipher.vatsyayana",
                json!({"text":"A"}),
                json!({"pairs":[["A","B"],["B","C"]]}),
                "invalid_parameter",
            ),
        ] {
            let request =
                json!({"operation":operation,"version":1,"inputs":inputs,"parameters":parameters});
            let error: Value =
                serde_json::from_str(&execute_json(&request.to_string()).unwrap_err()).unwrap();
            assert_eq!(error["code"], code, "{operation}");
            assert_eq!(error["operation"], operation);
        }
    }

    #[test]
    fn book_cipher_executes_arbitrary_named_inputs() {
        let output = execute_json(
            &json!({
                "operation":"cipher.book.decode", "version":1,
                "inputs":{"input":"1:1,2:1"},
                "parameters":{"source":"another book", "scheme":"word_character", "index_base":1}
            })
            .to_string(),
        )
        .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&output).unwrap(),
            json!({"output":"ab"})
        );
    }
    #[test]
    fn canonical_decimal_fields_in_nested_inputs_keep_their_types() {
        let fixtures: Vec<Value> = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../crates/cube-core/src/operations/contracts/conformance.json"
        )))
        .unwrap();
        let fixture = fixtures
            .iter()
            .find(|f| f["operation"] == "investigation.profile.run")
            .unwrap();
        let mut inputs = fixture["inputs"].clone();
        inputs["profile"]["budgets"]["max_elapsed_ms"] = json!("5000");
        let request = json!({"operation":fixture["operation"],"version":fixture["operation_version"],"inputs":inputs,"parameters":fixture["parameters"]});
        let output: Value =
            serde_json::from_str(&execute_json(&request.to_string()).unwrap()).unwrap();
        assert_eq!(output, fixture["outputs"]);
    }
    #[test]
    fn every_canonical_binding_matches_independent_known_answers() {
        let fixtures: Vec<Value> = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../crates/cube-core/src/operations/contracts/conformance.json"
        )))
        .unwrap();
        let advertised: Vec<Value> = serde_json::from_str(&executors_json().unwrap()).unwrap();
        let mut failures = Vec::new();
        for fixture in fixtures {
            let op = fixture["operation"].as_str().unwrap();
            if !advertised.iter().any(|entry| entry["operation"] == op) {
                failures.push(format!("{op}: no binding"));
                continue;
            }
            let request = json!({"operation":op,"version":fixture["operation_version"],"inputs":fixture["inputs"],"parameters":fixture["parameters"]});
            match execute_json(&request.to_string()) {
                Ok(value) => {
                    let actual: Value = serde_json::from_str(&value).unwrap();
                    if actual != fixture["outputs"] {
                        failures.push(format!(
                            "{op}: actual {actual} != expected {}",
                            fixture["outputs"]
                        ));
                    }
                }
                Err(error) => failures.push(format!("{op}: {error}")),
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
}
