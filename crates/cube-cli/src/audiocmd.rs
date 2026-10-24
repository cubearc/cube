//! CLI adapters for local PCM WAVE operations; algorithms stay in Rust core.
use clap::{Args, Subcommand};
use cube_core::media::wave;
use std::{error::Error, io::Write, path::PathBuf};
#[derive(Args)]
pub(crate) struct RangeArgs {
    path: PathBuf,
    /// Ordered unique zero-based channel indexes; omit for all channels
    #[arg(long, value_delimiter = ',')]
    channels: Option<Vec<u16>>,
    /// Absolute zero-based start frame, not byte offset
    #[arg(long, default_value_t = 0)]
    start_frame: u64,
    /// Frame count; omit to select through end, zero selects an empty interval
    #[arg(long)]
    frames: Option<u64>,
}
impl RangeArgs {
    fn selection(&self) -> cube_core::media::pcm::Selection {
        cube_core::media::pcm::Selection {
            start_frame: self.start_frame,
            frames: self.frames,
            channels: self.channels.clone(),
        }
    }
}
#[derive(Args)]
#[group(required=true,multiple=false,args=["output","stdout","json"])]
pub(crate) struct SampleOutput {
    /// New raw PCM file; interpretation metadata goes to stdout
    #[arg(long)]
    output: Option<PathBuf>,
    /// Exact raw PCM stdout without a newline or metadata
    #[arg(long)]
    stdout: bool,
    /// Exact signed values and raw hex, capped at 10000 samples
    #[arg(long)]
    json: bool,
}
#[derive(Subcommand)]
pub(crate) enum AudioCommand {
    /// Exact per-channel min/max buckets with absolute frame positions; no plot/FFT
    Waveform {
        #[command(flatten)]
        range: RangeArgs,
        #[arg(long, default_value_t = 1024)]
        bucket_frames: u64,
        #[arg(long, default_value_t = 1000)]
        max_buckets: usize,
        #[arg(long)]
        json: bool,
    },
    /// Extract exact PCM channels/frame ranges without resampling or normalization
    Samples {
        #[command(flatten)]
        range: RangeArgs,
        #[command(flatten)]
        destination: SampleOutput,
    },
    /// Inspect integer PCM RIFF/WAVE metadata/chunks without reading audio payloads
    Inspect {
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
}
pub(crate) fn run(command: AudioCommand, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        AudioCommand::Waveform {
            range,
            bucket_frames,
            max_buckets,
            json,
        } => {
            let result = cube_core::media::envelope::from_file(
                &range.path,
                &range.selection(),
                bucket_frames,
                max_buckets,
            )?;
            if json {
                writeln!(output, "{}", serde_json::to_string_pretty(&result)?)?;
            } else {
                writeln!(
                    output,
                    "Frames: {}; sample rate: {} frames/sec; channels: {:?}; bucket frames: {}",
                    result.frames, result.sample_rate, result.channels, result.bucket_frames
                )?;
                for bucket in result.buckets {
                    writeln!(
                        output,
                        "start_frame={} frames={} min={:?} max={:?}",
                        bucket.start_frame, bucket.frames, bucket.minimum, bucket.maximum
                    )?;
                }
            }
        }
        AudioCommand::Samples { range, destination } => {
            let samples = cube_core::media::pcm::extract_file(&range.path, &range.selection())?;
            if destination.stdout {
                output.write_all(samples.raw())?;
            } else {
                let mut info = serde_json::json!(samples.info());
                if destination.json {
                    info["values"] = serde_json::json!(samples.bounded_values()?);
                    info["data_hex"] =
                        serde_json::json!(cube_core::encoding::hex_encode(samples.raw()));
                }
                if let Some(path) = destination.output {
                    samples.save(&path)?;
                }
                writeln!(output, "{}", serde_json::to_string_pretty(&info)?)?;
            }
        }
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
