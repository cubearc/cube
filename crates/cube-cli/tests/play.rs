use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn cube(root: &Path) -> Command {
    let mut command = Command::new(
        root.join("installed")
            .join(format!("cube{}", std::env::consts::EXE_SUFFIX)),
    );
    command
        .current_dir(root.join("investigation 空 space"))
        .env_remove("CUBE_PLAY_LAUNCHER")
        .env_remove("CUBE_PLAY_FIXTURE_MODE")
        .env("CUBE_PLAY_RECORD", root.join("record.txt"));
    command
}

#[test]
fn delegates_only_to_the_trusted_launcher_and_preserves_the_contract() {
    let root =
        Fixture(std::env::temp_dir().join(format!("cube-play-contract-{}", std::process::id())));
    fs::create_dir_all(root.0.join("installed")).unwrap();
    fs::create_dir_all(root.0.join("investigation 空 space")).unwrap();
    let binary = format!("cube{}", std::env::consts::EXE_SUFFIX);
    let launcher = root
        .0
        .join("installed")
        .join(format!("cube-play{}", std::env::consts::EXE_SUFFIX));
    fs::copy(
        env!("CARGO_BIN_EXE_cube"),
        root.0.join("installed").join(binary),
    )
    .unwrap();
    let source = root.0.join("fixture.rs");
    fs::write(&source, r#"
        use std::{env, fs, io::Write};
        fn main() {
            let args:Vec<_>=env::args_os().skip(1).collect();
            if args.first().is_some_and(|v|v=="--launcher-protocol") {
                match env::var("CUBE_PLAY_FIXTURE_MODE").as_deref() {
                    Ok("incompatible")=>println!("cube-play-launcher 99"),
                    Ok("timeout")=>std::thread::sleep(std::time::Duration::from_secs(20)),
                    _=>println!("cube-play-launcher 1")
                }
                return;
            }
            let mut file=fs::File::create(env::var_os("CUBE_PLAY_RECORD").unwrap()).unwrap();
            for value in std::iter::once(env::current_dir().unwrap().into_os_string()).chain(args) {
                file.write_all(value.to_str().unwrap().as_bytes()).unwrap();file.write_all(&[0]).unwrap();
            }
            if env::var("CUBE_PLAY_FIXTURE_MODE").as_deref()==Ok("wait") {std::thread::sleep(std::time::Duration::from_secs(20));}
            if env::var("CUBE_PLAY_FIXTURE_MODE").as_deref()==Ok("failure") {std::process::exit(7);}
        }
    "#).unwrap();
    assert!(
        Command::new("rustc")
            .args(["--edition=2024", "--crate-name", "play_contract_fixture"])
            .arg(&source)
            .arg("-o")
            .arg(&launcher)
            .status()
            .unwrap()
            .success()
    );
    // A workspace executable never overrides the installed sibling.
    fs::write(
        root.0
            .join("investigation 空 space")
            .join(format!("cube-play{}", std::env::consts::EXE_SUFFIX)),
        "untrusted workspace file",
    )
    .unwrap();
    let help = cube(&root.0).args(["play", "--help"]).output().unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("current directory"));
    for path in [
        None,
        Some("sample 空 file.mp4"),
        Some("rabbit.cube"),
        Some("--literal.txt"),
    ] {
        let mut command = cube(&root.0);
        command.args(["play", "--no-browser", "--offline"]);
        if let Some(path) = path {
            command.arg("--").arg(path);
        }
        assert!(command.status().unwrap().success());
        let record = fs::read_to_string(root.0.join("record.txt")).unwrap();
        let values: Vec<_> = record.split('\0').filter(|part| !part.is_empty()).collect();
        assert_eq!(
            Path::new(values[0]).canonicalize().unwrap(),
            root.0
                .join("investigation 空 space")
                .canonicalize()
                .unwrap()
        );
        assert_eq!(
            &values[1..5],
            &["--protocol", "1", "--no-browser", "--offline"]
        );
        if let Some(path) = path {
            assert_eq!(&values[5..], &["--", path]);
        } else {
            assert_eq!(values.len(), 5);
        }
    }
    let output = cube(&root.0)
        .args(["--json-errors", "play", "--stop"])
        .env("CUBE_PLAY_FIXTURE_MODE", "failure")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(7));
    let record = fs::read_to_string(root.0.join("record.txt")).unwrap();
    assert!(record.contains("--json-errors\0--stop\0"));
    for mode in ["incompatible", "timeout"] {
        let output = cube(&root.0)
            .args(["--json-errors", "play"])
            .env("CUBE_PLAY_FIXTURE_MODE", mode)
            .output()
            .unwrap();
        assert!(!output.status.success());
        let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(error["schema_version"], 1);
        assert!(
            error["error"]["message"]
                .as_str()
                .unwrap()
                .contains("launcher")
        );
    }
    let relative = cube(&root.0)
        .args(["--json-errors", "play"])
        .env("CUBE_PLAY_LAUNCHER", "./cube-play")
        .output()
        .unwrap();
    assert!(!relative.status.success());
    let error: serde_json::Value = serde_json::from_slice(&relative.stderr).unwrap();
    assert_eq!(error["error"]["code"], "invalid_parameter");
    let explicit = cube(&root.0)
        .args(["play"])
        .env("CUBE_PLAY_LAUNCHER", &launcher)
        .output()
        .unwrap();
    assert!(
        explicit.status.success(),
        "explicit launcher failed ({}): {}",
        explicit.status,
        String::from_utf8_lossy(&explicit.stderr)
    );
    #[cfg(unix)]
    {
        use std::{
            os::unix::process::ExitStatusExt,
            time::{Duration, Instant},
        };
        fs::remove_file(root.0.join("record.txt")).unwrap();
        let mut child = cube(&root.0)
            .arg("play")
            .env("CUBE_PLAY_FIXTURE_MODE", "wait")
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !root.0.join("record.txt").exists() {
            assert!(
                Instant::now() < deadline,
                "delegation did not reach its launcher"
            );
            assert!(child.try_wait().unwrap().is_none());
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(
            Command::new("/bin/kill")
                .args(["-TERM", &child.id().to_string()])
                .status()
                .unwrap()
                .success()
        );
        assert_eq!(
            child.wait().unwrap().signal(),
            Some(15),
            "Unix delegation must retain process identity and signals"
        );
    }
    fs::remove_file(&launcher).unwrap();
    let missing = cube(&root.0)
        .args(["--json-errors", "play"])
        .output()
        .unwrap();
    assert!(!missing.status.success());
    let error: serde_json::Value = serde_json::from_slice(&missing.stderr).unwrap();
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("install")
    );
}
