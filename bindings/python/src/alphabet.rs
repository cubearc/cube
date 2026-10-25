//! Native custom scalar radix adapter.
use pyo3::prelude::*;
#[pyfunction]
fn convert_alphabet(
    text: &str,
    source_alphabet: &str,
    target_alphabet: &str,
    from_base: u32,
    to_base: u32,
) -> PyResult<String> {
    cube_core::numbers::alphabet::convert(
        text,
        source_alphabet,
        target_alphabet,
        from_base,
        to_base,
    )
    .map_err(super::file_error)
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(convert_alphabet, module)?)?;
    Ok(())
}
