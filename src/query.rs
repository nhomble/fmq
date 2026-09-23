use jaq_interpret::{Ctx, FilterT, ParseCtx, RcIter, Val};
use serde_json::Value;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct QueryError(pub String);

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for QueryError {}

use jaq_syn::filter::{BinaryOp, Filter};
use jaq_syn::MathOp;

/// True if the expression's final pipe stage is an update, i.e. its output
/// is intended to be the whole (modified) frontmatter object.
pub fn is_mutation(expr: &str) -> bool {
    let (main, errs) = jaq_parse::parse(expr, jaq_parse::main());
    match main {
        Some(m) if errs.is_empty() => is_update(&m.body.0),
        _ => false, // run() reports the parse error
    }
}

fn is_update(f: &Filter) -> bool {
    match f {
        Filter::Binary(_, BinaryOp::Pipe(_), r) => is_update(&r.0),
        Filter::Binary(_, BinaryOp::Assign(_), _) => true,
        Filter::Binary(l, BinaryOp::Math(MathOp::Add | MathOp::Mul), r) => matches!(
            (&l.0, &r.0),
            (Filter::Id, Filter::Object(_)) | (Filter::Object(_), Filter::Id)
        ),
        Filter::Call(name, _) => matches!(
            name.as_str(),
            "del" | "delpaths" | "setpath" | "with_entries" | "map_values" | "from_entries"
        ),
        _ => false,
    }
}

pub fn yaml_to_json(yaml: &str) -> Result<Value, QueryError> {
    serde_yaml::from_str(yaml).map_err(|e| QueryError(format!("invalid yaml: {e}")))
}

pub fn json_to_yaml(value: &Value) -> Result<String, QueryError> {
    serde_yaml::to_string(value).map_err(|e| QueryError(format!("yaml serialization failed: {e}")))
}

fn caret(expr: &str, start: usize) -> String {
    format!("  {expr}\n  {}^", " ".repeat(start))
}

fn format_parse_error(expr: &str, e: &jaq_parse::Error) -> String {
    let start = e.span().start;
    let found = match e.found() {
        Some(tok) => format!("unexpected {tok:?}"),
        None => "unexpected end of input".to_string(),
    };
    let mut expected: Vec<String> = e
        .expected()
        .map(|t| match t {
            Some(tok) => format!("{tok:?}"),
            None => "end of input".to_string(),
        })
        .collect();
    expected.sort();
    expected.dedup();
    let exp = match expected.len() {
        0 => String::new(),
        1 => format!(", expected {}", expected[0]),
        _ => format!(", expected one of {}", expected.join(", ")),
    };
    format!(
        "parse error at column {}: {found}{exp}\n{}",
        start + 1,
        caret(expr, start)
    )
}

fn format_compile_error(
    expr: &str,
    e: &impl fmt::Display,
    span: &std::ops::Range<usize>,
) -> String {
    let text: String = expr
        .chars()
        .skip(span.start)
        .take(span.end - span.start)
        .collect();
    let name = text.split('(').next().unwrap_or(&text).trim();
    format!(
        "compile error at column {}: {e} `{name}`\n{}",
        span.start + 1,
        caret(expr, span.start)
    )
}

fn compile(expr: &str) -> Result<jaq_interpret::Filter, QueryError> {
    let mut ctx = ParseCtx::new(Vec::new());
    ctx.insert_natives(jaq_core::core());
    ctx.insert_defs(jaq_std::std());

    let (filter, errs) = jaq_parse::parse(expr, jaq_parse::main());
    if !errs.is_empty() {
        let msgs: Vec<_> = errs.iter().map(|e| format_parse_error(expr, e)).collect();
        return Err(QueryError(msgs.join("\n")));
    }

    let filter =
        ctx.compile(filter.ok_or_else(|| QueryError("parse error: no filter parsed".into()))?);

    if !ctx.errs.is_empty() {
        let msgs: Vec<_> = ctx
            .errs
            .iter()
            .map(|(e, span)| format_compile_error(expr, e, span))
            .collect();
        return Err(QueryError(msgs.join("\n")));
    }

    Ok(filter)
}

/// All outputs of `expr` (query path). Errors on the first runtime error.
pub fn run_all(expr: &str, yaml: &str) -> Result<Vec<Value>, QueryError> {
    let json = yaml_to_json(yaml)?;
    let filter = compile(expr)?;

    let inputs = RcIter::new(core::iter::empty());
    filter
        .run((Ctx::new([], &inputs), Val::from(json)))
        .map(|r| {
            r.map(Value::from)
                .map_err(|e| QueryError(format!("runtime error: {e}")))
        })
        .collect()
}

/// Exactly one output (mutation / write path).
pub fn run(expr: &str, yaml: &str) -> Result<Value, QueryError> {
    let mut outs = run_all(expr, yaml)?;
    match outs.len() {
        1 => Ok(outs.pop().unwrap()),
        0 => Err(QueryError(
            "expression produced no output; nothing to write".into(),
        )),
        n => Err(QueryError(format!(
            "expression produced {n} outputs; writing frontmatter requires exactly one"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_query() {
        assert!(!is_mutation(".title"));
        assert!(!is_mutation(".foo.bar"));
        assert!(!is_mutation(".x == .y"));
    }

    #[test]
    fn detect_mutation() {
        assert!(is_mutation(".title = \"new\""));
        assert!(is_mutation(".count += 1"));
        assert!(is_mutation(".tags |= . + [\"new\"]"));
        assert!(is_mutation("del(.draft)"));
    }

    #[test]
    fn detect_query_false_positives_fixed() {
        assert!(!is_mutation(".title != \"x\""));
        assert!(!is_mutation(".x >= 1"));
        assert!(!is_mutation(".x <= 1"));
        assert!(!is_mutation(".title == \"a=b\""));
        assert!(!is_mutation("test(\"a=b\")"));
        assert!(!is_mutation(".title = \"x\" | .title"));
        assert!(!is_mutation(".author"));
        assert!(!is_mutation("{a: .title}"));
    }

    #[test]
    fn detect_mutation_false_negatives_fixed() {
        assert!(is_mutation(". + {x: 1}"));
        assert!(is_mutation(". * {x: 1}"));
        assert!(is_mutation("with_entries(.)"));
        assert!(is_mutation("map_values(.)"));
        assert!(is_mutation("to_entries | from_entries"));
        assert!(is_mutation(".a = 1 | .b = 2"));
        assert!(is_mutation("def f: .; .a = 1"));
    }

    #[test]
    fn yaml_json_roundtrip() {
        let yaml = "title: Hello\ntags:\n  - rust\n  - cli\n";
        let json = yaml_to_json(yaml).unwrap();
        assert_eq!(json["title"], "Hello");
        assert_eq!(json["tags"][0], "rust");
    }

    #[test]
    fn run_query() {
        let yaml = "title: Hello\n";
        let result = run(".title", yaml).unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn run_mutation() {
        let yaml = "title: Hello\n";
        let result = run(".title = \"World\"", yaml).unwrap();
        assert_eq!(result["title"], "World");
    }

    #[test]
    fn run_all_multiple_outputs() {
        let yaml = "tags: [a, b, c]\n";
        let result = run_all(".tags[]", yaml).unwrap();
        assert_eq!(
            result,
            vec![
                Value::String("a".into()),
                Value::String("b".into()),
                Value::String("c".into()),
            ]
        );
    }

    #[test]
    fn run_all_empty_output() {
        let yaml = "a: 1\n";
        let result = run_all("empty", yaml).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn run_no_output_is_error() {
        let yaml = "a: 1\n";
        let err = run("empty", yaml).unwrap_err();
        assert!(err.0.contains("no output"), "{}", err.0);
    }

    #[test]
    fn run_multiple_outputs_is_error() {
        let yaml = "a: 1\n";
        let err = run(".a = (1, 2)", yaml).unwrap_err();
        assert!(err.0.contains("2 outputs"), "{}", err.0);
    }
}
