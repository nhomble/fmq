mod frontmatter;
mod query;

pub use frontmatter::{extract, extract_reader, reassemble, Document};
pub use query::{is_mutation, run};

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

pub fn fmq(expr: &str, markdown: &str) -> Result<String, Error> {
    let doc = extract(markdown)?;
    let result = run(expr, &doc.frontmatter)?;

    if is_mutation(expr) {
        let yaml = query::json_to_yaml(&result)?;
        Ok(reassemble(&yaml, &doc.body))
    } else {
        Ok(format_output(&result))
    }
}

pub fn fmq_reader<R: BufRead>(expr: &str, reader: R) -> Result<String, Error> {
    let need_body = is_mutation(expr);
    let doc = extract_reader(reader, need_body)?;
    let result = run(expr, &doc.frontmatter)?;

    if need_body {
        let yaml = query::json_to_yaml(&result)?;
        Ok(reassemble(&yaml, &doc.body))
    } else {
        Ok(format_output(&result))
    }
}

fn format_output(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        _ => serde_json::to_string_pretty(value).unwrap_or_default(),
    }
}
