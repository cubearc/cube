//! Thin native adapters: typed Core parameters, bounded JSON, no duplicated ciphers.
use cube_core::ciphers::poly::{self, geometry, hill, schedules, stateful, tableaux};
use pyo3::{prelude::*, types::PyDict};
#[pyfunction]
fn gronsfeld_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.gronsfeld.encode",
        "parameters",
    )?;
    let p: schedules::Gronsfeld = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.gronsfeld.encode")))?;
    schedules::gronsfeld(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn gronsfeld_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.gronsfeld.decode",
        "parameters",
    )?;
    let p: schedules::Gronsfeld = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.gronsfeld.decode")))?;
    schedules::gronsfeld(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn trithemius_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.trithemius.encode",
        "parameters",
    )?;
    let p: schedules::Trithemius = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.trithemius.encode")))?;
    schedules::trithemius(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn trithemius_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.trithemius.decode",
        "parameters",
    )?;
    let p: schedules::Trithemius = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.trithemius.decode")))?;
    schedules::trithemius(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn autokey_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.autokey.encode",
        "parameters",
    )?;
    let p: schedules::Autokey = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.autokey.encode")))?;
    schedules::autokey(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn autokey_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.autokey.decode",
        "parameters",
    )?;
    let p: schedules::Autokey = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.autokey.decode")))?;
    schedules::autokey(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn running_key_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.running_key.encode",
        "parameters",
    )?;
    let p: schedules::RunningKey = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.running_key.encode")))?;
    schedules::running_key(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn running_key_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.running_key.decode",
        "parameters",
    )?;
    let p: schedules::RunningKey = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.running_key.decode")))?;
    schedules::running_key(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn porta(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.porta",
        "parameters",
    )?;
    let p: schedules::Key = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.porta")))?;
    schedules::porta(text, &p).map_err(super::file_error)
}
#[pyfunction]
fn gromark_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.gromark.encode",
        "parameters",
    )?;
    let p: stateful::Gromark = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.gromark.encode")))?;
    stateful::gromark(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn gromark_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.gromark.decode",
        "parameters",
    )?;
    let p: stateful::Gromark = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.gromark.decode")))?;
    stateful::gromark(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn periodic_gromark_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.periodic_gromark.encode",
        "parameters",
    )?;
    let p: stateful::PeriodicGromark = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.periodic_gromark.encode")))?;
    stateful::periodic_gromark(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn periodic_gromark_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.periodic_gromark.decode",
        "parameters",
    )?;
    let p: stateful::PeriodicGromark = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.periodic_gromark.decode")))?;
    stateful::periodic_gromark(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn solitaire_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.solitaire.encode",
        "parameters",
    )?;
    let p: stateful::Solitaire = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.solitaire.encode")))?;
    stateful::solitaire(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn solitaire_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.solitaire.decode",
        "parameters",
    )?;
    let p: stateful::Solitaire = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.solitaire.decode")))?;
    stateful::solitaire(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn slidefair_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.slidefair.encode",
        "parameters",
    )?;
    let p: schedules::Key = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.slidefair.encode")))?;
    tableaux::slidefair(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn slidefair_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.slidefair.decode",
        "parameters",
    )?;
    let p: schedules::Key = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.slidefair.decode")))?;
    tableaux::slidefair(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn portax(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.portax",
        "parameters",
    )?;
    let p: schedules::Key = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.portax")))?;
    tableaux::portax(text, &p).map_err(super::file_error)
}
#[pyfunction]
fn ragbaby_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.ragbaby.encode",
        "parameters",
    )?;
    let p: stateful::Ragbaby = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.ragbaby.encode")))?;
    stateful::ragbaby(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn ragbaby_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.ragbaby.decode",
        "parameters",
    )?;
    let p: stateful::Ragbaby = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.ragbaby.decode")))?;
    stateful::ragbaby(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn condi_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.condi.encode",
        "parameters",
    )?;
    let p: stateful::Condi = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.condi.encode")))?;
    stateful::condi(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn condi_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.condi.decode",
        "parameters",
    )?;
    let p: stateful::Condi = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.condi.decode")))?;
    stateful::condi(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn nicodemus_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.nicodemus.encode",
        "parameters",
    )?;
    let p: tableaux::Nicodemus = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.nicodemus.encode")))?;
    tableaux::nicodemus(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn nicodemus_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.nicodemus.decode",
        "parameters",
    )?;
    let p: tableaux::Nicodemus = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.nicodemus.decode")))?;
    tableaux::nicodemus(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn quagmire_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.quagmire.encode",
        "parameters",
    )?;
    let p: tableaux::Quagmire = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.quagmire.encode")))?;
    tableaux::quagmire(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn quagmire_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.quagmire.decode",
        "parameters",
    )?;
    let p: tableaux::Quagmire = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.quagmire.decode")))?;
    tableaux::quagmire(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn phillips_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.phillips.encode",
        "parameters",
    )?;
    let p: tableaux::Phillips = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.phillips.encode")))?;
    tableaux::phillips(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn phillips_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.phillips.decode",
        "parameters",
    )?;
    let p: tableaux::Phillips = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.phillips.decode")))?;
    tableaux::phillips(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn chaocipher_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.chaocipher.encode",
        "parameters",
    )?;
    let p: stateful::Chaocipher = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.chaocipher.encode")))?;
    stateful::chaocipher(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn chaocipher_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.chaocipher.decode",
        "parameters",
    )?;
    let p: stateful::Chaocipher = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.chaocipher.decode")))?;
    stateful::chaocipher(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn playfair_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.playfair.encode",
        "parameters",
    )?;
    let p: geometry::Playfair = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.playfair.encode")))?;
    let result = geometry::playfair(text, &p, false).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed result"))
}
#[pyfunction]
fn playfair_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.playfair.decode",
        "parameters",
    )?;
    let p: geometry::Playfair = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.playfair.decode")))?;
    let result = geometry::playfair(text, &p, true).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed result"))
}
#[pyfunction]
fn seriated_playfair_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.seriated_playfair.encode",
        "parameters",
    )?;
    let p: geometry::SeriatedPlayfair = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.seriated_playfair.encode")))?;
    let result = geometry::seriated_playfair(text, &p, false).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed result"))
}
#[pyfunction]
fn seriated_playfair_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.seriated_playfair.decode",
        "parameters",
    )?;
    let p: geometry::SeriatedPlayfair = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.seriated_playfair.decode")))?;
    let result = geometry::seriated_playfair(text, &p, true).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed result"))
}
#[pyfunction]
fn two_square_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.two_square.encode",
        "parameters",
    )?;
    let p: geometry::Squares = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.two_square.encode")))?;
    geometry::two_square(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn two_square_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.two_square.decode",
        "parameters",
    )?;
    let p: geometry::Squares = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.two_square.decode")))?;
    geometry::two_square(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn four_square_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.four_square.encode",
        "parameters",
    )?;
    let p: geometry::Squares = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.four_square.encode")))?;
    geometry::four_square(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn four_square_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.four_square.decode",
        "parameters",
    )?;
    let p: geometry::Squares = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.four_square.decode")))?;
    geometry::four_square(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn hill_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.hill.encode",
        "parameters",
    )?;
    let p: hill::Hill = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.hill.encode")))?;
    let result = hill::hill(text, &p, false).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed result"))
}
#[pyfunction]
fn hill_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.hill.decode",
        "parameters",
    )?;
    let p: hill::Hill = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.hill.decode")))?;
    let result = hill::hill(text, &p, true).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed result"))
}
#[pyfunction]
fn digrafid_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.digrafid.encode",
        "parameters",
    )?;
    let p: geometry::Digrafid = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.digrafid.encode")))?;
    let result = geometry::digrafid(text, &p, false).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed result"))
}
#[pyfunction]
fn digrafid_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.digrafid.decode",
        "parameters",
    )?;
    let p: geometry::Digrafid = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.digrafid.decode")))?;
    let result = geometry::digrafid(text, &p, true).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed result"))
}
#[pyfunction]
fn tri_square_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.tri_square.encode",
        "parameters",
    )?;
    let p: geometry::TriSquare = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.tri_square.encode")))?;
    geometry::tri_square(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn tri_square_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.tri_square.decode",
        "parameters",
    )?;
    let p: geometry::TriSquare = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.tri_square.decode")))?;
    geometry::tri_square(text, &p, true).map_err(super::file_error)
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(gronsfeld_encode, module)?)?;
    module.add_function(wrap_pyfunction!(gronsfeld_decode, module)?)?;
    module.add_function(wrap_pyfunction!(trithemius_encode, module)?)?;
    module.add_function(wrap_pyfunction!(trithemius_decode, module)?)?;
    module.add_function(wrap_pyfunction!(autokey_encode, module)?)?;
    module.add_function(wrap_pyfunction!(autokey_decode, module)?)?;
    module.add_function(wrap_pyfunction!(running_key_encode, module)?)?;
    module.add_function(wrap_pyfunction!(running_key_decode, module)?)?;
    module.add_function(wrap_pyfunction!(porta, module)?)?;
    module.add_function(wrap_pyfunction!(gromark_encode, module)?)?;
    module.add_function(wrap_pyfunction!(gromark_decode, module)?)?;
    module.add_function(wrap_pyfunction!(periodic_gromark_encode, module)?)?;
    module.add_function(wrap_pyfunction!(periodic_gromark_decode, module)?)?;
    module.add_function(wrap_pyfunction!(solitaire_encode, module)?)?;
    module.add_function(wrap_pyfunction!(solitaire_decode, module)?)?;
    module.add_function(wrap_pyfunction!(slidefair_encode, module)?)?;
    module.add_function(wrap_pyfunction!(slidefair_decode, module)?)?;
    module.add_function(wrap_pyfunction!(portax, module)?)?;
    module.add_function(wrap_pyfunction!(ragbaby_encode, module)?)?;
    module.add_function(wrap_pyfunction!(ragbaby_decode, module)?)?;
    module.add_function(wrap_pyfunction!(condi_encode, module)?)?;
    module.add_function(wrap_pyfunction!(condi_decode, module)?)?;
    module.add_function(wrap_pyfunction!(nicodemus_encode, module)?)?;
    module.add_function(wrap_pyfunction!(nicodemus_decode, module)?)?;
    module.add_function(wrap_pyfunction!(quagmire_encode, module)?)?;
    module.add_function(wrap_pyfunction!(quagmire_decode, module)?)?;
    module.add_function(wrap_pyfunction!(phillips_encode, module)?)?;
    module.add_function(wrap_pyfunction!(phillips_decode, module)?)?;
    module.add_function(wrap_pyfunction!(chaocipher_encode, module)?)?;
    module.add_function(wrap_pyfunction!(chaocipher_decode, module)?)?;
    module.add_function(wrap_pyfunction!(playfair_encode, module)?)?;
    module.add_function(wrap_pyfunction!(playfair_decode, module)?)?;
    module.add_function(wrap_pyfunction!(seriated_playfair_encode, module)?)?;
    module.add_function(wrap_pyfunction!(seriated_playfair_decode, module)?)?;
    module.add_function(wrap_pyfunction!(two_square_encode, module)?)?;
    module.add_function(wrap_pyfunction!(two_square_decode, module)?)?;
    module.add_function(wrap_pyfunction!(four_square_encode, module)?)?;
    module.add_function(wrap_pyfunction!(four_square_decode, module)?)?;
    module.add_function(wrap_pyfunction!(hill_encode, module)?)?;
    module.add_function(wrap_pyfunction!(hill_decode, module)?)?;
    module.add_function(wrap_pyfunction!(digrafid_encode, module)?)?;
    module.add_function(wrap_pyfunction!(digrafid_decode, module)?)?;
    module.add_function(wrap_pyfunction!(tri_square_encode, module)?)?;
    module.add_function(wrap_pyfunction!(tri_square_decode, module)?)?;
    Ok(())
}
