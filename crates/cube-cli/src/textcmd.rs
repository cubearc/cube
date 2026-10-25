//! Adaptation of explicit Unicode text operations. Algorithms live in Rust core.
use crate::Input;
use clap::{Args, Subcommand};
use cube_core::text;
use std::{error::Error, io::Write};
#[derive(Args)]
#[group(required=true,multiple=false,args=["indices","stride"])]
pub(crate) struct ExtractArgs {
    #[command(flatten)]
    input: Input,
    /// Ordered zero-based Unicode scalar indexes; duplicates preserved; empty string means empty list
    #[arg(long,conflicts_with_all=["start","stride","stop"])]
    indices: Option<String>,
    /// Stride start, default zero (requires --stride)
    #[arg(long, requires = "stride")]
    start: Option<usize>,
    /// Positive stride; --start 0/1 --stride 2 extracts alternating characters
    #[arg(long)]
    stride: Option<usize>,
    /// Exclusive stop, default scalar length (requires --stride)
    #[arg(long, requires = "stride")]
    stop: Option<usize>,
}
#[derive(Subcommand)]
pub(crate) enum TextCommand {
    /// Scalar frequencies, overlapping n-grams, IC and entropy; observations only
    Statistics {
        #[command(flatten)]
        input: Input,
        #[arg(long, default_value_t = 1)]
        ngram_size: usize,
        /// none, NFC, NFD, NFKC or NFKD
        #[arg(long, default_value = "none")]
        normalization: String,
        /// preserve or ascii_lower
        #[arg(long, default_value = "preserve")]
        case: String,
        /// all or ascii_letters
        #[arg(long, default_value = "all")]
        filter: String,
        #[arg(long, default_value_t = 1000)]
        max_results: usize,
        #[arg(long)]
        json: bool,
    },
    /// First/last scalar per word/line; line_first is an acrostic without trimming
    Select {
        #[command(flatten)]
        input: Input,
        #[arg(long)]
        selector: String,
    },
    /// Explicit UTF-8 text to bytes (no normalization/newline); <=1 MiB
    ToBytes {
        #[command(flatten)]
        input: Input,
        #[arg(long,value_enum,default_value_t=crate::Output::Hex)]
        output: crate::Output,
    },
    /// Explicit bytes to UTF-8 text; invalid bytes fail rather than being replaced
    FromBytes(Input),
    /// Reverse scalar values (not graphemes or UTF-8 bytes), including CRLF/combining marks; <=1 MiB
    Reverse(Input),
    /// Explicit Unicode NFC/NFD/NFKC/NFKD; <=1 MiB input, <=8 MiB output
    Normalize {
        #[command(flatten)]
        input: Input,
        #[arg(long)]
        form: String,
    },
    /// Select scalar positions (not UTF-8 bytes); no normalization, <=1 MiB
    Extract(ExtractArgs),
}
pub(crate) fn run(command: TextCommand, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let result = match command {
        TextCommand::Statistics {
            input,
            ngram_size,
            normalization,
            case,
            filter,
            max_results,
            json,
        } => {
            use text::statistics::{self, CaseMode, Normalization, Options, TextFilter};
            let report = statistics::analyze(
                &input.read_text("text.statistics")?,
                Options {
                    ngram_size,
                    normalization: Normalization::parse(&normalization)?,
                    case: CaseMode::parse(&case)?,
                    filter: TextFilter::parse(&filter)?,
                    max_results,
                },
            )?;
            if json {
                writeln!(output, "{}", serde_json::to_string_pretty(&report)?)?;
            } else {
                writeln!(
                    output,
                    "Scalars: {}; windows: {}; distinct n-grams: {}; listing complete: {}",
                    report.scalar_count, report.windows, report.distinct_ngrams, report.complete
                )?;
                let number = |value: Option<f64>| {
                    value
                        .map(|v| format!("{v:.12}"))
                        .unwrap_or_else(|| "undefined".into())
                };
                writeln!(
                    output,
                    "IC: {}; entropy (bits/scalar): {}",
                    number(report.index_of_coincidence),
                    number(report.shannon_entropy_bits)
                )?;
                for item in report.frequencies {
                    writeln!(output, "{:?}\t{}", item.ngram, item.count)?;
                }
            }
            return Ok(());
        }
        TextCommand::Select { input, selector } => text::selection::select(
            &input.read_text("text.select")?,
            text::selection::Selector::parse(&selector)?,
        )?,
        TextCommand::ToBytes {
            input,
            output: format,
        } => {
            let data = text::to_utf8(&input.read_text("text.utf8.encode")?)?;
            crate::write_bytes(output, &data, format)?;
            return Ok(());
        }
        TextCommand::FromBytes(input) => {
            text::from_utf8(&input.read_bounded(text::MAX_INPUT, "text.utf8.decode")?)?
        }
        TextCommand::Reverse(input) => text::reverse(&input.read_text("text.reverse")?)?,
        TextCommand::Normalize { input, form } => {
            text::normalize(&input.read_text("text.normalize")?, &form)?
        }
        TextCommand::Extract(args) => {
            let indices = args
                .indices
                .map(|value| {
                    if value.is_empty() {
                        return Ok(Vec::new());
                    }
                    if value.len() > text::MAX_INDICES * 21 {
                        return Err(cube_core::errors::OperationError::new(
                            cube_core::errors::ErrorCode::ResourceLimit,
                            "index argument exceeds byte budget",
                        )
                        .operation("text.extract")
                        .parameter("indices"));
                    }
                    let mut indices = Vec::new();
                    for field in value.split(',') {
                        if indices.len() == text::MAX_INDICES {
                            return Err(cube_core::errors::OperationError::new(
                                cube_core::errors::ErrorCode::ResourceLimit,
                                "selection exceeds 65536 indexes",
                            )
                            .operation("text.extract")
                            .parameter("indices"));
                        }
                        indices.push(field.parse::<usize>().map_err(|_| {
                            cube_core::errors::OperationError::new(
                                cube_core::errors::ErrorCode::InvalidParameter,
                                "indices must be comma-separated unsigned scalar positions",
                            )
                            .operation("text.extract")
                            .parameter("indices")
                        })?);
                    }
                    Ok(indices)
                })
                .transpose()?;
            text::extract(
                &args.input.read_text("text.extract")?,
                indices.as_deref(),
                args.start,
                args.stride,
                args.stop,
            )?
        }
    };
    crate::write_text(output, &result)?;
    Ok(())
}
