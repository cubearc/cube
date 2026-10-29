//! Typed native transport; algorithms and bounds remain in Core.
use cube_core::{
    composition::structured as s,
    errors::{ErrorCode, OperationError},
};
use pyo3::{
    prelude::*,
    types::{PyInt, PyList},
};
fn parsed(text: &str, operation: &'static str) -> PyResult<serde_json::Value> {
    s::parse_json(text).map_err(|e| super::file_error(e.operation(operation)))
}
#[pyfunction]
fn parse_json(py: Python<'_>, document: &str) -> PyResult<Py<PyAny>> {
    super::structured(py, &s::parse_json(document).map_err(super::file_error)?)
}
#[pyfunction]
#[pyo3(signature=(document,pointer=""))]
fn select_field(py: Python<'_>, document: &str, pointer: &str) -> PyResult<Py<PyAny>> {
    super::structured(
        py,
        &s::select_field(&parsed(document, "json.select")?, pointer).map_err(super::file_error)?,
    )
}
#[pyfunction]
fn json_text(document: &str) -> PyResult<String> {
    s::json_text(&parsed(document, "json.text")?).map_err(super::file_error)
}
#[pyfunction]
fn parse_integer(text: &str) -> PyResult<i64> {
    s::parse_integer(text).map_err(super::file_error)
}
fn integer(value: &Bound<'_, PyAny>, operation: &'static str) -> PyResult<i64> {
    if !value.is_exact_instance_of::<PyInt>() {
        return Err(super::file_error(
            OperationError::new(
                ErrorCode::InvalidInput,
                "expected a builtin integer, not Boolean, float or text",
            )
            .operation(operation),
        ));
    }
    value.extract::<i64>().map_err(|_| {
        super::file_error(
            OperationError::new(
                ErrorCode::ResourceLimit,
                "integer exceeds fixed-width transport",
            )
            .operation(operation),
        )
    })
}
#[pyfunction]
fn maximum(values: &Bound<'_, PyList>) -> PyResult<i64> {
    if values.len() > s::MAX_NODES {
        return Err(super::file_error(
            OperationError::new(ErrorCode::ResourceLimit, "integer list exceeds 4096 items")
                .operation("number.maximum"),
        ));
    }
    s::maximum(
        &values
            .iter()
            .map(|v| integer(&v, "number.maximum"))
            .collect::<PyResult<Vec<_>>>()?,
    )
    .map_err(super::file_error)
}
#[pyfunction]
fn add_integers(left: &Bound<'_, PyAny>, right: &Bound<'_, PyAny>) -> PyResult<i64> {
    s::add(integer(left, "number.add")?, integer(right, "number.add")?).map_err(super::file_error)
}
#[pyfunction]
fn image_dimensions(
    py: Python<'_>,
    image: PyRef<'_, super::image::PyImage>,
) -> PyResult<Py<PyAny>> {
    super::structured(
        py,
        &serde_json::json!(cube_core::image::dimensions::inspect(&image.inner)),
    )
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(parse_json, module)?)?;
    module.add_function(wrap_pyfunction!(select_field, module)?)?;
    module.add_function(wrap_pyfunction!(json_text, module)?)?;
    module.add_function(wrap_pyfunction!(parse_integer, module)?)?;
    module.add_function(wrap_pyfunction!(maximum, module)?)?;
    module.add_function(wrap_pyfunction!(add_integers, module)?)?;
    module.add_function(wrap_pyfunction!(image_dimensions, module)?)?;
    Ok(())
}
