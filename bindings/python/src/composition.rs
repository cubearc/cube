//! Thin native Python wrappers for ordered composition.
use pyo3::prelude::*;
#[pyfunction]
#[pyo3(signature = (left, right, separator=""))]
fn concat_text(left: &str, right: &str, separator: &str) -> PyResult<String> {
    cube_core::composition::concat_text(left, right, separator).map_err(super::file_error)
}
#[pyfunction]
fn concat_bytes<'py>(
    py: Python<'py>,
    left: &[u8],
    right: &[u8],
) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
    Ok(pyo3::types::PyBytes::new(
        py,
        &cube_core::composition::concat_bytes(left, right).map_err(super::file_error)?,
    ))
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(self::concat_text, module)?)?;
    module.add_function(wrap_pyfunction!(self::concat_bytes, module)?)?;
    Ok(())
}
