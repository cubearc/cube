//! Exact finite model JSON transport, without inference or rounded counts.
use clap::{Args, Subcommand};
use cube_core::probability as p;
use std::{error::Error, io::Write};
#[derive(Args)]
pub(crate) struct Model {
    #[command(flatten)]
    input: crate::Input,
    #[arg(long,default_value_t=p::MAX_WORK)]
    max_work: usize,
    #[arg(long,default_value_t=p::MAX_MEMORY)]
    max_memory_bytes: usize,
    #[arg(long,default_value_t=p::MAX_REPORT)]
    max_report_bytes: usize,
}
#[derive(Subcommand)]
pub(crate) enum Command {
    Finite {
        #[command(subcommand)]
        command: Finite,
    },
    Dice {
        #[command(subcommand)]
        command: Dice,
    },
}
#[derive(Subcommand)]
pub(crate) enum Finite {
    Analyze(Model),
}
#[derive(Subcommand)]
pub(crate) enum Dice {
    Sum(Model),
    Compare(Model),
}
pub(crate) fn run(command: Command, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let (args, kind, op) = match command {
        Command::Finite {
            command: Finite::Analyze(a),
        } => (a, 0, "probability.finite.analyze"),
        Command::Dice {
            command: Dice::Sum(a),
        } => (a, 1, "probability.dice.sum"),
        Command::Dice {
            command: Dice::Compare(a),
        } => (a, 2, "probability.dice.compare"),
    };
    let text = args.input.read_text_limit(p::MAX_INPUT, op)?;
    let limits = p::Limits {
        max_work: args.max_work,
        max_memory_bytes: args.max_memory_bytes,
        max_report_bytes: args.max_report_bytes,
    };
    match kind {
        0 => serde_json::to_writer(
            &mut *out,
            &p::finite_analyze(&p::parse_finite(&text)?, limits)?,
        )?,
        1 => serde_json::to_writer(&mut *out, &p::dice_sum(&p::parse_dice(&text)?, limits)?)?,
        _ => serde_json::to_writer(&mut *out, &p::dice_compare(&p::parse_pair(&text)?, limits)?)?,
    };
    writeln!(out)?;
    Ok(())
}
