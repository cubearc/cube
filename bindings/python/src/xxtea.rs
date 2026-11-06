//! Native byte adapter for Core's complete XXTEA word sequences.
use cube_core::ciphers::xxtea::{self, Key};
use pyo3::{prelude::*, types::PyBytes};
use zeroize::Zeroizing;

#[pyfunction]
fn xxtea_encrypt_words<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
) -> PyResult<Bound<'py, PyBytes>> {
    let key = Key::new(key).map_err(super::file_error)?;
    let output = Zeroizing::new(xxtea::encrypt(data, &key).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
#[pyfunction]
fn xxtea_decrypt_words<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
) -> PyResult<Bound<'py, PyBytes>> {
    let key = Key::new(key).map_err(super::file_error)?;
    let output = Zeroizing::new(xxtea::decrypt(data, &key).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(xxtea_encrypt_words, module)?)?;
    module.add_function(wrap_pyfunction!(xxtea_decrypt_words, module)?)
}
