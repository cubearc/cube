use cube_core::image::{ColorModel, Corner, Image, LsbOptions, Region, Traversal};
fn region(w: u32, h: u32) -> Region {
    Region {
        x: 0,
        y: 0,
        width: w,
        height: h,
    }
}
#[test]
fn exact_diagonals_zigzags_and_directed_spirals() {
    for (pattern, expected) in [
        (
            "diagonal",
            vec![
                (0, 0),
                (0, 1),
                (1, 0),
                (0, 2),
                (1, 1),
                (2, 0),
                (1, 2),
                (2, 1),
                (2, 2),
            ],
        ),
        (
            "zigzag",
            vec![
                (0, 0),
                (1, 0),
                (0, 1),
                (0, 2),
                (1, 1),
                (2, 0),
                (2, 1),
                (1, 2),
                (2, 2),
            ],
        ),
        (
            "spiral_cw",
            vec![
                (0, 0),
                (1, 0),
                (2, 0),
                (2, 1),
                (2, 2),
                (1, 2),
                (0, 2),
                (0, 1),
                (1, 1),
            ],
        ),
        (
            "spiral_ccw",
            vec![
                (0, 0),
                (0, 1),
                (0, 2),
                (1, 2),
                (2, 2),
                (2, 1),
                (2, 0),
                (1, 0),
                (1, 1),
            ],
        ),
    ] {
        let actual: Vec<_> = Traversal::parse(pattern)
            .unwrap()
            .coordinates(region(3, 3), Corner::TopLeft, false)
            .unwrap()
            .collect();
        assert_eq!(actual, expected, "{pattern}");
    }
    let image = Image::from_bytes(3, 2, ColorModel::Gray, &[0, 1, 2, 3, 4, 5], &[], &[]).unwrap();
    for (pattern, corner, expected) in [
        (Traversal::SpiralCw, Corner::TopLeft, vec![0, 1, 2, 5, 4, 3]),
        (
            Traversal::SpiralCcw,
            Corner::TopLeft,
            vec![0, 3, 4, 5, 2, 1],
        ),
        (
            Traversal::SpiralCw,
            Corner::TopRight,
            vec![2, 5, 4, 3, 0, 1],
        ),
    ] {
        let result = image
            .lsb_with_corner(
                &LsbOptions {
                    bits: (0..8).rev().collect(),
                    traversal: pattern,
                    ..Default::default()
                },
                corner,
            )
            .unwrap();
        assert_eq!(result.result.data, expected);
    }
}
#[test]
fn every_coordinate_is_visited_once_for_all_small_shapes_corners_and_reversal() {
    for w in 1..=8 {
        for h in 1..=8 {
            for name in cube_core::image::traversal::NAMES {
                for corner in cube_core::image::traversal::CORNER_NAMES {
                    let region = Region {
                        x: 5,
                        y: 7,
                        ..region(w, h)
                    };
                    let order = Traversal::parse(name).unwrap();
                    let corner = Corner::parse(corner).unwrap();
                    let mut iter = order.coordinates(region, corner, false).unwrap();
                    assert_eq!(iter.len(), (w * h) as usize);
                    let forward: Vec<_> = iter.by_ref().collect();
                    assert!(iter.next().is_none());
                    assert_eq!(iter.len(), 0);
                    let unique: std::collections::BTreeSet<_> = forward.iter().copied().collect();
                    assert_eq!(unique.len(), (w * h) as usize);
                    assert!(
                        unique
                            .iter()
                            .all(|(x, y)| *x >= 5 && *x < 5 + w && *y >= 7 && *y < 7 + h)
                    );
                    let reverse: Vec<_> =
                        order.coordinates(region, corner, true).unwrap().collect();
                    assert_eq!(reverse, forward.into_iter().rev().collect::<Vec<_>>());
                }
            }
        }
    }
    assert!(Corner::parse("middle").is_err());
    for r in [
        region(0, 1),
        region(1, 0),
        region(16384, 16384),
        Region {
            x: u32::MAX,
            ..region(1, 1)
        },
    ] {
        assert!(
            Traversal::Row
                .coordinates(r, Corner::TopLeft, false)
                .is_err()
        );
    }
}
