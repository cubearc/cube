//! Versioned in-memory provenance. No paths, automatic persistence or raw input retention.
use super::triage;
use crate::{
    errors::ErrorCode,
    forensics::{self, ByteRange},
    hashing,
    pipeline::{self, Kind, Limits, Parameter, Plan, Step, Value},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{Cursor, Write},
    sync::atomic::AtomicBool,
};
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
            "invalid investigation schema: check required/unknown fields and explicit scalar types",
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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub id: u32,
    pub size: u64,
    pub digest: Option<Digest>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedStep {
    pub operation: String,
    pub version: u32,
    pub parameters: Option<BTreeMap<String, Parameter>>,
}
/// Explicit supported recording/replay contracts; no arbitrary operation dispatch.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
            Self::Pipeline { .. } => "pipeline.value",
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
    fn plan(&self) -> Result<Option<Plan>> {
        if let Self::Pipeline { steps, limits } = self {
            let steps=steps.iter().map(|s|Ok(Step{operation:s.operation.clone(),version:s.version,parameters:s.parameters.clone().ok_or_else(||invalid("pipeline parameters were redacted; replay requires the original explicit definition"))?})).collect::<Result<Vec<_>>>()?;
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
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Succeeded,
    Failed,
    Skipped,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    Observation,
    Candidate,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub id: u32,
    pub input_id: u32,
    pub run_id: u32,
    pub offset: Option<u64>,
    pub kind: FindingKind,
    pub description: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hypothesis {
    pub id: u32,
    pub text: String,
    pub findings: Vec<u32>,
}
/// Immutable fields and validated loading preserve session-local IDs and provenance.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    format_version: u32,
    engine_api: u32,
    engine_version: String,
    input: Identity,
    limits: SessionLimits,
    runs: Vec<Run>,
    findings: Vec<Finding>,
    hypotheses: Vec<Hypothesis>,
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
#[derive(Serialize, Deserialize)]
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
impl Session {
    pub fn new(size: u64, limits: SessionLimits) -> Result<Self> {
        limits.validate()?;
        let session = Self {
            format_version: 1,
            engine_api: pipeline::ENGINE_API,
            engine_version: env!("CARGO_PKG_VERSION").into(),
            input: Identity {
                id: 1,
                size,
                digest: None,
            },
            limits,
            runs: Vec::new(),
            findings: Vec::new(),
            hypotheses: Vec::new(),
        };
        session.check_size()?;
        Ok(session)
    }
    pub fn identity(&self) -> &Identity {
        &self.input
    }
    pub fn runs(&self) -> &[Run] {
        &self.runs
    }
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }
    pub fn partial(&self) -> bool {
        self.runs
            .iter()
            .any(|r| r.state != State::Succeeded || r.warning.is_some())
    }
    fn check_size(&self) -> Result<()> {
        json_bytes(self, self.limits.max_record_bytes)?;
        Ok(())
    }
    fn retained_bytes(&self) -> Result<usize> {
        self.runs
            .iter()
            .filter_map(|r| r.retained.as_ref())
            .try_fold(0, |sum, v| {
                Ok(sum + json_bytes(v, MAX_RETAINED_BYTES)?.len())
            })
    }
    fn verify_input(&self, data: &[u8]) -> Result<()> {
        if data.len() > MAX_INPUT_BYTES || data.len() as u64 > self.input.size {
            return Err(limit(
                "input buffer exceeds session hard bound or declared size",
            ));
        }
        if let Some(digest) = &self.input.digest
            && (data.len() as u64 != self.input.size || Digest::bytes(data) != *digest)
        {
            return Err(invalid(
                "input size or SHA-256 identity changed; execution was not started",
            ));
        }
        Ok(())
    }
    /// Execute one explicit contract with the common core APIs. Invalid contracts
    /// fail before execution; runtime failures remain in provenance as failed runs.
    pub fn execute(
        &mut self,
        data: &[u8],
        spec: RunSpec,
        capture: Capture,
        cancel: Option<&AtomicBool>,
    ) -> Result<u32> {
        spec.access(self.input.size)?;
        let plan = spec.plan()?;
        self.verify_input(data)?;
        let (offset, length) = spec.access(self.input.size)?;
        if offset + length > data.len() as u64 {
            return Err(limit(
                "caller buffer does not cover the declared inspected range",
            ));
        }
        self.ensure_room(&spec, capture.parameters)?;
        let result = if cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed)) {
            Err(pipeline::Error::new(
                ErrorCode::Cancelled,
                "investigation cancelled before operation",
            ))
        } else {
            self.calculate(data, &spec, plan, cancel)
        };
        let mut recorded = spec.clone();
        if !capture.parameters {
            recorded.redact();
        }
        let id = self.runs.len() as u32 + 1;
        let mut run = Run {
            id,
            input_id: 1,
            operation: spec.operation().into(),
            version: 1,
            parameters: recorded,
            state: State::Succeeded,
            diagnostic: None,
            result_kind: spec.result_kind().into(),
            result_digest: None,
            retained: None,
            warning: None,
        };
        let findings = match result {
            Ok(result) => {
                run.warning = result.warning;
                let bytes = json_bytes(&result.output, 8 * 1024 * 1024)?;
                run.result_digest = Some(Digest::bytes(&bytes));
                if capture.result {
                    if bytes.len() + self.retained_bytes()? <= self.limits.max_retained_bytes {
                        run.retained = Some(result.output);
                    } else {
                        run.warning =
                            Some("selected result omitted: retention byte budget exhausted".into());
                    }
                }
                if matches!(spec, RunSpec::Identity) {
                    self.input.digest = Some(Digest::bytes(data));
                }
                result.findings
            }
            Err(error) => {
                run.state = State::Failed;
                run.diagnostic = Some(Diagnostic {
                    code: error.error.code.as_str().into(),
                    message: error.to_string().chars().take(256).collect(),
                });
                Vec::new()
            }
        };
        self.append(run, findings)?;
        Ok(id)
    }
    fn ensure_room(&self, spec: &RunSpec, parameters: bool) -> Result<()> {
        if self.runs.len() >= self.limits.max_runs {
            return Err(limit("session run budget exhausted"));
        }
        let mut candidate = self.clone();
        let mut spec = spec.clone();
        if !parameters {
            spec.redact();
        }
        if matches!(spec, RunSpec::Identity) {
            candidate.input.digest = Some(Digest::bytes(b""));
        }
        candidate.runs.push(Run {
            id: candidate.runs.len() as u32 + 1,
            input_id: 1,
            operation: spec.operation().into(),
            version: 1,
            result_kind: spec.result_kind().into(),
            parameters: spec,
            state: State::Failed,
            diagnostic: Some(Diagnostic {
                code: "invalid_parameter".into(),
                message: "x".repeat(256),
            }),
            result_digest: Some(Digest::bytes(b"")),
            retained: None,
            warning: Some("x".repeat(128)),
        });
        candidate.check_size()
    }
    fn append(
        &mut self,
        mut run: Run,
        findings: Vec<(Option<u64>, FindingKind, String)>,
    ) -> Result<()> {
        let mut candidate = self.clone();
        for (offset, kind, description) in findings {
            if candidate.findings.len() >= self.limits.max_findings {
                run.warning =
                    Some("finding count budget exhausted; additional observations omitted".into());
                break;
            }
            candidate.findings.push(Finding {
                id: candidate.findings.len() as u32 + 1,
                input_id: 1,
                run_id: run.id,
                offset,
                kind,
                description,
            });
        }
        candidate.runs.push(run.clone());
        if candidate.check_size().is_err() {
            candidate.findings.truncate(self.findings.len());
            run.retained = None;
            run.warning = Some(
                "record byte budget exhausted; observations and selected result omitted".into(),
            );
            candidate.runs.pop();
            candidate.runs.push(run);
            candidate.check_size()?;
        }
        *self = candidate;
        Ok(())
    }
    pub(super) fn skip(&mut self, spec: RunSpec, reason: &str) -> Result<u32> {
        self.ensure_room(&spec, false)?;
        let id = self.runs.len() as u32 + 1;
        let mut recorded = spec.clone();
        recorded.redact();
        self.append(
            Run {
                id,
                input_id: 1,
                operation: spec.operation().into(),
                version: 1,
                parameters: recorded,
                state: State::Skipped,
                diagnostic: Some(Diagnostic {
                    code: "resource_limit".into(),
                    message: reason.chars().take(256).collect(),
                }),
                result_kind: spec.result_kind().into(),
                result_digest: None,
                retained: None,
                warning: None,
            },
            Vec::new(),
        )?;
        Ok(id)
    }
    fn calculate(
        &self,
        data: &[u8],
        spec: &RunSpec,
        plan: Option<Plan>,
        cancel: Option<&AtomicBool>,
    ) -> Result<Calculated> {
        let mut facts = Vec::new();
        let mut warning = None;
        let mut cursor = Cursor::new(data);
        let output = match spec {
            RunSpec::Identity => {
                facts.push((
                    None,
                    FindingKind::Observation,
                    "Computed SHA-256 input identity; identity does not establish trust".into(),
                ));
                serde_json::to_value(Digest::bytes(data)).expect("digest")
            }
            RunSpec::Triage { .. } => {
                let result = triage::header(
                    &data[..self.input.size.min(12) as usize],
                    self.input.size,
                    &crate::operations::catalog(),
                );
                for fact in &result.facts {
                    facts.push((
                        fact.offset,
                        if fact.offset.is_some() {
                            FindingKind::Candidate
                        } else {
                            FindingKind::Observation
                        },
                        fact.description.clone(),
                    ));
                }
                serde_json::to_value(result).expect("triage")
            }
            RunSpec::Hex {
                offset,
                length,
                width,
            } => {
                let rows = forensics::ranges::hex_rows(
                    &mut cursor,
                    ByteRange {
                        offset: *offset,
                        length: Some(*length as u64),
                    },
                    *width,
                )
                .map_err(pipeline::Error::from_operation)?;
                facts.push((Some(*offset),FindingKind::Observation,format!("Inspected {length} selected bytes as hexadecimal; contents omitted unless selected")));
                serde_json::to_value(rows).expect("hex")
            }
            RunSpec::Strings {
                offset,
                length,
                min_length,
                max_results,
            } => {
                let result = forensics::ranges::strings(
                    &mut cursor,
                    ByteRange {
                        offset: *offset,
                        length: Some(*length as u64),
                    },
                    *min_length,
                    *max_results,
                )
                .map_err(pipeline::Error::from_operation)?;
                if !result.complete {
                    warning =
                        Some("string result listing incomplete: result budget exhausted".into());
                }
                facts.push((Some(*offset),FindingKind::Observation,format!("Found {} printable runs in {} selected bytes; selected range complete: {}. Printable runs do not establish concealed data",result.strings.len(),result.scanned_bytes,result.complete)));
                serde_json::to_value(result).expect("strings")
            }
            RunSpec::Pipeline { .. } => {
                let result = plan.expect("validated pipeline").execute(
                    Value::Bytes {
                        data: data.to_vec(),
                    },
                    &[],
                    cancel,
                )?;
                facts.push((None,FindingKind::Observation,format!("Pipeline completed with {:?} output; inspect parameters and identity gaps before replay",result.output.kind())));
                serde_json::to_value(result.output).expect("typed value")
            }
        };
        if cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed)) {
            return Err(pipeline::Error::new(
                ErrorCode::Cancelled,
                "investigation cancelled after bounded operation",
            ));
        }
        Ok(Calculated {
            warning,
            output,
            findings: facts,
        })
    }
    pub fn add_hypothesis(&mut self, text: &str, findings: &[u32]) -> Result<u32> {
        if text.len() > 1024
            || findings.len() > 128
            || self.hypotheses.len() >= self.limits.max_hypotheses
        {
            return Err(limit("hypothesis text/link/count budget exceeded"));
        }
        if findings
            .iter()
            .enumerate()
            .any(|(i, id)| !self.findings.iter().any(|f| f.id == *id) || findings[..i].contains(id))
        {
            return Err(invalid(
                "hypothesis references require distinct existing finding IDs",
            ));
        }
        let mut candidate = self.clone();
        let id = candidate.hypotheses.len() as u32 + 1;
        candidate.hypotheses.push(Hypothesis {
            id,
            text: text.into(),
            findings: findings.to_vec(),
        });
        candidate.check_size()?;
        *self = candidate;
        Ok(id)
    }
    pub fn redact_parameters(&mut self, id: u32) -> Result<()> {
        self.runs
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or_else(|| invalid("unknown session-local run ID"))?
            .parameters
            .redact();
        Ok(())
    }
    pub fn discard_result(&mut self, id: u32) -> Result<()> {
        self.runs
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or_else(|| invalid("unknown session-local run ID"))?
            .retained = None;
        Ok(())
    }
    pub fn replay_gaps(&self) -> Vec<String> {
        let mut gaps = Vec::new();
        if self.input.digest.is_none() {
            gaps.push("Input content identity is absent; size alone cannot verify replay".into());
        }
        for run in &self.runs {
            if run.state != State::Skipped && run.parameters.redacted() {
                gaps.push(format!(
                    "Run {} parameters were redacted; replay is unavailable",
                    run.id
                ));
            }
        }
        gaps
    }
    fn exported(&self, options: &Export) -> Result<Self> {
        for ids in [&options.include_results, &options.include_parameters] {
            if ids
                .iter()
                .enumerate()
                .any(|(i, id)| !self.runs.iter().any(|r| r.id == *id) || ids[..i].contains(id))
            {
                return Err(invalid(
                    "export selections require distinct existing run IDs",
                ));
            }
        }
        for run in &self.runs {
            if options.include_results.contains(&run.id) && run.retained.is_none() {
                return Err(invalid(
                    "selected result was not retained; omitted content cannot be recovered",
                ));
            }
            if options.include_parameters.contains(&run.id) && run.parameters.redacted() {
                return Err(invalid(
                    "selected parameters were already redacted and cannot be recovered",
                ));
            }
        }
        let mut result = self.clone();
        for run in &mut result.runs {
            if !options.include_results.contains(&run.id) {
                run.retained = None;
            }
            if !options.include_parameters.contains(&run.id) {
                run.parameters.redact();
            }
        }
        if !options.include_hypotheses {
            result.hypotheses.clear();
        }
        Ok(result)
    }
    /// Versioned compact JSON; opt-in saved fields are selected per run. No I/O.
    pub fn to_json(&self, options: &Export) -> Result<String> {
        let session = self.exported(options)?;
        let record = Record {
            schema_version: 1,
            status: if session.partial() {
                "partial"
            } else {
                "complete"
            }
            .into(),
            replay_gaps: session.replay_gaps(),
            uncertainty: UNCERTAINTY.into(),
            session,
        };
        let document = String::from_utf8(json_bytes(&record, self.limits.max_record_bytes)?)
            .map_err(|_| invalid("record serialization failed"))?;
        pipeline::json::parse(&document)?;
        Ok(document)
    }
    /// Load strict versioned JSON and verify derived status, IDs and references.
    pub fn from_json(document: &str) -> Result<Self> {
        let record: Record = decode(document)?;
        if record.schema_version != 1
            || record.status
                != if record.session.partial() {
                    "partial"
                } else {
                    "complete"
                }
            || record.replay_gaps != record.session.replay_gaps()
            || record.uncertainty != UNCERTAINTY
        {
            return Err(invalid("record schema/status/replay gaps are inconsistent"));
        }
        record.session.validate()?;
        if document.len() > record.session.limits.max_record_bytes {
            return Err(limit("loaded record exceeds configured byte limit"));
        }
        Ok(record.session)
    }
    fn validate(&self) -> Result<()> {
        self.limits.validate()?;
        if self.format_version != 1
            || self.engine_api != pipeline::ENGINE_API
            || self.engine_version.len() > 64
            || self.input.id != 1
        {
            return Err(invalid(
                "unsupported/missing session format, engine contract or input identity",
            ));
        }
        if let Some(digest) = &self.input.digest {
            digest.validate()?;
        }
        if self.runs.len() > self.limits.max_runs
            || self.findings.len() > self.limits.max_findings
            || self.hypotheses.len() > self.limits.max_hypotheses
            || self.retained_bytes()? > self.limits.max_retained_bytes
        {
            return Err(limit("loaded session exceeds count/retention bounds"));
        }
        for (index, run) in self.runs.iter().enumerate() {
            if run.id != index as u32 + 1
                || run.input_id != 1
                || run.operation != run.parameters.operation()
                || run.version != 1
                || run.result_kind != run.parameters.result_kind()
            {
                return Err(invalid("invalid run IDs, operation/version or result kind"));
            }
            run.parameters.access(self.input.size)?;
            if !run.parameters.redacted() {
                run.parameters.plan()?;
            } else if let RunSpec::Pipeline { steps, limits } = &run.parameters {
                limits.validate()?;
                if steps.len() > limits.max_steps
                    || steps.iter().any(|s| {
                        !pipeline::supported_adapters()
                            .iter()
                            .any(|a| a.operation == s.operation && a.version == s.version)
                    })
                {
                    return Err(invalid(
                        "redacted pipeline has unsupported steps or versions",
                    ));
                }
            }
            if let Some(digest) = &run.result_digest {
                digest.validate()?;
            }
            if run.warning.as_ref().is_some_and(|s| s.len() > 256)
                || run
                    .diagnostic
                    .as_ref()
                    .is_some_and(|d| d.code.len() > 64 || d.message.len() > 1024)
            {
                return Err(limit("run diagnostic bounds exceeded"));
            }
            if run.state == State::Succeeded
                && (run.result_digest.is_none() || run.diagnostic.is_some())
                || run.state != State::Succeeded
                    && (run.result_digest.is_some()
                        || run.retained.is_some()
                        || run.diagnostic.is_none())
            {
                return Err(invalid("run state, result and diagnostic are inconsistent"));
            }
            if let Some(value) = &run.retained
                && run.result_digest != Some(Digest::bytes(&json_bytes(value, MAX_RETAINED_BYTES)?))
            {
                return Err(invalid(
                    "retained result does not match its typed JSON digest",
                ));
            }
        }
        for (index, finding) in self.findings.iter().enumerate() {
            if finding.id != index as u32 + 1
                || finding.input_id != 1
                || !self
                    .runs
                    .iter()
                    .any(|r| r.id == finding.run_id && r.state == State::Succeeded)
                || finding.offset.is_some_and(|o| o > self.input.size)
                || finding.description.len() > 1024
            {
                return Err(invalid(
                    "invalid finding IDs, successful run reference, offset or description",
                ));
            }
        }
        for (index, note) in self.hypotheses.iter().enumerate() {
            if note.id != index as u32 + 1
                || note.text.len() > 1024
                || note.findings.len() > 128
                || note.findings.iter().enumerate().any(|(i, id)| {
                    !self.findings.iter().any(|f| f.id == *id) || note.findings[..i].contains(id)
                })
            {
                return Err(invalid(
                    "invalid hypothesis IDs, text or evidence references",
                ));
            }
        }
        self.check_size()
    }
    /// Preflight all versions, parameters and identities before replaying any run.
    /// Skipped runs stay skipped. Matching digests establish reproducibility, not trust.
    pub fn replay(&self, input: &[u8], cancel: Option<&AtomicBool>) -> Result<Self> {
        self.validate()?;
        if !self.replay_gaps().is_empty() {
            return Err(invalid(
                "session cannot replay: content identity or required parameters are absent/redacted",
            ));
        }
        for run in &self.runs {
            if run.state != State::Skipped {
                run.parameters.plan()?;
            }
        }
        self.verify_input(input)?;
        let mut result = Self::new(self.input.size, self.limits.clone())?;
        result.input.digest = self.input.digest.clone();
        for run in &self.runs {
            if run.state == State::Skipped {
                result.skip(
                    run.parameters.clone(),
                    "original run was skipped; replay did not execute it",
                )?;
            } else {
                result.execute(
                    input,
                    run.parameters.clone(),
                    Capture {
                        parameters: true,
                        result: false,
                    },
                    cancel,
                )?;
                let actual = result.runs.last().expect("recorded");
                if actual.state != run.state || actual.result_digest != run.result_digest {
                    return Err(invalid(
                        "replay diverged from the recorded state/result digest; inspect engine compatibility",
                    ));
                }
            }
        }
        // Notes and selected original values are not replayed as machine evidence.
        Ok(result)
    }
    /// Safe terminal rendering: all untrusted strings use Rust default escaping.
    pub fn human(&self) -> String {
        let escape = |s: &str| s.escape_default().to_string();
        let mut report = format!(
            "Cube investigation record v{}; engine API {}; package {}\nInput #{}: {} bytes; SHA-256 identity: {}\nStatus: {}\n",
            self.format_version,
            self.engine_api,
            escape(&self.engine_version),
            self.input.id,
            self.input.size,
            self.input
                .digest
                .as_ref()
                .map_or_else(|| "not recorded".into(), |d| escape(&d.value)),
            if self.partial() {
                "partial"
            } else {
                "complete for selected checks"
            }
        );
        for run in &self.runs {
            report.push_str(&format!(
                "Run #{} input #{}: {} v{} {:?}; result {}\n",
                run.id,
                run.input_id,
                escape(&run.operation),
                run.version,
                run.state,
                escape(&run.result_kind)
            ));
            if let Some(d) = &run.diagnostic {
                report.push_str(&format!("  {}: {}\n", escape(&d.code), escape(&d.message)));
            }
            if let Some(w) = &run.warning {
                report.push_str(&format!("  Warning: {}\n", escape(w)));
            }
        }
        for finding in &self.findings {
            report.push_str(&format!(
                "Finding #{} input #{} run #{} offset {:?} {:?}: {}\n",
                finding.id,
                finding.input_id,
                finding.run_id,
                finding.offset,
                finding.kind,
                escape(&finding.description)
            ));
        }
        for note in &self.hypotheses {
            report.push_str(&format!(
                "User hypothesis #{} (not machine evidence; findings {:?}): {}\n",
                note.id,
                note.findings,
                escape(&note.text)
            ));
        }
        for gap in self.replay_gaps() {
            report.push_str(&format!("Replay gap: {}\n", escape(&gap)));
        }
        report.push_str("Uncertainty: signatures and printable content do not prove concealed data. Scope is limited to the recorded ranges/checks.\n");
        report
    }
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
