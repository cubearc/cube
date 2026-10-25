//! Deterministic bounded sequential execution with cooperative cancellation boundaries.
use super::{
    Error, ForensicReport, HexEvidence, Plan, SignatureEvidence, StringEvidence, Value,
    adapters::{Action, Arguments, Prepared},
    invalid,
};
use crate::{
    binary, ciphers, encoding,
    errors::{ErrorCode, OperationError},
    hashing, text,
};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, Ordering},
};
#[derive(Debug, Serialize)]
pub struct Execution {
    pub output: Value,
    pub intermediates: BTreeMap<usize, Value>,
    pub work_bytes_upper_bound: u64,
}
impl Prepared {
    /// Conservative output-size upper bounds guard allocation before algorithms.
    fn ceiling(&self, size: usize) -> Result<usize, Error> {
        if matches!(self.adapter.action, Action::TextSelect) && size > text::MAX_INPUT {
            return Err(Error::new(
                ErrorCode::ResourceLimit,
                "text selection input exceeds 1 MiB",
            ));
        }
        if (matches!(self.adapter.action, Action::A1z26Encode) && size > text::MAX_INPUT)
            || (matches!(self.adapter.action, Action::A1z26Decode)
                && size > crate::symbols::a1z26::MAX_ENCODED)
        {
            return Err(Error::new(
                ErrorCode::ResourceLimit,
                "A1Z26 pipeline input exceeds its text budget",
            ));
        }
        if let Arguments::Forensic {
            range,
            width,
            min_length,
            max_results,
        } = self.arguments
        {
            let (_, selected) = range
                .resolve(size as u64, super::MAX_VALUE_BYTES as u64)
                .map_err(Error::from_operation)?;
            let selected = selected as usize;
            return Ok(match self.adapter.action {
                Action::HexInspect => 64 + selected * 8 + selected.div_ceil(width) * 160,
                Action::Strings => {
                    64 + selected + selected.div_ceil(min_length + 1).min(max_results) * 128
                }
                _ => return Err(invalid("inconsistent forensic adapter")),
            });
        }
        if matches!(
            self.adapter.action,
            Action::Base58Encode | Action::Base58Decode
        ) && size > 4096
        {
            return Err(Error::new(
                ErrorCode::ResourceLimit,
                "Base58 pipeline adapters accept at most 4096 input bytes per step",
            ));
        }
        Ok(match self.adapter.action {
            Action::A1z26Encode => size * 3,
            Action::Base32Encode => size.div_ceil(5) * 8,
            Action::Base32Decode => size / 8 * 5,
            Action::Base64Encode => size.div_ceil(3) * 4,
            Action::Base64Decode => size / 4 * 3,
            Action::HexEncode => size * 2,
            Action::HexDecode => size / 2,
            Action::Base58Encode => size * 2,
            Action::Crc32 => 8,
            Action::Sha256 => 64,
            Action::Signatures => 8192,
            _ => size,
        })
    }
    fn apply(&self, value: Value) -> Result<Value, OperationError> {
        let legacy = |e: crate::Error| {
            OperationError::new(e.code(), e.to_string())
                .operation(e.operation_id())
                .caused_by(e)
        };
        let bytes = |data| Value::Bytes { data };
        let text = |data| Value::Text { data };
        Ok(match (self.adapter.action, value, &self.arguments) {
            (Action::And, Value::Bytes { data }, Arguments::Bytes(key)) => {
                bytes(binary::transform::and(&data, key)?)
            }
            (Action::Or, Value::Bytes { data }, Arguments::Bytes(key)) => {
                bytes(binary::transform::or(&data, key)?)
            }
            (Action::Not, Value::Bytes { data }, _) => bytes(binary::transform::not(&data)?),
            (Action::ReverseBytes, Value::Bytes { data }, _) => {
                bytes(binary::transform::reverse(&data)?)
            }
            (Action::ShiftLeft, Value::Bytes { data }, Arguments::Units { bits, unit_bytes }) => {
                bytes(binary::transform::shift_left(&data, *bits, *unit_bytes)?)
            }
            (Action::ShiftRight, Value::Bytes { data }, Arguments::Units { bits, unit_bytes }) => {
                bytes(binary::transform::shift_right(&data, *bits, *unit_bytes)?)
            }
            (Action::SwapEndian, Value::Bytes { data }, Arguments::Units { unit_bytes, .. }) => {
                bytes(binary::transform::swap_endian(&data, *unit_bytes)?)
            }
            (Action::TextSelect, Value::Text { data }, Arguments::Selector(selector)) => {
                text(crate::text::selection::select(&data, *selector)?)
            }
            (Action::A1z26Encode, Value::Text { data }, _) => {
                text(crate::symbols::a1z26::encode(&data)?)
            }
            (Action::A1z26Decode, Value::Text { data }, _) => {
                text(crate::symbols::a1z26::decode(&data)?)
            }
            (Action::Signatures, Value::Bytes { data }, _) => Value::ForensicReport {
                data: ForensicReport::Signatures {
                    matches: crate::forensics::signatures::detect(&data)
                        .into_iter()
                        .map(|r| SignatureEvidence {
                            format: r.format.into(),
                            mime: r.mime.into(),
                            offset: r.offset,
                            evidence_hex: r.evidence_hex,
                            rule: r.rule.into(),
                        })
                        .collect(),
                },
            },
            (
                Action::HexInspect,
                Value::Bytes { data },
                Arguments::Forensic { range, width, .. },
            ) => {
                let rows = crate::forensics::ranges::hex_rows(
                    &mut std::io::Cursor::new(data),
                    *range,
                    *width,
                )?;
                Value::ForensicReport {
                    data: ForensicReport::Hex {
                        rows: rows
                            .into_iter()
                            .map(|r| HexEvidence {
                                offset: r.offset,
                                bytes: r.bytes,
                                hex: r.hex,
                                ascii: r.ascii,
                            })
                            .collect(),
                    },
                }
            }
            (
                Action::Strings,
                Value::Bytes { data },
                Arguments::Forensic {
                    range,
                    min_length,
                    max_results,
                    ..
                },
            ) => {
                let result = crate::forensics::ranges::strings(
                    &mut std::io::Cursor::new(data),
                    *range,
                    *min_length,
                    *max_results,
                )?;
                Value::ForensicReport {
                    data: ForensicReport::Strings {
                        strings: result
                            .strings
                            .into_iter()
                            .map(|r| StringEvidence {
                                offset: r.offset,
                                length: r.length,
                                text: r.text,
                            })
                            .collect(),
                        scanned_bytes: result.scanned_bytes,
                        complete: result.complete,
                    },
                }
            }
            (Action::Base32Encode, Value::Bytes { data }, _) => {
                text(encoding::base32_encode(&data))
            }
            (Action::Base58Encode, Value::Bytes { data }, _) => {
                text(encoding::base58_encode(&data))
            }
            (Action::Base64Encode, Value::Bytes { data }, _) => {
                text(encoding::base64_encode(&data))
            }
            (Action::HexEncode, Value::Bytes { data }, _) => text(encoding::hex_encode(&data)),
            (Action::Base32Decode, Value::Text { data }, _) => {
                bytes(encoding::base32_decode(&data).map_err(legacy)?)
            }
            (Action::Base58Decode, Value::Text { data }, _) => {
                bytes(encoding::base58_decode(&data).map_err(legacy)?)
            }
            (Action::Base64Decode, Value::Text { data }, _) => {
                bytes(encoding::base64_decode(&data).map_err(legacy)?)
            }
            (Action::HexDecode, Value::Text { data }, _) => {
                bytes(encoding::hex_decode(&data).map_err(legacy)?)
            }
            (Action::CaesarEncode, Value::Text { data }, Arguments::Shift(shift)) => {
                text(ciphers::caesar_encode(&data, *shift))
            }
            (Action::CaesarDecode, Value::Text { data }, Arguments::Shift(shift)) => {
                text(ciphers::caesar_decode(&data, *shift))
            }
            (Action::Rot13, Value::Text { data }, _) => text(ciphers::rot13(&data)),
            (Action::Atbash, Value::Text { data }, _) => text(ciphers::atbash(&data)?),
            (Action::VigenereEncode, Value::Text { data }, Arguments::Key(key)) => {
                text(ciphers::vigenere_encode(&data, key)?)
            }
            (Action::VigenereDecode, Value::Text { data }, Arguments::Key(key)) => {
                text(ciphers::vigenere_decode(&data, key)?)
            }
            (Action::Beaufort, Value::Text { data }, Arguments::Key(key)) => {
                text(ciphers::beaufort(&data, key)?)
            }
            (Action::AffineEncode, Value::Text { data }, Arguments::Affine(a, b)) => {
                text(ciphers::affine_encode(&data, *a, *b)?)
            }
            (Action::AffineDecode, Value::Text { data }, Arguments::Affine(a, b)) => {
                text(ciphers::affine_decode(&data, *a, *b)?)
            }
            (Action::Crc32, Value::Bytes { data }, _) => text(hashing::crc32(&data)),
            (Action::Sha256, Value::Bytes { data }, _) => text(hashing::sha256(&data)),
            (Action::Xor, Value::Bytes { data }, Arguments::Bytes(key)) => {
                bytes(binary::xor(&data, key).map_err(legacy)?)
            }
            (Action::Utf8Encode, Value::Text { data }, _) => bytes(text::to_utf8(&data)?),
            (Action::Utf8Decode, Value::Bytes { data }, _) => text(text::from_utf8(&data)?),
            _ => {
                return Err(OperationError::new(
                    ErrorCode::InvalidInput,
                    "validated adapter/value contract is inconsistent",
                ));
            }
        })
    }
}
impl Plan {
    /// Validate actual input, capture selections and conservative size/work bounds
    /// for every step before executing any algorithm. Core direct APIs are unchanged.
    /// Cancellation is checked at entry and before/after each bounded operation;
    /// an operation already running is not preempted. No wall-clock timeout is claimed.
    pub fn execute(
        &self,
        input: Value,
        capture: &[usize],
        cancel: Option<&AtomicBool>,
    ) -> Result<Execution, Error> {
        let cancelled = || cancel.is_some_and(|flag| flag.load(Ordering::Relaxed));
        if cancelled() {
            return Err(Error::new(
                ErrorCode::Cancelled,
                "pipeline cancelled before execution",
            ));
        }
        if input.kind() != self.input {
            return Err(
                invalid("input value does not match the validated plan type").parameter("input"),
            );
        }
        input.validate(self.limits.max_value_bytes)?;
        if capture.len() > self.steps.len()
            || capture
                .iter()
                .enumerate()
                .any(|(i, s)| *s >= self.steps.len() || capture[..i].contains(s))
        {
            return Err(invalid(
                "capture indexes must be unique, zero-based and present in the plan",
            )
            .parameter("capture"));
        }
        let mut size = input.size();
        let mut work = size as u64;
        let mut retained = 0usize;
        for (index, step) in self.steps.iter().enumerate() {
            let result = (|| {
                let output = step.ceiling(size)?;
                if output > self.limits.max_value_bytes {
                    return Err(Error::new(
                        ErrorCode::ResourceLimit,
                        "conservative intermediate/output bound exceeds value budget",
                    ));
                }
                work += size as u64 + output as u64;
                if work > self.limits.max_work_bytes {
                    return Err(Error::new(
                        ErrorCode::ResourceLimit,
                        "conservative execution work-byte budget exceeded",
                    ));
                }
                if capture.contains(&index) {
                    retained += output;
                    if retained > self.limits.max_capture_bytes {
                        return Err(Error::new(
                            ErrorCode::ResourceLimit,
                            "conservative intermediate-capture budget exceeded",
                        ));
                    }
                }
                size = output;
                Ok(())
            })();
            result.map_err(|e: Error| e.at(index, Some(step.adapter.id)))?;
        }
        if work > self.limits.max_work_bytes {
            return Err(Error::new(
                ErrorCode::ResourceLimit,
                "input exceeds execution work-byte budget",
            ));
        }
        let mut output = input;
        let mut intermediates = BTreeMap::new();
        for (index, step) in self.steps.iter().enumerate() {
            if cancelled() {
                return Err(Error::new(
                    ErrorCode::Cancelled,
                    "pipeline cancelled at operation boundary",
                )
                .at(index, Some(step.adapter.id)));
            }
            output = step
                .apply(output)
                .map_err(|e| Error::from_operation(e).at(index, Some(step.adapter.id)))?;
            output
                .validate(self.limits.max_value_bytes)
                .map_err(|e| e.at(index, Some(step.adapter.id)))?;
            if cancelled() {
                return Err(Error::new(
                    ErrorCode::Cancelled,
                    "pipeline cancelled at operation boundary",
                )
                .at(index, Some(step.adapter.id)));
            }
            if capture.contains(&index) {
                intermediates.insert(index, output.clone());
            }
        }
        Ok(Execution {
            output,
            intermediates,
            work_bytes_upper_bound: work,
        })
    }
}
