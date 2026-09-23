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

pub fn run(expr: &str, yaml: &str) -> Result<Value, QueryError> {
    let json = yaml_to_json(yaml)?;

    let mut ctx = ParseCtx::new(Vec::new());
    ctx.insert_natives(jaq_core::core());
    ctx.insert_defs(jaq_std::std());

    let (filter, errs) = jaq_parse::parse(expr, jaq_parse::main());
    if !errs.is_empty() {
        return Err(QueryError(format!("parse error: {:?}", errs)));
    }

    let filter = ctx.compile(filter.ok_or_else(|| QueryError("parse failed".into()))?);

    if !ctx.errs.is_empty() {
        return Err(QueryError(format!(
            "compile error: {} errors",
            ctx.errs.len()
        )));
    }

    let inputs = RcIter::new(core::iter::empty());
    let mut out = filter.run((Ctx::new([], &inputs), Val::from(json)));

    match out.next() {
        Some(Ok(val)) => Ok(Value::from(val)),
        Some(Err(e)) => Err(QueryError(format!("runtime error: {e}"))),
        None => Err(QueryError("no output".into())),
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
}
