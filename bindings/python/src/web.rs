//! Native wrappers only: offline parsing stays in Rust and opens no references.
use pyo3::prelude::*;
#[pyfunction]
fn inspect_html(py: Python<'_>, document: &str) -> PyResult<Py<PyAny>> {
    let report = cube_core::web::html::inspect(document).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(report))
}
#[pyfunction]
fn inspect_har(py: Python<'_>, document: &str) -> PyResult<Py<PyAny>> {
    let report = cube_core::web::har::inspect(document).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(report))
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(inspect_html, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_har, module)?)?;
    Ok(())
}
