use cube_core::image::{ColorModel, ColorOutput, Image};
#[test]
fn exact_masks_layers_and_alpha_preserve_coordinates() {
    let source = Image::from_bytes(
        2,
        2,
        ColorModel::Rgba,
        &[1, 2, 3, 0, 1, 2, 3, 255, 4, 5, 6, 255, 1, 2, 3, 0],
        &[],
        &[],
    )
    .unwrap();
    let selected = vec![vec![1, 2, 3, 0]];
    let mask = source.exact_colors(&selected, ColorOutput::Mask).unwrap();
    assert_eq!(mask.selected_pixels, 2);
    assert_eq!(mask.image.samples(), [255, 0, 0, 255]);
    let layer = source.exact_colors(&selected, ColorOutput::Layer).unwrap();
    assert_eq!(
        layer.image.samples(),
        [1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 0]
    );
    assert_eq!(
        Image::decode(&layer.image.encode_png().unwrap())
            .unwrap()
            .samples(),
        layer.image.samples()
    );
    assert_eq!(
        source
            .exact_colors(&[vec![8; 4]], ColorOutput::Mask)
            .unwrap()
            .selected_pixels,
        0
    );
    for invalid in [
        vec![],
        vec![vec![1, 2, 3]],
        vec![vec![0; 4]; 257],
        vec![vec![1; 4], vec![1; 4]],
    ] {
        assert!(source.exact_colors(&invalid, ColorOutput::Mask).is_err());
    }
    assert!(ColorOutput::parse("fuzzy").is_err());
    assert!(ColorOutput::parse("layer").is_ok());
}
#[test]
fn native_indices_expanded_colors_and_transparency_are_distinct() {
    let indexed = Image::from_bytes(
        2,
        1,
        ColorModel::Indexed,
        &[0, 1],
        &[[1, 2, 3], [1, 2, 3]],
        &[],
    )
    .unwrap();
    let native = indexed
        .exact_colors(&[vec![0]], ColorOutput::Layer)
        .unwrap();
    assert_eq!(native.selected_pixels, 1);
    assert_eq!(native.image.samples(), [1, 2, 3, 255, 0, 0, 0, 0]);
    assert_eq!(
        indexed
            .expand_palette()
            .unwrap()
            .exact_colors(&[vec![1, 2, 3, 255]], ColorOutput::Mask)
            .unwrap()
            .image
            .samples(),
        [255, 255]
    );
    for (model, pixel, trns, expected) in [
        (ColorModel::Gray, vec![7], vec![0, 7], vec![7, 7, 7, 0]),
        (
            ColorModel::GrayAlpha,
            vec![7, 99],
            vec![],
            vec![7, 7, 7, 99],
        ),
        (
            ColorModel::Rgb,
            vec![1, 2, 3],
            vec![0, 1, 0, 2, 0, 3],
            vec![1, 2, 3, 0],
        ),
    ] {
        let image = Image::from_bytes(1, 1, model, &pixel, &[], &trns).unwrap();
        assert_eq!(
            image
                .exact_colors(&[pixel], ColorOutput::Layer)
                .unwrap()
                .image
                .samples(),
            expected
        );
    }
}
