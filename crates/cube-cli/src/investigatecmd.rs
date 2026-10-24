//! Explicit caller-owned local file boundaries for investigation.
use crate::pipelinecmd::{read_file, write_new};
use clap::{Args, Subcommand};
use cube_core::{
    forensics,
    investigation::{
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
#[derive(Subcommand)]
pub(crate) enum InvestigateCommand {
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
