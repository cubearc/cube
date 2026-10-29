//! Explicit ordered operands; composition algorithms remain in Core.
use clap::Subcommand;
use cube_core::{
    composition, encoding,
    errors::{ErrorCode, OperationError},
};
use std::{error::Error, io::Write};

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Parse bounded JSON without executing content
    ParseJson {
        #[arg(long)]
        text: String,
    },
    /// Select a JSON field or array index without converting its type
    Select {
        #[arg(long)]
        json: String,
        #[arg(long, default_value = "")]
        pointer: String,
    },
    /// Use a JSON string as exact text without an added newline
    JsonText {
        #[arg(long)]
        json: String,
    },
    /// Parse exact canonical signed decimal integer text
    Integer {
        #[arg(long, allow_hyphen_values = true)]
        text: String,
    },
    /// Find the maximum in a nonempty JSON array of exact integers
    Maximum {
        #[arg(long)]
        values: String,
    },
    /// Add exact integers without wrapping or precision loss
    Add {
        #[arg(long, allow_hyphen_values = true)]
        left: String,
        #[arg(long, allow_hyphen_values = true)]
        right: String,
    },
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
    use composition::structured as s;
    match command {
        Command::ParseJson { text } => writeln!(output, "{}", s::parse_json(&text)?)?,
        Command::Select { json, pointer } => writeln!(
            output,
            "{}",
            s::select_field(&s::parse_json(&json)?, &pointer)?
        )?,
        Command::JsonText { json } => {
            output.write_all(s::json_text(&s::parse_json(&json)?)?.as_bytes())?
        }
        Command::Integer { text } => writeln!(output, "{}", s::parse_integer(&text)?)?,
        Command::Maximum { values } => {
            let values: Vec<i64> =
                serde_json::from_value(s::parse_json(&values)?).map_err(|_| {
                    OperationError::new(
                        ErrorCode::InvalidInput,
                        "maximum requires a JSON array of integers",
                    )
                    .operation("number.maximum")
                })?;
            writeln!(output, "{}", s::maximum(&values)?)?;
        }
        Command::Add { left, right } => writeln!(
            output,
            "{}",
            s::add(s::parse_integer(&left)?, s::parse_integer(&right)?)?
        )?,
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
