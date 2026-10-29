//! Explicit ordered operands; composition algorithms remain in Core.
use clap::Subcommand;
use cube_core::{
    composition, encoding,
    errors::{ErrorCode, OperationError},
};
use std::{error::Error, io::Write};

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Join exact Unicode text without normalization or an added newline
    Text {
        #[arg(long)]
        left: String,
        #[arg(long)]
        right: String,
        #[arg(long, default_value = "")]
        separator: String,
    },
    /// Join exact binary operands supplied as explicit hexadecimal digits
    Bytes {
        #[arg(long)]
        left_hex: String,
        #[arg(long)]
        right_hex: String,
        #[arg(long, value_enum, default_value_t = crate::Output::Hex)]
        output: crate::Output,
    },
}
fn bytes(text: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if text.len() > composition::MAX_BYTES * 2 {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "hex operand exceeds 1 MiB decoded bytes",
        )
        .operation("bytes.concat")
        .into());
    }
    Ok(encoding::hex_decode(text)?)
}
pub(crate) fn run(command: Command, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        Command::Text {
            left,
            right,
            separator,
        } => output.write_all(composition::concat_text(&left, &right, &separator)?.as_bytes())?,
        Command::Bytes {
            left_hex,
            right_hex,
            output: format,
        } => {
            let joined = composition::concat_bytes(&bytes(&left_hex)?, &bytes(&right_hex)?)?;
            crate::write_bytes(output, &joined, format)?;
        }
    }
    Ok(())
}
