//! Native adapters for the three qualified Vigenere key profiles.
use cube_core::ciphers::poly::{self, key_profiles};
use pyo3::{prelude::*, types::PyDict};

#[pyfunction]
fn interrupted_key_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.interrupted_key.encode",
        "parameters",
    )?;
    let p: key_profiles::InterruptedKey = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.interrupted_key.encode")))?;
    key_profiles::interrupted_key(text, &p, false).map_err(super::file_error)
}

#[pyfunction]
fn interrupted_key_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.interrupted_key.decode",
        "parameters",
    )?;
    let p: key_profiles::InterruptedKey = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.interrupted_key.decode")))?;
    key_profiles::interrupted_key(text, &p, true).map_err(super::file_error)
}

#[pyfunction]
fn progressive_key_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.progressive_key.encode",
        "parameters",
    )?;
    let p: key_profiles::ProgressiveKey = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.progressive_key.encode")))?;
    key_profiles::progressive_key(text, &p, false).map_err(super::file_error)
}

#[pyfunction]
fn progressive_key_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.progressive_key.decode",
        "parameters",
    )?;
    let p: key_profiles::ProgressiveKey = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.progressive_key.decode")))?;
    key_profiles::progressive_key(text, &p, true).map_err(super::file_error)
}

#[pyfunction]
fn vigenere_multiplicative_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.vigenere_multiplicative.encode",
        "parameters",
    )?;
    let p: key_profiles::Multiplicative = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.vigenere_multiplicative.encode")))?;
    key_profiles::multiplicative(text, &p, false).map_err(super::file_error)
}

#[pyfunction]
fn vigenere_multiplicative_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.vigenere_multiplicative.decode",
        "parameters",
    )?;
    let p: key_profiles::Multiplicative = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.vigenere_multiplicative.decode")))?;
    key_profiles::multiplicative(text, &p, true).map_err(super::file_error)
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(interrupted_key_encode, module)?)?;
    module.add_function(wrap_pyfunction!(interrupted_key_decode, module)?)?;
    module.add_function(wrap_pyfunction!(progressive_key_encode, module)?)?;
    module.add_function(wrap_pyfunction!(progressive_key_decode, module)?)?;
    module.add_function(wrap_pyfunction!(vigenere_multiplicative_encode, module)?)?;
    module.add_function(wrap_pyfunction!(vigenere_multiplicative_decode, module)?)?;
    Ok(())
}
