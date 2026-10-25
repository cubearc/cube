//! Explicit local key input and safe reporting; algorithms remain in cube-core.
use clap::Subcommand;
use cube_core::{
    blockchain::{self, Network, PrivateInput},
    errors::OperationError,
};
use std::{error::Error, io::Write};
#[derive(Subcommand)]
pub(crate) enum Command {
    /// Validate WIF without exporting material; prefer exact stdin to argv
    Wif {
        #[command(flatten)]
        input: crate::Input,
        /// Encoded family: mainnet or testnet (shared with signet/regtest)
        #[arg(long)]
        expected_network: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Validate exact raw32 or strict hex64 secp256k1 scalar; no private output
    Scalar {
        #[command(flatten)]
        input: crate::Input,
        #[arg(long, default_value = "raw")]
        format: String,
        #[arg(long)]
        json: bool,
    },
}
fn material<'a>(
    bytes: &'a [u8],
    format: &str,
    operation: &'static str,
) -> Result<PrivateInput<'a>, OperationError> {
    blockchain::PrivateInput::parse(format, bytes).map_err(|e| e.operation(operation))
}
pub(crate) fn run(command: Command, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        Command::Wif {
            input,
            expected_network,
            json,
        } => {
            let bytes =
                input.read_bounded(blockchain::MAX_MATERIAL_BYTES, "blockchain.wif.validate")?;
            let PrivateInput::Wif(text) = material(&bytes, "wif", "blockchain.wif.validate")?
            else {
                return Err("inconsistent WIF transport".into());
            };
            let network = expected_network
                .as_deref()
                .map(Network::parse)
                .transpose()?;
            let info = blockchain::wif::validate(text, network)?;
            if json {
                writeln!(out, "{}", serde_json::to_string_pretty(&info)?)?;
            } else {
                writeln!(
                    out,
                    "Valid WIF; version 0x{:02x}; family {:?}; compressed public-key intent: {}",
                    info.version, info.network_family, info.compressed
                )?;
            }
        }
        Command::Scalar {
            input,
            format,
            json,
        } => {
            let bytes =
                input.read_bounded(blockchain::MAX_MATERIAL_BYTES, "blockchain.scalar.validate")?;
            let info = blockchain::validate_scalar(material(
                &bytes,
                &format,
                "blockchain.scalar.validate",
            )?)?;
            if json {
                writeln!(out, "{}", serde_json::to_string_pretty(&info)?)?;
            } else {
                writeln!(
                    out,
                    "Valid secp256k1 scalar (32 bytes); private material omitted"
                )?;
            }
        }
    }
    Ok(())
}
