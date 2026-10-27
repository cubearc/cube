//! Exact builtin-integer transport for bounded combinatorial counts.
use super::discrete_transport as t;
use cube_core::numbers::combinatorics as c;
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
fn limits(v: Option<&Bound<'_, PyDict>>, op: &'static str) -> PyResult<c::Limits> {
    let v = t::config(v, &["max_work", "max_report_bytes"], op)?;
    Ok(c::Limits {
        max_work: t::number(v, "max_work", c::MAX_WORK, op)?,
        max_report_bytes: t::size(v, "max_report_bytes", c::MAX_REPORT_BYTES, op)?,
    })
}
#[pyfunction]
#[pyo3(signature=(n,k,*,limits=None))]
fn permutations(
    n: &Bound<'_, PyAny>,
    k: &Bound<'_, PyAny>,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<u128> {
    const OP: &str = "number.permutations";
    c::permutations(
        super::modular::operand(n, OP, "n")?,
        super::modular::operand(k, OP, "k")?,
        self::limits(limits, OP)?,
    )
    .map(|r| r.value)
    .map_err(super::file_error)
}
#[pyfunction]
#[pyo3(signature=(n,k,*,limits=None))]
fn binomial(
    n: &Bound<'_, PyAny>,
    k: &Bound<'_, PyAny>,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<u128> {
    const OP: &str = "number.binomial";
    c::binomial(
        super::modular::operand(n, OP, "n")?,
        super::modular::operand(k, OP, "k")?,
        self::limits(limits, OP)?,
    )
    .map(|r| r.value)
    .map_err(super::file_error)
}
#[pyfunction]
#[pyo3(signature=(groups,*,limits=None))]
fn multinomial(groups: &Bound<'_, PyList>, limits: Option<&Bound<'_, PyDict>>) -> PyResult<u128> {
    const OP: &str = "number.multinomial";
    if groups.len() > c::MAX_GROUPS {
        return Err(t::invalid(OP, "groups", "at most 128 groups are supported"));
    }
    let groups = groups
        .iter()
        .map(|v| super::modular::operand(&v, OP, "groups"))
        .collect::<PyResult<Vec<_>>>()?;
    c::multinomial(&groups, self::limits(limits, OP)?)
        .map(|r| r.value)
        .map_err(super::file_error)
}
#[pyfunction]
#[pyo3(signature=(rows,*,limits=None))]
fn pascal_triangle(
    rows: &Bound<'_, PyAny>,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<Vec<Vec<u128>>> {
    const OP: &str = "number.pascal";
    let limits = t::config(limits, &["max_work", "max_output_bytes"], OP)?;
    c::pascal(
        super::modular::operand(rows, OP, "rows")?,
        t::number(limits, "max_work", c::MAX_WORK, OP)?,
        t::size(limits, "max_output_bytes", c::MAX_TRIANGLE_BYTES, OP)?,
    )
    .map(|r| r.rows)
    .map_err(super::file_error)
}
pub(super) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(permutations, m)?)?;
    m.add_function(wrap_pyfunction!(binomial, m)?)?;
    m.add_function(wrap_pyfunction!(multinomial, m)?)?;
    m.add_function(wrap_pyfunction!(pascal_triangle, m)?)?;
    Ok(())
}
