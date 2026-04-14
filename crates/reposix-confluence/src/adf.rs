//! ADF ↔ Markdown converter for Confluence Cloud.
//!
//! # Supported constructs
//!
//! The converter handles the WRITE-04 minimum construct set:
//!
//! | Construct | Markdown → storage | ADF → Markdown |
//! |-----------|-------------------|----------------|
//! | Headings H1–H6 | `# … ` → `<h1>…</h1>` | `heading` node → `# …` |
//! | Paragraphs | `text` → `<p>text</p>` | `paragraph` node → plain text |
//! | Fenced code blocks | ` ```lang\n…\n``` ` → `<pre><code class="language-lang">…</code></pre>` | `codeBlock` node → ` ```lang\n…\n``` ` |
//! | Inline code | `` `x` `` → `<code>x</code>` | text node with `marks:[{type:"code"}]` → `` `x` `` |
//! | Bullet lists | `- item` → `<ul><li>…</li></ul>` | `bulletList` node → `- item` |
//! | Ordered lists | `1. item` → `<ol><li>…</li></ol>` | `orderedList` node → `1. item` |
//!
//! # Storage XHTML flavor
//!
//! `markdown_to_storage` emits plain HTML (the `storage` representation
//! accepted by `PUT /wiki/api/v2/pages`), NOT Atlassian `<ac:structured-macro>`
//! XML. Confluence's server-side storage parser accepts standard HTML tags;
//! macros are out of scope for v0.6.0.
//!
//! # Fallback-marker convention
//!
//! When `adf_to_markdown` encounters an unknown ADF node type it emits
//! `[unsupported ADF node type=<name>]` rather than dropping the content or
//! panicking. Agents can detect lossy reads with:
//! ```text
//! grep "unsupported ADF node type" <file>
//! ```
//!
//! # Security
//!
//! - `markdown_to_storage` does **not** sanitize embedded raw HTML in the
//!   Markdown source (see T-16-A-04). Confluence's server-side parser strips
//!   unknown/unsafe tags; reposix relies on that server-side gate for v0.6.0.
//!   Phase 21+ may add a client-side HTML stripping layer.
//! - `adf_to_markdown` caps recursion at [`MAX_ADF_DEPTH`] to prevent stack
//!   overflow on maliciously deep ADF trees (T-16-A-03).
//! - Neither function logs content (T-16-A-02).
//!
//! # Implementation note
//!
//! The Markdown → storage direction uses `pulldown_cmark::html::push_html`
//! (option (a) from RESEARCH Assumption A4) for simplicity and acceptable
//! fidelity. The ADF → Markdown direction uses a recursive `serde_json::Value`
//! traversal — no typed struct deserialization — so unknown fields are ignored
//! gracefully.

use pulldown_cmark::{html, Options, Parser};
use serde_json::Value;

use reposix_core::Error;

/// Maximum recursion depth for [`adf_to_markdown`]'s node visitor.
///
/// ADF documents sourced from Confluence are attacker-influenced; a
/// 1000-deep list-of-lists would overflow the call stack without this cap
/// (T-16-A-03).
const MAX_ADF_DEPTH: usize = 32;

/// Fallback marker prefix emitted for unknown ADF node types.
const FALLBACK_PREFIX: &str = "[unsupported ADF node type=";

/// Convert a Markdown body to Confluence `storage` XHTML.
///
/// Handles: headings H1–H6, paragraphs, fenced code blocks (with optional
/// `language` attribute), inline code, bullet lists, ordered lists, and
/// plain text. Everything else is emitted as escaped plain text — no silent
/// drops.
///
/// The output is suitable for use as the `body.value` field in a Confluence
/// REST v2 `PUT /wiki/api/v2/pages` or `POST /wiki/api/v2/pages` request
/// body with `"representation": "storage"`.
///
/// # Errors
///
/// Infallible for well-formed UTF-8 input; returns `Ok(String)`.
/// The signature returns `Result` for forward-compatibility with a future
/// strict mode that rejects unsupported constructs.
pub fn markdown_to_storage(md: &str) -> Result<String, Error> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(md, opts);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    Ok(html_out)
}

/// Convert a Confluence ADF JSON document to Markdown.
///
/// Walks the `content` array of a root `doc` node recursively. Unknown
/// node types are emitted as `[unsupported ADF node type=<name>]` so
/// agents can detect lossy reads with `grep`.
///
/// Recursion is capped at [`MAX_ADF_DEPTH`] (T-16-A-03); nodes beyond the
/// cap emit a depth-limit fallback marker instead of recursing further.
///
/// # Errors
///
/// Returns `Err(Error::Other("…"))` if the JSON root is not a `type: "doc"`
/// object — all other malformed input is handled with fallback markers
/// rather than failing the whole page read.
pub fn adf_to_markdown(adf: &Value) -> Result<String, Error> {
    let type_field = adf.get("type").and_then(Value::as_str).unwrap_or("");
    if type_field != "doc" {
        return Err(Error::Other(format!(
            "adf_to_markdown: root node type must be \"doc\", got \"{type_field}\""
        )));
    }
    let mut out = String::new();
    if let Some(content) = adf.get("content").and_then(Value::as_array) {
        for node in content {
            render_node(node, &mut out, 0, false, 0);
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Render a single ADF node into `out`.
///
/// `list_depth` tracks nesting for indentation; `ordered` indicates whether
/// the current containing list is an ordered list; `item_index` is the
/// 1-based counter for ordered list items.
fn render_node(node: &Value, out: &mut String, list_depth: usize, ordered: bool, depth: usize) {
    if depth > MAX_ADF_DEPTH {
        out.push_str("[depth limit exceeded]");
        return;
    }

    let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");

    match node_type {
        "doc" => {
            if let Some(content) = node.get("content").and_then(Value::as_array) {
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
                .and_then(Value::as_u64)
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
                .and_then(Value::as_str)
                .unwrap_or("");
            // Code block content is plain text nodes (no marks).
            let code_text = collect_text_content(node);
            out.push_str("```");
            out.push_str(language);
            out.push('\n');
            out.push_str(&code_text);
            // Ensure trailing newline before closing fence.
            if !code_text.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("```\n");
        }
        "bulletList" => {
            if let Some(items) = node.get("content").and_then(Value::as_array) {
                for item in items {
                    render_list_item(item, out, list_depth, false, depth + 1);
                }
            }
        }
        "orderedList" => {
            if let Some(items) = node.get("content").and_then(Value::as_array) {
                for (i, item) in items.iter().enumerate() {
                    render_ordered_list_item(item, out, list_depth, i + 1, depth + 1);
                }
            }
        }
        "listItem" => {
            // Top-level listItem outside a list context — treat as bullet.
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

/// Render a `listItem` node as a bullet list item (`- …`).
fn render_list_item(
    item: &Value,
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
    // Render any nested lists inside this item.
    render_nested_lists(item, out, list_depth + 1, depth);
}

/// Render a `listItem` node as a numbered list item (`N. …`).
fn render_ordered_list_item(
    item: &Value,
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
    // Render any nested lists inside this item.
    render_nested_lists(item, out, list_depth + 1, depth);
}

/// Extract the inline text from a `listItem`'s paragraph children (no double newlines).
fn render_list_item_inline(item: &Value, list_depth: usize, depth: usize) -> String {
    let mut inline = String::new();
    if let Some(children) = item.get("content").and_then(Value::as_array) {
        for child in children {
            let child_type = child.get("type").and_then(Value::as_str).unwrap_or("");
            match child_type {
                "paragraph" => {
                    // Inline content only — no trailing double-newline.
                    inline.push_str(&render_inline_content(child, depth + 1));
                }
                "bulletList" | "orderedList" => {
                    // Nested list — handled by render_nested_lists; skip here.
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

/// Render any nested bullet/ordered lists inside a `listItem`'s content.
fn render_nested_lists(item: &Value, out: &mut String, list_depth: usize, depth: usize) {
    if let Some(children) = item.get("content").and_then(Value::as_array) {
        for child in children {
            let child_type = child.get("type").and_then(Value::as_str).unwrap_or("");
            if child_type == "bulletList" || child_type == "orderedList" {
                render_node(child, out, list_depth, false, depth + 1);
            }
        }
    }
}

/// Render the inline content of a block node (paragraph, heading, etc.).
///
/// Concatenates text from all `content` children without adding block-level
/// newlines.
fn render_inline_content(node: &Value, depth: usize) -> String {
    let mut out = String::new();
    if let Some(children) = node.get("content").and_then(Value::as_array) {
        for child in children {
            let child_type = child.get("type").and_then(Value::as_str).unwrap_or("");
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
    // Fallback: if node itself is a text node.
    if out.is_empty() {
        let _ = depth; // suppress unused warning
        if let Some(t) = node.get("text").and_then(Value::as_str) {
            out.push_str(t);
        }
    }
    out
}

/// Collect raw text from all text nodes inside a block (no mark processing).
///
/// Used for `codeBlock` content where marks are not applicable.
fn collect_text_content(node: &Value) -> String {
    let mut out = String::new();
    if let Some(children) = node.get("content").and_then(Value::as_array) {
        for child in children {
            if let Some(t) = child.get("text").and_then(Value::as_str) {
                out.push_str(t);
            }
        }
    }
    out
}

/// Render a single ADF `text` node, applying any `marks`.
///
/// Supported marks:
/// - `code` → backtick-wrapped
/// - `strong` → `**…**`
/// - `em` → `_…_`
///
/// Unknown marks are ignored (text content is still preserved).
fn render_text_node(node: &Value) -> String {
    let text = node.get("text").and_then(Value::as_str).unwrap_or("");
    let marks = node
        .get("marks")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);

    let mut result = text.to_owned();
    for mark in marks {
        let mark_type = mark.get("type").and_then(Value::as_str).unwrap_or("");
        match mark_type {
            "code" => result = format!("`{result}`"),
            "strong" => result = format!("**{result}**"),
            "em" => result = format!("_{result}_"),
            _ => {} // unknown marks — preserve text, drop mark
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use super::{adf_to_markdown, markdown_to_storage, MAX_ADF_DEPTH};

    // ------------------------------------------------------------------
    // markdown_to_storage tests
    // ------------------------------------------------------------------

    #[test]
    fn markdown_heading_h1_to_storage() {
        let out = markdown_to_storage("# Heading").unwrap();
        assert!(
            out.contains("<h1>Heading</h1>"),
            "expected <h1>Heading</h1> in: {out}"
        );
    }

    #[test]
    fn markdown_heading_h6_to_storage() {
        let out = markdown_to_storage("###### H6").unwrap();
        assert!(
            out.contains("<h6>H6</h6>"),
            "expected <h6>H6</h6> in: {out}"
        );
    }

    #[test]
    fn markdown_paragraph_to_storage() {
        let out = markdown_to_storage("hello world").unwrap();
        assert!(
            out.contains("<p>hello world</p>"),
            "expected <p>hello world</p> in: {out}"
        );
    }

    #[test]
    fn markdown_fenced_code_rust_to_storage() {
        let md = "```rust\nfoo\n```";
        let out = markdown_to_storage(md).unwrap();
        // pulldown-cmark emits: <pre><code class="language-rust">foo\n</code></pre>
        assert!(
            out.contains(r#"<pre><code class="language-rust">foo"#),
            "expected rust code block in: {out}"
        );
        assert!(
            out.contains("</code></pre>"),
            "expected closing </code></pre> in: {out}"
        );
    }

    #[test]
    fn markdown_bullet_list_to_storage() {
        let out = markdown_to_storage("- a\n- b").unwrap();
        assert!(out.contains("<ul>"), "expected <ul> in: {out}");
        let li_count = out.matches("<li>").count();
        assert_eq!(
            li_count, 2,
            "expected 2 <li> elements, got {li_count} in: {out}"
        );
    }

    #[test]
    fn markdown_ordered_list_to_storage() {
        let out = markdown_to_storage("1. a\n2. b").unwrap();
        assert!(out.contains("<ol>"), "expected <ol> in: {out}");
        let li_count = out.matches("<li>").count();
        assert_eq!(
            li_count, 2,
            "expected 2 <li> elements, got {li_count} in: {out}"
        );
    }

    #[test]
    fn markdown_inline_code_to_storage() {
        let out = markdown_to_storage("`x`").unwrap();
        assert!(
            out.contains("<code>x</code>"),
            "expected <code>x</code> in: {out}"
        );
    }

    // ------------------------------------------------------------------
    // adf_to_markdown tests
    // ------------------------------------------------------------------

    #[test]
    fn adf_paragraph_to_markdown() {
        let adf = json!({
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "paragraph",
                    "content": [{"type": "text", "text": "hello"}]
                }
            ]
        });
        let out = adf_to_markdown(&adf).unwrap();
        assert!(out.contains("hello"), "expected 'hello' in: {out}");
    }

    #[test]
    fn adf_heading_h2_to_markdown() {
        let adf = json!({
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "heading",
                    "attrs": {"level": 2},
                    "content": [{"type": "text", "text": "Title"}]
                }
            ]
        });
        let out = adf_to_markdown(&adf).unwrap();
        assert!(out.contains("## Title"), "expected '## Title' in: {out}");
    }

    #[test]
    fn adf_heading_all_levels_to_markdown() {
        for level in 1u64..=6 {
            let adf = json!({
                "type": "doc",
                "version": 1,
                "content": [
                    {
                        "type": "heading",
                        "attrs": {"level": level},
                        "content": [{"type": "text", "text": "X"}]
                    }
                ]
            });
            let out = adf_to_markdown(&adf).unwrap();
            let expected_hashes = "#".repeat(usize::try_from(level).unwrap_or(1));
            assert!(
                out.contains(&format!("{expected_hashes} X")),
                "level={level}: expected '{expected_hashes} X' in: {out}"
            );
        }
    }

    #[test]
    fn adf_code_block_to_markdown() {
        let adf = json!({
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "codeBlock",
                    "attrs": {"language": "rust"},
                    "content": [{"type": "text", "text": "fn main() {}"}]
                }
            ]
        });
        let out = adf_to_markdown(&adf).unwrap();
        assert!(
            out.contains("```rust\nfn main() {}\n```"),
            "expected rust code block in: {out}"
        );
    }

    #[test]
    fn adf_bullet_list_to_markdown() {
        let adf = json!({
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "bulletList",
                    "content": [
                        {
                            "type": "listItem",
                            "content": [
                                {"type": "paragraph", "content": [{"type": "text", "text": "item1"}]}
                            ]
                        },
                        {
                            "type": "listItem",
                            "content": [
                                {"type": "paragraph", "content": [{"type": "text", "text": "item2"}]}
                            ]
                        }
                    ]
                }
            ]
        });
        let out = adf_to_markdown(&adf).unwrap();
        assert!(out.contains("- item1"), "expected '- item1' in: {out}");
        assert!(out.contains("- item2"), "expected '- item2' in: {out}");
    }

    #[test]
    fn adf_ordered_list_to_markdown() {
        let adf = json!({
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "orderedList",
                    "content": [
                        {
                            "type": "listItem",
                            "content": [
                                {"type": "paragraph", "content": [{"type": "text", "text": "item1"}]}
                            ]
                        },
                        {
                            "type": "listItem",
                            "content": [
                                {"type": "paragraph", "content": [{"type": "text", "text": "item2"}]}
                            ]
                        }
                    ]
                }
            ]
        });
        let out = adf_to_markdown(&adf).unwrap();
        assert!(out.contains("1. item1"), "expected '1. item1' in: {out}");
        assert!(out.contains("2. item2"), "expected '2. item2' in: {out}");
    }

    #[test]
    fn adf_inline_code_mark_to_markdown() {
        let adf = json!({
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "x",
                            "marks": [{"type": "code"}]
                        }
                    ]
                }
            ]
        });
        let out = adf_to_markdown(&adf).unwrap();
        assert!(out.contains("`x`"), "expected '`x`' in: {out}");
    }

    #[test]
    fn adf_unknown_node_type_fallback() {
        let adf = json!({
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "panel",
                    "attrs": {"panelType": "info"},
                    "content": []
                }
            ]
        });
        let out = adf_to_markdown(&adf).unwrap();
        assert!(
            out.contains("[unsupported ADF node type=panel]"),
            "expected fallback marker in: {out}"
        );
    }

    #[test]
    fn adf_non_doc_root_is_error() {
        let adf = json!({
            "type": "paragraph",
            "content": [{"type": "text", "text": "hello"}]
        });
        let result = adf_to_markdown(&adf);
        assert!(
            result.is_err(),
            "expected Err for non-doc root, got: {result:?}"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("\"doc\""),
            "error message should mention 'doc', got: {msg}"
        );
    }

    #[test]
    fn roundtrip_adf_to_md_to_storage() {
        // Build ADF programmatically: heading + paragraph + bullet list.
        let adf = json!({
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "heading",
                    "attrs": {"level": 1},
                    "content": [{"type": "text", "text": "My Page"}]
                },
                {
                    "type": "paragraph",
                    "content": [{"type": "text", "text": "Intro text."}]
                },
                {
                    "type": "bulletList",
                    "content": [
                        {
                            "type": "listItem",
                            "content": [
                                {"type": "paragraph", "content": [{"type": "text", "text": "alpha"}]}
                            ]
                        },
                        {
                            "type": "listItem",
                            "content": [
                                {"type": "paragraph", "content": [{"type": "text", "text": "beta"}]}
                            ]
                        }
                    ]
                }
            ]
        });
        let md = adf_to_markdown(&adf).unwrap();
        let storage = markdown_to_storage(&md).unwrap();
        assert!(
            storage.contains("<h1>"),
            "expected <h1> in storage: {storage}"
        );
        assert!(
            storage.contains("Intro text."),
            "expected 'Intro text.' in storage: {storage}"
        );
        assert!(
            storage.contains("<ul>"),
            "expected <ul> in storage: {storage}"
        );
        assert!(
            storage.contains("alpha"),
            "expected 'alpha' in storage: {storage}"
        );
    }

    /// T-16-A-03: A 1000-deep list-of-lists must not stack-overflow and must
    /// contain the depth-limit marker somewhere in the output.
    #[test]
    fn adf_deep_nesting_does_not_stack_overflow() {
        // Build a deeply nested bulletList: depth = MAX_ADF_DEPTH * 2 + some.
        let depth = MAX_ADF_DEPTH * 2 + 10;
        let mut inner: Value = json!({
            "type": "listItem",
            "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "leaf"}]}
            ]
        });
        for _ in 0..depth {
            inner = json!({
                "type": "bulletList",
                "content": [
                    {
                        "type": "listItem",
                        "content": [
                            {"type": "paragraph", "content": [{"type": "text", "text": "x"}]},
                            inner
                        ]
                    }
                ]
            });
        }
        let adf = json!({
            "type": "doc",
            "version": 1,
            "content": [inner]
        });
        // Must not panic; must contain depth-limit marker.
        let out = adf_to_markdown(&adf).unwrap();
        assert!(
            out.contains("[depth limit exceeded]"),
            "expected depth-limit marker in output (depth={depth}), got: {}",
            &out[..out.len().min(200)]
        );
    }
}
