use cube_core::image::{
    Image,
    barcode::{Code39Checksum, Orientation, ScanOptions, Status, Symbology, scanlines},
    qr_encode::{Ecc, Options, Raster, Segment, encode},
};

#[test]
fn qr_forced_matrices_and_all_penalty_terms_match_independent_vectors() {
    let oracle: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/visual-codes/qr-masks-v1.json")).unwrap();
    let segments = [Segment::Byte { data: vec![0, 255] }];
    let options = Options {
        ecc: Ecc::L,
        version: Some(1),
        mask: None,
        raster: None,
    };
    let auto = encode(&segments, options).unwrap();
    for mask in 0..8u8 {
        let expected = &oracle[mask as usize];
        let forced = encode(
            &segments,
            Options {
                mask: Some(mask),
                ..options
            },
        )
        .unwrap();
        let matrix =
            cube_core::encoding::base64_decode(expected["modules"]["base64"].as_str().unwrap())
                .unwrap();
        assert_eq!(forced.modules, matrix, "mask {mask}");
        let penalty = serde_json::to_value(&auto.report.penalties[mask as usize]).unwrap();
        for key in ["mask", "runs", "blocks", "finder_like", "balance", "total"] {
            assert_eq!(penalty[key], expected[key], "mask {mask} {key}");
        }
    }
    for (ecc, max) in [(Ecc::L, 17), (Ecc::M, 14), (Ecc::Q, 11), (Ecc::H, 7)] {
        let options = Options {
            ecc,
            mask: Some(0),
            ..options
        };
        assert!(encode(&[Segment::Byte { data: vec![1; max] }], options).is_ok());
        assert!(
            encode(
                &[Segment::Byte {
                    data: vec![1; max + 1]
                }],
                options
            )
            .is_err()
        );
    }
    let options = Options {
        version: Some(40),
        mask: Some(7),
        ..options
    };
    assert_eq!(
        encode(
            &[Segment::Byte {
                data: vec![1; 2953]
            }],
            options
        )
        .unwrap()
        .modules
        .len(),
        177 * 177
    );
    assert!(
        encode(
            &[Segment::Byte {
                data: vec![1; 2954]
            }],
            options
        )
        .is_err()
    );
}

#[test]
fn qr_outputs_binary_safe_pixels_and_explicit_mask_scores() {
    let data = vec![0, 255, 42, 128];
    let result = encode(
        &[Segment::Byte { data: data.clone() }],
        Options {
            ecc: Ecc::L,
            version: Some(1),
            mask: None,
            raster: Some(Raster {
                quiet_zone: 4,
                scale: 5,
            }),
        },
    )
    .unwrap();
    assert_eq!(result.report.size, 21);
    assert_eq!(result.report.penalties.len(), 8);
    let best = result
        .report
        .penalties
        .iter()
        .min_by_key(|p| (p.total, p.mask))
        .unwrap();
    assert_eq!(result.report.mask, best.mask);
    let image = result.raster.unwrap();
    assert_eq!((image.width(), image.height()), (145, 145));
    let mut decoder = quircs::Quirc::default();
    let codes: Vec<_> = decoder
        .identify(145, 145, image.samples())
        .map(|code| code.unwrap().decode().unwrap())
        .collect();
    assert_eq!(codes.len(), 1);
    assert_eq!(codes[0].payload, data);
    assert!(
        encode(
            &[Segment::Numeric { text: "12a".into() }],
            Options {
                ecc: Ecc::L,
                version: None,
                mask: None,
                raster: None
            }
        )
        .is_err()
    );
}

#[test]
fn barcode_preserves_digits_and_orientation_and_validates_checksum() {
    // GS1 known EAN-13 4006381333931 with leading-4 LGLLGG parity.
    let left = [
        "0001101", "0100111", "0101111", "0111101", "0001001", "0110011",
    ]
    .concat();
    let right = [
        "1000010", "1000010", "1000010", "1110100", "1000010", "1100110",
    ]
    .concat();
    let modules = format!("000000000000101{left}01010{right}101000000000000");
    let pixels: Vec<_> = modules
        .bytes()
        .flat_map(|b| [if b == b'1' { 0 } else { 255 }; 3])
        .collect();
    let settings = ScanOptions {
        symbology: Symbology::Ean13,
        orientation: Orientation::Forward,
        threshold: 128,
    };
    let report = scanlines(&[pixels.clone(), vec![255; pixels.len()]], settings).unwrap();
    assert_eq!(report.rows[0].status, Status::Decoded);
    assert_eq!(
        report.rows[0].payload.as_deref(),
        Some(b"4006381333931".as_slice())
    );
    assert_eq!(report.rows[1].status, Status::NoDetection);
    let wrong = format!(
        "000000000000101{left}01010{}1101100101000000000000",
        &right[..35]
    );
    let wrong = wrong
        .bytes()
        .flat_map(|b| [if b == b'1' { 0 } else { 255 }; 3])
        .collect();
    assert_eq!(
        scanlines(&[wrong], settings).unwrap().rows[0].status,
        Status::Invalid
    );
    let reversed = pixels.into_iter().rev().collect();
    assert_eq!(
        scanlines(
            &[reversed],
            ScanOptions {
                orientation: Orientation::Reverse,
                ..settings
            }
        )
        .unwrap()
        .rows[0]
            .payload
            .as_deref(),
        Some(b"4006381333931".as_slice()),
    );
}

#[test]
fn independent_linear_codes_preserve_checksums_switches_and_fnc1() {
    for (symbology, fixture, payload, algorithm) in [
        (
            Symbology::Upca,
            include_bytes!("fixtures/visual-codes/upca.png").as_slice(),
            b"036000291452".as_slice(),
            "mod10",
        ),
        (
            Symbology::Code39 {
                checksum: Code39Checksum::Mod43,
            },
            include_bytes!("fixtures/visual-codes/code39.png").as_slice(),
            b"CODE39".as_slice(),
            "mod43",
        ),
        (
            Symbology::Code128,
            include_bytes!("fixtures/visual-codes/code128.png").as_slice(),
            b"AB123456Z".as_slice(),
            "mod103",
        ),
    ] {
        let image = Image::decode(fixture).unwrap();
        let settings = ScanOptions {
            symbology,
            orientation: Orientation::Forward,
            threshold: 128,
        };
        let result = scanlines(
            &[image.samples()[..image.width() as usize].to_vec()],
            settings,
        )
        .unwrap();
        let row = &result.rows[0];
        assert_eq!(row.status, Status::Decoded, "{symbology:?}");
        assert_eq!(row.payload.as_deref(), Some(payload), "{symbology:?}");
        let checksum = row.checksum.as_ref().unwrap();
        assert_eq!(checksum.algorithm, algorithm);
        assert!(checksum.verified);
        if matches!(symbology, Symbology::Code128) {
            let codes = row.codewords.as_ref().unwrap();
            assert_eq!(codes.last(), Some(&106));
            assert!(codes.contains(&99));
            assert!(codes.contains(&102));
        }
    }
    let settings = ScanOptions {
        symbology: Symbology::Upca,
        orientation: Orientation::Forward,
        threshold: 128,
    };
    assert!(scanlines(&vec![vec![255]; 1024], settings).is_ok());
    assert!(scanlines(&vec![vec![255]; 1025], settings).is_err());
    assert!(scanlines(&[vec![255; 4096]], settings).is_ok());
    assert!(scanlines(&[vec![255; 4097]], settings).is_err());
    let noisy = [0, 255].repeat(2048);
    assert!(scanlines(&vec![noisy; 64], settings).is_err());
}
