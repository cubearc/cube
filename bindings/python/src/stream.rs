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
/// Encrypt a raw profile; packed-bit profiles require data_bits and zero unused bits.
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
/// Decrypt the same raw profile and packed bit count; output has no authentication.
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

/// Return exact packed ciphertext and detached original Grain128a MAC bytes.
#[pyfunction]
fn stream_auth_encrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    parameters_json: &str,
) -> PyResult<(Bound<'py, PyBytes>, Bound<'py, PyBytes>)> {
    let settings = stream::reference::authentication::Settings::parse(parameters_json)
        .map_err(super::file_error)?;
    let result =
        stream::reference::authentication::encrypt(data, &settings).map_err(super::file_error)?;
    let ciphertext = Zeroizing::new(result.ciphertext);
    let tag = Zeroizing::new(result.tag);
    Ok((PyBytes::new(py, &ciphertext), PyBytes::new(py, &tag)))
}
/// Verify the detached tag before returning any plaintext bytes.
#[pyfunction]
fn stream_auth_decrypt<'py>(
    py: Python<'py>,
    data: &[u8],
    tag: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let settings = stream::reference::authentication::Settings::parse(parameters_json)
        .map_err(super::file_error)?;
    let result = Zeroizing::new(
        stream::reference::authentication::decrypt(data, tag, &settings)
            .map_err(super::file_error)?,
    );
    Ok(PyBytes::new(py, &result))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(stream_auth_encrypt, module)?)?;
    module.add_function(wrap_pyfunction!(stream_auth_decrypt, module)?)?;
    module.add_function(wrap_pyfunction!(stream_encrypt, module)?)?;
    module.add_function(wrap_pyfunction!(stream_decrypt, module)?)?;
    module.add_function(wrap_pyfunction!(stream_transform, module)?)?;
    module.add_function(wrap_pyfunction!(vernam, module)?)
}
