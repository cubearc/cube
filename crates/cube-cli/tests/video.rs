#[path = "../../cube-core/tests/common/video.rs"]
mod fixture;
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
