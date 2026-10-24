//! Privacy-controlled records, strict loading, safe rendering and verified replay.
use super::*;
use std::sync::atomic::AtomicBool;
impl Session {
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
            if options.include_results.contains(&run.id)
                && run.retained.is_none()
                && run.state == State::Succeeded
                && run.warning.is_none()
            {
                return Err(invalid(
                    "selected result was not retained; omitted content cannot be recovered",
                ));
            }
            if options.include_parameters.contains(&run.id)
                && run.parameters.redacted()
                && run.state != State::Skipped
            {
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
    pub(super) fn validate(&self) -> Result<()> {
        self.limits.validate()?;
        if let Some(context) = &self.profile {
            context.validate()?;
        }
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
        if let Some(context) = &self.profile {
            result.set_profile(context.clone())?;
        }
        for run in &self.runs {
            if run.state == State::Skipped {
                result.skip(
                    run.parameters.clone(),
                    "original run was skipped; replay did not execute it",
                )?;
            } else {
                result.execute_prechecked(
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
