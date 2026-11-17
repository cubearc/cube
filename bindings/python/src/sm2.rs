//! Native private-key/byte transport; Core owns SM2 and all profile validation.
use cube_core::asymmetric::sm2;
use pyo3::{prelude::*, types::PyBytes};
use zeroize::Zeroizing;
#[pyfunction]
fn sm2_encrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let settings =
        sm2::EncryptionSettings::parse(parameters_json, key).map_err(super::file_error)?;
    let output = Zeroizing::new(sm2::encrypt(data, &settings).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
#[pyfunction]
fn sm2_decrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let settings =
        sm2::DecryptionSettings::parse(parameters_json, key).map_err(super::file_error)?;
    let output = Zeroizing::new(sm2::decrypt(data, &settings).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(sm2_encrypt, module)?)?;
    module.add_function(wrap_pyfunction!(sm2_decrypt, module)?)?;
    Ok(())
}
