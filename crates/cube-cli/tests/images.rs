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
fn qr_json_raw_and_exclusive_output_preserve_binary_payloads() {
    let files = Files::new("qr");
    let path = files.path("symbol.png");
    let output = files.path("payload.bin");
    let matrix: Vec<_> = include_str!("../../../tests/fixtures/qr-binary.matrix")
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
