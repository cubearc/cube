use cube_core::{
    errors::ErrorCode,
    image::{Image, RawLayout, SampleLayout},
};
fn layout(width: u32, height: u32, name: &str) -> RawLayout {
    RawLayout {
        width,
        height,
        layout: SampleLayout::parse(name).unwrap(),
        row_stride: None,
        offset: 0,
    }
}
#[test]
fn raw_layouts_preserve_known_samples_and_explicit_padding() {
    for (name, bytes, expected) in [
        ("gray", vec![0, 255], vec![0, 255]),
        (
            "rgb",
            vec![1, 2, 3, 0, 128, 255],
            vec![1, 2, 3, 0, 128, 255],
        ),
        (
            "bgr",
            vec![3, 2, 1, 255, 128, 0],
            vec![1, 2, 3, 0, 128, 255],
        ),
        (
            "rgba",
            vec![1, 2, 3, 0, 4, 5, 6, 255],
            vec![1, 2, 3, 0, 4, 5, 6, 255],
        ),
        (
            "bgra",
            vec![3, 2, 1, 0, 6, 5, 4, 255],
            vec![1, 2, 3, 0, 4, 5, 6, 255],
        ),
    ] {
        let original = bytes.clone();
        let image = Image::from_raw(&bytes, &layout(2, 1, name)).unwrap();
        assert_eq!(image.samples(), expected);
        assert_eq!(image.provenance().representation, "explicit_raw8_samples");
        assert_eq!(
            Image::decode(&image.encode_png().unwrap())
                .unwrap()
                .samples(),
            expected
        );
        assert_eq!(bytes, original);
    }
    let config = RawLayout {
        row_stride: Some(4),
        offset: 2,
        ..layout(1, 2, "bgr")
    };
    let bytes = [88, 99, 3, 2, 1, 77, 6, 5, 4, 66];
    let info = config.inspect(bytes.len()).unwrap();
    assert_eq!(
        (info.required_bytes, info.row_padding, info.sample_bytes),
        (10, 1, 6)
    );
    let image = Image::from_raw(&bytes, &config).unwrap();
    assert_eq!(image.pixel(0, 1).unwrap(), [4, 5, 6]);
    assert!(Image::from_raw(&bytes[..9], &config).is_err()); // Final padding is mandatory.
    assert!(Image::from_raw(&[bytes.as_slice(), &[0]].concat(), &config).is_err());
}
#[test]
fn raw_shapes_extents_and_resource_limits_fail_before_allocation() {
    assert!(SampleLayout::parse("rgb16").is_err());
    for config in [
        layout(0, 1, "gray"),
        layout(1, 0, "gray"),
        layout(u32::MAX, 1, "gray"),
        layout(16384, 16384, "gray"),
        RawLayout {
            row_stride: Some(2),
            ..layout(1, 1, "rgb")
        },
        RawLayout {
            offset: usize::MAX,
            ..layout(1, 1, "gray")
        },
        RawLayout {
            row_stride: Some(usize::MAX),
            ..layout(1, 2, "gray")
        },
    ] {
        assert!(config.inspect(1).is_err());
    }
    assert!(Image::from_raw(&[], &layout(1, 1, "gray")).is_err());
    assert_eq!(
        layout(1, 1, "gray")
            .inspect(32 * 1024 * 1024 + 1)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
