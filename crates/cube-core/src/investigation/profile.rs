//! Inspectable local profiles with preflight budgets and cooperative cancellation/time.
use super::session::{
    self, Capture, Result, RunSpec, Session, SessionLimits, State, invalid, limit,
};
use crate::pipeline::{self, Limits, Step};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Seek, SeekFrom},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Depth {
    #[default]
    Metadata,
    Preview,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Budgets {
    pub max_input_bytes: usize,
    pub max_inspected_bytes: u64,
    pub max_operations: usize,
    pub max_retained_bytes: usize,
    pub max_elapsed_ms: u64,
}
impl Default for Budgets {
    fn default() -> Self {
        Self {
            max_input_bytes: session::MAX_INPUT_BYTES,
            max_inspected_bytes: 4 * 1024 * 1024,
            max_operations: 16,
            max_retained_bytes: session::MAX_RETAINED_BYTES,
            max_elapsed_ms: 5000,
        }
    }
}
impl Budgets {
    pub fn validate(&self) -> Result<()> {
        if self.max_input_bytes > session::MAX_INPUT_BYTES
            || self.max_inspected_bytes > 4 * 1024 * 1024
            || self.max_operations > 64
            || self.max_retained_bytes > session::MAX_RETAINED_BYTES
            || self.max_elapsed_ms > 60000
        {
            return Err(invalid(
                "profile budgets may only lower hard input(1 MiB)/inspection(4 MiB)/operation(64)/retention(64 KiB)/time(60s) bounds",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pipeline {
    pub steps: Vec<Step>,
    #[serde(default)]
    pub limits: Limits,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub format_version: u32,
    #[serde(default)]
    pub depth: Depth,
    #[serde(default)]
    pub identify: bool,
    #[serde(default)]
    pub budgets: Budgets,
    #[serde(default)]
    pub pipeline: Option<Pipeline>,
    /// Session-local run IDs, matching dry-run IDs, to retain in memory.
    #[serde(default)]
    pub retain_results: Vec<u32>,
    #[serde(default)]
    pub record_parameters: bool,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            format_version: 1,
            depth: Depth::Metadata,
            identify: false,
            budgets: Budgets::default(),
            pipeline: None,
            retain_results: Vec::new(),
            record_parameters: false,
        }
    }
}
/// Safe saved profile provenance; parameter values and content are absent.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub format_version: u32,
    pub depth: Depth,
    pub budgets: Budgets,
    pub input_read_bytes: usize,
    pub inspected_bytes: u64,
    pub operation_units: usize,
}
impl Context {
    pub fn validate(&self) -> Result<()> {
        if self.format_version != 1 {
            return Err(invalid("unsupported profile context version"));
        }
        self.budgets.validate()?;
        if self.input_read_bytes > self.budgets.max_input_bytes
            || self.inspected_bytes > self.budgets.max_inspected_bytes
            || self.operation_units > self.budgets.max_operations
        {
            return Err(invalid("profile execution counters exceed saved budgets"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct PlannedRun {
    pub id: u32,
    pub operation: &'static str,
    pub version: u32,
    pub offset: u64,
    pub length: u64,
    pub operation_units: usize,
    pub retention_upper_bound: usize,
    pub retain_result: bool,
    pub execute: bool,
    pub reason: Option<String>,
    /// Redacted plan metadata; no parameter secrets.
    pub pipeline: Option<serde_json::Value>,
    #[serde(skip)]
    spec: RunSpec,
}
#[derive(Clone, Debug, Serialize)]
pub struct DryRun {
    pub format_version: u32,
    pub engine_api: u32,
    pub engine_version: &'static str,
    pub depth: Depth,
    pub input_size: u64,
    pub input_read_bytes: usize,
    pub inspected_bytes: u64,
    pub operation_units: usize,
    pub retained_upper_bound: usize,
    pub budgets: Budgets,
    pub runs: Vec<PlannedRun>,
}
/// Session record is exported separately with its explicit privacy controls.
#[derive(Debug)]
pub struct Investigation {
    pub session: Session,
    pub plan: DryRun,
    pub input_read_bytes: usize,
    pub inspected_bytes: u64,
    pub operation_units: usize,
    pub elapsed_ms: u128,
}
impl Profile {
    pub fn parse(document: &str) -> Result<Self> {
        let profile: Self = session::decode(document)?;
        profile.validate()?;
        Ok(profile)
    }
    pub fn validate(&self) -> Result<()> {
        if self.format_version != 1 {
            return Err(invalid(
                "unsupported investigation profile format; expected 1",
            ));
        }
        self.budgets.validate()?;
        if let Some(definition) = &self.pipeline {
            pipeline::Plan::validate(
                pipeline::Kind::Bytes,
                definition.steps.clone(),
                definition.limits.clone(),
            )?;
        }
        if self.retain_results.len() > 5
            || self
                .retain_results
                .iter()
                .enumerate()
                .any(|(i, id)| *id == 0 || self.retain_results[..i].contains(id))
        {
            return Err(invalid(
                "retained run IDs must be distinct and match the dry-run",
            ));
        }
        Ok(())
    }
    fn specs(&self, size: u64) -> Vec<RunSpec> {
        let mut specs = Vec::new();
        if self.identify {
            specs.push(RunSpec::Identity);
        }
        specs.push(RunSpec::Triage { max_read_bytes: 12 });
        if self.depth == Depth::Preview {
            specs.push(RunSpec::Hex {
                offset: 0,
                length: size.min(256) as usize,
                width: 16,
            });
            specs.push(RunSpec::Strings {
                offset: 0,
                length: size.min(4096) as usize,
                min_length: 4,
                max_results: 32,
            });
        }
        if let Some(p) = &self.pipeline {
            specs.push(RunSpec::pipeline(p.steps.clone(), p.limits.clone()));
        }
        specs
    }
    /// Metadata-only planning: no source bytes, filesystem reads or algorithms.
    /// Conservative retention bounds may reject data-dependent smaller outputs.
    pub fn dry_run(&self, size: u64) -> Result<DryRun> {
        self.validate()?;
        let specs = self.specs(size);
        if self
            .retain_results
            .iter()
            .any(|id| *id as usize > specs.len())
        {
            return Err(invalid("retained run ID is absent from this profile"));
        }
        let mut plan = DryRun {
            format_version: 1,
            engine_api: pipeline::ENGINE_API,
            engine_version: env!("CARGO_PKG_VERSION"),
            depth: self.depth,
            input_size: size,
            input_read_bytes: 0,
            inspected_bytes: 0,
            operation_units: 0,
            retained_upper_bound: 0,
            budgets: self.budgets.clone(),
            runs: Vec::new(),
        };
        let mut stopped = false;
        for (index, spec) in specs.into_iter().enumerate() {
            let (offset, length) = spec.access(size)?;
            let compiled = spec.plan()?;
            let pipeline = compiled
                .as_ref()
                .map(|p| serde_json::to_value(p.info()).expect("plan"));
            let units = if let RunSpec::Pipeline { steps, .. } = &spec {
                1 + steps.len()
            } else {
                1
            };
            let retain = self.retain_results.contains(&(index as u32 + 1));
            let ceiling = if retain {
                match &spec {
                    RunSpec::Identity => 256,
                    RunSpec::Triage { .. } => 8192,
                    RunSpec::Hex { length, width, .. } => length * (4 + 256 / width) + 1024,
                    RunSpec::Strings {
                        length,
                        max_results,
                        ..
                    } => length * 6 + max_results * 128 + 1024,
                    RunSpec::Pipeline { limits, .. } => limits.max_value_bytes * 6 + 1024,
                }
            } else {
                0
            };
            let reason = if stopped {
                Some("earlier profile budget exhausted".into())
            } else if offset + length > self.budgets.max_input_bytes as u64 {
                Some("input byte budget cannot cover this inspected range".into())
            } else if plan.inspected_bytes + length > self.budgets.max_inspected_bytes {
                Some("total inspected byte budget exhausted".into())
            } else if plan.operation_units + units > self.budgets.max_operations {
                Some(
                    "operation count budget exhausted (pipeline wrapper and each step are charged)"
                        .into(),
                )
            } else if plan.retained_upper_bound + ceiling > self.budgets.max_retained_bytes {
                Some("conservative selected-result retention budget exhausted".into())
            } else {
                None
            };
            let execute = reason.is_none();
            if execute {
                plan.input_read_bytes = plan.input_read_bytes.max((offset + length) as usize);
                plan.inspected_bytes += length;
                plan.operation_units += units;
                plan.retained_upper_bound += ceiling;
            } else {
                stopped = true;
            }
            plan.runs.push(PlannedRun {
                id: index as u32 + 1,
                operation: spec.operation(),
                version: 1,
                offset,
                length,
                operation_units: units,
                retention_upper_bound: ceiling,
                retain_result: retain,
                execute,
                reason,
                pipeline,
                spec,
            });
        }
        // Preflight metadata storage for all run definitions before any algorithm.
        let mut session = self.session(size)?;
        for run in &plan.runs {
            session.preflight_record(&run.spec, self.record_parameters)?;
        }
        Ok(plan)
    }
    fn session(&self, size: u64) -> Result<Session> {
        let mut session = Session::new(
            size,
            SessionLimits {
                max_retained_bytes: self.budgets.max_retained_bytes,
                ..Default::default()
            },
        )?;
        session.set_profile(Context {
            format_version: 1,
            depth: self.depth,
            budgets: self.budgets.clone(),
            input_read_bytes: 0,
            inspected_bytes: 0,
            operation_units: 0,
        })?;
        Ok(session)
    }
    /// Read only the admitted prefix in fixed blocks. No source paths are stored.
    /// Cancellation and time are checked around reads and bounded operations;
    /// currently running operations are not forcibly preempted.
    pub fn run_reader(
        &self,
        reader: &mut (impl Read + Seek),
        cancel: Option<&AtomicBool>,
    ) -> Result<Investigation> {
        let started = Instant::now();
        let size = reader
            .seek(SeekFrom::End(0))
            .map_err(crate::errors::OperationError::from)
            .map_err(pipeline::Error::from_operation)?;
        let plan = self.dry_run(size)?;
        let mut data = Vec::with_capacity(plan.input_read_bytes);
        reader
            .seek(SeekFrom::Start(0))
            .map_err(crate::errors::OperationError::from)
            .map_err(pipeline::Error::from_operation)?;
        let mut reason = None;
        let mut buffer = [0u8; 64 * 1024];
        while data.len() < plan.input_read_bytes {
            if let Some(stop) = self.interruption(started, cancel) {
                reason = Some(stop);
                break;
            }
            let count = (plan.input_read_bytes - data.len()).min(buffer.len());
            reader
                .read_exact(&mut buffer[..count])
                .map_err(crate::errors::OperationError::from)
                .map_err(pipeline::Error::from_operation)?;
            data.extend_from_slice(&buffer[..count]);
        }
        self.run_buffer(&data, plan, started, cancel, reason)
    }
    pub fn run_bytes(&self, data: &[u8], cancel: Option<&AtomicBool>) -> Result<Investigation> {
        self.run_reader(&mut std::io::Cursor::new(data), cancel)
    }
    fn interruption(&self, started: Instant, cancel: Option<&AtomicBool>) -> Option<&'static str> {
        if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
            Some("cooperative cancellation requested; further analysis suppressed")
        } else if started.elapsed() >= Duration::from_millis(self.budgets.max_elapsed_ms) {
            Some("cooperative elapsed-time budget exhausted; further analysis suppressed")
        } else {
            None
        }
    }
    fn run_buffer(
        &self,
        data: &[u8],
        plan: DryRun,
        started: Instant,
        cancel: Option<&AtomicBool>,
        mut stopped: Option<&str>,
    ) -> Result<Investigation> {
        let mut session = self.session(plan.input_size)?;
        let mut inspected_bytes = 0;
        let mut operation_units = 0;
        for run in &plan.runs {
            if stopped.is_none() {
                stopped = self.interruption(started, cancel);
            }
            if let Some(reason) = stopped {
                session.skip(run.spec.clone(), reason)?;
                continue;
            }
            if !run.execute {
                session.skip(
                    run.spec.clone(),
                    run.reason.as_deref().expect("skipped reason"),
                )?;
                continue;
            }
            if run.offset + run.length > data.len() as u64 {
                return Err(limit("loaded prefix does not cover the prevalidated run"));
            }
            // This buffer is immutable and owned by the profile. Identity is computed
            // once as an explicit charged run, not secretly rehashed for each preview.
            session.execute_prechecked(
                data,
                run.spec.clone(),
                Capture {
                    parameters: self.record_parameters,
                    result: run.retain_result,
                },
                cancel,
            )?;
            inspected_bytes += run.length;
            operation_units += run.operation_units;
            let actual = session.runs().last().expect("recorded");
            if actual.state == State::Failed {
                stopped = Some("previous operation failed; further analysis suppressed");
            } else if actual.warning.is_some() {
                stopped = Some(
                    "previous result/metadata budget was exhausted; further analysis suppressed",
                );
            }
        }
        session.set_profile(Context {
            format_version: 1,
            depth: self.depth,
            budgets: self.budgets.clone(),
            input_read_bytes: data.len(),
            inspected_bytes,
            operation_units,
        })?;
        Ok(Investigation {
            session,
            plan,
            input_read_bytes: data.len(),
            inspected_bytes,
            operation_units,
            elapsed_ms: started.elapsed().as_millis(),
        })
    }
}
pub const OPERATIONS: &[crate::operations::Descriptor] = &[
    crate::operations::Descriptor {
        id: "investigation.profile.plan",
        category: "investigation",
        description: "Preflight a versioned bounded profile and expose exact ranges, operation units, effective budgets and skipped reasons without reading content.",
        input_type: "investigation.profile_request",
        output_type: "investigation.profile_plan",
        parameters: &[],
        deterministic: true,
        reversible: None,
        command: "investigate plan",
        example: "cube investigate plan sample.bin --depth preview --json",
    },
    crate::operations::Descriptor {
        id: "investigation.profile.run",
        category: "investigation",
        description: "Run an explicit local metadata/preview profile through shared core operations and record executed/skipped/failed provenance with cooperative limits.",
        input_type: "byte_reader",
        output_type: "investigation.session",
        parameters: &[],
        deterministic: false,
        reversible: None,
        command: "investigate run",
        example: "cube investigate run sample.bin --depth preview --identify --json",
    },
];
