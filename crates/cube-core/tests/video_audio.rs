#[path = "common/pcm_mov.rs"]
mod fixture;
use cube_core::{
    errors::ErrorCode,
    hashing,
    media::{pcm, video::audio, wave},
};
use std::io::{Cursor, Read, Seek, SeekFrom};
fn position(data: &[u8], kind: &[u8; 4]) -> usize {
    data.windows(4).position(|b| b == kind).unwrap()
}
#[test]
fn selected_pcm_tracks_remux_exact_bytes_channels_times_hashes_and_padding() {
    for channels in [1, 2] {
        for wide in [false, true] {
            let raw = vec![0, 255, 128, 64, 1, 254];
            let other: Vec<_> = raw.iter().copied().rev().collect();
            let data = fixture::movie(&[raw, other.clone()], channels, 8000, wide);
            let result = audio::extract(&mut Cursor::new(&data), 2).unwrap();
            let info = wave::inspect(&mut Cursor::new(&result.data)).unwrap();
            assert_eq!(info.format.channels, channels);
            assert_eq!(info.format.sample_rate, 8000);
            assert_eq!(info.format.bits_per_sample, 8);
            let block =
                pcm::extract(&mut Cursor::new(&result.data), &pcm::Selection::default()).unwrap();
            assert_eq!(block.raw(), other);
            assert_eq!(block.values()[0], i32::from(other[0]) - 128);
            let p = &result.provenance;
            assert_eq!(p.track_id, 2);
            assert_eq!(p.frames, 6 / u64::from(channels));
            assert_eq!(p.duration_ticks, p.frames);
            assert_eq!(p.time_base.denominator, 8000);
            assert_eq!(p.pcm_sha256, hashing::sha256(&other));
            assert_eq!(p.wav_sha256, hashing::sha256(&result.data));
            let copied: Vec<_> = p
                .source_ranges
                .iter()
                .flat_map(|r| {
                    data[r.offset as usize..(r.offset + r.length) as usize]
                        .iter()
                        .copied()
                })
                .collect();
            assert_eq!(copied, other);
        }
    }
    let result = audio::extract(
        &mut Cursor::new(fixture::movie(&[vec![0, 128, 255]], 1, 8000, false)),
        1,
    )
    .unwrap();
    assert_eq!(result.data.len(), 48);
    assert_eq!(result.data[47], 0);
    assert_eq!(
        wave::inspect(&mut Cursor::new(result.data))
            .unwrap()
            .data_bytes,
        3
    );
    assert!(
        cube_core::operations::require("video.audio.extract")
            .unwrap()
            .discovery()
            .pipeline
            .is_none()
    );
}
#[test]
fn chunk_order_is_explicit_and_only_selected_payload_is_read() {
    let raw = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
    let mut data = fixture::chunked(std::slice::from_ref(&raw), 2, 8000, false, Some(&[3, 3]));
    let p = position(&data, b"stco") + 12;
    let a = data[p..p + 4].to_vec();
    let b = data[p + 4..p + 8].to_vec();
    data[p..p + 4].copy_from_slice(&b);
    data[p + 4..p + 8].copy_from_slice(&a);
    let result = audio::extract(&mut Cursor::new(&data), 1).unwrap();
    let samples = pcm::extract(&mut Cursor::new(result.data), &pcm::Selection::default()).unwrap();
    assert_eq!(samples.raw(), [&raw[6..], &raw[..6]].concat());
    assert_eq!(result.provenance.source_ranges.len(), 2);
    struct Observed {
        source: Cursor<Vec<u8>>,
        end: u64,
        payload_reads: usize,
    }
    impl Read for Observed {
        fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
            let at = self.source.position();
            let n = self.source.read(b)?;
            if at >= 24 && at < self.end {
                self.payload_reads += n;
            }
            Ok(n)
        }
    }
    impl Seek for Observed {
        fn seek(&mut self, p: SeekFrom) -> std::io::Result<u64> {
            self.source.seek(p)
        }
    }
    let data = fixture::movie(&[raw.clone(), raw.clone()], 2, 8000, false);
    let end = 24 + raw.len() as u64 * 2;
    let mut reader = Observed {
        source: Cursor::new(data.clone()),
        end,
        payload_reads: 0,
    };
    audio::extract(&mut reader, 2).unwrap();
    assert_eq!(reader.payload_reads, raw.len());
    let mut bad = data;
    let p = position(&bad, b"stco") + 12;
    bad[p..p + 4].copy_from_slice(&0u32.to_be_bytes());
    let mut reader = Observed {
        source: Cursor::new(bad),
        end,
        payload_reads: 0,
    };
    assert!(audio::extract(&mut reader, 1).is_err());
    assert_eq!(reader.payload_reads, 0);
}
#[test]
fn unsupported_formats_bad_tables_external_references_and_limits_fail_before_copy() {
    let data = fixture::chunked(&[vec![0; 12]], 2, 8000, false, Some(&[3, 3]));
    for (kind, relative, value) in [
        (b"stsz", 8, 1),
        (b"stts", 16, 2),
        (b"stsc", 16, 99),
        (b"stco", 12, 0),
        (b"stco", 16, 24),
        (b"url ", 4, 0),
    ] {
        let mut bad = data.clone();
        let p = position(&bad, kind) + relative;
        bad[p..p + 4].copy_from_slice(&(value as u32).to_be_bytes());
        assert_eq!(
            audio::extract(&mut Cursor::new(bad), 1)
                .unwrap_err()
                .operation,
            Some("video.audio.extract")
        );
    }
    for (relative, value) in [(20, 3u16), (22, 16), (24, 1), (26, 1), (28, 0), (12, 1)] {
        let mut bad = data.clone();
        let p = position(&bad, b"raw ") + relative;
        bad[p..p + 2].copy_from_slice(&value.to_be_bytes());
        assert!(audio::extract(&mut Cursor::new(bad), 1).is_err());
    }
    let mut bad = data.clone();
    let p = position(&bad, b"raw ");
    bad[p..p + 4].copy_from_slice(b"sowt");
    assert_eq!(
        audio::extract(&mut Cursor::new(bad), 1).unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
    for id in [0, 2, u64::MAX] {
        assert!(audio::extract(&mut Cursor::new(&data), id).is_err());
    }
    let mut bad = data.clone();
    let p = position(&bad, b"stsz") + 12;
    bad[p..p + 4].copy_from_slice(&(pcm::MAX_FRAMES as u32 + 1).to_be_bytes());
    assert_eq!(
        audio::extract(&mut Cursor::new(bad), 1).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let bad = fixture::movie(&[vec![0; 301]], 1, 1, false);
    assert_eq!(
        audio::extract(&mut Cursor::new(bad), 1).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    for n in [0, 7, data.len() / 2, data.len() - 1] {
        assert!(audio::extract(&mut Cursor::new(&data[..n]), 1).is_err());
    }
}
#[test]
fn fixed_pcm_tables_support_exact_maximum_frames_without_per_frame_reads() {
    let raw = vec![128; pcm::MAX_FRAMES as usize];
    let data = fixture::movie(std::slice::from_ref(&raw), 1, 8000, true);
    let result = audio::extract(&mut Cursor::new(data), 1).unwrap();
    assert_eq!(result.provenance.frames, pcm::MAX_FRAMES);
    assert_eq!(result.provenance.source_ranges.len(), 1);
    assert_eq!(
        pcm::extract(&mut Cursor::new(result.data), &pcm::Selection::default())
            .unwrap()
            .raw(),
        raw
    );
}
