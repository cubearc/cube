//! Typed named-port adapter over public Core APIs; no algorithm implementation.
use crate::{
    errors::{ErrorCode, OperationError},
    operations::{self, ParameterType},
    pipeline,
};
use serde_json::{Value, json};

pub(super) fn supports(operation: &str) -> bool {
    operation != "text.transform.masked"
        && pipeline::adapter_version(operation).is_some()
        && operations::describe(operation).is_some_and(|d| {
            matches!(d.input_type, "bytes" | "text") && matches!(d.output_type, "bytes" | "text")
        })
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "use exact typed pipeline ports and parameters",
    )
}
pub(super) fn execute(
    operation: &str,
    inputs: Value,
    parameters: Value,
    _budget: crate::operations::execution::Budget,
    _clock: Option<crate::operations::execution::Elapsed<'_>>,
) -> Result<Value, OperationError> {
    // Direct invocations have the operation's own declared limits.
    if let Some(profile) = crate::hashing::extended::Profile::find(operation) {
        return Ok(json!({"output": profile.digest(&super::bytes(&inputs["input"])?)?}));
    }
    use crate::binary::compression as c;
    if matches!(operation, c::ENCODE | c::DECODE) {
        let data = super::bytes(&inputs["input"])?;
        let output = if operation == c::ENCODE {
            c::compress(
                &data,
                serde_json::from_value(parameters).map_err(|_| invalid())?,
            )?
        } else {
            c::decompress(
                &data,
                serde_json::from_value(parameters).map_err(|_| invalid())?,
            )?
        };
        return Ok(json!({"output": {"base64": crate::encoding::base64_encode(&output)}}));
    }
    if operation == "binary.reverse_bits" {
        let output = crate::binary::transform::reverse_bits(
            &super::bytes(&inputs["input"])?,
            super::field(&parameters, "unit_bytes")?,
        )?;
        return Ok(json!({"output": {"base64": crate::encoding::base64_encode(&output)}}));
    }
    if let Some((profile, encode)) = crate::encoding::byte_text::Profile::find(operation) {
        if encode {
            return Ok(json!({"output": profile.encode(&super::bytes(&inputs["input"])?)?}));
        }
        let output = profile.decode(inputs["input"].as_str().ok_or_else(invalid)?)?;
        return Ok(json!({"output": {"base64": crate::encoding::base64_encode(&output)}}));
    }
    if let Some(output) = representation(operation, &inputs["input"], &parameters)? {
        return Ok(json!({"output": output}));
    }
    // The direct Core contract accepts full-width u64 counts; pipeline integers
    // have a smaller signed range. Preserve the declared direct contract here.
    if matches!(operation, "binary.rotate_left" | "binary.rotate_right") {
        let data = super::bytes(&inputs["input"])?;
        let count = parameters["count"]
            .as_u64()
            .or_else(|| parameters["count"].as_str()?.parse().ok())
            .ok_or_else(invalid)?;
        let unit = parameters["unit_bytes"].as_u64().ok_or_else(invalid)?;
        let unit = usize::try_from(unit).map_err(|_| invalid())?;
        let output = if operation == "binary.rotate_left" {
            crate::binary::transform::rotate_left(&data, count, unit)?
        } else {
            crate::binary::transform::rotate_right(&data, count, unit)?
        };
        return Ok(json!({"output":{"base64":crate::encoding::base64_encode(&output)}}));
    }
    let descriptor = operations::describe(operation).ok_or_else(invalid)?;
    let input = match descriptor.input_type {
        "bytes" => pipeline::Value::Bytes {
            data: super::bytes(&inputs["input"])?,
        },
        "text" => pipeline::Value::Text {
            data: inputs["input"].as_str().ok_or_else(invalid)?.into(),
        },
        _ => return Err(invalid()),
    };
    let mut bindings = std::collections::BTreeMap::new();
    for (name, value) in parameters.as_object().ok_or_else(invalid)? {
        let parameter = descriptor
            .parameters
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(invalid)?;
        let binding = match parameter.constraints {
            ParameterType::Bytes { .. } => pipeline::Parameter::Bytes {
                data: serde_json::from_value(value.clone()).map_err(|_| invalid())?,
            },
            ParameterType::UnsignedList { .. } => pipeline::Parameter::UnsignedList {
                values: serde_json::from_value(value.clone()).map_err(|_| invalid())?,
            },
            ParameterType::TextList { .. } => pipeline::Parameter::TextList {
                values: serde_json::from_value(value.clone()).map_err(|_| invalid())?,
            },
            _ => serde_json::from_value(value.clone()).map_err(|_| invalid())?,
        };
        bindings.insert(name.clone(), binding);
    }
    let step = pipeline::Step {
        operation: operation.into(),
        version: pipeline::adapter_version(operation).ok_or_else(invalid)?,
        parameters: bindings,
    };
    let limits = pipeline::Limits {
        max_steps: 1,
        max_value_bytes: pipeline::MAX_VALUE_BYTES,
        max_work_bytes: pipeline::MAX_WORK_BYTES,
        max_capture_bytes: 0,
    };
    let output = pipeline::Plan::validate(input.kind(), vec![step], limits)
        .and_then(|plan| plan.execute(input, &[], None))
        .map_err(|e| *e.error)?
        .output;
    match output {
        pipeline::Value::Bytes { data } => {
            Ok(json!({"output":{"base64":crate::encoding::base64_encode(&data)}}))
        }
        pipeline::Value::Text { data } => Ok(json!({"output":data})),
        _ => Err(invalid()),
    }
}

fn representation(
    operation: &str,
    input: &Value,
    parameters: &Value,
) -> Result<Option<Value>, OperationError> {
    use crate::text::representation as r;
    let text = || input.as_str().ok_or_else(invalid);
    let settings =
        || serde_json::from_value::<r::Utf16Settings>(parameters.clone()).map_err(|_| invalid());
    let base64 = |value: &[u8]| json!({"base64": crate::encoding::base64_encode(value)});
    Ok(Some(match operation {
        "text.ascii.encode" => base64(&r::ascii_encode(text()?)?),
        "text.ascii.decode" => json!(r::ascii_decode(&super::bytes(input)?)?),
        "text.utf16.encode" => base64(&r::utf16_encode(text()?, settings()?)?),
        "text.utf16.decode" => json!(r::utf16_decode(&super::bytes(input)?, settings()?)?),
        "text.json_escape.encode" => json!(r::json_escape_encode(
            text()?,
            super::field(parameters, "ascii_only")?
        )?),
        "text.json_escape.decode" => json!(r::json_escape_decode(text()?)?),
        "text.html_entity.encode" => json!(r::html_entity_encode(text()?)?),
        "text.html_entity.decode" => json!(r::html_entity_decode(text()?)?),
        _ => return Ok(None),
    }))
}
