//! Native Python adapters to the shared typed text profiles.
use cube_core::text::profiles;
use pyo3::prelude::*;

fn call(
    py: Python<'_>,
    request: &Bound<'_, PyAny>,
    operation: &'static str,
) -> PyResult<Py<PyAny>> {
    let document = super::discrete_transport::typed_document(
        py,
        request,
        profiles::MAX_DOCUMENT,
        262_144,
        12,
        operation,
        true,
    )?;
    let inputs = profiles::parse(&document).map_err(super::file_error)?;
    let report = profiles::invoke(operation, inputs).map_err(super::file_error)?;
    super::structured(py, &report)
}

macro_rules! native_profiles {
    ($($name:ident => $id:literal,)*) => {
        $(
            #[pyfunction]
            fn $name(py: Python<'_>, request: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
                call(py, request, concat!("text.", $id))
            }
        )*
        pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
            $(module.add_function(wrap_pyfunction!($name, module)?)?;)*
            Ok(())
        }
    };
}

native_profiles! {
    text_unicode_transform => "unicode_transform",
    text_graphemes => "graphemes",
    text_regex => "regex",
    text_tokenize => "tokenize",
    text_scalar_weave => "scalar_weave",
    text_select_words => "select_words",
    text_sort => "sort",
    text_phonetic => "phonetic",
    text_match_rating => "match_rating",
    text_daitch_mokotoff => "daitch_mokotoff",
    text_beider_morse => "beider_morse",
}
