//! Owned, inert snapshots of concrete forensic results; no paths or I/O capability.
use super::{Error, invalid};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SignatureEvidence {
    pub format: String,
    pub mime: String,
    pub offset: u64,
    pub evidence_hex: String,
    pub rule: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HexEvidence {
    pub offset: u64,
    /// Exact bytes; JSON uses canonical base64, Python uses bytes.
    #[serde(with = "super::value::base64_wire")]
    #[schemars(with = "String", extend("contentEncoding" = "base64"))]
    pub bytes: Vec<u8>,
    pub hex: String,
    pub ascii: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StringEvidence {
    pub offset: u64,
    pub length: u64,
    pub text: String,
}

/// Only these concrete result shapes are supported. These observations do not
/// validate file contents or confer extraction permissions. Caller-supplied
/// snapshots are data, not independently verified evidence.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ForensicReport {
    Signatures {
        matches: Vec<SignatureEvidence>,
    },
    Hex {
        rows: Vec<HexEvidence>,
    },
    Strings {
        strings: Vec<StringEvidence>,
        scanned_bytes: u64,
        complete: bool,
    },
}

impl ForensicReport {
    /// Conservative logical allocation units, including per-record overhead.
    pub fn size(&self) -> usize {
        let add = |a: usize, b| a.saturating_add(b);
        match self {
            Self::Signatures { matches } => matches.iter().fold(64, |sum, row| {
                [
                    160,
                    row.format.len(),
                    row.mime.len(),
                    row.evidence_hex.len(),
                    row.rule.len(),
                ]
                .into_iter()
                .fold(sum, add)
            }),
            Self::Hex { rows } => rows.iter().fold(64, |sum, row| {
                [160, row.bytes.len(), row.hex.len(), row.ascii.len()]
                    .into_iter()
                    .fold(sum, add)
            }),
            Self::Strings { strings, .. } => strings.iter().fold(64, |sum, row| {
                sum.saturating_add(128).saturating_add(row.text.len())
            }),
        }
    }

    pub(super) fn validate(&self) -> Result<(), Error> {
        let range = |offset: u64, length: u64| {
            offset
                .checked_add(length)
                .ok_or_else(|| invalid("forensic result range overflows"))
                .map(|_| ())
        };
        match self {
            Self::Signatures { matches } => {
                for row in matches {
                    let bytes = crate::encoding::hex_decode(&row.evidence_hex)
                        .map_err(|_| invalid("invalid signature evidence hex"))?;
                    range(row.offset, bytes.len() as u64)?;
                }
            }
            Self::Hex { rows } => {
                for row in rows {
                    range(row.offset, row.bytes.len() as u64)?;
                    if crate::encoding::hex_encode(&row.bytes) != row.hex
                        || row.ascii
                            != row
                                .bytes
                                .iter()
                                .map(|b| {
                                    if (0x20..=0x7e).contains(b) {
                                        *b as char
                                    } else {
                                        '.'
                                    }
                                })
                                .collect::<String>()
                    {
                        return Err(invalid("inconsistent hexadecimal result"));
                    }
                }
            }
            Self::Strings { strings, .. } => {
                for row in strings {
                    range(row.offset, row.length)?;
                    if row.length != row.text.len() as u64
                        || !row.text.bytes().all(|b| (0x20..=0x7e).contains(&b))
                    {
                        return Err(invalid("inconsistent printable string result"));
                    }
                }
            }
        }
        Ok(())
    }
}
