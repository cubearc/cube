//! Trusted executable delegation only; Cube Play owns the local runtime.
use std::{
    error::Error,
    ffi::OsString,
    fmt,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use clap::Args;

const PROTOCOL: &[u8] = b"cube-play 1\n";

#[derive(Args)]
pub struct PlayArgs {
    /// Attach this command to a revocable context file explicitly issued by Cube Play
    #[arg(long)]
    pub context: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Option<cube_cli::play::Command>,
    /// Open a local source folder, file, package or Workspace link
    pub path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct DesktopExit(pub u8);
impl fmt::Display for DesktopExit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Cube Play exited with status {}", self.0)
    }
}
impl Error for DesktopExit {}

fn executable(configured: Option<OsString>, cube: &Path) -> io::Result<PathBuf> {
    let path = if let Some(path) = configured {
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "CUBE_PLAY_APP must be an absolute path to a trusted Cube Play desktop app",
            ));
        }
        path
    } else {
        cube.parent()
            .ok_or_else(|| io::Error::other("Cannot locate the Cube installation"))?
            .join(format!("cube-play{}", std::env::consts::EXE_SUFFIX))
    };
    let path = path.canonicalize().map_err(|_| io::Error::new(io::ErrorKind::NotFound,
        "Cube Play desktop app is not installed; install cube-play beside cube or set CUBE_PLAY_APP to its trusted absolute path"))?;
    if !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "CUBE_PLAY_APP must identify an executable file",
        ));
    }
    Ok(path)
}

struct Probe(Child);
impl Drop for Probe {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

fn verify_protocol(path: &Path) -> io::Result<()> {
    let mut child = Probe(Command::new(path).arg("--desktop-protocol")
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn()
        .map_err(|_| io::Error::other("Could not start the Cube Play desktop app; check installation and executable permissions"))?);
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut stdout = child
        .0
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("Desktop protocol output is unavailable"))?;
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let mut output = Vec::new();
        let result = (&mut stdout)
            .take(65)
            .read_to_end(&mut output)
            .map(|_| output);
        let _ = sender.send(result);
    });
    loop {
        if let Some(status) = child.0.try_wait()? {
            // A helper retaining stdout must not bypass the protocol deadline.
            let output = receiver
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::TimedOut,
                        "Cube Play desktop app protocol output timed out",
                    )
                })??;
            return if status.success() && output == PROTOCOL {
                Ok(())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Incompatible Cube Play desktop app; install a desktop app supporting protocol 1",
                ))
            };
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Cube Play desktop app protocol check timed out; reinstall the trusted desktop app",
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

pub fn run(args: &PlayArgs) -> Result<(), Box<dyn Error>> {
    let path = executable(std::env::var_os("CUBE_PLAY_APP"), &std::env::current_exe()?)?;
    verify_protocol(&path)?;
    let mut command = Command::new(path);
    if args.context.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "A context file requires an Asset, Pipeline or Run command",
        )
        .into());
    }
    if let Some(value) = &args.path {
        let path = if value
            .to_str()
            .is_some_and(|s| s.starts_with("cube-play-open:"))
        {
            value.clone()
        } else {
            value.canonicalize()?
        };
        command.arg("--desktop-open").arg(path);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Err(command.exec().into())
    }
    #[cfg(not(unix))]
    {
        let status = command.status()?;
        if status.success() {
            Ok(())
        } else {
            Err(Box::new(DesktopExit(
                status
                    .code()
                    .and_then(|code| u8::try_from(code).ok())
                    .unwrap_or(1),
            )))
        }
    }
}
