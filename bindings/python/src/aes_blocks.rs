//! Borrow binary inputs and keys; all block work and validation belong to Core.
use cube_core::ciphers::aes_blocks;
use pyo3::{prelude::*, types::PyBytes};
use zeroize::Zeroizing;
#[pyfunction]
fn aes_encrypt_blocks<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
) -> PyResult<Bound<'py, PyBytes>> {
    let output = Zeroizing::new(aes_blocks::encrypt(data, key).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
#[pyfunction]
fn aes_decrypt_blocks<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
) -> PyResult<Bound<'py, PyBytes>> {
    let output = Zeroizing::new(aes_blocks::decrypt(data, key).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(aes_encrypt_blocks, module)?)?;
    module.add_function(wrap_pyfunction!(aes_decrypt_blocks, module)?)
}
