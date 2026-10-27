//! Exact native integers, including u128 masses; model data is bounded before dumps.
use super::discrete_transport as t;
use cube_core::probability as p;
use pyo3::{
    IntoPyObjectExt,
    prelude::*,
    types::{PyDict, PyList},
};
fn limits(v: Option<&Bound<'_, PyDict>>, op: &'static str) -> PyResult<p::Limits> {
    let v = t::config(v, &["max_work", "max_memory_bytes", "max_report_bytes"], op)?;
    Ok(p::Limits {
        max_work: t::size(v, "max_work", p::MAX_WORK, op)?,
        max_memory_bytes: t::size(v, "max_memory_bytes", p::MAX_MEMORY, op)?,
        max_report_bytes: t::size(v, "max_report_bytes", p::MAX_REPORT, op)?,
    })
}
// Convert only schema-owned exact decimal fields; caller IDs remain strings.
fn native(py: Python<'_>, v: &serde_json::Value, key: &str) -> PyResult<Py<PyAny>> {
    use serde_json::Value;
    match v {
        Value::String(s)
            if [
                "numerator",
                "denominator",
                "weight",
                "mass",
                "total_mass",
                "win_mass",
                "tie_mass",
                "loss_mass",
            ]
            .contains(&key) =>
        {
            s.parse::<u128>()
                .expect("Core exact decimal")
                .into_py_any(py)
        }
        Value::Object(m) => {
            let d = PyDict::new(py);
            for (k, v) in m {
                d.set_item(k, native(py, v, k)?)?;
            }
            Ok(d.into_any().unbind())
        }
        Value::Array(a) => {
            let list = PyList::empty(py);
            for v in a {
                list.append(native(py, v, "")?)?;
            }
            Ok(list.into_any().unbind())
        }
        Value::Null => Ok(py.None()),
        Value::Bool(v) => v.into_py_any(py),
        Value::String(s) => s.into_py_any(py),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into_py_any(py)
            } else if let Some(i) = n.as_u64() {
                i.into_py_any(py)
            } else {
                n.as_f64().expect("finite number").into_py_any(py)
            }
        }
    }
}
fn run(
    py: Python<'_>,
    model: &Bound<'_, PyAny>,
    l: Option<&Bound<'_, PyDict>>,
    kind: u8,
) -> PyResult<Py<PyAny>> {
    let op = match kind {
        0 => "probability.finite.analyze",
        1 => "probability.dice.sum",
        _ => "probability.dice.compare",
    };
    let limits = limits(l, op)?;
    let s = t::document(py, model, p::MAX_INPUT, 65536, 8, op)?;
    let r = match kind {
        0 => serde_json::json!(
            p::finite_analyze(&p::parse_finite(&s).map_err(super::file_error)?, limits)
                .map_err(super::file_error)?
        ),
        1 => serde_json::json!(
            p::dice_sum(&p::parse_dice(&s).map_err(super::file_error)?, limits)
                .map_err(super::file_error)?
        ),
        _ => serde_json::json!(
            p::dice_compare(&p::parse_pair(&s).map_err(super::file_error)?, limits)
                .map_err(super::file_error)?
        ),
    };
    native(py, &r, "")
}
#[pyfunction]
#[pyo3(signature=(model,*,limits=None))]
fn finite_probability(
    py: Python<'_>,
    model: &Bound<'_, PyAny>,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    run(py, model, limits, 0)
}
#[pyfunction]
#[pyo3(signature=(model,*,limits=None))]
fn dice_sum(
    py: Python<'_>,
    model: &Bound<'_, PyAny>,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    run(py, model, limits, 1)
}
#[pyfunction]
#[pyo3(signature=(model,*,limits=None))]
fn dice_compare(
    py: Python<'_>,
    model: &Bound<'_, PyAny>,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    run(py, model, limits, 2)
}
pub(super) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(finite_probability, m)?)?;
    m.add_function(wrap_pyfunction!(dice_sum, m)?)?;
    m.add_function(wrap_pyfunction!(dice_compare, m)?)?;
    Ok(())
}
