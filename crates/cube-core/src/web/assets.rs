//! Explicit bounded embedded assets; never resolve a reference into I/O.
use super::*;
use crate::{
    encoding, hashing,
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::Serialize;
pub const MAX_ASSET_BYTES: usize = 65536;
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HtmlSelector {
    Inline,
    DataUrl,
}
impl HtmlSelector {
    pub fn parse(value: &str) -> Result<Self, OperationError> {
        match value {
            "inline" => Ok(Self::Inline),
            "data_url" => Ok(Self::DataUrl),
            _ => Err(OperationError::new(
                ErrorCode::InvalidParameter,
                "selector must be inline or data_url",
            )
            .operation("web.html.asset")
            .parameter("selector")),
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Provenance {
    pub artifact: &'static str,
    pub artifact_sha256: String,
    pub selection: usize,
    pub source_pointer: String,
    pub source_range: Option<[usize; 2]>,
    pub representation: &'static str,
    pub media_type: Option<String>,
    pub bytes: usize,
    pub sha256: String,
}
#[derive(Debug, Serialize)]
pub struct Asset {
    pub provenance: Provenance,
    #[serde(skip)]
    pub data: Vec<u8>,
}
impl Asset {
    /// Bind exact bytes to the existing engine; no new value kind or permissions.
    pub fn into_value(self) -> crate::pipeline::Value {
        crate::pipeline::Value::Bytes { data: self.data }
    }
}
fn make(
    document: &str,
    op: &'static str,
    mut provenance: Provenance,
    data: Vec<u8>,
) -> Result<Asset, OperationError> {
    if data.len() > MAX_ASSET_BYTES {
        return Err(limit(op, "selected embedded asset exceeds 64 KiB"));
    }
    provenance.artifact_sha256 = hashing::sha256(document.as_bytes());
    provenance.bytes = data.len();
    provenance.sha256 = hashing::sha256(&data);
    Ok(Asset { provenance, data })
}
fn provenance(
    artifact: &'static str,
    selection: usize,
    pointer: String,
    range: Option<[usize; 2]>,
    representation: &'static str,
    mime: Option<String>,
) -> Provenance {
    Provenance {
        artifact,
        artifact_sha256: String::new(),
        selection,
        source_pointer: pointer,
        source_range: range,
        representation,
        media_type: mime,
        bytes: 0,
        sha256: String::new(),
    }
}
/// Select a reported inline script/style source range or strict base64 data URL.
/// Inline bodies must have an explicit closing tag; source UTF-8 bytes are exact.
/// Attribute values undergo HTML character decoding before strict base64 decoding.
/// Other URL schemes, relative paths, percent-encoded/non-base64 data URLs and
/// unterminated inline bodies are not opened, fetched, guessed or decoded.
pub fn html(document: &str, index: usize, selector: HtmlSelector) -> Result<Asset, OperationError> {
    const OP: &str = "web.html.asset";
    let result = (|| {
        let report = super::html::inspect(document)?;
        match selector {
            HtmlSelector::Inline => {
                let item = report
                    .embedded
                    .get(index)
                    .ok_or_else(|| invalid(OP, "inline selection index is unavailable"))?;
                if !item.terminated {
                    return Err(invalid(OP, "inline asset has no explicit closing tag"));
                }
                let [start, end] = item.source_range;
                let data = document
                    .get(start..end)
                    .ok_or_else(|| invalid(OP, "invalid inline source range"))?
                    .as_bytes()
                    .to_vec();
                make(
                    document,
                    OP,
                    provenance(
                        "html",
                        index,
                        format!("/embedded/{index}"),
                        Some(item.source_range),
                        "exact_source_utf8",
                        None,
                    ),
                    data,
                )
            }
            HtmlSelector::DataUrl => {
                let item = report
                    .references
                    .get(index)
                    .ok_or_else(|| invalid(OP, "reference selection index is unavailable"))?;
                let value = &item.value;
                if !value
                    .get(..5)
                    .is_some_and(|s| s.eq_ignore_ascii_case("data:"))
                {
                    return Err(unsupported(
                        OP,
                        "selected reference is not an embedded data URL; references are never resolved",
                    ));
                }
                let (header, body) = value[5..]
                    .split_once(',')
                    .ok_or_else(|| invalid(OP, "data URL requires an explicit comma separator"))?;
                if !header
                    .get(header.len().saturating_sub(7)..)
                    .is_some_and(|s| s.eq_ignore_ascii_case(";base64"))
                {
                    return Err(unsupported(
                        OP,
                        "only strict base64 data URLs are supported; no percent decoding",
                    ));
                }
                let mime = header
                    .split(';')
                    .next()
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned);
                if mime.as_ref().is_some_and(|s| s.len() > 256) {
                    return Err(limit(OP, "data URL media type exceeds 256 bytes"));
                }
                let data = encoding::base64_decode(body).map_err(|e| {
                    invalid(OP, "data URL body is not canonical standard base64").caused_by(e)
                })?;
                make(
                    document,
                    OP,
                    provenance(
                        "html",
                        index,
                        format!("/references/{index}"),
                        Some(item.attribute_span),
                        "html_character_decoding_then_base64",
                        mime,
                    ),
                    data,
                )
            }
        }
    })();
    result.map_err(|e: OperationError| e.operation(OP))
}
/// Select one explicitly embedded HAR response body. Base64 yields exact decoded
/// bytes; omitted encoding yields UTF-8 of the logical JSON text, not wire bytes.
/// Declared compressed sizes are observations; no decompression/transcoding occurs.
/// Missing text, custom external-body references and unknown encodings fail.
pub fn har(document: &str, entry: usize) -> Result<Asset, OperationError> {
    const OP: &str = "web.har.asset";
    let result = (|| {
        let report = super::har::inspect(document)?;
        let info = report
            .entries
            .get(entry)
            .ok_or_else(|| invalid(OP, "HAR entry selection is unavailable"))?;
        let value = super::har::parse(document)?;
        let content = &value["log"]["entries"][entry]["response"]["content"];
        let text = content
            .get("text")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                invalid(
                    OP,
                    "selected HAR body text is absent; external references are never opened",
                )
            })?;
        let (data, representation) = match info.body_encoding.as_deref() {
            None => (text.as_bytes().to_vec(), "har_logical_text_utf8"),
            Some("base64") => (
                encoding::base64_decode(text).map_err(|e| {
                    invalid(OP, "HAR body is not canonical standard base64").caused_by(e)
                })?,
                "har_base64_decoded_bytes",
            ),
            _ => {
                return Err(unsupported(
                    OP,
                    "unsupported HAR body encoding; only omitted encoding or base64 are supported",
                ));
            }
        };
        make(
            document,
            OP,
            provenance(
                "har",
                entry,
                format!("/log/entries/{entry}/response/content/text"),
                None,
                representation,
                info.mime_type.clone(),
            ),
            data,
        )
    })();
    result.map_err(|e: OperationError| e.operation(OP))
}
const INDEX: Parameter = Parameter {
    name: "index",
    description: "Explicit zero-based embedded record/reference selection; no discovery guess or reference resolution.",
    required: true,
    default: None,
    constraints: ParameterType::Unsigned { min: 0, max: 255 },
};
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "web.html.asset",
        category: "web",
        description: "Explicit inline source or strict base64 data-URL asset to bounded bytes with source/range/encoding/hash provenance; no URL/path I/O or execution; <=64 KiB.",
        input_type: "text",
        output_type: "bytes",
        parameters: &[
            INDEX,
            Parameter {
                name: "selector",
                description: "inline indexes embedded descriptors; data_url indexes references.",
                required: false,
                default: Some(ParameterValue::Text("inline")),
                constraints: ParameterType::Text {
                    choices: &["inline", "data_url"],
                },
            },
        ],
        deterministic: true,
        reversible: Some(false),
        command: "web asset html",
        example: "cube web asset html saved.html --index 0 --selector inline --stdout",
    },
    Descriptor {
        id: "web.har.asset",
        category: "web",
        description: "Explicit embedded HAR response body to bytes with logical-text/base64 provenance; no body download/decompression/path access; <=64 KiB.",
        input_type: "text",
        output_type: "bytes",
        parameters: &[Parameter {
            name: "index",
            description: "Explicit zero-based HAR entry with embedded response.content.text.",
            required: true,
            default: None,
            constraints: ParameterType::Unsigned { min: 0, max: 127 },
        }],
        deterministic: true,
        reversible: Some(false),
        command: "web asset har",
        example: "cube web asset har saved.har --index 0 --stdout",
    },
];
