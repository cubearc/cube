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
#[derive(Args)]
pub(crate) struct MaskedArgs {
    #[command(flatten)]
    input: Input,
    /// indices, every_nth or fibonacci; every Unicode scalar counts
    #[arg(long)]
    selector: String,
    /// Comma-separated explicit scalar positions, sorted/deduplicated; empty selects none
    #[arg(long)]
    indices: Option<String>,
    /// Select the Nth, 2Nth, ... scalar (positive)
    #[arg(long)]
    nth: Option<usize>,
    /// Comma-separated Fibonacci values to exclude (not sequence ordinals)
    #[arg(long)]
    exclude: Option<String>,
    #[arg(long, default_value_t = 0)]
    index_base: u8,
    /// atbash, caesar, rot13 or identity
    #[arg(long)]
    selected: String,
    /// Explicit signed i32 Caesar shift; only valid for selected caesar
    #[arg(long, allow_hyphen_values = true)]
    selected_shift: Option<i32>,
    #[arg(long, default_value = "identity")]
    otherwise: String,
    #[arg(long, allow_hyphen_values = true)]
    otherwise_shift: Option<i32>,
    /// Explicit POSITION=SCALAR replacement after transformation; repeat to apply in order
    #[arg(long = "override")]
    overrides: Vec<String>,
    /// Include original input, effective configuration, positions and override audit
    #[arg(long)]
    json: bool,
}
#[derive(Subcommand)]
pub(crate) enum TransformCommand {
    /// Apply separate ASCII ciphers by scalar position; no normalization
    Masked(MaskedArgs),
}
fn position_list(
    value: Option<String>,
    parameter: &'static str,
) -> Result<Option<Vec<usize>>, cube_core::errors::OperationError> {
    value
        .map(|value| {
            let error = |code, message| {
                cube_core::errors::OperationError::new(code, message)
                    .operation("text.transform.masked")
                    .parameter(parameter)
            };
            let limit = || {
                error(
                    cube_core::errors::ErrorCode::ResourceLimit,
                    "position list exceeds byte or entry budget",
                )
            };
            if value.len() > text::MAX_INDICES * 21 {
                return Err(limit());
            }
            if value.is_empty() {
                return Ok(Vec::new());
            }
            let mut positions = Vec::new();
            for field in value.split(',') {
                if positions.len() == text::MAX_INDICES {
                    return Err(limit());
                }
                positions.push(field.parse().map_err(|_| {
                    error(
                        cube_core::errors::ErrorCode::InvalidParameter,
                        "positions must be comma-separated unsigned integers",
                    )
                })?);
            }
            Ok(positions)
        })
        .transpose()
}
#[derive(Subcommand)]
pub(crate) enum TextCommand {
    /// Exact caller-dictionary multiword anagrams (JSON)
    Anagram {
        #[command(subcommand)]
        command: crate::lexicalcmd::Anagram,
    },
    /// Exact substitution-only dictionary ladders (JSON)
    WordLadder {
        #[command(subcommand)]
        command: crate::lexicalcmd::Ladder,
    },
    /// Complete canonical scalar inventories and exact subtraction (JSON)
    Multiset {
        #[command(subcommand)]
        command: crate::multisetcmd::Command,
    },
    /// Versioned case/category observations with exact counts and bounded listings
    UnicodeInspect(crate::unicodecmd::Listing),
    /// Original and explicitly normalized scalar-index differences (JSON)
    UnicodeCompare(crate::unicodecmd::Compare),
    /// Original word boundaries and exact scalar lengths (JSON)
    WordLengths(crate::Input),
    /// Exact numeric scalar values and explicit presentation (JSON)
    Codepoints(crate::valuescmd::Codepoints),
    /// Per-word values, explicit tables and checked sums (JSON)
    WordValues(crate::valuescmd::WordValues),
    /// Signed shared/per-word positions and explicit short-word omission records (JSON)
    WordExtract(crate::wordcmd::Args),
    /// Retained scalar ring with explicit cyclic steps/count, JSON provenance
    KeyedSkip(crate::traversalcmd::Args),
    /// Remove selected scalars and resume next remaining, JSON provenance
    Josephus(crate::traversalcmd::Args),
    /// Remap exact scalars by physical key/layer positions
    Keyboard(crate::puzzlecmd::KeyboardArgs),
    /// Explicit row,column,layer coordinates with exact position reports
    KeyboardCoordinates(crate::puzzlecodescmd::Reversible),
    /// Signed displacement on explicit geometry; inverse only if bijective
    KeyboardShift(crate::puzzlecodescmd::Reversible),
    /// Map disjoint scalar classes with original positions and explicit filtering (JSON)
    Class(crate::puzzlecmd::ClassArgs),
    /// Position-dependent text transformations
    Transform {
        #[command(subcommand)]
        command: TransformCommand,
    },
    /// Stream bounded Cartesian strings as JSONL, preserving embedded newlines
    Combinations {
        #[command(flatten)]
        input: Input,
        #[arg(long, default_value = "")]
        separator: String,
        #[arg(long, default_value_t = 1000)]
        max_results: usize,
        /// Validate and show counts/bounds without generating results
        #[arg(long)]
        plan: bool,
    },
    /// Inspect factor pairs or render exact scalar grids; no inferred message
    Grid {
        #[command(flatten)]
        input: Input,
        #[arg(long)]
        width: Option<u32>,
        #[arg(long)]
        drop_whitespace: bool,
        #[arg(long, default_value = "row", requires = "width")]
        traversal: String,
        #[arg(long, default_value_t = 128)]
        max_candidates: usize,
        /// JSON array of [scalar,eight RGBA Hex digits]; requires --output
        #[arg(long,requires_all=["width","output"])]
        colors: Option<String>,
        #[arg(long, requires = "colors")]
        output: Option<std::path::PathBuf>,
        #[arg(long)]
        json: bool,
    },
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
        TextCommand::Anagram { command } => return crate::lexicalcmd::anagram(command, output),
        TextCommand::WordLadder { command } => return crate::lexicalcmd::ladder(command, output),
        TextCommand::Multiset { command } => return crate::multisetcmd::run(command, output),
        TextCommand::UnicodeInspect(args) => return crate::unicodecmd::inspect(args, output),
        TextCommand::UnicodeCompare(args) => return crate::unicodecmd::compare(args, output),
        TextCommand::WordLengths(input) => return crate::valuescmd::lengths(input, output),
        TextCommand::Codepoints(args) => return crate::valuescmd::codepoints(args, output),
        TextCommand::WordValues(args) => return crate::valuescmd::words(args, output),
        TextCommand::WordExtract(args) => return crate::wordcmd::run(args, output),
        TextCommand::KeyedSkip(args) => return crate::traversalcmd::run(args, false, output),
        TextCommand::Josephus(args) => return crate::traversalcmd::run(args, true, output),
        TextCommand::Keyboard(args) => crate::puzzlecmd::keyboard(args)?,
        TextCommand::KeyboardCoordinates(args) => {
            crate::puzzlecodescmd::keyboard_coordinates(args)?
        }
        TextCommand::KeyboardShift(args) => crate::puzzlecodescmd::keyboard_shift(args)?,
        TextCommand::Class(args) => return crate::puzzlecmd::classes(args, output),
        TextCommand::Transform {
            command: TransformCommand::Masked(args),
        } => {
            use text::{
                positions::{IndexBase, Selector},
                transform::{self, Config, Override, Transformation},
            };
            let report = transform::masked(
                &args.input.read_text("text.transform.masked")?,
                &Config {
                    index_base: IndexBase::try_from(args.index_base)?,
                    selector: Selector::from_options(
                        &args.selector,
                        position_list(args.indices, "indices")?,
                        args.nth,
                        position_list(args.exclude, "exclude")?,
                    )?,
                    selected: Transformation::parse(&args.selected, args.selected_shift)?,
                    otherwise: Transformation::parse(&args.otherwise, args.otherwise_shift)?,
                    overrides: args
                        .overrides
                        .iter()
                        .map(|s| Override::parse(s))
                        .collect::<Result<_, _>>()?,
                },
            )?;
            if args.json {
                writeln!(output, "{}", serde_json::to_string_pretty(&report)?)?;
            } else {
                crate::write_text(output, &report.final_text)?;
            }
            return Ok(());
        }
        TextCommand::Combinations {
            input,
            separator,
            max_results,
            plan,
        } => {
            let groups = text::combinations::parse_groups(&input.read_text_limit(
                cube_core::pipeline::json::MAX_DOCUMENT_BYTES,
                "text.combinations",
            )?)?;
            let combinations =
                text::combinations::Combinations::new(groups, separator, max_results)?;
            if plan {
                writeln!(
                    output,
                    "{}",
                    serde_json::to_string_pretty(combinations.plan())?
                )?;
            } else {
                for value in combinations {
                    writeln!(output, "{}", serde_json::to_string(&value)?)?;
                }
            }
            return Ok(());
        }
        TextCommand::Grid {
            input,
            width,
            drop_whitespace,
            traversal,
            max_candidates,
            colors,
            output: destination,
            json,
        } => {
            let input = input.read_text("text.grid.render")?;
            if let Some(width) = width {
                let grid = text::grid::render(
                    &input,
                    width,
                    drop_whitespace,
                    cube_core::image::Traversal::parse(&traversal)?,
                )?;
                if let Some(colors) = colors {
                    grid.image(&text::grid::parse_colors(&colors)?)?
                        .save_png(&destination.expect("required output"))?;
                }
                if json {
                    writeln!(output, "{}", serde_json::to_string_pretty(&grid)?)?;
                } else {
                    output.write_all(grid.text()?.as_bytes())?;
                }
            } else {
                let factors = text::grid::factors(&input, drop_whitespace, max_candidates)?;
                if json {
                    writeln!(output, "{}", serde_json::to_string_pretty(&factors)?)?;
                } else {
                    for d in factors.candidates {
                        writeln!(output, "{}x{}", d.width, d.height)?;
                    }
                }
            }
            return Ok(());
        }
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
