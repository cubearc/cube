//! Unicode JSON observations; normalized offsets are a separate coordinate space.
use cube_core::text::unicode::{self, Options};
use std::{error::Error, io::Write};
#[derive(clap::Args)]
pub(crate) struct Listing {
    #[command(flatten)]
    input: crate::Input,
    #[arg(long, default_value_t = 1000)]
    max_results: usize,
    #[arg(long,default_value_t=unicode::MAX_WORK)]
    max_work: u64,
    #[arg(long,default_value_t=cube_core::text::MAX_OUTPUT)]
    max_output: usize,
}
impl Listing {
    fn options(&self) -> Options {
        Options {
            max_results: self.max_results,
            max_work: self.max_work,
            max_output: self.max_output,
        }
    }
}
#[derive(clap::Args)]
#[group(required=true,multiple=false,args=["other","other_file"])]
pub(crate) struct Compare {
    #[command(flatten)]
    listing: Listing,
    #[arg(long)]
    other: Option<String>,
    /// Exact caller-selected UTF-8 file for the right input, <=1 MiB
    #[arg(long)]
    other_file: Option<std::path::PathBuf>,
    #[arg(long, default_value = "none")]
    normalization: String,
}
pub(crate) fn inspect(args: Listing, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let options = args.options();
    let report = unicode::inspect(&args.input.read_text("text.unicode.inspect")?, &options)?;
    writeln!(output, "{}", serde_json::to_string(&report)?)?;
    Ok(())
}
pub(crate) fn compare(args: Compare, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let options = args.listing.options();
    let other = if let Some(other) = args.other {
        other
    } else {
        let bytes = crate::pipelinecmd::read_file(
            &args.other_file.expect("clap requires a right input"),
            cube_core::text::MAX_INPUT,
        )?;
        String::from_utf8(bytes).map_err(|_| {
            cube_core::errors::OperationError::new(
                cube_core::errors::ErrorCode::InvalidInput,
                "right input must be UTF-8",
            )
            .operation("text.unicode.compare")
            .parameter("other")
        })?
    };
    let report = unicode::compare(
        &args.listing.input.read_text("text.unicode.compare")?,
        &other,
        &args.normalization,
        &options,
    )?;
    writeln!(output, "{}", serde_json::to_string(&report)?)?;
    Ok(())
}
