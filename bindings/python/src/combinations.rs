//! Native iterator boundary; generation belongs to Rust core.
use cube_core::text::combinations::{self, Combinations};
use pyo3::{prelude::*, types::PyList};
#[pyclass(module = "cube._native")]
struct StringCombinations {
    inner: Combinations,
}
#[pymethods]
impl StringCombinations {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(&mut self) -> Option<String> {
        self.inner.next()
    }
    fn __len__(&self) -> usize {
        self.inner.len()
    }
    fn plan(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let plan = super::structured(py, &serde_json::json!(self.inner.plan()))?;
        plan.bind(py)
            .set_item("remaining_results", self.inner.len())?;
        Ok(plan)
    }
}
fn limit(message: &str) -> PyErr {
    super::file_error(
        cube_core::errors::OperationError::new(
            cube_core::errors::ErrorCode::ResourceLimit,
            message,
        )
        .operation("text.combinations"),
    )
}
#[pyfunction]
#[pyo3(signature=(groups, *, separator="", max_results=1000))]
fn string_combinations(
    groups: &Bound<'_, PyList>,
    separator: &str,
    max_results: usize,
) -> PyResult<StringCombinations> {
    if groups.len() > combinations::MAX_GROUPS
        || separator.len() > combinations::MAX_SEPARATOR_BYTES
    {
        return Err(limit("group/separator input limit exceeded"));
    }
    let mut output = Vec::new();
    let mut fragments = 0usize;
    let mut bytes = separator.len();
    for group in groups.iter() {
        let group = group.cast::<PyList>()?;
        fragments += group.len();
        if fragments > combinations::MAX_FRAGMENTS {
            return Err(limit("input exceeds 4096 fragments"));
        }
        let mut values = Vec::new();
        for value in group.iter() {
            let text = value.extract::<&str>()?;
            bytes = bytes
                .checked_add(text.len())
                .ok_or_else(|| limit("input byte count overflow"))?;
            if text.len() > combinations::MAX_FRAGMENT_BYTES
                || bytes > combinations::MAX_INPUT_BYTES
            {
                return Err(limit("fragment/input byte limit exceeded"));
            }
            values.push(text.to_owned());
        }
        output.push(values);
    }
    Ok(StringCombinations {
        inner: Combinations::new(output, separator.into(), max_results)
            .map_err(super::file_error)?,
    })
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<StringCombinations>()?;
    module.add_function(wrap_pyfunction!(string_combinations, module)?)?;
    Ok(())
}
