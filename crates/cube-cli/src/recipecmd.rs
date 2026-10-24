//! Explicit caller-selected I/O for declarative recipes. Documents grant no I/O.
use crate::pipelinecmd::{Output, read_file};
use clap::{Args, Subcommand};
use cube_core::pipeline::{self, Value, recipe::Recipe};
use std::{
    collections::BTreeMap,
    error::Error,
    io::Write,
    path::{Path, PathBuf},
};
#[derive(Args)]
pub(crate) struct Bindings {
    /// Bind a text slot: name=value (no interpolation)
    #[arg(long)]
    text: Vec<String>,
    /// Bind raw bytes: name=hex
    #[arg(long)]
    hex: Vec<String>,
    /// Explicitly read a regular file as bytes: name=path
    #[arg(long)]
    input: Vec<String>,
    /// Bind a tagged JSON value: name={...}
    #[arg(long)]
    value: Vec<String>,
}
impl Bindings {
    fn read(self) -> Result<BTreeMap<String, Value>, Box<dyn Error>> {
        if self.text.len() + self.hex.len() + self.input.len() + self.value.len() > 1 {
            return Err("recipe format 1 permits at most one named input binding".into());
        }
        let mut result = BTreeMap::new();
        for (values, kind) in [
            (self.text, "text"),
            (self.hex, "hex"),
            (self.input, "input"),
            (self.value, "value"),
        ] {
            for binding in values {
                let (name, data) = binding
                    .split_once('=')
                    .ok_or("binding requires name=value")?;
                pipeline::recipe::validate_slot(name)?;
                let value = match kind {
                    "text" => Value::Text { data: data.into() },
                    "hex" => {
                        if data.len() > pipeline::MAX_VALUE_BYTES * 2 {
                            return Err("hex binding exceeds byte limit".into());
                        }
                        Value::Bytes {
                            data: cube_core::encoding::hex_decode(data)?,
                        }
                    }
                    "input" => Value::Bytes {
                        data: read_file(Path::new(data), pipeline::MAX_VALUE_BYTES)?,
                    },
                    _ => pipeline::json::value(data)?,
                };
                value.validate(pipeline::MAX_VALUE_BYTES)?;
                result.insert(name.into(), value);
            }
        }
        Ok(result)
    }
}
#[derive(Subcommand)]
pub(crate) enum RecipeCommand {
    /// Validate all recipe contracts without reading slot data or executing algorithms
    Validate { recipe: PathBuf },
    /// Execute a recipe with explicit caller-selected input/output paths
    Run {
        recipe: PathBuf,
        #[command(flatten)]
        bindings: Bindings,
        #[command(flatten)]
        output: Output,
    },
    /// Independently test exact typed output; no output file is written
    Check {
        recipe: PathBuf,
        #[command(flatten)]
        bindings: Bindings,
        /// Exact expected tagged JSON value (bytes use canonical base64)
        #[arg(long)]
        expect: String,
    },
}
fn load(path: &Path) -> Result<Recipe, Box<dyn Error>> {
    let data = read_file(path, pipeline::json::MAX_DOCUMENT_BYTES)?;
    Ok(Recipe::parse(std::str::from_utf8(&data)?)?)
}
pub(crate) fn run(command: RecipeCommand, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        RecipeCommand::Validate { recipe } => writeln!(
            out,
            "{}",
            serde_json::to_string_pretty(&load(&recipe)?.info())?
        )?,
        RecipeCommand::Run {
            recipe,
            bindings,
            output,
        } => {
            output.check()?;
            let recipe = load(&recipe)?;
            let result = recipe.execute(&bindings.read()?, &output.capture, None)?;
            output.write(&result, out)?;
        }
        RecipeCommand::Check {
            recipe,
            bindings,
            expect,
        } => {
            let recipe = load(&recipe)?;
            let expected = pipeline::json::value(&expect)?;
            let result = recipe.check(&bindings.read()?, &expected)?;
            writeln!(out, "{}", serde_json::to_string_pretty(&result)?)?;
        }
    }
    Ok(())
}
