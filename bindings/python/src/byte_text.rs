//! Native Python adapters over the exact Core byte encodings.
use cube_core::encoding::byte_text;
use pyo3::{prelude::*, types::PyBytes};

macro_rules! byte_text_bindings {
    ($(($encode:ident, $decode:ident)),+ $(,)?) => {
        $(
            #[pyfunction]
            fn $encode(data: &[u8]) -> PyResult<String> {
                byte_text::$encode(data).map_err(super::file_error)
            }
            #[pyfunction]
            fn $decode<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, PyBytes>> {
                let value = byte_text::$decode(text).map_err(super::file_error)?;
                Ok(PyBytes::new(py, &value))
            }
        )+
        pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
            $(
                module.add_function(wrap_pyfunction!($encode, module)?)?;
                module.add_function(wrap_pyfunction!($decode, module)?)?;
            )+
            Ok(())
        }
    };
}
byte_text_bindings![
    (binary_encode, binary_decode),
    (octal_encode, octal_decode),
    (base36_encode, base36_decode),
    (base62_encode, base62_decode),
    (ascii85_encode, ascii85_decode),
    (ascii85_bare_encode, ascii85_bare_decode),
    (base91_encode, base91_decode),
    (base92_encode, base92_decode),
    (url_component_encode, url_component_decode),
];
