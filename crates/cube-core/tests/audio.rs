mod common;
use cube_core::{errors::ErrorCode, media::wave};
use std::io::{Cursor, Read, Seek, SeekFrom};
fn chunk(id: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut result = id.to_vec();
    result.extend((data.len() as u32).to_le_bytes());
    result.extend(data);
    if !data.len().is_multiple_of(2) {
        result.push(0);
    }
    result
}
fn format(channels: u16, bits: u16, rate: u32) -> Vec<u8> {
    let align = channels * (bits / 8);
    let mut result = 1u16.to_le_bytes().to_vec();
    result.extend(channels.to_le_bytes());
    result.extend(rate.to_le_bytes());
    result.extend((rate * u32::from(align)).to_le_bytes());
    result.extend(align.to_le_bytes());
    result.extend(bits.to_le_bytes());
    result
}
fn container(chunks: &[Vec<u8>]) -> Vec<u8> {
    let mut result = b"RIFF".to_vec();
    result.extend((4 + chunks.iter().map(Vec::len).sum::<usize>() as u32).to_le_bytes());
    result.extend(b"WAVE");
    for c in chunks {
        result.extend(c);
    }
    result
}
fn wav(channels: u16, bits: u16, data: &[u8]) -> Vec<u8> {
    container(&[
        chunk(b"fmt ", &format(channels, bits, 8000)),
        chunk(b"data", data),
    ])
}
struct Counted {
    inner: Cursor<Vec<u8>>,
    read_bytes: usize,
}
impl Read for Counted {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(bytes)?;
        self.read_bytes += n;
        Ok(n)
    }
}
impl Seek for Counted {
    fn seek(&mut self, from: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(from)
    }
}
#[test]
fn wave_inspection_preserves_metadata_and_skips_sample_payloads() {
    for bits in [8, 16, 24, 32] {
        for channels in [1, 2, 32] {
            let input = wav(
                channels,
                bits,
                &vec![0; usize::from(channels) * usize::from(bits / 8) * 3],
            );
            let info = wave::inspect(&mut Cursor::new(&input)).unwrap();
            assert_eq!(info.frames, 3);
            assert_eq!(info.format.channels, channels);
            assert_eq!(info.format.bits_per_sample, bits);
            assert_eq!(info.duration_seconds.numerator, 3);
            assert_eq!(info.duration_seconds.denominator, 8000);
            assert_eq!(info.data_offset, 44);
            assert_eq!(info.file_bytes, input.len() as u64);
            assert_eq!(info.chunks[1].id, "data");
        }
    }
    let mut fmt = format(1, 8, 8000);
    fmt.extend([0, 0]);
    let mut input = container(&[
        chunk(b"JUNK", b"abc"),
        chunk(b"fmt ", &fmt),
        chunk(b"LIST", b"INFO"),
        chunk(b"data", &[0, 128, 255]),
        chunk(b"note", b"tail"),
    ]);
    input.extend(b"trailer");
    let info = wave::inspect(&mut Cursor::new(input)).unwrap();
    assert_eq!(info.trailing_bytes, 7);
    assert_eq!(info.chunks[0].padding_bytes, 1);
    assert_eq!(info.chunks[0].offset, 12);
    assert_eq!(info.chunks[1].offset, 24);
    assert_eq!(info.frames, 3);
    let input = wav(2, 16, &vec![0; 4 * 1024 * 1024]);
    let mut reader = Counted {
        inner: Cursor::new(input),
        read_bytes: 0,
    };
    let info = wave::inspect(&mut reader).unwrap();
    assert_eq!(info.frames, 1024 * 1024);
    assert_eq!(reader.read_bytes, 44);
    assert_eq!(
        wave::inspect(&mut Cursor::new(wav(1, 8, &[])))
            .unwrap()
            .frames,
        0
    );
    let workspace = common::Workspace::new();
    let path = workspace.path("audio.wav");
    let input = wav(1, 8, &[0, 128, 255]);
    std::fs::write(&path, &input).unwrap();
    assert_eq!(wave::inspect_file(&path).unwrap().frames, 3);
    assert_eq!(std::fs::read(path).unwrap(), input);
}
#[test]
fn wave_rejects_bad_framing_formats_alignment_and_limits() {
    for marker in [b"RF64", b"RIFX"] {
        let mut input = wav(1, 8, &[]);
        input[..4].copy_from_slice(marker);
        assert_eq!(
            wave::inspect(&mut Cursor::new(input)).unwrap_err().code,
            ErrorCode::UnsupportedFormat
        );
    }
    for tag in [3u16, 6, 65534] {
        let mut fmt = format(1, 16, 8000);
        fmt[..2].copy_from_slice(&tag.to_le_bytes());
        assert_eq!(
            wave::inspect(&mut Cursor::new(container(&[
                chunk(b"fmt ", &fmt),
                chunk(b"data", &[])
            ])))
            .unwrap_err()
            .code,
            ErrorCode::UnsupportedFormat
        );
    }
    for (offset, bytes) in [
        (2, 0u16.to_le_bytes().to_vec()),
        (4, 0u32.to_le_bytes().to_vec()),
        (8, 1u32.to_le_bytes().to_vec()),
        (12, 1u16.to_le_bytes().to_vec()),
        (14, 12u16.to_le_bytes().to_vec()),
    ] {
        let mut fmt = format(2, 16, 8000);
        fmt[offset..offset + bytes.len()].copy_from_slice(&bytes);
        assert!(
            wave::inspect(&mut Cursor::new(container(&[
                chunk(b"fmt ", &fmt),
                chunk(b"data", &[])
            ])))
            .is_err()
        );
    }
    for input in [
        vec![],
        b"not a WAVE file".to_vec(),
        container(&[]),
        container(&[chunk(b"fmt ", &format(1, 16, 8000))]),
        container(&[chunk(b"data", &[]), chunk(b"fmt ", &format(1, 8, 8000))]),
        container(&[chunk(b"fmt ", &format(1, 16, 8000)), chunk(b"data", &[0])]),
        container(&[
            chunk(b"fmt ", &format(1, 8, 8000)),
            chunk(b"fmt ", &format(1, 8, 8000)),
            chunk(b"data", &[]),
        ]),
    ] {
        assert!(wave::inspect(&mut Cursor::new(input)).is_err());
    }
    let mut truncated = wav(1, 8, &[1]);
    truncated.pop();
    assert!(wave::inspect(&mut Cursor::new(truncated)).is_err());
    let mut forged = wav(1, 8, &[1, 2]);
    forged[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(wave::inspect(&mut Cursor::new(forged)).is_err());
    let fmt = chunk(b"fmt ", &format(1, 8, 8000));
    assert!(
        wave::inspect(&mut Cursor::new(container(&[
            fmt.clone(),
            chunk(b"data", &[]),
            chunk(b"data", &[])
        ])))
        .is_err()
    );
    assert!(
        wave::inspect(&mut Cursor::new(container(&[
            fmt.clone(),
            chunk(b"LIST", b"wavl"),
            chunk(b"data", &[])
        ])))
        .is_err()
    );
    let mut chunks = vec![fmt];
    chunks.extend((0..wave::MAX_CHUNKS).map(|_| chunk(b"JUNK", &[])));
    assert_eq!(
        wave::inspect(&mut Cursor::new(container(&chunks)))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let workspace = common::Workspace::new();
    let path = workspace.path("large.wav");
    std::fs::File::create(&path)
        .unwrap()
        .set_len(wave::MAX_FILE_BYTES + 1)
        .unwrap();
    assert_eq!(
        wave::inspect_file(&path).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}
