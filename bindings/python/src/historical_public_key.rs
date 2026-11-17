//! Bounded exact token/key transport only; Core owns the reference algorithms.
use cube_core::asymmetric::historical_public_key as h;
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyList, PyString},
};
use zeroize::Zeroizing;
fn tokens(value: &Bound<'_, PyList>) -> PyResult<Zeroizing<Vec<String>>> {
    if value.len() > h::MAX_TOKENS {
        return Err(PyValueError::new_err("reference token count exceeds bound"));
    }
    let mut size = 2usize;
    let mut output = Zeroizing::new(Vec::with_capacity(value.len()));
    for value in value.iter() {
        let text = value.cast::<PyString>()?;
        if text.as_any().len()? > h::MAX_TOKEN_BYTES {
            return Err(PyValueError::new_err(
                "reference token length exceeds bound",
            ));
        }
        let text = text.to_str()?;
        if text.len() > h::MAX_TOKEN_BYTES {
            return Err(PyValueError::new_err(
                "reference token length exceeds bound",
            ));
        }
        size = size
            .checked_add(text.len() + 3)
            .ok_or_else(|| PyValueError::new_err("reference token transport exceeds bound"))?;
        if size > h::MAX_REPORT_BYTES {
            return Err(PyValueError::new_err(
                "reference token transport exceeds bound",
            ));
        }
        output.push(text.to_owned());
    }
    Ok(output)
}
#[pyfunction]
fn rabin_encrypt(
    data: &Bound<'_, PyList>,
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Vec<String>> {
    let settings =
        h::rabin::EncryptionSettings::parse(parameters_json, key).map_err(super::file_error)?;
    let data = tokens(data)?;
    h::rabin::encrypt(&data, &settings).map_err(super::file_error)
}
#[pyfunction]
fn rabin_decrypt(
    data: &Bound<'_, PyList>,
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Vec<Vec<String>>> {
    let settings =
        h::rabin::DecryptionSettings::parse(parameters_json, key).map_err(super::file_error)?;
    let data = tokens(data)?;
    h::rabin::decrypt(&data, &settings).map_err(super::file_error)
}
#[pyfunction]
fn goldwasser_micali_encrypt(
    data: &Bound<'_, PyList>,
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Vec<String>> {
    let settings = h::goldwasser_micali::EncryptionSettings::parse(parameters_json, key)
        .map_err(super::file_error)?;
    let data = tokens(data)?;
    h::goldwasser_micali::encrypt(&data, &settings).map_err(super::file_error)
}
#[pyfunction]
fn goldwasser_micali_decrypt(
    data: &Bound<'_, PyList>,
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Vec<String>> {
    let settings = h::goldwasser_micali::DecryptionSettings::parse(parameters_json, key)
        .map_err(super::file_error)?;
    let data = tokens(data)?;
    h::goldwasser_micali::decrypt(&data, &settings).map_err(super::file_error)
}
#[pyfunction]
fn benaloh_encrypt(
    data: &Bound<'_, PyList>,
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Vec<String>> {
    let settings =
        h::benaloh::EncryptionSettings::parse(parameters_json, key).map_err(super::file_error)?;
    let data = tokens(data)?;
    h::benaloh::encrypt(&data, &settings).map_err(super::file_error)
}
#[pyfunction]
fn benaloh_decrypt(
    data: &Bound<'_, PyList>,
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Vec<String>> {
    let settings =
        h::benaloh::DecryptionSettings::parse(parameters_json, key).map_err(super::file_error)?;
    let data = tokens(data)?;
    h::benaloh::decrypt(&data, &settings).map_err(super::file_error)
}
#[pyfunction]
fn merkle_hellman_encrypt(
    data: &Bound<'_, PyList>,
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Vec<String>> {
    let settings = h::merkle_hellman::EncryptionSettings::parse(parameters_json, key)
        .map_err(super::file_error)?;
    let data = tokens(data)?;
    h::merkle_hellman::encrypt(&data, &settings).map_err(super::file_error)
}
#[pyfunction]
fn merkle_hellman_decrypt(
    data: &Bound<'_, PyList>,
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Vec<String>> {
    let settings = h::merkle_hellman::DecryptionSettings::parse(parameters_json, key)
        .map_err(super::file_error)?;
    let data = tokens(data)?;
    h::merkle_hellman::decrypt(&data, &settings).map_err(super::file_error)
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(rabin_encrypt, module)?)?;
    module.add_function(wrap_pyfunction!(rabin_decrypt, module)?)?;
    module.add_function(wrap_pyfunction!(goldwasser_micali_encrypt, module)?)?;
    module.add_function(wrap_pyfunction!(goldwasser_micali_decrypt, module)?)?;
    module.add_function(wrap_pyfunction!(benaloh_encrypt, module)?)?;
    module.add_function(wrap_pyfunction!(benaloh_decrypt, module)?)?;
    module.add_function(wrap_pyfunction!(merkle_hellman_encrypt, module)?)?;
    module.add_function(wrap_pyfunction!(merkle_hellman_decrypt, module)?)?;
    Ok(())
}
