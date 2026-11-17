//! Byte/key adapters only; Core owns the raw group schemes and validation.
use cube_core::asymmetric::group_encryption as g;
use pyo3::{prelude::*, types::PyBytes};
use zeroize::Zeroizing;
#[pyfunction]
fn elgamal_encrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let s = g::ElGamalEncryptionSettings::parse(parameters_json, key).map_err(super::file_error)?;
    let output = Zeroizing::new(g::elgamal_encrypt(data, &s).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
#[pyfunction]
fn elgamal_decrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let s = g::ElGamalDecryptionSettings::parse(parameters_json, key).map_err(super::file_error)?;
    let output = Zeroizing::new(g::elgamal_decrypt(data, &s).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
#[pyfunction]
fn cramer_shoup_encrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let s =
        g::CramerShoupEncryptionSettings::parse(parameters_json, key).map_err(super::file_error)?;
    let output = Zeroizing::new(g::cramer_shoup_encrypt(data, &s).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
#[pyfunction]
fn cramer_shoup_decrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let s =
        g::CramerShoupDecryptionSettings::parse(parameters_json, key).map_err(super::file_error)?;
    let output = Zeroizing::new(g::cramer_shoup_decrypt(data, &s).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(elgamal_encrypt, module)?)?;
    module.add_function(wrap_pyfunction!(elgamal_decrypt, module)?)?;
    module.add_function(wrap_pyfunction!(cramer_shoup_encrypt, module)?)?;
    module.add_function(wrap_pyfunction!(cramer_shoup_decrypt, module)?)?;
    Ok(())
}
