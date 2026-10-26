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

const PROTOCOL: &[u8] = b"cube-play-launcher 1\n";

#[derive(Args)]
pub struct PlayArgs {
    /// Local directory, file or challenge; defaults to the current directory
    pub path: Option<PathBuf>,
    /// Stop this workspace's runtime without deleting investigation data
    #[arg(long)]
    pub stop: bool,
    /// Print the session link without opening a browser
    #[arg(long, conflicts_with = "stop")]
    pub no_browser: bool,
    /// Require an installed runtime image; never build or download during startup
    #[arg(long, conflicts_with = "stop")]
    pub offline: bool,
}

#[derive(Debug)]
pub struct LauncherExit(pub u8);
impl fmt::Display for LauncherExit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Cube Play exited with status {}", self.0)
    }
}
impl Error for LauncherExit {}

fn executable(configured: Option<OsString>, cube: &Path) -> io::Result<PathBuf> {
    let path = if let Some(path) = configured {
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "CUBE_PLAY_LAUNCHER must be an absolute path to a trusted Cube Play launcher",
            ));
        }
        path
    } else {
        cube.parent()
            .ok_or_else(|| io::Error::other("Cannot locate the Cube installation"))?
            .join(format!("cube-play{}", std::env::consts::EXE_SUFFIX))
    };
    let path = path.canonicalize().map_err(|_| io::Error::new(io::ErrorKind::NotFound,
        "Cube Play launcher is not installed; install cube-play beside cube or set CUBE_PLAY_LAUNCHER to its trusted absolute path"))?;
    if !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "CUBE_PLAY_LAUNCHER must identify an executable file",
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
    let mut child = Probe(Command::new(path).arg("--launcher-protocol")
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn()
        .map_err(|_| io::Error::other("Could not start the Cube Play launcher; check installation and executable permissions"))?);
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut stdout = child
        .0
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("Launcher protocol output is unavailable"))?;
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
                        "Cube Play launcher protocol output timed out",
                    )
                })??;
            return if status.success() && output == PROTOCOL {
                Ok(())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Incompatible Cube Play launcher; install a launcher supporting protocol 1",
                ))
            };
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Cube Play launcher protocol check timed out; reinstall the trusted launcher",
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

pub fn run(args: &PlayArgs, json_errors: bool) -> Result<(), Box<dyn Error>> {
    let path = executable(
        std::env::var_os("CUBE_PLAY_LAUNCHER"),
        &std::env::current_exe()?,
    )?;
    verify_protocol(&path)?;
    let mut command = Command::new(path);
    command.args(["--protocol", "1"]);
    if json_errors {
        command.arg("--json-errors");
    }
    if args.stop {
        command.arg("--stop");
    }
    if args.no_browser {
        command.arg("--no-browser");
    }
    if args.offline {
        command.arg("--offline");
    }
    if let Some(path) = &args.path {
        command.arg("--").arg(path);
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
            Err(Box::new(LauncherExit(
                status
                    .code()
                    .and_then(|code| u8::try_from(code).ok())
                    .unwrap_or(1),
            )))
        }
    }
}
