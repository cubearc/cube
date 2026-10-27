//! Explicit result export only. The ordinary CLI output is never parsed or rerun.
use crate::{Cli, Command, Decode, imagecmd::ImageCommand, textcmd::TextCommand};
use cube_core::investigation::session::typed_capture::TypedCapture;
use serde_json::Value;
use std::{error::Error, fs::OpenOptions, io::Write, path::PathBuf, time::SystemTime};

pub(crate) struct Target {
    path: PathBuf,
    id: String,
}
pub(crate) struct Pending {
    path: PathBuf,
    document: String,
}

impl Target {
    pub(crate) fn check_image(
        &self,
        image: &cube_core::image::Image,
    ) -> Result<(), Box<dyn Error>> {
        // Bound before Base64/JSON allocations. The wire limit also includes
        // palette arrays, named-port schemas and provenance.
        let info = image.info();
        let bytes = image.samples().len() + info.palette.len() * 3 + info.transparency.len();
        if bytes > cube_core::investigation::session::typed_capture::MAX_BYTES / 2 {
            return Err("Typed capture supports at most 64 KiB of logical image data; run without --lab-capture for larger images".into());
        }
        Ok(())
    }
    pub(crate) fn requested(cli: &Cli) -> Result<Option<Self>, Box<dyn Error>> {
        let Some(request) = &cli.lab_capture else {
            return Ok(None);
        };
        if !matches!(
            &cli.command,
            Command::Text {
                command: TextCommand::Reverse(_)
            } | Command::Decode {
                command: Decode::Hex(_)
            } | Command::Image {
                command: ImageCommand::Invert { .. } | ImageCommand::Histogram { .. }
            }
        ) {
            return Err("Typed capture supports text reverse, decode hex, image invert and image histogram; run other commands without --lab-capture".into());
        }
        let id = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_nanos()
        );
        let path = match request {
            Some(path) => path.clone(),
            None => {
                let directory = std::env::var_os("CUBE_PLAY_CAPTURE_DIR")
                    .ok_or("Use --lab-capture=PATH outside the configured Play Runner")?;
                PathBuf::from(directory).join(format!("{id}.json"))
            }
        };
        // create_new at publication is authoritative; this early check avoids
        // executing an operation when a destination is already occupied.
        if std::fs::symlink_metadata(&path).is_ok() {
            return Err("Capture destination already exists; choose a new path".into());
        }
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(std::path::Path::new("."));
        if !parent.is_dir() {
            return Err("Capture parent directory does not exist".into());
        }
        Ok(Some(Self { path, id }))
    }
    pub(crate) fn prepare(
        &self,
        operation: &str,
        inputs: &Value,
        parameters: &Value,
        outputs: &Value,
    ) -> Result<Pending, Box<dyn Error>> {
        let document = TypedCapture::from_result(operation, &self.id, inputs, parameters, outputs)?
            .to_json()?;
        Ok(Pending {
            path: self.path.clone(),
            document,
        })
    }
}
impl Pending {
    pub(crate) fn publish(self) -> Result<(), Box<dyn Error>> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(self.path)?;
        // Incomplete writes are invalid captures. Never delete or overwrite a
        // pathname that another local process might have replaced.
        file.write_all(self.document.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }
}
