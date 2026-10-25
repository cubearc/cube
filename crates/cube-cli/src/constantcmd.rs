//! Numeric CLI transport; computation and selection validation stay in cube-core.
use clap::Args;
use cube_core::numbers::constants::{self, Constant, DigitOptions};
use std::{error::Error, io::Write};

#[derive(Args)]
pub(crate) struct DigitsArgs {
    /// Exact constant name: pi, e, phi or sqrt2
    constant: String,
    /// Zero-based fractional start (default 0); excludes integer part/decimal point
    #[arg(long, conflicts_with = "indices")]
    start: Option<usize>,
    /// Range length (default 16); no implicit expansion of max-digits
    #[arg(long, conflicts_with = "indices")]
    count: Option<usize>,
    /// Ordered comma-separated indexes instead of a range; <=1024, empty allowed
    #[arg(long)]
    indices: Option<String>,
    /// Explicit certified prefix budget; hard cap 4096
    #[arg(long, default_value_t = constants::DEFAULT_MAX_DIGITS)]
    max_digits: usize,
    /// Include integer part, convention, prefix budget/hash and certification
    #[arg(long)]
    json: bool,
}

pub(crate) fn run(args: DigitsArgs, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let indices = args
        .indices
        .as_deref()
        .map(constants::parse_indices)
        .transpose()?;
    let result = constants::digits(
        Constant::parse(&args.constant)?,
        DigitOptions {
            start: args.start,
            count: args.count,
            indices: indices.as_deref(),
            max_digits: args.max_digits,
        },
    )?;
    if args.json {
        writeln!(out, "{}", serde_json::to_string_pretty(&result)?)?;
    } else {
        writeln!(out, "{}", result.digits)?;
    }
    Ok(())
}
