//! Secret file-format adapter only; all derivation and decryption lives in Core.
use cube_core::ciphers::openssl;
use pyo3::{prelude::*, types::PyBytes};
use zeroize::Zeroizing;
#[pyfunction]
fn openssl_decrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let s = openssl::Settings::parse(parameters_json).map_err(super::file_error)?;
    let output = Zeroizing::new(openssl::decrypt(data, &s).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(openssl_decrypt, module)?)
}
