//! Python wrappers over strict character-set and escape operations.
use super::file_error;
use cube_core::text::representation as r;
use pyo3::{prelude::*, types::PyBytes};

#[pyfunction]
fn ascii_encode<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, PyBytes>> {
    Ok(PyBytes::new(
        py,
        &r::ascii_encode(text).map_err(file_error)?,
    ))
}
#[pyfunction]
fn ascii_decode(data: &[u8]) -> PyResult<String> {
    r::ascii_decode(data).map_err(file_error)
}
#[pyfunction(signature = (text, *, byte_order, bom))]
fn utf16_encode<'py>(
    py: Python<'py>,
    text: &str,
    byte_order: &str,
    bom: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let settings = r::Utf16Settings::parse(byte_order, bom)
        .map_err(|error| file_error(error.operation("text.utf16.encode")))?;
    Ok(PyBytes::new(
        py,
        &r::utf16_encode(text, settings).map_err(file_error)?,
    ))
}
#[pyfunction(signature = (data, *, byte_order, bom))]
fn utf16_decode(data: &[u8], byte_order: &str, bom: &str) -> PyResult<String> {
    let settings = r::Utf16Settings::parse(byte_order, bom)
        .map_err(|error| file_error(error.operation("text.utf16.decode")))?;
    r::utf16_decode(data, settings).map_err(file_error)
}
#[pyfunction(signature = (text, *, ascii_only=true))]
fn json_escape_encode(text: &str, ascii_only: bool) -> PyResult<String> {
    r::json_escape_encode(text, ascii_only).map_err(file_error)
}
#[pyfunction]
fn json_escape_decode(text: &str) -> PyResult<String> {
    r::json_escape_decode(text).map_err(file_error)
}
#[pyfunction]
fn html_entity_encode(text: &str) -> PyResult<String> {
    r::html_entity_encode(text).map_err(file_error)
}
#[pyfunction]
fn html_entity_decode(text: &str) -> PyResult<String> {
    r::html_entity_decode(text).map_err(file_error)
}
#[pyfunction(signature = (data, *, profile, component_id, start_bit, max_bits))]
fn image_jpeg_coefficients(
    py: Python<'_>,
    data: &[u8],
    profile: &str,
    component_id: u8,
    start_bit: u32,
    max_bits: u32,
) -> PyResult<Py<PyAny>> {
    use cube_core::image::jpeg_coefficients as jpeg;
    let parameters = serde_json::from_value(serde_json::json!({
        "profile":profile,"component_id":component_id,"start_bit":start_bit,"max_bits":max_bits,
    }))
    .map_err(|_| {
        super::file_error(
            cube_core::errors::OperationError::new(
                cube_core::errors::ErrorCode::InvalidParameter,
                "use the exact JPEG coefficient settings",
            )
            .operation(jpeg::EXTRACT),
        )
    })?;
    let result = jpeg::extract(data, parameters).map_err(super::file_error)?;
    let value = super::structured(py, &serde_json::json!({"report":result.report}))?;
    value
        .bind(py)
        .set_item("bits", PyBytes::new(py, &result.bits))?;
    Ok(value)
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(ascii_encode, module)?)?;
    module.add_function(wrap_pyfunction!(image_jpeg_coefficients, module)?)?;
    module.add_function(wrap_pyfunction!(ascii_decode, module)?)?;
    module.add_function(wrap_pyfunction!(utf16_encode, module)?)?;
    module.add_function(wrap_pyfunction!(utf16_decode, module)?)?;
    module.add_function(wrap_pyfunction!(json_escape_encode, module)?)?;
    module.add_function(wrap_pyfunction!(json_escape_decode, module)?)?;
    module.add_function(wrap_pyfunction!(html_entity_encode, module)?)?;
    module.add_function(wrap_pyfunction!(html_entity_decode, module)?)?;
    Ok(())
}
