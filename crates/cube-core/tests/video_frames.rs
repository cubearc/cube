#[path = "common/mov.rs"]
mod fixture;
use cube_core::{errors::ErrorCode, hashing, image::Image, media::video::frames};
use std::io::Cursor;
fn extract(
    data: &[u8],
    indexes: &[usize],
) -> Result<Vec<frames::Frame>, cube_core::errors::OperationError> {
    frames::extract(&mut Cursor::new(data), 1, indexes)
}
#[test]
fn selected_jpegs_preserve_order_exact_decoder_pixels_ranges_times_and_hashes() {
    let originals = vec![fixture::jpeg(0), fixture::jpeg(1), fixture::jpeg(0)];
    for wide in [false, true] {
        for fixed in [false, true] {
            let data = fixture::movie(&originals, wide, fixed);
            let result = extract(&data, &[2, 0, 1]).unwrap();
            for (frame, &index) in result.iter().zip(&[2, 0, 1]) {
                let p = &frame.provenance;
                assert_eq!(p.frame_index, index);
                assert_eq!(p.track_id, 1);
                assert_eq!(p.decode_start_ticks, [0, 10, 30][index]);
                assert_eq!(p.duration_ticks, (index as u32 + 1) * 10);
                assert_eq!(p.time_base.denominator, 1000);
                assert_eq!(
                    &data[p.offset as usize..p.offset as usize + p.length as usize],
                    originals[index]
                );
                assert_eq!(p.encoded_sha256, hashing::sha256(&originals[index]));
                let expected = Image::decode(&originals[index]).unwrap();
                assert_eq!(frame.image.samples(), expected.samples());
                assert!(!frame.image.provenance().source_samples_preserved);
                assert_eq!(
                    Image::decode(&frame.image.encode_png().unwrap())
                        .unwrap()
                        .samples(),
                    expected.samples()
                );
            }
        }
    }
    let mut originals = originals;
    originals[1] = b"not a JPEG".to_vec();
    let data = fixture::movie(&originals, false, false);
    assert!(extract(&data, &[0]).is_ok()); // Unselected media is indexed, never decoded.
    assert!(extract(&data, &[1]).is_err());
    assert!(
        cube_core::operations::require("video.frames.extract")
            .unwrap()
            .discovery()
            .pipeline
            .is_none()
    );
    let originals = vec![fixture::jpeg(0), fixture::jpeg(1), fixture::jpeg(0)];
    let data = fixture::movie_with_chunks(&originals, true, false, &[2, 1], true);
    let frames = extract(&data, &[2, 1, 0]).unwrap();
    assert_eq!(frames[1].provenance.decode_start_ticks, 10);
    assert_eq!(
        frames[1].image.samples(),
        Image::decode(&originals[1]).unwrap().samples()
    );
    let mut bad = data.clone();
    let p = position(&bad, b"stss") + 16;
    bad[p..p + 4].copy_from_slice(&1u32.to_be_bytes());
    assert!(extract(&bad, &[0]).is_err());
}
#[test]
fn malformed_relationships_unknown_features_external_references_and_overlaps_fail() {
    let original = fixture::movie(&[fixture::jpeg(0), fixture::jpeg(1)], false, false);
    for (kind, relative, value) in [
        (b"stsc", 12, 0),
        (b"stsc", 16, 0),
        (b"stsc", 16, 3),
        (b"stsc", 20, 2),
        (b"stsz", 12, 3),
        (b"stsz", 16, 0),
        (b"stco", 12, 0),
        (b"stco", 16, 24),
        (b"stts", 12, 0),
        (b"stts", 12, 3),
        (b"stts", 16, 0),
        (b"stts", 16, 11),
        (b"dref", 8, 2),
        (b"url ", 4, 0),
    ] {
        let mut data = original.clone();
        let p = position(&data, kind) + relative;
        data[p..p + 4].copy_from_slice(&(value as u32).to_be_bytes());
        let error = extract(&data, &[0]).unwrap_err();
        assert_eq!(error.operation, Some("video.frames.extract"));
    }
    for (from, to) in [
        (b"stts", b"ctts"),
        (b"dinf", b"edts"),
        (b"jpeg", b"avc1"),
        (b"url ", b"alis"),
        (b"stsc", b"stz2"),
    ] {
        let mut data = original.clone();
        let p = position(&data, from);
        data[p..p + 4].copy_from_slice(to);
        assert!(extract(&data, &[0]).is_err());
    }
    let mut data = fixture::movie(&[fixture::jpeg(0)], true, false);
    let p = position(&data, b"co64") + 12;
    data[p..p + 8].copy_from_slice(&u64::MAX.to_be_bytes());
    assert!(extract(&data, &[0]).is_err());
    let mut data = original.clone();
    let p = position(&data, b"jpeg") + 10;
    data[p..p + 2].copy_from_slice(&2u16.to_be_bytes());
    assert!(extract(&data, &[0]).is_err());
    let mut data = original.clone();
    let p = position(&data, b"jpeg") + 28;
    data[p..p + 2].copy_from_slice(&3u16.to_be_bytes());
    assert!(extract(&data, &[0]).is_err());
    for cut in [0, 1, 7, original.len() / 2, original.len() - 1] {
        assert!(extract(&original[..cut], &[0]).is_err());
    }
    let start = position(&original, b"moov");
    for p in start..original.len() {
        let mut data = original.clone();
        data[p] ^= 255;
        let _ = extract(&data, &[0]);
    }
}
#[test]
fn selections_encoded_decoded_record_and_work_budgets_are_preflighted() {
    let data = fixture::movie(&[fixture::jpeg(0)], false, false);
    for indexes in [vec![], vec![0, 0], vec![1], vec![usize::MAX], vec![0; 9]] {
        assert!(extract(&data, &indexes).is_err());
    }
    assert!(frames::extract(&mut Cursor::new(&data), 0, &[0]).is_err());
    assert!(frames::extract(&mut Cursor::new(&data), 2, &[0]).is_err());
    let data = fixture::movie(&[vec![0; frames::MAX_FRAME_BYTES + 1]], false, false);
    assert_eq!(
        extract(&data, &[0]).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let mut data = fixture::movie(&[fixture::jpeg(0)], false, false);
    let p = position(&data, b"jpeg") + 28;
    data[p..p + 4].copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
    assert_eq!(
        extract(&data, &[0]).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let mut data = fixture::movie(&vec![fixture::jpeg(0); 8], false, false);
    let p = position(&data, b"jpeg") + 28;
    data[p..p + 4].copy_from_slice(&[8, 1, 4, 0]);
    assert_eq!(
        extract(&data, &[0, 1, 2, 3, 4, 5, 6, 7]).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let mut data = fixture::movie(&[fixture::jpeg(0)], false, false);
    let p = position(&data, b"stsz") + 12;
    data[p..p + 4].copy_from_slice(&10001u32.to_be_bytes());
    assert_eq!(
        extract(&data, &[0]).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let mut trailing = fixture::jpeg(0);
    trailing.push(0);
    assert!(extract(&fixture::movie(&[trailing], false, false), &[0]).is_err());
}

fn position(data: &[u8], kind: &[u8; 4]) -> usize {
    data.windows(4).position(|b| b == kind).unwrap()
}
