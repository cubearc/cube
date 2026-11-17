//! Native key transport only; Core owns wrapping, bounds and integrity checks.
use cube_core::ciphers::key_wrap;
use pyo3::{prelude::*, types::PyBytes};
use zeroize::Zeroizing;
#[pyfunction]
fn aes_key_wrap<'py>(
    py: Python<'py>,
    data: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let s = key_wrap::Settings::parse(parameters_json).map_err(super::file_error)?;
    let output = Zeroizing::new(key_wrap::wrap(data, &s).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
#[pyfunction]
fn aes_key_unwrap<'py>(
    py: Python<'py>,
    data: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let s = key_wrap::Settings::parse(parameters_json).map_err(super::file_error)?;
    let output = Zeroizing::new(key_wrap::unwrap(data, &s).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(aes_key_wrap, module)?)?;
    module.add_function(wrap_pyfunction!(aes_key_unwrap, module)?)
}
