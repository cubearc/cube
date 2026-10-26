//! Strict decimal transport; arithmetic belongs to cube-core.
use cube_core::numbers::modular;
use std::{error::Error, io::Write};
#[derive(clap::Args)]
pub(crate) struct Pair {
    /// Unsigned decimal u64, <=20 ASCII digits; signs/prefixes are rejected
    #[arg(long, allow_hyphen_values = true)]
    a: String,
    #[arg(long, allow_hyphen_values = true)]
    b: String,
    /// Exact integers are serialized as decimal strings
    #[arg(long)]
    json: bool,
}
#[derive(clap::Args)]
pub(crate) struct Inverse {
    #[arg(long, allow_hyphen_values = true)]
    a: String,
    /// Explicit unsigned decimal u64 modulus >=2
    #[arg(long, allow_hyphen_values = true)]
    modulus: String,
    #[arg(long)]
    json: bool,
}
#[derive(clap::Args)]
pub(crate) struct Power {
    #[arg(long, allow_hyphen_values = true)]
    base: String,
    #[arg(long, allow_hyphen_values = true)]
    exponent: String,
    /// Explicit positive unsigned decimal u64 modulus
    #[arg(long, allow_hyphen_values = true)]
    modulus: String,
    #[arg(long)]
    json: bool,
}
fn scalar(value: u64, json: bool, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    if json {
        writeln!(output, "{}", serde_json::json!({"value":value.to_string()}))?;
    } else {
        writeln!(output, "{value}")?;
    }
    Ok(())
}
pub(crate) fn pair(
    args: Pair,
    extended: bool,
    output: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    let op = if extended {
        "number.extended_gcd"
    } else {
        "number.gcd"
    };
    let a = modular::parse_operand(&args.a, op, "a")?;
    let b = modular::parse_operand(&args.b, op, "b")?;
    if extended {
        let r = modular::extended_gcd(a, b)?;
        if args.json {
            writeln!(output, "{}", serde_json::to_string(&r)?)?;
        } else {
            writeln!(output, "gcd={} x={} y={}", r.gcd, r.x, r.y)?;
        }
        Ok(())
    } else {
        scalar(modular::gcd(a, b)?, args.json, output)
    }
}
pub(crate) fn inverse(args: Inverse, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    const OP: &str = "number.mod_inverse";
    let a = modular::parse_operand(&args.a, OP, "a")?;
    let m = modular::parse_operand(&args.modulus, OP, "modulus")?;
    scalar(modular::mod_inverse(a, m)?, args.json, output)
}
pub(crate) fn power(args: Power, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    const OP: &str = "number.mod_pow";
    let base = modular::parse_operand(&args.base, OP, "base")?;
    let exponent = modular::parse_operand(&args.exponent, OP, "exponent")?;
    let m = modular::parse_operand(&args.modulus, OP, "modulus")?;
    scalar(modular::mod_pow(base, exponent, m)?, args.json, output)
}
