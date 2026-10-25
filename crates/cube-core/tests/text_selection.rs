use cube_core::{
    errors::ErrorCode,
    pipeline::{Kind, Limits, Parameter, Plan, Step, Value},
    text::{
        self,
        selection::{Selector, select},
    },
};
#[test]
fn words_lines_punctuation_and_unicode_scalars_have_explicit_semantics() {
    let input = "\tAlpha,\u{2003}βeta!\r\n\n🧊cube \n";
    for (mode, expected) in [
        ("word_first", "Aβ🧊"),
        ("word_last", ",!e"),
        ("line_first", "\t🧊"),
        ("line_last", "! "),
    ] {
        assert_eq!(
            select(input, Selector::parse(mode).unwrap()).unwrap(),
            expected
        );
    }
    assert_eq!(
        select("Cube Uses Binary Evidence", Selector::WordFirst).unwrap(),
        "CUBE"
    );
    assert_eq!(
        select("e\u{301} 🧊雪", Selector::WordLast).unwrap(),
        "\u{301}雪"
    );
    assert_eq!(select("A\rB", Selector::LineLast).unwrap(), "B");
    for input in ["", "\n\r\n"] {
        assert_eq!(select(input, Selector::LineFirst).unwrap(), "");
    }
    assert_eq!(select("  \t", Selector::WordFirst).unwrap(), "");
    assert_eq!(select("  ", Selector::LineFirst).unwrap(), " ");
    assert_eq!(
        text::extract("aé🧊z", Some(&[2, 1, 2]), None, None, None).unwrap(),
        "🧊é🧊"
    );
    assert_eq!(
        text::extract("abcdef", None, Some(1), Some(2), None).unwrap(),
        "bdf"
    );
}
#[test]
fn input_bound_and_selector_validation_hold() {
    assert!(Selector::parse("grapheme").is_err());
    assert_eq!(
        select(&"a".repeat(text::MAX_INPUT), Selector::WordFirst).unwrap(),
        "a"
    );
    assert_eq!(
        select(&"a".repeat(text::MAX_INPUT + 1), Selector::WordFirst)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
#[test]
fn pipeline_selection_validates_parameters_before_execution() {
    let mut step = Step::new("text.select");
    step.parameters
        .insert("selector".into(), Parameter::Text("word_first".into()));
    let plan = Plan::validate(Kind::Text, vec![step.clone()], Limits::default()).unwrap();
    assert_eq!(
        plan.execute(
            Value::Text {
                data: "Cube Uses Binary Evidence".into()
            },
            &[],
            None
        )
        .unwrap()
        .output,
        Value::Text {
            data: "CUBE".into()
        }
    );
    step.parameters
        .insert("selector".into(), Parameter::Text("guess".into()));
    assert!(Plan::validate(Kind::Text, vec![step], Limits::default()).is_err());
    assert!(
        Plan::validate(
            Kind::Text,
            vec![Step::new("text.select")],
            Limits::default()
        )
        .is_err()
    );
}
