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
