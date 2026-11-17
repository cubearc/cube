//! Exact digit transport only; Core owns profile checks and all rounds.
use cube_core::ciphers::fpe;
use pyo3::prelude::*;
use zeroize::Zeroizing;
#[pyfunction]
fn fpe_encrypt(digits: Vec<u16>, parameters_json: &str) -> PyResult<Vec<u16>> {
    let input = Zeroizing::new(digits);
    let s = fpe::Settings::parse(parameters_json).map_err(super::file_error)?;
    fpe::encrypt(&input, &s).map_err(super::file_error)
}
#[pyfunction]
fn fpe_decrypt(digits: Vec<u16>, parameters_json: &str) -> PyResult<Vec<u16>> {
    let input = Zeroizing::new(digits);
    let s = fpe::Settings::parse(parameters_json).map_err(super::file_error)?;
    fpe::decrypt(&input, &s).map_err(super::file_error)
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(fpe_encrypt, module)?)?;
    module.add_function(wrap_pyfunction!(fpe_decrypt, module)?)
}
