//! Native typed pipeline boundary; execution and validation live entirely in Rust.
use cube_core::pipeline::{self, Kind, Limits, Value};
use pyo3::{
    prelude::*,
    types::{PyAny, PyBytes, PyDict, PyList, PyString},
};
use std::sync::atomic::{AtomicBool, Ordering};
pub(super) fn error(py: Python<'_>, error: pipeline::Error) -> PyErr {
    let exception = pyo3::exceptions::PyValueError::new_err(error.to_string());
    let value = exception.value(py);
    if let Err(e) = (|| -> PyResult<()> {
        value.setattr("code", error.error.code.as_str())?;
        value.setattr("step", error.step)?;
        value.setattr("operation", error.operation)?;
        value.setattr("parameter", error.parameter)?;
        value.setattr("offset", error.error.offset)?;
        Ok(())
    })() {
        return e;
    }
    exception
}
fn json_text(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<String> {
    let kwargs = PyDict::new(py);
    kwargs.set_item("allow_nan", false)?;
    py.import("json")?
        .call_method("dumps", (value,), Some(&kwargs))?
        .extract()
}
fn limits(py: Python<'_>, value: Option<&Bound<'_, PyDict>>) -> PyResult<Limits> {
    value
        .map(|v| {
            json_text(py, v).and_then(|s| pipeline::json::limits(&s).map_err(|e| error(py, e)))
        })
        .transpose()
        .map(Option::unwrap_or_default)
}
fn steps(py: Python<'_>, value: &Bound<'_, PyList>) -> PyResult<Vec<pipeline::Step>> {
    if value.len() > pipeline::MAX_STEPS {
        return Err(error(
            py,
            pipeline::Error::new(
                cube_core::errors::ErrorCode::ResourceLimit,
                "pipeline exceeds 64 steps",
            ),
        ));
    }
    pipeline::json::steps(&json_text(py, value)?).map_err(|e| error(py, e))
}
pub(super) fn input(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Value> {
    let result = if let Ok(bytes) = value.cast::<PyBytes>() {
        if bytes.as_bytes().len() > pipeline::MAX_VALUE_BYTES {
            return Err(error(
                py,
                pipeline::Error::new(
                    cube_core::errors::ErrorCode::ResourceLimit,
                    "input exceeds 1 MiB",
                ),
            ));
        }
        Value::Bytes {
            data: bytes.as_bytes().to_vec(),
        }
    } else if let Ok(text) = value.cast::<PyString>() {
        let text = text.to_str()?;
        if text.len() > pipeline::MAX_VALUE_BYTES {
            return Err(error(
                py,
                pipeline::Error::new(
                    cube_core::errors::ErrorCode::ResourceLimit,
                    "input exceeds 1 MiB",
                ),
            ));
        }
        Value::Text { data: text.into() }
    } else if value.is_instance_of::<PyDict>() {
        pipeline::json::value(&json_text(py, value)?).map_err(|e| error(py, e))?
    } else {
        return Err(pyo3::exceptions::PyTypeError::new_err(
            "pipeline input must be bytes, str or an explicit tagged value dict",
        ));
    };
    Ok(result)
}
pub(super) fn output(py: Python<'_>, value: &Value) -> PyResult<Py<PyAny>> {
    match value {
        Value::Bytes { data } => Ok(PyBytes::new(py, data).into_any().unbind()),
        Value::Text { data } => Ok(PyString::new(py, data).into_any().unbind()),
        Value::ByteRange { .. } => super::structured(py, &serde_json::json!(value)),
    }
}
pub(super) fn execution(py: Python<'_>, result: &pipeline::Execution) -> PyResult<Py<PyAny>> {
    let dictionary = PyDict::new(py);
    dictionary.set_item("output", output(py, &result.output)?)?;
    dictionary.set_item(
        "output_type",
        serde_json::to_value(result.output.kind())
            .expect("kind")
            .as_str()
            .expect("kind string"),
    )?;
    let captures = PyDict::new(py);
    for (index, value) in &result.intermediates {
        captures.set_item(index, output(py, value)?)?;
    }
    dictionary.set_item("intermediates", captures)?;
    dictionary.set_item("work_bytes_upper_bound", result.work_bytes_upper_bound)?;
    Ok(dictionary.into_any().unbind())
}
/// Sticky cancellation token. Another Python thread can cancel during detached execution.
#[pyclass(frozen, module = "cube._native", name = "CancellationToken")]
pub struct CancellationToken {
    pub(super) flag: AtomicBool,
}
#[pymethods]
impl CancellationToken {
    #[new]
    fn new() -> Self {
        Self {
            flag: AtomicBool::new(false),
        }
    }
    fn cancel(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }
    #[getter]
    fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }
}
#[pyfunction]
fn pipeline_adapters(py: Python<'_>) -> PyResult<Py<PyAny>> {
    super::structured(
        py,
        &serde_json::json!({"engine_api":pipeline::ENGINE_API,"engine_version":env!("CARGO_PKG_VERSION"),"adapters":pipeline::supported_adapters()}),
    )
}
#[pyfunction]
#[pyo3(signature=(steps, input_type, *, limits=None))]
fn validate_pipeline(
    py: Python<'_>,
    steps: &Bound<'_, PyList>,
    input_type: &str,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    let kind: Kind = input_type.parse().map_err(|e| error(py, e))?;
    let plan = pipeline::Plan::validate(
        kind,
        crate::pipeline::steps(py, steps)?,
        crate::pipeline::limits(py, limits)?,
    )
    .map_err(|e| error(py, e))?;
    super::structured(py, &serde_json::json!(plan.info()))
}
#[pyfunction]
#[pyo3(signature=(steps, input, *, limits=None, capture=None, cancellation=None))]
fn execute_pipeline(
    py: Python<'_>,
    steps: &Bound<'_, PyList>,
    input: &Bound<'_, PyAny>,
    limits: Option<&Bound<'_, PyDict>>,
    capture: Option<Vec<usize>>,
    cancellation: Option<&CancellationToken>,
) -> PyResult<Py<PyAny>> {
    let input = crate::pipeline::input(py, input)?;
    let plan = pipeline::Plan::validate(
        input.kind(),
        crate::pipeline::steps(py, steps)?,
        crate::pipeline::limits(py, limits)?,
    )
    .map_err(|e| error(py, e))?;
    let capture = capture.unwrap_or_default();
    let token = cancellation.map(|c| &c.flag);
    let result = py
        .detach(|| plan.execute(input, &capture, token))
        .map_err(|e| error(py, e))?;
    execution(py, &result)
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<CancellationToken>()?;
    module.add_function(wrap_pyfunction!(pipeline_adapters, module)?)?;
    module.add_function(wrap_pyfunction!(validate_pipeline, module)?)?;
    module.add_function(wrap_pyfunction!(execute_pipeline, module)?)?;
    Ok(())
}
