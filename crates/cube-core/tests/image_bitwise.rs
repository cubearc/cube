use cube_core::{
    binary,
    errors::ErrorCode,
    image::{
        BitwiseOperation as Op, ColorModel as Model, Image, MAX_DIMENSION, MAX_PIXELS, MAX_SAMPLES,
    },
    operations::{self, ParameterValue},
};

#[test]
fn known_pairs_cover_every_direct_model_alpha_and_lossless_export() {
    for model in [Model::Gray, Model::GrayAlpha, Model::Rgb, Model::Rgba] {
        let left = Image::from_samples(
            2,
            2,
            model,
            [0, 255, 170, 85].repeat(model.channels()),
            vec![],
            vec![],
        )
        .unwrap();
        let right = Image::from_samples(
            2,
            2,
            model,
            [255, 15, 85, 170].repeat(model.channels()),
            vec![],
            vec![],
        )
        .unwrap();
        for (operation, expected) in [
            (Op::Xor, [255, 240, 255, 255]),
            (Op::And, [0, 15, 0, 0]),
            (Op::Or, [255, 255, 255, 255]),
        ] {
            let combined = left.bitwise(&right, operation).unwrap();
            assert_eq!(combined.samples(), expected.repeat(model.channels()));
            assert_eq!(
                (combined.width(), combined.height(), combined.model()),
                (2, 2, model)
            );
            let restored = Image::decode(&combined.encode_png().unwrap()).unwrap();
            assert_eq!(restored.samples(), combined.samples());
            assert_eq!(restored.model(), model);
            assert_eq!(left.samples(), [0, 255, 170, 85].repeat(model.channels()));
            assert_eq!(right.samples(), [255, 15, 85, 170].repeat(model.channels()));
        }
        assert!(
            left.bitwise(&left, Op::Xor)
                .unwrap()
                .samples()
                .iter()
                .all(|n| *n == 0)
        );
        assert_eq!(
            left.bitwise(&right, Op::Xor)
                .unwrap()
                .bitwise(&right, Op::Xor)
                .unwrap()
                .samples(),
            left.samples()
        );
        assert_eq!(
            left.compare(&right, None, None, 100, false)
                .unwrap()
                .differing_pixels,
            4
        );
    }
    let left = Image::from_bytes(1, 1, Model::Rgba, &[10, 20, 30, 0], &[], &[]).unwrap();
    let right = Image::from_bytes(1, 1, Model::Rgba, &[10, 20, 30, 255], &[], &[]).unwrap();
    assert_eq!(
        left.bitwise(&right, Op::Xor).unwrap().samples(),
        [0, 0, 0, 255]
    );
    assert_eq!(
        left.bitwise(&right, Op::And).unwrap().samples(),
        [10, 20, 30, 0]
    );
    assert_eq!(
        left.bitwise(&right, Op::Or).unwrap().samples(),
        [10, 20, 30, 255]
    );
    // The existing byte XOR keeps its repeating-key contract.
    assert_eq!(binary::xor(&[0, 1, 2], &[255]).unwrap(), [255, 254, 253]);
}

#[test]
fn native_indices_preserve_exact_palette_transparency_and_validate_results() {
    let palette = [[1, 2, 3], [4, 5, 6], [7, 8, 9], [10, 11, 12]];
    let left =
        Image::from_bytes(3, 1, Model::Indexed, &[0, 1, 2], &palette, &[0, 64, 128]).unwrap();
    let right =
        Image::from_bytes(3, 1, Model::Indexed, &[1, 2, 0], &palette, &[0, 64, 128]).unwrap();
    for (operation, expected, second_alpha) in [
        (Op::Xor, [1, 3, 2], 255),
        (Op::And, [0, 0, 0], 0),
        (Op::Or, [1, 3, 2], 255),
    ] {
        let output = left.bitwise(&right, operation).unwrap();
        assert_eq!(output.samples(), expected);
        assert_eq!(output.palette(), palette);
        assert_eq!(output.transparency(), [0, 64, 128]);
        assert_eq!(output.expand_palette().unwrap().samples()[7], second_alpha);
        let restored = Image::decode(&output.encode_png().unwrap()).unwrap();
        assert_eq!(restored.samples(), output.samples());
        assert_eq!(restored.palette(), palette);
        assert_eq!(restored.transparency(), output.transparency());
    }
    let left = Image::from_bytes(1, 1, Model::Indexed, &[1], &palette[..3], &[]).unwrap();
    let right = Image::from_bytes(1, 1, Model::Indexed, &[2], &palette[..3], &[]).unwrap();
    for operation in [Op::Xor, Op::Or] {
        let error = left.bitwise(&right, operation).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidInput);
        assert_eq!(error.operation, Some("image.bitwise"));
    }
    assert_eq!(left.bitwise(&right, Op::And).unwrap().samples(), [0]);
    assert_eq!(left.samples(), [1]);
    assert_eq!(right.samples(), [2]);
}

#[test]
fn mismatched_geometry_models_palettes_and_raw_transparency_fail_explicitly() {
    let gray = Image::from_bytes(2, 1, Model::Gray, &[1, 2], &[], &[]).unwrap();
    let palette = [[0, 0, 0], [255, 255, 255], [3, 4, 5]];
    for other in [
        Image::from_bytes(1, 2, Model::Gray, &[1, 2], &[], &[]).unwrap(),
        Image::from_bytes(2, 1, Model::Indexed, &[1, 2], &palette, &[]).unwrap(),
        Image::from_bytes(2, 1, Model::GrayAlpha, &[1, 2, 3, 4], &[], &[]).unwrap(),
        Image::from_bytes(2, 1, Model::Gray, &[1, 2], &[], &[0, 1]).unwrap(),
    ] {
        for operation in [Op::Xor, Op::And, Op::Or] {
            let error = gray.bitwise(&other, operation).unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidParameter);
            assert_eq!(error.operation, Some("image.bitwise"));
            assert_eq!(error.parameter, Some("other"));
        }
    }
    let indexed = Image::from_bytes(1, 1, Model::Indexed, &[1], &palette, &[0]).unwrap();
    for other in [
        Image::from_bytes(
            1,
            1,
            Model::Indexed,
            &[1],
            &[[0, 0, 0], [1, 2, 3], [3, 4, 5]],
            &[0],
        )
        .unwrap(),
        Image::from_bytes(1, 1, Model::Indexed, &[1], &palette, &[0, 255]).unwrap(),
    ] {
        assert!(indexed.bitwise(&other, Op::Xor).is_err());
        // An explicit expansion makes the physical RGBA samples comparable.
        assert!(
            indexed
                .expand_palette()
                .unwrap()
                .bitwise(&other.expand_palette().unwrap(), Op::Xor)
                .is_ok()
        );
    }
    // Non-square geometry is preserved, not transposed or implicitly resized.
    let rectangle = Image::from_bytes(3, 2, Model::Gray, &[0, 1, 2, 3, 4, 5], &[], &[]).unwrap();
    let output = rectangle.bitwise(&rectangle, Op::Or).unwrap();
    assert_eq!(
        (output.width(), output.height(), output.samples()),
        (3, 2, rectangle.samples())
    );
}

#[test]
fn color_keys_remain_literal_and_source_metadata_is_not_reused() {
    for (model, samples, key) in [
        (Model::Gray, vec![17], vec![0, 0]),
        (Model::Rgb, vec![1, 2, 3], vec![0, 0, 0, 0, 0, 0]),
    ] {
        let image = Image::from_samples(1, 1, model, samples, vec![], key.clone()).unwrap();
        let decoded = Image::decode(&image.encode_png().unwrap()).unwrap();
        assert!(!decoded.info().chunks.is_empty());
        let output = decoded.bitwise(&decoded, Op::Xor).unwrap();
        assert!(output.samples().iter().all(|n| *n == 0));
        assert_eq!(output.transparency(), key);
        assert!(output.info().chunks.is_empty());
        assert!(!output.info().has_exif);
        assert_eq!(
            output.provenance().representation,
            "constructed_8bit_samples"
        );
        assert!(!output.provenance().source_samples_preserved);
        assert!(decoded.provenance().source_samples_preserved);
    }
}

#[test]
fn exact_sample_budget_succeeds_and_invalid_shapes_never_reach_combination() {
    let left = Image::from_samples(
        2048,
        2048,
        Model::Rgba,
        vec![170; MAX_SAMPLES],
        vec![],
        vec![],
    )
    .unwrap();
    let right = Image::from_samples(
        2048,
        2048,
        Model::Rgba,
        vec![85; MAX_SAMPLES],
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(
        u64::from(left.width()) * u64::from(left.height()),
        MAX_PIXELS
    );
    for (operation, expected) in [(Op::Xor, 255), (Op::And, 0), (Op::Or, 255)] {
        let output = left.bitwise(&right, operation).unwrap();
        assert_eq!(output.samples().len(), MAX_SAMPLES);
        assert!(output.samples().iter().all(|v| *v == expected));
    }
    assert_eq!(
        Image::from_bytes(MAX_DIMENSION + 1, 1, Model::Gray, &[], &[], &[])
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        Image::from_bytes(2048, 2049, Model::Rgba, &[], &[], &[])
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    for samples in [&[][..], &[0, 1][..]] {
        assert!(Image::from_bytes(1, 1, Model::Gray, samples, &[], &[]).is_err());
    }
    assert!(Image::from_bytes(0, 1, Model::Gray, &[], &[], &[]).is_err());
}

#[test]
fn metadata_requires_an_exact_operation_and_keeps_pipeline_support_explicit() {
    let descriptor = operations::require("image.bitwise").unwrap();
    assert_eq!(descriptor.input_type, "image.sample_pair");
    assert_eq!(descriptor.output_type, "image.samples");
    assert!(descriptor.discovery().pipeline.is_none());
    assert!(descriptor.validate(&[]).is_err());
    for name in ["xor", "and", "or"] {
        assert!(Op::parse(name).is_ok());
        descriptor
            .validate(&[("operation", ParameterValue::Text(name))])
            .unwrap();
    }
    for name in ["", "XOR", "not", " xor", "and "] {
        let error = Op::parse(name).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidParameter);
        assert_eq!(error.operation, Some("image.bitwise"));
        assert_eq!(error.parameter, Some("operation"));
        assert!(
            descriptor
                .validate(&[("operation", ParameterValue::Text(name))])
                .is_err()
        );
    }
}
