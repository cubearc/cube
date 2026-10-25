//! Thin native recipe boundary. Definitions cannot grant access to Python objects or I/O.
use crate::pipeline::{CancellationToken, error, execution, input};
use cube_core::pipeline::{self, Value, recipe::Recipe};
use pyo3::{prelude::*, types::PyDict};
use std::collections::BTreeMap;
fn bindings(py: Python<'_>, values: &Bound<'_, PyDict>) -> PyResult<BTreeMap<String, Value>> {
    if values.len() > 1 {
        return Err(error(
            py,
            pipeline::Error::new(
                cube_core::errors::ErrorCode::InvalidParameter,
                "recipe format 1 permits at most one named input binding",
            ),
        ));
    }
    values
        .iter()
        .map(|(name, value)| {
            let name: String = name.extract()?;
            pipeline::recipe::validate_slot(&name).map_err(|e| error(py, e))?;
            Ok((name, input(py, &value)?))
        })
        .collect()
}
#[pyfunction]
fn validate_recipe(py: Python<'_>, document: &str) -> PyResult<Py<PyAny>> {
    let recipe = Recipe::parse(document).map_err(|e| error(py, e))?;
    super::structured(py, &serde_json::json!(recipe.info()))
}
#[pyfunction]
#[pyo3(signature=(document, inputs, *, capture=None, cancellation=None))]
fn execute_recipe(
    py: Python<'_>,
    document: &str,
    inputs: &Bound<'_, PyDict>,
    capture: Option<Vec<usize>>,
    cancellation: Option<&CancellationToken>,
) -> PyResult<Py<PyAny>> {
    let recipe = Recipe::parse(document).map_err(|e| error(py, e))?;
    let bindings = bindings(py, inputs)?;
    let capture = capture.unwrap_or_default();
    let token = cancellation.map(|c| &c.flag);
    let result = py
        .detach(|| recipe.execute(&bindings, &capture, token))
        .map_err(|e| error(py, e))?;
    execution(py, &result)
}
#[pyfunction]
fn check_recipe(
    py: Python<'_>,
    document: &str,
    inputs: &Bound<'_, PyDict>,
    expected: &Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    let recipe = Recipe::parse(document).map_err(|e| error(py, e))?;
    let bindings = bindings(py, inputs)?;
    let expected = input(py, expected)?;
    let result = py
        .detach(|| recipe.check(&bindings, &expected))
        .map_err(|e| error(py, e))?;
    let report = super::structured(py, &serde_json::json!(result))?;
    if !result.masked_audits.is_empty() {
        let audits = PyDict::new(py);
        for (index, audit) in &result.masked_audits {
            audits.set_item(index, super::structured(py, &serde_json::json!(audit))?)?;
        }
        report.bind(py).set_item("masked_audits", audits)?;
    }
    Ok(report)
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(validate_recipe, module)?)?;
    module.add_function(wrap_pyfunction!(execute_recipe, module)?)?;
    module.add_function(wrap_pyfunction!(check_recipe, module)?)?;
    Ok(())
}
