//! Typed native transport; Core owns word indexing and phrase ambiguity.
use cube_core::ciphers::{book::words as a, mono::key_phrase as k, poly};
use pyo3::{prelude::*, types::PyDict};
fn parse_parameters<T: serde::de::DeserializeOwned>(
    py: Python<'_>,
    value: &Bound<'_, PyDict>,
    op: &'static str,
) -> PyResult<T> {
    poly::parse_parameters(&super::mono::document(
        py,
        value.as_any(),
        65536,
        op,
        "parameters",
    )?)
    .map_err(|e| super::file_error(e.operation(op)))
}
fn phrase(
    py: Python<'_>,
    text: &str,
    value: &Bound<'_, PyDict>,
    decode: bool,
) -> PyResult<Py<PyAny>> {
    let op = if decode {
        "cipher.key_phrase.decode"
    } else {
        "cipher.key_phrase.encode"
    };
    let p: k::Parameters = parse_parameters(py, value, op)?;
    let result = if decode {
        serde_json::to_value(k::decode(text, &p).map_err(super::file_error)?)
    } else {
        serde_json::to_value(k::encode(text, &p).map_err(super::file_error)?)
    }
    .expect("typed report");
    super::structured(py, &result)
}
#[pyfunction]
fn key_phrase_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    phrase(py, text, parameters, false)
}
#[pyfunction]
fn key_phrase_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    phrase(py, text, parameters, true)
}
#[pyfunction]
fn arnold_encode(
    py: Python<'_>,
    pages: &Bound<'_, PyAny>,
    words: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let op = "cipher.arnold.encode";
    let p: a::Parameters = parse_parameters(py, parameters, op)?;
    let pages = super::mono::input::<Vec<String>>(py, pages, 1048576, op, "pages")?;
    let words = super::mono::input::<Vec<String>>(py, words, 1048576, op, "words")?;
    let result = a::encode(&pages, &words, &p).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn arnold_decode(
    py: Python<'_>,
    pages: &Bound<'_, PyAny>,
    coordinates: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let op = "cipher.arnold.decode";
    let p: a::Parameters = parse_parameters(py, parameters, op)?;
    let pages = super::mono::input::<Vec<String>>(py, pages, 1048576, op, "pages")?;
    let coordinates =
        super::mono::input::<Vec<a::Coordinate>>(py, coordinates, 1048576, op, "coordinates")?;
    let result = a::decode(&pages, &coordinates, &p).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(key_phrase_encode, module)?)?;
    module.add_function(wrap_pyfunction!(key_phrase_decode, module)?)?;
    module.add_function(wrap_pyfunction!(arnold_encode, module)?)?;
    module.add_function(wrap_pyfunction!(arnold_decode, module)?)?;
    Ok(())
}
