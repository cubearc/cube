//! Explicit caller-owned local file boundaries for investigation.
use crate::pipelinecmd::{read_file, write_new};
use clap::{Args, Subcommand};
use cube_core::{
    forensics,
    investigation::{
        profile::{Depth, Profile},
        session::{
            Capture, Export, MAX_INPUT_BYTES, MAX_RECORD_BYTES, RunSpec, Session, SessionLimits,
        },
        triage,
    },
    operations, pipeline,
};
use std::{
    error::Error,
    io::Write,
    path::{Path, PathBuf},
};
#[derive(Args)]
pub(crate) struct ReportOptions {
    #[arg(long)]
    json: bool,
    /// Explicit new JSON record destination; never overwrite existing files
    #[arg(long)]
    save: Option<PathBuf>,
    /// Retain and export only these session-local run IDs
    #[arg(long, value_delimiter = ',')]
    include_result: Vec<u32>,
    /// Retain and export parameters only for these run IDs; may contain keys/secrets
    #[arg(long, value_delimiter = ',')]
    include_parameters: Vec<u32>,
    #[arg(long)]
    include_hypotheses: bool,
    /// User hypotheses, never machine evidence (not saved unless explicitly included)
    #[arg(long)]
    hypothesis: Vec<String>,
}
impl ReportOptions {
    fn export(&self) -> Export {
        Export {
            include_results: self.include_result.clone(),
            include_parameters: self.include_parameters.clone(),
            include_hypotheses: self.include_hypotheses,
        }
    }
    fn write(&self, mut session: Session, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
        for text in &self.hypothesis {
            session.add_hypothesis(text, &[])?;
        }
        let record = session.to_json(&self.export())?;
        if let Some(path) = &self.save {
            write_new(path, record.as_bytes())?;
        }
        if self.json {
            writeln!(out, "{record}")?;
        } else {
            write!(out, "{}", session.human())?;
        }
        Ok(())
    }
}
#[derive(Args)]
pub(crate) struct ProfileOptions {
    /// Explicit local versioned JSON profile; definitions cannot grant file access
    #[arg(long)]
    profile: Option<PathBuf>,
    #[arg(long,value_parser=["metadata","preview"])]
    depth: Option<String>,
    #[arg(long)]
    identify: bool,
    /// Explicit JSON budget overrides; unspecified fields preserve loaded limits
    #[arg(long)]
    budgets: Option<String>,
}
impl ProfileOptions {
    fn load(&self) -> Result<Profile, Box<dyn Error>> {
        let mut profile = if let Some(path) = &self.profile {
            let data = read_file(path, pipeline::json::MAX_DOCUMENT_BYTES)?;
            Profile::parse(std::str::from_utf8(&data)?)?
        } else {
            Profile::default()
        };
        if let Some(depth) = &self.depth {
            profile.depth = if depth == "metadata" {
                Depth::Metadata
            } else {
                Depth::Preview
            };
        }
        if self.identify {
            profile.identify = true;
        }
        if let Some(budgets) = &self.budgets {
            let overrides = pipeline::json::parse(budgets)?;
            let overrides = overrides
                .as_object()
                .ok_or("budget overrides require a JSON object")?;
            let mut value = serde_json::to_value(&profile.budgets)?;
            for (key, item) in overrides {
                value
                    .as_object_mut()
                    .expect("budgets object")
                    .insert(key.clone(), item.clone());
            }
            profile.budgets =
                serde_json::from_value(value).map_err(|_| "invalid budget names or types")?;
        }
        profile.validate()?;
        Ok(profile)
    }
}
#[derive(Subcommand)]
pub(crate) enum InvestigateCommand {
    /// Inspect effective budgets and byte ranges without reading source contents
    Plan {
        path: PathBuf,
        #[command(flatten)]
        profile: ProfileOptions,
        #[arg(long)]
        json: bool,
    },
    /// Run a bounded local profile; optional saving and selected content are explicit
    Run {
        path: PathBuf,
        #[command(flatten)]
        profile: ProfileOptions,
        #[command(flatten)]
        report: ReportOptions,
    },
    /// Suggest available inspections from up to 12 header bytes; never execute suggestions
    Triage {
        path: PathBuf,
        #[arg(long, default_value_t = 12)]
        max_read_bytes: usize,
        #[arg(long)]
        json: bool,
    },
    /// Record a byte-input pipeline with optional identity and explicit saving
    Pipeline {
        path: PathBuf,
        #[arg(long)]
        steps: String,
        #[arg(long)]
        identify: bool,
        #[command(flatten)]
        report: ReportOptions,
    },
    /// Render a bounded versioned record with safe text/JSON and explicit export controls
    Report {
        record: PathBuf,
        #[command(flatten)]
        report: ReportOptions,
    },
    /// Verify identity and all contracts before replay; does not load input paths from records
    Replay {
        record: PathBuf,
        path: PathBuf,
        #[command(flatten)]
        report: ReportOptions,
    },
}
fn load_record(path: &Path) -> Result<Session, Box<dyn Error>> {
    let bytes = read_file(path, MAX_RECORD_BYTES)?;
    Ok(Session::from_json(std::str::from_utf8(&bytes)?)?)
}
pub(crate) fn run(command: InvestigateCommand, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        InvestigateCommand::Plan {
            path,
            profile,
            json,
        } => {
            let profile = profile.load()?;
            let file = forensics::open_regular(&path)?;
            let plan = profile.dry_run(file.metadata()?.len())?;
            if json {
                writeln!(out, "{}", serde_json::to_string_pretty(&plan)?)?;
            } else {
                writeln!(
                    out,
                    "Profile v1 {:?}: input {} bytes; read at most {} bytes; inspect at most {} bytes; {} operation units",
                    plan.depth,
                    plan.input_size,
                    plan.input_read_bytes,
                    plan.inspected_bytes,
                    plan.operation_units
                )?;
                writeln!(out, "Budgets: {}", serde_json::to_string(&plan.budgets)?)?;
                for run in &plan.runs {
                    writeln!(
                        out,
                        "Run #{} {} v{}: offset {}, length {}, execute {}, reason {:?}",
                        run.id,
                        run.operation,
                        run.version,
                        run.offset,
                        run.length,
                        run.execute,
                        run.reason
                    )?;
                }
            }
        }
        InvestigateCommand::Run {
            path,
            profile,
            report,
        } => {
            let mut profile = profile.load()?;
            for id in &report.include_result {
                if !profile.retain_results.contains(id) {
                    profile.retain_results.push(*id);
                }
            }
            let pipeline_id = 1
                + u32::from(profile.identify)
                + if profile.depth == Depth::Preview {
                    2
                } else {
                    0
                }
                + u32::from(profile.pipeline.is_some());
            if profile.pipeline.is_some() && report.include_parameters.contains(&pipeline_id) {
                profile.record_parameters = true;
            }
            let mut file = forensics::open_regular(&path)?;
            let result = profile.run_reader(&mut file, None)?;
            if !report.json {
                writeln!(
                    out,
                    "Profile: read {} bytes; charged {} inspected bytes; {} operation units; {} ms (diagnostic)",
                    result.input_read_bytes,
                    result.inspected_bytes,
                    result.operation_units,
                    result.elapsed_ms
                )?;
            }
            report.write(result.session, out)?;
        }
        InvestigateCommand::Triage {
            path,
            max_read_bytes,
            json,
        } => {
            let mut file = forensics::open_regular(&path)?;
            let result = triage::reader(&mut file, max_read_bytes, &operations::catalog())?;
            if json {
                writeln!(out, "{}", serde_json::to_string_pretty(&result)?)?;
            } else {
                write!(out, "{}", result.human())?;
            }
        }
        InvestigateCommand::Pipeline {
            path,
            steps,
            identify,
            report,
        } => {
            let steps = pipeline::json::steps(&steps)?;
            pipeline::Plan::validate(
                pipeline::Kind::Bytes,
                steps.clone(),
                pipeline::Limits::default(),
            )?;
            let data = read_file(&path, MAX_INPUT_BYTES)?;
            let mut session = Session::new(data.len() as u64, SessionLimits::default())?;
            if identify {
                session.execute(&data, RunSpec::Identity, Capture::default(), None)?;
            }
            let id = session.runs().len() as u32 + 1;
            session.execute(
                &data,
                RunSpec::pipeline(steps, pipeline::Limits::default()),
                Capture {
                    parameters: report.include_parameters.contains(&id),
                    result: report.include_result.contains(&id),
                },
                None,
            )?;
            report.write(session, out)?;
        }
        InvestigateCommand::Report { record, report } => {
            report.write(load_record(&record)?, out)?
        }
        InvestigateCommand::Replay {
            record,
            path,
            report,
        } => {
            let original = load_record(&record)?;
            let data = read_file(&path, MAX_INPUT_BYTES)?;
            report.write(original.replay(&data, None)?, out)?;
        }
    }
    Ok(())
}
