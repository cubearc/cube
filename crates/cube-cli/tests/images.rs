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
