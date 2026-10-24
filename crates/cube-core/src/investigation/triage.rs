//! Small explicit recommendation rules; signature candidates are not validity proofs.
use crate::{
    errors::{ErrorCode, OperationError},
    forensics::{inspect::FileInfo, signatures},
    operations::Descriptor,
};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom};
pub const MAX_HEADER_BYTES: usize = 12;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Fact {
    pub id: String,
    pub operation: String,
    pub version: u32,
    pub offset: Option<u64>,
    pub description: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Suggestion {
    pub rule: String,
    pub operation: String,
    pub evidence: Vec<String>,
    pub uncertainty: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Triage {
    pub schema_version: u32,
    pub input_size: u64,
    pub inspected_bytes: usize,
    pub evidence_source: String,
    pub facts: Vec<Fact>,
    pub suggestions: Vec<Suggestion>,
    pub limitations: Vec<String>,
}
fn budget(size: u64, maximum: usize) -> Result<usize, OperationError> {
    if maximum > MAX_HEADER_BYTES {
        return Err(OperationError::new(
            ErrorCode::InvalidParameter,
            "triage read budget must be between 0 and 12 bytes",
        )
        .operation("investigation.triage")
        .parameter("max_read_bytes"));
    }
    let needed = size.min(MAX_HEADER_BYTES as u64) as usize;
    if needed > maximum {
        return Err(OperationError::new(ErrorCode::ResourceLimit,"triage requires up to 12 header bytes; lower input size or explicitly increase read budget").operation("investigation.triage"));
    }
    Ok(needed)
}
/// Inspect at most 12 header bytes. Only available descriptors are suggested;
/// no suggestion executes an operation, decodes pixels or extracts content.
pub fn bytes(
    data: &[u8],
    max_read_bytes: usize,
    available: &[&Descriptor],
) -> Result<Triage, OperationError> {
    let length = budget(data.len() as u64, max_read_bytes)?;
    Ok(header(&data[..length], data.len() as u64, available))
}
/// Read an exact bounded header from a caller-owned seekable stream. No path,
/// filename, filesystem timestamps, network access or mutation is involved.
pub fn reader(
    reader: &mut (impl Read + Seek),
    max_read_bytes: usize,
    available: &[&Descriptor],
) -> Result<Triage, OperationError> {
    let size = reader.seek(SeekFrom::End(0))?;
    let length = budget(size, max_read_bytes)?;
    let mut header = vec![0; length];
    reader.seek(SeekFrom::Start(0))?;
    reader.read_exact(&mut header)?;
    Ok(self::header(&header, size, available))
}
/// Reuse explicitly supplied basic inspection evidence, without further reads.
/// These observations describe caller-provided evidence, not independently verified bytes.
pub fn from_file_info(
    info: &FileInfo,
    available: &[&Descriptor],
) -> Result<Triage, OperationError> {
    if info.signatures.len() > 16
        || info
            .signatures
            .iter()
            .any(|s| s.format.len() > 32 || s.offset >= info.size)
    {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            "caller file evidence exceeds signature/count/offset limits",
        )
        .operation("investigation.triage"));
    }
    Ok(build(
        info.size,
        0,
        "caller_file_info",
        info.signatures
            .iter()
            .map(|s| signatures::Signature {
                format: s.format,
                mime: s.mime,
                offset: s.offset,
                evidence_hex: String::new(),
                rule: "caller evidence",
            })
            .collect(),
        available,
    ))
}
pub(super) fn header(data: &[u8], total_size: u64, available: &[&Descriptor]) -> Triage {
    build(
        total_size,
        data.len(),
        "input_header",
        signatures::detect(data),
        available,
    )
}
fn build(
    size: u64,
    read: usize,
    source: &str,
    mut signatures: Vec<signatures::Signature>,
    available: &[&Descriptor],
) -> Triage {
    signatures.sort_by_key(|s| (s.offset, s.format));
    signatures.dedup_by_key(|s| (s.offset, s.format));
    let mut result=Triage{schema_version:1,input_size:size,inspected_bytes:read,evidence_source:source.into(),facts:vec![Fact{id:"fact-1".into(),operation:"investigation.triage".into(),version:1,offset:None,description:format!("Input size is {size} bytes")}],suggestions:Vec::new(),limitations:vec!["Header matches are format candidates, not structural validation or evidence of concealed data. No extraction or decoding was executed.".into(),"Only the bounded header was considered; embedded signatures and other formats may be missed.".into()]};
    let mut suggest = |rule: &str, operation: &str, evidence: String, uncertainty: &str| {
        if available.iter().any(|d| d.id == operation) {
            result.suggestions.push(Suggestion {
                rule: rule.into(),
                operation: operation.into(),
                evidence: vec![evidence],
                uncertainty: uncertainty.into(),
            });
        }
    };
    suggest(
        "triage.basic_info.v1",
        "file.info.inspect",
        "fact-1".into(),
        "Filesystem facts require an explicitly selected regular file; they are not inferred from bytes.",
    );
    if signatures.is_empty() {
        suggest(
            "triage.unknown_hex.v1",
            "file.hex.inspect",
            "fact-1".into(),
            "No supported header matched; this does not prove the input has no structure.",
        );
        suggest(
            "triage.unknown_strings.v1",
            "file.strings.extract",
            "fact-1".into(),
            "Printable runs may be ordinary content or coincidences, not hidden-data findings.",
        );
    }
    for signature in signatures {
        let id = format!("fact-{}", result.facts.len() + 1);
        result.facts.push(Fact {
            id: id.clone(),
            operation: "file.signature.detect".into(),
            version: 1,
            offset: Some(signature.offset),
            description: format!("{} header signature candidate", signature.format),
        });
        match signature.format {
            "png" => suggest(
                "triage.png_chunks.v1",
                "image.png.inspect",
                id,
                "A PNG prefix alone does not validate chunks, CRCs, dimensions or pixel data.",
            ),
            "jpeg" | "tiff" => suggest(
                "triage.exif_candidate.v1",
                "file.exif.inspect",
                id,
                "This container may support EXIF; metadata can be absent, malformed or sensitive.",
            ),
            "wave" => suggest(
                "triage.wave_metadata.v1",
                "audio.wave.inspect",
                id,
                "RIFF/WAVE markers alone do not validate streams or establish a supported codec.",
            ),
            _ => (),
        }
    }
    result
}
impl Triage {
    pub fn human(&self) -> String {
        let mut text = format!(
            "Input: {} bytes; inspected: {} header bytes; evidence: {}\n",
            self.input_size,
            self.inspected_bytes,
            self.evidence_source.escape_default()
        );
        for fact in &self.facts {
            text.push_str(&format!(
                "{} [{} v{}]: {}\n",
                fact.id.escape_default(),
                fact.operation.escape_default(),
                fact.version,
                fact.description.escape_default()
            ));
        }
        for suggestion in &self.suggestions {
            text.push_str(&format!(
                "Suggest {} via {} using {}: {}\n",
                suggestion.operation.escape_default(),
                suggestion.rule.escape_default(),
                suggestion.evidence.join(", ").escape_default(),
                suggestion.uncertainty.escape_default()
            ));
        }
        for limitation in &self.limitations {
            text.push_str(&format!("Limitation: {}\n", limitation.escape_default()));
        }
        text
    }
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "investigation.triage",
    category: "investigation",
    description: "Recommend available inspections from bounded header evidence using explainable rules, without executing suggestions.",
    input_type: "bytes",
    output_type: "investigation.triage",
    parameters: &[crate::operations::Parameter {
        name: "max_read_bytes",
        description: "Maximum header bytes; no more than 12. Insufficient budgets fail before reading.",
        required: false,
        default: Some(crate::operations::ParameterValue::Unsigned(12)),
        constraints: crate::operations::ParameterType::Unsigned { min: 0, max: 12 },
    }],
    deterministic: true,
    reversible: None,
    command: "investigate triage",
    example: "cube investigate triage sample.bin --json",
}];
