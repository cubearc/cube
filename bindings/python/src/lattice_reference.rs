//! Native byte adapters for the educational key pairs.
use cube_core::asymmetric::lattice_reference::{self as l, Profile};
use pyo3::{prelude::*, types::PyBytes};
use zeroize::Zeroizing;

macro_rules! lattice_transform {
    ($function:ident, $transform:ident, $profile:ident, $key:ident) => {
        #[pyfunction]
        fn $function<'py>(
            py: Python<'py>,
            input: &[u8],
            $key: &[u8],
            parameters_json: &str,
        ) -> PyResult<Bound<'py, PyBytes>> {
            let profile = Profile::$profile;
            profile
                .validate_parameters(parameters_json)
                .map_err(super::file_error)?;
            let output =
                Zeroizing::new(l::$transform(input, $key, profile).map_err(super::file_error)?);
            Ok(PyBytes::new(py, &output))
        }
    };
}

lattice_transform!(ntru_encrypt_encrypt, encrypt, NtruEncrypt, public_key);
lattice_transform!(ntru_encrypt_decrypt, decrypt, NtruEncrypt, private_key);
lattice_transform!(regev_encrypt, encrypt, Regev, public_key);
lattice_transform!(regev_decrypt, decrypt, Regev, private_key);

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(ntru_encrypt_encrypt, module)?)?;
    module.add_function(wrap_pyfunction!(ntru_encrypt_decrypt, module)?)?;
    module.add_function(wrap_pyfunction!(regev_encrypt, module)?)?;
    module.add_function(wrap_pyfunction!(regev_decrypt, module)?)
}
