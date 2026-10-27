use cube_core::{
    image::{ColorModel, Image},
    investigation::session::typed_capture::{MAX_BYTES, TypedCapture},
};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};
struct Files(PathBuf);
impl Files {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("cube-capture-{}-{name}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
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
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(args)
        .env_remove("CUBE_PLAY_CAPTURE_DIR")
        .output()
        .unwrap()
}
fn capture(args: &[&str], path: &Path) -> (Output, TypedCapture) {
    let flag = format!("--lab-capture={}", path.display());
    let mut arguments = vec![flag.as_str()];
    arguments.extend_from_slice(args);
    let result = run(&arguments);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let document = std::fs::read_to_string(path).unwrap();
    assert!(document.len() <= MAX_BYTES);
    (result, TypedCapture::from_json(&document).unwrap())
}
#[test]
fn real_cli_captures_text_and_arbitrary_bytes_without_changing_normal_output() {
    let files = Files::new("values");
    let args = ["text", "reverse", "--text", "Aé🧊"];
    let plain = run(&args);
    let (result, record) = capture(&args, &files.path("text.json"));
    assert_eq!(result.stdout, plain.stdout);
    assert_eq!(result.stderr, plain.stderr);
    assert_eq!(record.outputs["output"], "🧊éA");
    assert_eq!(record.operation, "text.reverse");
    assert_eq!(record.inputs[0].semantic_type, "text");
    assert_eq!(record.inputs[0].identity.size, 9);
    assert!(!record.replay_gaps.is_empty());
    let args = ["decode", "hex", "--text", "00ff80", "--output", "raw"];
    let plain = run(&args);
    let (result, record) = capture(&args, &files.path("bytes.json"));
    assert_eq!(result.stdout, plain.stdout);
    assert_eq!(result.stdout, [0, 255, 128]);
    assert_eq!(record.outputs["output"]["base64"], "AP+A");
    assert_eq!(record.ports[0].semantic_type, "bytes");
    assert_eq!(std::fs::read_dir(&files.0).unwrap().count(), 2);
}
#[test]
fn real_cli_captures_image_and_report_using_exact_effective_parameters() {
    let files = Files::new("images");
    let source = files.path("original.png");
    let output = files.path("inverted.png");
    Image::from_bytes(2, 1, ColorModel::Rgb, &[10, 20, 30, 10, 20, 30], &[], &[])
        .unwrap()
        .save_png(&source)
        .unwrap();
    let original = std::fs::read(&source).unwrap();
    let (_, record) = capture(
        &[
            "image",
            "invert",
            source.to_str().unwrap(),
            "--channels",
            "red",
            "--output",
            output.to_str().unwrap(),
        ],
        &files.path("image.json"),
    );
    assert_eq!(record.parameters["channels"], serde_json::json!(["red"]));
    assert_eq!(record.parameters["expand_palette"], false);
    let bytes = cube_core::encoding::base64_decode(
        record.outputs["output"]["samples"]["base64"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(bytes, [245, 20, 30, 245, 20, 30]);
    assert_eq!(Image::open(&output).unwrap().samples(), bytes);
    let args = [
        "image",
        "histogram",
        source.to_str().unwrap(),
        "--region",
        "1,0,1,1",
        "--channels",
        "red",
        "--max-colors",
        "2",
        "--json",
    ];
    let plain = run(&args);
    let (result, record) = capture(&args, &files.path("histogram.json"));
    assert_eq!(result.stdout, plain.stdout);
    assert_eq!(result.stderr, plain.stderr);
    assert_eq!(record.parameters["region"], serde_json::json!([1, 0, 1, 1]));
    assert_eq!(record.parameters["max_colors"], 2);
    assert_eq!(record.outputs["output"]["pixels"], 1);
    assert_eq!(
        record.outputs["output"]["colors"][0]["values"],
        serde_json::json!([10])
    );
    assert_eq!(std::fs::read(&source).unwrap(), original);
}
#[test]
fn captures_are_explicit_bounded_non_overwriting_and_fail_without_a_valid_result() {
    let files = Files::new("failures");
    let destination = files.path("record.json");
    let flag = format!("--lab-capture={}", destination.display());
    let refused = run(&[
        &flag,
        "cipher",
        "solitaire",
        "--parameters",
        "must-not-be-read.json",
        "--text",
        "HELLO",
    ]);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("Typed capture supports"));
    assert!(!destination.exists());
    assert!(
        !run(&[&flag, "decode", "hex", "--text", "invalid"])
            .status
            .success()
    );
    assert!(!destination.exists());
    let oversized = files.path("oversized.txt");
    std::fs::write(&oversized, "A".repeat(MAX_BYTES)).unwrap();
    assert!(
        !run(&[
            &flag,
            "text",
            "reverse",
            "--file",
            oversized.to_str().unwrap()
        ])
        .status
        .success()
    );
    assert!(!destination.exists());
    let missing = format!(
        "--lab-capture={}",
        files.path("missing/file.json").display()
    );
    assert!(
        !run(&[&missing, "text", "reverse", "--text", "ABC"])
            .status
            .success()
    );
    assert!(
        !run(&["--lab-capture", "text", "reverse", "--text", "ABC"])
            .status
            .success()
    );
    std::fs::write(&destination, b"preserve").unwrap();
    assert!(
        !run(&[&flag, "text", "reverse", "--text", "ABC"])
            .status
            .success()
    );
    assert_eq!(std::fs::read(&destination).unwrap(), b"preserve");
    let staged = files.path("staged");
    std::fs::create_dir(&staged).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args(["--lab-capture", "text", "reverse", "--text", "ABC"])
        .env("CUBE_PLAY_CAPTURE_DIR", &staged)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let path = std::fs::read_dir(&staged)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    TypedCapture::from_json(&std::fs::read_to_string(path).unwrap()).unwrap();
    let image = files.path("source.png");
    Image::from_bytes(1, 1, ColorModel::Gray, &[1], &[], &[])
        .unwrap()
        .save_png(&image)
        .unwrap();
    let failed = files.path("output-failed.json");
    let flag = format!("--lab-capture={}", failed.display());
    let result = run(&[
        &flag,
        "image",
        "invert",
        image.to_str().unwrap(),
        "--output",
        destination.to_str().unwrap(),
    ]);
    assert!(!result.status.success());
    assert!(!failed.exists());
    assert_eq!(std::fs::read(&destination).unwrap(), b"preserve");
}
#[cfg(unix)]
#[test]
fn capture_never_follows_an_existing_destination_symlink() {
    let files = Files::new("symlink");
    let target = files.path("target");
    let link = files.path("link");
    std::fs::write(&target, b"private").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let flag = format!("--lab-capture={}", link.display());
    assert!(
        !run(&[&flag, "text", "reverse", "--text", "ABC"])
            .status
            .success()
    );
    assert_eq!(std::fs::read(target).unwrap(), b"private");
}

#[test]
fn logical_image_capture_limit_is_checked_before_wire_allocation() {
    let files = Files::new("image-limits");
    let exact = files.path("exact.png");
    Image::from_bytes(256, 256, ColorModel::Gray, &vec![1; 65536], &[], &[])
        .unwrap()
        .save_png(&exact)
        .unwrap();
    capture(
        &[
            "image",
            "invert",
            exact.to_str().unwrap(),
            "--output",
            files.path("exact-out.png").to_str().unwrap(),
        ],
        &files.path("exact.json"),
    );
    let over = files.path("over.png");
    Image::from_bytes(255, 257, ColorModel::Gray, &vec![1; 65535], &[], &[0, 1])
        .unwrap()
        .save_png(&over)
        .unwrap();
    let flag = format!("--lab-capture={}", files.path("over.json").display());
    let output = files.path("over-out.png");
    let result = run(&[
        &flag,
        "image",
        "invert",
        over.to_str().unwrap(),
        "--output",
        output.to_str().unwrap(),
    ]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("64 KiB"));
    assert!(!output.exists());
    assert!(!files.path("over.json").exists());
}
