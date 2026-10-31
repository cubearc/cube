//! Typed interface adapters; canonical contracts and all algorithms remain in Core.
mod brainfuck;
mod crypto;
mod mono;
mod native;
mod puzzle_codes;

mod analysis;
mod fractionation;
mod poly;
mod transposition;

mod audio;
mod blockchain;
mod files;
mod image_analysis;
mod images;
mod linear;
mod numbers;
mod reasoning;
mod symbols;
mod text;
mod text_analysis;
mod video;
mod workflows;
use cube_core::{
    errors::{ErrorCode, OperationError},
    operations::{
        self,
        contracts::{self, PortContract},
    },
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

type Invoke = fn(&str, Value, Value) -> Result<Value, OperationError>;
type Supports = fn(&str) -> bool;
const MODULES: &[(Supports, Invoke)] = &[
    (mono::supports, mono::execute),
    (crypto::supports, crypto::execute),
    (brainfuck::supports, brainfuck::execute),
    (poly::supports, poly::execute),
    (transposition::supports, transposition::execute),
    (analysis::supports, analysis::execute),
    (fractionation::supports, fractionation::execute),
    (puzzle_codes::supports, puzzle_codes::execute),
    (audio::supports, audio::execute),
    (blockchain::supports, blockchain::execute),
    (files::supports, files::execute),
    (image_analysis::supports, image_analysis::execute),
    (images::supports, images::execute),
    (linear::supports, linear::execute),
    (numbers::supports, numbers::execute),
    (reasoning::supports, reasoning::execute),
    (symbols::supports, symbols::execute),
    (text::supports, text::execute),
    (text_analysis::supports, text_analysis::execute),
    (video::supports, video::execute),
    (workflows::supports, workflows::execute),
];
fn resolve(operation: &str) -> Option<Invoke> {
    MODULES
        .iter()
        .find(|(supports, _)| supports(operation))
        .map(|(_, execute)| *execute)
}
pub(super) fn supports(operation: &str) -> bool {
    resolve(operation).is_some()
}
fn field<T: DeserializeOwned>(value: &Value, name: &str) -> Result<T, OperationError> {
    serde_json::from_value(value.get(name).cloned().unwrap_or(Value::Null)).map_err(|_| {
        OperationError::new(
            ErrorCode::InvalidInput,
            "field does not match its declared type",
        )
    })
}
fn bytes(value: &Value) -> Result<Vec<u8>, OperationError> {
    cube_core::encoding::base64_decode(&field::<String>(value, "base64")?).map_err(|_| {
        OperationError::new(ErrorCode::InvalidInput, "use canonical exact Base64 bytes")
    })
}
fn kind(semantic: &str, schema: &Value) -> &'static str {
    if semantic == "file.source" || schema["properties"].get("base64").is_some() {
        "bytes"
    } else if semantic == "bytes.array" {
        "byte_array"
    } else if semantic == "image.samples" {
        "image_samples"
    } else if schema["type"] == "string" {
        "text"
    } else {
        "json"
    }
}
fn representation(port: &PortContract) -> &'static str {
    kind(&port.semantic_type, port.schema.as_value())
}
fn defaults(schema: &Value) -> Value {
    Value::Object(
        schema["properties"]
            .as_object()
            .into_iter()
            .flatten()
            .filter_map(|(name, s)| s.get("default").map(|v| (name.clone(), v.clone())))
            .collect(),
    )
}
pub(super) fn describe(contract: &Value) -> Result<Value, OperationError> {
    let invalid = || {
        OperationError::new(
            ErrorCode::InvalidInput,
            "embedded canonical metadata is invalid",
        )
    };
    let ports = |ports: &Value| -> Result<Value, OperationError> {
        ports
            .as_array()
            .ok_or_else(invalid)?
            .iter()
            .map(|p| {
                Ok((
                    p["name"].as_str().ok_or_else(invalid)?.to_owned(),
                    json!(kind(
                        p["semantic_type"].as_str().ok_or_else(invalid)?,
                        &p["schema"]
                    )),
                ))
            })
            .collect::<Result<serde_json::Map<_, _>, _>>()
            .map(Value::Object)
    };
    Ok(
        json!({"operation":contract["operation"],"version":contract["operation_version"],
        "effects":if contract["operation"]=="investigation.profile.run" {"monotonic_clock"} else {"pure"},
        "inputs":ports(&contract["inputs"] )?,"outputs":ports(&contract["outputs"] )?,"parameters":defaults(&contract["parameters"])}),
    )
}
fn invalid(code: ErrorCode, message: &'static str, op: &'static str) -> OperationError {
    OperationError::new(code, message).operation(op)
}
pub(super) fn execute(
    operation: &str,
    version: u32,
    inputs: Value,
    parameters: Value,
    max_bytes: usize,
) -> Result<Value, OperationError> {
    let identity = operations::require(operation)?.id;
    let contract = operations::require(operation)?.contract()?;
    if version != contract.operation_version {
        return Err(invalid(
            ErrorCode::InvalidParameter,
            "operation version differs from the installed contract",
            identity,
        ));
    }
    contract
        .validate_inputs(&inputs)
        .map_err(|e| e.operation(identity))?;
    contract.validate_parameters(&parameters).map_err(|_| {
        invalid(
            ErrorCode::InvalidParameter,
            "parameters do not match the operation contract",
            identity,
        )
    })?;
    let bytes =
        port_bytes(&contract.inputs, &inputs, max_bytes).map_err(|e| e.operation(identity))?;
    let param_bytes = serde_json::to_vec(&parameters)
        .map_err(|_| {
            invalid(
                ErrorCode::InvalidParameter,
                "parameter serialization failed",
                identity,
            )
        })?
        .len();
    if bytes.checked_add(param_bytes).is_none_or(|n| n > max_bytes) {
        return Err(invalid(
            ErrorCode::ResourceLimit,
            "invocation exceeds the SDK input budget",
            identity,
        ));
    }
    let mut effective = defaults(contract.parameters.as_value());
    effective
        .as_object_mut()
        .ok_or_else(|| {
            invalid(
                ErrorCode::InvalidInput,
                "invalid contract defaults",
                identity,
            )
        })?
        .extend(
            parameters
                .as_object()
                .ok_or_else(|| {
                    invalid(
                        ErrorCode::InvalidParameter,
                        "use named parameters",
                        identity,
                    )
                })?
                .clone(),
        );
    contract.validate_parameters(&effective)?;
    let inputs = native::inputs(inputs, &contract.inputs).map_err(|e| e.operation(identity))?;
    let effective = native::parameters(effective, contract.parameters.as_value())
        .map_err(|e| e.operation(identity))?;
    let output = contracts::canonical_value(
        resolve(operation).ok_or_else(|| {
            invalid(
                ErrorCode::InvalidInput,
                "operation has no installed binding",
                identity,
            )
        })?(operation, inputs, effective)
        .map_err(|e| e.operation(identity))?,
    );
    contract
        .validate_outputs(&output)
        .map_err(|e| e.operation(identity))?;
    if port_bytes(&contract.outputs, &output, max_bytes).map_err(|e| e.operation(identity))?
        > max_bytes
    {
        return Err(invalid(
            ErrorCode::ResourceLimit,
            "result exceeds the SDK output budget",
            identity,
        ));
    }
    Ok(output)
}

fn port_bytes(
    ports: &[PortContract],
    values: &Value,
    max_bytes: usize,
) -> Result<usize, OperationError> {
    let limit = || {
        OperationError::new(
            ErrorCode::ResourceLimit,
            "named data exceeds the SDK byte budget",
        )
    };
    ports.iter().try_fold(0usize, |total, port| {
        let Some(value) = values.get(&port.name) else {
            return Ok(total);
        };
        let count = match representation(port) {
            "bytes" => bytes(value)?.len(),
            "text" => value.as_str().map(str::len).unwrap_or(0),
            "byte_array" => value.as_array().map(Vec::len).unwrap_or(0),
            "image_samples" => {
                let samples = bytes(&value["samples"])?.len();
                if samples > max_bytes {
                    return Err(limit());
                }
                let mut metadata = value.clone();
                metadata["samples"] = json!(null);
                samples
                    .checked_add(serde_json::to_vec(&metadata).map_err(|_| limit())?.len())
                    .ok_or_else(limit)?
            }
            _ => serde_json::to_vec(value).map_err(|_| limit())?.len(),
        };
        total
            .checked_add(count)
            .filter(|n| *n <= max_bytes)
            .ok_or_else(limit)
    })
}
