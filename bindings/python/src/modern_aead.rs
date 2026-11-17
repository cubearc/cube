//! Borrow exact byte inputs; every primitive and release decision belongs to Core.
use cube_core::ciphers::modern_aead as aead;
use pyo3::{prelude::*, types::PyBytes};
use zeroize::Zeroizing;
fn detached<'py>(
    py: Python<'py>,
    result: aead::Detached,
) -> (Bound<'py, PyBytes>, Bound<'py, PyBytes>) {
    let body = Zeroizing::new(result.ciphertext);
    let tag = Zeroizing::new(result.tag);
    (PyBytes::new(py, &body), PyBytes::new(py, &tag))
}
#[pyfunction]
fn aead_encrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    associated_data: &[u8],
    parameters_json: &str,
) -> PyResult<(Bound<'py, PyBytes>, Bound<'py, PyBytes>)> {
    let s = aead::Settings::parse(parameters_json).map_err(super::file_error)?;
    Ok(detached(
        py,
        aead::encrypt(data, associated_data, &s).map_err(super::file_error)?,
    ))
}
#[pyfunction]
fn aead_decrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    associated_data: &[u8],
    tag: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let s = aead::Settings::parse(parameters_json).map_err(super::file_error)?;
    let plain =
        Zeroizing::new(aead::decrypt(data, associated_data, tag, &s).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &plain))
}
#[pyfunction]
fn aes_siv_encrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    associated_data: Vec<Vec<u8>>,
    parameters_json: &str,
) -> PyResult<(Bound<'py, PyBytes>, Bound<'py, PyBytes>)> {
    let s = aead::SivSettings::parse(parameters_json).map_err(super::file_error)?;
    Ok(detached(
        py,
        aead::siv_encrypt(data, &associated_data, &s).map_err(super::file_error)?,
    ))
}
#[pyfunction]
fn aes_siv_decrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    associated_data: Vec<Vec<u8>>,
    tag: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let s = aead::SivSettings::parse(parameters_json).map_err(super::file_error)?;
    let plain = Zeroizing::new(
        aead::siv_decrypt(data, &associated_data, tag, &s).map_err(super::file_error)?,
    );
    Ok(PyBytes::new(py, &plain))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(aead_encrypt, module)?)?;
    module.add_function(wrap_pyfunction!(aead_decrypt, module)?)?;
    module.add_function(wrap_pyfunction!(aes_siv_encrypt, module)?)?;
    module.add_function(wrap_pyfunction!(aes_siv_decrypt, module)?)
}
