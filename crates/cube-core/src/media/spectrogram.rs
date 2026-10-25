//! Bounded windowed spectra; reuse the spectrum transform and exact PCM decoder.
use super::{
    pcm::Samples,
    spectrum::{self, Transform, Window},
};
use crate::{
    errors::OperationError,
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::Serialize;
#[derive(Clone, Copy, Debug, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FinalWindow {
    #[default]
    Drop,
    Pad,
}
impl FinalWindow {
    pub fn parse(name: &str) -> Result<Self, OperationError> {
        match name {
            "drop" => Ok(Self::Drop),
            "pad" => Ok(Self::Pad),
            _ => Err(
                spectrum::invalid("audio.spectrogram", "final_window must be drop or pad")
                    .parameter("final_window"),
            ),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub fft_size: usize,
    pub hop_size: usize,
    pub window: Window,
    pub final_window: FinalWindow,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            fft_size: 1024,
            hop_size: 512,
            window: Window::Hann,
            final_window: FinalWindow::Drop,
        }
    }
}
impl Options {
    pub fn validate(self) -> Result<(), OperationError> {
        spectrum::validate_size(self.fft_size, "audio.spectrogram")?;
        if self.hop_size == 0 || self.hop_size > self.fft_size {
            return Err(
                spectrum::invalid("audio.spectrogram", "hop_size must be in 1..=fft_size")
                    .parameter("hop_size"),
            );
        }
        Ok(())
    }
}
#[derive(Debug, Serialize)]
pub struct Spectrogram {
    pub sample_rate: u32,
    pub channels: Vec<u16>,
    pub start_frame: u64,
    pub frames: u64,
    pub fft_size: usize,
    pub hop_size: usize,
    pub window: Window,
    pub final_window: FinalWindow,
    pub amplitude_units: &'static str,
    pub time_origin: &'static str,
    pub frame_starts: Vec<u64>,
    pub times_seconds: Vec<f64>,
    pub valid_frames: Vec<usize>,
    pub dropped_frames: usize,
    pub frequencies_hz: Vec<f64>,
    pub amplitudes: Vec<Vec<Vec<f64>>>,
}
/// Channel-major [channel][window][bin] one-sided peak amplitudes, identical to
/// spectrum scaling. Window times are absolute source frame starts/sample_rate,
/// not center times. Drop retains only full windows; pad visits each hop start
/// before end and zero-fills its tail, using full-window normalization. Empty
/// input gives no windows with either policy; short drop gives no windows.
/// Preflight checks cell/work products before planning FFT or creating a matrix.
/// At most 65536 total magnitude cells and 4 Mi channel-window Nlog2N work units.
pub fn analyze(samples: &Samples, options: Options) -> Result<Spectrogram, OperationError> {
    const OP: &str = "audio.spectrogram";
    options.validate()?;
    let frames = samples.frames() as usize;
    let windows = match options.final_window {
        FinalWindow::Drop => {
            if frames < options.fft_size {
                0
            } else {
                1 + (frames - options.fft_size) / options.hop_size
            }
        }
        FinalWindow::Pad => frames.div_ceil(options.hop_size),
    };
    spectrum::check_shape(options.fft_size, windows, samples.channels().len(), OP)?;
    let starts: Vec<usize> = (0..windows).map(|w| w * options.hop_size).collect();
    let valid_frames = starts
        .iter()
        .map(|start| (frames - start).min(options.fft_size))
        .collect();
    let dropped_frames =
        frames.saturating_sub(starts.last().map(|s| s + options.fft_size).unwrap_or(0));
    let mut amplitudes = Vec::with_capacity(samples.channels().len());
    if windows == 0 {
        amplitudes.resize(samples.channels().len(), Vec::new());
    } else {
        let mut transform = Transform::new(spectrum::Options {
            fft_size: options.fft_size,
            window: options.window,
        })?;
        for c in 0..samples.channels().len() {
            let values = spectrum::channel(samples, c);
            amplitudes.push(
                starts
                    .iter()
                    .map(|start| {
                        transform
                            .amplitudes(&values[*start..(*start + options.fft_size).min(frames)])
                    })
                    .collect(),
            );
        }
    }
    let frame_starts: Vec<u64> = starts
        .iter()
        .map(|s| samples.start_frame() + *s as u64)
        .collect();
    let times_seconds = frame_starts
        .iter()
        .map(|f| *f as f64 / f64::from(samples.format().sample_rate))
        .collect();
    Ok(Spectrogram {
        sample_rate: samples.format().sample_rate,
        channels: samples.channels().to_vec(),
        start_frame: samples.start_frame(),
        frames: samples.frames(),
        fft_size: options.fft_size,
        hop_size: options.hop_size,
        window: options.window,
        final_window: options.final_window,
        amplitude_units: "one_sided_peak_full_scale",
        time_origin: "absolute_window_start",
        frame_starts,
        times_seconds,
        valid_frames,
        dropped_frames,
        frequencies_hz: (0..=options.fft_size / 2)
            .map(|k| k as f64 * f64::from(samples.format().sample_rate) / options.fft_size as f64)
            .collect(),
        amplitudes,
    })
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "audio.spectrogram",
    category: "audio",
    description: "Bounded channel/window/bin numeric spectra using shared FFT, absolute window-start axes, explicit drop/pad tails and peak full-scale amplitudes; <=65536 cells/4 Mi work.",
    input_type: "audio.pcm_samples",
    output_type: "audio.spectrogram",
    parameters: &[
        Parameter {
            description: "Power-of-two FFT window length; independent of total selected frames.",
            ..spectrum::PARAMETERS[0]
        },
        spectrum::PARAMETERS[1],
        Parameter {
            name: "hop_size",
            description: "Frame distance between window starts, 1..=fft_size.",
            required: false,
            default: Some(ParameterValue::Integer(512)),
            constraints: ParameterType::Unsigned {
                min: 1,
                max: spectrum::MAX_FFT as u64,
            },
        },
        Parameter {
            name: "final_window",
            description: "drop keeps full windows; pad zero-fills tails at each hop start before end. Empty input has no windows.",
            required: false,
            default: Some(ParameterValue::Text("drop")),
            constraints: ParameterType::Text {
                choices: &["drop", "pad"],
            },
        },
    ],
    deterministic: true,
    reversible: Some(false),
    command: "audio spectrogram",
    example: "cube audio spectrogram sample.wav --channels 0 --fft-size 1024 --hop-size 512 --frames 4096 --json",
}];
