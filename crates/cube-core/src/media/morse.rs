//! Tone intervals and separately requested Morse timing interpretation.
use super::{
    pcm::Samples,
    spectrum::{invalid, limit},
    tone,
};
use crate::{
    errors::OperationError,
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Tone,
    Silence,
    Uncertain,
}
impl State {
    pub fn parse(name: &str) -> Result<Self, OperationError> {
        match name {
            "tone" => Ok(Self::Tone),
            "silence" => Ok(Self::Silence),
            "uncertain" => Ok(Self::Uncertain),
            _ => Err(invalid(
                "audio.morse.transcribe",
                "interval state must be tone, silence or uncertain",
            )),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Interval {
    pub start_frame: u64,
    pub frames: u64,
    pub state: State,
}
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub frequency_hz: u32,
    pub window_ms: u32,
    pub min_level_dbfs: i32,
    pub purity_percent: u32,
}
impl Options {
    pub fn validate(self, rate: u32) -> Result<(), OperationError> {
        if rate < 8000
            || self.frequency_hz < 100
            || self.frequency_hz >= rate / 2
            || !(5..=50).contains(&self.window_ms)
            || !(-60..=0).contains(&self.min_level_dbfs)
            || !(50..=100).contains(&self.purity_percent)
        {
            return Err(invalid(
                "audio.morse.detect",
                "require rate >=8000, frequency in [100, Nyquist), window 5..50 ms, level -60..0 dBFS and purity 50..100 percent",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Detection {
    pub sample_rate: u32,
    pub channel: u16,
    pub frequency_hz: u32,
    pub start_frame: u64,
    pub frames: u64,
    pub window_frames: usize,
    pub min_level_dbfs: i32,
    pub purity_percent: u32,
    pub intervals: Vec<Interval>,
}
/// Detect intervals only at an explicitly supplied frequency, using shared bounded
/// PCM/Goertzel evidence. No search, transcription or automatic speed inference.
/// RMS below threshold is silence; sufficient nominal-tone energy is tone; other
/// audible/partial windows are uncertain. Maximal contiguous states are merged.
/// <=262144 frames/4096 windows; timings are quantized to one analysis window.
pub fn detect(samples: &Samples, options: Options) -> Result<Detection, OperationError> {
    const OP: &str = "audio.morse.detect";
    let rate = samples.format().sample_rate;
    options.validate(rate)?;
    let window = (u64::from(rate) * u64::from(options.window_ms)).div_ceil(1000) as usize;
    if (samples.frames() as usize).div_ceil(window) > tone::MAX_WINDOWS {
        return Err(limit(OP, "tone analysis exceeds 4096 windows"));
    }
    let values = tone::selected(samples, OP)?;
    let threshold = 10f64.powf(f64::from(options.min_level_dbfs) / 20.0);
    let mut intervals: Vec<Interval> = Vec::new();
    for (i, part) in values.chunks(window).enumerate() {
        let rms = tone::rms(part);
        let amplitude = tone::amplitude(part, rate, f64::from(options.frequency_hz));
        let purity = amplitude * amplitude / (2.0 * (rms * rms).max(1e-24));
        let state = if rms < threshold {
            State::Silence
        } else if part.len() == window && purity >= f64::from(options.purity_percent) / 100.0 {
            State::Tone
        } else {
            State::Uncertain
        };
        if let Some(last) = intervals.last_mut().filter(|last| last.state == state) {
            last.frames += part.len() as u64;
        } else {
            intervals.push(Interval {
                start_frame: samples.start_frame() + (i * window) as u64,
                frames: part.len() as u64,
                state,
            });
        }
    }
    Ok(Detection {
        sample_rate: rate,
        channel: samples.channels()[0],
        frequency_hz: options.frequency_hz,
        start_frame: samples.start_frame(),
        frames: samples.frames(),
        window_frames: window,
        min_level_dbfs: options.min_level_dbfs,
        purity_percent: options.purity_percent,
        intervals,
    })
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Transcription {
    pub status: &'static str,
    pub unit_ms: u32,
    pub tolerance_percent: u32,
    pub classes: Vec<Option<&'static str>>,
    pub uncertain_segments: Vec<usize>,
    pub tokens: Option<String>,
    pub text: Option<String>,
}
/// Interpret explicit maximal contiguous intervals at a caller-selected dot unit
/// (40..1000 ms) and relative duration tolerance (0..40%). Tone lengths are 1/3
/// units, internal gaps 1/3/7 units; boundary silence is ignored. Multiple matching
/// classes or any uncertain/invalid-duration segment prevents text generation.
/// Valid tokens are passed to the existing Morse text decoder; unsupported token
/// sequences retain tokens without invented letters. <=4096 records/262144 frames;
/// invalid/noncontiguous/overlapping/overflowing ranges fail before interpretation.
pub fn transcribe(
    intervals: &[Interval],
    sample_rate: u32,
    unit_ms: u32,
    tolerance_percent: u32,
) -> Result<Transcription, OperationError> {
    const OP: &str = "audio.morse.transcribe";
    if sample_rate == 0
        || sample_rate > super::wave::MAX_SAMPLE_RATE
        || !(40..=1000).contains(&unit_ms)
        || tolerance_percent > 40
    {
        return Err(invalid(
            OP,
            "invalid sample_rate, dot unit (40..1000 ms) or tolerance (0..40%)",
        ));
    }
    if intervals.len() > tone::MAX_WINDOWS {
        return Err(limit(OP, "transcription exceeds 4096 intervals"));
    }
    let mut total = 0u64;
    let mut end = None;
    let mut previous = None;
    for interval in intervals {
        let next = interval
            .start_frame
            .checked_add(interval.frames)
            .ok_or_else(|| invalid(OP, "interval range overflows"))?;
        total = total
            .checked_add(interval.frames)
            .ok_or_else(|| limit(OP, "interval frames overflow"))?;
        if interval.frames == 0
            || end.is_some_and(|end| interval.start_frame != end)
            || previous == Some(interval.state)
        {
            return Err(invalid(
                OP,
                "intervals must be nonempty, maximal and contiguous with no gaps/overlap",
            ));
        }
        if total > tone::MAX_FRAMES as u64 {
            return Err(limit(OP, "transcription exceeds 262144 frames"));
        }
        end = Some(next);
        previous = Some(interval.state);
    }
    let unit = f64::from(sample_rate) * f64::from(unit_ms) / 1000.0;
    let tolerance = f64::from(tolerance_percent) / 100.0;
    let mut classes = Vec::with_capacity(intervals.len());
    let mut uncertain_segments = Vec::new();
    for (i, interval) in intervals.iter().enumerate() {
        let candidates: &[(f64, &str)] = match interval.state {
            State::Tone => &[(1.0, "dot"), (3.0, "dash")],
            State::Silence => &[(1.0, "element_gap"), (3.0, "letter_gap"), (7.0, "word_gap")],
            State::Uncertain => &[],
        };
        let class = if interval.state == State::Silence && (i == 0 || i + 1 == intervals.len()) {
            Some("boundary_silence")
        } else {
            let matches: Vec<_> = candidates
                .iter()
                .filter(|(n, _)| {
                    (interval.frames as f64 - unit * n).abs() <= unit * n * tolerance + 1e-9
                })
                .map(|(_, name)| *name)
                .collect();
            if matches.len() == 1 {
                Some(matches[0])
            } else {
                None
            }
        };
        if class.is_none() {
            uncertain_segments.push(i)
        }
        classes.push(class);
    }
    let mut result = Transcription {
        status: if intervals.is_empty() {
            "no_result"
        } else {
            "uncertain"
        },
        unit_ms,
        tolerance_percent,
        classes,
        uncertain_segments,
        tokens: None,
        text: None,
    };
    if !result.uncertain_segments.is_empty() || intervals.is_empty() {
        return Ok(result);
    }
    let mut tokens = Vec::new();
    let mut current = String::new();
    for class in result.classes.iter().flatten() {
        match *class {
            "dot" => current.push('.'),
            "dash" => current.push('-'),
            "letter_gap" | "word_gap" => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
                if *class == "word_gap" && !tokens.is_empty() {
                    tokens.push("/".into());
                }
            }
            _ => {}
        }
    }
    if !current.is_empty() {
        tokens.push(current)
    }
    let tokens = tokens.join(" ");
    if tokens.is_empty() {
        result.status = "no_result";
        return Ok(result);
    }
    match crate::symbols::morse_decode(&tokens) {
        Ok(text) => {
            result.text = Some(text);
            result.status = "decoded"
        }
        Err(_) => result.status = "unsupported_tokens",
    }
    result.tokens = Some(tokens);
    Ok(result)
}
pub const MAX_INTERVALS: usize = tone::MAX_WINDOWS;
const fn p(
    name: &'static str,
    description: &'static str,
    min: i64,
    max: i64,
    default: Option<i64>,
) -> Parameter {
    Parameter {
        name,
        description,
        required: default.is_none(),
        default: match default {
            Some(n) => Some(ParameterValue::Integer(n)),
            None => None,
        },
        constraints: ParameterType::Integer { min, max },
    }
}
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "audio.morse.detect",
        category: "audio",
        description: "Detect bounded tone/silence/uncertain intervals at an explicit nominal frequency; quantized timings, no automatic frequency/speed search or transcription.",
        input_type: "audio.pcm_samples",
        output_type: "audio.tone_intervals",
        parameters: &[
            p(
                "frequency_hz",
                "Caller-supplied expected frequency; must be below actual Nyquist.",
                100,
                191999,
                None,
            ),
            p(
                "window_ms",
                "Nonoverlapping rectangular windows; timing resolution.",
                5,
                50,
                Some(10),
            ),
            p(
                "min_level_dbfs",
                "RMS full-scale threshold; not calibrated SPL.",
                -60,
                0,
                Some(-40),
            ),
            p(
                "purity_percent",
                "Required nominal tone energy ratio.",
                50,
                100,
                Some(80),
            ),
        ],
        deterministic: true,
        reversible: Some(false),
        command: "audio morse detect",
        example: "cube audio morse detect sample.wav --channels 0 --frequency-hz 700 --json",
    },
    Descriptor {
        id: "audio.morse.transcribe",
        category: "audio",
        description: "Separately interpret bounded explicit intervals using caller dot-unit/tolerance, 1/3 tone and 1/3/7 gap timing; reuse existing Morse decoder, retain uncertainty.",
        input_type: "audio.tone_intervals",
        output_type: "audio.morse_transcription",
        parameters: &[
            p(
                "sample_rate",
                "Time base of interval frame coordinates.",
                1,
                384000,
                None,
            ),
            p(
                "unit_ms",
                "Caller-supplied dot duration; no automatic speed estimate.",
                40,
                1000,
                None,
            ),
            p(
                "tolerance_percent",
                "Relative duration tolerance; ambiguous matches remain uncertain.",
                0,
                40,
                Some(20),
            ),
        ],
        deterministic: true,
        reversible: Some(false),
        command: "audio morse transcribe",
        example: "cube audio morse transcribe --unit-ms 100 < intervals.json",
    },
];
