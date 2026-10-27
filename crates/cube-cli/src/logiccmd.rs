//! Tagged inert Boolean models and exact JSON evidence.
use clap::{Args, Subcommand};
use cube_core::logic::boolean as b;
use std::{error::Error, io::Write};
#[derive(Subcommand)]
pub(crate) enum Command {
    Boolean {
        #[command(subcommand)]
        command: Boolean,
    },
}
#[derive(Subcommand)]
pub(crate) enum Boolean {
    Evaluate(Evaluate),
    Analyze(Model),
}
#[derive(Args)]
pub(crate) struct Model {
    #[command(flatten)]
    input: crate::Input,
    #[arg(long,default_value_t=b::MAX_WORK)]
    max_work: u64,
    #[arg(long, default_value_t = 32)]
    max_rows: usize,
    #[arg(long,default_value_t=b::MAX_MEMORY)]
    max_memory_bytes: usize,
    #[arg(long,default_value_t=b::MAX_REPORT)]
    max_report_bytes: usize,
}
#[derive(Args)]
pub(crate) struct Evaluate {
    #[command(flatten)]
    model: Model,
    /// JSON array of {name,value} records with Boolean values
    #[arg(long)]
    assignment: String,
}
impl Model {
    fn load(self) -> Result<(b::Model, b::Limits), Box<dyn Error>> {
        let text = self
            .input
            .read_text_limit(b::MAX_INPUT, "logic.boolean.analyze")?;
        Ok((
            b::parse(&text)?,
            b::Limits {
                max_work: self.max_work,
                max_rows: self.max_rows,
                max_memory_bytes: self.max_memory_bytes,
                max_report_bytes: self.max_report_bytes,
            },
        ))
    }
}
pub(crate) fn run(command: Command, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let Command::Boolean { command } = command;
    match command {
        Boolean::Analyze(args) => {
            let (model, limits) = args.load()?;
            serde_json::to_writer(&mut *out, &b::analyze(&model, limits)?)?;
        }
        Boolean::Evaluate(args) => {
            let assignment = b::parse_assignment(&args.assignment)?;
            let (model, limits) = args.model.load()?;
            serde_json::to_writer(&mut *out, &b::evaluate(&model, &assignment, limits)?)?;
        }
    }
    writeln!(out)?;
    Ok(())
}
