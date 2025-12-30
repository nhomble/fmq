mod frontmatter;
mod query;

pub use frontmatter::{extract, reassemble, Document, ParseError};
pub use query::{is_mutation, run, QueryError};

use std::error::Error;

pub fn fmq(expr: &str, markdown: &str) -> Result<String, Box<dyn Error>> {
    let doc = extract(markdown)?;
    let result = run(expr, &doc.frontmatter)?;

    if is_mutation(expr) {
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
