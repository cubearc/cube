//! Native private-key/byte transport; Core owns IES and all profile validation.
use cube_core::asymmetric::ies;
use pyo3::{prelude::*, types::PyBytes};
use zeroize::Zeroizing;
#[pyfunction]
fn ies_encrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let settings =
        ies::EncryptionSettings::parse(parameters_json, key).map_err(super::file_error)?;
    let output = Zeroizing::new(ies::encrypt(data, &settings).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
#[pyfunction]
fn ies_decrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let settings =
        ies::DecryptionSettings::parse(parameters_json, key).map_err(super::file_error)?;
    let output = Zeroizing::new(ies::decrypt(data, &settings).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(ies_encrypt, module)?)?;
    module.add_function(wrap_pyfunction!(ies_decrypt, module)?)?;
    Ok(())
}
