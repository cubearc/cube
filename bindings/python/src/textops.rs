//! Native adapters for classical ciphers and explicit text/symbol operations.
use cube_core::ciphers;
use pyo3::prelude::*;
#[pyfunction]
fn select_text(text: &str, selector: &str) -> PyResult<String> {
    cube_core::text::selection::select(
        text,
        cube_core::text::selection::Selector::parse(selector).map_err(super::file_error)?,
    )
    .map_err(super::file_error)
}
#[pyfunction]
fn a1z26_encode(text: &str) -> PyResult<String> {
    cube_core::symbols::a1z26::encode(text).map_err(super::file_error)
}
#[pyfunction]
fn a1z26_decode(text: &str) -> PyResult<String> {
    cube_core::symbols::a1z26::decode(text).map_err(super::file_error)
}
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
#[pyfunction]
fn morse_encode(text: &str) -> PyResult<String> {
    cube_core::symbols::morse_encode(text).map_err(super::file_error)
}
#[pyfunction]
fn morse_decode(text: &str) -> PyResult<String> {
    cube_core::symbols::morse_decode(text).map_err(super::file_error)
}
#[pyfunction]
fn braille_encode(data: &[u8]) -> PyResult<String> {
    cube_core::symbols::braille_encode(data).map_err(super::file_error)
}
#[pyfunction]
fn braille_decode<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
    Ok(pyo3::types::PyBytes::new(
        py,
        &cube_core::symbols::braille_decode(text).map_err(super::file_error)?,
    ))
}
#[pyfunction]
fn convert_base(text: &str, from_base: u32, to_base: u32) -> PyResult<String> {
    cube_core::numbers::convert_base(text, from_base, to_base).map_err(super::file_error)
}
#[pyfunction]
fn utf8_encode<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
    Ok(pyo3::types::PyBytes::new(
        py,
        &cube_core::text::to_utf8(text).map_err(super::file_error)?,
    ))
}
#[pyfunction]
fn utf8_decode(data: &[u8]) -> PyResult<String> {
    cube_core::text::from_utf8(data).map_err(super::file_error)
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(select_text, module)?)?;
    module.add(
        "WORD_UNICODE_VERSION",
        cube_core::text::selection::WORD_UNICODE_VERSION,
    )?;
    module.add_function(wrap_pyfunction!(a1z26_encode, module)?)?;
    module.add_function(wrap_pyfunction!(a1z26_decode, module)?)?;
    module.add_function(wrap_pyfunction!(utf8_encode, module)?)?;
    module.add_function(wrap_pyfunction!(utf8_decode, module)?)?;
    module.add_function(wrap_pyfunction!(convert_base, module)?)?;
    module.add_function(wrap_pyfunction!(morse_encode, module)?)?;
    module.add_function(wrap_pyfunction!(morse_decode, module)?)?;
    module.add_function(wrap_pyfunction!(braille_encode, module)?)?;
    module.add_function(wrap_pyfunction!(braille_decode, module)?)?;
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
