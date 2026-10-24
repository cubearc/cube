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
#[pyfunction]
fn normalize_text(text: &str, form: &str) -> PyResult<String> {
    cube_core::text::normalize(text, form).map_err(super::file_error)
}
#[pyfunction]
fn reverse_text(text: &str) -> PyResult<String> {
    cube_core::text::reverse(text).map_err(super::file_error)
}
#[pyfunction]
#[pyo3(signature=(text, *, indices=None, start=None, stride=None, stop=None))]
fn extract_characters(
    text: &str,
    indices: Option<Vec<usize>>,
    start: Option<usize>,
    stride: Option<usize>,
    stop: Option<usize>,
) -> PyResult<String> {
    cube_core::text::extract(text, indices.as_deref(), start, stride, stop)
        .map_err(super::file_error)
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("UNICODE_VERSION", cube_core::text::UNICODE_VERSION)?;
    module.add_function(wrap_pyfunction!(normalize_text, module)?)?;
    module.add_function(wrap_pyfunction!(reverse_text, module)?)?;
    module.add_function(wrap_pyfunction!(extract_characters, module)?)?;
    module.add_function(wrap_pyfunction!(atbash, module)?)?;
    module.add_function(wrap_pyfunction!(vigenere_encode, module)?)?;
    module.add_function(wrap_pyfunction!(vigenere_decode, module)?)?;
    module.add_function(wrap_pyfunction!(beaufort, module)?)?;
    module.add_function(wrap_pyfunction!(affine_encode, module)?)?;
    module.add_function(wrap_pyfunction!(affine_decode, module)?)?;
    Ok(())
}
