//! CLI adapters for local PCM WAVE operations; algorithms stay in Rust core.
use clap::Subcommand;
use cube_core::media::wave;
use std::{error::Error, io::Write, path::PathBuf};
#[derive(Subcommand)]
pub(crate) enum AudioCommand {
    /// Inspect integer PCM RIFF/WAVE metadata/chunks without reading audio payloads
    Inspect {
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
}
pub(crate) fn run(command: AudioCommand, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        AudioCommand::Inspect { path, json } => {
            let info = wave::inspect_file(&path)?;
            if json {
                writeln!(output, "{}", serde_json::to_string_pretty(&info)?)?;
            } else {
                writeln!(
                    output,
                    "{}; PCM {}-bit; {} channels; {} frames/sec",
                    info.container,
                    info.format.bits_per_sample,
                    info.format.channels,
                    info.format.sample_rate
                )?;
                writeln!(
                    output,
                    "Frames: {}; duration seconds: {}/{}; trailing bytes: {}",
                    info.frames,
                    info.duration_seconds.numerator,
                    info.duration_seconds.denominator,
                    info.trailing_bytes
                )?;
                for chunk in info.chunks {
                    writeln!(
                        output,
                        "{:08x} {} bytes={} data_offset={} padding={}",
                        chunk.offset, chunk.id, chunk.size, chunk.data_offset, chunk.padding_bytes
                    )?;
                }
            }
        }
    }
    Ok(())
}
