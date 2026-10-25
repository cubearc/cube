//! Reserve an entire explicit image export before writing; roll back only our files.
use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
    path::{Component, Path},
};

pub(super) fn write_outputs(directory: &Path, outputs: &[(String, Vec<u8>)]) -> io::Result<()> {
    write_with(directory, outputs, |file, bytes| {
        file.write_all(bytes)?;
        file.flush()
    })
}
fn write_with(
    directory: &Path,
    outputs: &[(String, Vec<u8>)],
    mut write: impl FnMut(&mut File, &[u8]) -> io::Result<()>,
) -> io::Result<()> {
    let mut files = Vec::new();
    let result = (|| {
        for (name, _) in outputs {
            let mut components = Path::new(name).components();
            if !matches!(components.next(), Some(Component::Normal(_)))
                || components.next().is_some()
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "export name must be a single filename",
                ));
            }
            let path = directory.join(name);
            let file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)?;
            files.push((path, file));
        }
        for ((_, file), (_, bytes)) in files.iter_mut().zip(outputs) {
            write(file, bytes)?;
        }
        Ok(())
    })();
    let paths: Vec<_> = files.iter().map(|(path, _)| path.clone()).collect();
    drop(files);
    if result.is_err() {
        for path in paths {
            let _ = std::fs::remove_file(path);
        }
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn collisions_and_injected_failures_remove_only_our_outputs() {
        let dir = std::env::temp_dir().join(format!("cube-image-exports-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let outputs = vec![("a.png".into(), vec![1]), ("b.png".into(), vec![2])];
        std::fs::write(dir.join("b.png"), b"keep").unwrap();
        assert!(write_outputs(&dir, &outputs).is_err());
        assert!(!dir.join("a.png").exists());
        assert_eq!(std::fs::read(dir.join("b.png")).unwrap(), b"keep");
        std::fs::remove_file(dir.join("b.png")).unwrap();
        let mut calls = 0;
        assert!(
            write_with(&dir, &outputs, |file, bytes| {
                calls += 1;
                if calls == 2 {
                    return Err(io::Error::other("injected"));
                }
                file.write_all(bytes)
            })
            .is_err()
        );
        assert!(!dir.join("a.png").exists() && !dir.join("b.png").exists());
        assert!(write_outputs(&dir, &[("../escape".into(), vec![])]).is_err());
        write_outputs(&dir, &outputs).unwrap();
        assert_eq!(std::fs::read(dir.join("a.png")).unwrap(), [1]);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
