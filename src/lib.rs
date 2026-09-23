mod frontmatter;
mod query;

pub use frontmatter::{extract, extract_reader, reassemble, Document};
pub use query::{is_mutation, run, run_all};

use std::io::BufRead;

use std::fmt;

#[derive(Debug)]
pub enum Error {
    Parse(String),
    Query(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse(msg) => write!(f, "{}", msg),
            Error::Query(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<frontmatter::ParseError> for Error {
    fn from(e: frontmatter::ParseError) -> Self {
        Error::Parse(e.0)
    }
}

impl From<query::QueryError> for Error {
    fn from(e: query::QueryError) -> Self {
        Error::Query(e.0)
    }
}

pub fn fmq(expr: &str, markdown: &str, init: bool) -> Result<String, Error> {
    let doc = extract(markdown, init)?;

    let frontmatter: &str = if doc.frontmatter.is_empty() {
        "{}"
    } else {
        &doc.frontmatter
    };

    if is_mutation(expr) {
        let result = run(expr, frontmatter)?;
        to_document(&result, &doc.body)
    } else {
        let results = run_all(expr, frontmatter)?;
        Ok(format_outputs(&results))
    }
}

pub fn fmq_reader<R: BufRead>(expr: &str, reader: R, init: bool) -> Result<String, Error> {
    let need_body = is_mutation(expr);
    let doc = extract_reader(reader, need_body, init)?;

    let frontmatter: &str = if doc.frontmatter.is_empty() {
        "{}"
    } else {
        &doc.frontmatter
    };

    if need_body {
        let result = run(expr, frontmatter)?;
        to_document(&result, &doc.body)
    } else {
        let results = run_all(expr, frontmatter)?;
        Ok(format_outputs(&results))
    }
}

/// Always treats the result as the new frontmatter (used by --in-place).
/// Errors (without producing output) if the result is not an object.
pub fn fmq_document(expr: &str, markdown: &str, init: bool) -> Result<String, Error> {
    let doc = extract(markdown, init)?;
    let frontmatter: &str = if doc.frontmatter.is_empty() {
        "{}"
    } else {
        &doc.frontmatter
    };
    let result = run(expr, frontmatter)?;
    to_document(&result, &doc.body)
}

fn to_document(result: &serde_json::Value, body: &str) -> Result<String, Error> {
    if !result.is_object() {
        return Err(Error::Query(format!(
            "expression result must be an object to write as frontmatter, got {}",
            match result {
                serde_json::Value::Null => "null",
                serde_json::Value::Bool(_) => "boolean",
                serde_json::Value::Number(_) => "number",
                serde_json::Value::String(_) => "string",
                serde_json::Value::Array(_) => "array",
                _ => "object",
            }
        )));
    }
    let yaml = query::json_to_yaml(result)?;
    Ok(reassemble(&yaml, body))
}

fn format_output(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        _ => serde_json::to_string_pretty(value).unwrap_or_default(),
    }
}

fn format_outputs(values: &[serde_json::Value]) -> String {
    values
        .iter()
        .map(|v| format!("{}\n", format_output(v)))
        .collect()
}
