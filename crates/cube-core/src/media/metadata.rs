//! Native FLAC metadata profile: bounded framing before maintained tag parsing.
use crate::{
    errors::{ErrorCode, OperationError},
    forensics,
    operations::Descriptor,
};
use lofty::{
    config::{ParseOptions, ParsingMode},
    file::AudioFile,
    flac::FlacFile,
};
use serde::Serialize;
use std::{
    io::{Cursor, Read, Seek, SeekFrom},
    path::Path,
};
const OP: &str = "audio.metadata.inspect";
pub const MAX_METADATA: usize = 1024 * 1024;
pub const MAX_BLOCKS: usize = 256;
pub const MAX_COMMENTS: usize = 256;
pub const MAX_FIELD: usize = 65536;
pub const MAX_TEXT: usize = 16384;
fn invalid(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message).operation(OP)
}
fn limit(message: &str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message).operation(OP)
}
fn read(reader: &mut impl Read, bytes: &mut [u8]) -> Result<(), OperationError> {
    reader.read_exact(bytes).map_err(|e| {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            invalid("truncated FLAC metadata").caused_by(e)
        } else {
            OperationError::io(e).operation(OP)
        }
    })
}
fn comments(data: &[u8]) -> Result<(), OperationError> {
    let mut offset = 0usize;
    fn number(data: &[u8], offset: &mut usize) -> Result<usize, OperationError> {
        let bytes = data
            .get(*offset..*offset + 4)
            .ok_or_else(|| invalid("truncated Vorbis comment length"))?;
        *offset += 4;
        Ok(u32::from_le_bytes(bytes.try_into().expect("four bytes")) as usize)
    }
    fn field<'a>(data: &'a [u8], offset: &mut usize) -> Result<&'a str, OperationError> {
        let size = number(data, offset)?;
        if size > MAX_FIELD {
            return Err(limit("Vorbis vendor/comment field exceeds 64 KiB"));
        }
        let end = offset
            .checked_add(size)
            .ok_or_else(|| invalid("comment extent overflow"))?;
        let value = data
            .get(*offset..end)
            .ok_or_else(|| invalid("comment extent exceeds block"))?;
        *offset = end;
        std::str::from_utf8(value)
            .map_err(|_| invalid("Vorbis vendor/comments must be valid UTF-8"))
    }
    field(data, &mut offset)?;
    let count = number(data, &mut offset)?;
    if count > MAX_COMMENTS {
        return Err(limit("Vorbis comments exceed 256 records"));
    }
    for _ in 0..count {
        let value = field(data, &mut offset)?;
        let (key, _) = value
            .split_once('=')
            .ok_or_else(|| invalid("Vorbis comment must contain KEY=VALUE"))?;
        if key.is_empty() || !key.bytes().all(|b| (0x20..=0x7d).contains(&b) && b != b'=') {
            return Err(invalid(
                "Vorbis comment key must be printable ASCII without '='",
            ));
        }
    }
    if offset != data.len() {
        return Err(invalid("unexpected bytes after Vorbis comments"));
    }
    Ok(())
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Block {
    pub offset: u64,
    pub block_type: u8,
    pub length: usize,
    pub interpreted: bool,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Comment {
    pub key: String,
    pub value: String,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Metadata {
    pub format: &'static str,
    pub codec: &'static str,
    pub file_bytes: u64,
    pub metadata_bytes: usize,
    pub sample_rate: u32,
    pub channels: u8,
    pub bits_per_sample: u8,
    pub declared_total_samples: Option<u64>,
    pub duration_milliseconds: Option<u64>,
    pub duration_source: &'static str,
    pub declared_md5: Option<String>,
    pub vendor: Option<String>,
    pub comments: Vec<Comment>,
    pub blocks: Vec<Block>,
    pub verification: &'static str,
    pub artwork: &'static str,
}
/// Inspect native FLAC STREAMINFO and one optional Vorbis-comment block. Reject
/// ID3-prefixed FLAC, MP3, Ogg and reserved block types explicitly. At most 1 GiB
/// physical source, 1 MiB total metadata, 256 blocks/comments, 64 KiB per text field
/// and 16 KiB exported text. Artwork/other blocks are skipped without expansion.
/// A bounded prefix is checked before Lofty parsing; audio bytes are never read
/// or decoded, MD5 is declared/unverified, and zero total samples means unknown
/// duration. Duration uses maintained-parser millisecond precision, not estimation.
/// No parser thread-global policy is changed. Container/audio-frame validity is
/// not established by this metadata-only operation.
pub fn inspect(reader: &mut (impl Read + Seek)) -> Result<Metadata, OperationError> {
    let result = (|| {
        let file_bytes = reader.seek(SeekFrom::End(0))?;
        if file_bytes > super::wave::MAX_FILE_BYTES {
            return Err(limit("metadata source exceeds 1 GiB"));
        }
        reader.seek(SeekFrom::Start(0))?;
        let mut magic = [0; 4];
        read(reader, &mut magic)?;
        if &magic != b"fLaC" {
            return Err(OperationError::new(ErrorCode::UnsupportedFormat,"only native FLAC metadata is supported; MP3, Ogg and ID3-prefixed FLAC are unsupported").operation(OP));
        }
        let mut prefix = magic.to_vec();
        let mut blocks = Vec::new();
        let mut seen_comments = false;
        loop {
            if blocks.len() == MAX_BLOCKS {
                return Err(limit("FLAC metadata exceeds 256 blocks"));
            }
            let offset = prefix.len();
            let mut header = [0; 4];
            read(reader, &mut header)?;
            let kind = header[0] & 127;
            let length = usize::from(header[1]) * 65536
                + usize::from(header[2]) * 256
                + usize::from(header[3]);
            if kind > 6 {
                return Err(OperationError::new(
                    ErrorCode::UnsupportedFormat,
                    "reserved/unsupported FLAC metadata block type",
                )
                .operation(OP));
            }
            if blocks.is_empty() {
                if kind != 0 || length != 34 {
                    return Err(invalid("first FLAC block must be exact 34-byte STREAMINFO"));
                }
            } else if kind == 0 {
                return Err(invalid("duplicate FLAC STREAMINFO"));
            }
            if kind == 4 {
                if seen_comments {
                    return Err(invalid("duplicate Vorbis-comment blocks are unsupported"));
                }
                seen_comments = true;
            }
            let end = offset
                .checked_add(4)
                .and_then(|n| n.checked_add(length))
                .ok_or_else(|| limit("metadata extent overflow"))?;
            if end > MAX_METADATA {
                return Err(limit("FLAC metadata exceeds 1 MiB"));
            }
            if end as u64 > file_bytes {
                return Err(invalid("FLAC metadata extent exceeds source"));
            }
            prefix.extend(header);
            prefix.resize(end, 0);
            read(reader, &mut prefix[offset + 4..end])?;
            if kind == 4 {
                comments(&prefix[offset + 4..end])?
            }
            blocks.push(Block {
                offset: offset as u64,
                block_type: kind,
                length,
                interpreted: kind == 0 || kind == 4,
            });
            if header[0] & 128 != 0 {
                break;
            }
        }
        let min_block = u16::from_be_bytes(prefix[8..10].try_into().expect("STREAMINFO"));
        let max_block = u16::from_be_bytes(prefix[10..12].try_into().expect("STREAMINFO"));
        if min_block < 16 || max_block < min_block {
            return Err(invalid("invalid FLAC block-size bounds"));
        }
        let total =
            u64::from_be_bytes(prefix[18..26].try_into().expect("STREAMINFO")) & ((1u64 << 36) - 1);
        let options = ParseOptions::new()
            .parsing_mode(ParsingMode::Strict)
            .read_cover_art(false)
            .implicit_conversions(false)
            .max_junk_bytes(0);
        let parsed = FlacFile::read_from(&mut Cursor::new(&prefix), options).map_err(|e| {
            invalid("FLAC metadata parser rejected the supported profile").caused_by(e)
        })?;
        let props = parsed.properties();
        if props.sample_rate() == 0 || !(4..=32).contains(&props.bit_depth()) {
            return Err(invalid("invalid FLAC sample rate or bit depth"));
        }
        let mut vendor = None;
        let mut output = Vec::new();
        if let Some(tag) = parsed.vorbis_comments() {
            let mut text_bytes = tag.vendor().len();
            for (key, value) in tag.items() {
                text_bytes = text_bytes
                    .checked_add(key.len())
                    .and_then(|n| n.checked_add(value.len()))
                    .ok_or_else(|| limit("exported tag size overflow"))?;
                if text_bytes > MAX_TEXT {
                    return Err(limit("exported FLAC text exceeds 16 KiB"));
                }
                output.push(Comment {
                    key: key.into(),
                    value: value.into(),
                });
            }
            if text_bytes > MAX_TEXT {
                return Err(limit("exported FLAC text exceeds 16 KiB"));
            }
            vendor = Some(tag.vendor().into());
        }
        Ok(Metadata {
            format: "flac",
            codec: "flac",
            file_bytes,
            metadata_bytes: prefix.len(),
            sample_rate: props.sample_rate(),
            channels: props.channels(),
            bits_per_sample: props.bit_depth(),
            declared_total_samples: (total != 0).then_some(total),
            duration_milliseconds: (total != 0).then_some(props.duration().as_millis() as u64),
            duration_source: "declared_streaminfo_millisecond_precision",
            declared_md5: (props.signature() != 0).then(|| format!("{:032x}", props.signature())),
            vendor,
            comments: output,
            blocks,
            verification: "metadata_only_audio_payload_unverified",
            artwork: "not_decoded_or_exported",
        })
    })();
    result.map_err(|e: OperationError| e.operation(OP))
}
/// Inspect a regular caller-selected local file without modifying it.
pub fn inspect_file(path: &Path) -> Result<Metadata, OperationError> {
    forensics::open_regular(path)
        .and_then(|mut file| inspect(&mut file))
        .map_err(|e| e.operation(OP))
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: OP,
    category: "audio",
    description: "Native FLAC metadata only: STREAMINFO and bounded Vorbis comments; unknown duration explicit, audio/MD5/artwork unverified, <=1 MiB metadata/256 records/16 KiB text. MP3/Ogg unsupported.",
    input_type: "byte_reader",
    output_type: "audio.metadata",
    parameters: &[],
    deterministic: true,
    reversible: None,
    command: "audio metadata",
    example: "cube audio metadata sample.flac --json",
}];
