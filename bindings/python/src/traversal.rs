//! Native scalar traversal boundary; no algorithm duplication.
use cube_core::{
    errors::{ErrorCode, OperationError},
    text::traversal::{self, Options},
};
use pyo3::{
    prelude::*,
    types::{PyBool, PyDict, PyList},
};
fn steps(values: &Bound<'_, PyList>) -> PyResult<Vec<u64>> {
    if values.len() > 65536 {
        return Err(super::file_error(
            OperationError::new(ErrorCode::ResourceLimit, "steps exceed 65536 entries")
                .parameter("steps"),
        ));
    }
    values
        .iter()
        .map(|v| {
            if v.is_instance_of::<PyBool>() {
                return Err(super::file_error(
                    OperationError::new(
                        ErrorCode::InvalidParameter,
                        "steps must be exact positive u64 integers",
                    )
                    .parameter("steps"),
                ));
            }
            v.extract().map_err(|_| {
                super::file_error(
                    OperationError::new(
                        ErrorCode::InvalidParameter,
                        "steps must be exact positive u64 integers",
                    )
                    .parameter("steps"),
                )
            })
        })
        .collect()
}
fn options(py: Python<'_>, value: Option<&Bound<'_, PyDict>>) -> PyResult<Options> {
    value
        .map(|v| {
            super::puzzles::document(py, v).and_then(|s| {
                serde_json::from_str(&s).map_err(|_| {
                    super::file_error(
                        OperationError::new(
                            ErrorCode::InvalidParameter,
                            "invalid concrete traversal options",
                        )
                        .parameter("options"),
                    )
                })
            })
        })
        .transpose()
        .map(Option::unwrap_or_default)
}
#[pyfunction]
#[pyo3(signature=(text,steps,count,*,options=None))]
fn keyed_skip(
    py: Python<'_>,
    text: &str,
    steps: &Bound<'_, PyList>,
    count: usize,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    let result = traversal::keyed(
        text,
        &self::steps(steps)?,
        count,
        &self::options(py, options)?,
    )
    .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(result))
}
#[pyfunction]
#[pyo3(signature=(text,steps,*,count=None,options=None))]
fn josephus_extract(
    py: Python<'_>,
    text: &str,
    steps: &Bound<'_, PyList>,
    count: Option<usize>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    let result = traversal::josephus(
        text,
        &self::steps(steps)?,
        count,
        &self::options(py, options)?,
    )
    .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(result))
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(keyed_skip, module)?)?;
    module.add_function(wrap_pyfunction!(josephus_extract, module)?)?;
    Ok(())
}
