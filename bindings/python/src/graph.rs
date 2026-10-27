//! Concrete graph/selection transport and exact native structured evidence.
use super::discrete_transport as t;
use cube_core::graph::{self, Graph, Limits};
use pyo3::{prelude::*, types::PyDict};
fn load(
    py: Python<'_>,
    v: &Bound<'_, PyAny>,
    limits: Option<&Bound<'_, PyDict>>,
    op: &'static str,
) -> PyResult<(Graph, Limits)> {
    let l = t::config(
        limits,
        &["max_work", "max_memory_bytes", "max_report_bytes"],
        op,
    )?;
    let l = Limits {
        max_work: t::number(l, "max_work", graph::MAX_WORK, op)?,
        max_memory_bytes: t::size(l, "max_memory_bytes", graph::MAX_MEMORY, op)?,
        max_report_bytes: t::size(l, "max_report_bytes", graph::MAX_REPORT, op)?,
    };
    let g = graph::parse(&t::document(py, v, graph::MAX_INPUT, 262144, 8, op)?)
        .and_then(|d| Graph::new(d, l))
        .map_err(super::file_error)?;
    Ok((g, l))
}
#[pyfunction]
#[pyo3(signature=(graph,*,limits=None))]
fn graph_components(
    py: Python<'_>,
    graph: &Bound<'_, PyAny>,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    let (g, l) = load(py, graph, limits, "graph.components")?;
    let r = g.components(l).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(r))
}
#[pyfunction]
#[pyo3(signature=(graph,source,target,*,limits=None))]
fn graph_shortest_path(
    py: Python<'_>,
    graph: &Bound<'_, PyAny>,
    source: &str,
    target: &str,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    let (g, l) = load(py, graph, limits, "graph.shortest_path")?;
    let r = g
        .shortest_path(source, target, l)
        .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(r))
}
#[pyfunction]
#[pyo3(signature=(graph,*,limits=None))]
fn graph_cycle(
    py: Python<'_>,
    graph: &Bound<'_, PyAny>,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    let (g, l) = load(py, graph, limits, "graph.cycle")?;
    let r = g.cycle(l).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(r))
}
#[pyfunction]
#[pyo3(signature=(graph,*,start=None,limits=None))]
fn graph_euler_trail(
    py: Python<'_>,
    graph: &Bound<'_, PyAny>,
    start: Option<&str>,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    let (g, l) = load(py, graph, limits, "graph.euler_trail")?;
    let r = g.euler_trail(start, l).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(r))
}
pub(super) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(graph_components, m)?)?;
    m.add_function(wrap_pyfunction!(graph_shortest_path, m)?)?;
    m.add_function(wrap_pyfunction!(graph_cycle, m)?)?;
    m.add_function(wrap_pyfunction!(graph_euler_trail, m)?)?;
    Ok(())
}
