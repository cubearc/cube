//! Native Python adapters to the shared typed sequence operations.
use cube_core::sequence;
use pyo3::prelude::*;

fn call(
    py: Python<'_>,
    request: &Bound<'_, PyAny>,
    operation: &'static str,
) -> PyResult<Py<PyAny>> {
    let document = super::discrete_transport::typed_document(
        py,
        request,
        sequence::MAX_DOCUMENT,
        262_144,
        12,
        operation,
        true,
    )?;
    let inputs = sequence::parse(&document).map_err(super::file_error)?;
    let report = sequence::invoke(operation, inputs).map_err(super::file_error)?;
    super::structured(py, &report)
}

macro_rules! native_sequence {
    ($($name:ident => $id:literal,)*) => {
        $(
            #[pyfunction]
            fn $name(py: Python<'_>, request: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
                call(py, request, concat!("sequence.", $id))
            }
        )*
        pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
            $(module.add_function(wrap_pyfunction!($name, module)?)?;)*
            Ok(())
        }
    };
}

native_sequence! {
    sequence_hamming => "hamming",
    sequence_edit_distance => "edit_distance",
    sequence_jaro => "jaro",
    sequence_lcs => "lcs",
    sequence_common_spans => "common_spans",
    sequence_ratcliff_obershelp => "ratcliff_obershelp",
    sequence_monge_elkan => "monge_elkan",
    sequence_align => "align",
    sequence_dtw => "dtw",
    sequence_set_similarity => "set_similarity",
    sequence_vector_similarity => "vector_similarity",
    sequence_tfidf_model => "tfidf_model",
    sequence_tfidf_compare => "tfidf_compare",
    sequence_exact_search => "exact_search",
    sequence_multi_search => "multi_search",
    sequence_bitap => "bitap",
    sequence_diff => "diff",
}
