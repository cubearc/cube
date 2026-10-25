//! Native adapters for classical ciphers and explicit text/symbol operations.
use cube_core::ciphers;
use pyo3::prelude::*;
#[pyfunction]
fn semaphore_encode(text: &str) -> PyResult<String> {
    cube_core::symbols::semaphore::encode(text).map_err(super::file_error)
}
#[pyfunction]
fn semaphore_decode(text: &str) -> PyResult<String> {
    cube_core::symbols::semaphore::decode(text).map_err(super::file_error)
}
#[pyfunction]
#[pyo3(signature=(text, *, drop_whitespace=false, max_candidates=128))]
fn grid_factors(
    py: Python<'_>,
    text: &str,
    drop_whitespace: bool,
    max_candidates: usize,
) -> PyResult<Py<PyAny>> {
    let result = cube_core::text::grid::factors(text, drop_whitespace, max_candidates)
        .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(result))
}
#[pyfunction]
#[pyo3(signature=(text, width, *, drop_whitespace=false, traversal="row", colors=None))]
fn text_grid(
    py: Python<'_>,
    text: &str,
    width: u32,
    drop_whitespace: bool,
    traversal: &str,
    colors: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<Py<PyAny>> {
    use cube_core::{image::Traversal, text::grid};
    let grid = grid::render(
        text,
        width,
        drop_whitespace,
        Traversal::parse(traversal).map_err(super::file_error)?,
    )
    .map_err(super::file_error)?;
    let result = super::structured(py, &serde_json::json!(grid))?;
    result
        .bind(py)
        .set_item("text", grid.text().map_err(super::file_error)?)?;
    if let Some(colors) = colors {
        if colors.len() > 256 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "mapping count exceeds 256",
            ));
        }
        let colors = colors
            .iter()
            .map(|(key, value)| {
                if key.len()? != 1 || value.len()? != 4 {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "mapping requires a scalar and four RGBA bytes",
                    ));
                }
                Ok((key.extract::<String>()?, value.extract::<[u8; 4]>()?))
            })
            .collect::<PyResult<Vec<_>>>()?;
        let png = grid
            .image(&colors)
            .map_err(super::file_error)?
            .encode_png()
            .map_err(super::file_error)?;
        result
            .bind(py)
            .set_item("png", pyo3::types::PyBytes::new(py, &png))?;
    } else {
        result.bind(py).set_item("png", py.None())?;
    }
    Ok(result)
}
#[pyfunction]
#[pyo3(signature=(text, *, ngram_size=1, normalization="none", case="preserve", filter="all", max_results=1000))]
fn text_statistics(
    py: Python<'_>,
    text: &str,
    ngram_size: usize,
    normalization: &str,
    case: &str,
    filter: &str,
    max_results: usize,
) -> PyResult<Py<PyAny>> {
    use cube_core::text::statistics::{self, CaseMode, Normalization, Options, TextFilter};
    let report = statistics::analyze(
        text,
        Options {
            ngram_size,
            normalization: Normalization::parse(normalization).map_err(super::file_error)?,
            case: CaseMode::parse(case).map_err(super::file_error)?,
            filter: TextFilter::parse(filter).map_err(super::file_error)?,
            max_results,
        },
    )
    .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(report))
}
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
    module.add_function(wrap_pyfunction!(semaphore_encode, module)?)?;
    module.add_function(wrap_pyfunction!(semaphore_decode, module)?)?;
    module.add_function(wrap_pyfunction!(grid_factors, module)?)?;
    module.add_function(wrap_pyfunction!(text_grid, module)?)?;
    module.add_function(wrap_pyfunction!(text_statistics, module)?)?;
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
