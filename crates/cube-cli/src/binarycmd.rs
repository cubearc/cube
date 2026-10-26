//! Byte-only CLI adaptation; algorithms and unit validation live in cube-core.
use crate::{DecodeArgs, Input, Output};
use clap::{Args, Subcommand};
use cube_core::{
    binary::{self, transform as t},
    encoding,
};
use std::{
    error::Error,
    io::{self, Write},
};
#[derive(Args)]
#[group(required=true,multiple=false,args=["key","key_hex"])]
pub(crate) struct XorArgs {
    #[command(flatten)]
    input: Input,
    /// Nonempty UTF-8 key
    #[arg(short = 'k', long)]
    key: Option<String>,
    /// Nonempty key as hexadecimal digits
    #[arg(long, value_name = "HEX")]
    key_hex: Option<String>,
    /// Binary output representation: hex adds a newline; raw writes exact bytes
    #[arg(long,value_enum,default_value_t=Output::Hex)]
    output: Output,
}
#[derive(Args)]
pub(crate) struct MaskArgs {
    #[command(flatten)]
    data: DecodeArgs,
    /// Exact equally long operand as hex; no key repetition
    #[arg(long)]
    key_hex: String,
}
#[derive(Args)]
pub(crate) struct UnitArgs {
    #[command(flatten)]
    data: DecodeArgs,
    /// Independent unsigned big-endian unit width: 1, 2, 4 or 8 bytes
    #[arg(long, default_value_t = 1)]
    unit_bytes: usize,
}
#[derive(Args)]
pub(crate) struct ShiftArgs {
    #[command(flatten)]
    unit: UnitArgs,
    /// Zero-filled logical shift count, at most the unit bit width
    #[arg(long)]
    bits: u8,
}
#[derive(Subcommand)]
pub(crate) enum BinaryCommand {
    /// XOR bytes with a repeating key (not secure encryption)
    Xor(XorArgs),
    /// Per-byte AND of equal-length operands; <=16 MiB
    And(MaskArgs),
    /// Per-byte OR of equal-length operands; <=16 MiB
    Or(MaskArgs),
    /// Invert eight bits per byte, preserving length; <=16 MiB
    Not(DecodeArgs),
    /// Reverse bytes, without text interpretation; <=16 MiB
    Reverse(DecodeArgs),
    /// Logical left shift within independent fixed-width unsigned units
    ShiftLeft(ShiftArgs),
    /// Logical right shift within independent fixed-width unsigned units
    ShiftRight(ShiftArgs),
    /// Reverse byte order within complete units, preserving unit order
    SwapEndian(UnitArgs),
}
fn data(args: DecodeArgs, op: &'static str) -> Result<(Vec<u8>, Output), Box<dyn Error>> {
    Ok((args.input.read_bounded(t::MAX_INPUT, op)?, args.output))
}
fn key(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if hex.len() > t::MAX_INPUT * 2 {
        return Err("binary operand exceeds 16 MiB".into());
    }
    Ok(encoding::hex_decode(hex)?)
}
pub(crate) fn run(command: BinaryCommand, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let (bytes, format) = match command {
        BinaryCommand::Xor(args) => {
            let key = match (args.key, args.key_hex) {
                (Some(key), None) => key.into_bytes(),
                (None, Some(key)) => encoding::hex_decode(&key)?,
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "provide exactly one of --key or --key-hex",
                    )
                    .into());
                }
            };
            (binary::xor(&args.input.read()?, &key)?, args.output)
        }
        BinaryCommand::And(args) => {
            let key = key(&args.key_hex)?;
            let (bytes, format) = data(args.data, "binary.and")?;
            (t::and(&bytes, &key)?, format)
        }
        BinaryCommand::Or(args) => {
            let key = key(&args.key_hex)?;
            let (bytes, format) = data(args.data, "binary.or")?;
            (t::or(&bytes, &key)?, format)
        }
        BinaryCommand::Not(args) => {
            let (bytes, format) = data(args, "binary.not")?;
            (t::not(&bytes)?, format)
        }
        BinaryCommand::Reverse(args) => {
            let (bytes, format) = data(args, "binary.reverse")?;
            (t::reverse(&bytes)?, format)
        }
        BinaryCommand::ShiftLeft(args) => {
            let (bytes, format) = data(args.unit.data, "binary.shift_left")?;
            (
                t::shift_left(&bytes, args.bits, args.unit.unit_bytes)?,
                format,
            )
        }
        BinaryCommand::ShiftRight(args) => {
            let (bytes, format) = data(args.unit.data, "binary.shift_right")?;
            (
                t::shift_right(&bytes, args.bits, args.unit.unit_bytes)?,
                format,
            )
        }
        BinaryCommand::SwapEndian(args) => {
            let (bytes, format) = data(args.data, "binary.swap_endian")?;
            (t::swap_endian(&bytes, args.unit_bytes)?, format)
        }
    };
    crate::write_bytes(output, &bytes, format)?;
    Ok(())
}
