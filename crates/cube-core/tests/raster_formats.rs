use cube_core::{errors::ErrorCode, image::Image};
use raster_codecs::{ExtendedColorType, ImageEncoder, codecs};

fn encoded(format: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    let rgb = [128, 128, 128, 128, 128, 128];
    match format {
        "bmp" => codecs::bmp::BmpEncoder::new(&mut bytes)
            .write_image(&rgb, 2, 1, ExtendedColorType::Rgb8)
            .unwrap(),
        "jpeg" => codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 100)
            .write_image(&rgb, 2, 1, ExtendedColorType::Rgb8)
            .unwrap(),
        "webp" => codecs::webp::WebPEncoder::new_lossless(&mut bytes)
            .write_image(&rgb, 2, 1, ExtendedColorType::Rgb8)
            .unwrap(),
        "gif" => {
            let frame = raster_codecs::RgbaImage::from_raw(
                2,
                1,
                vec![128, 128, 128, 255, 128, 128, 128, 255],
            )
            .unwrap();
            codecs::gif::GifEncoder::new(&mut bytes)
                .encode_frame(raster_codecs::Frame::new(frame))
                .unwrap();
        }
        _ => unreachable!(),
    }
    bytes
}
#[test]
fn header_geometry_is_separate_from_raw_hypotheses_and_payload_validation() {
    use cube_core::image::{RawLayout, SampleLayout};
    let png = Image::from_bytes(2, 1, cube_core::image::ColorModel::Gray, &[1, 2], &[], &[])
        .unwrap()
        .encode_png()
        .unwrap();
    for bytes in [
        encoded("bmp"),
        encoded("gif"),
        encoded("jpeg"),
        encoded("webp"),
        png.clone(),
    ] {
        let layout = RawLayout {
            width: bytes.len() as u32,
            height: 1,
            layout: SampleLayout::Gray,
            row_stride: None,
            offset: 0,
        };
        let report = layout.inspect_bytes(&bytes).unwrap();
        let declared = report.declared.unwrap();
        assert_eq!((declared.width, declared.height), (2, 1));
        assert_eq!(declared.verification, "header_only_no_payload_validation");
        assert_eq!(report.candidate.width, bytes.len() as u32);
        assert!(report.header_error.is_none());
        assert_eq!(Image::from_raw(&bytes, &layout).unwrap().samples(), bytes);
    }
    let prefix = &png[..33];
    let layout = RawLayout {
        width: 33,
        height: 1,
        layout: SampleLayout::Gray,
        row_stride: None,
        offset: 0,
    };
    assert!(Image::decode(prefix).is_err());
    assert!(layout.inspect_bytes(prefix).unwrap().declared.is_some());
    let bad = [b'B', b'M', 0, 0];
    let layout = RawLayout { width: 4, ..layout };
    let report = layout.inspect_bytes(&bad).unwrap();
    assert!(report.declared.is_none());
    assert!(report.header_error.is_some());
    assert!(layout.inspect_bytes(&[0; 4]).unwrap().declared.is_none());
    assert!(layout.inspect_bytes(&[0; 3]).is_err());
}

#[test]
fn original_rasters_decode_explicit_models_and_reuse_exact_operations() {
    for format in ["jpeg", "gif", "bmp", "webp"] {
        let bytes = encoded(format);
        let image = Image::decode(&bytes).unwrap();
        assert_eq!((image.width(), image.height()), (2, 1));
        assert_eq!(image.info().format, format);
        assert_eq!(image.provenance().representation, "decoded_raster8_samples");
        assert!(!image.provenance().source_samples_preserved);
        assert_eq!(&image.pixel(0, 0).unwrap()[..3], &[128, 128, 128]);
        assert!(image.compare(&image, None, None, 10, false).unwrap().equal);
        let round_trip = Image::decode(&image.encode_png().unwrap()).unwrap();
        assert_eq!(round_trip.samples(), image.samples());
        assert_eq!(
            round_trip.provenance().representation,
            "native_png8_samples"
        );
        assert!(
            Image::decode(&bytes[..bytes.len() / 2]).is_err(),
            "{format}"
        );
        for length in 0..bytes.len().min(32) {
            assert!(
                Image::decode(&bytes[..length]).is_err(),
                "{format}: {length}"
            );
        }
    }
}

#[test]
fn excessive_dimensions_and_multiple_gif_frames_fail() {
    for format in ["gif", "bmp"] {
        let mut bytes = encoded(format);
        if format == "gif" {
            bytes[6..8].copy_from_slice(&u16::MAX.to_le_bytes());
            bytes[8..10].copy_from_slice(&u16::MAX.to_le_bytes());
        } else {
            bytes[18..22].copy_from_slice(&i32::MAX.to_le_bytes());
        }
        assert!(Image::decode(&bytes).is_err());
    }
    let mut bytes = Vec::new();
    {
        let mut encoder = codecs::gif::GifEncoder::new(&mut bytes);
        for _ in 0..2 {
            encoder
                .encode_frame(raster_codecs::Frame::new(
                    raster_codecs::RgbaImage::from_pixel(1, 1, raster_codecs::Rgba([0, 0, 0, 255])),
                ))
                .unwrap();
        }
    }
    assert_eq!(
        Image::decode(&bytes).unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
    let mut webp = encoded("webp");
    webp[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(Image::decode(&webp).is_err());
    assert!(Image::decode(b"RIFF\0\0\0\0WEBP").is_err());
}
