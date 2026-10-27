use cube_core::{
    errors::ErrorCode,
    image::{ColorModel, Image, qr::QrStatus},
};
// Original byte-mode Cube\0\xff symbol generated with qrcode 0.14.1, not a
// third-party message or copyrighted image. Matrix excludes its quiet zone.
fn fixture(scale: usize) -> Image {
    let matrix: Vec<_> = include_str!("../src/image/qr/known-binary.matrix")
        .lines()
        .collect();
    let side = (matrix.len() + 8) * scale;
    let mut samples = vec![255; side * side];
    for (y, row) in matrix.iter().enumerate() {
        for (x, value) in row.bytes().enumerate() {
            for dy in 0..scale {
                for dx in 0..scale {
                    samples[((y + 4) * scale + dy) * side + (x + 4) * scale + dx] =
                        if value == b'1' { 0 } else { 255 };
                }
            }
        }
    }
    Image::from_samples(
        side as u32,
        side as u32,
        ColorModel::Gray,
        samples,
        vec![],
        vec![],
    )
    .unwrap()
}
#[test]
fn original_binary_qr_decodes_at_documented_scales_and_rotations() {
    for scale in [2, 4, 8] {
        for angle in [0, 90, 180, 270] {
            let image = fixture(scale).rotate(angle).unwrap();
            let report = image.qr(8).unwrap();
            assert_eq!(report.status, QrStatus::Decoded, "{scale}/{angle}");
            assert_eq!(report.symbols.len(), 1);
            assert_eq!(
                report.symbols[0].payload.as_deref(),
                Some(&b"Cube\0\xff"[..])
            );
            assert_eq!(report.symbols[0].version, Some(1));
            assert!(report.symbols[0].corners.is_some());
            assert!(!report.detector_limits_reached);
        }
    }
}
#[test]
fn no_detection_undecodable_candidates_limits_and_opaque_policy_are_explicit() {
    let blank = Image::from_bytes(16, 16, ColorModel::Gray, &[255; 256], &[], &[]).unwrap();
    assert_eq!(blank.qr(8).unwrap().status, QrStatus::NoDetection);
    assert_eq!(blank.qr(0).unwrap_err().code, ErrorCode::InvalidParameter);
    assert!(blank.qr(9).is_err());
    let large = Image::from_samples(
        513,
        512,
        ColorModel::Gray,
        vec![255; 513 * 512],
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(large.qr(8).unwrap_err().code, ErrorCode::ResourceLimit);
    let transparent = Image::from_bytes(1, 1, ColorModel::Rgba, &[0; 4], &[], &[]).unwrap();
    assert_eq!(
        transparent.qr(8).unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
    let image = fixture(4);
    let side = image.width() as usize;
    let mut data = image.samples().to_vec();
    for y in (4 + 8) * 4..(4 + 21) * 4 {
        for x in (4 + 8) * 4..(4 + 21) * 4 {
            data[y * side + x] = 255;
        }
    }
    let damaged = Image::from_samples(
        side as u32,
        side as u32,
        ColorModel::Gray,
        data,
        vec![],
        vec![],
    )
    .unwrap();
    let report = damaged.qr(8).unwrap();
    assert_eq!(report.status, QrStatus::DetectedUndecodable);
    assert!(report.symbols[0].payload.is_none());
    assert!(report.symbols[0].error.is_some());
    let mut pair = Vec::with_capacity(side * side * 2);
    for row in image.samples().chunks_exact(side) {
        pair.extend_from_slice(row);
        pair.extend_from_slice(row);
    }
    let pair = Image::from_samples(
        (side * 2) as u32,
        side as u32,
        ColorModel::Gray,
        pair,
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(pair.qr(8).unwrap().symbols.len(), 2);
    assert_eq!(pair.qr(1).unwrap_err().code, ErrorCode::ResourceLimit);
}
