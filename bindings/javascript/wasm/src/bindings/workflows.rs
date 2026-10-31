//! Typed named-port adapter over public Core APIs; no algorithm implementation.
use super::field;
use cube_core::{
    errors::{ErrorCode, OperationError},
    investigation::{profile, session, triage},
    pipeline::{self, recipe::Recipe},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
enum Adapter {
    Pipeline,
    Validate,
    Check,
    Recipe,
    RecipeValidate,
    Plan,
    Profile,
    Replay,
    Report,
    Record,
    Triage,
}
fn adapter(op: &str) -> Option<Adapter> {
    Some(match op {
        "pipeline.execute" => Adapter::Pipeline,
        "pipeline.validate" => Adapter::Validate,
        "recipe.check" => Adapter::Check,
        "recipe.execute" => Adapter::Recipe,
        "recipe.validate" => Adapter::RecipeValidate,
        "investigation.profile.plan" => Adapter::Plan,
        "investigation.profile.run" => Adapter::Profile,
        "investigation.replay" => Adapter::Replay,
        "investigation.report" => Adapter::Report,
        "investigation.session.record" => Adapter::Record,
        "investigation.triage" => Adapter::Triage,
        _ => return None,
    })
}
pub(super) fn supports(op: &str) -> bool {
    adapter(op).is_some()
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "use an explicit bounded Core workflow definition and exact typed values",
    )
}
fn document(v: &Value) -> Result<String, OperationError> {
    serde_json::to_string(v).map_err(|_| invalid())
}
fn unsigned(value: &Value) -> Result<u64, OperationError> {
    value
        .as_u64()
        .or_else(|| value.as_str()?.parse().ok())
        .ok_or_else(invalid)
}
// Restore only the Core tagged value's native integer fields. Text and byte
// representations remain explicit; no arbitrary JSON strings are coerced.
fn pipeline_value(value: &Value) -> Result<pipeline::Value, OperationError> {
    if value["type"] == "byte_range" {
        return Ok(pipeline::Value::ByteRange {
            offset: unsigned(&value["offset"])?,
            length: if value["length"].is_null() {
                None
            } else {
                Some(unsigned(&value["length"])?)
            },
        });
    }
    serde_json::from_value(value.clone()).map_err(|_| invalid())
}
fn recipe(value: &Value) -> Result<Recipe, OperationError> {
    let mut native = value.clone();
    if native["input"]["source"] == "inline" {
        native["input"]["value"] = serde_json::to_value(pipeline_value(&native["input"]["value"])?)
            .map_err(|_| invalid())?;
    }
    Recipe::parse(&document(&native)?).map_err(|e| *e.error)
}
fn session(value: &Value) -> Result<session::Session, OperationError> {
    let mut native = value.clone();
    native["input"]["size"] = json!(unsigned(&native["input"]["size"])?);
    serde_json::from_value(native).map_err(|_| invalid())
}
fn plan(v: &Value) -> Result<pipeline::Plan, OperationError> {
    pipeline::Plan::validate(
        field(v, "input_type")?,
        field(v, "steps")?,
        field(v, "limits")?,
    )
    .map_err(|e| *e.error)
}
pub(super) fn execute(op: &str, inputs: Value, p: Value) -> Result<Value, OperationError> {
    let identity = cube_core::operations::require(op)?.id;
    let selected = adapter(op).ok_or_else(invalid)?;
    let result = (|| {
        let request = &inputs["input"];
        Ok(match selected {
            Adapter::Pipeline => {
                json!({"output":plan(&request["definition"] )?.execute(pipeline_value(&request["input"])?,&[],None).map_err(|e|*e.error)?})
            }
            Adapter::Validate => {
                let plan = plan(request)?;
                json!({"output":plan.info()})
            }
            Adapter::RecipeValidate => {
                let recipe = recipe(request)?;
                json!({"output":recipe.info()})
            }
            Adapter::Recipe => {
                let recipe = recipe(&request["recipe"])?;
                let bindings = request["bindings"]
                    .as_object()
                    .ok_or_else(invalid)?
                    .iter()
                    .map(|(k, v)| pipeline_value(v).map(|v| (k.clone(), v)))
                    .collect::<Result<BTreeMap<_, _>, _>>()?;
                json!({"output":recipe.execute(&bindings,&[],None).map_err(|e|*e.error)?})
            }
            Adapter::Check => {
                let recipe = recipe(&request["recipe"])?;
                let input = pipeline_value(&request["input"])?;
                let bindings = if let Some(slot) = recipe.info().input_slot {
                    BTreeMap::from([(slot.into(), input)])
                } else {
                    let declared = pipeline_value(&request["recipe"]["input"]["value"])?;
                    if declared != input {
                        return Err(invalid().parameter("input"));
                    }
                    BTreeMap::new()
                };
                json!({"output":recipe.check(&bindings,&pipeline_value(&request["expected"])?).map_err(|e|*e.error)?})
            }
            Adapter::Plan => {
                let profile: profile::Profile = field(request, "profile")?;
                let size = request["input_size"]
                    .as_u64()
                    .or_else(|| request["input_size"].as_str()?.parse().ok())
                    .ok_or_else(invalid)?;
                json!({"output":profile.dry_run(size).map_err(|e|*e.error)?})
            }
            Adapter::Profile => {
                let profile: profile::Profile = field(&inputs, "profile")?;
                json!({"output":run_profile(&profile,&super::bytes(request)?)?.session})
            }
            Adapter::Replay => {
                let session = session(&request["session"])?;
                json!({"output":session.replay(&super::bytes(&request["input"])?,None).map_err(|e|*e.error)?})
            }
            Adapter::Report => {
                let session = session(&inputs["input"])?;
                let report = session
                    .to_json(&session::Export::default())
                    .map_err(|e| *e.error)?;
                json!({"output":serde_json::from_str::<Value>(&report).map_err(|_|invalid())?})
            }
            Adapter::Record => {
                let bytes = super::bytes(&request["input"])?;
                let mut session =
                    session::Session::new(bytes.len() as u64, field(request, "limits")?)
                        .map_err(|e| *e.error)?;
                session
                    .execute(
                        &bytes,
                        field(request, "spec")?,
                        session::Capture {
                            parameters: field(request, "record_parameters")?,
                            result: field(request, "retain_result")?,
                        },
                        None,
                    )
                    .map_err(|e| *e.error)?;
                json!({"output":session})
            }
            Adapter::Triage => {
                json!({"output":triage::bytes(&super::bytes(request)?,field(&p,"max_read_bytes")?,&cube_core::operations::catalog())?})
            }
        })
    })();
    result.map_err(|e: OperationError| e.operation(identity))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(catch, js_namespace = performance, js_name = now)]
    fn monotonic_now() -> Result<f64, wasm_bindgen::JsValue>;
}
fn run_profile(
    profile: &profile::Profile,
    data: &[u8],
) -> Result<profile::Investigation, OperationError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        profile.run_bytes(data, None).map_err(|e| *e.error)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let clock_error =
            || OperationError::new(ErrorCode::Io, "monotonic clock is unavailable or invalid");
        let start = monotonic_now().map_err(|_| clock_error())?;
        if !start.is_finite() {
            return Err(clock_error());
        }
        profile
            .run_bytes_with_clock(data, None, || {
                let elapsed = monotonic_now()
                    .map_err(|_| pipeline::Error::from_operation(clock_error()))?
                    - start;
                std::time::Duration::try_from_secs_f64(elapsed / 1000.0)
                    .map_err(|_| pipeline::Error::from_operation(clock_error()))
            })
            .map_err(|e| *e.error)
    }
}
