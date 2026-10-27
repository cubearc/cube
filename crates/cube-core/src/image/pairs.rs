//! Explicit bounded image pairs, delegating every sample decision to existing APIs.
use super::{BitwiseOperation, Comparison, Image, invalid, limit};
use crate::{
    errors::OperationError,
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::Serialize;
#[derive(Clone, Copy, Debug)]
pub enum PairOperation {
    Bitwise(BitwiseOperation),
    Compare,
}
impl PairOperation {
    pub fn parse(value: &str) -> Result<Self, OperationError> {
        if value == "compare" {
            Ok(Self::Compare)
        } else {
            BitwiseOperation::parse(value)
                .map(Self::Bitwise)
                .map_err(|e| e.operation("image.pairs"))
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Compare => "compare",
            Self::Bitwise(BitwiseOperation::Xor) => "xor",
            Self::Bitwise(BitwiseOperation::And) => "and",
            Self::Bitwise(BitwiseOperation::Or) => "or",
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Plan {
    pub operation: &'static str,
    pub input_count: usize,
    pub pair_count: usize,
    pub pixel_work: u64,
    pub input_sample_bytes: usize,
    pub png_bound: usize,
    pub max_differences: usize,
}
/// Exact checked n*(n-1)/2, including zero/single collections.
pub fn pair_count(n: usize) -> Result<usize, OperationError> {
    let previous = n.saturating_sub(1);
    let (left, right) = if n.is_multiple_of(2) {
        (n / 2, previous)
    } else {
        (n, previous / 2)
    };
    left.checked_mul(right)
        .ok_or_else(|| limit("image pair count overflow").operation("image.pairs"))
}
/// At most eight distinct object identities, 28 pairs, 64 MiB input samples,
/// 16 Mi compared pixel pairs, 64 MiB conservative PNG output and 1000 retained
/// differences across the collection. Distinct objects with equal bytes are valid.
pub fn plan(
    images: &[&Image],
    operation: PairOperation,
    max_pairs: usize,
    max_differences: usize,
) -> Result<Plan, OperationError> {
    let result = (|| {
        let count = pair_count(images.len())?;
        if images.len() > 8 || max_pairs > 28 || count > max_pairs {
            return Err(limit(
                "image collection exceeds eight inputs or selected pair cap (hard cap 28)",
            ));
        }
        if !(1..=1000).contains(&max_differences) {
            return Err(invalid("max_differences must be 1..=1000 across all pairs"));
        }
        for (i, image) in images.iter().enumerate() {
            if images[..i].iter().any(|other| std::ptr::eq(*image, *other)) {
                return Err(invalid("duplicate image object; supply each input once"));
            }
        }
        let input_sample_bytes = images.iter().try_fold(0usize, |sum, image| {
            sum.checked_add(image.samples().len())
                .ok_or_else(|| limit("input sample count overflow"))
        })?;
        let mut pixel_work = 0u64;
        let mut png_bound = 0usize;
        for (i, left) in images.iter().enumerate() {
            for right in &images[i + 1..] {
                pixel_work = pixel_work
                    .checked_add(
                        (u64::from(left.width()) * u64::from(left.height()))
                            .max(u64::from(right.width()) * u64::from(right.height())),
                    )
                    .ok_or_else(|| limit("pair work overflow"))?;
                if matches!(operation, PairOperation::Bitwise(_)) {
                    png_bound = png_bound
                        .checked_add(left.samples().len() * 2 + 4096)
                        .ok_or_else(|| limit("pair PNG bound overflow"))?;
                }
            }
        }
        if input_sample_bytes > 64 * 1024 * 1024
            || pixel_work > 16 * 1024 * 1024
            || png_bound > 64 * 1024 * 1024
        {
            return Err(limit("image batch exceeds input, pixel-work or PNG budget"));
        }
        Ok(Plan {
            operation: operation.name(),
            input_count: images.len(),
            pair_count: count,
            pixel_work,
            input_sample_bytes,
            png_bound,
            max_differences,
        })
    })();
    result.map_err(|e: OperationError| e.operation("image.pairs"))
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct PairResult {
    pub left: usize,
    pub right: usize,
    pub status: &'static str,
    pub name: Option<String>,
    pub comparison: Option<Comparison>,
    pub error: Option<OperationError>,
    #[serde(skip)]
    pub png: Option<Vec<u8>>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Batch {
    pub plan: Plan,
    pub successes: usize,
    pub failures: usize,
    pub encoded_bytes: usize,
    pub retained_differences: usize,
    pub results: Vec<PairResult>,
}
/// Enumerate (i,j), i<j in caller order. Compatibility failures are per-pair
/// errors; successful outputs remain available. No I/O or implicit conversions.
pub fn execute(
    images: &[&Image],
    operation: PairOperation,
    max_pairs: usize,
    max_differences: usize,
) -> Result<Batch, OperationError> {
    let plan = plan(images, operation, max_pairs, max_differences)?;
    let mut batch = Batch {
        plan,
        successes: 0,
        failures: 0,
        encoded_bytes: 0,
        retained_differences: 0,
        results: Vec::new(),
    };
    for (i, left) in images.iter().enumerate() {
        for (j, right) in images.iter().enumerate().skip(i + 1) {
            let mut result = PairResult {
                left: i,
                right: j,
                status: "ready",
                name: None,
                comparison: None,
                error: None,
                png: None,
            };
            let outcome = (|| {
                match operation {
                    PairOperation::Bitwise(op) => {
                        let png = left.bitwise(right, op)?.encode_png()?;
                        batch.encoded_bytes = batch
                            .encoded_bytes
                            .checked_add(png.len())
                            .ok_or_else(|| limit("pair output overflow"))?;
                        if batch.encoded_bytes > 64 * 1024 * 1024 {
                            return Err(limit("pair PNG bytes exceed 64 MiB"));
                        }
                        result.name = Some(format!("pair-{i:02}-{j:02}.png"));
                        result.png = Some(png);
                    }
                    PairOperation::Compare => {
                        let remaining = max_differences - batch.retained_differences;
                        let mut comparison =
                            left.compare(right, None, None, remaining.max(1), false)?;
                        comparison.differences.truncate(remaining);
                        comparison.complete =
                            comparison.differing_pixels as usize <= comparison.differences.len();
                        batch.retained_differences += comparison.differences.len();
                        result.comparison = Some(comparison);
                    }
                }
                Ok(())
            })();
            if let Err(error) = outcome {
                result.status = "error";
                result.error = Some(error);
                batch.failures += 1;
            } else {
                batch.successes += 1;
            }
            batch.results.push(result);
        }
    }
    if serde_json::to_vec(&batch)
        .map_err(|e| invalid("batch manifest serialization failed").caused_by(e))?
        .len()
        > 512 * 1024
    {
        return Err(limit("pair manifest exceeds 512 KiB").operation("image.pairs"));
    }
    Ok(batch)
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "image.pairs",
    category: "image",
    description: "Plan/execute lexicographic unordered pairs over <=8 explicit image objects; reuse XOR/AND/OR/compare, retain per-pair failures, bounded PNGs and differences; no I/O.",
    input_type: "image.collection",
    output_type: "image.pair_batch",
    parameters: &[
        Parameter {
            name: "operation",
            description: "Concrete single-pair allowlist; physical alpha and strict existing model rules.",
            required: true,
            default: None,
            constraints: ParameterType::Text {
                choices: &["xor", "and", "or", "compare"],
            },
        },
        Parameter {
            name: "max_pairs",
            description: "Reject excessive exact pair count; 0 allows empty/single inputs.",
            required: false,
            default: Some(ParameterValue::Unsigned(28)),
            constraints: ParameterType::Unsigned { min: 0, max: 28 },
        },
        Parameter {
            name: "max_differences",
            description: "Global retained comparison records; counts remain exact.",
            required: false,
            default: Some(ParameterValue::Unsigned(1000)),
            constraints: ParameterType::Unsigned { min: 1, max: 1000 },
        },
        Parameter {
            name: "plan_only",
            description: "Validate budgets/counts without running single-pair operations.",
            required: false,
            default: Some(ParameterValue::Boolean(false)),
            constraints: ParameterType::Boolean,
        },
    ],
    deterministic: true,
    reversible: None,
    command: "image pairs",
    example: "cube image pairs first.png second.png --operation xor --plan",
}];
