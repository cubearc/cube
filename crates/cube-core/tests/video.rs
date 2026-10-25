#[path = "common/video.rs"]
mod fixture;
use cube_core::{errors::ErrorCode, media::video};
use std::io::{self, Cursor, Read, Seek, SeekFrom};
#[test]
fn declared_mp4_mov_and_webm_metadata_is_typed_exact_and_unverified() {
    for version in [false, true] {
        let data = fixture::mp4(2, version);
        let info = video::inspect(&mut Cursor::new(&data)).unwrap();
        assert_eq!(info.container, "mp4");
        assert_eq!(info.tracks.len(), 2);
        assert_eq!(info.duration_seconds, Some(2.0));
        assert_eq!(info.tracks[0].time_base.unwrap().denominator, 48000);
        assert_eq!(info.tracks[0].duration_seconds, Some(2.0));
        assert_eq!(info.tracks[0].codec_id.as_deref(), Some("avc1"));
        assert_eq!(
            (info.tracks[0].width, info.tracks[0].height),
            (Some(1280), Some(720))
        );
        assert_eq!(
            info.tracks[0].display_dimensions_fixed_16_16,
            Some([1920 << 16, 1080 << 16])
        );
        assert!(info.verification.contains("unverified"));
        let mut mov = data;
        mov[8..12].copy_from_slice(b"qt  ");
        assert_eq!(
            video::inspect(&mut Cursor::new(mov)).unwrap().container,
            "mov"
        );
    }
    let r = video::inspect(&mut Cursor::new(fixture::webm())).unwrap();
    assert_eq!(r.container, "webm");
    assert_eq!(r.duration_seconds, Some(2.5));
    assert_eq!(r.tracks[0].codec_id.as_deref(), Some("V_VP9"));
    assert_eq!(
        (r.tracks[0].width, r.tracks[0].height),
        (Some(640), Some(480))
    );
    assert!(
        cube_core::operations::require("video.container.inspect")
            .unwrap()
            .discovery()
            .pipeline
            .is_none()
    );
}
#[test]
fn malformed_extents_truncation_versions_and_unsupported_profiles_fail_safely() {
    for data in [fixture::mp4(1, false), fixture::webm()] {
        for n in 0..data.len() {
            let result = video::inspect(&mut Cursor::new(&data[..n]));
            if let Ok(info) = result {
                assert!(info.verification.contains("unverified"));
            }
        }
    }
    let mut bad = fixture::mp4(1, false);
    bad[..4].copy_from_slice(&7u32.to_be_bytes());
    assert!(video::inspect(&mut Cursor::new(bad)).is_err());
    let mut bad = fixture::mp4(1, false);
    bad.extend(fixture::atom(b"moof", &[]));
    assert_eq!(
        video::inspect(&mut Cursor::new(bad)).unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
    let mut bad = fixture::mp4(1, false);
    let p = bad.windows(4).position(|b| b == b"avc1").unwrap();
    bad[p..p + 4].copy_from_slice(b"encv");
    assert_eq!(
        video::inspect(&mut Cursor::new(bad)).unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
    let mut bad = fixture::mp4(1, false);
    let p = bad.windows(4).position(|b| b == b"mdhd").unwrap() + 4;
    bad[p + 12..p + 16].fill(0);
    assert!(video::inspect(&mut Cursor::new(bad)).is_err());
    assert!(video::inspect(&mut Cursor::new(b"notvideo")).is_err());
    assert!(video::inspect(&mut Cursor::new([0x1a, 0x45, 0xdf, 0xa3, 0, 0, 0, 0])).is_err());
    let mut bad = fixture::webm();
    let p = bad.windows(4).position(|b| b == b"webm").unwrap();
    bad[p..p + 4].copy_from_slice(b"mkv!");
    assert_eq!(
        video::inspect(&mut Cursor::new(bad)).unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
}
#[test]
fn limits_large_sizes_unknown_payloads_and_mutations_never_escape_bounds() {
    let mut b = Vec::new();
    for _ in 0..10001 {
        b.extend(fixture::atom(b"free", &[]));
    }
    assert_eq!(
        video::inspect(&mut Cursor::new(b)).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        video::inspect(&mut Cursor::new(fixture::mp4(33, false)))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let mut b = 1u32.to_be_bytes().to_vec();
    b.extend(b"ftyp");
    b.extend(u64::MAX.to_be_bytes());
    assert!(video::inspect(&mut Cursor::new(b)).is_err());
    struct Oversize;
    impl Read for Oversize {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            panic!("must not read oversized source")
        }
    }
    impl Seek for Oversize {
        fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
            Ok(video::MAX_FILE_BYTES + 1)
        }
    }
    assert_eq!(
        video::inspect(&mut Oversize).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    for data in [fixture::mp4(1, false), fixture::webm()] {
        for n in 0..data.len() {
            let mut mutated = data.clone();
            mutated[n] ^= 255;
            let _ = video::inspect(&mut Cursor::new(mutated));
        }
    }
}
#[test]
fn audio_tracks_unknown_durations_large_headers_and_webm_edge_cases_are_explicit() {
    let mut b = fixture::mp4(1, false);
    let p = b.windows(4).position(|b| b == b"vide").unwrap();
    b[p..p + 4].copy_from_slice(b"soun");
    let p = b.windows(4).position(|b| b == b"avc1").unwrap();
    b[p..p + 4].copy_from_slice(b"mp4a");
    let body = p + 4;
    b[body + 16..body + 18].copy_from_slice(&2u16.to_be_bytes());
    b[body + 24..body + 28].copy_from_slice(&(44100u32 << 16).to_be_bytes());
    let info = video::inspect(&mut Cursor::new(b)).unwrap();
    assert_eq!(info.tracks[0].kind, "audio");
    assert_eq!(info.tracks[0].channels, Some(2));
    assert_eq!(info.tracks[0].sample_rate, Some(44100.0));
    assert!(info.tracks[0].width.is_none());
    let mut b = fixture::mp4(1, true);
    let p = b.windows(4).position(|b| b == b"mvhd").unwrap() + 4;
    b[p + 24..p + 32].copy_from_slice(&u64::MAX.to_be_bytes());
    assert!(
        video::inspect(&mut Cursor::new(b))
            .unwrap()
            .duration_ticks
            .is_none()
    );
    let b = fixture::mp4(1, false);
    let first = u32::from_be_bytes(b[..4].try_into().unwrap()) as usize;
    let mut extended = 1u32.to_be_bytes().to_vec();
    extended.extend(b"ftyp");
    extended.extend((first as u64 + 8).to_be_bytes());
    extended.extend(&b[8..]);
    assert!(video::inspect(&mut Cursor::new(extended)).is_ok());
    let mut b = fixture::webm();
    let p = b.windows(2).position(|b| b == [0x44, 0x89]).unwrap() + 3;
    b[p..p + 8].copy_from_slice(&f64::NAN.to_be_bytes());
    assert!(video::inspect(&mut Cursor::new(b)).is_err());
    let mut b = fixture::webm();
    let p = b
        .windows(4)
        .position(|b| b == [0x1f, 0x43, 0xb6, 0x75])
        .unwrap()
        + 4;
    b[p] = 255;
    assert_eq!(
        video::inspect(&mut Cursor::new(b)).unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
}
#[test]
fn reserved_ebml_ids_and_incomplete_standard_time_headers_are_rejected() {
    let mut b = fixture::webm();
    let p = b.windows(2).position(|b| b == [0xd7, 0x81]).unwrap();
    b[p] = 255;
    assert!(video::inspect(&mut Cursor::new(b)).is_err());
    let mut b = fixture::mp4(1, false);
    let p = b.windows(4).position(|b| b == b"mvhd").unwrap();
    b[p - 4..p].copy_from_slice(&28u32.to_be_bytes());
    assert!(video::inspect(&mut Cursor::new(b)).is_err());
}
