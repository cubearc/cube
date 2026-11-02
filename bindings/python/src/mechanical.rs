//! Native transport over Core-owned mechanical cipher implementations.
use cube_core::ciphers::mechanical::{
    self, alberti, disks, hagelin, kl7, kryha, lorenz, nema, sigaba, t52ab, typex, wheatstone,
};
use pyo3::{prelude::*, types::PyDict};
fn transform<P: serde::de::DeserializeOwned, R: serde::Serialize>(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
    op: &'static str,
    run: fn(&str, &P) -> Result<R, cube_core::errors::OperationError>,
) -> PyResult<Py<PyAny>> {
    let p = settings(py, parameters, op)?;
    let report = run(text, &p).map_err(super::file_error)?;
    super::structured(
        py,
        &serde_json::to_value(report).expect("typed mechanical report"),
    )
}
fn settings<P: serde::de::DeserializeOwned>(
    py: Python<'_>,
    parameters: &Bound<'_, PyDict>,
    op: &'static str,
) -> PyResult<P> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        mechanical::MAX_PARAMETERS,
        op,
        "parameters",
    )?;
    mechanical::parse_parameters(&document, op).map_err(super::file_error)
}
#[pyfunction]
fn m94_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(py, text, parameters, "cipher.m94.encode", disks::m94_encode)
}
#[pyfunction]
fn m94_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(py, text, parameters, "cipher.m94.decode", disks::m94_decode)
}
#[pyfunction]
fn jefferson_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.jefferson.encode",
        disks::jefferson_encode,
    )
}
#[pyfunction]
fn jefferson_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.jefferson.decode",
        disks::jefferson_decode,
    )
}
#[pyfunction]
fn m138a_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.m138a.encode",
        disks::m138a_encode,
    )
}
#[pyfunction]
fn m138a_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.m138a.decode",
        disks::m138a_decode,
    )
}
#[pyfunction]
fn wheatstone_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.wheatstone.encode",
        wheatstone::encode,
    )
}
#[pyfunction]
fn wheatstone_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.wheatstone.decode",
        wheatstone::decode,
    )
}
#[pyfunction]
fn kryha_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(py, text, parameters, "cipher.kryha.encode", kryha::encode)
}
#[pyfunction]
fn kryha_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(py, text, parameters, "cipher.kryha.decode", kryha::decode)
}
#[pyfunction]
fn alberti_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.alberti.encode",
        alberti::encode,
    )
}
#[pyfunction]
fn alberti_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.alberti.decode",
        alberti::decode,
    )
}
fn codes_transform<P: serde::de::DeserializeOwned, R: serde::Serialize>(
    py: Python<'_>,
    codes: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
    op: &'static str,
    run: fn(&[u8], &P) -> Result<R, cube_core::errors::OperationError>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(py, codes, mechanical::MAX_OUTPUT, op, "codes")?;
    let codes = lorenz::parse_codes(&document, op).map_err(super::file_error)?;
    let p = settings(py, parameters, op)?;
    let report = run(&codes, &p).map_err(super::file_error)?;
    super::structured(
        py,
        &serde_json::to_value(report).expect("typed five-bit report"),
    )
}
#[pyfunction]
fn lorenz_encode(
    py: Python<'_>,
    codes: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    codes_transform(
        py,
        codes,
        parameters,
        "cipher.lorenz.encode",
        lorenz::encode,
    )
}
#[pyfunction]
fn lorenz_decode(
    py: Python<'_>,
    codes: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    codes_transform(
        py,
        codes,
        parameters,
        "cipher.lorenz.decode",
        lorenz::decode,
    )
}
#[pyfunction]
fn hagelin_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.hagelin.encode",
        hagelin::encode,
    )
}
#[pyfunction]
fn hagelin_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.hagelin.decode",
        hagelin::decode,
    )
}
#[pyfunction]
fn t52ab_encode(
    py: Python<'_>,
    codes: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    codes_transform(py, codes, parameters, "cipher.t52ab.encode", t52ab::encode)
}
#[pyfunction]
fn t52ab_decode(
    py: Python<'_>,
    codes: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    codes_transform(py, codes, parameters, "cipher.t52ab.decode", t52ab::decode)
}
#[pyfunction]
fn nema_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(py, text, parameters, "cipher.nema.encode", nema::encode)
}
#[pyfunction]
fn nema_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(py, text, parameters, "cipher.nema.decode", nema::decode)
}
#[pyfunction]
fn typex_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(py, text, parameters, "cipher.typex.encode", typex::encode)
}
#[pyfunction]
fn typex_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(py, text, parameters, "cipher.typex.decode", typex::decode)
}
#[pyfunction]
fn sigaba_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    transform(py, text, parameters, "cipher.sigaba.encode", sigaba::encode)
}
#[pyfunction]
fn sigaba_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    transform(py, text, parameters, "cipher.sigaba.decode", sigaba::decode)
}
#[pyfunction]
fn kl7_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(py, text, parameters, "cipher.kl7.encode", kl7::encode)
}
#[pyfunction]
fn kl7_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(py, text, parameters, "cipher.kl7.decode", kl7::decode)
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(sigaba_encode, module)?)?;
    module.add_function(wrap_pyfunction!(sigaba_decode, module)?)?;
    module.add_function(wrap_pyfunction!(kl7_encode, module)?)?;
    module.add_function(wrap_pyfunction!(kl7_decode, module)?)?;
    module.add_function(wrap_pyfunction!(typex_encode, module)?)?;
    module.add_function(wrap_pyfunction!(typex_decode, module)?)?;
    module.add_function(wrap_pyfunction!(nema_encode, module)?)?;
    module.add_function(wrap_pyfunction!(nema_decode, module)?)?;
    module.add_function(wrap_pyfunction!(t52ab_encode, module)?)?;
    module.add_function(wrap_pyfunction!(t52ab_decode, module)?)?;
    module.add_function(wrap_pyfunction!(hagelin_encode, module)?)?;
    module.add_function(wrap_pyfunction!(hagelin_decode, module)?)?;
    module.add_function(wrap_pyfunction!(lorenz_encode, module)?)?;
    module.add_function(wrap_pyfunction!(lorenz_decode, module)?)?;
    module.add_function(wrap_pyfunction!(wheatstone_encode, module)?)?;
    module.add_function(wrap_pyfunction!(wheatstone_decode, module)?)?;
    module.add_function(wrap_pyfunction!(kryha_encode, module)?)?;
    module.add_function(wrap_pyfunction!(kryha_decode, module)?)?;
    module.add_function(wrap_pyfunction!(alberti_encode, module)?)?;
    module.add_function(wrap_pyfunction!(alberti_decode, module)?)?;
    module.add_function(wrap_pyfunction!(m94_encode, module)?)?;
    module.add_function(wrap_pyfunction!(m94_decode, module)?)?;
    module.add_function(wrap_pyfunction!(jefferson_encode, module)?)?;
    module.add_function(wrap_pyfunction!(jefferson_decode, module)?)?;
    module.add_function(wrap_pyfunction!(m138a_encode, module)?)?;
    module.add_function(wrap_pyfunction!(m138a_decode, module)?)
}
