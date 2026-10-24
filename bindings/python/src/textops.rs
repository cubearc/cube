//! Native adapters for classical ciphers and explicit text/symbol operations.
use cube_core::ciphers;
use pyo3::prelude::*;
#[pyfunction]
fn atbash(text: &str) -> PyResult<String> {
    ciphers::atbash(text).map_err(super::file_error)
}
#[pyfunction]
fn vigenere_encode(text: &str, key: &str) -> PyResult<String> {
    ciphers::vigenere_encode(text, key).map_err(super::file_error)
}
#[pyfunction]
fn vigenere_decode(text: &str, key: &str) -> PyResult<String> {
    ciphers::vigenere_decode(text, key).map_err(super::file_error)
}
#[pyfunction]
fn beaufort(text: &str, key: &str) -> PyResult<String> {
    ciphers::beaufort(text, key).map_err(super::file_error)
}
#[pyfunction]
fn affine_encode(text: &str, a: i32, b: i32) -> PyResult<String> {
    ciphers::affine_encode(text, a, b).map_err(super::file_error)
}
#[pyfunction]
fn affine_decode(text: &str, a: i32, b: i32) -> PyResult<String> {
    ciphers::affine_decode(text, a, b).map_err(super::file_error)
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(atbash, module)?)?;
    module.add_function(wrap_pyfunction!(vigenere_encode, module)?)?;
    module.add_function(wrap_pyfunction!(vigenere_decode, module)?)?;
    module.add_function(wrap_pyfunction!(beaufort, module)?)?;
    module.add_function(wrap_pyfunction!(affine_encode, module)?)?;
    module.add_function(wrap_pyfunction!(affine_decode, module)?)?;
    Ok(())
}
