//! Native transport only; every cipher is implemented and validated by Core.
use cube_core::ciphers::mono::{
    self, alphabet,
    checkerboard::{Profile, Table},
    grandpre, homophonic, rotations,
};
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
pub(crate) fn document(
    py: Python<'_>,
    value: &Bound<'_, PyAny>,
    maximum: usize,
    op: &'static str,
    parameter: &'static str,
) -> PyResult<String> {
    let kwargs = PyDict::new(py);
    kwargs.set_item("allow_nan", false)?;
    kwargs.set_item("ensure_ascii", false)?;
    kwargs.set_item("separators", (",", ":"))?;
    let encoder = py
        .import("json")?
        .getattr("JSONEncoder")?
        .call((), Some(&kwargs))?;
    let mut output = String::new();
    for chunk in encoder.call_method1("iterencode", (value,))?.try_iter()? {
        let chunk = chunk?;
        let chunk = chunk.extract::<&str>()?;
        if chunk.len() > maximum - output.len() {
            return Err(super::file_error(
                cube_core::errors::OperationError::new(
                    cube_core::errors::ErrorCode::ResourceLimit,
                    "JSON transport exceeds the operation byte budget",
                )
                .operation(op)
                .parameter(parameter),
            ));
        }
        output.push_str(chunk);
    }
    Ok(output)
}
#[pyfunction]
#[pyo3(signature=(text,source,*,target=None,keyword=None,unmapped="reject"))]
fn substitution_encode(
    text: &str,
    source: &str,
    target: Option<&str>,
    keyword: Option<&str>,
    unmapped: &str,
) -> PyResult<String> {
    alphabet::configured(source, target, keyword)
        .and_then(|m| m.encode(text, unmapped))
        .map_err(|e| super::file_error(e.operation("cipher.substitution.encode")))
}
#[pyfunction]
#[pyo3(signature=(text,source,*,target=None,keyword=None,unmapped="reject"))]
fn substitution_decode(
    text: &str,
    source: &str,
    target: Option<&str>,
    keyword: Option<&str>,
    unmapped: &str,
) -> PyResult<String> {
    alphabet::configured(source, target, keyword)
        .and_then(|m| m.decode(text, unmapped))
        .map_err(|e| super::file_error(e.operation("cipher.substitution.decode")))
}
#[pyfunction]
fn rot5(text: &str) -> PyResult<String> {
    rotations::rot5(text).map_err(super::file_error)
}
#[pyfunction]
fn rot18(text: &str) -> PyResult<String> {
    rotations::rot18(text).map_err(super::file_error)
}
#[pyfunction]
fn rot47(text: &str) -> PyResult<String> {
    rotations::rot47(text).map_err(super::file_error)
}
#[pyfunction]
#[pyo3(signature=(text,definition,*,counters=None))]
fn homophonic_encode(
    py: Python<'_>,
    text: &str,
    definition: &Bound<'_, PyDict>,
    counters: Option<&Bound<'_, PyList>>,
) -> PyResult<Py<PyAny>> {
    let op = "cipher.homophonic.encode";
    let map = homophonic::Mapping::parse(&document(
        py,
        definition.as_any(),
        mono::MAX_DEFINITION,
        op,
        "definition",
    )?)
    .map_err(super::file_error)?;
    let counters = counters
        .map(|v| {
            document(py, v.as_any(), mono::MAX_DEFINITION, op, "counters")
                .and_then(|s| mono::starting_counters(&s).map_err(super::file_error))
        })
        .transpose()?
        .unwrap_or_default();
    let result = map.encode(text, &counters).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(result))
}
#[pyfunction]
fn homophonic_decode(
    py: Python<'_>,
    tokens: &Bound<'_, PyList>,
    definition: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let op = "cipher.homophonic.decode";
    let map = homophonic::Mapping::parse(&document(
        py,
        definition.as_any(),
        mono::MAX_DEFINITION,
        op,
        "definition",
    )?)
    .map_err(|e| super::file_error(e.operation(op)))?;
    let tokens = mono::symbol_tokens(&document(
        py,
        tokens.as_any(),
        mono::MAX_INPUT,
        op,
        "tokens",
    )?)
    .map_err(super::file_error)?;
    map.decode(&tokens).map_err(super::file_error)
}
fn table(
    py: Python<'_>,
    definition: &Bound<'_, PyDict>,
    profile: Profile,
    op: &'static str,
) -> PyResult<Table> {
    Table::parse(
        profile,
        &document(
            py,
            definition.as_any(),
            mono::MAX_DEFINITION,
            op,
            "definition",
        )?,
    )
    .map_err(|e| super::file_error(e.operation(op)))
}
#[pyfunction]
fn straddling_encode(
    py: Python<'_>,
    text: &str,
    definition: &Bound<'_, PyDict>,
) -> PyResult<String> {
    table(
        py,
        definition,
        Profile::Straddling,
        "cipher.straddling.encode",
    )?
    .encode(text)
    .map_err(super::file_error)
}
#[pyfunction]
fn straddling_decode(
    py: Python<'_>,
    text: &str,
    definition: &Bound<'_, PyDict>,
) -> PyResult<String> {
    table(
        py,
        definition,
        Profile::Straddling,
        "cipher.straddling.decode",
    )?
    .decode(text)
    .map_err(super::file_error)
}
#[pyfunction]
fn monome_dinome_encode(
    py: Python<'_>,
    text: &str,
    definition: &Bound<'_, PyDict>,
) -> PyResult<String> {
    table(
        py,
        definition,
        Profile::MonomeDinome,
        "cipher.monome_dinome.encode",
    )?
    .encode(text)
    .map_err(super::file_error)
}
#[pyfunction]
fn monome_dinome_decode(
    py: Python<'_>,
    text: &str,
    definition: &Bound<'_, PyDict>,
) -> PyResult<String> {
    table(
        py,
        definition,
        Profile::MonomeDinome,
        "cipher.monome_dinome.decode",
    )?
    .decode(text)
    .map_err(super::file_error)
}
#[pyfunction]
fn tridigital_encode(
    py: Python<'_>,
    text: &str,
    definition: &Bound<'_, PyDict>,
) -> PyResult<String> {
    table(
        py,
        definition,
        Profile::Tridigital,
        "cipher.tridigital.encode",
    )?
    .encode(text)
    .map_err(super::file_error)
}
#[pyfunction]
fn tridigital_decode(
    py: Python<'_>,
    text: &str,
    definition: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let result = table(
        py,
        definition,
        Profile::Tridigital,
        "cipher.tridigital.decode",
    )?
    .alternatives(text)
    .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(result))
}
#[pyfunction]
fn grandpre_encode(
    py: Python<'_>,
    text: &str,
    definition: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let op = "cipher.grandpre.encode";
    let square = grandpre::Square::parse(&document(
        py,
        definition.as_any(),
        mono::MAX_DEFINITION,
        op,
        "definition",
    )?)
    .map_err(super::file_error)?;
    let result = square.encode(text).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(result))
}
#[pyfunction]
fn grandpre_decode(
    py: Python<'_>,
    tokens: &Bound<'_, PyList>,
    definition: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let op = "cipher.grandpre.decode";
    let square = grandpre::Square::parse(&document(
        py,
        definition.as_any(),
        mono::MAX_DEFINITION,
        op,
        "definition",
    )?)
    .map_err(|e| super::file_error(e.operation(op)))?;
    let tokens = mono::coordinate_tokens(&document(
        py,
        tokens.as_any(),
        mono::MAX_INPUT,
        op,
        "tokens",
    )?)
    .map_err(super::file_error)?;
    square.decode(&tokens).map_err(super::file_error)
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(substitution_encode, module)?)?;
    module.add_function(wrap_pyfunction!(substitution_decode, module)?)?;
    module.add_function(wrap_pyfunction!(rot5, module)?)?;
    module.add_function(wrap_pyfunction!(rot18, module)?)?;
    module.add_function(wrap_pyfunction!(rot47, module)?)?;
    module.add_function(wrap_pyfunction!(homophonic_encode, module)?)?;
    module.add_function(wrap_pyfunction!(homophonic_decode, module)?)?;
    module.add_function(wrap_pyfunction!(straddling_encode, module)?)?;
    module.add_function(wrap_pyfunction!(straddling_decode, module)?)?;
    module.add_function(wrap_pyfunction!(monome_dinome_encode, module)?)?;
    module.add_function(wrap_pyfunction!(monome_dinome_decode, module)?)?;
    module.add_function(wrap_pyfunction!(tridigital_encode, module)?)?;
    module.add_function(wrap_pyfunction!(tridigital_decode, module)?)?;
    module.add_function(wrap_pyfunction!(grandpre_encode, module)?)?;
    module.add_function(wrap_pyfunction!(grandpre_decode, module)?)?;
    Ok(())
}
