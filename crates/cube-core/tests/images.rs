mod common;
use common::Workspace;
use cube_core::{
    errors::ErrorCode,
    forensics::png,
    image::{ColorModel, Image, MAX_DIMENSION},
};
use std::io::Cursor;
fn chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut bytes = (data.len() as u32).to_be_bytes().to_vec();
    bytes.extend(kind);
    bytes.extend(data);
    let mut crc = crc32fast::Hasher::new();
    crc.update(kind);
    crc.update(data);
    bytes.extend(crc.finalize().to_be_bytes());
    bytes
}
fn sample(model: ColorModel) -> Image {
    let data: Vec<u8> = match model {
        ColorModel::Indexed => vec![0, 1, 1, 0],
        _ => (0..4 * model.channels()).map(|v| (v * 17) as u8).collect(),
    };
    Image::from_samples(
        2,
        2,
        model,
        data,
        if model == ColorModel::Indexed {
            vec![[1, 2, 3], [4, 5, 6]]
        } else {
            vec![]
        },
        if model == ColorModel::Indexed {
            vec![0, 128]
        } else {
            vec![]
        },
    )
    .unwrap()
}
#[test]
fn png8_round_trips_preserve_every_sample_model_palette_and_alpha() {
    for model in [
        ColorModel::Gray,
        ColorModel::GrayAlpha,
        ColorModel::Rgb,
        ColorModel::Rgba,
        ColorModel::Indexed,
    ] {
        let source = sample(model);
        let bytes = source.encode_png().unwrap();
        let decoded = Image::decode(&bytes).unwrap();
        assert_eq!(decoded.model(), model);
        assert_eq!(decoded.width(), 2);
        assert_eq!(decoded.height(), 2);
        assert_eq!(decoded.samples(), source.samples());
        assert_eq!(decoded.palette(), source.palette());
        assert_eq!(decoded.transparency(), source.transparency());
        assert_eq!(decoded.info().row_stride, 2 * model.channels());
        assert!(decoded.exif().unwrap().is_none());
    }
    for (model, trns) in [
        (ColorModel::Gray, vec![0, 17]),
        (ColorModel::Rgb, vec![0, 0, 0, 17, 0, 34]),
    ] {
        let source = sample(model);
        let image = Image::from_bytes(2, 2, model, source.samples(), &[], &trns).unwrap();
        assert_eq!(
            Image::decode(&image.encode_png().unwrap())
                .unwrap()
                .transparency(),
            trns
        );
    }
    let indexed = sample(ColorModel::Indexed);
    let rgba = indexed.expand_palette().unwrap();
    assert_eq!(rgba.model(), ColorModel::Rgba);
    assert_eq!(
        rgba.samples(),
        [1, 2, 3, 0, 4, 5, 6, 128, 4, 5, 6, 128, 1, 2, 3, 0]
    );
    assert!(rgba.expand_palette().is_err());
}
#[test]
fn image_inputs_are_checked_before_allocation_or_file_overwrite() {
    assert_eq!(
        Image::from_samples(
            MAX_DIMENSION + 1,
            1,
            ColorModel::Gray,
            vec![],
            vec![],
            vec![]
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
    assert!(Image::from_bytes(0, 1, ColorModel::Gray, &[], &[], &[]).is_err());
    assert!(Image::from_bytes(1, 1, ColorModel::Rgb, &[0], &[], &[]).is_err());
    assert!(ColorModel::parse("unknown").is_err());
    for (samples, palette, trns) in [
        (vec![1], vec![[0, 0, 0]], vec![]),
        (vec![0], vec![], vec![]),
        (vec![0], vec![[0, 0, 0]], vec![0, 0]),
    ] {
        assert!(Image::from_samples(1, 1, ColorModel::Indexed, samples, palette, trns).is_err());
    }
    assert!(Image::from_samples(1, 1, ColorModel::Gray, vec![1], vec![[0, 0, 0]], vec![]).is_err());
    assert!(Image::from_samples(1, 1, ColorModel::Rgba, vec![1; 4], vec![], vec![0]).is_err());
    assert_eq!(
        Image::decode(b"not a png").unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
    let dir = Workspace::new();
    let path = dir.path("image.png");
    let image = sample(ColorModel::Rgba);
    image.save_png(&path).unwrap();
    assert!(image.save_png(&path).is_err());
    assert_eq!(Image::open(&path).unwrap().samples(), image.samples());
    let original = image.encode_png().unwrap();
    for length in [0, 8, 20, original.len() - 1] {
        assert!(Image::decode(&original[..length]).is_err());
    }
    let mut depth16 = Vec::new();
    {
        let mut encoder = ::png::Encoder::new(&mut depth16, 1, 1);
        encoder.set_depth(::png::BitDepth::Sixteen);
        encoder.set_color(::png::ColorType::Grayscale);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&[0, 1])
            .unwrap();
    }
    assert_eq!(
        Image::decode(&depth16).unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
}
#[test]
fn chunk_inspector_reuses_strict_parser_and_exports_payloads_only_explicitly() {
    let original = sample(ColorModel::Rgb).encode_png().unwrap();
    let mut input = original[..33].to_vec();
    input.extend(chunk(b"ruSt", b"original synthetic bytes"));
    input.extend_from_slice(&original[33..]);
    input.extend(b"tail");
    let report = png::inspect(&mut Cursor::new(&input), true, 0).unwrap();
    assert!(report.crc_valid);
    assert_eq!(report.boundary.trailing_bytes, 4);
    assert!(report.chunks.iter().all(|c| c.payload_hex.is_none()));
    assert_eq!(report.chunks[1].offset, 33);
    assert_eq!(report.chunks[1].kind, "ruSt");
    let report = png::inspect(&mut Cursor::new(&input), true, 1024).unwrap();
    assert_eq!(
        report.chunks[1].payload_hex.as_deref(),
        Some("6f726967696e616c2073796e746865746963206279746573")
    );
    let offset = 33 + 8 + b"original synthetic bytes".len();
    input[offset] ^= 1;
    assert!(png::appended(&mut Cursor::new(&input)).is_err());
    let report = png::inspect(&mut Cursor::new(&input), false, 0).unwrap();
    assert!(!report.crc_valid);
    assert!(!report.chunks[1].valid);
    assert!(png::inspect(&mut Cursor::new(&input), false, 1).is_err());
    assert!(png::inspect(&mut Cursor::new(&input), false, 1048577).is_err());
}
