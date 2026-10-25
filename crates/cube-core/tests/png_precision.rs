//! Research fixtures: the dependency preserves native precision, while Cube's
//! public PNG8 API deliberately rejects other depths until a representation is agreed.
use cube_core::{errors::ErrorCode, image::Image};
use std::io::Cursor;
fn original(
    depth: png::BitDepth,
    color: png::ColorType,
    width: u32,
    samples: &[u8],
    palette: Option<&[u8]>,
    trns: Option<&[u8]>,
) -> Vec<u8> {
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, width, 1);
        encoder.set_depth(depth);
        encoder.set_color(color);
        if let Some(palette) = palette {
            encoder.set_palette(palette.to_vec());
        }
        if let Some(trns) = trns {
            encoder.set_trns(trns.to_vec());
        }
        encoder
            .write_header()
            .unwrap()
            .write_image_data(samples)
            .unwrap();
    }
    encoded
}
fn identity(
    encoded: &[u8],
    depth: png::BitDepth,
    expected: &[u8],
    palette: Option<&[u8]>,
    trns: Option<&[u8]>,
) {
    let mut decoder = png::Decoder::new_with_limits(
        Cursor::new(encoded),
        png::Limits {
            bytes: 32 * 1024 * 1024,
        },
    );
    decoder.set_transformations(png::Transformations::IDENTITY);
    let mut reader = decoder.read_info().unwrap();
    assert_eq!(reader.info().bit_depth, depth);
    assert_eq!(reader.info().palette.as_deref(), palette);
    assert_eq!(reader.info().trns.as_deref(), trns);
    let mut data = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut data).unwrap();
    assert_eq!(info.bit_depth, depth);
    assert_eq!(&data[..info.buffer_size()], expected);
    reader.finish().unwrap();
    assert_eq!(
        Image::decode(encoded).unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
}
#[test]
fn low_depth_gray_and_indexed_samples_remain_packed_unscaled_and_palette_separate() {
    for (depth, width, samples, palette, trns) in [
        (
            png::BitDepth::One,
            5,
            &[0x68][..],
            &[1, 2, 3, 4, 5, 6][..],
            &[0, 255][..],
        ),
        (
            png::BitDepth::Two,
            4,
            &[0x1b][..],
            &[1, 2, 3, 4, 5, 6, 1, 2, 3, 0, 0, 0][..],
            &[0, 64, 128, 255][..],
        ),
        (
            png::BitDepth::Four,
            4,
            &[0x01, 0x23][..],
            &[1, 2, 3, 4, 5, 6, 1, 2, 3, 0, 0, 0][..],
            &[0, 64, 128, 255][..],
        ),
    ] {
        let gray = original(depth, png::ColorType::Grayscale, width, samples, None, None);
        identity(&gray, depth, samples, None, None);
        let indexed = original(
            depth,
            png::ColorType::Indexed,
            width,
            samples,
            Some(palette),
            Some(trns),
        );
        identity(&indexed, depth, samples, Some(palette), Some(trns));
    }
}
#[test]
fn sixteen_bit_samples_and_transparency_keep_big_endian_low_bits() {
    for (color, samples, trns) in [
        (
            png::ColorType::Grayscale,
            vec![0x12, 0x34],
            Some(vec![0x12, 0x34]),
        ),
        (
            png::ColorType::GrayscaleAlpha,
            vec![0x12, 0x34, 0xab, 0xcd],
            None,
        ),
        (
            png::ColorType::Rgb,
            vec![0x00, 0x01, 0x12, 0x34, 0xab, 0xcd],
            Some(vec![0x00, 0x01, 0x12, 0x34, 0xab, 0xcd]),
        ),
        (
            png::ColorType::Rgba,
            vec![0x00, 0x01, 0x12, 0x34, 0xab, 0xcd, 0xff, 0xfe],
            None,
        ),
    ] {
        let encoded = original(
            png::BitDepth::Sixteen,
            color,
            1,
            &samples,
            None,
            trns.as_deref(),
        );
        identity(
            &encoded,
            png::BitDepth::Sixteen,
            &samples,
            None,
            trns.as_deref(),
        );
    }
}
