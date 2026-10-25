//! Ordered caller-supplied candidates. No generation, inference, I/O or secret capture.
use super::{Network, PrivateInput, ScalarInfo, bitcoin, ethereum, wif};
use crate::{
    errors::{ErrorCode, OperationError},
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::Serialize;
use std::fmt;
pub const MAX_ENTRIES: usize = 256;
pub const MAX_ENTRY_BYTES: usize = 128;
pub const MAX_INPUT_BYTES: usize = 32 * 1024;
pub const MAX_DOCUMENT_BYTES: usize = 64 * 1024;
pub const MAX_WORK: usize = 1024 * 1024;
pub const MAX_OUTPUT_BYTES: usize = 1024 * 1024;
pub const MAX_DERIVATIONS: usize = 32;
const OP: &str = "blockchain.candidates.validate";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    Wif,
    Hex,
    Unsupported,
}
impl Format {
    /// Unknown labels have one fixed classification; never retain the label.
    pub fn parse(name: &str) -> Self {
        match name {
            "wif" => Self::Wif,
            "hex" => Self::Hex,
            _ => Self::Unsupported,
        }
    }
}
pub struct Candidate {
    format: Format,
    value: String,
}
impl fmt::Debug for Candidate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Candidate([REDACTED])")
    }
}
impl Candidate {
    /// Bound before cloning. No Serialize; the original material stays private.
    pub fn new(format: Format, value: &str) -> Result<Self, OperationError> {
        if value.len() > MAX_ENTRY_BYTES {
            return Err(budget("candidate exceeds 128 bytes"));
        }
        Ok(Self {
            format,
            value: value.to_owned(),
        })
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub enum Derive {
    #[default]
    None,
    Bitcoin {
        network: Network,
        compressed: bool,
    },
    Ethereum {
        checksum: ethereum::Checksum,
    },
}
/// Shared CLI/SDK option validation; unused context is rejected.
pub fn derivation(
    name: Option<&str>,
    network: Option<Network>,
    compressed: Option<bool>,
    checksum: Option<&str>,
) -> Result<Derive, OperationError> {
    let invalid = || {
        super::invalid("derive", "select no derivation, bitcoin with network/compression, or ethereum with optional checksum; unused options are rejected").operation(OP)
    };
    match name {
        None | Some("none") if network.is_none() && compressed.is_none() && checksum.is_none() => {
            Ok(Derive::None)
        }
        Some("bitcoin") if checksum.is_none() => Ok(Derive::Bitcoin {
            network: network.ok_or_else(invalid)?,
            compressed: compressed.ok_or_else(invalid)?,
        }),
        Some("ethereum") if network.is_none() && compressed.is_none() => Ok(Derive::Ethereum {
            checksum: ethereum::Checksum::parse(checksum.unwrap_or("eip55"))
                .map_err(|e| e.operation(OP))?,
        }),
        _ => Err(invalid()),
    }
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct BatchPlan {
    pub entries: usize,
    pub input_bytes: usize,
    pub work_units: usize,
    pub max_output_bytes: usize,
    pub max_public_derivations: usize,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Progress {
    pub total: usize,
    pub processed: usize,
    pub valid: usize,
    pub invalid: usize,
    pub public_derived: usize,
    pub complete: bool,
}
#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum PublicAddress {
    Bitcoin(bitcoin::BitcoinAddress),
    Ethereum(ethereum::EthereumAddress),
}
#[derive(Debug, Serialize)]
pub struct CandidateResult {
    pub index: usize,
    pub format: Format,
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scalar: Option<ScalarInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wif: Option<wif::WifInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<OperationError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_address: Option<PublicAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_error: Option<OperationError>,
}
fn budget(message: &str) -> OperationError {
    super::limit(message).operation(OP)
}
fn schema() -> OperationError {
    super::malformed(
        "candidates must be an array of objects containing only string format/value fields",
    )
    .operation(OP)
}
/// Strict bounded JSON transport. Discard parser causes so secret fragments never
/// appear through Debug/error source traversal. No duplicate keys or extra fields.
pub fn parse_json(document: &[u8]) -> Result<Vec<Candidate>, OperationError> {
    if document.len() > MAX_DOCUMENT_BYTES {
        return Err(budget("candidate JSON exceeds 64 KiB"));
    }
    let text = std::str::from_utf8(document).map_err(|_| schema())?;
    let value = crate::pipeline::json::parse(text).map_err(|e| {
        OperationError::new(
            e.error.code,
            "invalid candidate JSON: syntax, duplicate keys or nesting/node budget",
        )
        .operation(OP)
    })?;
    let entries = value.as_array().ok_or_else(schema)?;
    if entries.len() > MAX_ENTRIES {
        return Err(budget("candidate count exceeds 256"));
    }
    let mut result = Vec::with_capacity(entries.len());
    let mut bytes = 0usize;
    for entry in entries {
        let obj = entry.as_object().ok_or_else(schema)?;
        if obj.len() != 2 {
            return Err(schema());
        }
        let format = obj
            .get("format")
            .and_then(|v| v.as_str())
            .ok_or_else(schema)?;
        let value = obj
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(schema)?;
        if format.len() > MAX_ENTRY_BYTES {
            return Err(budget("candidate format label exceeds 128 bytes"));
        }
        bytes = bytes
            .checked_add(value.len())
            .ok_or_else(|| budget("candidate byte count overflow"))?;
        if bytes > MAX_INPUT_BYTES {
            return Err(budget("candidate material exceeds 32 KiB"));
        }
        result.push(Candidate::new(Format::parse(format), value)?);
    }
    Ok(result)
}
/// Preflight all bounds before the first result. Iterator consumption performs
/// one independent validation at a time. Dropping early leaves progress incomplete.
/// Secret input copies are caller-owned; no process-wide zeroization guarantee.
pub struct CandidateBatch {
    remaining: std::vec::IntoIter<Candidate>,
    derive: Derive,
    plan: BatchPlan,
    progress: Progress,
}
impl fmt::Debug for CandidateBatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CandidateBatch")
            .field("plan", &self.plan)
            .field("progress", &self.progress)
            .finish()
    }
}
impl CandidateBatch {
    pub fn new(entries: Vec<Candidate>, derive: Derive) -> Result<Self, OperationError> {
        if entries.len() > MAX_ENTRIES {
            return Err(budget("candidate count exceeds 256"));
        }
        let mut input_bytes = 0usize;
        let mut work_units = 0usize;
        for entry in &entries {
            let len = entry.value.len();
            input_bytes = input_bytes
                .checked_add(len)
                .ok_or_else(|| budget("candidate byte count overflow"))?;
            let work = if entry.format == Format::Wif {
                (len + 2) * (len + 2)
            } else {
                len + 1
            };
            work_units = work_units
                .checked_add(work)
                .ok_or_else(|| budget("candidate work overflow"))?;
        }
        if input_bytes > MAX_INPUT_BYTES || work_units > MAX_WORK {
            return Err(budget("candidate input/work budget exceeded"));
        }
        let max_public_derivations = if matches!(derive, Derive::None) {
            0
        } else {
            entries.len()
        };
        if max_public_derivations > MAX_DERIVATIONS {
            return Err(budget(
                "public derivation is limited to 32 candidate entries",
            ));
        }
        // Bounds include all safe fields, longest errors, public SEC1 data and pretty JSON.
        let max_output_bytes = entries
            .len()
            .checked_mul(2048)
            .and_then(|n| n.checked_add(2048))
            .ok_or_else(|| budget("candidate output bound overflow"))?;
        if max_output_bytes > MAX_OUTPUT_BYTES {
            return Err(budget("candidate output budget exceeded"));
        }
        let count = entries.len();
        Ok(Self {
            remaining: entries.into_iter(),
            derive,
            plan: BatchPlan {
                entries: count,
                input_bytes,
                work_units,
                max_output_bytes,
                max_public_derivations,
            },
            progress: Progress {
                total: count,
                processed: 0,
                valid: 0,
                invalid: 0,
                public_derived: 0,
                complete: count == 0,
            },
        })
    }
    pub fn plan(&self) -> BatchPlan {
        self.plan
    }
    pub fn progress(&self) -> Progress {
        self.progress
    }
}
impl Iterator for CandidateBatch {
    type Item = CandidateResult;
    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.remaining.next()?;
        let mut result = CandidateResult {
            index: self.progress.processed,
            format: entry.format,
            valid: false,
            scalar: None,
            wif: None,
            error: None,
            public_address: None,
            address_error: None,
        };
        let validated = match entry.format {
            Format::Wif => super::parse_private(PrivateInput::Wif(&entry.value), None, None),
            Format::Hex => super::parse_private(PrivateInput::Hex(&entry.value), None, None),
            Format::Unsupported => Err(OperationError::new(
                ErrorCode::UnsupportedFormat,
                "candidate format must be wif or hex",
            )),
        };
        match validated {
            Err(error) => {
                result.error = Some(error.operation(OP));
                self.progress.invalid += 1;
            }
            Ok((key, info)) => {
                result.valid = true;
                result.scalar = Some(key.info());
                result.wif = info;
                self.progress.valid += 1;
                let public = match self.derive {
                    Derive::None => Ok(None),
                    Derive::Bitcoin {
                        network,
                        compressed,
                    } => (|| {
                        if let Some(info) = info {
                            wif::check_intent(&info, Some(network), Some(compressed))?;
                        }
                        bitcoin::from_key(&key, network, compressed)
                            .map(|p| Some(PublicAddress::Bitcoin(p)))
                    })(),
                    Derive::Ethereum { checksum } => {
                        if info.is_some() {
                            Err(super::invalid(
                                "format",
                                "Ethereum derivation accepts raw or hex scalars",
                            ))
                        } else {
                            ethereum::from_key(&key, checksum)
                                .map(|p| Some(PublicAddress::Ethereum(p)))
                        }
                    }
                };
                match public {
                    Ok(public) => {
                        if public.is_some() {
                            self.progress.public_derived += 1;
                        }
                        result.public_address = public;
                    }
                    Err(error) => result.address_error = Some(error.operation(OP)),
                }
            }
        }
        self.progress.processed += 1;
        self.progress.complete = self.remaining.len() == 0;
        Some(result)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.remaining.size_hint()
    }
}
impl ExactSizeIterator for CandidateBatch {}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: OP,
    category: "blockchain",
    description: "Ordered caller-supplied WIF/hex64 candidates; <=256 entries, <=128 bytes each, <=32 KiB material, <=1 Mi work units, <=1 MiB bounded result, <=64 KiB strict JSON with depth16/nodes4096. Safe fixed validation facts/errors only, no inputs/fingerprints. Optional explicit Bitcoin/Ethereum public derivation <=32 entries; source intent remains enforced. Incremental results/progress mark incomplete consumption. No generation, I/O, pipeline/capture adapter or zeroization promise.",
    input_type: "private.candidates",
    output_type: "key.validation_batch",
    deterministic: true,
    reversible: None,
    command: "blockchain candidates",
    example: "cube blockchain candidates --json < public-test-candidates.json",
    parameters: &[
        Parameter {
            name: "derive",
            description: "Explicit public-address output; none by default.",
            required: false,
            default: Some(ParameterValue::Text("none")),
            constraints: ParameterType::Text {
                choices: &["none", "bitcoin", "ethereum"],
            },
        },
        Parameter {
            name: "network",
            description: "Required only for Bitcoin derivation; WIF intent must match.",
            required: false,
            default: None,
            constraints: ParameterType::Text {
                choices: &["mainnet", "testnet"],
            },
        },
        Parameter {
            name: "compressed",
            description: "Required only for Bitcoin derivation; WIF intent must match.",
            required: false,
            default: None,
            constraints: ParameterType::Boolean,
        },
        Parameter {
            name: "checksum",
            description: "Only for Ethereum derivation; defaults to eip55.",
            required: false,
            default: None,
            constraints: ParameterType::Text {
                choices: &["lowercase", "eip55"],
            },
        },
    ],
}];
