use cube_core::{
    encoding::base64_decode,
    image::{
        Corner, Image, Region, Traversal,
        native_samples::{PngProfile, gif_indices, png_scanlines},
        traversal::permutation,
    },
};

const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAMAAAAFCAAAAAClGgl+AAAAAklEQVR4nGKkkSsAAAAaSURBVGNg+s/IyMzCyMTw7yczM/MfFtafTAAsvgUOLMbMDgAAAABJRU5ErkJggg==";
const GIF: &str = "R0lGODlhBAADAIAAAAoUHgoUHiH5BAEAAAEALAEAAQACAAEAAAICDAoAOw==";

#[test]
fn split_zlib_and_all_five_png_predictors_preserve_exact_rows() {
    let png = base64_decode(PNG).unwrap();
    let inflated = png_scanlines(&png, PngProfile::Filtered).unwrap();
    assert_eq!(
        inflated.data,
        [
            0, 2, 255, 1, 1, 3, 4, 1, 2, 0, 254, 249, 3, 3, 3, 252, 4, 5, 249, 2
        ]
    );
    assert_eq!(inflated.report.filter_offsets, [0, 4, 8, 12, 16]);
    let recovered = png_scanlines(&png, PngProfile::Reconstructed).unwrap();
    assert_eq!(
        recovered.data,
        [2, 255, 1, 3, 7, 8, 3, 5, 1, 4, 7, 0, 9, 2, 2]
    );
    assert_eq!(Image::decode(&png).unwrap().samples(), recovered.data);
    let mut damaged = png;
    damaged[47] ^= 1;
    assert!(png_scanlines(&damaged, PngProfile::Filtered).is_err());
}

#[test]
fn gif_native_indices_distinguish_duplicate_colors_and_frame_offset() {
    let data = base64_decode(GIF).unwrap();
    let frame = gif_indices(&data, 0).unwrap();
    assert_eq!(frame.indices, [1, 0]);
    assert_eq!(
        (frame.report.canvas_width, frame.report.canvas_height),
        (4, 3)
    );
    assert_eq!((frame.report.frame.x, frame.report.frame.y), (1, 1));
    assert_eq!(frame.report.palette, [[10, 20, 30], [10, 20, 30]]);
    assert_eq!(frame.report.transparent_index, Some(1));
    assert!(gif_indices(&data, 1).is_err());
    assert!(gif_indices(&data[..data.len() - 1], 0).is_err());
}

#[test]
fn traversal_curves_compose_corners_and_permutations() {
    let region = Region {
        x: 4,
        y: 6,
        width: 2,
        height: 2,
    };
    let points = |traversal: Traversal, corner, reverse| {
        traversal
            .coordinates(region, corner, reverse)
            .unwrap()
            .collect::<Vec<_>>()
    };
    assert_eq!(
        points(Traversal::Hilbert, Corner::TopLeft, false),
        [(4, 6), (4, 7), (5, 7), (5, 6)]
    );
    assert_eq!(
        points(Traversal::Hilbert, Corner::BottomLeft, false),
        [(4, 7), (4, 6), (5, 6), (5, 7)]
    );
    assert_eq!(
        points(Traversal::Morton, Corner::TopRight, true),
        [(4, 7), (5, 7), (4, 6), (5, 6)]
    );
    assert_eq!(
        points(Traversal::Serpentine, Corner::TopLeft, false),
        [(4, 6), (5, 6), (5, 7), (4, 7)]
    );
    assert_eq!(
        permutation(region, &[3, 1, 2, 0]).unwrap(),
        [[5, 7], [5, 6], [4, 7], [4, 6]]
    );
    assert!(permutation(region, &[0, 0, 1, 3]).is_err());
    assert!(
        Traversal::Hilbert
            .coordinates(Region { width: 3, ..region }, Corner::TopLeft, false)
            .is_err()
    );
}
