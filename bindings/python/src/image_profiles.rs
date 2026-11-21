//! Native image and byte objects at the shared Core image-profile boundary.
use super::{discrete_transport::typed_document, file_error, image::PyImage, structured};
use cube_core::{
    encoding,
    errors::{ErrorCode, OperationError},
    image::{
        advanced_catalog::{MAX_DOCUMENT, OPERATIONS},
        stereo::Source,
    },
    operations::{
        contracts::values,
        execution::{self, Budget, InvocationError},
    },
};
use pyo3::{
    prelude::*,
    types::{PyBytes, PyDict, PyList},
};
use serde_json::{Map, Value, json};

fn failure(id: &'static str, code: ErrorCode, message: &'static str) -> PyErr {
    file_error(OperationError::new(code, message).operation(id))
}

fn json_value(value: &Bound<'_, PyAny>, limit: usize, id: &'static str) -> PyResult<Value> {
    // Use an array wrapper so JSON strings are treated as values, rather than document text.
    let outer = PyList::new(value.py(), [value])?;
    let text = typed_document(value.py(), outer.as_any(), limit, 2_500_000, 20, id, true)?;
    serde_json::from_str::<[Value; 1]>(&text)
        .map(|[value]| value)
        .map_err(|_| failure(id, ErrorCode::InvalidInput, "use finite JSON values"))
}

/// Invoke a bounded image profile with native Image and bytes ports.
#[pyfunction]
pub fn execute_image_profile<'py>(
    py: Python<'py>,
    operation: &str,
    inputs: &Bound<'py, PyDict>,
    parameters: &Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyDict>> {
    let descriptor = OPERATIONS
        .iter()
        .find(|op| op.id == operation)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("unknown image profile"))?;
    let id = descriptor.id;
    let contract = descriptor.contract().map_err(file_error)?;
    if inputs.len() > contract.inputs.len() {
        return Err(failure(
            id,
            ErrorCode::InvalidInput,
            "unexpected image profile inputs",
        ));
    }
    let bound = |limit: Option<u64>| limit.unwrap_or(u64::MAX).min(MAX_DOCUMENT as u64) as usize;
    let input_limit = bound(contract.limits.max_input_bytes);
    let mut fields = Map::new();
    let mut provenance = Map::new();
    for (key, value) in inputs.iter() {
        let key = key.extract::<String>()?;
        let port = contract
            .inputs
            .iter()
            .find(|p| p.name == key)
            .ok_or_else(|| {
                failure(
                    id,
                    ErrorCode::InvalidInput,
                    "unexpected image profile input",
                )
            })?;
        let value = match port.semantic_type.as_str() {
            "image.samples" => {
                let native = value.extract::<PyRef<'_, PyImage>>()?;
                let image = &native.inner;
                provenance.insert(key.clone(), json!(Source::from(image)));
                json!(values::Image::from(image))
            }
            "bytes" => {
                let raw = value.cast::<PyBytes>()?.as_bytes();
                if raw.len() > input_limit {
                    return Err(failure(
                        id,
                        ErrorCode::ResourceLimit,
                        "image profile bytes exceed the input limit",
                    ));
                }
                json!({"base64": encoding::base64_encode(raw)})
            }
            _ if id == "image.barcode.decode" => {
                let crops = value.cast::<PyList>()?;
                if crops.len() > 8 {
                    return Err(failure(
                        id,
                        ErrorCode::ResourceLimit,
                        "at most eight optical crops",
                    ));
                }
                let mut bytes = 0;
                let mut converted = Vec::with_capacity(crops.len());
                for crop in crops.iter() {
                    let native = crop.extract::<PyRef<'_, PyImage>>()?;
                    let image = &native.inner;
                    bytes += image.samples().len();
                    if image.samples().len() > 262_144 || bytes > input_limit {
                        return Err(failure(
                            id,
                            ErrorCode::ResourceLimit,
                            "optical crops exceed the sample ceiling",
                        ));
                    }
                    converted.push(values::Image::from(image));
                }
                json!(converted)
            }
            _ => json_value(&value, input_limit, id)?,
        };
        fields.insert(key, value);
    }
    let budget = Budget {
        input_bytes: input_limit,
        output_bytes: bound(contract.limits.max_output_bytes),
        ..Budget::default()
    };
    let parameters = json_value(parameters.as_any(), input_limit, id)?;
    let mut result = execution::invoke(
        id,
        contract.operation_version,
        &Value::Object(fields),
        &parameters,
        budget,
    )
    .map_err(|error| match error {
        InvocationError::Operation(error) => file_error(error),
        InvocationError::Code(_) => failure(
            id,
            ErrorCode::InvalidInput,
            "image profile invocation failed",
        ),
    })?;
    // A logical samples port omits codec history. Native callers supplied the
    // validated Image itself, so keep that original evidence where reported.
    match id {
        "image.warp" | "image.stereo.separate" => {
            result["report"]["source"] = provenance.remove("input").expect("validated source");
        }
        "image.stereo.anaglyph" => {
            result["report"]["sources"] = json!([provenance["left"], provenance["right"]]);
        }
        _ => {}
    }
    let output = PyDict::new(py);
    for port in &contract.outputs {
        let Some(value) = result.get(&port.name) else {
            continue;
        };
        match port.semantic_type.as_str() {
            "image.samples" => {
                let inner = execution::decode_image_samples(value).map_err(file_error)?;
                output.set_item(&port.name, Py::new(py, PyImage { inner })?)?;
            }
            "bytes" => {
                let base64 = value["base64"].as_str().expect("validated bytes output");
                let bytes = encoding::base64_decode(base64).map_err(|_| {
                    failure(id, ErrorCode::InvalidInput, "invalid Core bytes output")
                })?;
                output.set_item(&port.name, PyBytes::new(py, &bytes))?;
            }
            _ => output.set_item(&port.name, structured(py, value)?)?,
        }
    }
    Ok(output)
}
