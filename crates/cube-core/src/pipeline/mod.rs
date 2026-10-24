//! Small explicit linear composition boundary; no file, shell, network or plugin capabilities.
mod adapters;
mod execute;
pub mod json;
pub mod recipe;
mod value;
use crate::errors::{ErrorCode, OperationError};
pub use execute::Execution;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, error::Error as StdError, fmt};
pub use value::{Kind, Parameter, Value};
pub const ENGINE_API: u32 = 1;
pub const MAX_STEPS: usize = 64;
pub const MAX_VALUE_BYTES: usize = 1024 * 1024;
pub const MAX_WORK_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_CAPTURE_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_PARAMETER_BYTES: usize = 256 * 1024;
/// Preserve original operation/parser errors and add contextual failing-step data.
#[derive(Debug)]
pub struct Error {
    pub error: Box<OperationError>,
    pub step: Option<usize>,
    pub operation: Option<String>,
    pub parameter: Option<String>,
}
impl Error {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::from_operation(OperationError::new(code, message))
    }
    pub fn from_operation(error: OperationError) -> Self {
        let operation = error.operation.map(str::to_owned);
        let parameter = error.parameter.map(str::to_owned);
        Self {
            error: Box::new(error),
            step: None,
            operation,
            parameter,
        }
    }
    pub(super) fn at(mut self, step: usize, operation: Option<&str>) -> Self {
        self.step = Some(step);
        self.operation = operation.filter(|s| valid_id(s)).map(str::to_owned);
        self
    }
    pub(super) fn parameter(mut self, field: &str) -> Self {
        self.parameter = Some(field.chars().take(64).collect());
        self
    }
    pub fn diagnostic(&self) -> serde_json::Value {
        serde_json::json!({"code":self.error.code,"message":self.error.message,"step":self.step,"operation":self.operation,"parameter":self.parameter,"offset":self.error.offset})
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(step) = self.step {
            write!(f, "step {step}")?;
            if let Some(id) = &self.operation {
                write!(f, " ({id})")?;
            }
            f.write_str(": ")?;
        }
        if let Some(p) = &self.parameter {
            write!(f, "parameter '{p}': ")?;
        }
        f.write_str(&self.error.message)
    }
}
impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.error.as_ref())
    }
}
impl Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.diagnostic().serialize(s)
    }
}
pub(super) fn invalid(message: &str) -> Error {
    Error::new(ErrorCode::InvalidParameter, message)
}
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'_')
}
/// Hard bounds may be lowered per plan. Zero permits only zero-size/identity work.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Limits {
    pub max_steps: usize,
    pub max_value_bytes: usize,
    pub max_work_bytes: u64,
    pub max_capture_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_steps: MAX_STEPS,
            max_value_bytes: MAX_VALUE_BYTES,
            max_work_bytes: MAX_WORK_BYTES,
            max_capture_bytes: MAX_CAPTURE_BYTES,
        }
    }
}
impl Limits {
    pub fn validate(&self) -> Result<(), Error> {
        if self.max_steps > MAX_STEPS
            || self.max_value_bytes > MAX_VALUE_BYTES
            || self.max_work_bytes > MAX_WORK_BYTES
            || self.max_capture_bytes > MAX_CAPTURE_BYTES
        {
            return Err(
                invalid("configured limits exceed hard step/value/work/capture bounds")
                    .parameter("limits"),
            );
        }
        Ok(())
    }
}
/// Each adapter has an explicit operation-contract version, independent of package
/// version. Recipes must name it; matching only an ID is insufficient compatibility.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub operation: String,
    pub version: u32,
    #[serde(default)]
    pub parameters: BTreeMap<String, Parameter>,
}
impl Step {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            version: 1,
            parameters: BTreeMap::new(),
        }
    }
}
/// Immutable validated plan. No algorithm executes while preparing it.
#[derive(Clone, Debug)]
pub struct Plan {
    input: Kind,
    output: Kind,
    steps: Vec<adapters::Prepared>,
    limits: Limits,
}
#[derive(Serialize)]
pub struct StepInfo {
    pub step: usize,
    pub operation: &'static str,
    pub version: u32,
    pub input_type: Kind,
    pub output_type: Kind,
}
#[derive(Serialize)]
pub struct PlanInfo<'a> {
    pub engine_api: u32,
    pub engine_version: &'static str,
    pub input_type: Kind,
    pub output_type: Kind,
    pub steps: Vec<StepInfo>,
    pub limits: &'a Limits,
}
impl Plan {
    pub fn validate(input: Kind, steps: Vec<Step>, limits: Limits) -> Result<Self, Error> {
        limits.validate()?;
        if steps.len() > limits.max_steps {
            return Err(Error::new(
                ErrorCode::ResourceLimit,
                "step count exceeds configured budget",
            ));
        }
        let mut kind = input;
        let mut prepared = Vec::with_capacity(steps.len());
        let mut parameters = 0usize;
        for (index, step) in steps.into_iter().enumerate() {
            let result = (|| {
                if !valid_id(&step.operation) {
                    return Err(invalid(
                        "operation must be a stable lowercase namespaced ID, at most 128 bytes",
                    ));
                }
                if step.parameters.len() > 8 {
                    return Err(Error::new(
                        ErrorCode::ResourceLimit,
                        "step exceeds eight parameter bindings",
                    ));
                }
                parameters += step.parameters.values().map(Parameter::size).sum::<usize>();
                if parameters > MAX_PARAMETER_BYTES {
                    return Err(Error::new(
                        ErrorCode::ResourceLimit,
                        "parameter values exceed 256 KiB",
                    ));
                }
                let item = adapters::prepare(&step)?;
                let (expected, output) = item.kinds()?;
                if kind != expected {
                    return Err(invalid(
                        "adjacent types differ; use an explicit byte/text conversion operation",
                    )
                    .parameter("input_type"));
                }
                kind = output;
                Ok(item)
            })()
            .map_err(|e: Error| e.at(index, Some(&step.operation)))?;
            prepared.push(result);
        }
        Ok(Self {
            input,
            output: kind,
            steps: prepared,
            limits,
        })
    }
    pub fn info(&self) -> PlanInfo<'_> {
        PlanInfo {
            engine_api: ENGINE_API,
            engine_version: env!("CARGO_PKG_VERSION"),
            input_type: self.input,
            output_type: self.output,
            steps: self
                .steps
                .iter()
                .enumerate()
                .map(|(step, p)| {
                    let (input_type, output_type) = p.kinds().expect("validated descriptor");
                    StepInfo {
                        step,
                        operation: p.adapter.id,
                        version: p.adapter.version,
                        input_type,
                        output_type,
                    }
                })
                .collect(),
            limits: &self.limits,
        }
    }
}
#[derive(Serialize)]
pub struct AdapterInfo {
    pub operation: &'static str,
    pub version: u32,
    pub descriptor: &'static crate::operations::Descriptor,
}
pub fn supported_adapters() -> Vec<AdapterInfo> {
    adapters::ADAPTERS
        .iter()
        .map(|a| AdapterInfo {
            operation: a.id,
            version: a.version,
            descriptor: crate::operations::describe(a.id).expect("maintained adapter descriptor"),
        })
        .collect()
}
pub const OPERATIONS: &[crate::operations::Descriptor] = &[
    crate::operations::Descriptor {
        id: "pipeline.execute",
        category: "pipelines",
        description: "Execute a prevalidated bounded linear plan; explicit typed values and optional bounded captures, no capabilities.",
        input_type: "pipeline.request",
        output_type: "pipeline.execution",
        parameters: &[],
        deterministic: true,
        reversible: None,
        command: "pipeline run",
        example: "cube pipeline run --steps '[]' --hex 00ff --format raw",
    },
    crate::operations::Descriptor {
        id: "pipeline.validate",
        category: "pipelines",
        description: "Validate all bounded linear step IDs, contract versions, parameters and adjacent types without executing algorithms.",
        input_type: "pipeline.definition",
        output_type: "pipeline.plan",
        parameters: &[],
        deterministic: true,
        reversible: None,
        command: "pipeline validate",
        example: "cube pipeline validate --input-type bytes --steps '[]'",
    },
];
