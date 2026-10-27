//! Exact decimal transport for bounded counts; algorithms stay in Core.
use clap::Args;
use cube_core::numbers::{combinatorics as c, modular};
use std::{error::Error, io::Write};
#[derive(Args)]
pub(crate) struct CountLimits {
    #[arg(long,default_value_t=c::MAX_WORK)]
    max_work: u64,
    #[arg(long,default_value_t=c::MAX_REPORT_BYTES)]
    max_report_bytes: usize,
    #[arg(long)]
    json: bool,
}
impl CountLimits {
    fn limits(&self) -> c::Limits {
        c::Limits {
            max_work: self.max_work,
            max_report_bytes: self.max_report_bytes,
        }
    }
}
#[derive(Args)]
pub(crate) struct Pair {
    /// Strict unsigned decimal, n<=128
    #[arg(long, allow_hyphen_values = true)]
    n: String,
    #[arg(long, allow_hyphen_values = true)]
    k: String,
    #[command(flatten)]
    limits: CountLimits,
}
#[derive(Args)]
pub(crate) struct Groups {
    /// Comma-separated group sizes, checked total<=128; use '' for an empty list
    #[arg(long, allow_hyphen_values = true)]
    groups: String,
    #[command(flatten)]
    limits: CountLimits,
}
#[derive(Args)]
pub(crate) struct Pascal {
    /// Number of rows: 0..129 (129 includes row index 128)
    #[arg(long, allow_hyphen_values = true)]
    rows: String,
    #[arg(long,default_value_t=c::MAX_WORK)]
    max_work: u64,
    #[arg(long,default_value_t=c::MAX_TRIANGLE_BYTES)]
    max_output_bytes: usize,
}
fn output(r: c::Count, json: bool, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    if json {
        serde_json::to_writer(&mut *out, &r)?;
        writeln!(out)?;
    } else {
        writeln!(out, "{}", r.value)?;
    }
    Ok(())
}
pub(crate) fn pair(args: Pair, ordered: bool, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let op = if ordered {
        "number.permutations"
    } else {
        "number.binomial"
    };
    let n = modular::parse_operand(&args.n, op, "n")?;
    let k = modular::parse_operand(&args.k, op, "k")?;
    let r = if ordered {
        c::permutations(n, k, args.limits.limits())?
    } else {
        c::binomial(n, k, args.limits.limits())?
    };
    output(r, args.limits.json, out)
}
pub(crate) fn groups(args: Groups, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    const OP: &str = "number.multinomial";
    if args.groups.len() > 4096 || args.groups.split(',').count() > 128 {
        return Err(cube_core::errors::OperationError::new(
            cube_core::errors::ErrorCode::ResourceLimit,
            "group transport exceeds 128 entries/4096 bytes",
        )
        .operation(OP)
        .parameter("groups")
        .into());
    }
    let groups = if args.groups.is_empty() {
        Vec::new()
    } else {
        args.groups
            .split(',')
            .map(|s| modular::parse_operand(s, OP, "groups"))
            .collect::<Result<Vec<_>, _>>()?
    };
    output(
        c::multinomial(&groups, args.limits.limits())?,
        args.limits.json,
        out,
    )
}
pub(crate) fn pascal(args: Pascal, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let n = modular::parse_operand(&args.rows, "number.pascal", "rows")?;
    let r = c::pascal(n, args.max_work, args.max_output_bytes)?;
    serde_json::to_writer(&mut *out, &r)?;
    writeln!(out)?;
    Ok(())
}
