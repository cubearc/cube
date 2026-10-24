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

fn encode_values(values: &[i32], bits: u16) -> Vec<u8> {
    values
        .iter()
        .flat_map(|v| {
            if bits == 8 {
                vec![(v + 128) as u8]
            } else {
                v.to_le_bytes()[..usize::from(bits / 8)].to_vec()
            }
        })
        .collect()
}
#[test]
fn pcm_extraction_preserves_all_widths_channel_order_and_signed_amplitudes() {
    use cube_core::media::pcm::{self, Selection};
    for bits in [8, 16, 24, 32] {
        let min = if bits == 32 {
            i32::MIN
        } else {
            -(1i32 << (bits - 1))
        };
        let max = if bits == 32 {
            i32::MAX
        } else {
            (1i32 << (bits - 1)) - 1
        };
        let source = [min, max, 0, -1, max, min];
        let raw = encode_values(&source, bits);
        let input = wav(2, bits, &raw);
        let block = pcm::extract(&mut Cursor::new(&input), &Selection::default()).unwrap();
        assert_eq!(block.raw(), raw);
        assert_eq!(block.values(), source);
        assert_eq!(block.frames(), 3);
        assert_eq!(block.info().sample_rate, 8000);
        let selected = pcm::extract(
            &mut Cursor::new(&input),
            &Selection {
                start_frame: 1,
                frames: Some(2),
                channels: Some(vec![1, 0]),
            },
        )
        .unwrap();
        assert_eq!(selected.values(), [-1, 0, min, max]);
        assert_eq!(selected.raw(), encode_values(&[-1, 0, min, max], bits));
        assert_eq!(selected.channels(), [1, 0]);
        assert_eq!(selected.start_frame(), 1);
        assert_eq!(selected.info().frame_bytes, usize::from(bits / 8) * 2);
        let empty = pcm::extract(
            &mut Cursor::new(&input),
            &Selection {
                start_frame: 3,
                frames: Some(0),
                channels: Some(vec![1]),
            },
        )
        .unwrap();
        assert!(empty.raw().is_empty());
        assert!(empty.values().is_empty());
    }
    let source = encode_values(&[1, -2].repeat(12000), 24);
    let input = wav(2, 24, &source);
    let block = pcm::extract(
        &mut Cursor::new(input),
        &Selection {
            channels: Some(vec![1]),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(block.raw(), encode_values(&[-2; 12000], 24));
    let workspace = common::Workspace::new();
    let input = workspace.path("input.wav");
    let output = workspace.path("samples.pcm");
    let original = wav(1, 8, &[0, 128, 255]);
    std::fs::write(&input, &original).unwrap();
    let block = pcm::extract_file(&input, &Selection::default()).unwrap();
    block.save(&output).unwrap();
    assert_eq!(std::fs::read(&output).unwrap(), [0, 128, 255]);
    assert!(block.save(&output).is_err());
    assert!(block.save(&input).is_err());
    assert_eq!(std::fs::read(&input).unwrap(), original);
}
#[test]
fn pcm_ranges_are_validated_before_reading_or_large_allocation() {
    use cube_core::media::pcm::{self, Selection};
    let input = wav(2, 16, &[0; 12]);
    for selection in [
        Selection {
            channels: Some(vec![]),
            ..Default::default()
        },
        Selection {
            channels: Some(vec![0, 0]),
            ..Default::default()
        },
        Selection {
            channels: Some(vec![2]),
            ..Default::default()
        },
        Selection {
            start_frame: 4,
            ..Default::default()
        },
        Selection {
            start_frame: u64::MAX,
            frames: Some(1),
            ..Default::default()
        },
        Selection {
            start_frame: 1,
            frames: Some(3),
            ..Default::default()
        },
    ] {
        assert_eq!(
            pcm::extract(&mut Cursor::new(&input), &selection)
                .unwrap_err()
                .code,
            ErrorCode::InvalidParameter
        );
    }
    let input = wav(1, 16, &vec![0; 200000]);
    let mut reader = Counted {
        inner: Cursor::new(input),
        read_bytes: 0,
    };
    let block = pcm::extract(
        &mut reader,
        &Selection {
            start_frame: 99998,
            frames: Some(2),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(block.raw(), [0; 4]);
    assert_eq!(reader.read_bytes, 48);
    let input = wav(1, 8, &[128; 10001]);
    let block = pcm::extract(&mut Cursor::new(input), &Selection::default()).unwrap();
    assert_eq!(
        block.bounded_values().unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let workspace = common::Workspace::new();
    for (name, channels, bits, frames) in [
        ("frames", 1u16, 8u16, pcm::MAX_FRAMES + 1),
        ("output", 3, 32, pcm::MAX_FRAMES),
        ("samples", 3, 8, pcm::MAX_FRAMES),
        ("source", 32, 32, pcm::MAX_FRAMES),
    ] {
        let path = workspace.path(name);
        let bytes = frames * u64::from(channels) * u64::from(bits / 8);
        let mut header = wav(channels, bits, &[]);
        header[4..8].copy_from_slice(&((36 + bytes + bytes % 2) as u32).to_le_bytes());
        header[40..44].copy_from_slice(&(bytes as u32).to_le_bytes());
        std::fs::write(&path, header).unwrap();
        std::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_len(44 + bytes + bytes % 2)
            .unwrap();
        assert_eq!(
            pcm::extract_file(&path, &Selection::default())
                .unwrap_err()
                .code,
            ErrorCode::ResourceLimit
        );
    }
    let mut slow = wav(1, 8, &[128; 301]);
    slow[24..28].copy_from_slice(&1u32.to_le_bytes());
    slow[28..32].copy_from_slice(&1u32.to_le_bytes());
    assert_eq!(
        pcm::extract(&mut Cursor::new(slow), &Selection::default())
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn waveform_extrema_have_absolute_frames_and_keep_final_partial_buckets() {
    use cube_core::media::{
        envelope,
        pcm::{self, Selection},
    };
    let values = [0, 4, 32767, -4, -32768, 20, 5, -20, -1, 7, 10, 8, 0, -9];
    let input = wav(2, 16, &encode_values(&values, 16));
    let samples = pcm::extract(&mut Cursor::new(&input), &Selection::default()).unwrap();
    let all = envelope::from_samples(&samples, 3, 100).unwrap();
    let stream =
        envelope::from_reader(&mut Cursor::new(&input), &Selection::default(), 3, 100).unwrap();
    assert_eq!(all.buckets, stream.buckets);
    assert_eq!(all.frames, 7);
    assert_eq!(all.buckets.len(), 3);
    assert_eq!(all.buckets[0].minimum, [-32768, -4]);
    assert_eq!(all.buckets[0].maximum, [32767, 20]);
    assert_eq!(all.buckets[1].minimum, [-1, -20]);
    assert_eq!(all.buckets[1].maximum, [10, 8]);
    assert_eq!(all.buckets[2].start_frame, 6);
    assert_eq!(all.buckets[2].frames, 1);
    assert_eq!(all.buckets[2].minimum, [0, -9]);
    let selection = Selection {
        start_frame: 1,
        frames: Some(5),
        channels: Some(vec![1, 0]),
    };
    let selected = pcm::extract(&mut Cursor::new(&input), &selection).unwrap();
    let expected = envelope::from_samples(&selected, 3, 100).unwrap();
    assert_eq!(expected.buckets[0].start_frame, 1);
    assert_eq!(expected.buckets[0].minimum, [-20, -32768]);
    assert_eq!(expected.buckets[1].start_frame, 4);
    assert_eq!(expected.buckets[1].frames, 2);
    let first = pcm::extract(
        &mut Cursor::new(&input),
        &Selection {
            frames: Some(1),
            ..selection.clone()
        },
    )
    .unwrap();
    let mut builder = envelope::Builder::new(&first, 3, 100).unwrap();
    builder.push(&first).unwrap();
    for (start, frames) in [(2, 3), (5, 1)] {
        let block = pcm::extract(
            &mut Cursor::new(&input),
            &Selection {
                start_frame: start,
                frames: Some(frames),
                channels: Some(vec![1, 0]),
            },
        )
        .unwrap();
        builder.push(&block).unwrap();
    }
    assert_eq!(builder.finish().buckets, expected.buckets);
    for (values, bucket, minimum, maximum) in [
        (vec![5; 5], 2, vec![5, 5, 5], vec![5, 5, 5]),
        (vec![0, 0, 127, 0, 0], 2, vec![0, 0, 0], vec![0, 127, 0]),
        (
            vec![0, 70, 100, 70, 0, -70, -100, -70],
            4,
            vec![0, -100],
            vec![100, 0],
        ),
        (vec![-128, 127], 1, vec![-128, 127], vec![-128, 127]),
    ] {
        let input = wav(1, 8, &encode_values(&values, 8));
        let report =
            envelope::from_reader(&mut Cursor::new(input), &Selection::default(), bucket, 100)
                .unwrap();
        assert_eq!(
            report
                .buckets
                .iter()
                .map(|b| b.minimum[0])
                .collect::<Vec<_>>(),
            minimum
        );
        assert_eq!(
            report
                .buckets
                .iter()
                .map(|b| b.maximum[0])
                .collect::<Vec<_>>(),
            maximum
        );
    }
    assert!(
        envelope::from_reader(
            &mut Cursor::new(wav(1, 8, &[])),
            &Selection::default(),
            1024,
            100
        )
        .unwrap()
        .buckets
        .is_empty()
    );
    let report =
        envelope::from_reader(&mut Cursor::new(&input), &Selection::default(), 100, 100).unwrap();
    assert_eq!(report.buckets.len(), 1);
    assert_eq!(report.buckets[0].frames, 7);
}
#[test]
fn waveform_streaming_crosses_io_boundaries_without_changing_buckets() {
    use cube_core::media::{
        envelope,
        pcm::{self, Selection},
    };
    let data = encode_values(&[1, -2, -1, 2].repeat(20000), 16);
    let input = wav(2, 16, &data);
    let expected =
        envelope::from_reader(&mut Cursor::new(&input), &Selection::default(), 30000, 100).unwrap();
    assert_eq!(expected.frames, 40000);
    assert_eq!(expected.buckets[1].frames, 10000);
    assert_eq!(expected.buckets[0].minimum, [-1, -2]);
    assert_eq!(expected.buckets[0].maximum, [1, 2]);
    let initial = pcm::extract(
        &mut Cursor::new(&input),
        &Selection {
            frames: Some(0),
            ..Default::default()
        },
    )
    .unwrap();
    let mut builder = envelope::Builder::new(&initial, 30000, 100).unwrap();
    let mut start = 0;
    for frames in [100, 16284, 1, 10000, 13615] {
        let block = pcm::extract(
            &mut Cursor::new(&input),
            &Selection {
                start_frame: start,
                frames: Some(frames),
                ..Default::default()
            },
        )
        .unwrap();
        builder.push(&block).unwrap();
        start += frames;
    }
    assert_eq!(builder.finish().buckets, expected.buckets);
    let workspace = common::Workspace::new();
    let path = workspace.path("waveform.wav");
    std::fs::write(&path, &input).unwrap();
    assert_eq!(
        envelope::from_file(&path, &Selection::default(), 30000, 100)
            .unwrap()
            .buckets,
        expected.buckets
    );
    assert_eq!(std::fs::read(&path).unwrap(), input);
}
#[test]
fn waveform_validates_metadata_continuity_and_budgets_before_reading() {
    use cube_core::media::{
        envelope,
        pcm::{self, Selection},
    };
    let input = wav(1, 8, &[128; 10]);
    let samples = pcm::extract(
        &mut Cursor::new(&input),
        &Selection {
            frames: Some(2),
            ..Default::default()
        },
    )
    .unwrap();
    for (bucket, max) in [(0, 1), (u64::MAX, 1), (1, 0), (1, 10001)] {
        assert_eq!(
            envelope::Builder::new(&samples, bucket, max)
                .err()
                .unwrap()
                .code,
            ErrorCode::InvalidParameter
        );
    }
    let mut builder = envelope::Builder::new(&samples, 3, 100).unwrap();
    builder.push(&samples).unwrap();
    assert!(builder.push(&samples).is_err());
    let different = pcm::extract(
        &mut Cursor::new(wav(2, 8, &[128; 20])),
        &Selection {
            start_frame: 2,
            frames: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(builder.push(&different).is_err());
    assert_eq!(builder.finish().frames, 2);
    let input = wav(2, 16, &vec![0; 4 * 6000]);
    let mut counted = Counted {
        inner: Cursor::new(input),
        read_bytes: 0,
    };
    assert_eq!(
        envelope::from_reader(&mut counted, &Selection::default(), 1, 10000)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(counted.read_bytes, 44);
    let many = wav(1, 8, &vec![128; (pcm::MAX_FRAMES + 1) as usize]);
    let first = pcm::extract(
        &mut Cursor::new(&many),
        &Selection {
            frames: Some(pcm::MAX_FRAMES),
            ..Default::default()
        },
    )
    .unwrap();
    let next = pcm::extract(
        &mut Cursor::new(&many),
        &Selection {
            start_frame: pcm::MAX_FRAMES,
            frames: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    let mut builder = envelope::Builder::new(&first, pcm::MAX_FRAMES, 100).unwrap();
    builder.push(&first).unwrap();
    assert_eq!(
        builder.push(&next).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        envelope::from_reader(&mut Cursor::new(b"bad"), &Selection::default(), 1, 100)
            .unwrap_err()
            .operation,
        Some("audio.waveform.envelope")
    );
}
