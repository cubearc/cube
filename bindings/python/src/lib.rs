//! Python boundary only: all algorithms and input validation live in cube-core.

use cube_core::{binary, ciphers, encoding, hashing};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

fn invalid_input(error: cube_core::Error) -> PyErr {
    PyValueError::new_err(error.to_string())
}

/// Encode bytes as padded standard Base64.
#[pyfunction]
fn base64_encode(data: &[u8]) -> String {
    encoding::base64_encode(data)
}

/// Decode strict, padded standard Base64. Invalid input raises ValueError.
#[pyfunction]
fn base64_decode<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, PyBytes>> {
    let bytes = encoding::base64_decode(text).map_err(invalid_input)?;
    Ok(PyBytes::new(py, &bytes))
}

/// Encode bytes as lowercase hexadecimal digits.
#[pyfunction]
fn hex_encode(data: &[u8]) -> String {
    encoding::hex_encode(data)
}

/// Decode hexadecimal digits in either case. Invalid input raises ValueError.
#[pyfunction]
fn hex_decode<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, PyBytes>> {
    let bytes = encoding::hex_decode(text).map_err(invalid_input)?;
    Ok(PyBytes::new(py, &bytes))
}

/// Rotate ASCII letters by 13, preserving all other characters.
#[pyfunction]
fn rot13(text: &str) -> String {
    ciphers::rot13(text)
}

/// Compute SHA-256 as 64 lowercase hexadecimal digits.
#[pyfunction]
fn sha256(data: &[u8]) -> String {
    hashing::sha256(data)
}

/// XOR bytes with a repeating key. An empty key raises ValueError.
#[pyfunction]
fn xor<'py>(py: Python<'py>, data: &[u8], key: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
    let bytes = binary::xor(data, key).map_err(invalid_input)?;
    Ok(PyBytes::new(py, &bytes))
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(base64_encode, module)?)?;
    module.add_function(wrap_pyfunction!(base64_decode, module)?)?;
    module.add_function(wrap_pyfunction!(hex_encode, module)?)?;
    module.add_function(wrap_pyfunction!(hex_decode, module)?)?;
    module.add_function(wrap_pyfunction!(rot13, module)?)?;
    module.add_function(wrap_pyfunction!(sha256, module)?)?;
    module.add_function(wrap_pyfunction!(xor, module)?)?;
    Ok(())
}
