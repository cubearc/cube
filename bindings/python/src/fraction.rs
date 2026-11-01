//! Typed native adapters; all behavior is owned by Core.
use cube_core::ciphers::{fractionation, poly, transposition};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyModule};
#[pyfunction]
fn scytale_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.scytale.encode",
        "parameters",
    )?;
    let p: transposition::Scytale = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.scytale.encode")))?;
    transposition::scytale(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn scytale_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.scytale.decode",
        "parameters",
    )?;
    let p: transposition::Scytale = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.scytale.decode")))?;
    transposition::scytale(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn rail_fence_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.rail_fence.encode",
        "parameters",
    )?;
    let p: transposition::Rails = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.rail_fence.encode")))?;
    transposition::rail_fence(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn rail_fence_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.rail_fence.decode",
        "parameters",
    )?;
    let p: transposition::Rails = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.rail_fence.decode")))?;
    transposition::rail_fence(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn redefence_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.redefence.encode",
        "parameters",
    )?;
    let p: transposition::Redefence = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.redefence.encode")))?;
    transposition::redefence(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn redefence_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.redefence.decode",
        "parameters",
    )?;
    let p: transposition::Redefence = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.redefence.decode")))?;
    transposition::redefence(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn sequence_transposition_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.sequence_transposition.encode",
        "parameters",
    )?;
    let p: transposition::Sequence = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.sequence_transposition.encode")))?;
    transposition::sequence(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn sequence_transposition_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.sequence_transposition.decode",
        "parameters",
    )?;
    let p: transposition::Sequence = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.sequence_transposition.decode")))?;
    transposition::sequence(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn skip_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.skip.encode",
        "parameters",
    )?;
    let p: transposition::Skip = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.skip.encode")))?;
    transposition::skip(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn skip_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.skip.decode",
        "parameters",
    )?;
    let p: transposition::Skip = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.skip.decode")))?;
    transposition::skip(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn columnar_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.columnar.encode",
        "parameters",
    )?;
    let p: transposition::Columnar = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.columnar.encode")))?;
    transposition::columnar(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn columnar_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.columnar.decode",
        "parameters",
    )?;
    let p: transposition::Columnar = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.columnar.decode")))?;
    transposition::columnar(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn complete_columnar_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.complete_columnar.encode",
        "parameters",
    )?;
    let p: transposition::CompleteColumnar = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.complete_columnar.encode")))?;
    let result = transposition::complete_columnar(text, &p, false).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed result"))
}
#[pyfunction]
fn complete_columnar_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.complete_columnar.decode",
        "parameters",
    )?;
    let p: transposition::CompleteColumnar = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.complete_columnar.decode")))?;
    let result = transposition::complete_columnar(text, &p, true).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed result"))
}
#[pyfunction]
fn double_columnar_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.double_columnar.encode",
        "parameters",
    )?;
    let p: transposition::DoubleColumnar = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.double_columnar.encode")))?;
    transposition::double_columnar(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn double_columnar_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.double_columnar.decode",
        "parameters",
    )?;
    let p: transposition::DoubleColumnar = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.double_columnar.decode")))?;
    transposition::double_columnar(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn myszkowski_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.myszkowski.encode",
        "parameters",
    )?;
    let p: transposition::Myszkowski = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.myszkowski.encode")))?;
    transposition::myszkowski(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn myszkowski_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.myszkowski.decode",
        "parameters",
    )?;
    let p: transposition::Myszkowski = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.myszkowski.decode")))?;
    transposition::myszkowski(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn amsco_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.amsco.encode",
        "parameters",
    )?;
    let p: transposition::Amsco = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.amsco.encode")))?;
    transposition::amsco(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn amsco_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.amsco.decode",
        "parameters",
    )?;
    let p: transposition::Amsco = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.amsco.decode")))?;
    transposition::amsco(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn route_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.route.encode",
        "parameters",
    )?;
    let p: transposition::routes::Routes = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.route.encode")))?;
    transposition::routes::route(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn route_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.route.decode",
        "parameters",
    )?;
    let p: transposition::routes::Routes = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.route.decode")))?;
    transposition::routes::route(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn spiral_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.spiral.encode",
        "parameters",
    )?;
    let p: transposition::routes::Routes = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.spiral.encode")))?;
    transposition::routes::spiral(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn spiral_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.spiral.decode",
        "parameters",
    )?;
    let p: transposition::routes::Routes = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.spiral.decode")))?;
    transposition::routes::spiral(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn caesar_box_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.caesar_box.encode",
        "parameters",
    )?;
    let p: transposition::routes::CaesarBox = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.caesar_box.encode")))?;
    transposition::routes::caesar_box(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn caesar_box_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.caesar_box.decode",
        "parameters",
    )?;
    let p: transposition::routes::CaesarBox = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.caesar_box.decode")))?;
    transposition::routes::caesar_box(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn turning_grille_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.turning_grille.encode",
        "parameters",
    )?;
    let p: transposition::grille::TurningGrille = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.turning_grille.encode")))?;
    transposition::grille::turning_grille(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn turning_grille_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.turning_grille.decode",
        "parameters",
    )?;
    let p: transposition::grille::TurningGrille = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.turning_grille.decode")))?;
    transposition::grille::turning_grille(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn bifid_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.bifid.encode",
        "parameters",
    )?;
    let p: fractionation::Bifid = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.bifid.encode")))?;
    fractionation::bifid(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn bifid_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.bifid.decode",
        "parameters",
    )?;
    let p: fractionation::Bifid = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.bifid.decode")))?;
    fractionation::bifid(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn trifid_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.trifid.encode",
        "parameters",
    )?;
    let p: fractionation::Trifid = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.trifid.encode")))?;
    fractionation::trifid(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn trifid_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.trifid.decode",
        "parameters",
    )?;
    let p: fractionation::Trifid = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.trifid.decode")))?;
    fractionation::trifid(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn adfgx_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.adfgx.encode",
        "parameters",
    )?;
    let p: fractionation::Adfgx = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.adfgx.encode")))?;
    fractionation::adfgx(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn adfgx_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.adfgx.decode",
        "parameters",
    )?;
    let p: fractionation::Adfgx = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.adfgx.decode")))?;
    fractionation::adfgx(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn adfgvx_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.adfgvx.encode",
        "parameters",
    )?;
    let p: fractionation::Adfgvx = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.adfgvx.encode")))?;
    fractionation::adfgvx(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn adfgvx_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.adfgvx.decode",
        "parameters",
    )?;
    let p: fractionation::Adfgvx = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.adfgvx.decode")))?;
    fractionation::adfgvx(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn nihilist_substitution_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.nihilist_substitution.encode",
        "parameters",
    )?;
    let p: fractionation::Nihilist = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.nihilist_substitution.encode")))?;
    let result = fractionation::nihilist_encode(text, &p).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed result"))
}
#[pyfunction]
fn nihilist_substitution_decode(
    py: Python<'_>,
    tokens: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.nihilist_substitution.decode",
        "parameters",
    )?;
    let p: fractionation::Nihilist = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.nihilist_substitution.decode")))?;
    let document = super::mono::document(
        py,
        tokens,
        poly::MAX_PARAMETERS,
        "cipher.nihilist_substitution.decode",
        "tokens",
    )?;
    let tokens = fractionation::parse_nihilist_tokens(&document).map_err(super::file_error)?;
    fractionation::nihilist_decode(&tokens, &p).map_err(super::file_error)
}
#[pyfunction]
fn bazeries_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.bazeries.encode",
        "parameters",
    )?;
    let p: fractionation::Bazeries = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.bazeries.encode")))?;
    fractionation::bazeries(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn bazeries_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.bazeries.decode",
        "parameters",
    )?;
    let p: fractionation::Bazeries = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.bazeries.decode")))?;
    fractionation::bazeries(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn fractionated_morse_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.fractionated_morse.encode",
        "parameters",
    )?;
    let p: fractionation::morse::FractionatedMorse = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.fractionated_morse.encode")))?;
    let result =
        fractionation::morse::fractionated_morse(text, &p, false).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed result"))
}
#[pyfunction]
fn fractionated_morse_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.fractionated_morse.decode",
        "parameters",
    )?;
    let p: fractionation::morse::FractionatedMorse = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.fractionated_morse.decode")))?;
    let result =
        fractionation::morse::fractionated_morse(text, &p, true).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed result"))
}
#[pyfunction]
fn morbit_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.morbit.encode",
        "parameters",
    )?;
    let p: fractionation::morse::Morbit = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.morbit.encode")))?;
    let result = fractionation::morse::morbit(text, &p, false).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed result"))
}
#[pyfunction]
fn morbit_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.morbit.decode",
        "parameters",
    )?;
    let p: fractionation::morse::Morbit = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.morbit.decode")))?;
    let result = fractionation::morse::morbit(text, &p, true).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed result"))
}
#[pyfunction]
fn pollux_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.pollux.encode",
        "parameters",
    )?;
    let p: fractionation::morse::Pollux = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.pollux.encode")))?;
    let result = fractionation::morse::pollux(text, &p, false).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed result"))
}
#[pyfunction]
fn pollux_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.pollux.decode",
        "parameters",
    )?;
    let p: fractionation::morse::Pollux = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.pollux.decode")))?;
    let result = fractionation::morse::pollux(text, &p, true).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed result"))
}
#[pyfunction]
fn cardan_extract(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.cardan.extract",
        "parameters",
    )?;
    let p: transposition::grille::Cardan =
        poly::parse_parameters(&document).map_err(super::file_error)?;
    transposition::grille::cardan_extract(text, &p).map_err(super::file_error)
}
#[pyfunction]
fn cardan_overlay(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.cardan.overlay",
        "parameters",
    )?;
    let p: transposition::grille::Overlay =
        poly::parse_parameters(&document).map_err(super::file_error)?;
    let result = transposition::grille::cardan_overlay(text, &p).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn cadenus_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.cadenus.encode",
        "parameters",
    )?;
    let p: transposition::tables::Cadenus = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.cadenus.encode")))?;
    let result = transposition::tables::cadenus(text, &p, false).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn cadenus_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.cadenus.decode",
        "parameters",
    )?;
    let p: transposition::tables::Cadenus = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.cadenus.decode")))?;
    let result = transposition::tables::cadenus(text, &p, true).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn swagman_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.swagman.encode",
        "parameters",
    )?;
    let p: transposition::tables::Swagman = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.swagman.encode")))?;
    let result = transposition::tables::swagman(text, &p, false).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn swagman_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.swagman.decode",
        "parameters",
    )?;
    let p: transposition::tables::Swagman = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.swagman.decode")))?;
    let result = transposition::tables::swagman(text, &p, true).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn nihilist_transposition_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.nihilist_transposition.encode",
        "parameters",
    )?;
    let p: transposition::tables::Nihilist = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.nihilist_transposition.encode")))?;
    let result = transposition::tables::nihilist(text, &p, false).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn nihilist_transposition_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.nihilist_transposition.decode",
        "parameters",
    )?;
    let p: transposition::tables::Nihilist = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.nihilist_transposition.decode")))?;
    let result = transposition::tables::nihilist(text, &p, true).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn ubchi_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.ubchi.encode",
        "parameters",
    )?;
    let p: transposition::ubchi::Ubchi = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.ubchi.encode")))?;
    let result = transposition::ubchi::transform(text, &p, false).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn ubchi_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.ubchi.decode",
        "parameters",
    )?;
    let p: transposition::ubchi::Ubchi = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.ubchi.decode")))?;
    let result = transposition::ubchi::transform(text, &p, true).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn cm_bifid_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.cm_bifid.encode",
        "parameters",
    )?;
    let p: fractionation::cm::CmBifid = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.cm_bifid.encode")))?;
    fractionation::cm::cm_bifid(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn cm_bifid_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.cm_bifid.decode",
        "parameters",
    )?;
    let p: fractionation::cm::CmBifid = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.cm_bifid.decode")))?;
    fractionation::cm::cm_bifid(text, &p, true).map_err(super::file_error)
}
#[pyfunction]
fn vic_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.vic.encode",
        "parameters",
    )?;
    let p: fractionation::vic::VicEnglish2010 = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.vic.encode")))?;
    fractionation::vic::transform(text, &p, false).map_err(super::file_error)
}
#[pyfunction]
fn vic_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<String> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        poly::MAX_PARAMETERS,
        "cipher.vic.decode",
        "parameters",
    )?;
    let p: fractionation::vic::VicEnglish2010 = poly::parse_parameters(&document)
        .map_err(|e| super::file_error(e.operation("cipher.vic.decode")))?;
    fractionation::vic::transform(text, &p, true).map_err(super::file_error)
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(cadenus_encode, module)?)?;
    module.add_function(wrap_pyfunction!(cadenus_decode, module)?)?;
    module.add_function(wrap_pyfunction!(swagman_encode, module)?)?;
    module.add_function(wrap_pyfunction!(swagman_decode, module)?)?;
    module.add_function(wrap_pyfunction!(nihilist_transposition_encode, module)?)?;
    module.add_function(wrap_pyfunction!(nihilist_transposition_decode, module)?)?;
    module.add_function(wrap_pyfunction!(ubchi_encode, module)?)?;
    module.add_function(wrap_pyfunction!(ubchi_decode, module)?)?;
    module.add_function(wrap_pyfunction!(cm_bifid_encode, module)?)?;
    module.add_function(wrap_pyfunction!(cm_bifid_decode, module)?)?;
    module.add_function(wrap_pyfunction!(vic_encode, module)?)?;
    module.add_function(wrap_pyfunction!(vic_decode, module)?)?;

    module.add_function(wrap_pyfunction!(scytale_encode, module)?)?;
    module.add_function(wrap_pyfunction!(scytale_decode, module)?)?;
    module.add_function(wrap_pyfunction!(rail_fence_encode, module)?)?;
    module.add_function(wrap_pyfunction!(rail_fence_decode, module)?)?;
    module.add_function(wrap_pyfunction!(redefence_encode, module)?)?;
    module.add_function(wrap_pyfunction!(redefence_decode, module)?)?;
    module.add_function(wrap_pyfunction!(sequence_transposition_encode, module)?)?;
    module.add_function(wrap_pyfunction!(sequence_transposition_decode, module)?)?;
    module.add_function(wrap_pyfunction!(skip_encode, module)?)?;
    module.add_function(wrap_pyfunction!(skip_decode, module)?)?;
    module.add_function(wrap_pyfunction!(columnar_encode, module)?)?;
    module.add_function(wrap_pyfunction!(columnar_decode, module)?)?;
    module.add_function(wrap_pyfunction!(complete_columnar_encode, module)?)?;
    module.add_function(wrap_pyfunction!(complete_columnar_decode, module)?)?;
    module.add_function(wrap_pyfunction!(double_columnar_encode, module)?)?;
    module.add_function(wrap_pyfunction!(double_columnar_decode, module)?)?;
    module.add_function(wrap_pyfunction!(myszkowski_encode, module)?)?;
    module.add_function(wrap_pyfunction!(myszkowski_decode, module)?)?;
    module.add_function(wrap_pyfunction!(amsco_encode, module)?)?;
    module.add_function(wrap_pyfunction!(amsco_decode, module)?)?;
    module.add_function(wrap_pyfunction!(route_encode, module)?)?;
    module.add_function(wrap_pyfunction!(route_decode, module)?)?;
    module.add_function(wrap_pyfunction!(spiral_encode, module)?)?;
    module.add_function(wrap_pyfunction!(spiral_decode, module)?)?;
    module.add_function(wrap_pyfunction!(caesar_box_encode, module)?)?;
    module.add_function(wrap_pyfunction!(caesar_box_decode, module)?)?;
    module.add_function(wrap_pyfunction!(turning_grille_encode, module)?)?;
    module.add_function(wrap_pyfunction!(turning_grille_decode, module)?)?;
    module.add_function(wrap_pyfunction!(bifid_encode, module)?)?;
    module.add_function(wrap_pyfunction!(bifid_decode, module)?)?;
    module.add_function(wrap_pyfunction!(trifid_encode, module)?)?;
    module.add_function(wrap_pyfunction!(trifid_decode, module)?)?;
    module.add_function(wrap_pyfunction!(adfgx_encode, module)?)?;
    module.add_function(wrap_pyfunction!(adfgx_decode, module)?)?;
    module.add_function(wrap_pyfunction!(adfgvx_encode, module)?)?;
    module.add_function(wrap_pyfunction!(adfgvx_decode, module)?)?;
    module.add_function(wrap_pyfunction!(nihilist_substitution_encode, module)?)?;
    module.add_function(wrap_pyfunction!(nihilist_substitution_decode, module)?)?;
    module.add_function(wrap_pyfunction!(bazeries_encode, module)?)?;
    module.add_function(wrap_pyfunction!(bazeries_decode, module)?)?;
    module.add_function(wrap_pyfunction!(fractionated_morse_encode, module)?)?;
    module.add_function(wrap_pyfunction!(fractionated_morse_decode, module)?)?;
    module.add_function(wrap_pyfunction!(morbit_encode, module)?)?;
    module.add_function(wrap_pyfunction!(morbit_decode, module)?)?;
    module.add_function(wrap_pyfunction!(pollux_encode, module)?)?;
    module.add_function(wrap_pyfunction!(pollux_decode, module)?)?;
    module.add_function(wrap_pyfunction!(cardan_extract, module)?)?;
    module.add_function(wrap_pyfunction!(cardan_overlay, module)?)?;
    Ok(())
}
