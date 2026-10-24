//! Caller-owned local I/O around the in-memory pipeline engine; recipes grant no I/O.
use clap::{Args, Subcommand, ValueEnum};
use cube_core::{
    forensics,
    pipeline::{self, Execution, Kind, Limits, Value},
};
use std::{
    error::Error,
    io::{Read, Write},
    path::{Path, PathBuf},
};
#[derive(Args)]
pub(crate) struct Definition {
    /// Ordered JSON steps with explicit operation-contract versions
    #[arg(long)]
    steps: String,
    /// Optional JSON limits, lowered from hard bounds
    #[arg(long)]
    limits: Option<String>,
}
#[derive(Args)]
#[group(multiple=false,args=["text","hex","input","value"])]
pub(crate) struct Input {
    #[arg(long)]
    text: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    hex: Option<String>,
    /// Explicit regular file containing raw input bytes; omitted selectors read raw stdin
    #[arg(long)]
    input: Option<PathBuf>,
    /// Explicit tagged pipeline JSON value (including inert byte_range)
    #[arg(long)]
    value: Option<String>,
}
impl Input {
    fn read(self) -> Result<Value, Box<dyn Error>> {
        Ok(match (self.text, self.hex, self.input, self.value) {
            (Some(data), _, _, _) => Value::Text { data },
            (_, Some(hex), _, _) => {
                if hex.len() > pipeline::MAX_VALUE_BYTES * 2 {
                    return Err("hex input exceeds value budget".into());
                }
                Value::Bytes {
                    data: cube_core::encoding::hex_decode(&hex)?,
                }
            }
            (_, _, Some(path), _) => Value::Bytes {
                data: read_file(&path, pipeline::MAX_VALUE_BYTES)?,
            },
            (_, _, _, Some(json)) => pipeline::json::value(&json)?,
            _ => Value::Bytes {
                data: crate::Input { text: None }
                    .read_bounded(pipeline::MAX_VALUE_BYTES, "pipeline.execute")?,
            },
        })
    }
}
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Format {
    Hex,
    Raw,
    Json,
}
#[derive(Args)]
pub(crate) struct Output {
    /// Byte output is hex by default; text is UTF-8 with a final newline; json preserves types/captures
    #[arg(long,value_enum,default_value_t=Format::Hex)]
    format: Format,
    /// Exclusive new destination; omit to write stdout
    #[arg(long)]
    output: Option<PathBuf>,
    /// Zero-based step outputs to retain; requires --format json
    #[arg(long, value_delimiter = ',')]
    pub(super) capture: Vec<usize>,
}
impl Output {
    pub(super) fn check(&self) -> Result<(), Box<dyn Error>> {
        if !self.capture.is_empty() && !matches!(self.format, Format::Json) {
            return Err("capture requires --format json".into());
        }
        Ok(())
    }
    pub(super) fn write(
        &self,
        result: &Execution,
        out: &mut impl Write,
    ) -> Result<(), Box<dyn Error>> {
        let mut data = match (&result.output, self.format) {
            (_, Format::Json) => serde_json::to_vec_pretty(result)?,
            (Value::Bytes { data }, Format::Hex) => {
                cube_core::encoding::hex_encode(data).into_bytes()
            }
            (Value::Bytes { data }, Format::Raw) => data.clone(),
            (Value::Text { data }, _) => data.as_bytes().to_vec(),
            (Value::ByteRange { .. }, Format::Hex) => serde_json::to_vec(&result.output)?,
            (Value::ByteRange { .. }, Format::Raw) => {
                return Err("byte_range requires JSON output".into());
            }
        };
        if !matches!(self.format, Format::Raw) {
            data.push(b'\n');
        }
        if let Some(path) = &self.output {
            write_new(path, &data)?;
        } else {
            out.write_all(&data)?;
        }
        Ok(())
    }
}
pub(super) fn read_file(path: &Path, limit: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let file = forensics::open_regular(path)?;
    if file.metadata()?.len() > limit as u64 {
        return Err("file exceeds explicit input byte limit".into());
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err("file grew beyond input byte limit".into());
    }
    Ok(bytes)
}
fn write_new(path: &Path, data: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    let result = file.write_all(data).and_then(|_| file.flush());
    drop(file);
    if result.is_err() {
        let _ = std::fs::remove_file(path);
    }
    result?;
    Ok(())
}
#[derive(Subcommand)]
pub(crate) enum PipelineCommand {
    /// List explicitly executable bounded adapters and operation-contract versions
    Adapters,
    /// Validate every step without running any algorithm or reading pipeline input
    Validate {
        #[command(flatten)]
        definition: Definition,
        #[arg(long)]
        input_type: Kind,
    },
    /// Execute a typed linear pipeline with deterministic bounded core operations
    Run {
        #[command(flatten)]
        definition: Definition,
        #[command(flatten)]
        input: Input,
        #[command(flatten)]
        output: Output,
    },
}
fn limits(definition: &Definition) -> Result<Limits, pipeline::Error> {
    definition
        .limits
        .as_deref()
        .map(pipeline::json::limits)
        .transpose()
        .map(Option::unwrap_or_default)
}
pub(crate) fn run(command: PipelineCommand, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        PipelineCommand::Adapters => writeln!(
            out,
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({"engine_api":pipeline::ENGINE_API,"engine_version":env!("CARGO_PKG_VERSION"),"adapters":pipeline::supported_adapters()})
            )?
        )?,
        PipelineCommand::Validate {
            definition,
            input_type,
        } => {
            let plan = pipeline::Plan::validate(
                input_type,
                pipeline::json::steps(&definition.steps)?,
                limits(&definition)?,
            )?;
            writeln!(out, "{}", serde_json::to_string_pretty(&plan.info())?)?;
        }
        PipelineCommand::Run {
            definition,
            input,
            output,
        } => {
            output.check()?;
            let value = input.read()?;
            let plan = pipeline::Plan::validate(
                value.kind(),
                pipeline::json::steps(&definition.steps)?,
                limits(&definition)?,
            )?;
            let result = plan.execute(value, &output.capture, None)?;
            output.write(&result, out)?;
        }
    }
    Ok(())
}
