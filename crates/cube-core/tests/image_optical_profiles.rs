use cube_core::{
    errors::ErrorCode,
    image::{
        ColorModel, Image,
        optical::{Format, Options, Orientation, Status, decode},
    },
};

#[test]
fn independent_zxing_fixtures_cover_each_detector() {
    for (format, bytes, expected) in [
        (
            Format::DataMatrix,
            include_bytes!("fixtures/visual-codes/datamatrix.png").as_slice(),
            "0123456789",
        ),
        (
            Format::Aztec,
            include_bytes!("fixtures/visual-codes/aztec.png").as_slice(),
            "abcdefghijklmnopqrstuvwxyz",
        ),
        (
            Format::Aztec,
            include_bytes!("fixtures/visual-codes/aztec-full.png").as_slice(),
            "Cube\0ÿ",
        ),
        (
            Format::DataMatrix,
            include_bytes!("fixtures/visual-codes/datamatrix-eci.png").as_slice(),
            "Cube\0ÿ",
        ),
        (
            Format::Pdf417,
            include_bytes!("fixtures/visual-codes/pdf417.png").as_slice(),
            "This is PDF417",
        ),
    ] {
        let image = Image::decode(bytes).unwrap();
        let settings = Options {
            format,
            orientation: Orientation::Deg0,
        };
        let result = decode(&[&image], settings).unwrap();
        assert_eq!(result.crops[0].status, Status::Decoded, "{format:?}");
        assert_eq!(result.crops[0].text.as_deref(), Some(expected));
        assert!(!result.crops[0].points.is_empty());
        for (rotation, orientation) in [
            (90, Orientation::Deg270),
            (180, Orientation::Deg180),
            (270, Orientation::Deg90),
        ] {
            let rotated = image.rotate(rotation).unwrap();
            let result = decode(
                &[&rotated],
                Options {
                    orientation,
                    ..settings
                },
            )
            .unwrap();
            assert_eq!(result.crops[0].text.as_deref(), Some(expected));
        }
    }
}

#[test]
fn pdf417_corrects_a_codeword_without_trusting_an_encoded_dimension() {
    let image = Image::decode(include_bytes!("fixtures/visual-codes/pdf417.png")).unwrap();
    let (width, height) = (image.width() as usize, image.height() as usize);
    let mut samples = image.samples().to_vec();
    let (top, x) = samples
        .iter()
        .position(|&v| v < 128)
        .map(|i| (i / width, i % width))
        .unwrap();
    let first = &image.samples()[top * width..(top + 1) * width];
    let scale = first[x..].iter().take_while(|&&v| v < 128).count() / 8;
    let word = x + 34 * scale..x + 51 * scale;
    for y in top..height {
        if image.samples()[y * width..(y + 1) * width]
            .iter()
            .zip(first)
            .any(|(sample, initial)| (*sample < 128) != (*initial < 128))
        {
            break;
        }
        samples[y * width + word.start..y * width + word.end].fill(255);
    }
    let altered = Image::from_bytes(
        image.width(),
        image.height(),
        ColorModel::Gray,
        &samples,
        &[],
        &[],
    )
    .unwrap();
    let settings = Options {
        format: Format::Pdf417,
        orientation: Orientation::Deg0,
    };
    let reference = decode(&[&image], settings).unwrap();
    let corrected = decode(&[&altered], settings).unwrap();
    assert_eq!(reference.crops[0].text, corrected.crops[0].text);
    assert_eq!(reference.crops[0].codewords, corrected.crops[0].codewords);
    assert_eq!(corrected.crops[0].corrected_errors, Some(1));
}

#[test]
fn optical_preflight_bounds_large_negative_crops() {
    let negative: Vec<u8> = (0u32..262_144)
        .map(|i| {
            let x = i.wrapping_mul(2_654_435_761);
            (x ^ (x >> 13) ^ (x >> 17)) as u8
        })
        .collect();
    let exact = Image::from_bytes(1024, 256, ColorModel::Gray, &negative, &[], &[]).unwrap();
    let over = Image::from_bytes(1025, 1, ColorModel::Gray, &[255; 1025], &[], &[]).unwrap();
    for format in [Format::DataMatrix, Format::Aztec, Format::Pdf417] {
        let settings = Options {
            format,
            orientation: Orientation::Deg0,
        };
        let repeat = if matches!(format, Format::Pdf417) {
            4
        } else {
            2
        };
        let start = std::time::Instant::now();
        let result = decode(&vec![&exact; repeat], settings).unwrap();
        eprintln!(
            "{format:?}: {} work units, {:?}",
            result.charged_steps,
            start.elapsed()
        );
        assert!(result.crops.iter().all(|row| row.text.is_none()));
        assert_eq!(
            decode(&[&over], settings).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
        assert_eq!(
            decode(&[&exact; 5], settings).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
    }
}

#[test]
fn blank_and_uncertain_crops_do_not_invent_payload() {
    let blank = Image::from_bytes(40, 40, ColorModel::Gray, &[255; 1600], &[], &[]).unwrap();
    let noise: Vec<_> = (0..1600)
        .map(|i| if (i * 13 + i / 7) % 9 < 3 { 0 } else { 255 })
        .collect();
    let uncertain = Image::from_bytes(40, 40, ColorModel::Gray, &noise, &[], &[]).unwrap();
    for format in [Format::DataMatrix, Format::Aztec, Format::Pdf417] {
        let settings = Options {
            format,
            orientation: Orientation::Deg0,
        };
        let result = decode(&[&blank, &uncertain], settings).unwrap();
        assert_eq!(result.crops[0].status, Status::NoDetection);
        assert!(result.crops.iter().all(|row| row.text.is_none()));
        assert_eq!(
            decode(&[&blank; 9], settings).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
    }
}
