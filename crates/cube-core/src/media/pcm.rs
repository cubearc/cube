//! Exact selected PCM samples with bounded, frame-aligned reads. No normalization.
use super::wave::{self, Format, Info, limit, read};
use crate::{
    errors::{ErrorCode, OperationError},
    forensics,
};
use serde::Serialize;
use std::{
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};
/// Processing limits are independent of metadata inspection's 1 GiB file cap.
pub const MAX_FRAMES: u64 = 1024 * 1024;
pub const MAX_DURATION_SECONDS: u64 = 300;
pub const MAX_PROCESS_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_PCM_BYTES: u64 = 8 * 1024 * 1024;
pub const MAX_SAMPLES: u64 = 2 * 1024 * 1024;
pub const MAX_VALUE_RECORDS: usize = 10000;
pub(super) fn invalid(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
}
/// Absolute half-open frame range [start_frame, start_frame+frames). None means
/// through the final source frame. Omitted channels preserve all source indexes.
#[derive(Clone, Debug, Default)]
pub struct Selection {
    pub start_frame: u64,
    pub frames: Option<u64>,
    pub channels: Option<Vec<u16>>,
}
/// Reusable, immutable selected PCM block; private fields preserve shape invariants.
#[derive(Clone, Debug)]
pub struct Samples {
    format: Format,
    channels: Vec<u16>,
    start_frame: u64,
    frames: u64,
    data: Vec<u8>,
}
/// Export metadata sufficient to interpret the raw interleaved sample bytes.
#[derive(Serialize)]
pub struct SampleInfo<'a> {
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    pub source_channels: u16,
    pub channels: &'a [u16],
    pub start_frame: u64,
    pub frames: u64,
    pub frame_bytes: usize,
    pub sample_count: u64,
    pub raw_bytes: usize,
    pub byte_order: &'static str,
    pub storage: &'static str,
    pub value_type: &'static str,
}
impl Samples {
    pub fn format(&self) -> Format {
        self.format
    }
    pub fn channels(&self) -> &[u16] {
        &self.channels
    }
    pub fn start_frame(&self) -> u64 {
        self.start_frame
    }
    pub fn frames(&self) -> u64 {
        self.frames
    }
    /// Exact row/frame-major interleaved bytes in requested channel order.
    pub fn raw(&self) -> &[u8] {
        &self.data
    }
    pub fn info(&self) -> SampleInfo<'_> {
        SampleInfo {
            sample_rate: self.format.sample_rate,
            bits_per_sample: self.format.bits_per_sample,
            source_channels: self.format.channels,
            channels: &self.channels,
            start_frame: self.start_frame,
            frames: self.frames,
            frame_bytes: self.channels.len() * usize::from(self.format.bits_per_sample / 8),
            sample_count: self.frames * self.channels.len() as u64,
            raw_bytes: self.data.len(),
            byte_order: "little",
            storage: if self.format.bits_per_sample == 8 {
                "unsigned_8"
            } else {
                "signed_twos_complement"
            },
            value_type: "signed_i32",
        }
    }
    /// Integer amplitude units, interleaved in exported channel order. 8-bit PCM
    /// subtracts 128; packed 16/24/32-bit little-endian values sign-extend to i32.
    /// This never rescales or clips; at most 2 Mi values for an extracted block.
    pub fn values(&self) -> Vec<i32> {
        self.iter_values().collect()
    }
    pub(super) fn iter_values(&self) -> impl Iterator<Item = i32> + '_ {
        self.data
            .chunks_exact(usize::from(self.format.bits_per_sample / 8))
            .map(|bytes| match bytes {
                [a] => i32::from(*a) - 128,
                [a, b] => i32::from(i16::from_le_bytes([*a, *b])),
                [a, b, c] => i32::from_le_bytes([*a, *b, *c, if c & 128 != 0 { 255 } else { 0 }]),
                [a, b, c, d] => i32::from_le_bytes([*a, *b, *c, *d]),
                _ => unreachable!("validated PCM width"),
            })
    }
    /// Bounded structured listings for CLI/SDK JSON; use raw bytes or a smaller
    /// selection for more than 10000 samples. No incomplete listing is returned.
    pub fn bounded_values(&self) -> Result<Vec<i32>, OperationError> {
        if self.info().sample_count > MAX_VALUE_RECORDS as u64 {
            return Err(limit(
                "typed listing exceeds 10000 samples; use raw bytes or a smaller frame range",
            )
            .operation("audio.pcm.extract"));
        }
        Ok(self.values())
    }
    /// Write exact raw PCM bytes exclusively. This is not a WAVE container;
    /// metadata must accompany the bytes. Existing paths/aliases are rejected.
    pub fn save(&self, path: &Path) -> Result<(), OperationError> {
        let result = (|| {
            let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
            let result = file.write_all(&self.data).and_then(|_| file.flush());
            drop(file);
            if result.is_err() {
                let _ = std::fs::remove_file(path);
            }
            result.map_err(OperationError::io)
        })();
        result.map_err(|e: OperationError| e.operation("audio.pcm.extract"))
    }
}
pub(super) struct Plan {
    format: Format,
    channels: Vec<u16>,
    start: u64,
    pub frames: u64,
    offset: u64,
    pub output_bytes: u64,
}
impl Plan {
    pub(super) fn resolve(
        info: &Info,
        selection: &Selection,
        output_budget: u64,
    ) -> Result<Self, OperationError> {
        let format = info.format;
        let frames = selection
            .frames
            .unwrap_or_else(|| info.frames.saturating_sub(selection.start_frame));
        let end = selection
            .start_frame
            .checked_add(frames)
            .ok_or_else(|| invalid("frame range overflows").parameter("frames"))?;
        if selection.start_frame > info.frames || end > info.frames {
            return Err(invalid("frame range exceeds available frames").parameter("frames"));
        }
        if frames > MAX_FRAMES || frames > MAX_DURATION_SECONDS * u64::from(format.sample_rate) {
            return Err(limit("selection exceeds 1 Mi frames or 300 seconds"));
        }
        let default: Vec<u16> = (0..format.channels).collect();
        let channels = selection.channels.as_deref().unwrap_or(&default);
        if channels.is_empty()
            || channels.len() > usize::from(format.channels)
            || channels
                .iter()
                .enumerate()
                .any(|(i, c)| *c >= format.channels || channels[..i].contains(c))
        {
            return Err(
                invalid("channels must be nonempty, unique, available zero-based indexes")
                    .parameter("channels"),
            );
        }
        let source_bytes = frames * u64::from(format.block_align);
        let output_bytes = frames * channels.len() as u64 * u64::from(format.bits_per_sample / 8);
        if source_bytes > MAX_PROCESS_BYTES || output_bytes > output_budget {
            return Err(limit(
                "selected source/output byte budget exceeded; choose a smaller frame range",
            ));
        }
        Ok(Self {
            format,
            channels: channels.to_vec(),
            start: selection.start_frame,
            frames,
            offset: info.data_offset + selection.start_frame * u64::from(format.block_align),
            output_bytes,
        })
    }
    pub(super) fn empty(&self) -> Samples {
        Samples {
            format: self.format,
            channels: self.channels.clone(),
            start_frame: self.start,
            frames: 0,
            data: Vec::new(),
        }
    }
}
/// Shared actual streaming path for extraction and envelope aggregation. Each
/// source read and selected block uses <=64 KiB; no complete data chunk is loaded.
pub(super) fn visit(
    reader: &mut (impl Read + Seek),
    plan: &Plan,
    mut consume: impl FnMut(Samples) -> Result<(), OperationError>,
) -> Result<(), OperationError> {
    reader.seek(SeekFrom::Start(plan.offset))?;
    let align = usize::from(plan.format.block_align);
    let width = usize::from(plan.format.bits_per_sample / 8);
    let mut buffer = [0; 65536];
    let batch_frames = buffer.len() / align;
    let mut cursor = 0;
    while cursor < plan.frames {
        let frames = (plan.frames - cursor).min(batch_frames as u64) as usize;
        read(reader, &mut buffer[..frames * align])?;
        let mut data = Vec::with_capacity(frames * plan.channels.len() * width);
        for frame in buffer[..frames * align].chunks_exact(align) {
            for channel in &plan.channels {
                let offset = usize::from(*channel) * width;
                data.extend_from_slice(&frame[offset..offset + width]);
            }
        }
        consume(Samples {
            format: plan.format,
            channels: plan.channels.clone(),
            start_frame: plan.start + cursor,
            frames: frames as u64,
            data,
        })?;
        cursor += frames as u64;
    }
    Ok(())
}
/// Validate format/selection before output allocation, then read only the chosen
/// frame interval in <=64 KiB chunks. Raw output <=8 MiB, <=2 Mi selected samples.
/// Shared frame, duration and 64 MiB source budgets also apply. Inspection never
/// decodes payload; this separate explicit operation interprets selected samples.
pub fn extract(
    reader: &mut (impl Read + Seek),
    selection: &Selection,
) -> Result<Samples, OperationError> {
    let result = (|| {
        let info = wave::inspect(reader)?;
        let plan = Plan::resolve(&info, selection, MAX_PCM_BYTES)?;
        if plan.frames * plan.channels.len() as u64 > MAX_SAMPLES {
            return Err(limit("selection exceeds 2 Mi typed samples"));
        }
        let mut result = plan.empty();
        result.frames = plan.frames;
        result.data = Vec::with_capacity(plan.output_bytes as usize);
        visit(reader, &plan, |block| {
            result.data.extend(block.data);
            Ok(())
        })?;
        Ok(result)
    })();
    result.map_err(|e: OperationError| e.operation("audio.pcm.extract"))
}
/// Extract from a regular local file without modifying it.
pub fn extract_file(path: &Path, selection: &Selection) -> Result<Samples, OperationError> {
    forensics::open_regular(path)
        .and_then(|mut reader| extract(&mut reader, selection))
        .map_err(|e| e.operation("audio.pcm.extract"))
}
use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue};
pub(super) const PARAMETERS: &[Parameter] = &[
    Parameter {
        name: "start_frame",
        description: "Absolute zero-based start frame, not byte or single-channel sample offset.",
        required: false,
        default: Some(ParameterValue::Unsigned(0)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: u64::MAX,
        },
    },
    Parameter {
        name: "frames",
        description: "Selected frame count; omitted selects to end. At most 1048576 frames, 300 seconds and 64 MiB source bytes.",
        required: false,
        default: None,
        constraints: ParameterType::Unsigned {
            min: 0,
            max: MAX_FRAMES,
        },
    },
    Parameter {
        name: "channels",
        description: "Ordered unique source channel indexes; omitted preserves all source channels.",
        required: false,
        default: None,
        constraints: ParameterType::UnsignedList {
            min: 0,
            max: 31,
            min_length: 1,
            max_length: 32,
            unique: true,
        },
    },
];
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "audio.pcm.extract",
    category: "audio",
    description: "Extract exact channel/range PCM bytes and signed values; no resampling, <=8 MiB output/2 Mi samples, <=10000 structured values.",
    input_type: "byte_reader",
    output_type: "audio.pcm_samples",
    parameters: PARAMETERS,
    deterministic: true,
    reversible: Some(false),
    command: "audio samples",
    example: "cube audio samples sample.wav --channels 1,0 --start-frame 0 --frames 100 --output samples.pcm",
}];
