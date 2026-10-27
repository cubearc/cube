use cube_core::image::{ColorModel, Image};
use std::{path::PathBuf, process::Command};
struct Files(PathBuf);
impl Files {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("cube-image-{}-{name}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        Self(dir)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn image_pairs_plan_and_export_reuse_single_pair_contracts() {
    let files = Files::new("pair-batches");
    let a = files.path("a.png");
    let b = files.path("b.png");
    Image::from_bytes(1, 1, ColorModel::Gray, &[170], &[], &[])
        .unwrap()
        .save_png(&a)
        .unwrap();
    Image::from_bytes(1, 1, ColorModel::Gray, &[85], &[], &[])
        .unwrap()
        .save_png(&b)
        .unwrap();
    let result = run(&[
        "image",
        "pairs",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
        "--operation",
        "xor",
        "--plan",
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let plan: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(plan["pair_count"], 1);
    let dir = files.path("pairs");
    std::fs::create_dir(&dir).unwrap();
    let args = [
        "image",
        "pairs",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
        "--operation",
        "xor",
        "--output-dir",
        dir.to_str().unwrap(),
        "--json",
    ];
    assert!(run(&args).status.success());
    assert_eq!(
        Image::open(&dir.join("pair-00-01.png")).unwrap().samples(),
        [255]
    );
    assert!(!run(&args).status.success());
    assert!(
        !run(&[
            "image",
            "pairs",
            a.to_str().unwrap(),
            a.to_str().unwrap(),
            "--operation",
            "compare",
            "--plan"
        ])
        .status
        .success()
    );
}
#[test]
fn color_layer_exports_have_deterministic_manifests_and_collision_safety() {
    let files = Files::new("color-exports");
    let source = files.path("source.png");
    let image = Image::from_bytes(2, 1, ColorModel::Gray, &[0, 255], &[], &[]).unwrap();
    image.save_png(&source).unwrap();
    let dir = files.path("layers");
    std::fs::create_dir(&dir).unwrap();
    let args = [
        "image",
        "color-layers",
        source.to_str().unwrap(),
        "--output-dir",
        dir.to_str().unwrap(),
    ];
    let result = run(&args);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let manifest: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(manifest["outputs"][0]["name"], "color-00.png");
    assert_eq!(
        Image::open(&dir.join("color-00.png")).unwrap().samples(),
        [255, 0]
    );
    let original = std::fs::read(dir.join("manifest.json")).unwrap();
    assert!(!run(&args).status.success());
    assert_eq!(std::fs::read(dir.join("manifest.json")).unwrap(), original);
    assert_eq!(Image::open(&source).unwrap().samples(), image.samples());
}

#[test]
fn bitwise_pairs_export_exact_samples_and_constructed_provenance() {
    let files = Files::new("bitwise-pairs");
    let first = files.path("first.png");
    let second = files.path("second.png");
    for model in [
        ColorModel::Gray,
        ColorModel::GrayAlpha,
        ColorModel::Rgb,
        ColorModel::Rgba,
    ] {
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
        // Each model gets fresh inputs; the CLI must never change them.
        std::fs::write(&first, left.encode_png().unwrap()).unwrap();
        std::fs::write(&second, right.encode_png().unwrap()).unwrap();
        let original = [
            std::fs::read(&first).unwrap(),
            std::fs::read(&second).unwrap(),
        ];
        for (operation, expected) in [
            ("xor", [255, 240, 255, 255]),
            ("and", [0, 15, 0, 0]),
            ("or", [255, 255, 255, 255]),
        ] {
            let output = files.path(&format!("{model:?}-{operation}.png"));
            let result = run(&[
                "image",
                "bitwise",
                first.to_str().unwrap(),
                second.to_str().unwrap(),
                "--operation",
                operation,
                "--output",
                output.to_str().unwrap(),
                "--json",
            ]);
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert!(result.stderr.is_empty());
            let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
            assert_eq!(report["operation"], operation);
            assert_eq!(report["image"]["sample_bytes"], 4 * model.channels());
            assert_eq!(
                report["provenance"]["representation"],
                "constructed_8bit_samples"
            );
            assert_eq!(report["provenance"]["source_samples_preserved"], false);
            let combined = Image::open(&output).unwrap();
            assert_eq!(combined.samples(), expected.repeat(model.channels()));
            assert_eq!(combined.model(), model);
            assert_eq!(
                [
                    std::fs::read(&first).unwrap(),
                    std::fs::read(&second).unwrap()
                ],
                original
            );
        }
    }
    let output = files.path("human-output.png");
    let result = run(&[
        "image",
        "bitwise",
        first.to_str().unwrap(),
        second.to_str().unwrap(),
        "--operation",
        "xor",
        "--output",
        output.to_str().unwrap(),
    ]);
    assert!(result.status.success());
    assert_eq!(result.stdout, b"Wrote PNG8 2 x 2\n");
    let result = run(&["image", "bitwise", "--help"]);
    assert!(result.status.success());
    let help = String::from_utf8_lossy(&result.stdout);
    assert!(help.contains("--operation") && help.contains("--output"));
    let result = run(&["ops", "describe", "image.bitwise", "--json"]);
    assert!(result.status.success());
    let descriptor: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(descriptor["operation"]["id"], "image.bitwise");
    assert_eq!(descriptor["operation"]["input_type"], "image.sample_pair");
    assert!(descriptor["operation"]["pipeline"].is_null());
}

#[test]
fn bitwise_indexed_outputs_validate_indices_and_require_explicit_expansion() {
    let files = Files::new("bitwise-indexed");
    let first = files.path("first.png");
    let second = files.path("second.png");
    let palette = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
    Image::from_bytes(1, 1, ColorModel::Indexed, &[1], &palette, &[0, 128])
        .unwrap()
        .save_png(&first)
        .unwrap();
    Image::from_bytes(1, 1, ColorModel::Indexed, &[2], &palette, &[0, 128])
        .unwrap()
        .save_png(&second)
        .unwrap();
    let output = files.path("invalid.png");
    let result = run(&[
        "--json-errors",
        "image",
        "bitwise",
        first.to_str().unwrap(),
        second.to_str().unwrap(),
        "--operation",
        "xor",
        "--output",
        output.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(1));
    let error: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(error["error"]["code"], "invalid_input");
    assert_eq!(error["error"]["operation"], "image.bitwise");
    assert!(!output.exists());
    let output = files.path("native.png");
    assert!(
        run(&[
            "image",
            "bitwise",
            first.to_str().unwrap(),
            second.to_str().unwrap(),
            "--operation",
            "and",
            "--output",
            output.to_str().unwrap()
        ])
        .status
        .success()
    );
    let combined = Image::open(&output).unwrap();
    assert_eq!(combined.samples(), [0]);
    assert_eq!(combined.palette(), palette);
    assert_eq!(combined.transparency(), [0, 128]);
    let rgba_first = files.path("first-rgba.png");
    let rgba_second = files.path("second-rgba.png");
    for (source, destination) in [(&first, &rgba_first), (&second, &rgba_second)] {
        assert!(
            run(&[
                "image",
                "convert",
                source.to_str().unwrap(),
                "--expand-palette",
                "--output",
                destination.to_str().unwrap()
            ])
            .status
            .success()
        );
    }
    let output = files.path("colors.png");
    assert!(
        run(&[
            "image",
            "bitwise",
            rgba_first.to_str().unwrap(),
            rgba_second.to_str().unwrap(),
            "--operation",
            "xor",
            "--output",
            output.to_str().unwrap()
        ])
        .status
        .success()
    );
    let combined = Image::open(&output).unwrap();
    assert_eq!(combined.model(), ColorModel::Rgba);
    assert_eq!(combined.samples(), [3, 13, 15, 127]);
}

#[test]
fn bitwise_failures_preserve_inputs_and_never_create_or_replace_outputs() {
    let files = Files::new("bitwise-failures");
    let first = files.path("first.png");
    Image::from_bytes(2, 1, ColorModel::Gray, &[1, 2], &[], &[])
        .unwrap()
        .save_png(&first)
        .unwrap();
    let original = std::fs::read(&first).unwrap();
    let second = files.path("second.png");
    let output = files.path("output.png");
    for other in [
        Image::from_bytes(1, 2, ColorModel::Gray, &[1, 2], &[], &[]).unwrap(),
        Image::from_bytes(2, 1, ColorModel::GrayAlpha, &[1, 2, 3, 4], &[], &[]).unwrap(),
    ] {
        std::fs::write(&second, other.encode_png().unwrap()).unwrap();
        let result = run(&[
            "--json-errors",
            "image",
            "bitwise",
            first.to_str().unwrap(),
            second.to_str().unwrap(),
            "--operation",
            "xor",
            "--output",
            output.to_str().unwrap(),
        ]);
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        let error: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
        assert_eq!(error["error"]["operation"], "image.bitwise");
        assert!(!output.exists());
    }
    for data in [
        &[][..],
        &b"not an image"[..],
        &original[..original.len() - 1],
    ] {
        std::fs::write(&second, data).unwrap();
        assert!(
            !run(&[
                "image",
                "bitwise",
                first.to_str().unwrap(),
                second.to_str().unwrap(),
                "--operation",
                "and",
                "--output",
                output.to_str().unwrap()
            ])
            .status
            .success()
        );
        assert!(!output.exists());
    }
    let oversized = std::fs::File::create(&second).unwrap();
    oversized
        .set_len(cube_core::image::MAX_ENCODED as u64 + 1)
        .unwrap();
    drop(oversized);
    let result = run(&[
        "--json-errors",
        "image",
        "bitwise",
        first.to_str().unwrap(),
        second.to_str().unwrap(),
        "--operation",
        "or",
        "--output",
        output.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(1));
    let error: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(error["error"]["code"], "resource_limit");
    assert!(!output.exists());
    std::fs::write(&second, &original).unwrap();
    for destination in [&first, &second, &output] {
        if destination == &output {
            std::fs::write(destination, b"existing destination").unwrap();
        }
        let before = std::fs::read(destination).unwrap();
        assert!(
            !run(&[
                "image",
                "bitwise",
                first.to_str().unwrap(),
                second.to_str().unwrap(),
                "--operation",
                "xor",
                "--output",
                destination.to_str().unwrap()
            ])
            .status
            .success()
        );
        assert_eq!(std::fs::read(destination).unwrap(), before);
    }
    assert_eq!(std::fs::read(&first).unwrap(), original);
    assert_eq!(std::fs::read(&second).unwrap(), original);
    let result = run(&[
        "--json-errors",
        "image",
        "bitwise",
        first.to_str().unwrap(),
        second.to_str().unwrap(),
        "--operation",
        "not",
        "--output",
        output.to_str().unwrap(),
    ]);
    let error: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(error["error"]["operation"], "image.bitwise");
    assert_eq!(error["error"]["parameter"], "operation");
}
#[test]
fn qr_json_raw_and_exclusive_output_preserve_binary_payloads() {
    let files = Files::new("qr");
    let path = files.path("symbol.png");
    let output = files.path("payload.bin");
    let matrix: Vec<_> = include_str!("../../cube-core/src/image/qr/known-binary.matrix")
        .lines()
        .collect();
    let side = (matrix.len() + 8) * 4;
    let mut samples = vec![255; side * side];
    for (y, row) in matrix.iter().enumerate() {
        for (x, v) in row.bytes().enumerate() {
            for dy in 0..4 {
                for dx in 0..4 {
                    samples[((y + 4) * 4 + dy) * side + (x + 4) * 4 + dx] =
                        if v == b'1' { 0 } else { 255 };
                }
            }
        }
    }
    Image::from_samples(
        side as u32,
        side as u32,
        ColorModel::Gray,
        samples,
        vec![],
        vec![],
    )
    .unwrap()
    .save_png(&path)
    .unwrap();
    let result = run(&["image", "qr", path.to_str().unwrap(), "--json"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["symbols"][0]["payload_hex"], "4375626500ff");
    let result = run(&["image", "qr", path.to_str().unwrap(), "--stdout"]);
    assert!(result.status.success());
    assert_eq!(result.stdout, b"Cube\0\xff");
    let args = [
        "image",
        "qr",
        path.to_str().unwrap(),
        "--output",
        output.to_str().unwrap(),
    ];
    assert!(run(&args).status.success());
    assert!(!run(&args).status.success());
    assert_eq!(std::fs::read(output).unwrap(), b"Cube\0\xff");
    let blank = files.path("blank.png");
    Image::from_bytes(1, 1, ColorModel::Gray, &[255], &[], &[])
        .unwrap()
        .save_png(&blank)
        .unwrap();
    assert!(
        !run(&["image", "qr", blank.to_str().unwrap(), "--stdout"])
            .status
            .success()
    );
}
#[test]
fn raw_geometry_comparison_labels_headers_and_candidates_separately() {
    let files = Files::new("geometry");
    let path = files.path("source.png");
    let image = Image::from_bytes(2, 1, ColorModel::Gray, &[1, 2], &[], &[]).unwrap();
    image.save_png(&path).unwrap();
    let source = std::fs::read(&path).unwrap();
    let width = source.len().to_string();
    let result = run(&[
        "image",
        "raw",
        path.to_str().unwrap(),
        "--width",
        &width,
        "--height",
        "1",
        "--layout",
        "gray",
        "--compare-header",
    ]);
    assert!(result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["declared"]["width"], 2);
    assert_eq!(report["candidate"]["width"], source.len());
    assert_eq!(std::fs::read(path).unwrap(), source);
}
#[test]
fn ordered_pixels_have_exact_hex_raw_and_coordinate_records() {
    let files = Files::new("stream");
    let path = files.path("source.png");
    Image::from_bytes(3, 2, ColorModel::Gray, &[0, 1, 2, 3, 4, 5], &[], &[])
        .unwrap()
        .save_png(&path)
        .unwrap();
    let args = [
        "image",
        "pixels",
        path.to_str().unwrap(),
        "--traversal",
        "spiral_cw",
        "--start-corner",
        "top_right",
    ];
    for (format, expected) in [
        ("--stdout", vec![2, 5, 4, 3, 0, 1]),
        ("--hex", b"020504030001\n".to_vec()),
    ] {
        let mut args = args.to_vec();
        args.push(format);
        let result = run(&args);
        assert!(result.status.success());
        assert_eq!(result.stdout, expected);
    }
    let mut json = args.to_vec();
    json.push("--json");
    let result = run(&json);
    assert!(result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["records"][0]["x"], 2);
    assert_eq!(report["records"][1]["y"], 1);
    let mut bad = args.to_vec();
    let output = files.path("ambiguous.png");
    bad.extend(["--png-output", output.to_str().unwrap()]);
    assert!(!run(&bad).status.success());
    assert!(!output.exists());
}
#[test]
fn lsb_spiral_corner_and_reverse_have_exact_binary_output() {
    let files = Files::new("spiral");
    let path = files.path("source.png");
    Image::from_bytes(3, 2, ColorModel::Gray, &[0, 1, 2, 3, 4, 5], &[], &[])
        .unwrap()
        .save_png(&path)
        .unwrap();
    let args = [
        "image",
        "lsb",
        path.to_str().unwrap(),
        "--bits",
        "7,6,5,4,3,2,1,0",
        "--traversal",
        "spiral_cw",
        "--start-corner",
        "top_right",
        "--stdout",
    ];
    let result = run(&args);
    assert!(result.status.success());
    assert_eq!(result.stdout, [2, 5, 4, 3, 0, 1]);
    let mut args = args.to_vec();
    args.push("--reverse");
    assert_eq!(run(&args).stdout, [1, 0, 3, 4, 5, 2]);
}
#[test]
fn exact_color_exports_have_known_masks_and_preserve_sources() {
    let files = Files::new("colors");
    let path = files.path("source.png");
    let output = files.path("mask.png");
    let image = Image::from_bytes(
        2,
        1,
        ColorModel::Rgba,
        &[1, 2, 3, 0, 1, 2, 3, 255],
        &[],
        &[],
    )
    .unwrap();
    image.save_png(&path).unwrap();
    let original = std::fs::read(&path).unwrap();
    let args = [
        "image",
        "colors",
        path.to_str().unwrap(),
        "--colors",
        "01020300",
        "--output",
        output.to_str().unwrap(),
    ];
    let result = run(&args);
    assert!(result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["selected_pixels"], 1);
    assert_eq!(Image::open(&output).unwrap().samples(), [255, 0]);
    assert!(!run(&args).status.success());
    assert_eq!(std::fs::read(path).unwrap(), original);
}
#[test]
fn raw_layout_preview_export_and_collision_preserve_input() {
    let files = Files::new("raw");
    let path = files.path("pixels.bin");
    let destination = files.path("raw.png");
    let bytes = [3, 2, 1, 99, 6, 5, 4, 88];
    std::fs::write(&path, bytes).unwrap();
    let args = [
        "image",
        "raw",
        path.to_str().unwrap(),
        "--width",
        "1",
        "--height",
        "2",
        "--layout",
        "bgr",
        "--row-stride",
        "4",
    ];
    let result = run(&args);
    assert!(result.status.success());
    let info: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(info["row_padding"], 1);
    let mut args = args.to_vec();
    args.extend(["--output", destination.to_str().unwrap()]);
    assert!(run(&args).status.success());
    assert!(!run(&args).status.success());
    assert_eq!(
        Image::open(&destination).unwrap().samples(),
        [1, 2, 3, 4, 5, 6]
    );
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}
#[test]
fn bmp_inspection_reports_decoded_provenance_and_binary_pixels() {
    let files = Files::new("bmp");
    let path = files.path("original.bmp");
    let mut bytes = vec![0; 62];
    bytes[..2].copy_from_slice(b"BM");
    for (offset, value) in [(2, 62u32), (10, 54), (14, 40), (18, 2), (22, 1), (34, 8)] {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    bytes[26] = 1;
    bytes[28] = 24;
    bytes[54..60].copy_from_slice(&[3, 2, 1, 6, 5, 4]);
    std::fs::write(&path, &bytes).unwrap();
    let input = path.to_str().unwrap();
    let result = run(&["image", "inspect", input, "--json"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let info: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(info["format"], "bmp");
    assert_eq!(info["provenance"]["source_samples_preserved"], false);
    let result = run(&["image", "pixels", input, "--stdout"]);
    assert!(result.status.success());
    assert_eq!(result.stdout, [1, 2, 3, 4, 5, 6]);
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}
#[test]
fn inspect_chunks_and_conversion_use_core_samples_and_safe_destinations() {
    let files = Files::new("foundation");
    let path = files.path("input.png");
    let output = files.path("output.png");
    let source = Image::from_samples(
        2,
        1,
        ColorModel::Indexed,
        vec![0, 1],
        vec![[1, 2, 3], [4, 5, 6]],
        vec![0],
    )
    .unwrap();
    source.save_png(&path).unwrap();
    let input = path.to_str().unwrap();
    for args in [
        vec!["image", "inspect", input, "--json"],
        vec!["image", "inspect", input, "--json", "--exif"],
        vec!["image", "chunks", input, "--json"],
    ] {
        let result = run(&args);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let _: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    }
    let result = run(&[
        "image",
        "convert",
        input,
        "--expand-palette",
        "--output",
        output.to_str().unwrap(),
    ]);
    assert!(result.status.success());
    assert_eq!(
        Image::open(&output).unwrap().samples(),
        [1, 2, 3, 0, 4, 5, 6, 255]
    );
    let result = run(&["image", "convert", input, "--output", input]);
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(Image::open(&path).unwrap().samples(), [0, 1]);
}

#[test]
fn pixel_channel_histogram_and_comparison_commands_preserve_exact_data() {
    let files = Files::new("analysis");
    let first = files.path("first.png");
    let second = files.path("second.png");
    let mask = files.path("mask.png");
    let bytes = files.path("pixels.bin");
    let source = Image::from_samples(
        2,
        1,
        ColorModel::Rgba,
        vec![1, 2, 3, 0, 4, 5, 6, 255],
        vec![],
        vec![],
    )
    .unwrap();
    source.save_png(&first).unwrap();
    Image::from_samples(
        2,
        1,
        ColorModel::Rgba,
        vec![1, 2, 3, 255, 4, 50, 6, 255],
        vec![],
        vec![],
    )
    .unwrap()
    .save_png(&second)
    .unwrap();
    let a = first.to_str().unwrap();
    let b = second.to_str().unwrap();
    for args in [
        vec!["image", "pixel", a, "--x", "1", "--y", "0", "--json"],
        vec![
            "image",
            "pixels",
            a,
            "--region",
            "1,0,1,1",
            "--channels",
            "blue,red",
            "--json",
        ],
        vec!["image", "histogram", a, "--json"],
        vec![
            "image",
            "compare",
            a,
            b,
            "--json",
            "--mask-output",
            mask.to_str().unwrap(),
        ],
    ] {
        let output = run(&args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let _: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    }
    assert_eq!(Image::open(&mask).unwrap().samples(), [255, 255]);
    let output = run(&["image", "channel", a, "--channel", "alpha", "--stdout"]);
    assert!(output.status.success());
    assert_eq!(output.stdout, [0, 255]);
    let output = run(&[
        "image",
        "pixels",
        a,
        "--channels",
        "blue,red",
        "--output",
        bytes.to_str().unwrap(),
    ]);
    assert!(output.status.success());
    assert_eq!(std::fs::read(bytes).unwrap(), [3, 1, 6, 4]);
}

#[test]
fn transforms_write_exact_new_pngs_and_reject_overwrites() {
    let files = Files::new("transforms");
    let input = files.path("input.png");
    let image = Image::from_samples(
        2,
        1,
        ColorModel::Rgba,
        vec![255, 0, 0, 7, 0, 0, 255, 8],
        vec![],
        vec![],
    )
    .unwrap();
    image.save_png(&input).unwrap();
    for (name, flags, expected) in [
        ("rotate", vec!["--degrees", "90"], image.rotate(90).unwrap()),
        (
            "mirror",
            vec!["--axis", "horizontal"],
            image.mirror("horizontal").unwrap(),
        ),
        (
            "crop",
            vec!["--region", "1,0,1,1"],
            image
                .crop(cube_core::image::Region {
                    x: 1,
                    y: 0,
                    width: 1,
                    height: 1,
                })
                .unwrap(),
        ),
        ("invert", vec![], image.invert(None).unwrap()),
        (
            "reorder",
            vec!["--channels", "blue,green,red,alpha"],
            image
                .reorder(&[
                    cube_core::image::Channel::Blue,
                    cube_core::image::Channel::Green,
                    cube_core::image::Channel::Red,
                    cube_core::image::Channel::Alpha,
                ])
                .unwrap(),
        ),
        ("grayscale", vec![], image.grayscale().unwrap()),
    ] {
        let output = files.path(&format!("{name}.png"));
        let mut args = vec!["image", name, input.to_str().unwrap()];
        args.extend(flags);
        args.extend(["--output", output.to_str().unwrap()]);
        let result = run(&args);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let actual = Image::open(&output).unwrap();
        assert_eq!(actual.samples(), expected.samples());
        assert_eq!(
            (actual.width(), actual.height()),
            (expected.width(), expected.height())
        );
        assert_eq!(run(&args).status.code(), Some(1));
    }
    assert_eq!(
        run(&[
            "image",
            "rotate",
            input.to_str().unwrap(),
            "--degrees",
            "45",
            "--output",
            input.to_str().unwrap()
        ])
        .status
        .code(),
        Some(1)
    );
    assert_eq!(Image::open(&input).unwrap().samples(), image.samples());
}

#[test]
fn bit_extraction_preserves_raw_stdout_and_explicit_representation() {
    let files = Files::new("bits");
    let input = files.path("input.png");
    let packed = files.path("packed.bin");
    let display = files.path("plane.png");
    let source = Image::from_samples(
        4,
        2,
        ColorModel::Gray,
        vec![1, 0, 1, 0, 0, 1, 1, 0],
        vec![],
        vec![],
    )
    .unwrap();
    source.save_png(&input).unwrap();
    let input = input.to_str().unwrap();
    let result = run(&["image", "lsb", input, "--stdout"]);
    assert!(result.status.success());
    assert_eq!(result.stdout, [0xa6]);
    let result = run(&[
        "image",
        "lsb",
        input,
        "--traversal",
        "column",
        "--reverse",
        "--bit-order",
        "lsb",
        "--stdout",
    ]);
    assert!(result.status.success());
    assert_eq!(result.stdout, [0x9c]);
    let result = run(&[
        "image",
        "lsb",
        input,
        "--region",
        "0,0,1,1",
        "--incomplete",
        "pad",
        "--output",
        packed.to_str().unwrap(),
    ]);
    assert!(result.status.success());
    let metadata: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(metadata["valid_bits"], 1);
    assert_eq!(std::fs::read(&packed).unwrap(), [128]);
    assert_eq!(
        run(&["image", "lsb", input, "--output", packed.to_str().unwrap()])
            .status
            .code(),
        Some(1)
    );
    assert_eq!(
        run(&["image", "lsb", input, "--bits", "0,0", "--stdout"])
            .status
            .code(),
        Some(1)
    );
    assert_eq!(
        run(&["image", "lsb", input, "--region", "0,0,1,1", "--stdout"])
            .status
            .code(),
        Some(1)
    );
    let result = run(&[
        "image",
        "bit-plane",
        input,
        "--channel",
        "gray",
        "--bit",
        "0",
        "--stdout",
    ]);
    assert_eq!(result.stdout, source.samples());
    let result = run(&[
        "image",
        "bit-plane",
        input,
        "--channel",
        "gray",
        "--bit",
        "0",
        "--packed",
        "--stdout",
    ]);
    assert_eq!(result.stdout, [0xa6]);
    let result = run(&[
        "image",
        "bit-plane",
        input,
        "--channel",
        "gray",
        "--bit",
        "0",
        "--png-output",
        display.to_str().unwrap(),
    ]);
    assert!(result.status.success());
    assert_eq!(
        Image::open(&display).unwrap().samples(),
        [255, 0, 255, 0, 0, 255, 255, 0]
    );
    let result = run(&[
        "image",
        "bit-plane",
        input,
        "--channel",
        "gray",
        "--bit",
        "0",
        "--json",
    ]);
    assert!(result.status.success());
    let data: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(data["packed_hex"], "a6");
    assert_eq!(
        Image::open(std::path::Path::new(input)).unwrap().samples(),
        source.samples()
    );
}

#[test]
fn csv_exchanges_exact_samples_with_exclusive_files_and_stdin() {
    use std::{io::Write, process::Stdio};
    let files = Files::new("csv");
    let source = files.path("source.png");
    let csv = files.path("pixels.csv");
    let restored = files.path("restored.png");
    let image = Image::from_bytes(
        2,
        1,
        ColorModel::Rgba,
        &[1, 2, 3, 0, 4, 5, 6, 255],
        &[],
        &[],
    )
    .unwrap();
    image.save_png(&source).unwrap();
    let original = std::fs::read(&source).unwrap();
    let output = run(&[
        "image",
        "csv",
        "export",
        source.to_str().unwrap(),
        "--stdout",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, image.to_csv().unwrap().as_bytes());
    let args = [
        "image",
        "csv",
        "export",
        source.to_str().unwrap(),
        "--output",
        csv.to_str().unwrap(),
    ];
    assert!(run(&args).status.success());
    assert!(!run(&args).status.success());
    assert_eq!(std::fs::read(&csv).unwrap(), output.stdout);
    let args = [
        "image",
        "csv",
        "import",
        "--input",
        csv.to_str().unwrap(),
        "--output",
        restored.to_str().unwrap(),
        "--json",
    ];
    let imported = run(&args);
    assert!(
        imported.status.success(),
        "{}",
        String::from_utf8_lossy(&imported.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&imported.stdout).unwrap()["width"],
        2
    );
    assert_eq!(Image::open(&restored).unwrap().samples(), image.samples());
    assert!(!run(&args).status.success());
    let piped = files.path("piped.png");
    let mut child = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "image",
            "csv",
            "import",
            "--output",
            piped.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&output.stdout)
        .unwrap();
    assert!(child.wait_with_output().unwrap().status.success());
    assert_eq!(Image::open(&piped).unwrap().samples(), image.samples());
    std::fs::write(&csv, [255]).unwrap();
    let invalid = files.path("invalid.png");
    assert!(
        !run(&[
            "image",
            "csv",
            "import",
            "--input",
            csv.to_str().unwrap(),
            "--output",
            invalid.to_str().unwrap()
        ])
        .status
        .success()
    );
    assert!(!invalid.exists());
    assert!(
        !run(&[
            "image",
            "csv",
            "import",
            "--text",
            "invalid",
            "--output",
            invalid.to_str().unwrap()
        ])
        .status
        .success()
    );
    assert!(
        !run(&["image", "csv", "export", source.to_str().unwrap()])
            .status
            .success()
    );
    assert_eq!(std::fs::read(&source).unwrap(), original);
}
