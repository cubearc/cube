//! Local video adapters; all parsing/algorithms stay in cube-core.
use clap::Subcommand;
use std::{error::Error, io::Write, path::PathBuf};
#[derive(Subcommand)]
pub(crate) enum VideoCommand {
    /// Inspect declared MP4/MOV/WebM metadata; unsupported profiles fail explicitly
    Inspect {
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
}
pub(crate) fn run(command: VideoCommand, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        VideoCommand::Inspect { path, json } => {
            let info = cube_core::media::video::inspect_file(&path)?;
            if json {
                writeln!(output, "{}", serde_json::to_string_pretty(&info)?)?;
            } else {
                writeln!(
                    output,
                    "{}; {} bytes; duration seconds {:?}; {}",
                    info.container, info.file_bytes, info.duration_seconds, info.verification
                )?;
                for t in info.tracks {
                    writeln!(
                        output,
                        "track {} {} codec {:?} dimensions {:?}x{:?} time base {:?} duration {:?}",
                        t.id, t.kind, t.codec_id, t.width, t.height, t.time_base, t.duration_ticks
                    )?;
                }
            }
        }
    }
    Ok(())
}
