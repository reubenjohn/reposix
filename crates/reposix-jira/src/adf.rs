//! ADF (Atlassian Document Format) utilities for JIRA Cloud.
//!
//! JIRA `fields.description` is an ADF JSON document (or `null`). This module
//! provides:
//! - [`adf_to_plain_text`]: walks the content tree and produces plain text.
//! - [`adf_to_markdown`]: converts ADF to Markdown (used on the read path).
//! - [`adf_paragraph_wrap`]: wraps plain text in minimal ADF structure for writes.
//!
//! [`Record::body`]: reposix_core::Record

use reposix_core::Error;

/// Maximum recursion depth for [`adf_to_markdown`]'s node visitor.
/// Prevents stack overflow on maliciously deep ADF documents.
const MAX_ADF_DEPTH: usize = 50;

/// Fallback marker prefix for unknown ADF node types.
const FALLBACK_PREFIX: &str = "[unknown-adf:";

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

/// Wrap plain text in a minimal Atlassian Document Format (ADF) structure
/// suitable for JIRA `description` fields in write requests.
///
/// JIRA Cloud REST v3 requires `description` to be a full ADF document
/// (not plain text). The minimal representation is a single paragraph node.
///
/// # Examples
///
/// ```
/// use reposix_jira::adf::adf_paragraph_wrap;
/// let adf = adf_paragraph_wrap("Hello world");
/// assert_eq!(adf["type"], "doc");
/// assert_eq!(adf["content"][0]["type"], "paragraph");
/// ```
#[must_use]
pub fn adf_paragraph_wrap(text: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "doc",
        "version": 1,
        "content": [{
            "type": "paragraph",
            "content": [{
                "type": "text",
                "text": text
            }]
        }]
    })
}

/// Convert an ADF JSON document to Markdown text.
///
/// JIRA Cloud uses the same Atlassian Document Format as Confluence.
/// Unknown node types recurse into their `content[]` children — forward-compat
/// with future ADF node types.
///
/// Recursion is capped at [`MAX_ADF_DEPTH`]; nodes beyond the cap emit a
/// depth-limit fallback marker instead of recursing further.
///
/// # Errors
///
/// Returns `Err(Error::Other("…"))` if the JSON root is not a `type: "doc"`
/// object. All other malformed input is handled with fallback markers.
pub fn adf_to_markdown(adf: &serde_json::Value) -> Result<String, Error> {
    let type_field = adf
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    if type_field != "doc" {
        return Err(Error::Other(format!(
            "adf_to_markdown: root node type must be \"doc\", got \"{type_field}\""
        )));
    }
    let mut out = String::new();
    if let Some(content) = adf.get("content").and_then(serde_json::Value::as_array) {
        for node in content {
            render_node(node, &mut out, 0, false, 0);
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Private helpers for adf_to_markdown
// ---------------------------------------------------------------------------

fn render_node(
    node: &serde_json::Value,
    out: &mut String,
    list_depth: usize,
    ordered: bool,
    depth: usize,
) {
    if depth > MAX_ADF_DEPTH {
        out.push_str("[depth limit exceeded]");
        return;
    }

    let node_type = node
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    match node_type {
        "doc" => {
            if let Some(content) = node.get("content").and_then(serde_json::Value::as_array) {
                for child in content {
                    render_node(child, out, list_depth, ordered, depth + 1);
                }
            }
        }
        "paragraph" => {
            let inline = render_inline_content(node, depth);
            out.push_str(&inline);
            out.push_str("\n\n");
        }
        "heading" => {
            let level = node
                .get("attrs")
                .and_then(|a| a.get("level"))
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(1)
                .min(6) as usize;
            let hashes = "#".repeat(level);
            let text = render_inline_content(node, depth);
            out.push_str(&hashes);
            out.push(' ');
            out.push_str(text.trim_end());
            out.push('\n');
        }
        "codeBlock" => {
            let language = node
                .get("attrs")
                .and_then(|a| a.get("language"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let code_text = collect_text_content(node);
            out.push_str("```");
            out.push_str(language);
            out.push('\n');
            out.push_str(&code_text);
            if !code_text.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("```\n");
        }
        "bulletList" => {
            if let Some(items) = node.get("content").and_then(serde_json::Value::as_array) {
                for item in items {
                    render_list_item(item, out, list_depth, false, depth + 1);
                }
            }
        }
        "orderedList" => {
            if let Some(items) = node.get("content").and_then(serde_json::Value::as_array) {
                for (i, item) in items.iter().enumerate() {
                    render_ordered_list_item(item, out, list_depth, i + 1, depth + 1);
                }
            }
        }
        "listItem" => {
            render_list_item(node, out, list_depth, ordered, depth + 1);
        }
        "text" => {
            out.push_str(&render_text_node(node));
        }
        "hardBreak" => {
            out.push('\n');
        }
        "" => {
            // Missing type field — emit nothing.
        }
        other => {
            out.push_str(FALLBACK_PREFIX);
            out.push_str(other);
            out.push(']');
        }
    }
}

fn render_list_item(
    item: &serde_json::Value,
    out: &mut String,
    list_depth: usize,
    _ordered: bool,
    depth: usize,
) {
    let indent = "  ".repeat(list_depth);
    let inline = render_list_item_inline(item, list_depth, depth);
    out.push_str(&indent);
    out.push_str("- ");
    out.push_str(inline.trim_end());
    out.push('\n');
    render_nested_lists(item, out, list_depth + 1, depth);
}

fn render_ordered_list_item(
    item: &serde_json::Value,
    out: &mut String,
    list_depth: usize,
    index: usize,
    depth: usize,
) {
    let indent = "  ".repeat(list_depth);
    let inline = render_list_item_inline(item, list_depth, depth);
    out.push_str(&indent);
    out.push_str(&index.to_string());
    out.push_str(". ");
    out.push_str(inline.trim_end());
    out.push('\n');
    render_nested_lists(item, out, list_depth + 1, depth);
}

fn render_list_item_inline(item: &serde_json::Value, list_depth: usize, depth: usize) -> String {
    let mut inline = String::new();
    if let Some(children) = item.get("content").and_then(serde_json::Value::as_array) {
        for child in children {
            let child_type = child
                .get("type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            match child_type {
                "paragraph" => {
                    inline.push_str(&render_inline_content(child, depth + 1));
                }
                "bulletList" | "orderedList" => {
                    // nested list — handled by render_nested_lists
                }
                _ => {
                    let mut tmp = String::new();
                    render_node(child, &mut tmp, list_depth, false, depth + 1);
                    inline.push_str(&tmp);
                }
            }
        }
    }
    inline
}

fn render_nested_lists(
    item: &serde_json::Value,
    out: &mut String,
    list_depth: usize,
    depth: usize,
) {
    if let Some(children) = item.get("content").and_then(serde_json::Value::as_array) {
        for child in children {
            let child_type = child
                .get("type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            if child_type == "bulletList" || child_type == "orderedList" {
                render_node(child, out, list_depth, false, depth + 1);
            }
        }
    }
}

fn render_inline_content(node: &serde_json::Value, depth: usize) -> String {
    let mut out = String::new();
    if let Some(children) = node.get("content").and_then(serde_json::Value::as_array) {
        for child in children {
            let child_type = child
                .get("type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            match child_type {
                "text" => out.push_str(&render_text_node(child)),
                "hardBreak" => out.push('\n'),
                other if !other.is_empty() => {
                    out.push_str(FALLBACK_PREFIX);
                    out.push_str(other);
                    out.push(']');
                }
                _ => {}
            }
        }
    }
    if out.is_empty() {
        let _ = depth; // suppress unused warning
        if let Some(t) = node.get("text").and_then(serde_json::Value::as_str) {
            out.push_str(t);
        }
    }
    out
}

fn collect_text_content(node: &serde_json::Value) -> String {
    let mut out = String::new();
    if let Some(children) = node.get("content").and_then(serde_json::Value::as_array) {
        for child in children {
            if let Some(t) = child.get("text").and_then(serde_json::Value::as_str) {
                out.push_str(t);
            }
        }
    }
    out
}

fn render_text_node(node: &serde_json::Value) -> String {
    let text = node
        .get("text")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let marks = node
        .get("marks")
        .and_then(serde_json::Value::as_array)
        .map_or(&[][..], Vec::as_slice);

    let mut result = text.to_owned();
    for mark in marks {
        let mark_type = mark
            .get("type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        match mark_type {
            "code" => result = format!("`{result}`"),
            "strong" => result = format!("**{result}**"),
            "em" => result = format!("_{result}_"),
            _ => {}
        }
    }
    result
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
    use serde_json::{json, Value};

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

    // ─── adf_paragraph_wrap tests ─────────────────────────────────────────

    #[test]
    fn paragraph_wrap_produces_doc() {
        let adf = adf_paragraph_wrap("hello");
        assert_eq!(adf["type"], "doc");
        assert_eq!(adf["version"], 1);
        assert_eq!(adf["content"][0]["type"], "paragraph");
        assert_eq!(adf["content"][0]["content"][0]["type"], "text");
        assert_eq!(adf["content"][0]["content"][0]["text"], "hello");
    }

    #[test]
    fn paragraph_wrap_empty_string() {
        let adf = adf_paragraph_wrap("");
        assert_eq!(adf["content"][0]["content"][0]["text"], "");
    }

    // ─── adf_to_markdown tests ────────────────────────────────────────────

    #[test]
    fn markdown_simple_paragraph() {
        let doc = json!({"type":"doc","version":1,"content":[
            {"type":"paragraph","content":[{"type":"text","text":"Hello world"}]}
        ]});
        assert_eq!(adf_to_markdown(&doc).unwrap().trim(), "Hello world");
    }

    #[test]
    fn markdown_unknown_root_errors() {
        let doc = json!({"type":"paragraph"});
        assert!(adf_to_markdown(&doc).is_err());
    }

    #[test]
    fn markdown_null_returns_via_plain_text() {
        assert_eq!(adf_to_plain_text(&Value::Null), "");
    }

    #[test]
    fn markdown_heading() {
        let doc = json!({"type":"doc","version":1,"content":[
            {"type":"heading","attrs":{"level":2},"content":[{"type":"text","text":"Title"}]}
        ]});
        let out = adf_to_markdown(&doc).unwrap();
        assert!(out.contains("## Title"), "expected '## Title' in: {out}");
    }

    #[test]
    fn markdown_code_block() {
        let doc = json!({"type":"doc","version":1,"content":[
            {"type":"codeBlock","attrs":{"language":"rust"},"content":[{"type":"text","text":"fn main() {}"}]}
        ]});
        let out = adf_to_markdown(&doc).unwrap();
        assert!(
            out.contains("```rust\nfn main() {}\n```"),
            "expected rust code block in: {out}"
        );
    }

    #[test]
    fn markdown_bullet_list() {
        let doc = json!({"type":"doc","version":1,"content":[
            {"type":"bulletList","content":[
                {"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"a"}]}]},
                {"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"b"}]}]}
            ]}
        ]});
        let out = adf_to_markdown(&doc).unwrap();
        assert!(out.contains("- a"), "expected '- a' in: {out}");
        assert!(out.contains("- b"), "expected '- b' in: {out}");
    }

    #[test]
    fn markdown_deep_nesting_does_not_overflow() {
        let depth = MAX_ADF_DEPTH * 2 + 5;
        let mut inner: Value = json!({"type":"listItem","content":[
            {"type":"paragraph","content":[{"type":"text","text":"leaf"}]}
        ]});
        for _ in 0..depth {
            inner = json!({"type":"bulletList","content":[
                {"type":"listItem","content":[
                    {"type":"paragraph","content":[{"type":"text","text":"x"}]},
                    inner
                ]}
            ]});
        }
        let doc = json!({"type":"doc","version":1,"content":[inner]});
        let out = adf_to_markdown(&doc).unwrap();
        assert!(
            out.contains("[depth limit exceeded]"),
            "expected depth-limit marker in output, got: {}",
            &out[..out.len().min(200)]
        );
    }
}
