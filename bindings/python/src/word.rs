//! Native signed word positions; exact integers and bounded concrete lists.
use cube_core::{
    errors::{ErrorCode, OperationError},
    text::word::{self, Positions},
};
use pyo3::{
    prelude::*,
    types::{PyBool, PyList},
};
fn invalid(message: &str) -> PyErr {
    super::file_error(
        OperationError::new(ErrorCode::InvalidParameter, message)
            .operation("text.word.extract")
            .parameter("positions"),
    )
}
fn positions(values: &Bound<'_, PyList>) -> PyResult<Vec<i64>> {
    if values.len() > 65536 {
        return Err(invalid("positions exceed 65536"));
    }
    values
        .iter()
        .map(|v| {
            if v.is_instance_of::<PyBool>() {
                Err(invalid("positions require exact signed i64 integers"))
            } else {
                v.extract()
                    .map_err(|_| invalid("positions require exact signed i64 integers"))
            }
        })
        .collect()
}
#[pyfunction]
#[pyo3(signature=(text,*,positions=None,per_word=None,index_base=1,short_word="error"))]
fn extract_word_characters(
    py: Python<'_>,
    text: &str,
    positions: Option<&Bound<'_, PyList>>,
    per_word: Option<&Bound<'_, PyList>>,
    index_base: u8,
    short_word: &str,
) -> PyResult<Py<PyAny>> {
    if positions.is_some() == per_word.is_some() {
        return Err(invalid("choose one shared or per-word position list"));
    }
    let shared = positions.map(self::positions).transpose()?;
    let groups = per_word
        .map(|groups| -> PyResult<Vec<Vec<i64>>> {
            if groups.len() > 65536 {
                return Err(invalid("per-word groups exceed 65536"));
            }
            let mut count = 0usize;
            let mut result = Vec::with_capacity(groups.len());
            for group in groups.iter() {
                let group = group.cast::<PyList>()?;
                count = count
                    .checked_add(group.len())
                    .filter(|n| *n <= 65536)
                    .ok_or_else(|| invalid("total positions exceed 65536"))?;
                result.push(self::positions(group)?);
            }
            Ok(result)
        })
        .transpose()?;
    let positions = if let Some(shared) = &shared {
        Positions::Shared(shared)
    } else {
        Positions::PerWord(groups.as_deref().expect("one explicit mode"))
    };
    let report =
        word::extract(text, positions, index_base, short_word).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(report))
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(extract_word_characters, module)?)?;
    Ok(())
}
