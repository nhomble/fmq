use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ParseError {}

pub struct Document {
    pub frontmatter: String,
    pub body: String,
}

pub fn extract(markdown: &str) -> Result<Document, ParseError> {
    let trimmed = markdown.trim_start();

    if !trimmed.starts_with("---") {
        return Err(ParseError("no frontmatter found".into()));
    }

    let after_first = &trimmed[3..];
    let after_newline = after_first
        .strip_prefix('\n')
        .or_else(|| after_first.strip_prefix("\r\n"))
        .ok_or_else(|| ParseError("invalid frontmatter delimiter".into()))?;

    let end = after_newline
        .find("\n---")
        .ok_or_else(|| ParseError("unclosed frontmatter".into()))?;

    let frontmatter = after_newline[..end].to_string();

    let rest = &after_newline[end + 4..];
    let body = rest
        .strip_prefix('\n')
        .or_else(|| rest.strip_prefix("\r\n"))
        .unwrap_or(rest)
        .to_string();

    Ok(Document { frontmatter, body })
}

pub fn reassemble(frontmatter: &str, body: &str) -> String {
    let mut result = String::new();
    result.push_str("---\n");
    result.push_str(frontmatter);
    if !frontmatter.ends_with('\n') {
        result.push('\n');
    }
    result.push_str("---\n");
    result.push_str(body);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_simple() {
        let md = "---\ntitle: Hello\n---\nBody text";
        let doc = extract(md).unwrap();
        assert_eq!(doc.frontmatter, "title: Hello");
        assert_eq!(doc.body, "Body text");
    }

    #[test]
    fn extract_no_frontmatter() {
        let md = "Just body text";
        assert!(extract(md).is_err());
    }

    #[test]
    fn reassemble_simple() {
        let result = reassemble("title: Hello", "Body text");
        assert_eq!(result, "---\ntitle: Hello\n---\nBody text");
    }

    #[test]
    fn roundtrip() {
        let md = "---\ntitle: Hello\ntags:\n  - rust\n---\nBody text\n";
        let doc = extract(md).unwrap();
        let reassembled = reassemble(&doc.frontmatter, &doc.body);
        assert_eq!(reassembled, md);
    }
}
