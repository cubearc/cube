//! Native adapters only; concrete algorithms and validation stay in cube-core.
use cube_core::{
    ciphers::{bacon, polybius},
    symbols::keypad,
    text::{classes, keyboard},
};
use pyo3::{
    prelude::*,
    types::{PyBool, PyDict, PyList, PyString},
};
#[pyfunction]
#[pyo3(signature=(text,variant,*,notation="ab"))]
fn bacon_encode(text: &str, variant: &str, notation: &str) -> PyResult<String> {
    bacon::encode(text, variant, notation).map_err(super::file_error)
}
#[pyfunction]
#[pyo3(signature=(text,variant,*,notation="ab"))]
fn bacon_decode(text: &str, variant: &str, notation: &str) -> PyResult<String> {
    bacon::decode(text, variant, notation).map_err(super::file_error)
}
fn square(
    preset: Option<&str>,
    alphabet: Option<&str>,
    side: Option<u8>,
    aliases: Vec<(char, char)>,
) -> PyResult<polybius::Square> {
    match (preset, alphabet, side) {
        (None, Some(alphabet), Some(side)) => {
            polybius::Square::new(side, alphabet, &aliases).map_err(super::file_error)
        }
        (preset, None, None) if aliases.is_empty() => {
            polybius::Square::preset(preset.unwrap_or("polybius")).map_err(super::file_error)
        }
        _ => Err(pyo3::exceptions::PyValueError::new_err(
            "choose preset or custom alphabet/side/aliases",
        )),
    }
}
#[pyfunction]
#[pyo3(signature=(text,*,preset=None,alphabet=None,side=None,aliases=None,index_base=1))]
fn polybius_encode(
    py: Python<'_>,
    text: &str,
    preset: Option<&str>,
    alphabet: Option<&str>,
    side: Option<u8>,
    aliases: Option<&Bound<'_, PyList>>,
    index_base: u8,
) -> PyResult<Py<PyAny>> {
    let tokens = square(preset, alphabet, side, alias_pairs(aliases)?)?
        .encode(text, index_base)
        .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(tokens))
}
#[pyfunction]
#[pyo3(signature=(tokens,*,preset=None,alphabet=None,side=None,aliases=None,index_base=1))]
fn polybius_decode(
    tokens: &Bound<'_, PyList>,
    preset: Option<&str>,
    alphabet: Option<&str>,
    side: Option<u8>,
    aliases: Option<&Bound<'_, PyList>>,
    index_base: u8,
) -> PyResult<String> {
    if tokens.len() > 65536 {
        return Err(super::file_error(
            cube_core::errors::OperationError::new(
                cube_core::errors::ErrorCode::ResourceLimit,
                "square input exceeds 65536 tokens",
            )
            .operation("cipher.polybius.decode")
            .parameter("tokens"),
        ));
    }
    for token in tokens.iter().filter(|token| !token.is_none()) {
        for coordinate in token.try_iter()? {
            if coordinate?.is_instance_of::<PyBool>() {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "coordinates must be exact unsigned integers, not booleans",
                ));
            }
        }
    }
    let tokens: Vec<polybius::Token> = tokens.extract()?;
    square(preset, alphabet, side, alias_pairs(aliases)?)?
        .decode(&tokens, index_base)
        .map_err(super::file_error)
}
fn alias_pairs(value: Option<&Bound<'_, PyList>>) -> PyResult<Vec<(char, char)>> {
    if let Some(value) = value {
        if value.len() > 4096 {
            return Err(super::file_error(
                cube_core::errors::OperationError::new(
                    cube_core::errors::ErrorCode::ResourceLimit,
                    "square aliases exceed 4096 entries",
                )
                .parameter("aliases"),
            ));
        }
        value.extract()
    } else {
        Ok(vec![])
    }
}
// Serialize native dictionaries incrementally; keep at most 64 KiB of transport.
// Concrete schema validation remains in core.
fn document(py: Python<'_>, value: &Bound<'_, PyDict>) -> PyResult<String> {
    let kwargs = PyDict::new(py);
    kwargs.set_item("allow_nan", false)?;
    kwargs.set_item("ensure_ascii", false)?;
    kwargs.set_item("separators", (",", ":"))?;
    let encoder = py
        .import("json")?
        .getattr("JSONEncoder")?
        .call((), Some(&kwargs))?;
    let mut document = String::new();
    for chunk in encoder.call_method1("iterencode", (value,))?.try_iter()? {
        let chunk = chunk?;
        let chunk = chunk.extract::<&str>()?;
        if chunk.len() > 65536 - document.len() {
            return Err(super::file_error(cube_core::errors::OperationError::new(
                cube_core::errors::ErrorCode::ResourceLimit,
                "mapping definition exceeds 64 KiB",
            )));
        }
        document.push_str(chunk);
    }
    Ok(document)
}
fn layout(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<keyboard::Layout> {
    if let Ok(value) = value.cast::<PyString>() {
        keyboard::Layout::preset(value.to_str()?).map_err(super::file_error)
    } else {
        keyboard::Layout::parse(&document(py, value.cast::<PyDict>()?)?).map_err(super::file_error)
    }
}
#[pyfunction]
#[pyo3(signature=(text,source,target,*,unmapped="preserve"))]
fn keyboard_remap(
    py: Python<'_>,
    text: &str,
    source: &Bound<'_, PyAny>,
    target: &Bound<'_, PyAny>,
    unmapped: &str,
) -> PyResult<String> {
    keyboard::remap(text, &layout(py, source)?, &layout(py, target)?, unmapped)
        .map_err(super::file_error)
}
fn mapping(py: Python<'_>, value: Option<&Bound<'_, PyDict>>) -> PyResult<keypad::Mapping> {
    value
        .map(|v| {
            document(py, v).and_then(|s| keypad::Mapping::parse(&s).map_err(super::file_error))
        })
        .transpose()
        .map(|v| v.unwrap_or_else(keypad::Mapping::standard))
}
#[pyfunction]
#[pyo3(signature=(text,mode,*,mapping=None))]
fn keypad_encode(
    py: Python<'_>,
    text: &str,
    mode: &str,
    mapping: Option<&Bound<'_, PyDict>>,
) -> PyResult<String> {
    self::mapping(py, mapping)?
        .encode(text, mode)
        .map_err(super::file_error)
}
#[pyfunction]
#[pyo3(signature=(text,mode,*,mapping=None))]
fn keypad_decode(
    py: Python<'_>,
    text: &str,
    mode: &str,
    mapping: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    let result = self::mapping(py, mapping)?
        .decode(text, mode)
        .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(result))
}
#[pyfunction]
#[pyo3(signature=(text,*,y=None,unmapped="error",definition=None))]
fn map_character_classes(
    py: Python<'_>,
    text: &str,
    y: Option<&str>,
    unmapped: &str,
    definition: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    let result = if let Some(definition) = definition {
        if y.is_some() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "custom definitions conflict with preset Y policy",
            ));
        }
        classes::Definition::parse(&document(py, definition)?)
            .map_err(super::file_error)?
            .map(text, unmapped)
    } else {
        classes::map(text, y, unmapped)
    }
    .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(result))
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(bacon_encode, module)?)?;
    module.add_function(wrap_pyfunction!(bacon_decode, module)?)?;
    module.add_function(wrap_pyfunction!(polybius_encode, module)?)?;
    module.add_function(wrap_pyfunction!(polybius_decode, module)?)?;
    module.add_function(wrap_pyfunction!(keyboard_remap, module)?)?;
    module.add_function(wrap_pyfunction!(keypad_encode, module)?)?;
    module.add_function(wrap_pyfunction!(keypad_decode, module)?)?;
    module.add_function(wrap_pyfunction!(map_character_classes, module)?)?;
    Ok(())
}
