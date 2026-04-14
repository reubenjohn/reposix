# Plan: Wave A — ADF ↔ Markdown converter module (`crates/reposix-confluence/src/adf.rs`)

## Goal

Ship a hand-rolled, network-free `adf` submodule that converts Markdown → Confluence `storage` XHTML (for write-path request bodies) and Confluence ADF JSON → Markdown (for read-path issue bodies), covering the WRITE-04 minimum construct set with inline unit tests — no backend, no HTTP, no wiremock in this wave.

## Wave

A (depends on: nothing; unblocks: Wave B + Wave D).

## Addresses

- **Requirement:** WRITE-04 (round-trip through ADF↔Markdown with no data loss for headings, paragraphs, code blocks, and lists).
- **Locked decisions:** none directly; Wave A is the pure-function substrate Wave B's write methods call into.

## Tasks

### A1. Add `pulldown-cmark = "0.13"` dependency

- **File:** `crates/reposix-confluence/Cargo.toml`
- **Action:** Add `pulldown-cmark = "0.13"` under `[dependencies]` (runtime dep — used on the write hot path, not just tests). Also add `pulldown-cmark` to the workspace `[workspace.dependencies]` block in the root `Cargo.toml` with version `"0.13"`, then reference it as `pulldown-cmark.workspace = true` from the Confluence crate. This matches the existing pattern for `serde`, `tokio`, etc.
- **Expected line impact:** ~2 lines in each Cargo.toml.
- **Verification:** `cargo check -p reposix-confluence` succeeds with the new dep resolved; no version conflicts surface in the root Cargo.lock.

### A2. Create `crates/reposix-confluence/src/adf.rs` with the public API surface

- **File (new):** `crates/reposix-confluence/src/adf.rs`
- **File:** `crates/reposix-confluence/src/lib.rs` — add `pub mod adf;` at the top (under the crate-level attrs, above the `use` block).
- **Action:** Create the module with the full documented API plus both direction implementations. Required items:
  - Module-level doc (`//!`) explaining: (a) the supported ADF construct set, (b) the storage XHTML flavor we emit (plain HTML, not Atlassian `<ac:structured-macro>` — documented limitation in RESEARCH Risk 1), (c) the fallback-marker convention `[unsupported ADF node type=X]` so agents can grep for lossy reads.
  - `#![forbid(unsafe_code)]` is already crate-level; `#![warn(clippy::pedantic, missing_docs)]` is inherited from `lib.rs`. Every public item needs a doc comment; every `Result`-returning function needs an `# Errors` section.
  - Two public free functions:
    ```rust
    /// Convert a Markdown body to Confluence `storage` XHTML.
    ///
    /// Handles: headings H1–H6, paragraphs, fenced code blocks (with
    /// optional `language` attribute), inline code, bullet lists,
    /// ordered lists, plain text. Everything else is emitted as
    /// escaped plain text — no silent drops.
    ///
    /// # Errors
    /// Infallible for well-formed UTF-8 input; returns `Ok(String)`.
    /// Signature returns `Result` for forward-compatibility with a
    /// future strict mode that rejects unsupported constructs.
    pub fn markdown_to_storage(md: &str) -> Result<String, Error>;

    /// Convert a Confluence ADF JSON document to Markdown.
    ///
    /// Walks the `content` array of a root `doc` node recursively.
    /// Unknown node types are emitted as `[unsupported ADF node
    /// type=<name>]` so agents can detect lossy reads with `grep`.
    ///
    /// # Errors
    /// Returns `Err(Error::Other("…"))` if the JSON root is not a
    /// `type: "doc"` object — all other malformed input is handled
    /// with fallback markers rather than failing the whole page read.
    pub fn adf_to_markdown(adf: &serde_json::Value) -> Result<String, Error>;
    ```
  - Use `reposix_core::Error` for error type (crate already re-exports it). Import via `use reposix_core::Error;`.
  - **Markdown → storage implementation:** Use `pulldown_cmark::{Parser, Event, Tag, TagEnd, CodeBlockKind, html}`. Either (a) call `html::push_html(&mut out, parser)` for a quick baseline and post-process, OR (b) walk events and emit XHTML manually to get control over code-block language attrs. Pick (a) for v0.6.0 simplicity (RESEARCH Assumption A4 — "HTML output gives acceptable fidelity"). Document the choice in the module doc.
  - **ADF → Markdown implementation:** Recursive `serde_json::Value` traversal. Helper fns per node type: `render_doc(&Value) -> String`, `render_node(&Value, out: &mut String, list_depth: usize, ordered: bool)`, `render_text_node(&Value) -> String` (inline — handles `marks: [{type: "code"}]` by wrapping in backticks). Emit trailing `\n\n` between top-level blocks. Lists emit one `- ` / `1. ` per `listItem`; nested paragraphs inside list items emit inline (no double newline).
- **Expected line impact:** ~300–400 lines (implementation + doc comments + tests), excluding tests.
- **Verification:** `cargo check -p reposix-confluence` compiles; `cargo clippy -p reposix-confluence --all-targets -- -D warnings` clean.

### A3. Inline unit tests covering the WRITE-04 construct matrix

- **File:** `crates/reposix-confluence/src/adf.rs` (same file, in `#[cfg(test)] mod tests { … }` at the bottom)
- **Action:** Author the 11 tests listed in RESEARCH.md §Test Strategy §ADF Converter Unit Tests, plus two extras to pin the fallback-marker behavior and inline-code round-trip:

  | Test fn name | What it asserts |
  |---|---|
  | `markdown_heading_h1_to_storage` | `# Heading` input → output contains `<h1>Heading</h1>` |
  | `markdown_heading_h6_to_storage` | `###### H6` input → output contains `<h6>H6</h6>` |
  | `markdown_paragraph_to_storage` | `hello world` → output contains `<p>hello world</p>` |
  | `markdown_fenced_code_rust_to_storage` | ` ```rust\nfoo\n``` ` → output contains `<pre><code class="language-rust">foo\n</code></pre>` (pulldown-cmark's default shape; lock it down) |
  | `markdown_bullet_list_to_storage` | `- a\n- b` → output contains `<ul>` with two `<li>` |
  | `markdown_ordered_list_to_storage` | `1. a\n2. b` → output contains `<ol>` with two `<li>` |
  | `markdown_inline_code_to_storage` | `` `x` `` → output contains `<code>x</code>` |
  | `adf_paragraph_to_markdown` | ADF `{type:"paragraph",content:[{type:"text",text:"hello"}]}` → string contains `"hello"` |
  | `adf_heading_h2_to_markdown` | ADF heading level=2, text="Title" → string contains `"## Title"` |
  | `adf_heading_all_levels_to_markdown` | iterate level 1..=6; assert output has the right count of `#` |
  | `adf_code_block_to_markdown` | ADF `codeBlock` with `attrs.language="rust"` and text "fn main() {}" → output contains ` ```rust\nfn main() {}\n``` ` |
  | `adf_bullet_list_to_markdown` | two-item `bulletList` with paragraph children → `"- item1\n- item2\n"` |
  | `adf_ordered_list_to_markdown` | two-item `orderedList` → `"1. item1\n2. item2\n"` |
  | `adf_inline_code_mark_to_markdown` | text node with `marks:[{type:"code"}]` → backtick-wrapped |
  | `adf_unknown_node_type_fallback` | `{type:"panel",…}` → output contains `[unsupported ADF node type=panel]`, no panic |
  | `adf_non_doc_root_is_error` | ADF root with `type:"paragraph"` instead of `"doc"` → returns `Err` |
  | `roundtrip_heading_paragraph_list` | Markdown → storage (pulldown-cmark HTML) → (parse storage as ADF-lite? skip if not straightforward). If direct storage→Markdown isn't implemented, test the simpler round-trip: build an ADF document programmatically, run `adf_to_markdown`, then run `markdown_to_storage` on the result, and assert the storage contains the expected tags. Names this test `roundtrip_adf_to_md_to_storage`. |

- **Expected line impact:** ~200–300 lines (test bodies + fixtures).
- **Verification:** `cargo test -p reposix-confluence adf` — all tests green; count should be ≥15.

## Verification

Before merging Wave A:

```bash
cargo test -p reposix-confluence adf
cargo test -p reposix-confluence                                     # full crate
cargo test --workspace                                               # no regressions elsewhere
cargo clippy --workspace --all-targets -- -D warnings                # clean
cargo fmt --all --check                                              # formatted
```

All five must pass. Test count MUST NOT decrease from baseline 278; expected new count ≈ 278 + 15–17 = **293–295** after Wave A.

## Threat model

| Threat ID | STRIDE | Component | Disposition | Mitigation |
|---|---|---|---|---|
| T-16-A-01 | Tampering | ADF tree from remote | Mitigate | `adf_to_markdown` never `unwrap()`s on a JSON field; every `.get(…)` returns `Option`, every branch has a fallback. No panic on malformed input. Enforced by the `adf_unknown_node_type_fallback` and `adf_non_doc_root_is_error` tests. |
| T-16-A-02 | Information Disclosure | Rendered Markdown output | Accept | Converter is a pure function; callers decide where the output goes. No logging of body contents in the converter itself. |
| T-16-A-03 | DoS | Deeply-nested malicious ADF (list-of-list-of-list…) | Mitigate | Recursion in `adf_to_markdown` must have a depth cap of 32 — returns fallback marker and stops recursing past the cap. Add a `adf_deep_nesting_does_not_stack_overflow` test that feeds a 1000-deep list and asserts the call returns without panicking and contains a depth-limit marker. |
| T-16-A-04 | Tampering | Markdown input with embedded `<script>` tags | Accept + Document | pulldown-cmark's HTML output passes through raw HTML in the Markdown source. Confluence's storage format will likely strip unknown tags server-side, but we document in the module doc that reposix does NOT sanitize Markdown HTML before sending. Future hardening (Phase 21+) can add a stripping layer. For v0.6.0: relies on Confluence server-side stripping. |

## Success criteria

1. `crates/reposix-confluence/src/adf.rs` exists and is referenced from `lib.rs` via `pub mod adf;`.
2. `pulldown-cmark = "0.13"` is in both the workspace and the Confluence crate's Cargo.toml; `cargo tree -p reposix-confluence | grep pulldown-cmark` prints a single matching line.
3. `cargo test -p reposix-confluence adf` runs ≥15 tests, all green.
4. `cargo clippy --workspace --all-targets -- -D warnings` is clean.
5. Workspace test count after Wave A ≥ 293 (baseline 278 + ≥15 new).
6. No network call is made from any test in this wave (grep-check: `grep -n 'MockServer\|wiremock\|reqwest' crates/reposix-confluence/src/adf.rs` returns nothing).
