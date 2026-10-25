#[path = "../../cube-core/tests/common/video.rs"]
mod fixture;
#[path = "../../cube-core/tests/common/mov.rs"]
mod mov;
#[path = "../../cube-core/tests/common/pcm_mov.rs"]
mod pcm_mov;
use std::process::Command;
#[test]
fn video_inspection_json_matches_core_and_preserves_source() {
    for (kind, input) in [("mp4", fixture::mp4(1, false)), ("webm", fixture::webm())] {
        let path = std::env::temp_dir().join(format!("cube-video-{}.{kind}", std::process::id()));
        std::fs::write(&path, &input).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(["video", "inspect", path.to_str().unwrap(), "--json"])
            .output()
            .unwrap();
        assert!(out.status.success(), "{:?}", out.stderr);
        let actual: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        let expected = serde_json::to_value(
            cube_core::media::video::inspect(&mut std::io::Cursor::new(&input)).unwrap(),
        )
        .unwrap();
        assert_eq!(actual, expected);
        assert_eq!(std::fs::read(&path).unwrap(), input);
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn selected_frame_pngs_preserve_source_and_existing_outputs_with_batch_cleanup() {
    let directory = std::env::temp_dir().join(format!("cube-mov-frames-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let source = directory.join("clip.mov");
    let input = mov::movie(&[mov::jpeg(0), mov::jpeg(1)], false, false);
    std::fs::write(&source, &input).unwrap();
    let run = |indexes: &str| {
        Command::new(env!("CARGO_BIN_EXE_cube"))
            .args([
                "video",
                "frames",
                source.to_str().unwrap(),
                "--track-id",
                "1",
                "--indexes",
                indexes,
                "--output-dir",
                directory.to_str().unwrap(),
            ])
            .output()
            .unwrap()
    };
    let second = directory.join("track-1-frame-1.png");
    std::fs::write(&second, b"existing").unwrap();
    let result = run("0,1");
    assert!(!result.status.success());
    assert!(!directory.join("track-1-frame-0.png").exists());
    assert_eq!(std::fs::read(&second).unwrap(), b"existing");
    std::fs::remove_file(&second).unwrap();
    let result = run("1,0");
    assert!(result.status.success(), "{:?}", result.stderr);
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["frames"][0]["provenance"]["frame_index"], 1);
    let image = cube_core::image::Image::open(&second).unwrap();
    assert_eq!(
        image.samples(),
        cube_core::image::Image::decode(&mov::jpeg(1))
            .unwrap()
            .samples()
    );
    assert!(!run("0").status.success());
    assert_eq!(std::fs::read(&source).unwrap(), input);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn frame_pair_cli_reports_exact_existing_comparison_and_source_provenance() {
    let input = mov::movie(&[mov::jpeg(0), mov::jpeg(1), mov::jpeg(0)], true, true);
    let path = std::env::temp_dir().join(format!("cube-video-pairs-{}.mov", std::process::id()));
    std::fs::write(&path, &input).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cube"))
        .args([
            "video",
            "compare",
            path.to_str().unwrap(),
            "--track-id",
            "1",
            "--pairs",
            "0:1,0:2",
            "--max-results",
            "1",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let actual: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let expected = cube_core::media::video::comparison::compare(
        &mut std::io::Cursor::new(&input),
        1,
        &[(0, 1), (0, 2)],
        None,
        1,
    )
    .unwrap();
    assert_eq!(actual, serde_json::to_value(expected).unwrap());
    assert_eq!(std::fs::read(&path).unwrap(), input);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn pcm_track_cli_preserves_source_and_requires_explicit_nonoverwriting_output() {
    let directory = std::env::temp_dir().join(format!("cube-mov-audio-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let source = directory.join("sound.mov");
    let target = directory.join("track.wav");
    let raw = vec![0, 255, 128, 64, 1, 254];
    let input = pcm_mov::movie(std::slice::from_ref(&raw), 2, 8000, true);
    std::fs::write(&source, &input).unwrap();
    let run = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_cube"))
            .args(
                [
                    &[
                        "video",
                        "audio",
                        source.to_str().unwrap(),
                        "--track-id",
                        "1",
                    ][..],
                    extra,
                ]
                .concat(),
            )
            .output()
            .unwrap()
    };
    let result = run(&["--stdout"]);
    assert!(result.status.success(), "{:?}", result.stderr);
    let expected =
        cube_core::media::video::audio::extract(&mut std::io::Cursor::new(&input), 1).unwrap();
    assert_eq!(result.stdout, expected.data);
    let result = run(&["--output", target.to_str().unwrap()]);
    assert!(result.status.success(), "{:?}", result.stderr);
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["provenance"]["pcm_bytes"], 6);
    assert_eq!(std::fs::read(&target).unwrap(), expected.data);
    assert!(
        !run(&["--output", target.to_str().unwrap()])
            .status
            .success()
    );
    assert!(
        !run(&["--output", source.to_str().unwrap()])
            .status
            .success()
    );
    assert!(!run(&[]).status.success());
    assert!(
        !run(&["--stdout", "--output", target.to_str().unwrap()])
            .status
            .success()
    );
    assert_eq!(std::fs::read(&source).unwrap(), input);
    std::fs::remove_dir_all(directory).unwrap();
}
