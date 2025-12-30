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

pub fn is_mutation(expr: &str) -> bool {
    let normalized = expr.replace(" ", "");

    if normalized.contains("|=")
        || normalized.contains("+=")
        || normalized.contains("-=")
        || normalized.contains("*=")
        || normalized.contains("/=")
        || normalized.contains("%=")
        || normalized.contains("//=")
    {
        return true;
    }

    let without_eq_eq = expr.replace("==", "");
    if without_eq_eq.contains('=') {
        return true;
    }

    if normalized.contains("del(") || normalized.contains("delpaths(") {
        return true;
    }

    if normalized.contains("setpath(") {
        return true;
    }

    false
}

pub fn yaml_to_json(yaml: &str) -> Result<Value, QueryError> {
    serde_yml::from_str(yaml).map_err(|e| QueryError(format!("invalid yaml: {e}")))
}

pub fn json_to_yaml(value: &Value) -> Result<String, QueryError> {
    serde_yml::to_string(value).map_err(|e| QueryError(format!("yaml serialization failed: {e}")))
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
