//! Bounded UTF-8 matching with the Rust Thompson NFA and PikeVM.
use super::{SourceSpan, check_report, invalid, parameter, resource};
use crate::errors::OperationError;
use crate::text::{MAX_INDICES, MAX_OUTPUT, check_input, mapping::ByteBudget, selection, word};
use regex_automata::{
    Input, PatternID, Span,
    nfa::thompson::{NFA, State, pikevm::PikeVM},
};
use regex_syntax::hir::{Hir, HirKind};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const OP: &str = "text.regex";
const TOKEN_OP: &str = "text.tokenize";
const PROFILE: &str = "regex-syntax-0.8.11/pikevm-0.4.18";
const MAX_PATTERN: usize = 4096;
const MAX_WORK: usize = 1 << 26;
const MAX_NFA: usize = 1 << 20;
const MAX_CACHE: usize = 8 << 20;
const MAX_BUILD_INPUT: usize = 12 << 20;
const MAX_RECORD_STORAGE: usize = 12 << 20;

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Exists,
    Matches,
    Captures,
    Split,
}

const fn default_results() -> usize {
    1000
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub text: String,
    pub pattern: String,
    pub action: Action,
    #[serde(default = "default_results")]
    #[schemars(range(max = 65536))]
    pub max_results: usize,
    #[serde(default)]
    pub keep_empty: Option<bool>,
}

#[derive(Serialize, JsonSchema)]
pub struct Record<'a> {
    pub text: &'a str,
    #[serde(flatten)]
    pub span: SourceSpan,
}

#[derive(Serialize, JsonSchema)]
pub struct Capture<'a> {
    pub group: usize,
    pub name: Option<String>,
    pub matched: Option<Record<'a>>,
}

#[derive(Serialize, JsonSchema)]
pub struct Captured<'a> {
    pub matched: Record<'a>,
    pub groups: Vec<Capture<'a>>,
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Report<'a> {
    Exists {
        profile: &'static str,
        exists: bool,
        work_bound: usize,
    },
    Matches {
        profile: &'static str,
        matches: Vec<Record<'a>>,
        work_bound: usize,
    },
    Captures {
        profile: &'static str,
        matches: Vec<Captured<'a>>,
        work_bound: usize,
    },
    Split {
        profile: &'static str,
        tokens: Vec<Record<'a>>,
        work_bound: usize,
    },
}

fn expansion(hir: &Hir) -> Option<usize> {
    let local = match hir.kind() {
        HirKind::Literal(literal) => literal.0.len().max(1),
        HirKind::Class(regex_syntax::hir::Class::Unicode(class)) => class.ranges().len(),
        HirKind::Class(regex_syntax::hir::Class::Bytes(class)) => class.ranges().len(),
        HirKind::Repetition(repetition) => {
            let copies = match repetition.max {
                Some(max) => max as usize,
                None => (repetition.min as usize).checked_add(1)?,
            };
            return expansion(&repetition.sub)?
                .checked_add(1)?
                .checked_mul(copies.max(1));
        }
        _ => 1,
    };
    hir.kind()
        .subs()
        .iter()
        .try_fold(local, |total, child| total.checked_add(expansion(child)?))
}

fn cache_bound(nfa: &NFA) -> usize {
    // Two active sets, two slot matrices, and the largest epsilon DFS stack.
    // Alternatives and capture restore frames are counted even if unreachable.
    let epsilon = nfa
        .states()
        .iter()
        .map(|state| match state {
            State::Union { alternates } => alternates.len(),
            State::BinaryUnion { .. } | State::Capture { .. } => 2,
            State::Look { .. } => 1,
            _ => 0,
        })
        .sum::<usize>();
    let states = nfa.states().len() + 1;
    let slots = nfa.group_info().slot_len();
    4 * 1024 + states * (32 + 16 * slots) + 64 * epsilon
}

fn compile(pattern: &str) -> Result<PikeVM, OperationError> {
    // Bound parser/HIR storage before Unicode property expansion. Every escape
    // can request a property table of up to 4096 scalar intervals. Reserve
    // 64 bytes per source byte for tokens, code points and AST bookkeeping.
    if pattern.len() > MAX_PATTERN {
        return Err(resource("regex pattern exceeds 4096 UTF-8 bytes"));
    }
    let class_atoms = pattern.bytes().filter(|&b| b == b'\\').count();
    if pattern.len() * 64 + class_atoms * 32_768 > MAX_BUILD_INPUT {
        return Err(resource("regex pattern exceeds its parser budget"));
    }
    let hir = regex_syntax::ParserBuilder::new().nest_limit(64).build().parse(pattern).map_err(|error| {
        if matches!(error, regex_syntax::Error::Parse(ref error) if matches!(error.kind(), regex_syntax::ast::ErrorKind::NestLimitExceeded(_))) {
            resource("regex pattern exceeds the nesting limit")
        } else {
            invalid("invalid or unsupported Rust regex syntax")
        }
    })?;
    if hir.properties().explicit_captures_len() > 64 {
        return Err(resource("regex pattern exceeds 64 capture groups"));
    }
    if expansion(&hir).is_none_or(|steps| steps > 1 << 17) {
        return Err(resource(
            "regex repeat expansion exceeds its construction limit",
        ));
    }
    let nfa = NFA::compiler()
        .configure(NFA::config().nfa_size_limit(Some(MAX_NFA)))
        .build_from_hir(&hir)
        .map_err(|error| {
            if error.size_limit().is_some() {
                resource("regex Thompson NFA exceeds 1 MiB")
            } else {
                invalid("unsupported Thompson NFA configuration")
            }
        })?;
    if nfa.memory_usage() > MAX_NFA || cache_bound(&nfa) > MAX_CACHE {
        return Err(resource(
            "regex NFA or PikeVM cache exceeds its memory limit",
        ));
    }
    PikeVM::new_from_nfa(nfa).map_err(|_| invalid("unsupported PikeVM configuration"))
}

fn search_bound(vm: &PikeVM, bytes: usize, calls: usize) -> Result<usize, OperationError> {
    [
        vm.get_nfa().states().len(),
        bytes + 1,
        calls,
        vm.get_nfa().group_info().group_len(PatternID::ZERO),
    ]
    .into_iter()
    .try_fold(8_usize, |total, part| total.checked_mul(part))
    .filter(|&steps| steps <= MAX_WORK)
    .ok_or_else(|| resource("regex conservative preflight exceeds its work limit"))
}

struct Source<'a> {
    text: &'a str,
    offsets: Vec<u32>,
}

impl<'a> Source<'a> {
    fn new(text: &'a str) -> Self {
        let mut offsets = Vec::with_capacity(text.chars().count() + 1);
        offsets.extend(text.char_indices().map(|(offset, _)| offset as u32));
        offsets.push(text.len() as u32);
        Self { text, offsets }
    }

    fn record(&self, bytes: Span) -> Record<'a> {
        let index = |offset| {
            self.offsets
                .binary_search(&(offset as u32))
                .expect("VM UTF-8 boundary")
        };
        Record {
            text: &self.text[bytes.start..bytes.end],
            span: SourceSpan {
                byte_start: bytes.start,
                byte_end: bytes.end,
                scalar_start: index(bytes.start),
                scalar_end: index(bytes.end),
            },
        }
    }
}

struct Records {
    bytes: ByteBudget,
    allocated: usize,
    count: usize,
    max: usize,
}

impl Records {
    fn new(max: usize) -> Self {
        Self {
            bytes: ByteBudget::new(MAX_OUTPUT),
            allocated: 0,
            count: 0,
            max,
        }
    }

    fn push<T: Serialize>(
        &mut self,
        output: &mut Vec<T>,
        row: T,
        dynamic: usize,
    ) -> Result<(), OperationError> {
        self.count += 1;
        self.allocated += 2 * std::mem::size_of::<T>() + dynamic;
        if self.count > self.max || self.allocated > MAX_RECORD_STORAGE {
            return Err(resource("regex report exceeds its record or storage cap"));
        }
        serde_json::to_writer(&mut self.bytes, &row)
            .map_err(|_| resource("regex report exceeds 8 MiB"))?;
        output.push(row);
        Ok(())
    }
}

fn advance(text: &str, matched: Span) -> usize {
    if matched.start != matched.end {
        return matched.end;
    }
    matched.end + text[matched.end..].chars().next().map_or(1, char::len_utf8)
}

fn list<'a>(
    vm: &PikeVM,
    request: &'a Request,
    work_bound: usize,
) -> Result<Report<'a>, OperationError> {
    let source = Source::new(&request.text);
    let mut cache = vm.create_cache();
    let mut caps = vm.create_captures();
    let mut at = 0;
    let mut previous = 0;
    let mut matches = Vec::new();
    let mut capture_rows = Vec::new();
    let mut tokens = Vec::new();
    let mut rows = Records::new(request.max_results);
    let mut delimiters = 0;
    while at <= source.text.len() {
        let input = Input::new(source.text).span(at..source.text.len());
        let matched = if matches!(request.action, Action::Captures) {
            vm.captures(&mut cache, input, &mut caps);
            caps.get_match()
        } else {
            vm.find(&mut cache, input)
        };
        let Some(matched) = matched else { break };
        delimiters += 1;
        if delimiters > request.max_results {
            return Err(resource("regex found an extra match beyond max_results"));
        }
        let span = matched.span();
        match request.action {
            Action::Captures => {
                let groups = (1..caps.group_len())
                    .map(|group| Capture {
                        group,
                        name: caps
                            .group_info()
                            .to_name(PatternID::ZERO, group)
                            .map(str::to_owned),
                        matched: caps.get_group(group).map(|span| source.record(span)),
                    })
                    .collect::<Vec<_>>();
                let dynamic = groups.capacity() * std::mem::size_of::<Capture>()
                    + groups
                        .iter()
                        .filter_map(|group| group.name.as_ref())
                        .map(String::capacity)
                        .sum::<usize>();
                rows.push(
                    &mut capture_rows,
                    Captured {
                        matched: source.record(span),
                        groups,
                    },
                    dynamic,
                )?;
            }
            Action::Matches => rows.push(&mut matches, source.record(span), 0)?,
            Action::Split => {
                split_record(
                    &source,
                    &mut rows,
                    &mut tokens,
                    previous,
                    span.start,
                    request.keep_empty,
                )?;
                previous = span.end;
            }
            Action::Exists => unreachable!("existence handled without listing"),
        }
        at = advance(source.text, span);
    }
    Ok(match request.action {
        Action::Matches => Report::Matches {
            profile: PROFILE,
            matches,
            work_bound,
        },
        Action::Captures => Report::Captures {
            profile: PROFILE,
            matches: capture_rows,
            work_bound,
        },
        Action::Split => {
            split_record(
                &source,
                &mut rows,
                &mut tokens,
                previous,
                source.text.len(),
                request.keep_empty,
            )?;
            Report::Split {
                profile: PROFILE,
                tokens,
                work_bound,
            }
        }
        Action::Exists => unreachable!("existence handled without listing"),
    })
}

fn split_record<'a>(
    source: &Source<'a>,
    rows: &mut Records,
    tokens: &mut Vec<Record<'a>>,
    start: usize,
    end: usize,
    keep_empty: Option<bool>,
) -> Result<(), OperationError> {
    if start != end || keep_empty == Some(true) {
        rows.push(tokens, source.record(Span::from(start..end)), 0)?;
    }
    Ok(())
}

fn execute(request: &Request) -> Result<Report<'_>, OperationError> {
    check_input(&request.text, OP)?;
    if request.max_results > MAX_INDICES
        || request.keep_empty.is_some() && !matches!(request.action, Action::Split)
    {
        return Err(parameter(
            "max_results must be at most 65536 and keep_empty applies to split only",
        ));
    }
    let vm = compile(&request.pattern)?;
    let exists = matches!(request.action, Action::Exists);
    let calls = if exists { 1 } else { request.max_results + 1 };
    let work_bound = search_bound(&vm, request.text.len(), calls)?;
    let report = if exists {
        Report::Exists {
            profile: PROFILE,
            exists: vm.is_match(&mut vm.create_cache(), &request.text),
            work_bound,
        }
    } else {
        list(&vm, request, work_bound)?
    };
    check_report(&report, MAX_OUTPUT)?;
    Ok(report)
}

/// Match or split exact UTF-8 under a bounded, case-sensitive Rust regex profile.
pub fn search(request: &Request) -> Result<Report<'_>, OperationError> {
    execute(request).map_err(|error| error.operation(OP))
}

#[derive(Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Tokenizer {
    UnicodeWhitespace,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TokenizeRequest {
    pub text: String,
    pub profile: Tokenizer,
    #[serde(default = "default_results")]
    #[schemars(range(max = 65536))]
    pub max_results: usize,
}

#[derive(Serialize, JsonSchema)]
pub struct TokenizeReport<'a> {
    pub profile: Tokenizer,
    pub unicode_version: (u8, u8, u8),
    pub tokens: Vec<Record<'a>>,
}

/// Split on the existing Unicode White_Space profile with original spans.
pub fn tokenize(request: &TokenizeRequest) -> Result<TokenizeReport<'_>, OperationError> {
    check_input(&request.text, TOKEN_OP)?;
    if request.max_results > MAX_INDICES {
        return Err(parameter("max_results must be at most 65536").operation(TOKEN_OP));
    }
    let index = word::Index::new(&request.text, TOKEN_OP)?;
    if index.words.len() > request.max_results {
        return Err(resource("tokenization exceeds max_results").operation(TOKEN_OP));
    }
    let tokens = index
        .words
        .iter()
        .map(|word| {
            let byte_end = word.byte_offset + word.byte_length;
            Record {
                text: &request.text[word.byte_offset..byte_end],
                span: SourceSpan {
                    byte_start: word.byte_offset,
                    byte_end,
                    scalar_start: word.scalar_index,
                    scalar_end: word.scalar_index + word.scalar_length,
                },
            }
        })
        .collect();
    let result = TokenizeReport {
        profile: request.profile,
        unicode_version: selection::WORD_UNICODE_VERSION,
        tokens,
    };
    check_report(&result, MAX_OUTPUT).map_err(|error| error.operation(TOKEN_OP))?;
    Ok(result)
}
