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

#[test]
fn exact_pixel_exports_channels_and_histograms_preserve_alpha_and_indices() {
    use cube_core::image::{Channel as C, Region};
    let image = Image::from_samples(
        2,
        2,
        ColorModel::Rgba,
        vec![
            10, 20, 30, 0, 40, 50, 60, 255, 10, 20, 30, 255, 40, 50, 60, 128,
        ],
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(image.pixel(1, 0).unwrap(), [40, 50, 60, 255]);
    assert!(image.pixel(2, 0).is_err());
    assert_eq!(
        image.channel(C::Alpha, None).unwrap().data,
        [0, 255, 255, 128]
    );
    let region = Region {
        x: 1,
        y: 0,
        width: 1,
        height: 2,
    };
    let data = image
        .pixels(Some(region), Some(&[C::Blue, C::Red]))
        .unwrap();
    assert_eq!(data.data, [60, 40, 60, 40]);
    let records = data.records().unwrap();
    assert_eq!((records[1].x, records[1].y), (1, 1));
    assert_eq!(records[1].values, [60, 40]);
    assert!(data.grayscale_image().is_err());
    assert_eq!(
        image
            .channel(C::Red, None)
            .unwrap()
            .grayscale_image()
            .unwrap()
            .samples(),
        [10, 40, 10, 40]
    );
    assert_eq!(image.histogram(None, None, 10).unwrap().colors.len(), 4);
    let rgb = image
        .histogram(None, Some(&[C::Red, C::Green, C::Blue]), 10)
        .unwrap();
    assert_eq!(rgb.pixels, 4);
    assert_eq!(rgb.colors.len(), 2);
    assert_eq!(rgb.colors[0].values, [10, 20, 30]);
    assert_eq!(rgb.colors[0].count, 2);
    assert!(image.histogram(None, None, 1).is_err());
    assert!(image.histogram(None, None, 0).is_err());
    assert!(image.pixels(None, Some(&[])).is_err());
    assert!(image.pixels(None, Some(&[C::Red, C::Red])).is_err());
    assert!(image.channel(C::Index, None).is_err());
    for region in [
        Region {
            x: u32::MAX,
            y: 0,
            width: 2,
            height: 1,
        },
        Region {
            x: 0,
            y: 0,
            width: 0,
            height: 1,
        },
        Region {
            x: 0,
            y: 2,
            width: 1,
            height: 1,
        },
    ] {
        assert!(image.pixels(Some(region), None).is_err());
    }
    let indexed = Image::from_samples(
        2,
        1,
        ColorModel::Indexed,
        vec![0, 1],
        vec![[1, 2, 3], [1, 2, 3]],
        vec![],
    )
    .unwrap();
    assert_eq!(indexed.channel(C::Index, None).unwrap().data, [0, 1]);
    assert!(indexed.channel(C::Red, None).is_err());
    assert_eq!(indexed.histogram(None, None, 10).unwrap().colors.len(), 2);
    assert_eq!(
        indexed
            .view(true)
            .unwrap()
            .histogram(None, None, 10)
            .unwrap()
            .colors[0]
            .count,
        2
    );
    let large =
        Image::from_samples(10001, 1, ColorModel::Gray, vec![0; 10001], vec![], vec![]).unwrap();
    assert!(large.pixels(None, None).unwrap().records().is_err());
    assert!(C::parse("unknown").is_err());
    for name in ["red", "green", "blue", "alpha", "gray", "index"] {
        assert!(C::parse(name).is_ok());
    }
}
#[test]
fn image_comparison_has_exact_counts_coordinates_and_binary_mask() {
    use cube_core::image::{Channel as C, Region};
    let a = Image::from_samples(
        2,
        1,
        ColorModel::Rgba,
        vec![1, 2, 3, 0, 4, 5, 6, 255],
        vec![],
        vec![],
    )
    .unwrap();
    let b = Image::from_samples(
        2,
        1,
        ColorModel::Rgba,
        vec![1, 2, 3, 255, 4, 50, 6, 255],
        vec![],
        vec![],
    )
    .unwrap();
    let result = a.compare(&b, None, None, 1, true).unwrap();
    assert_eq!(result.differing_pixels, 2);
    assert!(!result.complete);
    assert_eq!((result.differences[0].x, result.differences[0].y), (0, 0));
    assert_eq!(result.mask.unwrap().samples(), [255, 255]);
    assert!(a.compare(&a, None, None, 10, false).unwrap().equal);
    assert_eq!(
        a.compare(&b, None, Some(&[C::Red, C::Blue]), 10, false)
            .unwrap()
            .differing_pixels,
        0
    );
    assert_eq!(
        a.compare(
            &b,
            Some(Region {
                x: 1,
                y: 0,
                width: 1,
                height: 1
            }),
            None,
            10,
            false
        )
        .unwrap()
        .differences[0]
            .x,
        1
    );
    let rgb = Image::from_samples(
        2,
        1,
        ColorModel::Rgb,
        vec![1, 2, 3, 4, 5, 6],
        vec![],
        vec![],
    )
    .unwrap();
    assert!(a.compare(&rgb, None, None, 10, false).is_err());
    assert!(
        a.compare(&rgb, None, Some(&[C::Red, C::Blue]), 10, false)
            .unwrap()
            .equal
    );
    assert!(a.compare(&rgb, None, Some(&[C::Alpha]), 10, false).is_err());
    assert!(a.compare(&b, None, None, 0, false).is_err());
    assert!(
        a.compare(&sample(ColorModel::Rgba), None, None, 10, false)
            .is_err()
    );
}
