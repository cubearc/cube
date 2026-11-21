use cube_core::image::{
    Channel, ColorModel, Image, Region,
    binary_grid::{
        Connectivity, Element, ElementShape, Morphology, components, flood, morphology, otsu,
    },
    edges::{Canny, EdgeBorder, EdgeKernel, canny, responses},
    filters::{Filter, FilterBorder, apply},
    matching::{Metric, TemplateOptions, match_template},
    quality::{Metric as QualityMetric, measure},
    voting::{HoughOptions, hough},
};

fn gray(w: u32, h: u32, data: &[u8]) -> Image {
    Image::from_bytes(w, h, ColorModel::Gray, data, &[], &[]).unwrap()
}

#[test]
fn exact_quality_counts_and_ssim_identity() {
    let first = gray(2, 1, &[0, 4]);
    let second = gray(2, 1, &[3, 8]);
    let region = Region::full(&first);
    let result = measure(&first, &second, region, Channel::Gray, QualityMetric::Psnr).unwrap();
    assert_eq!(
        (result.sum_squared_difference, result.count, result.mse),
        (25, 2, 12.5)
    );
    assert!((result.value.unwrap() - 37.16170347859854).abs() < 1e-10);
    let image = gray(11, 11, &[32; 121]);
    let black = gray(11, 11, &[0; 121]);
    let region = Region::full(&image);
    let identity = measure(&image, &image, region, Channel::Gray, QualityMetric::Ssim).unwrap();
    assert_eq!(identity.value, Some(1.));
    let different = measure(&image, &black, region, Channel::Gray, QualityMetric::Ssim).unwrap();
    assert!((different.value.unwrap() - 6.5025 / 1030.5025).abs() < 1e-10);
    let perfect = measure(&image, &image, region, Channel::Gray, QualityMetric::Psnr).unwrap();
    assert!(perfect.perfect && perfect.value.is_none());
}

#[test]
fn filters_and_edges_match_small_hand_computed_grids() {
    let impulse = gray(3, 1, &[0, 100, 0]);
    let g = apply(
        &impulse,
        Filter::Gaussian {
            radius: 1,
            sigma: 1.,
        },
        None,
        FilterBorder::Replicate,
    )
    .unwrap();
    assert_eq!(g.samples(), [27, 45, 27]);
    assert_eq!(
        apply(
            &impulse,
            Filter::Median { width: 3 },
            None,
            FilterBorder::Replicate
        )
        .unwrap()
        .samples(),
        [0, 0, 0],
    );
    let row = gray(3, 1, &[0, 10, 30]);
    let region = Region::full(&row);
    let sobel = responses(
        &row,
        region,
        Channel::Gray,
        EdgeKernel::Sobel,
        EdgeBorder::Replicate,
        true,
    )
    .unwrap();
    assert_eq!(sobel.gx.unwrap(), [40, 120, 80]);
    assert_eq!(sobel.gy.unwrap(), [0, 0, 0]);
    assert_eq!(sobel.magnitude.unwrap(), [40., 120., 80.]);
    let lap = responses(
        &row,
        region,
        Channel::Gray,
        EdgeKernel::Laplacian,
        EdgeBorder::Replicate,
        false,
    )
    .unwrap();
    assert_eq!(lap.laplacian.unwrap(), [10, 10, -20]);
    let flat = gray(3, 2, &[16; 6]);
    assert_eq!(
        canny(
            &flat,
            Region::full(&flat),
            Channel::Gray,
            Canny {
                radius: 1,
                sigma: 1.,
                low: 1.,
                high: 2.
            }
        )
        .unwrap()
        .mask,
        [0; 6],
    );
}

#[test]
fn masks_and_components_preserve_row_major_identities() {
    let image = gray(2, 2, &[0, 255, 255, 0]);
    let threshold = otsu(&image, Region::full(&image), Channel::Gray).unwrap();
    assert_eq!(threshold.threshold, 0);
    assert_eq!(threshold.mask, image.samples());
    let labels = components(2, 2, &threshold.mask, Connectivity::Four).unwrap();
    assert_eq!(labels.labels, [0, 1, 2, 0]);
    assert_eq!(labels.components[1].area, 1);
    assert_eq!(
        components(2, 2, &threshold.mask, Connectivity::Eight)
            .unwrap()
            .labels,
        [0, 1, 1, 0]
    );
    let selected = flood(2, 2, &threshold.mask, Connectivity::Four, &[[0, 1], [0, 0]]).unwrap();
    assert_eq!(selected.seed_labels, [2, 0]);
    assert_eq!(selected.mask, [0, 0, 255, 0]);
    let dilated = morphology(
        2,
        2,
        &threshold.mask,
        Morphology::Dilate,
        Element {
            width: 3,
            height: 3,
            shape: ElementShape::Cross,
        },
        1,
    )
    .unwrap();
    assert_eq!(dilated.mask, [255; 4]);
    assert!(components(1, 1, &[128], Connectivity::Four).is_err());
}

#[test]
fn hough_and_template_ties_are_ordered_by_coordinates() {
    let votes = hough(
        2,
        2,
        &[255, 0, 255, 0],
        HoughOptions {
            theta_bins: 2,
            rho_step: 1.,
            minimum_votes: 1,
            max_peaks: 4,
        },
    )
    .unwrap();
    assert_eq!(
        (
            votes.peaks[0].votes,
            votes.peaks[0].theta_index,
            votes.peaks[0].rho
        ),
        (2, 0, 0.)
    );
    let source = gray(4, 1, &[0, 1, 0, 1]);
    let template = gray(2, 1, &[0, 1]);
    let result = match_template(
        &source,
        Channel::Gray,
        &template,
        Channel::Gray,
        TemplateOptions {
            metric: Metric::Ncc,
            best: 3,
            dense: true,
        },
    )
    .unwrap();
    assert_eq!(
        result
            .matches
            .iter()
            .map(|result| (result.x, result.score))
            .collect::<Vec<_>>(),
        [(0, Some(1.)), (2, Some(1.)), (1, Some(-1.))]
    );
    assert_eq!(result.dense.unwrap(), [Some(1.), Some(-1.), Some(1.)]);
}
