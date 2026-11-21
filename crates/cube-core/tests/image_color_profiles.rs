use cube_core::image::{
    ColorModel, Image,
    colorimetry::{Target, convert},
    perceptual::{Profile, digest},
    stereo::{AnaglyphProfile, EyeOrder, Mode, anaglyph, separate},
};

fn samples(width: u32, model: ColorModel, pixels: &[u8]) -> Image {
    Image::from_bytes(width, 1, model, pixels, &[], &[]).unwrap()
}

#[test]
fn color_coordinates_and_physical_alpha_are_explicit() {
    let image = samples(
        3,
        ColorModel::Rgba,
        &[255, 0, 0, 0, 0, 255, 255, 50, 128, 128, 128, 255],
    );
    let hsv = convert(&image, Target::Hsv).unwrap();
    assert_eq!(
        hsv.values,
        [[0., 1., 1.], [180., 1., 1.], [0., 0., 128. / 255.]]
    );
    assert_eq!(hsv.alpha.unwrap(), [0, 50, 255]);
    let hsl = convert(&image, Target::Hsl).unwrap();
    assert_eq!(hsl.values[1], [180., 1., 0.5]);
    let lab = convert(&image, Target::LabD50).unwrap();
    for (actual, expected) in
        lab.values[0]
            .into_iter()
            .zip([54.29054140467193, 80.80492817043522, 69.89096476862429])
    {
        assert!((actual - expected).abs() < 1e-6);
    }
    assert!(convert(&samples(1, ColorModel::Gray, &[1]), Target::Hsl).is_err());
}

#[test]
fn eye_order_and_projection_remain_explicit() {
    let rgba = samples(2, ColorModel::Rgba, &[4, 50, 98, 0, 55, 6, 78, 17]);
    let red_cyan = separate(&rgba, Mode::RedCyan, EyeOrder::LeftRight).unwrap();
    assert_eq!(red_cyan.left.samples(), [4, 0, 0, 0, 55, 0, 0, 17]);
    assert_eq!(red_cyan.right.samples(), [0, 50, 98, 0, 0, 6, 78, 17]);
    let split = separate(&rgba, Mode::SideBySide, EyeOrder::RightLeft).unwrap();
    assert_eq!(split.left.samples(), [55, 6, 78, 17]);
    assert_eq!(split.right.samples(), [4, 50, 98, 0]);
    let black = samples(1, ColorModel::Rgb, &[0, 0, 0]);
    let red = samples(1, ColorModel::Rgb, &[255, 0, 0]);
    let from_left = anaglyph(&red, &black, AnaglyphProfile::DuboisRedcyan2009).unwrap();
    let from_right = anaglyph(&black, &red, AnaglyphProfile::DuboisRedcyan2009).unwrap();
    assert_eq!(from_left.image.samples(), [177, 0, 0]);
    assert_eq!(from_right.image.samples(), [0, 165, 0]);
}

#[test]
fn flat_views_pin_digest_bit_order_and_tie_rules() {
    let flat = samples(1, ColorModel::Gray, &[9]);
    for (profile, bytes) in [
        (Profile::Ahash8, [255; 8]),
        (Profile::Dhash8, [0; 8]),
        (Profile::Phash32_8V1, [0; 8]),
    ] {
        assert_eq!(digest(&flat, profile).unwrap(), bytes);
    }
    let ramp = samples(2, ColorModel::Gray, &[255, 0]);
    assert_eq!(digest(&ramp, Profile::Ahash8).unwrap(), [0xf0; 8]);
    assert_eq!(digest(&ramp, Profile::Dhash8).unwrap(), [0x7e; 8]);
}
