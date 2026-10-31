//! Native JSON adaptation; all historical algorithms and limits belong to Core.
use cube_core::ciphers::mono::historical as h;
use pyo3::prelude::*;
use serde_json::{Value, json};
fn value(py: Python<'_>, v: &Bound<'_, PyAny>, parameter: &'static str) -> PyResult<Value> {
    let document =
        super::mono::document(py, v, h::MAX_BYTES, "cipher.nomenclator.encode", parameter)?;
    h::parse(&document).map_err(super::file_error)
}
fn call(
    py: Python<'_>,
    operation: &'static str,
    inputs: Value,
    parameters: Value,
    port: &str,
) -> PyResult<Py<PyAny>> {
    let mut outputs = h::execute(operation, &inputs, &parameters).map_err(super::file_error)?;
    let document = serde_json::to_string(&outputs[port].take()).expect("typed JSON result");
    Ok(py
        .import("json")?
        .call_method1("loads", (document,))?
        .unbind())
}
#[pyfunction]
fn malespin(text: &str) -> PyResult<String> {
    h::malespin(text).map_err(super::file_error)
}
#[pyfunction]
#[pyo3(signature=(text,pairs,unmapped="reject"))]
fn vatsyayana(
    py: Python<'_>,
    text: &str,
    pairs: &Bound<'_, PyAny>,
    unmapped: &str,
) -> PyResult<String> {
    let p = h::parse(&json!({"pairs":value(py,pairs,"pairs")?,"unmapped":unmapped}).to_string())
        .map_err(super::file_error)?;
    h::vatsyayana(text, &p).map_err(super::file_error)
}
#[pyfunction]
fn nomenclator_encode(
    py: Python<'_>,
    units: &Bound<'_, PyAny>,
    definition: &Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    call(
        py,
        "cipher.nomenclator.encode",
        json!({"units":value(py,units,"units")?}),
        json!({"definition":value(py,definition,"definition")?}),
        "tokens",
    )
}
#[pyfunction]
fn nomenclator_decode(
    py: Python<'_>,
    tokens: &Bound<'_, PyAny>,
    definition: &Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    call(
        py,
        "cipher.nomenclator.decode",
        json!({"tokens":value(py,tokens,"tokens")?}),
        json!({"definition":value(py,definition,"definition")?}),
        "report",
    )
}
#[pyfunction]
fn syllabary_encode(
    py: Python<'_>,
    units: &Bound<'_, PyAny>,
    definition: &Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    call(
        py,
        "cipher.syllabary.encode",
        json!({"units":value(py,units,"units")?}),
        json!({"definition":value(py,definition,"definition")?}),
        "tokens",
    )
}
#[pyfunction]
fn syllabary_decode(
    py: Python<'_>,
    tokens: &Bound<'_, PyAny>,
    definition: &Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    call(
        py,
        "cipher.syllabary.decode",
        json!({"tokens":value(py,tokens,"tokens")?}),
        json!({"definition":value(py,definition,"definition")?}),
        "report",
    )
}
#[pyfunction]
fn great_cipher_encode(py: Python<'_>, units: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    call(
        py,
        "cipher.great_cipher.encode",
        json!({"units":value(py,units,"units")?}),
        json!({}),
        "tokens",
    )
}
#[pyfunction]
fn great_cipher_decode(py: Python<'_>, tokens: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    call(
        py,
        "cipher.great_cipher.decode",
        json!({"tokens":value(py,tokens,"tokens")?}),
        json!({}),
        "report",
    )
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(malespin, module)?)?;
    module.add_function(wrap_pyfunction!(vatsyayana, module)?)?;
    module.add_function(wrap_pyfunction!(nomenclator_encode, module)?)?;
    module.add_function(wrap_pyfunction!(nomenclator_decode, module)?)?;
    module.add_function(wrap_pyfunction!(syllabary_encode, module)?)?;
    module.add_function(wrap_pyfunction!(syllabary_decode, module)?)?;
    module.add_function(wrap_pyfunction!(great_cipher_encode, module)?)?;
    module.add_function(wrap_pyfunction!(great_cipher_decode, module)?)?;
    Ok(())
}
