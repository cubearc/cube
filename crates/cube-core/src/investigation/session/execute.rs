//! Bounded shared-core execution and transactional provenance recording.
use super::*;
use crate::{forensics, investigation::triage, pipeline::Value};
use std::{io::Cursor, sync::atomic::AtomicBool};
impl Session {
    /// Start a metadata-only session for one input. Identity is opt-in through an
    /// explicit `RunSpec::Identity` execution; no source bytes or paths are stored.
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
            profile: None,
        };
        session.check_size()?;
        Ok(session)
    }
    pub(in crate::investigation) fn preflight_record(
        &mut self,
        spec: &RunSpec,
        parameters: bool,
    ) -> Result<()> {
        self.ensure_room(spec, parameters)?;
        self.skip(spec.clone(), "dry-run metadata preflight")?;
        if parameters {
            self.runs.last_mut().expect("reserved").parameters = spec.clone();
            self.check_size()?;
        }
        Ok(())
    }
    pub(in crate::investigation) fn set_profile(
        &mut self,
        context: crate::investigation::profile::Context,
    ) -> Result<()> {
        context.validate()?;
        let mut candidate = self.clone();
        candidate.profile = Some(context);
        candidate.check_size()?;
        *self = candidate;
        Ok(())
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
    /// True when selected checks failed, were skipped or lost results to a budget.
    pub fn partial(&self) -> bool {
        self.runs
            .iter()
            .any(|r| r.state != State::Succeeded || r.warning.is_some())
    }
    pub(super) fn check_size(&self) -> Result<()> {
        json_bytes(self, self.limits.max_record_bytes)?;
        Ok(())
    }
    pub(super) fn retained_bytes(&self) -> Result<usize> {
        self.runs
            .iter()
            .filter_map(|r| r.retained.as_ref())
            .try_fold(0, |sum, v| {
                Ok(sum + json_bytes(v, MAX_RETAINED_BYTES)?.len())
            })
    }
    pub(super) fn verify_input(&self, data: &[u8]) -> Result<()> {
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
        spec.plan()?;
        self.verify_input(data)?;
        self.execute_prechecked(data, spec, capture, cancel)
    }
    // Profile/replay own one immutable, bounded input buffer. Verify its identity
    // once at the boundary, instead of hashing the full input before each preview.
    pub(in crate::investigation) fn execute_prechecked(
        &mut self,
        data: &[u8],
        spec: RunSpec,
        capture: Capture,
        cancel: Option<&AtomicBool>,
    ) -> Result<u32> {
        if data.len() > MAX_INPUT_BYTES || data.len() as u64 > self.input.size {
            return Err(limit("input buffer exceeds session bounds"));
        }
        spec.access(self.input.size)?;
        let plan = spec.plan()?;
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
                let identity = if matches!(spec, RunSpec::Identity) {
                    Some(
                        serde_json::from_value::<Digest>(result.output.clone())
                            .expect("identity result"),
                    )
                } else {
                    None
                };
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
                if let Some(identity) = identity {
                    self.input.digest = Some(identity);
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
    pub(in crate::investigation) fn skip(&mut self, spec: RunSpec, reason: &str) -> Result<u32> {
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
                    code: if reason.starts_with("cooperative cancellation") {
                        "cancelled"
                    } else {
                        "resource_limit"
                    }
                    .into(),
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
                for suggestion in &result.suggestions {
                    let evidence = suggestion
                        .evidence
                        .first()
                        .and_then(|id| result.facts.iter().find(|f| f.id == *id));
                    facts.push((
                        evidence.and_then(|f| f.offset),
                        FindingKind::Candidate,
                        format!(
                            "Suggested {} via {} based on {}: {}",
                            suggestion.operation,
                            suggestion.rule,
                            evidence.map_or("input evidence", |f| f.description.as_str()),
                            suggestion.uncertainty
                        ),
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
}
