//! Explicit pure-data executors, registered by their owning modules.
//!
//! Catalog metadata alone never grants execution. Hosts negotiate representations,
//! enforce retention policy and OS resource isolation, then pass lower budgets.
//! Files, credentials, network and process authority are never accepted here.
mod values;
use super::contracts::Contract;
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub(crate) use values::{bytes, image, text};

/// Transport capabilities, independent of an operation's semantic port types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Representation {
    Text,
    Bytes,
    ByteArray,
    Json,
    ImageSamples,
}
/// This registry executes only pure transformations of supplied data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Effects {
    Pure,
}
/// Per-invocation ceilings; host process limits remain independently necessary.
#[derive(Clone, Copy, Debug)]
pub struct Budget {
    pub input_bytes: usize,
    pub output_bytes: usize,
    pub image_sample_bytes: usize,
    pub memory_bytes: usize,
    pub instructions: u64,
}
impl Default for Budget {
    fn default() -> Self {
        Self {
            input_bytes: 1024 * 1024,
            output_bytes: 1024 * 1024,
            image_sample_bytes: crate::image::MAX_SAMPLES,
            memory_bytes: crate::esolang::brainfuck::MAX_MEMORY_BYTES,
            instructions: crate::esolang::brainfuck::MAX_INSTRUCTIONS,
        }
    }
}
/// Stable failures contain no input contents or platform-specific paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Failure(pub &'static str);
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for Failure {}
pub type Result<T> = std::result::Result<T, Failure>;
type Executor = fn(&Value, &Value, Budget) -> Result<Value>;
type Configuration = fn(&Value, Budget) -> Result<()>;
type Preflight = fn(&Value, &Value, Budget) -> Result<()>;
/// Declaration lives beside the operation's algorithm, never in its host UI.
pub(crate) struct Entry {
    pub operation: &'static str,
    pub inputs: &'static [Representation],
    pub outputs: &'static [Representation],
    pub execute: Executor,
    pub configure: Option<Configuration>,
    pub defaults: Option<fn(&mut Value, Budget)>,
    pub preflight: Option<Preflight>,
}
impl Entry {
    pub(crate) const fn pure(
        operation: &'static str,
        inputs: &'static [Representation],
        outputs: &'static [Representation],
        execute: Executor,
    ) -> Self {
        Self {
            operation,
            inputs,
            outputs,
            execute,
            configure: None,
            defaults: None,
            preflight: None,
        }
    }
}
/// Concrete executable capabilities; contracts keep their canonical versions.
#[derive(Debug, Serialize)]
pub struct Executable {
    pub operation: &'static str,
    pub effects: Effects,
    pub inputs: BTreeMap<String, Representation>,
    pub outputs: BTreeMap<String, Representation>,
    pub parameters: Value,
    pub contract: Contract,
}
fn entry(operation: &str) -> Result<&'static Entry> {
    super::MODULES
        .iter()
        .flat_map(|m| m.executors)
        .find(|e| e.operation == operation)
        .ok_or(Failure("unsupported_operation"))
}
fn contract(operation: &str) -> Result<Contract> {
    super::require(operation)
        .map_err(|_| Failure("unsupported_operation"))?
        .contract()
        .map_err(|_| Failure("contract_mismatch"))
}
/// Discover only explicitly registered executors. No CLI reflection or fallback.
pub fn catalog(budget: Budget) -> Result<Vec<Executable>> {
    let mut result = Vec::new();
    for module in super::MODULES {
        for executor in module.executors {
            if !module
                .descriptors
                .iter()
                .any(|d| d.id == executor.operation)
                || result
                    .iter()
                    .any(|e: &Executable| e.operation == executor.operation)
            {
                return Err(Failure("invalid_registration"));
            }
            result.push(describe(executor.operation, budget)?);
        }
    }
    Ok(result)
}
/// Resolve representations and effective defaults for one installed executor.
pub fn describe(operation: &str, budget: Budget) -> Result<Executable> {
    let e = entry(operation)?;
    let c = contract(operation)?;
    let ports = |ports: &[super::contracts::PortContract], representations: &[Representation]| {
        if ports.len() != representations.len() {
            return Err(Failure("invalid_registration"));
        }
        Ok(ports
            .iter()
            .zip(representations)
            .map(|(p, r)| (p.name.clone(), *r))
            .collect())
    };
    Ok(Executable {
        operation: e.operation,
        effects: Effects::Pure,
        inputs: ports(&c.inputs, e.inputs)?,
        outputs: ports(&c.outputs, e.outputs)?,
        parameters: defaults(operation, budget)?,
        contract: c,
    })
}
/// Canonical schema defaults, specialized only by the owning module's budget policy.
pub fn defaults(operation: &str, budget: Budget) -> Result<Value> {
    let e = entry(operation)?;
    let c = contract(operation)?;
    let schema = c.parameters.as_value();
    let empty = serde_json::Map::new();
    let properties = match schema.get("properties") {
        Some(value) => value.as_object().ok_or(Failure("contract_mismatch"))?,
        None if schema["type"] == "object" => &empty,
        None => return Err(Failure("contract_mismatch")),
    };
    let mut p = Value::Object(
        properties
            .iter()
            .filter_map(|(k, s)| s.get("default").map(|v| (k.clone(), v.clone())))
            .collect(),
    );
    if let Some(defaults) = e.defaults {
        defaults(&mut p, budget);
    }
    Ok(p)
}
fn effective(operation: &str, parameters: &Value, budget: Budget) -> Result<Value> {
    let mut p = defaults(operation, budget)?;
    let supplied = parameters
        .as_object()
        .ok_or(Failure("invalid_parameters"))?;
    p.as_object_mut()
        .ok_or(Failure("invalid_registration"))?
        .extend(supplied.clone());
    Ok(p)
}
/// Validate a saved configuration without running its algorithm.
pub fn validate_parameters(
    operation: &str,
    version: u32,
    parameters: &Value,
    budget: Budget,
) -> Result<()> {
    let e = entry(operation)?;
    let c = contract(operation)?;
    if c.operation_version != version {
        return Err(Failure("contract_mismatch"));
    }
    // Host form defaults initialize controls; they cannot weaken required fields.
    c.validate_parameters(parameters)
        .map_err(|_| Failure("invalid_parameters"))?;
    let p = effective(operation, parameters, budget)?;
    c.validate_parameters(&p)
        .map_err(|_| Failure("invalid_parameters"))?;
    if let Some(configure) = e.configure {
        configure(&p, budget)?;
    }
    Ok(())
}
/// Validate named data and configuration before invocation. Image samples have a
/// separate ceiling: serialized raster JSON is not an encoded source-file size.
pub fn validate(
    operation: &str,
    version: u32,
    inputs: &Value,
    parameters: &Value,
    budget: Budget,
) -> Result<()> {
    validate_parameters(operation, version, parameters, budget)?;
    let e = entry(operation)?;
    let c = contract(operation)?;
    c.validate_inputs(inputs)
        .map_err(|_| Failure("contract_mismatch"))?;
    let p = effective(operation, parameters, budget)?;
    let size = values::size(&c.inputs, e.inputs, inputs, budget, false)?;
    let parameter_bytes = values::text_size(&p)?;
    if size
        .checked_add(parameter_bytes)
        .is_none_or(|n| n > budget.input_bytes)
    {
        return Err(Failure("input_limit"));
    }
    if let Some(preflight) = e.preflight {
        preflight(inputs, &p, budget)?;
    }
    Ok(())
}
/// Return complete named output ports; validate both sides of the real contract.
///
/// ```
/// use cube_core::operations::execution::{self, Budget};
/// use serde_json::json;
/// let result = execution::execute("text.reverse", 1,
///     &json!({"input":"Cube"}), &json!({}), Budget::default())?;
/// assert_eq!(result, json!({"output":"ebuC"}));
/// # Ok::<(), execution::Failure>(())
/// ```
pub fn execute(
    operation: &str,
    version: u32,
    inputs: &Value,
    parameters: &Value,
    budget: Budget,
) -> Result<Value> {
    validate(operation, version, inputs, parameters, budget)?;
    let e = entry(operation)?;
    let p = effective(operation, parameters, budget)?;
    let output = (e.execute)(inputs, &p, budget)?;
    let c = contract(operation)?;
    c.validate_outputs(&output)
        .map_err(|_| Failure("invalid_response"))?;
    if output_size(operation, &output)? > budget.output_bytes {
        return Err(Failure("output_limit"));
    }
    Ok(output)
}
/// Account all named outputs, including metadata and secondary byte ports.
pub fn output_size(operation: &str, output: &Value) -> Result<usize> {
    let e = entry(operation)?;
    values::size(
        &contract(operation)?.outputs,
        e.outputs,
        output,
        Budget {
            image_sample_bytes: usize::MAX,
            ..Budget::default()
        },
        true,
    )
}
/// Encode exact bytes using the contract's lossless logical representation.
pub(crate) fn byte_value(data: Vec<u8>) -> Value {
    json!(super::contracts::values::Bytes { data })
}
