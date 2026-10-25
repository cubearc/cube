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
    /// Structured values/extraction metadata and exact hex; operation-specific limits
    #[arg(long)]
    json: bool,
}
#[derive(Subcommand)]
pub(crate) enum AudioCommand {
    /// Timed nominal DTMF candidates with evidence and uncertainty; no forced string
    Dtmf {
        #[command(flatten)]
        range: RangeArgs,
        #[arg(long, default_value_t = 40)]
        window_ms: u32,
        #[arg(long, default_value_t = 80)]
        min_tone_ms: u32,
        #[arg(long, default_value_t = 40)]
        min_gap_ms: u32,
        #[arg(long,default_value_t=-40,allow_negative_numbers=true)]
        min_level_dbfs: i32,
        #[arg(long, default_value_t = 6)]
        dominance_db: u32,
        #[arg(long, default_value_t = 8)]
        max_twist_db: u32,
        #[arg(long, default_value_t = 75)]
        purity_percent: u32,
        #[arg(long)]
        json: bool,
    },
    /// Reverse PCM frames into a new minimal WAV; preserve channel groups and native bytes
    Reverse {
        #[command(flatten)]
        range: RangeArgs,
        /// New destination; existing files and source aliases are rejected
        #[arg(long)]
        output: PathBuf,
    },
    /// Extract exact native PCM bits; no amplitude normalization or text guess
    Bits {
        #[command(flatten)]
        range: RangeArgs,
        #[arg(long, value_delimiter = ',', default_value = "0")]
        bits: Vec<u8>,
        #[arg(long)]
        reverse_frames: bool,
        #[arg(long, default_value = "msb")]
        bit_order: String,
        #[arg(long, default_value = "reject")]
        incomplete: String,
        #[command(flatten)]
        destination: SampleOutput,
    },
    /// Bounded channel/window/bin spectra; absolute times and explicit final-window policy
    Spectrogram {
        #[command(flatten)]
        range: RangeArgs,
        #[arg(long, default_value_t = 1024)]
        fft_size: usize,
        #[arg(long, default_value_t = 512)]
        hop_size: usize,
        #[arg(long, default_value = "hann")]
        window: String,
        #[arg(long, default_value = "drop")]
        final_window: String,
        #[arg(long)]
        json: bool,
    },
    /// Numerical one-sided FFT; exact N frames, periodic window, no plot
    Spectrum {
        #[command(flatten)]
        range: RangeArgs,
        #[arg(long, default_value_t = 1024)]
        fft_size: usize,
        #[arg(long, default_value = "hann")]
        window: String,
        #[arg(long)]
        json: bool,
    },
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
        AudioCommand::Dtmf {
            range,
            window_ms,
            min_tone_ms,
            min_gap_ms,
            min_level_dbfs,
            dominance_db,
            max_twist_db,
            purity_percent,
            json,
        } => {
            use cube_core::media::{
                dtmf::{self, Options},
                pcm,
            };
            let opts = Options {
                window_ms,
                min_tone_ms,
                min_gap_ms,
                min_level_dbfs,
                dominance_db,
                max_twist_db,
                purity_percent,
            };
            opts.validate()?;
            let samples = pcm::extract_file(&range.path, &range.selection())?;
            let result = dtmf::detect(&samples, opts)?;
            if json {
                writeln!(output, "{}", serde_json::to_string_pretty(&result)?)?;
            } else {
                writeln!(
                    output,
                    "{} windows; {} candidate/uncertain runs; timing tolerance {} frames",
                    result.windows.len(),
                    result.events.len(),
                    result.timing_tolerance_frames
                )?;
                for event in result.events {
                    writeln!(
                        output,
                        "{}..{} {:?} {} ({})",
                        event.start_frame,
                        event.start_frame + event.frames,
                        event.symbol,
                        event.status,
                        event.reason
                    )?;
                }
            }
        }
        AudioCommand::Reverse {
            range,
            output: destination,
        } => {
            let samples = cube_core::media::pcm::extract_file(&range.path, &range.selection())?;
            let bytes = cube_core::media::reverse::wave(&samples)?;
            crate::pipelinecmd::write_new(&destination, &bytes)?;
            writeln!(
                output,
                "Wrote {} reversed PCM frames to a new WAV (fmt/data only)",
                samples.frames()
            )?;
        }
        AudioCommand::Bits {
            range,
            bits,
            reverse_frames,
            bit_order,
            incomplete,
            destination,
        } => {
            use cube_core::media::{
                bits::{self, BitOrder, Incomplete, Options},
                pcm,
            };
            let options = Options {
                bits,
                reverse_frames,
                bit_order: BitOrder::parse(&bit_order)?,
                incomplete: Incomplete::parse(&incomplete)?,
            };
            let samples = pcm::extract_file(&range.path, &range.selection())?;
            let result = bits::extract(&samples, &options)?;
            if destination.stdout {
                output.write_all(&result.data)?;
            } else {
                let mut info = serde_json::json!(result);
                if destination.json {
                    info["data_hex"] =
                        serde_json::json!(cube_core::encoding::hex_encode(&result.data));
                }
                if let Some(path) = destination.output {
                    crate::pipelinecmd::write_new(&path, &result.data)?;
                }
                writeln!(output, "{}", serde_json::to_string_pretty(&info)?)?;
            }
        }
        AudioCommand::Spectrogram {
            range,
            fft_size,
            hop_size,
            window,
            final_window,
            json,
        } => {
            use cube_core::media::{
                pcm,
                spectrogram::{self, FinalWindow, Options},
                spectrum::Window,
            };
            let options = Options {
                fft_size,
                hop_size,
                window: Window::parse(&window)?,
                final_window: FinalWindow::parse(&final_window)?,
            };
            options.validate()?;
            let samples = pcm::extract_file(&range.path, &range.selection())?;
            let result = spectrogram::analyze(&samples, options)?;
            if json {
                writeln!(output, "{}", serde_json::to_string_pretty(&result)?)?;
            } else {
                writeln!(
                    output,
                    "{} Hz; N={}; hop={}; channels={:?}; windows={}; bins={}; one-sided peak full-scale amplitudes",
                    result.sample_rate,
                    result.fft_size,
                    result.hop_size,
                    result.channels,
                    result.frame_starts.len(),
                    result.frequencies_hz.len()
                )?;
                for (i, f) in result.frame_starts.iter().enumerate() {
                    writeln!(
                        output,
                        "window {i}: start_frame={f} seconds={:.9} valid_frames={}",
                        result.times_seconds[i], result.valid_frames[i]
                    )?;
                }
            }
        }
        AudioCommand::Spectrum {
            mut range,
            fft_size,
            window,
            json,
        } => {
            use cube_core::media::{
                pcm,
                spectrum::{self, Options, Window},
            };
            range.frames = range.frames.or(Some(fft_size as u64));
            let options = Options {
                fft_size,
                window: Window::parse(&window)?,
            };
            options.validate()?;
            let samples = pcm::extract_file(&range.path, &range.selection())?;
            let result = spectrum::analyze(&samples, options)?;
            if json {
                writeln!(output, "{}", serde_json::to_string_pretty(&result)?)?;
            } else {
                writeln!(
                    output,
                    "{} Hz; N={}; {:?}; one-sided peak full-scale amplitudes",
                    result.sample_rate, result.fft_size, result.channels
                )?;
                for (k, hz) in result.frequencies_hz.iter().enumerate() {
                    writeln!(
                        output,
                        "{k} {hz:.6} {:?}",
                        result.amplitudes.iter().map(|c| c[k]).collect::<Vec<_>>()
                    )?;
                }
            }
        }
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
