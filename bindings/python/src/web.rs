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
fn asset(py: Python<'_>, value: cube_core::web::assets::Asset) -> PyResult<Py<PyAny>> {
    let result = super::structured(py, &serde_json::json!(value))?;
    result
        .bind(py)
        .set_item("data", pyo3::types::PyBytes::new(py, &value.data))?;
    Ok(result)
}
#[pyfunction]
#[pyo3(signature=(document,index,*,selector="inline"))]
fn extract_html_asset(
    py: Python<'_>,
    document: &str,
    index: usize,
    selector: &str,
) -> PyResult<Py<PyAny>> {
    let selector =
        cube_core::web::assets::HtmlSelector::parse(selector).map_err(super::file_error)?;
    asset(
        py,
        cube_core::web::assets::html(document, index, selector).map_err(super::file_error)?,
    )
}
#[pyfunction]
fn extract_har_asset(py: Python<'_>, document: &str, index: usize) -> PyResult<Py<PyAny>> {
    asset(
        py,
        cube_core::web::assets::har(document, index).map_err(super::file_error)?,
    )
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(extract_html_asset, module)?)?;
    module.add_function(wrap_pyfunction!(extract_har_asset, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_html, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_har, module)?)?;
    Ok(())
}
