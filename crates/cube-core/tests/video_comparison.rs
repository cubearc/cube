#[path = "common/mov.rs"]
mod fixture;
use cube_core::{
    errors::ErrorCode,
    image::{Channel, ColorModel, Image},
    media::video::{comparison, frames},
};
use std::io::Cursor;
#[test]
fn exact_pair_results_reuse_image_comparison_with_global_listing_and_provenance() {
    let data = fixture::movie(
        &[fixture::jpeg(0), fixture::jpeg(1), fixture::jpeg(0)],
        true,
        false,
    );
    let frames = frames::extract(&mut Cursor::new(&data), 1, &[0, 1, 2]).unwrap();
    let report =
        comparison::compare(&mut Cursor::new(&data), 1, &[(0, 1), (0, 2)], None, 100).unwrap();
    assert_eq!(report.compared_pair_pixels, 4);
    assert_eq!(
        report.provenance_origin,
        "validated_selected_sample_ranges_and_encoded_hashes"
    );
    let expected = frames[0]
        .image
        .compare(&frames[1].image, None, None, 100, false)
        .unwrap();
    assert_eq!(
        serde_json::to_value(&report.pairs[0].comparison).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    assert_eq!(report.pairs[1].left.frame_index, 0);
    assert_eq!(report.pairs[1].right.frame_index, 2);
    assert!(report.pairs[1].comparison.equal);
    let limited =
        comparison::compare_selected(&frames, &[(0, 1), (1, 0), (0, 2)], None, 1).unwrap();
    assert_eq!(limited.retained_differences, 1);
    assert_eq!(limited.pairs[0].comparison.differing_pixels, 2);
    assert!(!limited.pairs[0].comparison.complete);
    assert_eq!(limited.pairs[1].comparison.differing_pixels, 2);
    assert!(limited.pairs[1].comparison.differences.is_empty());
    assert!(!limited.pairs[1].comparison.complete);
    assert!(limited.pairs[2].comparison.complete);
    assert_eq!(
        limited.pairs[0].left.encoded_sha256,
        frames[0].provenance.encoded_sha256
    );
    assert!(
        cube_core::operations::require("video.frames.compare")
            .unwrap()
            .discovery()
            .pipeline
            .is_none()
    );
}
#[test]
fn previously_extracted_images_keep_explicit_model_alpha_and_dimension_semantics() {
    let data = fixture::movie(&[fixture::jpeg(0)], false, true);
    let mut frames = frames::extract(&mut Cursor::new(data), 1, &[0]).unwrap();
    frames[0].image = Image::from_bytes(1, 1, ColorModel::Rgba, &[1, 2, 3, 255], &[], &[]).unwrap();
    let mut other = frames[0].clone();
    other.image = Image::from_bytes(1, 1, ColorModel::Rgba, &[1, 2, 3, 0], &[], &[]).unwrap();
    frames.push(other);
    let compared =
        comparison::compare_selected(&frames, &[(0, 1)], Some(&[Channel::Red]), 10).unwrap();
    assert!(compared.pairs[0].comparison.equal);
    let compared =
        comparison::compare_selected(&frames, &[(0, 1)], Some(&[Channel::Alpha]), 10).unwrap();
    assert_eq!(compared.pairs[0].comparison.differing_pixels, 1);
    assert_eq!(
        compared.provenance_origin,
        "provided_frame_records_not_authenticated"
    );
    frames[1].image = Image::from_bytes(2, 1, ColorModel::Gray, &[1, 2], &[], &[]).unwrap();
    assert!(
        comparison::compare_selected(&frames, &[(0, 1)], None, 10)
            .unwrap_err()
            .to_string()
            .contains("dimensions")
    );
}
#[test]
fn pair_union_pixel_listing_channel_and_provenance_bounds_fail_safely() {
    let data = fixture::movie(&[fixture::jpeg(0)], false, false);
    for pairs in [
        vec![],
        vec![(0, 0); 9],
        vec![(0, 10000)],
        vec![(0, 1), (2, 3), (4, 5), (6, 7), (8, 9)],
    ] {
        assert!(comparison::compare(&mut Cursor::new(&data), 1, &pairs, None, 10).is_err());
    }
    let mut frames = frames::extract(&mut Cursor::new(&data), 1, &[0]).unwrap();
    for max in [0, 10001] {
        assert!(comparison::compare_selected(&frames, &[(0, 0)], None, max).is_err());
    }
    for channels in [vec![], vec![Channel::Gray, Channel::Gray]] {
        assert!(comparison::compare_selected(&frames, &[(0, 0)], Some(&channels), 10).is_err());
    }
    assert!(comparison::compare_selected(&frames, &[(0, 1)], None, 10).is_err());
    frames[0].provenance.encoded_sha256 = "invalid".into();
    assert!(comparison::compare_selected(&frames, &[(0, 0)], None, 10).is_err());
    frames[0].provenance.encoded_sha256 = "0".repeat(64);
    frames[0].image = Image::from_samples(
        2048,
        2048,
        ColorModel::Gray,
        vec![0; 4 * 1024 * 1024],
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(
        comparison::compare_selected(&frames, &[(0, 0); 3], None, 10)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
