use cube_core::{
    errors::ErrorCode,
    image::{ColorModel, Image, csv},
};

const RGB: &str = "cube_pixels,1,2,1,rgb,8\nx,y,red,green,blue\n0,0,1,2,3\n1,0,254,0,255\n";

#[test]
fn exact_samples_round_trip_in_every_supported_model_and_geometry() {
    assert_eq!(Image::from_csv(RGB).unwrap().to_csv().unwrap(), RGB);
    for model in [
        ColorModel::Gray,
        ColorModel::GrayAlpha,
        ColorModel::Rgb,
        ColorModel::Rgba,
    ] {
        for (width, height) in [(1, 1), (3, 2), (1, 3)] {
            let samples: Vec<_> = (0..width * height * model.channels() as u32)
                .map(|n| ((n * 37) % 256) as u8)
                .collect();
            let source = Image::from_bytes(width, height, model, &samples, &[], &[]).unwrap();
            let text = source.to_csv().unwrap();
            let result = Image::from_csv(&text).unwrap();
            assert_eq!(
                (result.width(), result.height(), result.model()),
                (width, height, model)
            );
            assert_eq!(result.samples(), samples);
            assert_eq!(result.to_csv().unwrap(), text);
            assert!(result.palette().is_empty() && result.transparency().is_empty());
            assert_eq!(
                Image::decode(&result.encode_png().unwrap())
                    .unwrap()
                    .samples(),
                samples
            );
        }
    }
    let quoted = "\"cube_pixels\",\"1\",\"2\",\"1\",\"rgb\",\"8\"\r\n\"x\",\"y\",\"red\",\"green\",\"blue\"\r\n\"1\",0,254,0,255\r\n0,0,1,2,3";
    assert_eq!(Image::from_csv(quoted).unwrap().to_csv().unwrap(), RGB);
    let alpha = Image::from_csv(
        "cube_pixels,1,2,1,rgba,8\nx,y,red,green,blue,alpha\n0,0,1,2,3,0\n1,0,1,2,3,255",
    )
    .unwrap();
    assert_eq!(alpha.samples(), [1, 2, 3, 0, 1, 2, 3, 255]);
}

#[test]
fn incomplete_invalid_and_unsupported_csv_is_rejected_with_safe_context() {
    let mut invalid = vec![
        String::new(),
        RGB.replace("cube_pixels", "pixels"),
        RGB.replace(",1,2,1,", ",2,2,1,"),
        RGB.replace(",rgb,8", ",rgb,16"),
        RGB.replace(",rgb,8", ",indexed,8"),
        RGB.replace(",rgb,8", ",cmyk,8"),
        RGB.replace("red,green,blue", "blue,green,red"),
        RGB.replace("0,0,1,2,3\n", ""),
        RGB.replace("1,0,254", "0,0,254"),
        RGB.replace("1,0,254", "2,0,254"),
        RGB.replace("1,0,254", "1,1,254"),
        RGB.replace("1,0,254", "-1,0,254"),
        RGB.replace("1,0,254", "4294967296,0,254"),
        RGB.replace("1,0,254", "1,0,256"),
        RGB.replace("1,0,254", "1,0,-1"),
        RGB.replace("1,0,254", "1,0,2.5"),
        RGB.replace("1,0,254", "1,0, 2"),
        RGB.replace("1,0,254", "1,0,+2"),
        RGB.replace("1,0,254", "1,0,0xff"),
        RGB.replace("1,0,254", "1,0,"),
        RGB.replace("1,0,254", "1,0,\"254"),
        RGB.replace("1,0,254", "1,0,25\"4"),
        RGB.replace("1,0,254", "1,0,\"25\"\"4\""),
        RGB.replace("1,0,254", "1,0,\"2\n54\""),
        RGB.replace("1,0,254", "1,0,\"2,54\""),
        RGB.replace("1,0,254", "1,0,\"254\"tail"),
        RGB.replace("1,0,254", "1,0,254,1"),
        RGB.replace("1,0,254", "1,0"),
        RGB.replace("1,0,254", "1,0,254\r"),
        format!("\u{feff}{RGB}"),
        RGB.replace("1,0,254", "1,0,é"),
        format!("{RGB}\n"),
        format!("{RGB}0,0,1,2,3\n"),
        RGB.replace("1,0,254,0,255\n", "\n"),
        RGB.replace(",2,1,rgb", ",0,1,rgb"),
        RGB.replace(",2,1,rgb", ",4294967295,4294967295,rgb"),
    ];
    // Deterministic parser mutation regression: accepted mutations must still
    // construct a complete valid image with a canonical round trip.
    for at in 0..RGB.len() {
        for byte in [b'"', b',', b'\r', b'\n', 0, 255] {
            let mut data = RGB.as_bytes().to_vec();
            data[at] = byte;
            if let Ok(text) = std::str::from_utf8(&data)
                && let Ok(image) = Image::from_csv(text)
            {
                assert_eq!(
                    image.samples().len(),
                    image.width() as usize * image.height() as usize * image.model().channels()
                );
                assert_eq!(
                    Image::from_csv(&image.to_csv().unwrap()).unwrap().samples(),
                    image.samples()
                );
            }
        }
    }
    for text in invalid.drain(..) {
        let error = Image::from_csv(&text).unwrap_err();
        assert_eq!(error.operation, Some("image.csv.import"));
        assert_eq!(error.parameter, Some("csv"));
    }
    let error = Image::from_csv(&RGB.replace("1,0,254", "1,0,256")).unwrap_err();
    assert!(error.message.contains("row 4, field red"));
    for image in [
        Image::from_bytes(1, 1, ColorModel::Indexed, &[0], &[[1, 2, 3]], &[0]).unwrap(),
        Image::from_bytes(1, 1, ColorModel::Gray, &[0], &[], &[0, 0]).unwrap(),
        Image::from_bytes(1, 1, ColorModel::Rgb, &[0, 0, 0], &[], &[0, 0, 0, 0, 0, 0]).unwrap(),
    ] {
        assert_eq!(
            image.to_csv().unwrap_err().code,
            ErrorCode::UnsupportedFormat
        );
    }
}

#[test]
fn csv_limits_apply_before_grid_allocation_and_export() {
    let image = Image::from_bytes(100, 100, ColorModel::Rgba, &vec![255; 40000], &[], &[]).unwrap();
    let text = image.to_csv().unwrap();
    assert_eq!(text.lines().count(), csv::MAX_RECORDS + 2);
    assert!(text.len() < csv::MAX_BYTES);
    assert_eq!(Image::from_csv(&text).unwrap().samples(), image.samples());
    assert_eq!(
        Image::from_bytes(101, 100, ColorModel::Gray, &vec![0; 10100], &[], &[])
            .unwrap()
            .to_csv()
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    for input in [
        "x".repeat(csv::MAX_BYTES + 1),
        "x".repeat(csv::MAX_ROW_BYTES + 1),
        format!(
            "cube_pixels,1,{},1,rgb,8",
            "0".repeat(csv::MAX_FIELD_BYTES + 1)
        ),
        "cube_pixels,1,101,100,rgba,8\nx,y,red,green,blue,alpha".into(),
    ] {
        assert_eq!(
            Image::from_csv(&input).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
    }
    assert!(Image::from_csv("cube_pixels,1,1,1,gray,8\nx,y,gray,extra,extra,extra,extra").is_err());
}
