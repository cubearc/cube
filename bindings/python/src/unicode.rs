//! Native Unicode observations and exact/normalized scalar comparison.
use cube_core::{
    errors::{ErrorCode, OperationError},
    text::unicode::{self, Options},
};
use pyo3::{prelude::*, types::PyDict};
fn options(py: Python<'_>, options: Option<&Bound<'_, PyDict>>) -> PyResult<Options> {
    options
        .map(|o| {
            super::puzzles::document(py, o).and_then(|s| {
                serde_json::from_str(&s).map_err(|_| {
                    super::file_error(
                        OperationError::new(
                            ErrorCode::InvalidParameter,
                            "invalid concrete Unicode listing options",
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
#[pyo3(signature=(text,*,options=None))]
fn inspect_unicode(
    py: Python<'_>,
    text: &str,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    let report = unicode::inspect(text, &self::options(py, options)?).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(report))
}
#[pyfunction]
#[pyo3(signature=(left,right,*,normalization="none",options=None))]
fn compare_unicode(
    py: Python<'_>,
    left: &str,
    right: &str,
    normalization: &str,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    let report = unicode::compare(left, right, normalization, &self::options(py, options)?)
        .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(report))
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(inspect_unicode, module)?)?;
    module.add_function(wrap_pyfunction!(compare_unicode, module)?)?;
    Ok(())
}
