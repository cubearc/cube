//! Versioned in-memory provenance. No paths, automatic persistence or raw input retention.
mod execute;
mod record;
use crate::{
    errors::ErrorCode,
    forensics::ByteRange,
    hashing,
    pipeline::{self, Kind, Limits, Parameter, Plan, Step},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::Write};
pub type Result<T> = std::result::Result<T, pipeline::Error>;
pub const MAX_INPUT_BYTES: usize = 1024 * 1024;
pub const MAX_RECORD_BYTES: usize = 128 * 1024;
pub const MAX_RETAINED_BYTES: usize = 64 * 1024;
pub(super) fn invalid(message: &str) -> pipeline::Error {
    pipeline::Error::new(ErrorCode::InvalidInput, message)
}
pub(super) fn limit(message: &str) -> pipeline::Error {
    pipeline::Error::new(ErrorCode::ResourceLimit, message)
}
pub(super) fn decode<T: serde::de::DeserializeOwned>(document: &str) -> Result<T> {
    serde_json::from_value(pipeline::json::parse(document)?).map_err(|_| {
        invalid(
            "invalid investigation schema: check required/unknown fields, supported versions, scalar types and provenance references",
        )
    })
}
struct BoundedWriter {
    data: Vec<u8>,
    maximum: usize,
}
impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.maximum.saturating_sub(self.data.len()) {
            return Err(std::io::Error::other("serialization budget exceeded"));
        }
        self.data.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub(super) fn json_bytes(value: &impl Serialize, maximum: usize) -> Result<Vec<u8>> {
    let mut writer = BoundedWriter {
        data: Vec::new(),
        maximum,
    };
    serde_json::to_writer(&mut writer, value)
        .map_err(|_| limit("serialized result/record exceeds byte budget"))?;
    Ok(writer.data)
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Digest {
    pub algorithm: String,
    pub value: String,
}
impl Digest {
    fn bytes(data: &[u8]) -> Self {
        Self {
            algorithm: "sha256".into(),
            value: hashing::sha256(data),
        }
    }
    fn validate(&self) -> Result<()> {
        if self.algorithm != "sha256"
            || self.value.len() != 64
            || !self
                .value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(invalid(
                "digest requires algorithm sha256 and 64 lowercase hexadecimal digits",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub id: u32,
    pub size: u64,
    pub digest: Option<Digest>,
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct SessionLimits {
    pub max_runs: usize,
    pub max_findings: usize,
    pub max_hypotheses: usize,
    pub max_record_bytes: usize,
    pub max_retained_bytes: usize,
}
impl Default for SessionLimits {
    fn default() -> Self {
        Self {
            max_runs: 64,
            max_findings: 128,
            max_hypotheses: 32,
            max_record_bytes: MAX_RECORD_BYTES,
            max_retained_bytes: MAX_RETAINED_BYTES,
        }
    }
}
impl SessionLimits {
    fn validate(&self) -> Result<()> {
        if self.max_runs > 64
            || self.max_findings > 128
            || self.max_hypotheses > 32
            || self.max_record_bytes > MAX_RECORD_BYTES
            || self.max_retained_bytes > MAX_RETAINED_BYTES
        {
            return Err(invalid(
                "session limits may only lower hard run/finding/note/record/retention bounds",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecordedStep {
    pub operation: String,
    pub version: u32,
    pub parameters: Option<BTreeMap<String, Parameter>>,
}
/// Explicit supported recording/replay contracts; no arbitrary operation dispatch.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RunSpec {
    Identity,
    Triage {
        max_read_bytes: usize,
    },
    Hex {
        offset: u64,
        length: usize,
        width: usize,
    },
    Strings {
        offset: u64,
        length: usize,
        min_length: usize,
        max_results: usize,
    },
    Pipeline {
        steps: Vec<RecordedStep>,
        limits: Limits,
    },
}
impl RunSpec {
    pub fn pipeline(steps: Vec<Step>, limits: Limits) -> Self {
        Self::Pipeline {
            steps: steps
                .into_iter()
                .map(|s| RecordedStep {
                    operation: s.operation,
                    version: s.version,
                    parameters: Some(s.parameters),
                })
                .collect(),
            limits,
        }
    }
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Identity => "hash.sha256",
            Self::Triage { .. } => "investigation.triage",
            Self::Hex { .. } => "file.hex.inspect",
            Self::Strings { .. } => "file.strings.extract",
            Self::Pipeline { .. } => "pipeline.execute",
        }
    }
    pub fn result_kind(&self) -> &'static str {
        match self {
            Self::Identity => "digest",
            Self::Triage { .. } => "investigation.triage",
            Self::Hex { .. } => "file.hex_rows",
            Self::Strings { .. } => "file.strings",
            Self::Pipeline { steps, .. } => steps
                .last()
                .and_then(|step| crate::operations::describe(&step.operation))
                .map_or("bytes", |descriptor| descriptor.output_type),
        }
    }
    pub fn access(&self, total: u64) -> Result<(u64, u64)> {
        let range = match self {
            Self::Identity | Self::Pipeline { .. } => (0, total),
            Self::Triage { max_read_bytes } => {
                if *max_read_bytes > 12 || total.min(12) > *max_read_bytes as u64 {
                    return Err(invalid(
                        "triage contract requires sufficient header budget, at most 12 bytes",
                    ));
                }
                (0, total.min(12))
            }
            Self::Hex {
                offset,
                length,
                width,
            } => {
                if *length > 4096 || !(1..=256).contains(width) {
                    return Err(invalid(
                        "session hex ranges are at most 4096 bytes; width must be 1..256",
                    ));
                }
                (*offset, *length as u64)
            }
            Self::Strings {
                offset,
                length,
                min_length,
                max_results,
            } => {
                if *length > 4096
                    || !(1..=4096).contains(min_length)
                    || !(1..=128).contains(max_results)
                {
                    return Err(invalid(
                        "session strings require <=4096 bytes, min_length 1..4096 and max_results 1..128",
                    ));
                }
                (*offset, *length as u64)
            }
        };
        ByteRange {
            offset: range.0,
            length: Some(range.1),
        }
        .resolve(total, u64::MAX)
        .map_err(pipeline::Error::from_operation)
    }
    pub(super) fn plan(&self) -> Result<Option<Plan>> {
        if let Self::Pipeline { steps, limits } = self {
            let steps: Vec<Step>=steps.iter().map(|s|Ok(Step{operation:s.operation.clone(),version:s.version,parameters:s.parameters.clone().ok_or_else(||invalid("pipeline parameters were redacted; replay requires the original explicit definition"))?})).collect::<Result<Vec<_>>>()?;
            if steps.iter().any(|s| s.operation == "text.transform.masked") {
                return Err(invalid(
                    "masked step audits require direct pipeline or recipe execution; session value records cannot retain them",
                ));
            }
            Ok(Some(Plan::validate(Kind::Bytes, steps, limits.clone())?))
        } else {
            Ok(None)
        }
    }
    fn redact(&mut self) {
        if let Self::Pipeline { steps, .. } = self {
            for step in steps {
                if step.parameters.as_ref().is_some_and(|p| !p.is_empty()) {
                    step.parameters = None;
                }
            }
        }
    }
    fn redacted(&self) -> bool {
        matches!(self,Self::Pipeline{steps,..} if steps.iter().any(|s|s.parameters.is_none()))
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Succeeded,
    Failed,
    Skipped,
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub id: u32,
    pub input_id: u32,
    pub operation: String,
    pub version: u32,
    pub parameters: RunSpec,
    pub state: State,
    pub diagnostic: Option<Diagnostic>,
    pub result_kind: String,
    /// SHA-256 over canonical compact JSON of the typed result, not raw source bytes.
    pub result_digest: Option<Digest>,
    pub retained: Option<serde_json::Value>,
    pub warning: Option<String>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    Observation,
    Candidate,
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub id: u32,
    pub input_id: u32,
    pub run_id: u32,
    pub offset: Option<u64>,
    pub kind: FindingKind,
    pub description: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Hypothesis {
    pub id: u32,
    pub text: String,
    pub findings: Vec<u32>,
}
/// Immutable fields and validated loading preserve session-local IDs and provenance.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "SessionWire")]
pub struct Session {
    format_version: u32,
    engine_api: u32,
    engine_version: String,
    input: Identity,
    limits: SessionLimits,
    runs: Vec<Run>,
    findings: Vec<Finding>,
    hypotheses: Vec<Hypothesis>,
    #[serde(default)]
    profile: Option<super::profile::Context>,
}
// Deserializing through Serde must preserve the same invariants as record loading.
#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct SessionWire {
    format_version: u32,
    engine_api: u32,
    engine_version: String,
    input: Identity,
    limits: SessionLimits,
    runs: Vec<Run>,
    findings: Vec<Finding>,
    hypotheses: Vec<Hypothesis>,
    #[serde(default)]
    profile: Option<super::profile::Context>,
}
impl TryFrom<SessionWire> for Session {
    type Error = pipeline::Error;
    fn try_from(value: SessionWire) -> Result<Self> {
        let session = Self {
            format_version: value.format_version,
            engine_api: value.engine_api,
            engine_version: value.engine_version,
            input: value.input,
            limits: value.limits,
            runs: value.runs,
            findings: value.findings,
            hypotheses: value.hypotheses,
            profile: value.profile,
        };
        session.validate()?;
        Ok(session)
    }
}

/// Saving inclusion is explicit per run. Data not retained in memory cannot be recovered.
#[derive(Clone, Debug, Default)]
pub struct Export {
    pub include_results: Vec<u32>,
    pub include_parameters: Vec<u32>,
    pub include_hypotheses: bool,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Capture {
    pub parameters: bool,
    pub result: bool,
}
#[derive(Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Record {
    schema_version: u32,
    status: String,
    replay_gaps: Vec<String>,
    uncertainty: String,
    session: Session,
}
const UNCERTAINTY: &str = "Signature candidates and printable content do not prove concealed data. Input/result digests identify content, not trust.";
struct Calculated {
    warning: Option<String>,
    output: serde_json::Value,
    findings: Vec<(Option<u64>, FindingKind, String)>,
}

pub const OPERATIONS: &[crate::operations::Descriptor] = &[
    crate::operations::Descriptor {
        id: "investigation.session.record",
        category: "investigation",
        description: "Record bounded operation provenance and separate observations from user hypotheses, without persistence or raw input retention.",
        input_type: "investigation.request",
        output_type: "investigation.session",
        parameters: &[],
        deterministic: true,
        reversible: None,
        command: "investigate pipeline",
        example: "cube investigate pipeline sample.bin --steps '[]' --identify --json",
    },
    crate::operations::Descriptor {
        id: "investigation.report",
        category: "investigation",
        description: "Export versioned JSON or escaped text with explicit redactions and per-run result inclusion; no automatic saving.",
        input_type: "investigation.session",
        output_type: "investigation.report",
        parameters: &[],
        deterministic: true,
        reversible: None,
        command: "investigate report",
        example: "cube investigate report session.json",
    },
    crate::operations::Descriptor {
        id: "investigation.replay",
        category: "investigation",
        description: "Verify input identity and all contracts before replaying supported deterministic runs; reject redaction gaps.",
        input_type: "investigation.replay_request",
        output_type: "investigation.session",
        parameters: &[],
        deterministic: true,
        reversible: None,
        command: "investigate replay",
        example: "cube investigate replay session.json sample.bin --json",
    },
];

/// Keep the report contract attached to its actual serialized record.
pub(crate) fn contract_report() -> schemars::Schema {
    schemars::schema_for!(Record)
}
