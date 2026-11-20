//! Native structured adapters for the shared graph and formal implementations.
use super::pipeline::CancellationToken;
use cube_core::operations::formal;
use pyo3::{prelude::*, types::PyInt};
use std::sync::atomic::Ordering;

fn call(
    py: Python<'_>,
    request: &Bound<'_, PyAny>,
    operation: &'static str,
    cancel: Option<PyRef<'_, CancellationToken>>,
) -> PyResult<Py<PyAny>> {
    let document = super::discrete_transport::typed_document(
        py,
        request,
        formal::MAX_DOCUMENT,
        2_100_000,
        16,
        operation,
        false,
    )?;
    let inputs = formal::parse(&document, operation).map_err(super::file_error)?;
    let flag = cancel.as_ref().map(|token| &token.flag);
    let report = py
        .detach(move || {
            formal::invoke_transport_with_cancel(operation, inputs, &mut || {
                flag.is_some_and(|flag| flag.load(Ordering::Relaxed))
            })
        })
        .map_err(super::file_error)?;
    let output = super::structured(py, &report)?;
    if matches!(
        operation,
        "graph.weighted_path" | "graph.all_pairs_paths" | "number.lfsr"
    ) {
        super::numeric_analysis::restore(
            output.bind(py),
            &report,
            "",
            operation,
            py.get_type::<PyInt>().as_any(),
        )?;
    }
    Ok(output)
}

macro_rules! operations {
    ($($name:ident => $id:literal,)* | $($polled:ident => $polled_id:literal,)*) => {
        $(
            #[pyfunction]
            fn $name(py: Python<'_>, request: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
                call(py, request, $id, None)
            }
        )*
        $(
            #[pyfunction]
            #[pyo3(signature = (request, *, cancel = None))]
            fn $polled(py: Python<'_>, request: &Bound<'_, PyAny>, cancel: Option<PyRef<'_, CancellationToken>>) -> PyResult<Py<PyAny>> {
                call(py, request, $polled_id, cancel)
            }
        )*
        pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
            $(module.add_function(wrap_pyfunction!($name, module)?)?;)*
            $(module.add_function(wrap_pyfunction!($polled, module)?)?;)*
            Ok(())
        }
    };
}

operations! {
    graph_traversal => "graph.traversal",
    graph_weighted_path => "graph.weighted_path",
    graph_all_pairs_paths => "graph.all_pairs_paths",
    graph_disjoint_sets => "graph.disjoint_sets",
    graph_bipartite_matching => "graph.bipartite_matching",
    esolang_cellular_automaton => "esolang.cellular_automaton",
    number_lfsr => "number.lfsr",
    number_prng => "number.prng",
    |
    logic_exact_cover => "logic.exact_cover",
    logic_dpll => "logic.dpll",
    logic_cdcl => "logic.cdcl",
    esolang_lambda => "esolang.lambda",
    esolang_ski => "esolang.ski",
}
