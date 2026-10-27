//! Timed DTMF candidates with measured evidence, not a certified telephony receiver.
use super::{
    pcm::Samples,
    spectrum::{invalid, limit},
    tone,
};
use crate::{
    errors::OperationError,
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::Serialize;
const OP: &str = "audio.dtmf.detect";
const ROWS: [u16; 4] = [697, 770, 852, 941];
const COLUMNS: [u16; 4] = [1209, 1336, 1477, 1633];
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
pub struct Options {
    pub window_ms: u32,
    pub min_tone_ms: u32,
    pub min_gap_ms: u32,
    pub min_level_dbfs: i32,
    pub dominance_db: u32,
    pub max_twist_db: u32,
    pub purity_percent: u32,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            window_ms: 40,
            min_tone_ms: 80,
            min_gap_ms: 40,
            min_level_dbfs: -40,
            dominance_db: 6,
            max_twist_db: 8,
            purity_percent: 75,
        }
    }
}
impl Options {
    pub fn validate(self) -> Result<(), OperationError> {
        if !(10..=100).contains(&self.window_ms)
            || !(40..=1000).contains(&self.min_tone_ms)
            || self.min_gap_ms > 1000
            || !(-60..=0).contains(&self.min_level_dbfs)
            || !(3..=30).contains(&self.dominance_db)
            || self.max_twist_db > 12
            || !(50..=100).contains(&self.purity_percent)
        {
            return Err(invalid(
                OP,
                "invalid DTMF window, duration, level, dominance, twist or purity threshold",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct WindowEvidence {
    pub start_frame: u64,
    pub frames: usize,
    pub status: &'static str,
    pub reason: &'static str,
    pub symbol: Option<char>,
    pub rms_full_scale: f64,
    pub row_hz: u16,
    pub column_hz: u16,
    pub row_amplitude: f64,
    pub column_amplitude: f64,
    pub row_dominance_db: f64,
    pub column_dominance_db: f64,
    pub twist_db: f64,
    pub pair_energy_ratio: f64,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Event {
    pub start_frame: u64,
    pub frames: u64,
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub status: &'static str,
    pub reason: &'static str,
    pub candidate_symbol: char,
    pub symbol: Option<char>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Detection {
    pub sample_rate: u32,
    pub channel: u16,
    pub start_frame: u64,
    pub frames: u64,
    pub options: Options,
    pub method: &'static str,
    pub timing_tolerance_frames: usize,
    pub windows: Vec<WindowEvidence>,
    pub events: Vec<Event>,
}
fn strongest(amplitudes: &[f64; 4]) -> (usize, f64) {
    let mut order = [0usize, 1, 2, 3];
    order.sort_by(|a, b| amplitudes[*b].total_cmp(&amplitudes[*a]));
    let best = amplitudes[order[0]];
    let ratio = (best.max(1e-12) / amplitudes[order[1]].max(1e-12)).max(1.0);
    (order[0], 20.0 * ratio.log10())
}
/// Evaluate the Q.23 nominal row/column frequencies using bounded Goertzel
/// measurements. Exactly one selected channel, actual sample rate >=8000 Hz,
/// <=262144 frames and <=4096 nonoverlapping windows. Defaults are conservative
/// analysis thresholds, not Q.24 certification. Rectangular fixed windows only;
/// offset frequencies, speech, distortion or boundary mixtures may be uncertain.
/// Timings are window boundaries with one-window tolerance. A candidate requires
/// level, band dominance, pair-energy, twist, full-window and duration/gap tests.
/// All windows/short or insufficient-gap runs remain inspectable; no forced string.
pub fn detect(samples: &Samples, options: Options) -> Result<Detection, OperationError> {
    options.validate()?;
    let rate = samples.format().sample_rate;
    if rate < 8000 {
        return Err(invalid(
            OP,
            "DTMF requires a sample rate of at least 8000 Hz",
        ));
    }
    let window = (u64::from(rate) * u64::from(options.window_ms)).div_ceil(1000) as usize;
    let count = (samples.frames() as usize).div_ceil(window);
    if count > tone::MAX_WINDOWS {
        return Err(limit(OP, "DTMF exceeds 4096 windows"));
    }
    let values = tone::selected(samples, OP)?;
    let threshold = 10f64.powf(f64::from(options.min_level_dbfs) / 20.0);
    let mut windows = Vec::with_capacity(count);
    for (i, part) in values.chunks(window).enumerate() {
        let rms = tone::rms(part);
        let rows = ROWS.map(|hz| tone::amplitude(part, rate, f64::from(hz)));
        let cols = COLUMNS.map(|hz| tone::amplitude(part, rate, f64::from(hz)));
        let (r, rd) = strongest(&rows);
        let (c, cd) = strongest(&cols);
        let twist = 20.0 * (rows[r].max(1e-12) / cols[c].max(1e-12)).log10();
        let purity = (rows[r] * rows[r] + cols[c] * cols[c]) / (2.0 * (rms * rms).max(1e-24));
        let reason = if rms < threshold {
            "below_level"
        } else if part.len() != window {
            "partial_window"
        } else if rd < f64::from(options.dominance_db) || cd < f64::from(options.dominance_db) {
            "ambiguous_bands"
        } else if twist.abs() > f64::from(options.max_twist_db) {
            "twist"
        } else if purity < f64::from(options.purity_percent) / 100.0 {
            "insufficient_pair_energy"
        } else {
            "nominal_pair"
        };
        let symbol = if reason == "nominal_pair" {
            Some(char::from(b"123A456B789C*0#D"[r * 4 + c]))
        } else {
            None
        };
        let status = if symbol.is_some() {
            "candidate"
        } else if reason == "below_level" {
            "silence"
        } else {
            "uncertain"
        };
        windows.push(WindowEvidence {
            start_frame: samples.start_frame() + (i * window) as u64,
            frames: part.len(),
            status,
            reason,
            symbol,
            rms_full_scale: rms,
            row_hz: ROWS[r],
            column_hz: COLUMNS[c],
            row_amplitude: rows[r],
            column_amplitude: cols[c],
            row_dominance_db: rd,
            column_dominance_db: cd,
            twist_db: twist,
            pair_energy_ratio: purity,
        });
    }
    let min_tone = (u64::from(rate) * u64::from(options.min_tone_ms)).div_ceil(1000);
    let min_gap = (u64::from(rate) * u64::from(options.min_gap_ms)).div_ceil(1000);
    let mut events = Vec::new();
    let mut i = 0;
    let mut previous_end = None;
    while i < windows.len() {
        let Some(symbol) = windows[i].symbol else {
            i += 1;
            continue;
        };
        let start = windows[i].start_frame;
        let mut end = start + windows[i].frames as u64;
        i += 1;
        while i < windows.len() && windows[i].symbol == Some(symbol) {
            end = windows[i].start_frame + windows[i].frames as u64;
            i += 1;
        }
        let reason = if end - start < min_tone {
            "short_run"
        } else if previous_end.is_some_and(|p| start - p < min_gap) {
            "insufficient_gap"
        } else {
            "thresholds_met"
        };
        previous_end = Some(end);
        let accepted = reason == "thresholds_met";
        events.push(Event {
            start_frame: start,
            frames: end - start,
            start_seconds: start as f64 / f64::from(rate),
            end_seconds: end as f64 / f64::from(rate),
            status: if accepted { "candidate" } else { "uncertain" },
            reason,
            candidate_symbol: symbol,
            symbol: accepted.then_some(symbol),
        });
    }
    Ok(Detection {
        sample_rate: rate,
        channel: samples.channels()[0],
        start_frame: samples.start_frame(),
        frames: samples.frames(),
        options,
        method: "nominal_frequency_goertzel_candidates",
        timing_tolerance_frames: window,
        windows,
        events,
    })
}
const fn parameter(
    name: &'static str,
    description: &'static str,
    min: i64,
    max: i64,
    default: i64,
) -> Parameter {
    Parameter {
        name,
        description,
        required: false,
        default: Some(ParameterValue::Integer(default)),
        constraints: ParameterType::Integer { min, max },
    }
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: OP,
    category: "audio",
    description: "Bounded timed DTMF candidates with nominal-band, level, dominance, twist, purity and duration/gap evidence; uncertainty retained; not a certified receiver.",
    input_type: "audio.pcm_samples",
    output_type: "audio.dtmf_detection",
    parameters: &[
        parameter(
            "window_ms",
            "Nonoverlapping rectangular analysis windows; timing tolerance is one window.",
            10,
            100,
            40,
        ),
        parameter(
            "min_tone_ms",
            "Minimum candidate run duration.",
            40,
            1000,
            80,
        ),
        parameter(
            "min_gap_ms",
            "Minimum gap from the preceding candidate run.",
            0,
            1000,
            40,
        ),
        parameter(
            "min_level_dbfs",
            "Minimum RMS dB relative to full scale; not calibrated SPL.",
            -60,
            0,
            -40,
        ),
        parameter(
            "dominance_db",
            "Required row/column dominance over second strongest nominal frequency.",
            3,
            30,
            6,
        ),
        parameter(
            "max_twist_db",
            "Maximum absolute row/column amplitude imbalance in dB.",
            0,
            12,
            8,
        ),
        parameter(
            "purity_percent",
            "Minimum fraction of measured energy explained by the pair.",
            50,
            100,
            75,
        ),
    ],
    deterministic: true,
    reversible: Some(false),
    command: "audio dtmf",
    example: "cube audio dtmf sample.wav --channels 0 --frames 16000 --json",
}];
