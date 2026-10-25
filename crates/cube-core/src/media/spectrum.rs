//! Bounded one-sided FFT peak amplitudes of exact selected integer PCM.
use super::pcm::Samples;
use crate::{
    errors::{ErrorCode, OperationError},
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use rustfft::{Fft, FftPlanner, num_complex::Complex};
use serde::Serialize;
use std::{f64::consts::TAU, sync::Arc};
pub const MAX_FFT: usize = 16384;
pub const MAX_CELLS: usize = 65536;
pub const MAX_WORK: usize = 4 * 1024 * 1024;
pub(super) fn invalid(op: &'static str, message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message).operation(op)
}
pub(super) fn limit(op: &'static str, message: &str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message).operation(op)
}
#[derive(Clone, Copy, Debug, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Window {
    Rectangular,
    #[default]
    Hann,
    Hamming,
}
impl Window {
    pub fn parse(name: &str) -> Result<Self, OperationError> {
        match name {
            "rectangular" => Ok(Self::Rectangular),
            "hann" => Ok(Self::Hann),
            "hamming" => Ok(Self::Hamming),
            _ => Err(invalid(
                "audio.spectrum",
                "window must be rectangular, hann or hamming",
            )
            .parameter("window")),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub fft_size: usize,
    pub window: Window,
}
impl Options {
    pub fn validate(self) -> Result<(), OperationError> {
        validate_size(self.fft_size, "audio.spectrum")
    }
}
impl Default for Options {
    fn default() -> Self {
        Self {
            fft_size: 1024,
            window: Window::Hann,
        }
    }
}
pub(super) fn validate_size(size: usize, op: &'static str) -> Result<(), OperationError> {
    if !(8..=MAX_FFT).contains(&size) || !size.is_power_of_two() {
        return Err(
            invalid(op, "fft_size must be a power of two in 8..=16384").parameter("fft_size")
        );
    }
    Ok(())
}
pub(super) fn check_shape(
    size: usize,
    windows: usize,
    channels: usize,
    op: &'static str,
) -> Result<(), OperationError> {
    validate_size(size, op)?;
    let cells = windows
        .checked_mul(channels)
        .and_then(|n| n.checked_mul(size / 2 + 1));
    let work = windows
        .checked_mul(channels)
        .and_then(|n| n.checked_mul(size))
        .and_then(|n| n.checked_mul(size.ilog2() as usize));
    if cells.is_none_or(|n| n > MAX_CELLS) || work.is_none_or(|n| n > MAX_WORK) {
        return Err(limit(
            op,
            "analysis exceeds 65536 magnitude cells or 4 Mi FFT work units",
        ));
    }
    Ok(())
}
pub(super) fn channel(samples: &Samples, index: usize) -> Vec<f64> {
    let scale = (1u64 << (samples.format().bits_per_sample - 1)) as f64;
    samples
        .iter_values()
        .skip(index)
        .step_by(samples.channels().len())
        .map(|v| f64::from(v) / scale)
        .collect()
}
/// Shared bounded FFT plan/window/buffers; spectrograms reuse one plan per call.
pub(super) struct Transform {
    fft: Arc<dyn Fft<f64>>,
    weights: Vec<f64>,
    sum: f64,
    buffer: Vec<Complex<f64>>,
    scratch: Vec<Complex<f64>>,
}
impl Transform {
    pub fn new(options: Options) -> Result<Self, OperationError> {
        validate_size(options.fft_size, "audio.spectrum")?;
        let n = options.fft_size;
        let weights: Vec<f64> = (0..n)
            .map(|i| match options.window {
                Window::Rectangular => 1.0,
                Window::Hann => 0.5 - 0.5 * (TAU * i as f64 / n as f64).cos(),
                Window::Hamming => 0.54 - 0.46 * (TAU * i as f64 / n as f64).cos(),
            })
            .collect();
        let sum = weights.iter().sum();
        let fft = FftPlanner::<f64>::new().plan_fft_forward(n);
        let scratch_len = fft.get_inplace_scratch_len();
        if scratch_len > 2 * n {
            return Err(limit(
                "audio.spectrum",
                "FFT scratch exceeds twice the FFT size",
            ));
        }
        Ok(Self {
            fft,
            weights,
            sum,
            buffer: vec![Complex::new(0.0, 0.0); n],
            scratch: vec![Complex::new(0.0, 0.0); scratch_len],
        })
    }
    pub fn amplitudes(&mut self, values: &[f64]) -> Vec<f64> {
        for (i, value) in self.buffer.iter_mut().enumerate() {
            *value = Complex::new(values.get(i).copied().unwrap_or(0.0) * self.weights[i], 0.0);
        }
        self.fft
            .process_with_scratch(&mut self.buffer, &mut self.scratch);
        let last = self.buffer.len() / 2;
        self.buffer[..=last]
            .iter()
            .enumerate()
            .map(|(i, c)| c.norm() / self.sum * if i == 0 || i == last { 1.0 } else { 2.0 })
            .collect()
    }
}
#[derive(Debug, Serialize)]
pub struct Spectrum {
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    pub channels: Vec<u16>,
    pub start_frame: u64,
    pub frames: u64,
    pub fft_size: usize,
    pub window: Window,
    pub window_definition: &'static str,
    pub amplitude_units: &'static str,
    pub frequencies_hz: Vec<f64>,
    pub amplitudes: Vec<Vec<f64>>,
}
/// Analyze exactly fft_size selected frames; no implicit padding/truncation.
/// Frequencies are k*sample_rate/N, k=0..N/2. Samples are divided by 2^(bits-1)
/// after the existing unsigned-8 centering/signed PCM decode. Periodic Hann/Hamming
/// use cos(2*pi*i/N). One-sided magnitudes divide by window sum and double interior
/// bins; DC/Nyquist are not doubled. Results are peak full-scale amplitudes, not
/// power/PSD or dB. Floating results agree within numerical tolerance, not bitwise
/// across architectures. Limits: 8..16384 power-of-two FFT, 65536 cells, 4 Mi Nlog2N
/// channel-work units. Silence yields zero; empty/short/long selections fail.
pub fn analyze(samples: &Samples, options: Options) -> Result<Spectrum, OperationError> {
    const OP: &str = "audio.spectrum";
    check_shape(options.fft_size, 1, samples.channels().len(), OP)?;
    if samples.frames() != options.fft_size as u64 {
        return Err(invalid(
            OP,
            "spectrum requires exactly fft_size frames; select the intended range explicitly",
        )
        .parameter("frames"));
    }
    let mut transform = Transform::new(options)?;
    let amplitudes = (0..samples.channels().len())
        .map(|i| transform.amplitudes(&channel(samples, i)))
        .collect();
    Ok(Spectrum {
        sample_rate: samples.format().sample_rate,
        bits_per_sample: samples.format().bits_per_sample,
        channels: samples.channels().to_vec(),
        start_frame: samples.start_frame(),
        frames: samples.frames(),
        fft_size: options.fft_size,
        window: options.window,
        window_definition: "periodic",
        amplitude_units: "one_sided_peak_full_scale",
        frequencies_hz: (0..=options.fft_size / 2)
            .map(|k| k as f64 * f64::from(samples.format().sample_rate) / options.fft_size as f64)
            .collect(),
        amplitudes,
    })
}
pub(super) const PARAMETERS: &[Parameter] = &[
    Parameter {
        name: "fft_size",
        description: "Power of two, exact selected frame count; no implicit padding/truncation.",
        required: false,
        default: Some(ParameterValue::Integer(1024)),
        constraints: ParameterType::Unsigned {
            min: 8,
            max: MAX_FFT as u64,
        },
    },
    Parameter {
        name: "window",
        description: "Periodic window; coherent-gain normalized one-sided peak amplitude.",
        required: false,
        default: Some(ParameterValue::Text("hann")),
        constraints: ParameterType::Text {
            choices: &["rectangular", "hann", "hamming"],
        },
    },
];
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "audio.spectrum",
    category: "audio",
    description: "Bounded one-sided FFT of selected integer PCM; actual sample-rate frequencies, periodic windows, peak full-scale amplitudes, <=65536 cells/4 Mi work.",
    input_type: "audio.pcm_samples",
    output_type: "audio.spectrum",
    parameters: PARAMETERS,
    deterministic: true,
    reversible: Some(false),
    command: "audio spectrum",
    example: "cube audio spectrum sample.wav --channels 0 --fft-size 1024 --json",
}];
