//! Local video adapters; all parsing/algorithms stay in cube-core.
use clap::{Args, Subcommand};
use std::{error::Error, io::Write, path::PathBuf};
#[derive(Args)]
#[group(required=true,multiple=false,args=["stdout","output"])]
pub(crate) struct AudioOutput {
    #[arg(long)]
    stdout: bool,
    #[arg(long)]
    output: Option<PathBuf>,
}
#[derive(Subcommand)]
pub(crate) enum VideoCommand {
    /// Extract one raw PCM8 MOV track as WAV, preserving exact sample bytes
    Audio {
        path: PathBuf,
        #[arg(long)]
        track_id: u64,
        #[command(flatten)]
        destination: AudioOutput,
    },
    /// Compare selected MOV/JPEG pairs with exact frame/time/range provenance
    Compare {
        path: PathBuf,
        #[arg(long)]
        track_id: u64,
        #[arg(long,value_delimiter=',',num_args=1..=8)]
        pairs: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        channels: Option<Vec<String>>,
        #[arg(long, default_value_t = 1000)]
        max_results: usize,
    },
    /// Export explicitly indexed independent MOV/JPEG frames as new PNG files
    Frames {
        path: PathBuf,
        #[arg(long)]
        track_id: u64,
        #[arg(long,value_delimiter=',',num_args=1..=8)]
        indexes: Vec<usize>,
        #[arg(long)]
        output_dir: PathBuf,
    },
    /// Inspect declared MP4/MOV/WebM metadata; unsupported profiles fail explicitly
    Inspect {
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
}
pub(crate) fn run(command: VideoCommand, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        VideoCommand::Audio {
            path,
            track_id,
            destination,
        } => {
            let result = cube_core::media::video::audio::extract_file(&path, track_id)?;
            if destination.stdout {
                output.write_all(&result.data)?;
            } else {
                crate::pipelinecmd::write_new(
                    &destination.output.expect("required output group"),
                    &result.data,
                )?;
                writeln!(output, "{}", serde_json::to_string_pretty(&result)?)?;
            }
        }
        VideoCommand::Compare {
            path,
            track_id,
            pairs,
            channels,
            max_results,
        } => {
            let pairs = pairs
                .iter()
                .map(|s| {
                    let (a, b) = s
                        .split_once(':')
                        .ok_or("pair must use left:right frame indexes")?;
                    Ok::<_, Box<dyn Error>>((a.parse::<usize>()?, b.parse::<usize>()?))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let channels = channels
                .map(|names| {
                    names
                        .iter()
                        .map(|s| cube_core::image::Channel::parse(s))
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?;
            let report = cube_core::media::video::comparison::compare_file(
                &path,
                track_id,
                &pairs,
                channels.as_deref(),
                max_results,
            )?;
            writeln!(output, "{}", serde_json::to_string_pretty(&report)?)?;
        }
        VideoCommand::Frames {
            path,
            track_id,
            indexes,
            output_dir,
        } => {
            let frames = cube_core::media::video::frames::extract_file(&path, track_id, &indexes)?;
            let report = write_frames(&frames, &output_dir)?;
            writeln!(output, "{}", serde_json::to_string_pretty(&report)?)?;
        }
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

fn write_frames(
    frames: &[cube_core::media::video::frames::Frame],
    directory: &std::path::Path,
) -> Result<serde_json::Value, Box<dyn Error>> {
    if !directory.is_dir() {
        return Err("output directory must already exist".into());
    }
    let mut created = Vec::new();
    let result = (|| {
        let mut files = Vec::new();
        for frame in frames {
            let name = format!(
                "track-{}-frame-{}.png",
                frame.provenance.track_id, frame.provenance.frame_index
            );
            let path = directory.join(&name);
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)?;
            created.push(path);
            files.push((name, file));
        }
        let mut report = Vec::new();
        let mut bytes = 0usize;
        for (frame, (name, mut file)) in frames.iter().zip(files) {
            let png = frame.image.encode_png()?;
            bytes = bytes
                .checked_add(png.len())
                .ok_or("PNG output size overflows")?;
            if bytes > 96 * 1024 * 1024 {
                return Err("PNG batch output exceeds 96 MiB".into());
            }
            file.write_all(&png)?;
            file.flush()?;
            report.push(serde_json::json!({"file":name,"provenance":frame.provenance,"image":frame.image.info()}));
        }
        Ok(serde_json::json!({"frames":report,"png_bytes":bytes}))
    })();
    if result.is_err() {
        for path in created {
            let _ = std::fs::remove_file(path);
        }
    }
    result
}
