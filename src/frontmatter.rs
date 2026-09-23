use std::error::Error;
use std::fmt;
use std::io::BufRead;

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

pub fn extract(markdown: &str, allow_empty: bool) -> Result<Document, ParseError> {
    extract_reader(markdown.as_bytes(), true, allow_empty)
}

pub fn extract_reader<R: BufRead>(
    mut reader: R,
    need_body: bool,
    allow_empty: bool,
) -> Result<Document, ParseError> {
    let mut first_line = String::new();
    reader
        .read_line(&mut first_line)
        .map_err(|e| ParseError(e.to_string()))?;

    if first_line.trim() != "---" {
        if allow_empty {
            let mut body = first_line;
            reader
                .read_to_string(&mut body)
                .map_err(|e| ParseError(e.to_string()))?;
            return Ok(Document {
                frontmatter: String::new(),
                body,
            });
        }
        return Err(ParseError("no frontmatter found".into()));
    }

    let mut frontmatter = String::new();
    loop {
        let mut line = String::new();
        let bytes = reader
            .read_line(&mut line)
            .map_err(|e| ParseError(e.to_string()))?;

        if bytes == 0 {
            return Err(ParseError("unclosed frontmatter".into()));
        }

        if line.trim() == "---" {
            break;
        }

        frontmatter.push_str(&line);
    }

    // Trim trailing newline from frontmatter
    if frontmatter.ends_with('\n') {
        frontmatter.pop();
        if frontmatter.ends_with('\r') {
            frontmatter.pop();
        }
    }

    let body = if need_body {
        let mut body = String::new();
        reader
            .read_to_string(&mut body)
            .map_err(|e| ParseError(e.to_string()))?;
        body
    } else {
        String::new()
    };

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
        let doc = extract(md, false).unwrap();
        assert_eq!(doc.frontmatter, "title: Hello");
        assert_eq!(doc.body, "Body text");
    }

    #[test]
    fn extract_no_frontmatter() {
        let md = "Just body text";
        assert!(extract(md, false).is_err());
    }

    #[test]
    fn extract_no_frontmatter_with_init() {
        let md = "Just body text";
        let doc = extract(md, true).unwrap();
        assert_eq!(doc.frontmatter, "");
        assert_eq!(doc.body, "Just body text");
    }

    #[test]
    fn reassemble_simple() {
        let result = reassemble("title: Hello", "Body text");
        assert_eq!(result, "---\ntitle: Hello\n---\nBody text");
    }

    #[test]
    fn extract_empty_frontmatter() {
        // B5
        let doc = extract("---\n---\nBody\n", false).unwrap();
        assert_eq!(doc.frontmatter, "");
        assert_eq!(doc.body, "Body\n");
    }

    #[test]
    fn extract_dash_suffix_is_not_delimiter() {
        // B6
        let doc = extract("---\na: 1\n---x\n---\nBody\n", false).unwrap();
        assert_eq!(doc.frontmatter, "a: 1\n---x");
        assert_eq!(doc.body, "Body\n");
    }

    #[test]
    fn extract_dash_suffix_only_is_unclosed() {
        // B6
        let err = extract("---\na: 1\n---x\nBody\n", false).err().unwrap();
        assert_eq!(err.0, "unclosed frontmatter");
    }

    #[test]
    fn extract_opening_delimiter_trailing_whitespace() {
        // B6
        let doc = extract("---   \na: 1\n---\nBody\n", false).unwrap();
        assert_eq!(doc.frontmatter, "a: 1");
        assert_eq!(doc.body, "Body\n");
    }

    #[test]
    fn extract_leading_blank_line_is_not_frontmatter() {
        let md = "\n---\na: 1\n---\nBody\n";
        assert_eq!(extract(md, false).err().unwrap().0, "no frontmatter found");
        let doc = extract(md, true).unwrap();
        assert_eq!(doc.frontmatter, "");
        assert_eq!(doc.body, md);
    }

    #[test]
    fn roundtrip() {
        let md = "---\ntitle: Hello\ntags:\n  - rust\n---\nBody text\n";
        let doc = extract(md, false).unwrap();
        let reassembled = reassemble(&doc.frontmatter, &doc.body);
        assert_eq!(reassembled, md);
    }
}
