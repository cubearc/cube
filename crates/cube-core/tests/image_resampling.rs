use cube_core::{
    errors::ErrorCode,
    image::{
        ColorModel, Image,
        resample::{Border, Kernel, Transform, WarpKernel, resize, warp},
    },
};

fn gray(width: u32, height: u32, samples: &[u8]) -> Image {
    Image::from_bytes(width, height, ColorModel::Gray, samples, &[], &[]).unwrap()
}

#[test]
fn resized_centers_use_explicit_kernels() {
    let row = gray(2, 1, &[0, 200]);
    for (kernel, expected) in [
        (Kernel::Nearest, [0, 0, 200, 200]),
        (Kernel::Bilinear, [0, 50, 150, 200]),
        (Kernel::Bicubic, [0, 41, 159, 214]),
    ] {
        let result = resize(&row, 4, 1, kernel).unwrap();
        assert_eq!(result.samples(), expected);
        assert_eq!(result.info().origin, "top_left");
    }
    let grid = gray(2, 3, &[0, 40, 80, 120, 160, 200]);
    let result = resize(&grid, 1, 2, Kernel::Bilinear).unwrap();
    assert_eq!(result.samples(), [40, 160]);
    for kernel in [Kernel::Nearest, Kernel::Bilinear, Kernel::Bicubic] {
        assert_eq!(
            resize(&grid, 2, 3, kernel).unwrap().samples(),
            grid.samples()
        );
    }
}

#[test]
fn warps_map_integer_centers_and_use_the_selected_border() {
    let row = gray(2, 1, &[20, 100]);
    let shift = Transform::Affine {
        matrix: [[1., 0., -0.5], [0., 1., 0.]],
    };
    let constant = Border::Constant { samples: vec![10] };
    assert_eq!(
        warp(&row, 3, 1, shift, WarpKernel::Bilinear, &constant)
            .unwrap()
            .samples(),
        [15, 60, 55],
    );
    assert_eq!(
        warp(&row, 3, 1, shift, WarpKernel::Bilinear, &Border::Clamp)
            .unwrap()
            .samples(),
        [20, 60, 100],
    );
    let projective = Transform::Projective {
        matrix: [[2., 0., 0.], [0., 2., 0.], [1., 0., 1.]],
    };
    assert_eq!(
        warp(&row, 2, 1, projective, WarpKernel::Nearest, &constant)
            .unwrap()
            .samples(),
        [20, 100],
    );
}

#[test]
fn rejects_alpha_keys_invalid_transforms_and_excess_work() {
    let keyed = Image::from_bytes(1, 1, ColorModel::Gray, &[7], &[], &[0, 7]).unwrap();
    assert!(resize(&keyed, 1, 1, Kernel::Nearest).is_err());
    let rgba = Image::from_bytes(
        2,
        1,
        ColorModel::Rgba,
        &[2, 3, 4, 0, 102, 103, 104, 2],
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(
        resize(&rgba, 1, 1, Kernel::Bilinear).unwrap().samples(),
        [52, 53, 54, 1],
    );
    let row = gray(2, 1, &[20, 100]);
    let identity = Transform::Affine {
        matrix: [[1., 0., 0.], [0., 1., 0.]],
    };
    for bad in [
        Transform::Affine {
            matrix: [[1., 2., 0.], [2., 4., 0.]],
        },
        Transform::Affine {
            matrix: [[1., 0., f64::NAN], [0., 1., 0.]],
        },
        Transform::Projective {
            matrix: [[1., 0., 0.], [0., 1., 0.], [1., 0., -1.]],
        },
    ] {
        assert!(warp(&row, 2, 1, bad, WarpKernel::Nearest, &Border::Clamp).is_err());
    }
    assert!(
        warp(
            &row,
            2,
            1,
            identity,
            WarpKernel::Nearest,
            &Border::Constant {
                samples: vec![1, 2]
            }
        )
        .is_err()
    );
    assert_eq!(
        resize(&rgba, 2048, 1024, Kernel::Bicubic).unwrap_err().code,
        ErrorCode::ResourceLimit,
    );
}
