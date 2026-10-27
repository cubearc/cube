//! Thin native adapters; source/geometry/token algorithms stay in Core.
use cube_core::{
    ciphers::{book, poly},
    symbols::tokens,
    text::keyboard::coordinates as keyboard,
};
use pyo3::{
    prelude::*,
    types::{PyAny, PyDict, PyModule},
};
#[pyfunction]
fn book_encode(
    py: Python<'_>,
    source: &str,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.book.encode",
        "parameters",
    )?;
    let p: book::EncodeOptions = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.book.encode")))?;
    let result = book::encode(source, text, &p).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn keyboard_coordinates_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "text.keyboard.coordinates.encode",
        "parameters",
    )?;
    let p: keyboard::Options = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("text.keyboard.coordinates.encode")))?;
    let result = keyboard::encode(text, &p).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn keyboard_coordinates_decode(
    py: Python<'_>,
    coordinates: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "text.keyboard.coordinates.decode",
        "parameters",
    )?;
    let p: keyboard::Options = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("text.keyboard.coordinates.decode")))?;
    let document = super::mono::document(
        py,
        coordinates,
        1048576,
        "text.keyboard.coordinates.decode",
        "coordinates",
    )?;
    let coordinates: Vec<keyboard::Coordinate> = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("text.keyboard.coordinates.decode")))?;
    let result = keyboard::decode(&coordinates, &p).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn keyboard_shift_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "text.keyboard.shift.encode",
        "parameters",
    )?;
    let p: keyboard::Shift = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("text.keyboard.shift.encode")))?;
    let result = keyboard::shift(text, &p, false).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn keyboard_shift_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "text.keyboard.shift.decode",
        "parameters",
    )?;
    let p: keyboard::Shift = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("text.keyboard.shift.decode")))?;
    let result = keyboard::shift(text, &p, true).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn symbol_tokens_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "symbol.tokens.decode",
        "parameters",
    )?;
    let p: tokens::Parameters = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("symbol.tokens.decode")))?;
    let result = tokens::transform(text, &p, false).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn symbol_tokens_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "symbol.tokens.encode",
        "parameters",
    )?;
    let p: tokens::Parameters = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("symbol.tokens.encode")))?;
    let result = tokens::transform(text, &p, true).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(book_encode, module)?)?;
    module.add_function(wrap_pyfunction!(keyboard_coordinates_encode, module)?)?;
    module.add_function(wrap_pyfunction!(keyboard_coordinates_decode, module)?)?;
    module.add_function(wrap_pyfunction!(keyboard_shift_encode, module)?)?;
    module.add_function(wrap_pyfunction!(keyboard_shift_decode, module)?)?;
    module.add_function(wrap_pyfunction!(symbol_tokens_decode, module)?)?;
    module.add_function(wrap_pyfunction!(symbol_tokens_encode, module)?)?;
    Ok(())
}
