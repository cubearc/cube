//! Typed named-port adapter over public Core APIs; no algorithm implementation.
use super::{field, images::bytes};
use cube_core::{
    errors::{ErrorCode, OperationError},
    media::{bits, dtmf, envelope, metadata, morse, pcm, reverse, spectrogram, spectrum, wave},
    operations::contracts::values,
};
use serde_json::{Value, json};
use std::io::Cursor;

enum Adapter {
    Bits,
    Dtmf,
    Metadata,
    Detect,
    Transcribe,
    Pcm,
    Reverse,
    Spectrogram,
    Spectrum,
    Wave,
    Envelope,
}
fn adapter(op: &str) -> Option<Adapter> {
    Some(match op {
        "audio.bits.extract" => Adapter::Bits,
        "audio.dtmf.detect" => Adapter::Dtmf,
        "audio.metadata.inspect" => Adapter::Metadata,
        "audio.morse.detect" => Adapter::Detect,
        "audio.morse.transcribe" => Adapter::Transcribe,
        "audio.pcm.extract" => Adapter::Pcm,
        "audio.reverse" => Adapter::Reverse,
        "audio.spectrogram" => Adapter::Spectrogram,
        "audio.spectrum" => Adapter::Spectrum,
        "audio.wave.inspect" => Adapter::Wave,
        "audio.waveform.envelope" => Adapter::Envelope,
        _ => return None,
    })
}
pub(super) fn supports(op: &str) -> bool {
    adapter(op).is_some()
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "use exact logical PCM fields and audio parameters",
    )
}
fn text<'a>(p: &'a Value, k: &str) -> Result<&'a str, OperationError> {
    p[k].as_str().ok_or_else(invalid)
}
fn number(p: &Value, k: &str) -> Result<u64, OperationError> {
    p[k].as_u64()
        .or_else(|| p[k].as_str()?.parse().ok())
        .ok_or_else(invalid)
}
fn samples(value: &Value) -> Result<pcm::Samples, OperationError> {
    let f = &value["format"];
    pcm::Samples::from_raw(
        wave::Format {
            format_tag: field(f, "format_tag")?,
            channels: field(f, "channels")?,
            sample_rate: field(f, "sample_rate")?,
            bits_per_sample: field(f, "bits_per_sample")?,
            block_align: field(f, "block_align")?,
            byte_rate: field(f, "byte_rate")?,
        },
        field(value, "channels")?,
        number(value, "start_frame")?,
        number(value, "frames")?,
        super::bytes(&value["data"])?,
    )
}
fn selection(p: &Value) -> Result<pcm::Selection, OperationError> {
    Ok(pcm::Selection {
        start_frame: number(p, "start_frame")?,
        frames: p.get("frames").map(|_| number(p, "frames")).transpose()?,
        channels: p
            .get("channels")
            .map(|_| field(p, "channels"))
            .transpose()?,
    })
}
pub(super) fn execute(op: &str, inputs: Value, p: Value) -> Result<Value, OperationError> {
    let identity = cube_core::operations::require(op)?.id;
    let selected = adapter(op).ok_or_else(invalid)?;
    let result = (|| {
        if matches!(
            selected,
            Adapter::Wave | Adapter::Pcm | Adapter::Metadata | Adapter::Envelope
        ) {
            let mut reader = Cursor::new(super::bytes(&inputs["input"])?);
            return Ok(match selected {
                Adapter::Wave => json!({"output":wave::inspect(&mut reader)?}),
                Adapter::Metadata => json!({"output":metadata::inspect(&mut reader)?}),
                Adapter::Pcm => {
                    json!({"output":values::Pcm::from(&pcm::extract(&mut reader,&selection(&p)?)?)})
                }
                _ => {
                    json!({"output":envelope::from_reader(&mut reader,&selection(&p)?,number(&p,"bucket_frames")?,field(&p,"max_buckets")?)?})
                }
            });
        }
        if matches!(selected, Adapter::Transcribe) {
            let detection: morse::Detection = field(&inputs, "input")?;
            let rate = field(&p, "sample_rate")?;
            if rate != detection.sample_rate {
                return Err(invalid().parameter("sample_rate"));
            }
            return Ok(
                json!({"output":morse::transcribe(&detection.intervals,rate,field(&p,"unit_ms")?,field(&p,"tolerance_percent")?)?}),
            );
        }
        let source = samples(&inputs["input"])?;
        Ok(match selected {
            Adapter::Bits => {
                json!({"output":bytes(&bits::extract(&source,&bits::Options {bits:field(&p,"bits")?,reverse_frames:field(&p,"reverse_frames")?,bit_order:bits::BitOrder::parse(text(&p,"bit_order")?)?,incomplete:bits::Incomplete::parse(text(&p,"incomplete")?)?})?.data)})
            }
            Adapter::Reverse => json!({"output":bytes(&reverse::wave(&source)?)}),
            Adapter::Dtmf => {
                json!({"output":dtmf::detect(&source,dtmf::Options {window_ms:field(&p,"window_ms")?,min_tone_ms:field(&p,"min_tone_ms")?,min_gap_ms:field(&p,"min_gap_ms")?,min_level_dbfs:field(&p,"min_level_dbfs")?,dominance_db:field(&p,"dominance_db")?,max_twist_db:field(&p,"max_twist_db")?,purity_percent:field(&p,"purity_percent")?})?})
            }
            Adapter::Detect => {
                json!({"output":morse::detect(&source,morse::Options {frequency_hz:field(&p,"frequency_hz")?,window_ms:field(&p,"window_ms")?,min_level_dbfs:field(&p,"min_level_dbfs")?,purity_percent:field(&p,"purity_percent")?})?})
            }
            Adapter::Spectrum => {
                json!({"output":spectrum::analyze(&source,spectrum::Options {fft_size:field(&p,"fft_size")?,window:spectrum::Window::parse(text(&p,"window")?)?})?})
            }
            Adapter::Spectrogram => {
                json!({"output":spectrogram::analyze(&source,spectrogram::Options {fft_size:field(&p,"fft_size")?,hop_size:field(&p,"hop_size")?,window:spectrum::Window::parse(text(&p,"window")?)?,final_window:spectrogram::FinalWindow::parse(text(&p,"final_window")?)?})?})
            }
            _ => return Err(invalid()),
        })
    })();
    result.map_err(|e: OperationError| e.operation(identity))
}
