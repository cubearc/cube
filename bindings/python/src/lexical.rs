//! Bounded dictionary/profile transport; algorithms are shared Rust APIs.
use super::discrete_transport as t;
use cube_core::text::{
    anagram as a, lexical, multiset::Profile, statistics::TextFilter, word_ladder as l,
};
use pyo3::{
    prelude::*,
    types::{PyBool, PyDict},
};
fn words(
    py: Python<'_>,
    v: &Bound<'_, PyAny>,
    max: usize,
    entries: usize,
    op: &'static str,
) -> PyResult<Vec<String>> {
    let s = t::document(py, v, max, entries + 1, 2, op)?;
    lexical::parse_dictionary(&s, max, entries).map_err(|e| super::file_error(e.operation(op)))
}
fn ladder_limits(v: Option<&Bound<'_, PyDict>>, op: &'static str) -> PyResult<l::Options> {
    let v = t::config(v, &["max_work", "max_memory_bytes", "max_report_bytes"], op)?;
    Ok(l::Options {
        max_work: t::size(v, "max_work", l::MAX_WORK, op)?,
        max_memory_bytes: t::size(v, "max_memory_bytes", l::MAX_MEMORY, op)?,
        max_report_bytes: t::size(v, "max_report_bytes", l::MAX_REPORT, op)?,
    })
}
#[pyfunction]
#[pyo3(signature=(source,dictionary,*,profile=None,options=None))]
fn anagram_search(
    py: Python<'_>,
    source: &str,
    dictionary: &Bound<'_, PyAny>,
    profile: Option<&Bound<'_, PyDict>>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    const OP: &str = "text.anagram.search";
    let v = t::config(
        options,
        &[
            "max_words",
            "reuse",
            "max_results",
            "max_work",
            "max_memory_bytes",
            "max_report_bytes",
        ],
        OP,
    )?;
    let reuse = if let Some(v) = v
        .and_then(|d| d.get_item("reuse").transpose())
        .transpose()?
    {
        v.cast::<PyBool>()
            .map_err(|_| t::invalid(OP, "reuse", "reuse must be a builtin Boolean"))?
            .extract::<bool>()?
    } else {
        false
    };
    let options = a::Options {
        max_words: t::size(v, "max_words", 8, OP)?,
        reuse,
        max_results: t::size(v, "max_results", 100, OP)?,
        max_work: t::size(v, "max_work", a::MAX_WORK, OP)?,
        max_memory_bytes: t::size(v, "max_memory_bytes", a::MAX_MEMORY, OP)?,
        max_report_bytes: t::size(v, "max_report_bytes", a::MAX_REPORT, OP)?,
    };
    let p = super::multiset::profile(profile, OP)?;
    let d = words(py, dictionary, a::MAX_DICTIONARY_BYTES, 10000, OP)?;
    let r = a::search(source, &d, p, options).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(r))
}
fn profile(v: Option<&Bound<'_, PyDict>>, op: &'static str) -> PyResult<Profile> {
    t::config(v, &["normalization", "case"], op)?;
    let mut p = super::multiset::profile(v, op)?;
    p.filter = TextFilter::All;
    Ok(p)
}
#[pyfunction]
#[pyo3(signature=(dictionary,path,*,profile=None,limits=None))]
fn word_ladder_validate(
    py: Python<'_>,
    dictionary: &Bound<'_, PyAny>,
    path: &Bound<'_, PyAny>,
    profile: Option<&Bound<'_, PyDict>>,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    const OP: &str = "text.word_ladder.validate";
    let limits = ladder_limits(limits, OP)?;
    let profile = self::profile(profile, OP)?;
    let words = words(py, dictionary, l::MAX_DICTIONARY_BYTES, 10000, OP)?;
    let path = self::words(py, path, l::MAX_DICTIONARY_BYTES, 4096, OP)?;
    let r = l::validate(&words, &path, profile, limits).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(r))
}
#[pyfunction]
#[pyo3(signature=(dictionary,start,end,*,profile=None,limits=None))]
fn word_ladder_shortest(
    py: Python<'_>,
    dictionary: &Bound<'_, PyAny>,
    start: &str,
    end: &str,
    profile: Option<&Bound<'_, PyDict>>,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    const OP: &str = "text.word_ladder.shortest";
    let limits = ladder_limits(limits, OP)?;
    let profile = self::profile(profile, OP)?;
    let words = words(py, dictionary, l::MAX_DICTIONARY_BYTES, 10000, OP)?;
    let r = l::shortest(&words, start, end, profile, limits).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(r))
}
pub(super) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(anagram_search, m)?)?;
    m.add_function(wrap_pyfunction!(word_ladder_validate, m)?)?;
    m.add_function(wrap_pyfunction!(word_ladder_shortest, m)?)?;
    Ok(())
}
