//! Versioned declarative JSON recipes. Definitions contain no paths or capabilities.
//!
//! Format 1 requires engine API 1 and exact adapter-contract versions. The package
//! version is reported for provenance, not used as an accidental compatibility pin.
//! Breaking engine/adapter behavior must receive a new corresponding contract version.
//!
//! The strict root fields are `format_version`, `engine_api`, `input`, `output`,
//! `steps` and optional lowered `limits`. Input is one symbolic typed `slot` or an
//! `inline` tagged value. Steps require stable `operation`, contract `version` and
//! optional typed `parameters`; output declares its `type`. Bytes use canonical
//! padded base64, not guessed text. JSON is capped before schema parsing.
//!
//! ```
//! use cube_core::pipeline::recipe::Recipe;
//! let recipe = Recipe::parse(r#"{
//!   "format_version":1,"engine_api":1,
//!   "input":{"source":"inline","value":{"type":"bytes","base64":"AP8="}},
//!   "output":{"type":"text"},
//!   "steps":[{"operation":"encoding.hex.encode","version":1}]
//! }"#)?;
//! assert_eq!(recipe.info().plan.steps.len(), 1);
//! # Ok::<(), cube_core::pipeline::Error>(())
//! ```
use super::{Error, Execution, Kind, Limits, Plan, Step, Value, invalid, json};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::atomic::AtomicBool};
pub const FORMAT_VERSION: u32 = 1;
#[derive(Clone, Debug, Deserialize, schemars::JsonSchema)]
#[serde(tag = "source", rename_all = "snake_case", deny_unknown_fields)]
enum Input {
    Slot {
        name: String,
        #[serde(rename = "type")]
        kind: Kind,
    },
    Inline {
        value: Value,
    },
}
impl Input {
    fn kind(&self) -> Kind {
        match self {
            Self::Slot { kind, .. } => *kind,
            Self::Inline { value } => value.kind(),
        }
    }
}
#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Output {
    #[serde(rename = "type")]
    kind: Kind,
}
#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Definition {
    format_version: u32,
    engine_api: u32,
    input: Input,
    output: Output,
    #[schemars(with = "Vec<super::Step>")]
    steps: Vec<serde_json::Value>,
    #[serde(default)]
    limits: Limits,
}
/// Validated immutable recipe. Inline data is retained only in caller-owned memory.
#[derive(Clone, Debug)]
pub struct Recipe {
    input: Input,
    plan: Plan,
}
/// Redacted validation report, with no inline contents, parameter secrets or bindings.
#[derive(Serialize, schemars::JsonSchema)]
pub struct Info<'a> {
    pub format_version: u32,
    pub input_slot: Option<&'a str>,
    pub plan: super::PlanInfo<'a>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Check {
    pub passed: bool,
    pub format_version: u32,
    pub engine_api: u32,
    pub engine_version: &'static str,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    #[schemars(default)]
    pub masked_audits: BTreeMap<usize, crate::text::transform::Report>,
}
/// Slot names are symbolic, never paths, environment lookups or expressions.
pub fn validate_slot(name: &str) -> Result<(), Error> {
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(invalid(
            "slot name requires 1..64 ASCII letters, digits, underscores or hyphens",
        )
        .parameter("input"));
    }
    Ok(())
}
impl Recipe {
    /// Validate the complete document without executing operations or reading inputs.
    /// Duplicate keys, unknown fields and implicit string/binary coercions are errors.
    pub fn parse(document: &str) -> Result<Self, Error> {
        let definition: Definition = json::schema(json::parse(document)?, "recipe")?;
        if definition.format_version != FORMAT_VERSION {
            return Err(
                invalid("unsupported recipe format; expected format_version 1")
                    .parameter("format_version"),
            );
        }
        if definition.engine_api != super::ENGINE_API {
            return Err(invalid(
                "unsupported engine compatibility contract; expected engine_api 1",
            )
            .parameter("engine_api"));
        }
        match &definition.input {
            Input::Slot { name, .. } => validate_slot(name)?,
            Input::Inline { value } => value.validate(definition.limits.max_value_bytes)?,
        }
        if definition.steps.len() > super::MAX_STEPS {
            return Err(Error::new(
                crate::errors::ErrorCode::ResourceLimit,
                "recipe exceeds 64 steps",
            ));
        }
        let steps: Vec<Step> = definition
            .steps
            .into_iter()
            .enumerate()
            .map(|(i, value)| {
                let operation = value
                    .get("operation")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned);
                json::schema(value, "steps").map_err(|e| e.at(i, operation.as_deref()))
            })
            .collect::<Result<_, _>>()?;
        let plan = Plan::validate(definition.input.kind(), steps, definition.limits)?;
        if plan.info().output_type != definition.output.kind {
            return Err(
                invalid("declared output type differs from validated final step type")
                    .parameter("output"),
            );
        }
        Ok(Self {
            input: definition.input,
            plan,
        })
    }
    pub fn info(&self) -> Info<'_> {
        Info {
            format_version: FORMAT_VERSION,
            input_slot: match &self.input {
                Input::Slot { name, .. } => Some(name),
                Input::Inline { .. } => None,
            },
            plan: self.plan.info(),
        }
    }
    /// Bind exactly the declared slot (or none for inline input), then execute the
    /// common engine. Files, environment variables and external resources are never resolved.
    pub fn execute(
        &self,
        bindings: &BTreeMap<String, Value>,
        capture: &[usize],
        cancellation: Option<&AtomicBool>,
    ) -> Result<Execution, Error> {
        let value = match &self.input {
            Input::Slot { name, kind } => {
                if bindings.len() != 1 || !bindings.contains_key(name) {
                    return Err(invalid("provide exactly the declared input slot; missing and extra bindings are errors").parameter("bindings"));
                }
                let value = &bindings[name];
                if value.kind() != *kind {
                    return Err(invalid("bound input type differs from declared slot type")
                        .parameter("bindings"));
                }
                value
            }
            Input::Inline { value } => {
                if !bindings.is_empty() {
                    return Err(
                        invalid("inline recipes accept no external bindings").parameter("bindings")
                    );
                }
                value
            }
        };
        value.validate(self.plan.limits.max_value_bytes)?;
        self.plan.execute(value.clone(), capture, cancellation)
    }
    /// Independent community contract test: compare exact typed final output,
    /// exposing no input contents on failure. No files are created or persisted.
    pub fn check(
        &self,
        bindings: &BTreeMap<String, Value>,
        expected: &Value,
    ) -> Result<Check, Error> {
        expected.validate(self.plan.limits.max_value_bytes)?;
        if expected.kind() != self.plan.info().output_type {
            return Err(
                invalid("expected result type differs from recipe output").parameter("expected")
            );
        }
        let actual = self.execute(bindings, &[], None)?;
        if actual.output != *expected {
            return Err(invalid(
                "recipe contract failed: exact typed output differs from expected result",
            )
            .parameter("expected"));
        }
        Ok(Check {
            passed: true,
            format_version: FORMAT_VERSION,
            engine_api: super::ENGINE_API,
            engine_version: env!("CARGO_PKG_VERSION"),
            masked_audits: actual.masked_audits,
        })
    }
}
pub const OPERATIONS: &[crate::operations::Descriptor] = &[
    crate::operations::Descriptor {
        id: "recipe.validate",
        category: "recipes",
        description: "Validate strict versioned JSON recipe contracts without executing operations or resolving input slots.",
        input_type: "recipe.document",
        output_type: "recipe.plan",
        parameters: &[],
        deterministic: true,
        reversible: None,
        command: "recipe validate",
        example: "cube recipe validate recipe.json",
    },
    crate::operations::Descriptor {
        id: "recipe.execute",
        category: "recipes",
        description: "Execute a validated recipe with exact caller-supplied typed bindings and bounded local operations.",
        input_type: "recipe.request",
        output_type: "pipeline.execution",
        parameters: &[],
        deterministic: true,
        reversible: None,
        command: "recipe run",
        example: "cube recipe run recipe.json --hex payload=00ff --format raw",
    },
    crate::operations::Descriptor {
        id: "recipe.check",
        category: "recipes",
        description: "Check an independent community recipe against an exact typed expected output without persisting results.",
        input_type: "recipe.contract",
        output_type: "recipe.check",
        parameters: &[],
        deterministic: true,
        reversible: None,
        command: "recipe check",
        example: "cube recipe check recipe.json --hex payload=00ff --expect '{\"type\":\"bytes\",\"base64\":\"AP8=\"}'",
    },
];

/// Schema from the exact private parser definition, without exposing its fields.
pub(crate) fn contract_definition(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
    generator.subschema_for::<Definition>()
}
pub(crate) fn contract_document() -> schemars::Schema {
    schemars::schema_for!(Definition)
}
