//! Native KEM byte transport. Core owns validation and algorithms.
use cube_core::asymmetric::{alternate_kem, kyber, ml_kem};
use pyo3::{prelude::*, types::PyBytes};

type BytePair<'py> = (Bound<'py, PyBytes>, Bound<'py, PyBytes>);

#[pyfunction]
fn ml_kem_keygen<'py>(py: Python<'py>, parameters_json: &str) -> PyResult<BytePair<'py>> {
    let profile = ml_kem::Profile::parse(parameters_json).map_err(super::file_error)?;
    let keys = ml_kem::keygen(profile).map_err(super::file_error)?;
    Ok((
        PyBytes::new(py, &keys.public_key),
        PyBytes::new(py, &keys.private_key),
    ))
}

#[pyfunction]
fn ml_kem_encapsulate<'py>(
    py: Python<'py>,
    public_key: &[u8],
    parameters_json: &str,
) -> PyResult<BytePair<'py>> {
    let profile = ml_kem::Profile::parse(parameters_json).map_err(super::file_error)?;
    let result = ml_kem::encapsulate(public_key, profile).map_err(super::file_error)?;
    Ok((
        PyBytes::new(py, &result.ciphertext),
        PyBytes::new(py, &result.shared_secret[..]),
    ))
}

#[pyfunction]
fn ml_kem_decapsulate<'py>(
    py: Python<'py>,
    ciphertext: &[u8],
    private_key: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let profile = ml_kem::Profile::parse(parameters_json).map_err(super::file_error)?;
    let shared =
        ml_kem::decapsulate(ciphertext, private_key, profile).map_err(super::file_error)?;
    Ok(PyBytes::new(py, &shared[..]))
}

#[pyfunction]
fn kyber_decapsulate<'py>(
    py: Python<'py>,
    ciphertext: &[u8],
    private_key: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let profile = kyber::Profile::parse(parameters_json).map_err(super::file_error)?;
    let shared = kyber::decapsulate(ciphertext, private_key, profile).map_err(super::file_error)?;
    Ok(PyBytes::new(py, &shared[..]))
}

macro_rules! raw_decapsulator {
    ($function:ident, $profile:ident) => {
        #[pyfunction]
        fn $function<'py>(
            py: Python<'py>,
            ciphertext: &[u8],
            private_key: &[u8],
            parameters_json: &str,
        ) -> PyResult<Bound<'py, PyBytes>> {
            let profile = alternate_kem::Profile::$profile;
            profile
                .validate_parameters(parameters_json)
                .map_err(super::file_error)?;
            let shared = alternate_kem::decapsulate(ciphertext, private_key, profile)
                .map_err(super::file_error)?;
            Ok(PyBytes::new(py, &shared))
        }
    };
}

raw_decapsulator!(frodo_kem_decapsulate, Frodo640Shake);
raw_decapsulator!(hqc_decapsulate, Hqc128February2026);
raw_decapsulator!(classic_mceliece_decapsulate, McEliece460896f);

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(ml_kem_keygen, module)?)?;
    module.add_function(wrap_pyfunction!(ml_kem_encapsulate, module)?)?;
    module.add_function(wrap_pyfunction!(ml_kem_decapsulate, module)?)?;
    module.add_function(wrap_pyfunction!(kyber_decapsulate, module)?)?;
    module.add_function(wrap_pyfunction!(frodo_kem_decapsulate, module)?)?;
    module.add_function(wrap_pyfunction!(hqc_decapsulate, module)?)?;
    module.add_function(wrap_pyfunction!(classic_mceliece_decapsulate, module)?)
}
