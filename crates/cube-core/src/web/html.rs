use super::*;
use crate::operations::Descriptor;
use html5gum::{DefaultEmitter, Token, Tokenizer};
use serde::Serialize;
const OP: &str = "web.html.inspect";
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Reference {
    pub index: usize,
    pub tag: String,
    pub attribute: String,
    pub value: String,
    pub kind: &'static str,
    pub tag_offset: usize,
    pub attribute_span: [usize; 2],
    pub observations: Vec<&'static str>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Embedded {
    pub index: usize,
    pub kind: &'static str,
    pub source_range: [usize; 2],
    pub terminated: bool,
    pub representation: &'static str,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Report {
    pub format: &'static str,
    pub input_bytes: usize,
    pub parse_errors: usize,
    pub tokens: usize,
    pub event_handler_attributes: usize,
    pub references: Vec<Reference>,
    pub embedded: Vec<Embedded>,
    pub text: String,
    pub text_semantics: &'static str,
    pub scope: &'static str,
}
fn text(bytes: &[u8]) -> Result<String, OperationError> {
    if bytes.len() > MAX_FIELD {
        return Err(limit(OP, "HTML token/attribute exceeds 64 KiB"));
    }
    String::from_utf8(bytes.to_vec()).map_err(|_| invalid(OP, "HTML token is not valid UTF-8"))
}
fn void(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}
fn is_reference(tag: &str, attribute: &str) -> bool {
    matches!(
        (tag, attribute),
        ("a" | "link" | "base", "href")
            | (
                "img"
                    | "script"
                    | "iframe"
                    | "audio"
                    | "video"
                    | "source"
                    | "track"
                    | "embed"
                    | "input",
                "src"
            )
            | ("video", "poster")
            | ("object", "data")
            | ("form", "action")
    )
}
/// Tokenize bounded UTF-8 HTML using maintained HTML5Gum with raw-text state
/// switching. This is not a DOM, renderer or browser-grade repair algorithm.
/// Report a documented reference subset and tokenized text excluding script/style;
/// inline script/style descriptors retain exact source ranges without their body.
/// Duplicate attributes use the tokenizer's first-value rule and count as errors.
/// <=1 MiB input, 10000 tokens/attributes, 64 attrs/tag, 128 syntactic depth,
/// 64 recoverable parse errors, 256 refs/inline records, 64 KiB fields/text and
/// 128 KiB aggregate exported text. References/code are inert, never accessed/run.
pub fn inspect(document: &str) -> Result<Report, OperationError> {
    if document.len() > MAX_HTML_BYTES {
        return Err(limit(OP, "HTML input exceeds 1 MiB"));
    }
    let mut emitter = DefaultEmitter::<usize>::new_with_span();
    emitter.naively_switch_states(true);
    let tokenizer = Tokenizer::new_with_emitter(document, emitter);
    let mut report = Report {
        format: "html",
        input_bytes: document.len(),
        parse_errors: 0,
        tokens: 0,
        event_handler_attributes: 0,
        references: Vec::new(),
        embedded: Vec::new(),
        text: String::new(),
        text_semantics: "HTML5 tokenized text excluding script/style; not rendered visibility; references character-decoded",
        scope: "offline_tokenizer_observations_no_network_execution_or_path_resolution",
    };
    let mut stack: Vec<String> = Vec::new();
    let mut inline: Option<(&'static str, usize)> = None;
    let mut attributes = 0;
    let mut output_bytes = 0;
    for token in tokenizer {
        let token = match token {
            Ok(token) => token,
            Err(never) => match never {},
        };
        report.tokens += 1;
        if report.tokens > 10000 {
            return Err(limit(OP, "HTML exceeds 10000 tokens"));
        }
        match token {
            Token::StartTag(tag) => {
                let name = text(&tag.name)?;
                if name.len() > 128 || tag.attributes.len() > 64 {
                    return Err(limit(OP, "HTML tag name/attribute-count limit exceeded"));
                }
                attributes += tag.attributes.len();
                if attributes > 10000 {
                    return Err(limit(OP, "HTML exceeds 10000 attributes"));
                }
                for (key, value) in tag.attributes {
                    let key = text(&key)?;
                    let value_text = text(&value)?;
                    if key.starts_with("on") {
                        report.event_handler_attributes += 1;
                    }
                    if is_reference(&name, &key) {
                        if report.references.len() == MAX_RECORDS {
                            return Err(limit(OP, "HTML exceeds 256 references"));
                        }
                        output_bytes += key.len() + name.len() + value_text.len();
                        report.references.push(Reference {
                            index: report.references.len(),
                            tag: name.clone(),
                            attribute: key,
                            value: value_text.clone(),
                            kind: reference_kind(&value_text),
                            tag_offset: tag.span.start,
                            attribute_span: [value.span.start, value.span.end],
                            observations: observations(&value_text),
                        });
                    }
                }
                if name == "script" || name == "style" {
                    inline = Some((
                        if name == "script" { "script" } else { "style" },
                        tag.span.end,
                    ));
                }
                if !void(&name) && !tag.self_closing {
                    if stack.len() == 128 {
                        return Err(limit(OP, "HTML syntactic nesting exceeds 128"));
                    }
                    stack.push(name);
                }
            }
            Token::EndTag(tag) => {
                let name = text(&tag.name)?;
                if let Some((kind, start)) = inline.filter(|(kind, _)| *kind == name) {
                    if report.embedded.len() == MAX_RECORDS {
                        return Err(limit(OP, "HTML exceeds 256 inline records"));
                    }
                    if tag.span.start.saturating_sub(start) > MAX_FIELD {
                        return Err(limit(OP, "HTML inline body exceeds 64 KiB"));
                    }
                    if tag.span.start < start || document.get(start..tag.span.start).is_none() {
                        return Err(invalid(OP, "invalid tokenizer source range"));
                    }
                    report.embedded.push(Embedded {
                        index: report.embedded.len(),
                        kind,
                        source_range: [start, tag.span.start],
                        terminated: true,
                        representation: "exact_source_utf8",
                    });
                    inline = None;
                }
                if let Some(index) = stack.iter().rposition(|n| *n == name) {
                    stack.truncate(index);
                }
            }
            Token::String(value) => {
                if value.len() > MAX_FIELD {
                    return Err(limit(OP, "HTML token exceeds 64 KiB"));
                }
                if inline.is_none() {
                    let value = text(&value)?;
                    if report.text.len() + value.len() > MAX_FIELD {
                        return Err(limit(OP, "HTML exported text exceeds 64 KiB"));
                    }
                    output_bytes += value.len();
                    report.text.push_str(&value);
                }
            }
            Token::Error(_) => {
                report.parse_errors += 1;
                if report.parse_errors > 64 {
                    return Err(limit(OP, "HTML exceeds 64 recoverable tokenizer errors"));
                }
            }
            Token::Comment(value) => {
                if value.len() > MAX_FIELD {
                    return Err(limit(OP, "HTML comment exceeds 64 KiB"));
                }
            }
            Token::Doctype(_) => {}
        }
        if output_bytes > MAX_OUTPUT_TEXT {
            return Err(limit(OP, "HTML exported text exceeds 128 KiB"));
        }
    }
    if let Some((kind, start)) = inline {
        if document.len().saturating_sub(start) > MAX_FIELD {
            return Err(limit(OP, "HTML inline body exceeds 64 KiB"));
        }
        if report.embedded.len() == MAX_RECORDS {
            return Err(limit(OP, "HTML exceeds 256 inline records"));
        }
        report.embedded.push(Embedded {
            index: report.embedded.len(),
            kind,
            source_range: [start, document.len()],
            terminated: false,
            representation: "exact_source_utf8",
        });
    }
    Ok(report)
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: OP,
    category: "web",
    description: "Bounded inert UTF-8 HTML token/reference/text/inline-range observations; no DOM/rendering/JS execution/network/path resolution; parse errors and limits explicit.",
    input_type: "text",
    output_type: "web.html_report",
    parameters: &[],
    deterministic: true,
    reversible: Some(false),
    command: "web html",
    example: "cube web html saved.html --json",
}];
