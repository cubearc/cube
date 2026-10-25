//! Explicit local key input and safe reporting; algorithms remain in cube-core.
use clap::Subcommand;
use cube_core::{
    blockchain::{self, Network, PrivateInput},
    errors::OperationError,
};
use std::{error::Error, io::Write};
#[derive(Subcommand)]
pub(crate) enum Command {
    /// Deliberately export private material; explicit intent and destination required
    #[command(group(clap::ArgGroup::new("private_destination").required(true).multiple(false).args(["output","stdout"])))]
    #[command(group(clap::ArgGroup::new("export_encoding").multiple(false).args(["compressed","uncompressed"])))]
    Export {
        #[command(flatten)]
        input: crate::Input,
        #[arg(long, default_value = "raw")]
        format: String,
        /// Exact private output: raw, hex or wif
        #[arg(long)]
        to: String,
        #[arg(long)]
        network: Option<String>,
        #[arg(long)]
        compressed: bool,
        #[arg(long)]
        uncompressed: bool,
        /// Required confirmation that output contains private material
        #[arg(long, required = true)]
        private_output: bool,
        /// New exclusive private file; Unix mode 0600, no overwrite
        #[arg(long)]
        output: Option<std::path::PathBuf>,
        /// Exact private bytes without metadata or newline
        #[arg(long)]
        stdout: bool,
    },
    /// Derive public SEC1/P2PKH offline; network/encoding must be explicit
    #[command(group(clap::ArgGroup::new("bitcoin_encoding").required(true).multiple(false).args(["compressed","uncompressed"])))]
    Bitcoin {
        #[command(flatten)]
        input: crate::Input,
        #[arg(long, default_value = "raw")]
        format: String,
        #[arg(long)]
        network: String,
        #[arg(long)]
        compressed: bool,
        #[arg(long)]
        uncompressed: bool,
        #[arg(long)]
        json: bool,
    },
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
        Command::Export {
            input,
            format,
            to,
            network,
            compressed,
            uncompressed,
            private_output,
            output,
            stdout,
        } => {
            if !private_output || stdout == output.is_some() || (compressed && uncompressed) {
                return Err(
                    "private output requires explicit intent and exactly one destination/encoding"
                        .into(),
                );
            }
            let network = network.as_deref().map(Network::parse).transpose()?;
            let compression = (compressed || uncompressed).then_some(compressed);
            let target = blockchain::private::OutputFormat::parse(&to)?;
            let bytes =
                input.read_bounded(blockchain::MAX_MATERIAL_BYTES, "blockchain.private.export")?;
            let exported = blockchain::private::export(
                material(&bytes, &format, "blockchain.private.export")?,
                target,
                network,
                compression,
            )?
            .into_private_bytes();
            if let Some(path) = output {
                write_private_new(&path, &exported)?;
                writeln!(out, "Wrote private output to a new file")?;
            } else {
                out.write_all(&exported)?;
            }
        }
        Command::Bitcoin {
            input,
            format,
            network,
            compressed,
            uncompressed,
            json,
        } => {
            if compressed == uncompressed {
                return Err("select exactly one public-key encoding".into());
            }
            let bytes =
                input.read_bounded(blockchain::MAX_MATERIAL_BYTES, "blockchain.bitcoin.derive")?;
            let result = blockchain::bitcoin::derive(
                material(&bytes, &format, "blockchain.bitcoin.derive")?,
                Network::parse(&network)?,
                compressed,
            )?;
            if json {
                writeln!(out, "{}", serde_json::to_string_pretty(&result)?)?;
            } else {
                writeln!(out, "{}", result.address)?;
            }
        }
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

fn write_private_new(path: &std::path::Path, data: &[u8]) -> Result<(), OperationError> {
    write_private_with(path, data, |file, bytes| {
        file.write_all(bytes).and_then(|()| file.flush())
    })
    .map_err(|e| e.operation("blockchain.private.export").parameter("output"))
}
fn write_private_with(
    path: &std::path::Path,
    data: &[u8],
    write: impl FnOnce(&mut std::fs::File, &[u8]) -> std::io::Result<()>,
) -> Result<(), OperationError> {
    use cube_core::errors::ErrorCode;
    if data.len() > blockchain::MAX_MATERIAL_BYTES {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "private output exceeds 128 bytes",
        )
        .operation("blockchain.private.export"));
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(OperationError::io)?;
    let result = write(&mut file, data);
    drop(file);
    if result.is_err() {
        let _cleanup = std::fs::remove_file(path);
    }
    result.map_err(OperationError::io)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_file_failure_removes_only_its_new_output() {
        let path =
            std::env::temp_dir().join(format!("cube-private-failure-{}", std::process::id()));
        assert!(!path.exists());
        let result = write_private_with(&path, b"public test material", |file, bytes| {
            file.write_all(&bytes[..2])?;
            Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "synthetic failure",
            ))
        });
        assert!(result.is_err() && !path.exists());
        assert!(write_private_new(&path, &[0; 129]).is_err() && !path.exists());
    }
}
