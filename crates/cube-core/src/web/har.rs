use super::*;
use crate::{operations::Descriptor, pipeline::json};
use serde::Serialize;
use serde_json::Value;
const OP: &str = "web.har.inspect";
pub const MAX_ENTRIES: usize = 128;
#[derive(Debug, Serialize)]
pub struct Entry {
    pub index: usize,
    pub pointer: String,
    pub method: String,
    pub url: String,
    pub reference_kind: &'static str,
    pub observations: Vec<&'static str>,
    pub status: u16,
    pub mime_type: Option<String>,
    pub declared_size: Option<i64>,
    pub started_date_time: Option<String>,
    pub time_ms: Option<f64>,
    pub body_text_bytes: Option<usize>,
    pub body_encoding: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub format: &'static str,
    pub version: String,
    pub input_bytes: usize,
    pub entries: Vec<Entry>,
    pub scope: &'static str,
}
pub(super) fn parse(document: &str) -> Result<Value, OperationError> {
    if document.len() > json::MAX_DOCUMENT_BYTES {
        return Err(limit(OP, "HAR input exceeds 256 KiB"));
    }
    let document = document.strip_prefix('\u{feff}').unwrap_or(document);
    let value = json::parse(document).map_err(|e| {
        OperationError::new(
            e.error.code,
            "invalid bounded HAR JSON; syntax, duplicate keys, depth/node/size limits are enforced",
        )
        .operation(OP)
        .caused_by(e)
    })?;
    let mut pending = vec![&value];
    while let Some(v) = pending.pop() {
        match v {
            Value::String(s) if s.len() > MAX_FIELD => {
                return Err(limit(OP, "HAR JSON string exceeds 64 KiB"));
            }
            Value::Object(map) => {
                for (k, v) in map {
                    if k.len() > 256 {
                        return Err(limit(OP, "HAR JSON key exceeds 256 bytes"));
                    }
                    pending.push(v);
                }
            }
            Value::Array(a) => pending.extend(a),
            _ => {}
        }
    }
    let log = value
        .get("log")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid(OP, "HAR requires a log object"))?;
    if log.get("version").and_then(Value::as_str) != Some("1.2") {
        return Err(unsupported(
            OP,
            "only the documented HAR 1.2 subset is supported",
        ));
    }
    let entries = log
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid(OP, "HAR log.entries must be an array"))?;
    if entries.len() > MAX_ENTRIES {
        return Err(limit(OP, "HAR exceeds 128 entries"));
    }
    Ok(value)
}
fn string(v: Option<&Value>, max: usize) -> Result<Option<String>, OperationError> {
    match v {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => {
            if s.len() > max {
                return Err(limit(OP, "HAR metadata field exceeds documented length"));
            }
            Ok(Some(s.clone()))
        }
        _ => Err(invalid(OP, "HAR metadata field must be text or null")),
    }
}
/// Inspect a minimal HAR 1.2 request/response metadata subset. URLs stay data.
/// Headers/cookies/post data/body text are not exported. Absent metadata is null;
/// body availability/encoding is described without decoding. Reuse bounded JSON
/// parsing: <=256 KiB, depth 16, 4096 nodes, no duplicate keys; <=128 entries,
/// 64 KiB any string, URLs <=8192 bytes, 128 KiB aggregate exported strings.
pub fn inspect(document: &str) -> Result<Report, OperationError> {
    let value = parse(document)?;
    let entries = value["log"]["entries"]
        .as_array()
        .expect("validated entries");
    let mut output = Vec::with_capacity(entries.len());
    let mut bytes = 0;
    for (index, entry) in entries.iter().enumerate() {
        let request = entry
            .get("request")
            .and_then(Value::as_object)
            .ok_or_else(|| invalid(OP, "HAR entry requires request object"))?;
        let response = entry
            .get("response")
            .and_then(Value::as_object)
            .ok_or_else(|| invalid(OP, "HAR entry requires response object"))?;
        let method = string(request.get("method"), 32)?
            .ok_or_else(|| invalid(OP, "HAR request method is absent"))?;
        let url = string(request.get("url"), 8192)?
            .ok_or_else(|| invalid(OP, "HAR request URL is absent"))?;
        let status = response
            .get("status")
            .and_then(Value::as_u64)
            .filter(|n| *n <= 999)
            .ok_or_else(|| invalid(OP, "HAR response status must be in 0..999"))?
            as u16;
        let content = match response.get("content") {
            None | Some(Value::Null) => None,
            Some(v) => Some(
                v.as_object()
                    .ok_or_else(|| invalid(OP, "HAR response content must be an object"))?,
            ),
        };
        let mime_type = string(content.and_then(|m| m.get("mimeType")), 256)?;
        let body_encoding = string(content.and_then(|m| m.get("encoding")), 64)?;
        let body = string(content.and_then(|m| m.get("text")), MAX_FIELD)?;
        let started_date_time = string(entry.get("startedDateTime"), 128)?;
        let declared_size = match content.and_then(|m| m.get("size")) {
            None | Some(Value::Null) => None,
            Some(v) => Some(
                v.as_i64()
                    .filter(|n| *n >= -1)
                    .ok_or_else(|| invalid(OP, "HAR content size must be integer >=-1"))?,
            ),
        };
        let time_ms = match entry.get("time") {
            None | Some(Value::Null) => None,
            Some(v) => Some(
                v.as_f64()
                    .filter(|n| n.is_finite() && *n >= 0.0)
                    .ok_or_else(|| invalid(OP, "HAR time must be finite and nonnegative"))?,
            ),
        };
        bytes += method.len()
            + url.len()
            + mime_type.as_ref().map_or(0, String::len)
            + body_encoding.as_ref().map_or(0, String::len)
            + started_date_time.as_ref().map_or(0, String::len);
        if bytes > MAX_OUTPUT_TEXT {
            return Err(limit(OP, "HAR exported text exceeds 128 KiB"));
        }
        let reference_kind = reference_kind(&url);
        let observations = observations(&url);
        output.push(Entry {
            index,
            pointer: format!("/log/entries/{index}"),
            method,
            url,
            reference_kind,
            observations,
            status,
            mime_type,
            declared_size,
            started_date_time,
            time_ms,
            body_text_bytes: body.map(|s| s.len()),
            body_encoding,
        });
    }
    Ok(Report {
        format: "har",
        version: "1.2".into(),
        input_bytes: document.len(),
        entries: output,
        scope: "offline_metadata_only_no_network_browser_credentials_execution_or_body_decoding",
    })
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: OP,
    category: "web",
    description: "Bounded inert HAR 1.2 metadata observations, explicit missing values/body availability; no requests, headers/cookies/body export or browser access; strict bounded JSON.",
    input_type: "text",
    output_type: "web.har_report",
    parameters: &[],
    deterministic: true,
    reversible: Some(false),
    command: "web har",
    example: "cube web har saved.har --json",
}];
