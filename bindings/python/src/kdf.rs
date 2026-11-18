//! Native private-material adapters with exact raw byte arguments.
use super::file_error;
use cube_core::{
    ciphers::kdf,
    errors::{ErrorCode, OperationError},
};
use pyo3::{prelude::*, types::PyBytes};

fn sha2(profile: &str, id: &'static str) -> PyResult<kdf::Sha2> {
    match profile {
        "sha256" => Ok(kdf::Sha2::Sha256),
        "sha512" => Ok(kdf::Sha2::Sha512),
        _ => Err(file_error(
            OperationError::new(ErrorCode::InvalidParameter, "use sha256 or sha512")
                .parameter("profile")
                .operation(id),
        )),
    }
}
fn owned<'py>(
    py: Python<'py>,
    value: Result<kdf::SecretBytes, OperationError>,
) -> PyResult<Bound<'py, PyBytes>> {
    Ok(PyBytes::new(py, value.map_err(file_error)?.as_slice()))
}
#[pyfunction(signature = (password, salt, *, profile, iterations, output_bytes))]
fn kdf_pbkdf2<'py>(
    py: Python<'py>,
    password: &[u8],
    salt: &[u8],
    profile: &str,
    iterations: u32,
    output_bytes: u16,
) -> PyResult<Bound<'py, PyBytes>> {
    let profile = sha2(profile, kdf::PBKDF2)?;
    owned(
        py,
        kdf::pbkdf2(
            password,
            salt,
            kdf::Pbkdf2 {
                profile,
                iterations,
                output_bytes,
            },
        ),
    )
}
#[pyfunction(signature = (ikm, salt, info, *, profile, output_bytes))]
fn kdf_hkdf<'py>(
    py: Python<'py>,
    ikm: &[u8],
    salt: &[u8],
    info: &[u8],
    profile: &str,
    output_bytes: u16,
) -> PyResult<Bound<'py, PyBytes>> {
    let profile = sha2(profile, kdf::HKDF)?;
    owned(
        py,
        kdf::hkdf(
            ikm,
            salt,
            info,
            kdf::Hkdf {
                profile,
                output_bytes,
            },
        ),
    )
}
#[pyfunction(signature = (password, salt, *, n, r, p, output_bytes))]
fn kdf_scrypt<'py>(
    py: Python<'py>,
    password: &[u8],
    salt: &[u8],
    n: u32,
    r: u32,
    p: u32,
    output_bytes: u16,
) -> PyResult<Bound<'py, PyBytes>> {
    owned(
        py,
        kdf::scrypt(
            password,
            salt,
            kdf::Scrypt {
                n,
                r,
                p,
                output_bytes,
            },
        ),
    )
}
#[pyfunction(signature = (password, salt, *, memory_kib, iterations, lanes, output_bytes))]
fn kdf_argon2id<'py>(
    py: Python<'py>,
    password: &[u8],
    salt: &[u8],
    memory_kib: u32,
    iterations: u32,
    lanes: u32,
    output_bytes: u16,
) -> PyResult<Bound<'py, PyBytes>> {
    owned(
        py,
        kdf::argon2id(
            password,
            salt,
            kdf::Argon2id {
                memory_kib,
                iterations,
                lanes,
                output_bytes,
            },
        ),
    )
}
#[pyfunction(signature = (password, salt, *, digest, count, key_bytes, iv_bytes))]
fn kdf_evp_bytes_to_key<'py>(
    py: Python<'py>,
    password: &[u8],
    salt: &[u8],
    digest: &str,
    count: u32,
    key_bytes: u16,
    iv_bytes: u16,
) -> PyResult<(Bound<'py, PyBytes>, Bound<'py, PyBytes>)> {
    let digest = match digest {
        "md5" => kdf::LegacyDigest::Md5,
        "sha256" => kdf::LegacyDigest::Sha256,
        _ => {
            return Err(file_error(
                OperationError::new(ErrorCode::InvalidParameter, "use md5 or sha256")
                    .parameter("digest")
                    .operation(kdf::EVP_BYTES_TO_KEY),
            ));
        }
    };
    let result = kdf::evp_bytes_to_key(
        password,
        salt,
        kdf::EvpBytesToKey {
            digest,
            count,
            key_bytes,
            iv_bytes,
        },
    )
    .map_err(file_error)?;
    Ok((
        PyBytes::new(py, result.key()),
        PyBytes::new(py, result.iv()),
    ))
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(kdf_pbkdf2, module)?)?;
    module.add_function(wrap_pyfunction!(kdf_hkdf, module)?)?;
    module.add_function(wrap_pyfunction!(kdf_scrypt, module)?)?;
    module.add_function(wrap_pyfunction!(kdf_argon2id, module)?)?;
    module.add_function(wrap_pyfunction!(kdf_evp_bytes_to_key, module)?)?;
    Ok(())
}
