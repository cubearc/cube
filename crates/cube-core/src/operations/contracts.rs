//! Canonical invocation contracts derived from owning declarations and Rust data
//! types. Legacy discovery remains unchanged; contracts never dispatch operations.
mod parameters;
mod types;
pub mod values;
mod wire;
pub use wire::canonical as canonical_value;

use super::{Descriptor, ParameterValue};
use crate::errors::{ErrorCode, OperationError};
use schemars::{JsonSchema, Schema};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub const FORMAT_VERSION: u32 = 1;
pub const MAX_DOCUMENT_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_INSTANCE_BYTES: usize = 64 * 1024 * 1024;

/// Schema callback for module-owned typed declarations, not operation execution.
pub fn schema<T: JsonSchema>() -> Schema {
    wire::schema(schemars::schema_for!(T))
}
/// Reuse an owning descriptor's parameter declarations in a richer module entry.
pub(crate) fn descriptor_parameters(descriptor: &Descriptor) -> Schema {
    parameters::schema(descriptor).expect("registered parameter declarations are valid")
}
#[derive(Clone, Copy)]
pub struct Port {
    pub name: &'static str,
    pub semantic_type: &'static str,
    pub schema: fn() -> Schema,
    pub required: bool,
}
/// Optional implementation-enforced ceilings. None means no separate ceiling is
/// declared here, not that the API is unbounded. Owning parsers and callers still
/// enforce their actual limits.
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct Limits {
    pub max_input_bytes: Option<u64>,
    pub max_output_bytes: Option<u64>,
    pub max_items: Option<u64>,
    pub max_work: Option<u64>,
    pub max_memory_bytes: Option<u64>,
}
/// A known-answer invocation whose outputs are produced by the actual operation.
#[derive(Debug, Serialize)]
pub struct Fixture {
    pub inputs: Value,
    pub parameters: Value,
    pub outputs: Value,
}
/// An owning module can supply richer named ports/typed parameters without
/// changing public Descriptor literals or adding another algorithm registry.
#[derive(Clone, Copy)]
pub(crate) struct Entry {
    pub operation_version: u32,
    pub descriptor: Descriptor,
    pub inputs: &'static [Port],
    pub parameters: fn() -> Schema,
    pub outputs: &'static [Port],
    pub limits: Limits,
    pub fixture: fn() -> Result<Fixture, OperationError>,
}
#[derive(Clone, Debug, Serialize)]
pub struct PortContract {
    pub name: String,
    pub semantic_type: String,
    pub required: bool,
    pub schema: Schema,
}
#[derive(Clone, Debug, Serialize)]
pub struct Policy {
    pub private_input: bool,
    pub private_output: bool,
    pub sensitive_parameters: Vec<String>,
    pub automatic_capture: bool,
    pub grants_filesystem_access: bool,
    pub grants_network_access: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct Contract {
    pub schema_version: u32,
    pub operation_version: u32,
    pub operation: String,
    pub category: String,
    pub description: String,
    pub example: String,
    pub deterministic: bool,
    pub reversible: Option<bool>,
    pub pipeline_adapter: Option<u32>,
    pub inputs: Vec<PortContract>,
    pub parameters: Schema,
    pub outputs: Vec<PortContract>,
    pub errors: Schema,
    pub limits: Limits,
    pub policy: Policy,
}
#[derive(Serialize)]
pub struct Document {
    pub format_version: u32,
    pub engine_version: &'static str,
    pub json_schema_dialect: &'static str,
    pub operations: Vec<Contract>,
}
fn invalid(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message).operation("ops.contracts")
}
fn entry(descriptor: &Descriptor) -> Option<&'static Entry> {
    super::MODULES
        .iter()
        .flat_map(|module| module.contracts)
        .find(|entry| entry.descriptor == *descriptor)
}
fn port(value: &Port) -> PortContract {
    PortContract {
        name: value.name.into(),
        semantic_type: value.semantic_type.into(),
        required: value.required,
        schema: wire::schema((value.schema)()),
    }
}
/// Return the complete contract for an exact registered descriptor. Unknown
/// semantic types fail rather than publishing name-only or permissive schemas.
pub fn describe(descriptor: &Descriptor) -> Result<Contract, OperationError> {
    if !super::catalog().contains(&descriptor) {
        return Err(invalid(
            "canonical contracts require an exact registered descriptor",
        ));
    }
    let (mut inputs, parameters, outputs, limits) = if let Some(entry) = entry(descriptor) {
        (
            entry.inputs.iter().map(port).collect(),
            (entry.parameters)(),
            entry.outputs.iter().map(port).collect(),
            entry.limits,
        )
    } else {
        (
            types::ports(descriptor.input_type, "input")?,
            parameters::schema(descriptor)?,
            types::ports(descriptor.output_type, "output")?,
            Limits::default(),
        )
    };
    let mut parameters_value = parameters.to_value();
    if entry(descriptor).is_none() {
        for input in &mut inputs {
            if let Some(properties) = parameters_value
                .get_mut("properties")
                .and_then(Value::as_object_mut)
                && let Some(schema) = properties.remove(&input.name)
            {
                // Promoting a real argument to a named port must preserve
                // its owning declaration's bounds, choices and defaults.
                input.schema = wire::schema(
                    Schema::try_from(schema)
                        .map_err(|_| invalid("invalid promoted input schema"))?,
                );
            }
            if let Some(required) = parameters_value
                .get_mut("required")
                .and_then(Value::as_array_mut)
            {
                required.retain(|name| name.as_str() != Some(&input.name));
            }
        }
    }
    let parameters = wire::schema(
        Schema::try_from(parameters_value).map_err(|_| invalid("invalid parameter schema"))?,
    );
    let metadata = descriptor.metadata();
    let contract = Contract {
        schema_version: FORMAT_VERSION,
        operation_version: entry(descriptor).map_or(1, |entry| entry.operation_version),
        operation: descriptor.id.into(),
        category: descriptor.category.into(),
        description: descriptor.description.into(),
        example: descriptor.example.into(),
        deterministic: descriptor.deterministic,
        reversible: descriptor.reversible,
        pipeline_adapter: crate::pipeline::adapter_version(descriptor.id),
        inputs,
        parameters,
        outputs,
        errors: schema::<crate::errors::OperationError>(),
        limits,
        policy: Policy {
            private_input: metadata.is_some_and(|m| m.security.private_input),
            private_output: metadata.is_some_and(|m| m.security.private_output),
            sensitive_parameters: metadata
                .map(|m| {
                    m.security
                        .sensitive_parameters
                        .iter()
                        .map(|s| (*s).into())
                        .collect()
                })
                .unwrap_or_default(),
            automatic_capture: false,
            grants_filesystem_access: false,
            grants_network_access: false,
        },
    };
    if contract.operation_version == 0 {
        return Err(invalid("operation versions must be positive"));
    }
    for ports in [&contract.inputs, &contract.outputs] {
        let mut names = BTreeSet::new();
        if ports.len() > 32 {
            return Err(invalid("contract exceeds 32 named ports"));
        }
        for port in ports {
            if port.name.is_empty() || !names.insert(&port.name) {
                return Err(invalid("contract port names must be unique and nonempty"));
            }
            validate_schema(&port.schema)?;
        }
    }
    validate_schema(&contract.parameters)?;
    Ok(contract)
}
/// Deterministic catalog-wide export. Pending algorithms never acquire contracts
/// merely because an issue or descriptor file exists outside module registration.
pub fn document() -> Result<Document, OperationError> {
    let mut descriptors = super::catalog();
    descriptors.sort_by_key(|d| d.id);
    let operations = descriptors
        .into_iter()
        .map(describe)
        .collect::<Result<_, _>>()?;
    Ok(Document {
        format_version: FORMAT_VERSION,
        engine_version: env!("CARGO_PKG_VERSION"),
        json_schema_dialect: "https://json-schema.org/draft/2020-12/schema",
        operations,
    })
}
/// The library is compiled with HTTP/file resolvers disabled. Reject nonlocal
/// references additionally, before a validator or UI can interpret a schema.
pub fn validate_schema(schema: &Schema) -> Result<(), OperationError> {
    fn walk(value: &Value, depth: usize, nodes: &mut usize) -> Result<(), OperationError> {
        *nodes += 1;
        if depth > 64 || *nodes > 32768 {
            return Err(invalid("schema exceeds traversal budget"));
        }
        match value {
            Value::Object(fields) => {
                if fields
                    .get("$ref")
                    .and_then(Value::as_str)
                    .is_some_and(|r| !r.starts_with('#'))
                {
                    return Err(invalid("contract schema references must be local"));
                }
                for child in fields.values() {
                    walk(child, depth + 1, nodes)?;
                }
            }
            Value::Array(items) => {
                for child in items {
                    walk(child, depth + 1, nodes)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    walk(schema.as_value(), 0, &mut 0)?;
    if schema.as_value().is_boolean() {
        return Err(invalid("a semantic contract requires a concrete schema"));
    }
    wire::validator(schema)?;
    Ok(())
}
pub fn validate_value(schema: &Schema, value: &Value) -> Result<(), OperationError> {
    validate_schema(schema)?;
    struct Budget(usize);
    impl std::io::Write for Budget {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(bytes.len())
                .filter(|n| *n <= MAX_INSTANCE_BYTES)
                .ok_or_else(|| std::io::Error::other("contract serialization budget exceeded"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(&mut Budget(0), value).map_err(|_| {
        OperationError::new(
            ErrorCode::ResourceLimit,
            "contract serialization exceeds 64 MiB",
        )
    })?;
    let validator = wire::validator(schema)?;
    validator
        .validate(value)
        .map_err(|_| invalid("value does not match the operation contract"))
}
fn object_schema(ports: &[PortContract]) -> Schema {
    let properties = ports
        .iter()
        .map(|p| (p.name.clone(), p.schema.as_value().clone()))
        .collect::<serde_json::Map<_, _>>();
    // Each port keeps a standalone definition scope. Embed as an external-free
    // allOf schema with its definitions rebased to this port's nested location.
    let mut properties = properties;
    for (name, value) in &mut properties {
        rebase_references(value, &format!("#/properties/{name}"));
    }
    Schema::try_from(json!({"type":"object","properties":properties,"required":ports.iter().filter(|p|p.required).map(|p|&p.name).collect::<Vec<_>>(),"additionalProperties":false})).expect("object schema")
}
fn rebase_references(value: &mut Value, base: &str) {
    match value {
        Value::Object(fields) => {
            for (name, value) in fields {
                if name == "$ref" {
                    if let Some(reference) = value.as_str().and_then(|r| r.strip_prefix('#')) {
                        *value = json!(format!("{base}{reference}"));
                    }
                } else {
                    rebase_references(value, base);
                }
            }
        }
        Value::Array(items) => {
            for value in items {
                rebase_references(value, base);
            }
        }
        _ => (),
    }
}
impl Contract {
    /// Validate named wire inputs without executing or granting capabilities.
    pub fn validate_inputs(&self, value: &Value) -> Result<(), OperationError> {
        validate_value(&object_schema(&self.inputs), value)
    }
    pub fn validate_parameters(&self, value: &Value) -> Result<(), OperationError> {
        validate_value(&self.parameters, value)
    }
    pub fn validate_outputs(&self, value: &Value) -> Result<(), OperationError> {
        validate_value(&object_schema(&self.outputs), value)
    }
    pub fn validate_fixture(&self, fixture: &Fixture) -> Result<(), OperationError> {
        self.validate_inputs(&canonical_value(fixture.inputs.clone()))?;
        self.validate_parameters(&canonical_value(fixture.parameters.clone()))?;
        self.validate_outputs(&canonical_value(fixture.outputs.clone()))
    }
    /// True only for the same semantic type. Formats/dimensions/capabilities still
    /// need the actual operation's runtime checks; conversions are explicit nodes.
    pub fn accepts(&self, input: &str, semantic_type: &str) -> bool {
        self.inputs
            .iter()
            .any(|p| p.name == input && p.semantic_type == semantic_type)
    }
}
/// Convert metadata defaults without interpreting bytes as UTF-8.
fn default_value(value: ParameterValue<'_>) -> Value {
    serde_json::to_value(value).expect("parameter default is serializable")
}

/// Exercise module-owned real fixtures without executing anything during catalog
/// discovery. Called explicitly by tests and the contract-check command.
pub fn validate_registered_fixtures() -> Result<usize, OperationError> {
    let mut count = 0;
    for module in super::MODULES {
        for entry in module.contracts {
            describe(&entry.descriptor)?.validate_fixture(&(entry.fixture)()?)?;
            count += 1;
        }
    }
    Ok(count)
}

/// Opt-in complete discovery. The legacy writer and its schemas remain unchanged.
pub fn discovery_document(
    request: super::DiscoveryRequest<'_>,
    format: super::DiscoveryFormat,
    maximum: usize,
) -> Result<String, OperationError> {
    use std::io::Write;
    if maximum > MAX_DOCUMENT_BYTES {
        return Err(invalid("contract discovery limit exceeds four MiB"));
    }
    let (metadata, cli) = match format {
        super::DiscoveryFormat::DirectJson { metadata } => (metadata, false),
        super::DiscoveryFormat::CliJson { metadata } => (metadata, true),
        super::DiscoveryFormat::Text => {
            return Err(invalid("complete contracts require explicit JSON output"));
        }
    };
    let describe = |descriptor: &Descriptor| -> Result<Value, OperationError> {
        let mut value = serde_json::to_value(descriptor.contract_discovery()?)
            .map_err(|_| invalid("contract serialization failed"))?;
        if !metadata {
            value
                .as_object_mut()
                .expect("discovery object")
                .remove("metadata");
        }
        Ok(value)
    };
    let value = match request {
        super::DiscoveryRequest::List { category } => {
            if category.is_some_and(|q| q.len() > super::MAX_DISCOVERY_QUERY_BYTES) {
                return Err(invalid("category query exceeds 256 bytes"));
            }
            let mut descriptors = super::filtered(category)?;
            descriptors.sort_by_key(|d| d.id);
            let values = descriptors
                .into_iter()
                .map(describe)
                .collect::<Result<Vec<_>, _>>()?;
            if cli {
                json!({"schema_version":1,"contract_schema_version":FORMAT_VERSION,"operations":values})
            } else {
                json!(values)
            }
        }
        super::DiscoveryRequest::Describe { id } => {
            if id.len() > super::MAX_DISCOVERY_QUERY_BYTES {
                return Err(invalid("operation query exceeds 256 bytes"));
            }
            let value = describe(super::require(id)?)?;
            if cli {
                json!({"schema_version":1,"contract_schema_version":FORMAT_VERSION,"operation":value})
            } else {
                value
            }
        }
    };
    struct Output {
        data: Vec<u8>,
        maximum: usize,
    }
    impl Write for Output {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.maximum.saturating_sub(self.data.len()) {
                return Err(std::io::Error::other(
                    "contract discovery output budget exceeded",
                ));
            }
            self.data.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut output = Output {
        data: Vec::new(),
        maximum,
    };
    serde_json::to_writer(&mut output, &value).map_err(|_| {
        OperationError::new(
            ErrorCode::ResourceLimit,
            "complete contract discovery exceeds output budget",
        )
    })?;
    output.write_all(b"\n").map_err(|_| {
        OperationError::new(
            ErrorCode::ResourceLimit,
            "complete contract discovery exceeds output budget",
        )
    })?;
    String::from_utf8(output.data).map_err(|_| invalid("contract JSON was not UTF-8"))
}
