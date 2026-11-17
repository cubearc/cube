//! Native private-key/byte transport; Core owns RSA and all profile validation.
use cube_core::asymmetric::rsa;
use pyo3::{prelude::*, types::PyBytes};
use zeroize::Zeroizing;
#[pyfunction]
fn rsa_oaep_encrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let settings =
        rsa::EncryptionSettings::parse(parameters_json, key).map_err(super::file_error)?;
    let output = Zeroizing::new(rsa::encrypt(data, &settings).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
#[pyfunction]
fn rsa_oaep_decrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let settings =
        rsa::DecryptionSettings::parse(parameters_json, key).map_err(super::file_error)?;
    let output = Zeroizing::new(rsa::decrypt(data, &settings).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
#[pyfunction]
fn rsa_pkcs1_v1_5_decrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let settings = rsa::LegacySettings::parse(parameters_json, key).map_err(super::file_error)?;
    let output = Zeroizing::new(rsa::legacy_decrypt(data, &settings).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(rsa_oaep_encrypt, module)?)?;
    module.add_function(wrap_pyfunction!(rsa_oaep_decrypt, module)?)?;
    module.add_function(wrap_pyfunction!(rsa_pkcs1_v1_5_decrypt, module)?)?;
    Ok(())
}
