//! ADF (Atlassian Document Format) → plain text extraction.
//!
//! JIRA `fields.description` is an ADF JSON document (or `null`). This module
//! walks the content tree and produces plain text — no Markdown rendering,
//! just extracted human-readable text. Suitable for storing as [`Issue::body`].
//!
//! [`Issue::body`]: reposix_core::Issue

/// Extract plain text from an ADF JSON document.
///
/// Returns an empty string for `null` input (JIRA descriptions may be null).
/// Unknown node types recurse into their `content[]` children and emit
/// their text — forward-compat: new ADF node types produce their text children
/// without requiring a code change in this module.
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use reposix_jira::adf::adf_to_plain_text;
///
/// let doc = json!({
///     "type": "doc", "version": 1,
///     "content": [{"type": "paragraph", "content": [
///         {"type": "text", "text": "Hello world"}
///     ]}]
/// });
/// assert_eq!(adf_to_plain_text(&doc), "Hello world");
/// ```
#[must_use]
pub fn adf_to_plain_text(value: &serde_json::Value) -> String {
    // null description → empty string
    if value.is_null() {
        return String::new();
    }
    let mut out = String::new();
    extract_text(value, &mut out);
    out.trim_end_matches('\n').to_string()
}

fn extract_text(node: &serde_json::Value, out: &mut String) {
    let node_type = node.get("type").and_then(|v| v.as_str()).unwrap_or("");
    match node_type {
        "text" => {
            if let Some(text) = node.get("text").and_then(|v| v.as_str()) {
                out.push_str(text);
            }
        }
        "hardBreak" => out.push('\n'),
        "paragraph" | "doc" => {
            if let Some(children) = node.get("content").and_then(|v| v.as_array()) {
                for child in children {
                    extract_text(child, out);
                }
            }
            out.push('\n');
        }
        "codeBlock" => {
            if let Some(children) = node.get("content").and_then(|v| v.as_array()) {
                for child in children {
                    extract_text(child, out);
                }
            }
            out.push('\n');
        }
        _ => {
            // Unknown node type — recurse into content[] children and emit their text.
            // This handles future ADF node types gracefully (forward-compat).
            if let Some(children) = node.get("content").and_then(|v| v.as_array()) {
                for child in children {
                    extract_text(child, out);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn null_returns_empty() {
        assert_eq!(adf_to_plain_text(&serde_json::Value::Null), "");
    }

    #[test]
    fn simple_paragraph() {
        let doc = json!({
            "type": "doc", "version": 1,
            "content": [{"type": "paragraph", "content": [
                {"type": "text", "text": "Hello world"}
            ]}]
        });
        assert_eq!(adf_to_plain_text(&doc), "Hello world");
    }

    #[test]
    fn code_block_extracted() {
        let doc = json!({
            "type": "doc", "version": 1,
            "content": [{"type": "codeBlock", "content": [
                {"type": "text", "text": "fn main() {}"}
            ]}]
        });
        assert_eq!(adf_to_plain_text(&doc), "fn main() {}");
    }

    #[test]
    fn hard_break_becomes_newline() {
        let doc = json!({
            "type": "doc", "version": 1,
            "content": [{"type": "paragraph", "content": [
                {"type": "text", "text": "line1"},
                {"type": "hardBreak"},
                {"type": "text", "text": "line2"}
            ]}]
        });
        let result = adf_to_plain_text(&doc);
        assert!(
            result.contains("line1\nline2"),
            "Expected newline between lines, got: {result:?}"
        );
    }

    #[test]
    fn unknown_node_type_recurses() {
        let doc = json!({
            "type": "doc", "version": 1,
            "content": [{"type": "futureNodeType", "content": [
                {"type": "text", "text": "extracted"}
            ]}]
        });
        assert_eq!(adf_to_plain_text(&doc), "extracted");
    }
}
