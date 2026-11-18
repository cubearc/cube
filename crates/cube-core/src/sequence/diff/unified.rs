//! Unified hunks over the exact shared edit script.
use super::*;
use std::fmt::Write as _;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UnifiedOptions {
    #[serde(default = "default_context")]
    #[schemars(range(max = 1000))]
    pub context: usize,
    #[serde(default = "left_label")]
    #[schemars(length(max = 1024))]
    pub left_label: String,
    #[serde(default = "right_label")]
    #[schemars(length(max = 1024))]
    pub right_label: String,
}
const fn default_context() -> usize {
    3
}
fn left_label() -> String {
    "a/input".into()
}
fn right_label() -> String {
    "b/input".into()
}

fn label(text: &str) -> bool {
    !text.is_empty() && text.len() <= 1024 && !text.chars().any(char::is_control)
}

fn write_range(out: &mut String, before: usize, count: usize) {
    let start = before + usize::from(count != 0);
    write!(out, "{start}").expect("string writer");
    if count != 1 {
        write!(out, ",{count}").expect("string writer");
    }
}

fn hunks(steps: &[EditKind], context: usize) -> Vec<std::ops::Range<usize>> {
    let mut result: Vec<std::ops::Range<usize>> = Vec::new();
    for (i, &step) in steps.iter().enumerate() {
        if step == EditKind::Equal {
            continue;
        }
        let start = i.saturating_sub(context);
        let end = (i + context + 1).min(steps.len());
        if let Some(last) = result.last_mut().filter(|h| h.end >= start) {
            last.end = end;
        } else {
            result.push(start..end);
        }
    }
    result
}

pub(super) fn render(
    steps: &[EditKind],
    left: &Lines,
    right: &Lines,
    options: &UnifiedOptions,
) -> Result<String, OperationError> {
    if options.context > 1000 || !label(&options.left_label) || !label(&options.right_label) {
        return Err(parameter(
            "unified diff needs bounded printable labels and context in 0..=1000",
        ));
    }
    let spans = hunks(steps, options.context);
    if spans.is_empty() {
        return Ok(String::new());
    }
    let mut before = vec![(0, 0); steps.len() + 1];
    for (i, &kind) in steps.iter().enumerate() {
        before[i + 1] = (
            before[i].0 + usize::from(kind != EditKind::Insert),
            before[i].1 + usize::from(kind != EditKind::Delete),
        );
    }
    let mut output = format!("--- {}\n+++ {}\n", options.left_label, options.right_label);
    for range in spans {
        let (a, b) = before[range.start];
        let (end_a, end_b) = before[range.end];
        output.push_str("@@ -");
        write_range(&mut output, a, end_a - a);
        output.push_str(" +");
        write_range(&mut output, b, end_b - b);
        output.push_str(" @@\n");
        for i in range {
            let (a, b) = before[i];
            let (prefix, line) = match steps[i] {
                EditKind::Equal => (' ', left.get(a)),
                EditKind::Delete => ('-', left.get(a)),
                _ => ('+', right.get(b)),
            };
            let line = std::str::from_utf8(line)
                .map_err(|_| invalid("unified rendering requires UTF-8 in shown lines"))?;
            if output.len() + line.len() + 32 > MAX_REPORT {
                return Err(resource("unified diff exceeds 8 MiB"));
            }
            output.push(prefix);
            output.push_str(line);
            if !line.ends_with('\n') {
                output.push_str("\n\\ No newline at end of file\n");
            }
        }
    }
    Ok(output)
}
