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

#[test]
fn geometric_transforms_preserve_exact_non_square_samples_and_palette() {
    use cube_core::image::Region;
    let image = Image::from_samples(
        2,
        3,
        ColorModel::Gray,
        vec![1, 2, 3, 4, 5, 6],
        vec![],
        vec![],
    )
    .unwrap();
    for (angle, expected, w, h) in [
        (0, vec![1, 2, 3, 4, 5, 6], 2, 3),
        (90, vec![5, 3, 1, 6, 4, 2], 3, 2),
        (180, vec![6, 5, 4, 3, 2, 1], 2, 3),
        (270, vec![2, 4, 6, 1, 3, 5], 3, 2),
    ] {
        let result = image.rotate(angle).unwrap();
        assert_eq!(result.samples(), expected);
        assert_eq!((result.width(), result.height()), (w, h));
        assert_eq!(
            result.rotate((360 - angle) % 360).unwrap().samples(),
            image.samples()
        );
        assert_eq!(
            Image::decode(&result.encode_png().unwrap())
                .unwrap()
                .samples(),
            expected
        );
    }
    let mut rotated = image.clone();
    for _ in 0..4 {
        rotated = rotated.rotate(90).unwrap();
    }
    assert_eq!(rotated.samples(), image.samples());
    assert_eq!(
        image.mirror("horizontal").unwrap().samples(),
        [2, 1, 4, 3, 6, 5]
    );
    assert_eq!(
        image.mirror("vertical").unwrap().samples(),
        [5, 6, 3, 4, 1, 2]
    );
    for axis in ["horizontal", "vertical"] {
        assert_eq!(
            image.mirror(axis).unwrap().mirror(axis).unwrap().samples(),
            image.samples()
        );
    }
    assert_eq!(
        image
            .crop(Region {
                x: 1,
                y: 1,
                width: 1,
                height: 2
            })
            .unwrap()
            .samples(),
        [4, 6]
    );
    assert!(
        image
            .crop(Region {
                x: u32::MAX,
                y: 0,
                width: 1,
                height: 1
            })
            .is_err()
    );
    assert!(
        image
            .crop(Region {
                x: 0,
                y: 0,
                width: 0,
                height: 1
            })
            .is_err()
    );
    assert!(image.rotate(45).is_err());
    assert!(image.mirror("diagonal").is_err());
    for model in [ColorModel::Indexed, ColorModel::Rgba] {
        let source = sample(model);
        let transformed = source.rotate(90).unwrap().mirror("vertical").unwrap();
        assert_eq!(transformed.palette(), source.palette());
        assert_eq!(transformed.transparency(), source.transparency());
        assert_eq!(
            transformed
                .mirror("vertical")
                .unwrap()
                .rotate(270)
                .unwrap()
                .samples(),
            source.samples()
        );
    }
    let single = Image::from_samples(1, 1, ColorModel::Gray, vec![255], vec![], vec![]).unwrap();
    assert_eq!(single.rotate(90).unwrap().samples(), [255]);
}

#[test]
fn color_transforms_are_reversible_or_explicitly_lossy_and_keep_alpha() {
    use cube_core::image::Channel::{Alpha, Blue, Green, Red};
    let rgb = Image::from_samples(
        3,
        1,
        ColorModel::Rgb,
        vec![255, 0, 0, 0, 255, 0, 0, 0, 255],
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(rgb.grayscale().unwrap().samples(), [77, 149, 29]);
    assert_eq!(
        rgb.invert(None).unwrap().invert(None).unwrap().samples(),
        rgb.samples()
    );
    assert_eq!(
        rgb.reorder(&[Blue, Red, Green])
            .unwrap()
            .reorder(&[Green, Blue, Red])
            .unwrap()
            .samples(),
        rgb.samples()
    );
    for channels in [
        &[Red, Red, Blue][..],
        &[Red, Green][..],
        &[Red, Green, Alpha][..],
    ] {
        assert!(rgb.reorder(channels).is_err());
    }
    let rgba =
        Image::from_samples(1, 1, ColorModel::Rgba, vec![255, 0, 0, 7], vec![], vec![]).unwrap();
    assert_eq!(rgba.invert(None).unwrap().samples(), [0, 255, 255, 7]);
    assert_eq!(
        rgba.invert(Some(&[Alpha])).unwrap().samples(),
        [255, 0, 0, 248]
    );
    assert_eq!(rgba.grayscale().unwrap().samples(), [77, 7]);
    assert_eq!(
        rgba.reorder(&[Alpha, Red, Green, Blue]).unwrap().samples(),
        [7, 255, 0, 0]
    );
    for model in [ColorModel::Gray, ColorModel::GrayAlpha] {
        let source = sample(model);
        assert_eq!(source.grayscale().unwrap().samples(), source.samples());
    }
    let indexed = sample(ColorModel::Indexed);
    assert!(indexed.invert(None).is_err());
    assert!(indexed.reorder(&[Red]).is_err());
    assert!(indexed.grayscale().is_err());
    assert!(indexed.expand_palette().unwrap().grayscale().is_ok());
    // Two colors sharing luma must not gain transparency through conversion.
    let key = Image::from_samples(
        2,
        1,
        ColorModel::Rgb,
        vec![0, 0, 0, 0, 0, 1],
        vec![],
        vec![0; 6],
    )
    .unwrap();
    assert_eq!(key.grayscale().unwrap().samples(), [0, 0, 0, 255]);
    assert_eq!(
        key.invert(None).unwrap().transparency(),
        [0, 255, 0, 255, 0, 255]
    );
    let key = Image::from_samples(
        1,
        1,
        ColorModel::Rgb,
        vec![1, 2, 3],
        vec![],
        vec![0, 1, 0, 2, 0, 3],
    )
    .unwrap();
    assert_eq!(
        key.reorder(&[Blue, Red, Green]).unwrap().transparency(),
        [0, 3, 0, 1, 0, 2]
    );
    let gray = Image::from_samples(2, 1, ColorModel::Gray, vec![7, 8], vec![], vec![0, 7]).unwrap();
    assert_eq!(gray.grayscale().unwrap().samples(), [7, 0, 8, 255]);
}

#[test]
fn bit_planes_preserve_exact_values_packing_and_explicit_visualization() {
    use cube_core::image::{Channel, Region};
    let image = Image::from_samples(
        8,
        1,
        ColorModel::Gray,
        (0..8).map(|i| 1 << i).collect(),
        vec![],
        vec![],
    )
    .unwrap();
    for bit in 0..8 {
        let plane = image.bit_plane(Channel::Gray, bit, None).unwrap();
        let expected: Vec<u8> = (0..8).map(|i| u8::from(i == bit)).collect();
        assert_eq!(plane.pixels.data, expected);
        assert_eq!(plane.packed, [1 << (7 - bit)]);
        assert_eq!(plane.valid_bits, 8);
        assert_eq!(
            plane.visualization().unwrap().samples(),
            expected.iter().map(|v| v * 255).collect::<Vec<_>>()
        );
    }
    let plane = image
        .bit_plane(
            Channel::Gray,
            1,
            Some(Region {
                x: 1,
                y: 0,
                width: 3,
                height: 1,
            }),
        )
        .unwrap();
    assert_eq!(plane.pixels.data, [1, 0, 0]);
    assert_eq!(plane.packed, [128]);
    assert_eq!(plane.valid_bits, 3);
    assert!(image.bit_plane(Channel::Gray, 8, None).is_err());
    assert!(image.bit_plane(Channel::Alpha, 0, None).is_err());
    let indexed = sample(ColorModel::Indexed);
    assert_eq!(
        indexed
            .bit_plane(Channel::Index, 0, None)
            .unwrap()
            .pixels
            .data,
        indexed.samples()
    );
    assert!(indexed.bit_plane(Channel::Red, 0, None).is_err());
    assert!(
        indexed
            .expand_palette()
            .unwrap()
            .bit_plane(Channel::Alpha, 7, None)
            .is_ok()
    );
}

#[test]
fn lsb_orders_pixels_channels_bits_and_packs_without_text_conversion() {
    use cube_core::image::{BitOrder, Channel, Incomplete, LsbOptions, Region, Traversal};
    let image = Image::from_samples(
        4,
        2,
        ColorModel::Gray,
        vec![1, 0, 1, 0, 0, 1, 1, 0],
        vec![],
        vec![],
    )
    .unwrap();
    for (traversal, reverse, msb, lsb) in [
        (Traversal::Row, false, 0xa6, 0x65),
        (Traversal::Row, true, 0x65, 0xa6),
        (Traversal::Column, false, 0x9c, 0x39),
        (Traversal::Column, true, 0x39, 0x9c),
    ] {
        for (bit_order, expected) in [(BitOrder::Msb, msb), (BitOrder::Lsb, lsb)] {
            let result = image
                .lsb(&LsbOptions {
                    traversal,
                    reverse,
                    bit_order,
                    ..Default::default()
                })
                .unwrap();
            assert_eq!(result.data, [expected]);
            assert_eq!(result.valid_bits, 8);
            assert_eq!(result.dropped_bits, 0);
        }
    }
    let rgb = Image::from_samples(
        2,
        1,
        ColorModel::Rgb,
        vec![1, 2, 3, 0, 3, 2],
        vec![],
        vec![],
    )
    .unwrap();
    for (channels, bits, expected) in [
        (vec![Channel::Green, Channel::Red], vec![1, 0], 0x9c),
        (vec![Channel::Green, Channel::Red], vec![0, 1], 0x6c),
        (vec![Channel::Red, Channel::Green], vec![1, 0], 0x63),
    ] {
        assert_eq!(
            rgb.lsb(&LsbOptions {
                channels: Some(channels),
                bits,
                ..Default::default()
            })
            .unwrap()
            .data,
            [expected]
        );
    }
    let payload = [0, 255, 128];
    let values = payload
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |bit| 0xf0 | ((byte >> bit) & 1)))
        .collect();
    let embedded = Image::from_samples(8, 3, ColorModel::Gray, values, vec![], vec![]).unwrap();
    assert_eq!(embedded.lsb(&LsbOptions::default()).unwrap().data, payload);
    let result = image
        .lsb(&LsbOptions {
            region: Some(Region {
                x: 1,
                y: 0,
                width: 2,
                height: 2,
            }),
            incomplete: Incomplete::Pad,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(result.data, [0x70]);
    assert_eq!(result.valid_bits, 4);
    let image = Image::from_samples(9, 1, ColorModel::Gray, vec![1; 9], vec![], vec![]).unwrap();
    assert!(image.lsb(&LsbOptions::default()).is_err());
    let pad = image
        .lsb(&LsbOptions {
            incomplete: Incomplete::Pad,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(pad.data, [255, 128]);
    assert_eq!(pad.valid_bits, 9);
    let drop = image
        .lsb(&LsbOptions {
            incomplete: Incomplete::Drop,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(drop.data, [255]);
    assert_eq!(drop.available_bits, 9);
    assert_eq!(drop.valid_bits, 8);
    assert_eq!(drop.dropped_bits, 1);
    for bits in [vec![], vec![0, 0], vec![8], vec![0; 9]] {
        assert!(
            image
                .lsb(&LsbOptions {
                    bits,
                    ..Default::default()
                })
                .is_err()
        );
    }
    for channels in [
        vec![],
        vec![Channel::Gray, Channel::Gray],
        vec![Channel::Alpha],
    ] {
        assert!(
            image
                .lsb(&LsbOptions {
                    channels: Some(channels),
                    ..Default::default()
                })
                .is_err()
        );
    }
    assert!(Traversal::parse("spiral").is_err());
    assert!(BitOrder::parse("other").is_err());
    assert!(Incomplete::parse("other").is_err());
}
