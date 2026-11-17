//! Native ciphertext byte adapters for public homomorphic operations.
use cube_core::asymmetric::homomorphic::{self as h, Profile};
use pyo3::{prelude::*, types::PyBytes};

macro_rules! add_ciphertexts {
    ($function:ident, $profile:ident) => {
        #[pyfunction]
        fn $function<'py>(
            py: Python<'py>,
            left: &[u8],
            right: &[u8],
            parameters_json: &str,
        ) -> PyResult<Bound<'py, PyBytes>> {
            let profile = Profile::$profile;
            profile
                .validate_parameters(parameters_json)
                .map_err(super::file_error)?;
            let output = h::add(left, right, profile).map_err(super::file_error)?;
            Ok(PyBytes::new(py, &output))
        }
    };
}

add_ciphertexts!(bfv_add, Bfv);
add_ciphertexts!(bgv_add, Bgv);
add_ciphertexts!(ckks_add, Ckks);

#[pyfunction]
fn tfhe_not<'py>(
    py: Python<'py>,
    ciphertext: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    Profile::Tfhe
        .validate_parameters(parameters_json)
        .map_err(super::file_error)?;
    let output = h::tfhe_not(ciphertext).map_err(super::file_error)?;
    Ok(PyBytes::new(py, &output))
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(bfv_add, module)?)?;
    module.add_function(wrap_pyfunction!(bgv_add, module)?)?;
    module.add_function(wrap_pyfunction!(ckks_add, module)?)?;
    module.add_function(wrap_pyfunction!(paillier_add, module)?)?;
    module.add_function(wrap_pyfunction!(damgard_jurik_add, module)?)?;
    module.add_function(wrap_pyfunction!(tfhe_not, module)?)
}

macro_rules! add_integer_ciphertexts {
    ($function:ident, $profile:ident) => {
        #[pyfunction]
        fn $function<'py>(
            py: Python<'py>,
            left: &[u8],
            right: &[u8],
            public_modulus: &[u8],
            parameters_json: &str,
        ) -> PyResult<Bound<'py, PyBytes>> {
            use cube_core::asymmetric::additive;
            let profile = additive::Profile::$profile;
            profile
                .validate_parameters(parameters_json)
                .map_err(super::file_error)?;
            let output =
                additive::add(left, right, public_modulus, profile).map_err(super::file_error)?;
            Ok(PyBytes::new(py, &output))
        }
    };
}

add_integer_ciphertexts!(paillier_add, Paillier);
add_integer_ciphertexts!(damgard_jurik_add, DamgardJurik);
