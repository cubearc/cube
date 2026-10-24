//! Python boundary only: all algorithms and input validation live in cube-core.

use cube_core::{binary, ciphers, encoding, hashing};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

fn invalid_input(error: cube_core::Error) -> PyErr {
    PyValueError::new_err(error.to_string())
}

/// Encode bytes as padded standard Base32 using uppercase letters.
#[pyfunction]
fn base32_encode(data: &[u8]) -> String {
    encoding::base32_encode(data)
}

/// Decode strict uppercase Base32 with canonical padding. Invalid input raises ValueError.
#[pyfunction]
fn base32_decode<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, PyBytes>> {
    let bytes = encoding::base32_decode(text).map_err(invalid_input)?;
    Ok(PyBytes::new(py, &bytes))
}

/// Encode bytes as Bitcoin Base58, preserving leading zero bytes as 1.
#[pyfunction]
fn base58_encode(data: &[u8]) -> String {
    encoding::base58_encode(data)
}

/// Decode strict Bitcoin Base58 to bytes. Invalid input raises ValueError.
#[pyfunction]
fn base58_decode<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, PyBytes>> {
    let bytes = encoding::base58_decode(text).map_err(invalid_input)?;
    Ok(PyBytes::new(py, &bytes))
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

/// Encode text by an ASCII Caesar shift from -2147483648 to 2147483647.
///
/// Shifts are normalized modulo 26; case and non-ASCII characters are preserved
/// without normalization. This is an educational cipher, not secure encryption.
/// Out-of-range integers raise OverflowError; incorrect types raise TypeError.
#[pyfunction]
fn caesar_encode(text: &str, shift: i32) -> String {
    ciphers::caesar_encode(text, shift)
}

/// Decode text with the same signed 32-bit shift used for Caesar encoding.
///
/// Only ASCII letters rotate. All other characters are preserved unchanged.
/// The inverse is safe for every shift, including -2147483648.
/// Out-of-range integers raise OverflowError; incorrect types raise TypeError.
#[pyfunction]
fn caesar_decode(text: &str, shift: i32) -> String {
    ciphers::caesar_decode(text, shift)
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
    module.add_function(wrap_pyfunction!(base32_encode, module)?)?;
    module.add_function(wrap_pyfunction!(base32_decode, module)?)?;
    module.add_function(wrap_pyfunction!(base58_encode, module)?)?;
    module.add_function(wrap_pyfunction!(base58_decode, module)?)?;
    module.add_function(wrap_pyfunction!(base64_encode, module)?)?;
    module.add_function(wrap_pyfunction!(base64_decode, module)?)?;
    module.add_function(wrap_pyfunction!(hex_encode, module)?)?;
    module.add_function(wrap_pyfunction!(hex_decode, module)?)?;
    module.add_function(wrap_pyfunction!(caesar_encode, module)?)?;
    module.add_function(wrap_pyfunction!(caesar_decode, module)?)?;
    module.add_function(wrap_pyfunction!(rot13, module)?)?;
    module.add_function(wrap_pyfunction!(sha256, module)?)?;
    module.add_function(wrap_pyfunction!(xor, module)?)?;
    Ok(())
}
