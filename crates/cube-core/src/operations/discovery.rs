//! Bounded views of the existing static catalog; no algorithm dispatcher.
use super::*;
use crate::errors::{ErrorCode, OperationError};
use std::io::{self, Write};
/// Hard ceiling for one discovery response, including CLI envelopes/whitespace.
pub const MAX_DISCOVERY_BYTES: usize = 1024 * 1024;
/// Query text is a category/ID, not data or a secret-valued operation parameter.
pub const MAX_DISCOVERY_QUERY_BYTES: usize = 256;
/// List and describe the existing catalog. Unknown queries retain typed context.
#[derive(Clone, Copy, Debug)]
pub enum DiscoveryRequest<'a> {
    List { category: Option<&'a str> },
    Describe { id: &'a str },
}
#[derive(Clone, Copy, Debug)]
pub enum DiscoveryFormat {
    /// Compact array/object used by native Python, without a CLI envelope.
    DirectJson { metadata: bool },
    /// Existing schema_version=1 envelope and pretty-printed CLI JSON.
    CliJson { metadata: bool },
    /// Existing command list or human description, extended with profile facts.
    Text,
}
struct Output {
    data: Vec<u8>,
    max: usize,
}
impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self
            .data
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > self.max)
        {
            return Err(io::Error::other("discovery output budget exceeded"));
        }
        self.data.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn limit(op: &'static str, name: &'static str, message: &str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message)
        .operation(op)
        .parameter(name)
}
/// Serialize one bounded catalog view. Limits may only lower 1 MiB. Bound checks
/// occur before appending each chunk, never after allocating a huge JSON string.
/// Scratch is <=2*max_output_bytes + 64 KiB for current catalog views; CPython
/// object overhead is outside this Core encoding budget. Output errors return no
/// document, so CLI/SDK callers cannot mistake a partial prefix for valid data.
pub fn discovery_document(
    request: DiscoveryRequest<'_>,
    format: DiscoveryFormat,
    max_output_bytes: usize,
) -> Result<String, OperationError> {
    let (op, query, name) = match request {
        DiscoveryRequest::List { category } => ("ops.list", category, "category"),
        DiscoveryRequest::Describe { id } => ("ops.describe", Some(id), "id"),
    };
    if max_output_bytes > MAX_DISCOVERY_BYTES {
        return Err(OperationError::new(
            ErrorCode::InvalidParameter,
            "discovery limit exceeds 1 MiB",
        )
        .operation(op)
        .parameter("max_output_bytes"));
    }
    if query.is_some_and(|q| q.len() > MAX_DISCOVERY_QUERY_BYTES) {
        return Err(limit(op, name, "discovery query exceeds 256 UTF-8 bytes"));
    }
    let mut out = Output {
        data: Vec::new(),
        max: max_output_bytes,
    };
    let failed = || {
        limit(
            op,
            "max_output_bytes",
            "complete discovery view exceeds output budget",
        )
    };
    match request {
        DiscoveryRequest::List { category } => {
            let mut rows = filtered(category)?;
            match format {
                DiscoveryFormat::Text => {
                    let mut seen = std::collections::BTreeSet::new();
                    for d in rows {
                        if seen.insert(d.command) {
                            writeln!(out, "{}", d.command).map_err(|_| failed())?;
                        }
                    }
                }
                _ => {
                    rows.sort_by_key(|d| d.id);
                    #[derive(Serialize)]
                    struct List<T> {
                        schema_version: u8,
                        operations: T,
                    }
                    match format {
                        DiscoveryFormat::CliJson { metadata: true } => {
                            let views: Vec<_> =
                                rows.iter().map(|d| d.profiled_discovery()).collect();
                            serde_json::to_writer_pretty(
                                &mut out,
                                &List {
                                    schema_version: 1,
                                    operations: views,
                                },
                            )
                            .map_err(|_| failed())?;
                        }
                        DiscoveryFormat::CliJson { metadata: false } => {
                            let views: Vec<_> = rows.iter().map(|d| d.discovery()).collect();
                            serde_json::to_writer_pretty(
                                &mut out,
                                &List {
                                    schema_version: 1,
                                    operations: views,
                                },
                            )
                            .map_err(|_| failed())?;
                        }
                        DiscoveryFormat::DirectJson { metadata: true } => {
                            let views: Vec<_> =
                                rows.iter().map(|d| d.profiled_discovery()).collect();
                            serde_json::to_writer(&mut out, &views).map_err(|_| failed())?;
                        }
                        DiscoveryFormat::DirectJson { metadata: false } => {
                            let views: Vec<_> = rows.iter().map(|d| d.discovery()).collect();
                            serde_json::to_writer(&mut out, &views).map_err(|_| failed())?;
                        }
                        DiscoveryFormat::Text => unreachable!("text list was handled above"),
                    }
                }
            }
        }
        DiscoveryRequest::Describe { id } => {
            let d = require(id)?;
            match format {
                DiscoveryFormat::DirectJson { metadata: true } => {
                    serde_json::to_writer(&mut out, &d.profiled_discovery())
                        .map_err(|_| failed())?
                }
                DiscoveryFormat::DirectJson { metadata: false } => {
                    serde_json::to_writer(&mut out, &d.discovery()).map_err(|_| failed())?
                }
                DiscoveryFormat::CliJson { metadata } => {
                    #[derive(Serialize)]
                    struct One<T> {
                        schema_version: u8,
                        operation: T,
                    }
                    if metadata {
                        serde_json::to_writer_pretty(
                            &mut out,
                            &One {
                                schema_version: 1,
                                operation: d.profiled_discovery(),
                            },
                        )
                        .map_err(|_| failed())?;
                    } else {
                        serde_json::to_writer_pretty(
                            &mut out,
                            &One {
                                schema_version: 1,
                                operation: d.discovery(),
                            },
                        )
                        .map_err(|_| failed())?;
                    }
                }
                DiscoveryFormat::Text => {
                    writeln!(
                        out,
                        "{} — {}\nCategory: {}\nInput: {}\nOutput: {}\nDeterministic: {}\nReversible: {}\nPipeline: {}\nExample: {}",
                        d.id, d.description, d.category, d.input_type, d.output_type,
                        d.deterministic,
                        d.reversible.map_or("not applicable", |v| if v { "yes" } else { "no" }),
                        d.discovery().pipeline.map_or_else(
                            || "unavailable (no bounded adapter)".into(),
                            |s| format!("adapter version {}", s.version),
                        ),
                        d.example,
                    ).map_err(|_| failed())?;
                    for p in d.parameters {
                        writeln!(
                            out,
                            "{}: {} (required: {}, constraints: {}, default: {})",
                            p.name,
                            p.description,
                            p.required,
                            p.constraints,
                            p.default
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "none".into())
                        )
                        .map_err(|_| failed())?;
                    }
                    if let Some(m) = d.metadata() {
                        writeln!(
                            out,
                            "Canonical name: {}\nFamily: {}\nSecurity: {:?}\nLimitations: {}\nProfile: {}\nAlphabet: {}\nKey: {}\nKey advancement: {}\nRust API: {}\nPython API: {}\nResource notes: {}",
                            m.canonical_name, m.family, m.security.classification,
                            m.security.limitations, m.profile.variant, m.profile.alphabet,
                            m.profile.key, m.profile.key_advancement, m.rust_api,
                            m.python_api, m.resource_notes,
                        ).map_err(|_| failed())?;
                        for a in m.aliases {
                            writeln!(out, "Alias: {} — {}", a.name, a.usage)
                                .map_err(|_| failed())?;
                        }
                        for b in m.resources {
                            writeln!(
                                out,
                                "Bound: {} <= {} {} ({:?})",
                                b.name, b.maximum, b.unit, b.scope
                            )
                            .map_err(|_| failed())?;
                        }
                    }
                }
            }
        }
    }
    if !matches!(format, DiscoveryFormat::Text) {
        out.write_all(b"\n").map_err(|_| failed())?;
    }
    Ok(String::from_utf8(out.data).expect("UTF-8 static metadata and serde JSON"))
}
