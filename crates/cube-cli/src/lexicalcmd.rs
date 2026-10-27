//! Explicit local dictionary transport; search and graph algorithms stay in Core.
use clap::{Args, Subcommand};
use cube_core::text::{
    anagram as a, lexical,
    multiset::Profile,
    statistics::{CaseMode, Normalization, TextFilter},
    word_ladder as l,
};
use std::{
    error::Error,
    io::{Read, Write},
    path::PathBuf,
};
#[derive(Args)]
pub(crate) struct Dictionary {
    /// Ordered JSON string array, no bundled corpus
    #[arg(
        long,
        required_unless_present = "dictionary_file",
        conflicts_with = "dictionary_file"
    )]
    dictionary: Option<String>,
    /// Bounded regular UTF-8 file containing a JSON string array
    #[arg(
        long,
        required_unless_present = "dictionary",
        conflicts_with = "dictionary"
    )]
    dictionary_file: Option<PathBuf>,
    #[arg(long, default_value = "none")]
    normalization: String,
    #[arg(long, default_value = "preserve")]
    case: String,
    #[arg(long,default_value_t=32*1024*1024)]
    max_memory_bytes: usize,
    #[arg(long,default_value_t=8*1024*1024)]
    max_report_bytes: usize,
}
impl Dictionary {
    fn load(&self, max: usize) -> Result<Vec<String>, Box<dyn Error>> {
        let text = if let Some(s) = &self.dictionary {
            s.clone()
        } else {
            let mut bytes = Vec::new();
            cube_core::forensics::open_regular(
                self.dictionary_file
                    .as_ref()
                    .ok_or("explicit dictionary is required")?,
            )?
            .take(max as u64 + 1)
            .read_to_end(&mut bytes)?;
            if bytes.len() > max {
                return Err(cube_core::errors::OperationError::new(
                    cube_core::errors::ErrorCode::ResourceLimit,
                    "dictionary JSON exceeds transport limit",
                )
                .operation("text.dictionary.parse")
                .parameter("dictionary")
                .into());
            }
            String::from_utf8(bytes).map_err(|e| {
                cube_core::errors::OperationError::new(
                    cube_core::errors::ErrorCode::InvalidInput,
                    "dictionary file must contain UTF-8 JSON",
                )
                .operation("text.dictionary.parse")
                .parameter("dictionary")
                .caused_by(e)
            })?
        };
        Ok(lexical::parse_dictionary(&text, max, 10000)?)
    }
    fn profile(&self, filter: TextFilter) -> Result<Profile, Box<dyn Error>> {
        Ok(Profile {
            normalization: Normalization::parse(&self.normalization)?,
            case: CaseMode::parse(&self.case)?,
            filter,
        })
    }
}
#[derive(Subcommand)]
pub(crate) enum Anagram {
    Search(AnagramArgs),
}
#[derive(Args)]
pub(crate) struct AnagramArgs {
    #[command(flatten)]
    input: crate::Input,
    #[command(flatten)]
    dictionary: Dictionary,
    #[arg(long, default_value = "all")]
    filter: String,
    #[arg(long, default_value_t = 8)]
    max_words: usize,
    #[arg(long)]
    reuse: bool,
    #[arg(long, default_value_t = 100)]
    max_results: usize,
    #[arg(long,default_value_t=a::MAX_WORK)]
    max_work: usize,
}
pub(crate) fn anagram(command: Anagram, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let Anagram::Search(x) = command;
    let words = x.dictionary.load(a::MAX_DICTIONARY_BYTES)?;
    let report = a::search(
        &x.input.read_text("text.anagram.search")?,
        &words,
        x.dictionary.profile(TextFilter::parse(&x.filter)?)?,
        a::Options {
            max_words: x.max_words,
            reuse: x.reuse,
            max_results: x.max_results,
            max_work: x.max_work,
            max_memory_bytes: x.dictionary.max_memory_bytes,
            max_report_bytes: x.dictionary.max_report_bytes,
        },
    )?;
    serde_json::to_writer(&mut *out, &report)?;
    writeln!(out)?;
    Ok(())
}
#[derive(Args)]
pub(crate) struct LadderArgs {
    #[command(flatten)]
    dictionary: Dictionary,
    #[arg(long,default_value_t=l::MAX_WORK)]
    max_work: usize,
}
#[derive(Args)]
pub(crate) struct Validate {
    #[command(flatten)]
    args: LadderArgs,
    #[arg(long)]
    path: String,
}
#[derive(Args)]
pub(crate) struct Shortest {
    #[command(flatten)]
    args: LadderArgs,
    #[arg(long)]
    start: String,
    #[arg(long)]
    end: String,
}
#[derive(Subcommand)]
pub(crate) enum Ladder {
    Validate(Validate),
    Shortest(Shortest),
}
impl LadderArgs {
    fn options(&self) -> l::Options {
        l::Options {
            max_work: self.max_work,
            max_memory_bytes: self.dictionary.max_memory_bytes,
            max_report_bytes: self.dictionary.max_report_bytes,
        }
    }
}
pub(crate) fn ladder(command: Ladder, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let report = match command {
        Ladder::Validate(v) => {
            let words = v.args.dictionary.load(l::MAX_DICTIONARY_BYTES)?;
            let path = lexical::parse_dictionary(&v.path, l::MAX_DICTIONARY_BYTES, 4096)?;
            l::validate(
                &words,
                &path,
                v.args.dictionary.profile(TextFilter::All)?,
                v.args.options(),
            )?
        }
        Ladder::Shortest(v) => {
            let words = v.args.dictionary.load(l::MAX_DICTIONARY_BYTES)?;
            l::shortest(
                &words,
                &v.start,
                &v.end,
                v.args.dictionary.profile(TextFilter::All)?,
                v.args.options(),
            )?
        }
    };
    serde_json::to_writer(&mut *out, &report)?;
    writeln!(out)?;
    Ok(())
}
