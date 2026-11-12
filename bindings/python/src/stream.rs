//! Native byte transport only; profile semantics and bounds belong to Core.
use cube_core::{
    ciphers::stream,
    errors::{ErrorCode, OperationError},
};
use pyo3::{prelude::*, types::PyBytes};
use zeroize::Zeroizing;
#[pyfunction]
fn stream_transform<'py>(
    py: Python<'py>,
    profile: &str,
    data: &[u8],
    key: &[u8],
    nonce: &[u8],
    counter: &Bound<'_, PyAny>,
) -> PyResult<Bound<'py, PyBytes>> {
    let id = format!("cipher.{profile}");
    let p = stream::Profile::from_id(&id).ok_or_else(|| {
        super::file_error(
            OperationError::new(
                ErrorCode::InvalidParameter,
                "unsupported exact stream profile",
            )
            .parameter("profile"),
        )
    })?;
    let counter = super::modular::operand(counter, p.id(), "counter")?;
    let output =
        Zeroizing::new(stream::transform(p, data, key, nonce, counter).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
#[pyfunction]
fn vernam<'py>(py: Python<'py>, data: &[u8], pad: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
    let output = Zeroizing::new(stream::vernam(data, pad).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
#[pyfunction]
fn stream_encrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let settings =
        stream::reference::Settings::parse(parameters_json).map_err(super::file_error)?;
    let output =
        Zeroizing::new(stream::reference::encrypt(data, &settings).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
#[pyfunction]
fn stream_decrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let settings =
        stream::reference::Settings::parse(parameters_json).map_err(super::file_error)?;
    let output =
        Zeroizing::new(stream::reference::decrypt(data, &settings).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(stream_encrypt, module)?)?;
    module.add_function(wrap_pyfunction!(stream_decrypt, module)?)?;
    module.add_function(wrap_pyfunction!(stream_transform, module)?)?;
    module.add_function(wrap_pyfunction!(vernam, module)?)
}
