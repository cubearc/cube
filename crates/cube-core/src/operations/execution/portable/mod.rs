//! Shared typed named-port adapters over the owning Core algorithms.
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
use crate::{
    errors::{ErrorCode, OperationError},
    operations::{
        self,
        contracts::{self, PortContract},
    },
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

type Invoke = for<'a> fn(
    &str,
    Value,
    Value,
    super::Budget,
    Option<super::Elapsed<'a>>,
) -> Result<Value, OperationError>;
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
    crate::encoding::base64_decode(&field::<String>(value, "base64")?).map_err(|_| {
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
fn default_parameters(contract: &contracts::Contract, budget: super::Budget) -> Value {
    let mut parameters = defaults(contract.parameters.as_value());
    brainfuck::defaults(&contract.operation, &mut parameters, budget);
    parameters
}
pub(super) fn describe(
    operation: &str,
    budget: super::Budget,
) -> Result<super::Executable, OperationError> {
    let descriptor = operations::require(operation)?;
    let contract = descriptor.contract()?;
    let ports = |ports: &[PortContract]| {
        ports
            .iter()
            .map(|port| {
                let kind = match representation(port) {
                    "text" => super::Representation::Text,
                    "bytes" => super::Representation::Bytes,
                    "byte_array" => super::Representation::ByteArray,
                    "image_samples" => super::Representation::ImageSamples,
                    _ => super::Representation::Json,
                };
                (port.name.clone(), kind)
            })
            .collect()
    };
    Ok(super::Executable {
        operation: descriptor.id,
        effects: workflows::effects(operation),
        inputs: ports(&contract.inputs),
        outputs: ports(&contract.outputs),
        parameters: default_parameters(&contract, budget),
        contract,
    })
}
fn invalid(code: ErrorCode, message: &'static str, op: &'static str) -> OperationError {
    OperationError::new(code, message).operation(op)
}
fn contract(operation: &str, version: u32) -> Result<contracts::Contract, OperationError> {
    let descriptor = operations::require(operation)?;
    let contract = descriptor.contract()?;
    if version != contract.operation_version {
        return Err(invalid(
            ErrorCode::InvalidParameter,
            "operation version differs from the installed contract",
            descriptor.id,
        ));
    }
    Ok(contract)
}
fn effective(
    contract: &contracts::Contract,
    parameters: &Value,
    budget: super::Budget,
) -> Result<Value, OperationError> {
    let identity = operations::require(&contract.operation)?.id;
    contract.validate_parameters(parameters).map_err(|_| {
        invalid(
            ErrorCode::InvalidParameter,
            "parameters do not match the operation contract",
            identity,
        )
    })?;
    let mut effective = default_parameters(contract, budget);
    let object = parameters
        .as_object()
        .ok_or_else(|| OperationError::new(ErrorCode::InvalidParameter, "use named parameters"))?;
    effective
        .as_object_mut()
        .ok_or_else(|| {
            OperationError::new(ErrorCode::InvalidParameter, "invalid contract defaults")
        })?
        .extend(object.clone());
    contract.validate_parameters(&effective).map_err(|_| {
        invalid(
            ErrorCode::InvalidParameter,
            "parameters do not match the operation contract",
            identity,
        )
    })?;
    let effective = native::parameters(effective, contract.parameters.as_value())?;
    brainfuck::configure(&contract.operation, &effective, budget)?;
    Ok(effective)
}
pub(super) fn configure(
    operation: &str,
    version: u32,
    parameters: &Value,
    budget: super::Budget,
) -> Result<(), OperationError> {
    let contract = contract(operation, version)?;
    let bytes = serde_json::to_vec(parameters)
        .map_err(|_| {
            OperationError::new(
                ErrorCode::InvalidParameter,
                "parameter serialization failed",
            )
        })?
        .len();
    if bytes > budget.input_bytes {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "parameters exceed the invocation budget",
        ));
    }
    effective(&contract, parameters, budget).map(|_| ())
}
fn prepare(
    operation: &str,
    version: u32,
    inputs: &Value,
    parameters: &Value,
    budget: super::Budget,
) -> Result<(contracts::Contract, Value, Value), OperationError> {
    let contract = contract(operation, version)?;
    contract.validate_inputs(inputs)?;
    let effective = effective(&contract, parameters, budget)?;
    let bytes = port_bytes(
        &contract.inputs,
        inputs,
        budget.input_bytes,
        budget.image_sample_bytes,
    )?;
    let parameters_size = serde_json::to_vec(parameters)
        .map_err(|_| {
            OperationError::new(
                ErrorCode::InvalidParameter,
                "parameter serialization failed",
            )
        })?
        .len();
    if bytes
        .checked_add(parameters_size)
        .is_none_or(|n| n > budget.input_bytes)
    {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "invocation exceeds the input budget",
        ));
    }
    let inputs = native::inputs(inputs.clone(), &contract.inputs)?;
    Ok((contract, inputs, effective))
}
pub(super) fn validate(
    operation: &str,
    version: u32,
    inputs: &Value,
    parameters: &Value,
    budget: super::Budget,
) -> Result<(), OperationError> {
    prepare(operation, version, inputs, parameters, budget).map(|_| ())
}
pub(super) fn output_size(operation: &str, output: &Value) -> Result<usize, OperationError> {
    let contract = operations::require(operation)?.contract()?;
    contract.validate_outputs(output)?;
    port_bytes(&contract.outputs, output, usize::MAX, usize::MAX)
}
pub(super) fn execute(
    operation: &str,
    version: u32,
    inputs: &Value,
    parameters: &Value,
    budget: super::Budget,
    clock: Option<super::Elapsed<'_>>,
) -> Result<Value, OperationError> {
    let (contract, inputs, effective) = prepare(operation, version, inputs, parameters, budget)?;
    let executor = resolve(operation).ok_or_else(|| {
        OperationError::new(
            ErrorCode::InvalidInput,
            "operation has no installed executor",
        )
    })?;
    let output = contracts::canonical_value(executor(operation, inputs, effective, budget, clock)?);
    contract.validate_outputs(&output)?;
    port_bytes(
        &contract.outputs,
        &output,
        budget.output_bytes,
        budget.image_sample_bytes,
    )?;
    Ok(output)
}

fn port_bytes(
    ports: &[PortContract],
    values: &Value,
    max_bytes: usize,
    image_sample_bytes: usize,
) -> Result<usize, OperationError> {
    let limit = || {
        OperationError::new(
            ErrorCode::ResourceLimit,
            "named data exceeds the invocation byte budget",
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
                if samples > max_bytes || samples > image_sample_bytes {
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
