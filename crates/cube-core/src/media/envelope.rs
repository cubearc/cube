//! Incremental exact integer waveform envelopes; no plotting, FFT or resampling.
use super::{
    pcm::{self, Plan, Samples, Selection, invalid},
    wave::{self, Format, limit},
};
use crate::{errors::OperationError, forensics};
use serde::Serialize;
use std::{
    io::{Read, Seek},
    path::Path,
};
const OP: &str = "audio.waveform.envelope";
/// Bound both total buckets and channel/bucket pairs to 10000.
pub const MAX_BUCKETS: usize = 10000;
/// Exact extrema for one bucket, in the accompanying original channel-index order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Bucket {
    pub start_frame: u64,
    pub frames: u64,
    pub minimum: Vec<i32>,
    pub maximum: Vec<i32>,
}
/// Sample rate is the exact time base for absolute frame positions; amplitudes
/// retain signed integer sample units, including centered unsigned-8 storage.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Envelope {
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    pub source_channels: u16,
    pub channels: Vec<u16>,
    pub start_frame: u64,
    pub frames: u64,
    pub bucket_frames: u64,
    pub buckets: Vec<Bucket>,
}
/// Aggregate consecutive immutable PCM blocks without keeping sample buffers.
/// Blocks must have identical original format/channel order and contiguous ranges.
pub struct Builder {
    format: Format,
    channels: Vec<u16>,
    start: u64,
    frames: u64,
    bucket_frames: u64,
    max_buckets: usize,
    pending_frames: u64,
    minimum: Vec<i32>,
    maximum: Vec<i32>,
    buckets: Vec<Bucket>,
}
impl Builder {
    /// Initialize from a validated block's metadata without consuming that block.
    /// Call push(first) explicitly. Empty blocks/empty finished series are valid.
    pub fn new(
        first: &Samples,
        bucket_frames: u64,
        max_buckets: usize,
    ) -> Result<Self, OperationError> {
        if bucket_frames == 0 || bucket_frames > pcm::MAX_FRAMES {
            return Err(invalid("bucket_frames must be in 1..=1048576")
                .operation(OP)
                .parameter("bucket_frames"));
        }
        if max_buckets == 0 || max_buckets > MAX_BUCKETS {
            return Err(invalid("max_buckets must be in 1..=10000")
                .operation(OP)
                .parameter("max_buckets"));
        }
        let channels = first.channels().to_vec();
        let count = channels.len();
        Ok(Self {
            format: first.format(),
            channels,
            start: first.start_frame(),
            frames: 0,
            bucket_frames,
            max_buckets,
            pending_frames: 0,
            minimum: vec![i32::MAX; count],
            maximum: vec![i32::MIN; count],
            buckets: Vec::new(),
        })
    }
    fn check_budget(&self, frames: u64) -> Result<(), OperationError> {
        if frames > pcm::MAX_FRAMES
            || frames > pcm::MAX_DURATION_SECONDS * u64::from(self.format.sample_rate)
            || frames * u64::from(self.format.block_align) > pcm::MAX_PROCESS_BYTES
        {
            return Err(
                limit("envelope exceeds shared frame, duration or 64 MiB source budget")
                    .operation(OP),
            );
        }
        let buckets = frames.div_ceil(self.bucket_frames);
        if buckets > self.max_buckets as u64
            || buckets * self.channels.len() as u64 > MAX_BUCKETS as u64
        {
            return Err(limit("envelope exceeds bucket or 10000 channel/bucket-pair budget; increase bucket_frames or select fewer channels").operation(OP));
        }
        Ok(())
    }
    fn complete_bucket(&mut self) {
        let count = self.channels.len();
        self.buckets.push(Bucket {
            start_frame: self.start + self.frames - self.pending_frames,
            frames: self.pending_frames,
            minimum: std::mem::replace(&mut self.minimum, vec![i32::MAX; count]),
            maximum: std::mem::replace(&mut self.maximum, vec![i32::MIN; count]),
        });
        self.pending_frames = 0;
    }
    /// Validate the entire block/budget before consuming it. No normalization,
    /// decimation or clipping; bucket boundaries do not depend on input batching.
    pub fn push(&mut self, block: &Samples) -> Result<(), OperationError> {
        if block.format() != self.format
            || block.channels() != self.channels
            || block.start_frame() != self.start + self.frames
        {
            return Err(invalid(
                "PCM blocks must have matching format/channel order and contiguous frame ranges",
            )
            .operation(OP)
            .parameter("samples"));
        }
        let frames = self
            .frames
            .checked_add(block.frames())
            .ok_or_else(|| limit("envelope frame count overflow").operation(OP))?;
        self.check_budget(frames)?;
        let mut values = block.iter_values();
        for _ in 0..block.frames() {
            for channel in 0..self.channels.len() {
                let value = values.next().expect("validated PCM block shape");
                self.minimum[channel] = self.minimum[channel].min(value);
                self.maximum[channel] = self.maximum[channel].max(value);
            }
            self.frames += 1;
            self.pending_frames += 1;
            if self.pending_frames == self.bucket_frames {
                self.complete_bucket();
            }
        }
        Ok(())
    }
    /// Include the final partial bucket with its actual frame count; no padding.
    pub fn finish(mut self) -> Envelope {
        if self.pending_frames != 0 {
            self.complete_bucket();
        }
        Envelope {
            sample_rate: self.format.sample_rate,
            bits_per_sample: self.format.bits_per_sample,
            source_channels: self.format.channels,
            channels: self.channels,
            start_frame: self.start,
            frames: self.frames,
            bucket_frames: self.bucket_frames,
            buckets: self.buckets,
        }
    }
}
/// Aggregate one extracted block; the incremental Builder accepts the same type.
pub fn from_samples(
    samples: &Samples,
    bucket_frames: u64,
    max_buckets: usize,
) -> Result<Envelope, OperationError> {
    let mut builder = Builder::new(samples, bucket_frames, max_buckets)?;
    builder.push(samples)?;
    Ok(builder.finish())
}
/// Reuse PCM selection/streaming with <=64 KiB source and selected-block buffers.
/// Check complete result budgets before any audio read; metadata remains separate.
pub fn from_reader(
    reader: &mut (impl Read + Seek),
    selection: &Selection,
    bucket_frames: u64,
    max_buckets: usize,
) -> Result<Envelope, OperationError> {
    let result = (|| {
        let info = wave::inspect(reader)?;
        let plan = Plan::resolve(&info, selection, pcm::MAX_PROCESS_BYTES)?;
        let mut builder = Builder::new(&plan.empty(), bucket_frames, max_buckets)?;
        builder.check_budget(plan.frames)?;
        pcm::visit(reader, &plan, |block| builder.push(&block))?;
        Ok(builder.finish())
    })();
    result.map_err(|e: OperationError| e.operation(OP))
}
/// Stream a selected regular local file; no full sample buffering or writes.
pub fn from_file(
    path: &Path,
    selection: &Selection,
    bucket_frames: u64,
    max_buckets: usize,
) -> Result<Envelope, OperationError> {
    forensics::open_regular(path)
        .and_then(|mut reader| from_reader(&mut reader, selection, bucket_frames, max_buckets))
        .map_err(|e| e.operation(OP))
}
use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue};
const BUCKET: Parameter = Parameter {
    name: "bucket_frames",
    description: "Positive frames per bucket (1..=1048576), including a final partial bucket without padding.",
    required: false,
    default: Some(ParameterValue::Unsigned(1024)),
    constraints: ParameterType::Unsigned {
        min: 1,
        max: pcm::MAX_FRAMES,
    },
};
const RESULTS: Parameter = Parameter {
    name: "max_buckets",
    description: "Maximum complete buckets, 1..=10000; total channel/bucket pairs also <=10000. Exceeding either budget fails.",
    required: false,
    default: Some(ParameterValue::Unsigned(1000)),
    constraints: ParameterType::Unsigned {
        min: 1,
        max: MAX_BUCKETS as u64,
    },
};
/// Metadata uses the same selection parameters as PCM extraction.
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: OP,
    category: "audio",
    description: "Stream exact integer min/max envelopes per selected channel; preserve frame positions and partial buckets; explicit bounded results.",
    input_type: "byte_reader",
    output_type: "audio.waveform_envelope",
    deterministic: true,
    reversible: Some(false),
    command: "audio waveform",
    example: "cube audio waveform sample.wav --channels 0 --bucket-frames 1024 --json",
    parameters: &[
        pcm::PARAMETERS[0],
        pcm::PARAMETERS[1],
        pcm::PARAMETERS[2],
        BUCKET,
        RESULTS,
    ],
}];
