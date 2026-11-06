//! Native byte adapters; all profile validation and algorithms belong to Core.
use cube_core::{
    ciphers::block_profiles::{self, Profile, Settings},
    errors::{ErrorCode, OperationError},
};
use pyo3::{
    prelude::*,
    types::{PyBool, PyBytes},
};
use zeroize::Zeroizing;

fn settings(
    profile: &str,
    key: &[u8],
    bits: Option<&Bound<'_, PyAny>>,
    tweak: Option<&[u8]>,
) -> PyResult<Settings> {
    let invalid = || {
        super::file_error(
            OperationError::new(
                ErrorCode::InvalidParameter,
                "effective_key_bits must be an integer for the selected profile",
            )
            .parameter("effective_key_bits"),
        )
    };
    let bits = bits
        .map(|value| {
            if value.is_instance_of::<PyBool>() {
                return Err(invalid());
            }
            value.extract::<u16>().map_err(|_| invalid())
        })
        .transpose()?;
    Settings::new_with_tweak(
        Profile::parse(profile).map_err(super::file_error)?,
        key,
        bits,
        tweak,
    )
    .map_err(super::file_error)
}
#[pyfunction(signature=(data,key,profile,*,effective_key_bits=None,tweak=None))]
fn block_encrypt_blocks<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
    profile: &str,
    effective_key_bits: Option<&Bound<'_, PyAny>>,
    tweak: Option<&[u8]>,
) -> PyResult<Bound<'py, PyBytes>> {
    let settings = settings(profile, key, effective_key_bits, tweak)?;
    let output =
        Zeroizing::new(block_profiles::encrypt(data, &settings).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
#[pyfunction(signature=(data,key,profile,*,effective_key_bits=None,tweak=None))]
fn block_decrypt_blocks<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
    profile: &str,
    effective_key_bits: Option<&Bound<'_, PyAny>>,
    tweak: Option<&[u8]>,
) -> PyResult<Bound<'py, PyBytes>> {
    let settings = settings(profile, key, effective_key_bits, tweak)?;
    let output =
        Zeroizing::new(block_profiles::decrypt(data, &settings).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(block_encrypt_blocks, module)?)?;
    module.add_function(wrap_pyfunction!(block_decrypt_blocks, module)?)
}
