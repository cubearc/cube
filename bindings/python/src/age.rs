//! Authenticated age adapter only; Core owns bounds and complete verification.
use cube_core::ciphers::age;
use pyo3::{prelude::*, types::PyBytes};
use zeroize::Zeroizing;
#[pyfunction]
fn age_decrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let s = age::Settings::parse(parameters_json).map_err(super::file_error)?;
    let output = Zeroizing::new(age::decrypt(data, &s).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(age_decrypt, module)?)
}
