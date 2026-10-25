//! Native transport of the bounded core codec.
use cube_core::encoding::base58check;
use pyo3::{prelude::*, types::PyBytes};

#[pyfunction]
#[pyo3(signature=(data, *, version))]
fn base58check_encode(data: &[u8], version: u8) -> PyResult<String> {
    base58check::encode(version, data).map_err(super::file_error)
}
#[pyfunction]
fn base58check_decode(py: Python<'_>, text: &str) -> PyResult<Py<PyAny>> {
    let decoded = base58check::decode(text).map_err(super::file_error)?;
    let result = super::structured(py, &serde_json::json!(decoded))?;
    result
        .bind(py)
        .set_item("payload", PyBytes::new(py, &decoded.payload))?;
    Ok(result)
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(base58check_encode, module)?)?;
    module.add_function(wrap_pyfunction!(base58check_decode, module)?)?;
    Ok(())
}
