use cube_core::{
    errors::ErrorCode,
    image::{ColorModel, ColorOutput, Image},
};
#[test]
fn exact_layers_are_sorted_counted_and_preserve_hidden_color_bytes() {
    let image = Image::from_bytes(
        3,
        1,
        ColorModel::Rgba,
        &[2, 3, 4, 0, 1, 3, 4, 0, 2, 3, 4, 255],
        &[],
        &[],
    )
    .unwrap();
    let report = image.color_layers(None, ColorOutput::Layer, 3, 3).unwrap();
    assert_eq!(report.outputs.len(), 3);
    assert_eq!(report.outputs[0].name, "color-01030400.png");
    assert_eq!(
        report
            .outputs
            .iter()
            .map(|v| v.selected_pixels)
            .sum::<u64>(),
        3
    );
    let layer = Image::decode(&report.outputs[0].png).unwrap();
    assert_eq!(layer.samples(), [0, 0, 0, 0, 1, 3, 4, 0, 0, 0, 0, 0]);
    assert_eq!(image.samples()[0], 2);
    let subset = image
        .color_layers(Some(&[vec![2, 3, 4, 255]]), ColorOutput::Mask, 3, 1)
        .unwrap();
    assert_eq!(
        Image::decode(&subset.outputs[0].png).unwrap().samples(),
        [0, 0, 255]
    );
    assert_eq!(subset.outputs[0].selected_pixels, 1);
}
#[test]
fn native_indices_and_explicit_expansion_have_distinct_color_identity() {
    let image = Image::from_bytes(
        2,
        1,
        ColorModel::Indexed,
        &[0, 1],
        &[[1, 2, 3], [1, 2, 3]],
        &[],
    )
    .unwrap();
    assert_eq!(
        image
            .color_layers(None, ColorOutput::Mask, 2, 2)
            .unwrap()
            .outputs
            .len(),
        2
    );
    let expanded = image
        .expand_palette()
        .unwrap()
        .color_layers(None, ColorOutput::Mask, 1, 1)
        .unwrap();
    assert_eq!(expanded.outputs.len(), 1);
    assert_eq!(expanded.outputs[0].selected_pixels, 2);
    assert_eq!(
        Image::decode(&expanded.outputs[0].png).unwrap().samples(),
        [255, 255]
    );
}
#[test]
fn selection_count_work_and_png_budgets_fail_before_export() {
    let image = Image::from_bytes(2, 1, ColorModel::Gray, &[0, 1], &[], &[]).unwrap();
    for colors in [
        vec![],
        vec![vec![0], vec![0]],
        vec![vec![2]],
        vec![vec![0, 1]],
    ] {
        assert!(
            image
                .color_layers(Some(&colors), ColorOutput::Mask, 2, 2)
                .is_err()
        );
    }
    for (colors, outputs) in [(0, 2), (65, 2), (2, 0), (2, 65), (1, 2), (2, 1)] {
        assert!(
            image
                .color_layers(None, ColorOutput::Mask, colors, outputs)
                .is_err()
        );
    }
    let image = Image::from_samples(
        2048,
        2048,
        ColorModel::Gray,
        (0..4 * 1024 * 1024).map(|i| (i % 4) as u8).collect(),
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(
        image
            .color_layers(None, ColorOutput::Mask, 4, 4)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let large = Image::from_samples(
        2048,
        2048,
        ColorModel::Gray,
        (0..4 * 1024 * 1024).map(|i| (i % 2) as u8).collect(),
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(
        large
            .color_layers(None, ColorOutput::Layer, 2, 2)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
