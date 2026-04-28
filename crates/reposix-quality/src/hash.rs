//! Hash semantics for the docs-alignment dimension.
//!
//! - `source_hash`: sha256 of the cited line range bytes (1-based, inclusive),
//!   joined by '\n'. Hex-encoded.
//! - `test_body_hash`: sha256 of `to_token_stream().to_string()` of the named
//!   fn (free fn or impl-block method). Comments + whitespace normalize away
//!   via the syn -> token-stream round-trip.
//!
//! Spec: `.planning/research/v0.12.0-docs-alignment-design/02-architecture.md`
//! § "Hash semantics".

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use quote::ToTokens;
use sha2::{Digest, Sha256};
use syn::visit::{self, Visit};
use syn::{ImplItemFn, ItemFn};

/// Hash a 1-based, inclusive line range from a UTF-8 file.
///
/// Lines are joined by `\n` (no trailing newline) before hashing.
///
/// # Errors
///
/// Errors if `line_start == 0`, if `line_end < line_start`, if the file
/// cannot be read, or if `line_end` exceeds the file's line count.
pub fn source_hash(file: &Path, line_start: usize, line_end: usize) -> Result<String> {
    if line_start == 0 {
        return Err(anyhow!(
            "source_hash: line_start must be >= 1 (got 0) for {}",
            file.display()
        ));
    }
    if line_end < line_start {
        return Err(anyhow!(
            "source_hash: line_end ({line_end}) < line_start ({line_start}) for {}",
            file.display()
        ));
    }

    let raw = fs::read_to_string(file)
        .with_context(|| format!("reading source file {}", file.display()))?;
    let lines: Vec<&str> = raw.split('\n').collect();
    if line_end > lines.len() {
        return Err(anyhow!(
            "source_hash: line_end ({line_end}) out of bounds (file has {} lines) for {}",
            lines.len(),
            file.display()
        ));
    }
    // 1-based inclusive: indices [line_start-1 .. line_end].
    let slice = &lines[line_start - 1..line_end];
    let joined = slice.join("\n");

    let mut h = Sha256::new();
    h.update(joined.as_bytes());
    Ok(hex::encode(h.finalize()))
}

/// Hash entire file content; used for non-Rust test verifiers (shell scripts,
/// Python, YAML).
///
/// Returns the sha256 of the file's raw bytes as a lowercase hex string. No
/// normalization (no whitespace stripping, no line-ending fix-up) -- the
/// caller commits the verifier exactly as-is.
///
/// # Errors
///
/// Errors if the file cannot be read.
pub fn file_hash(path: &Path) -> Result<String> {
    let bytes =
        fs::read(path).with_context(|| format!("reading verifier file {}", path.display()))?;
    let mut h = Sha256::new();
    h.update(&bytes);
    Ok(hex::encode(h.finalize()))
}

/// Hash the body of a named fn (free fn or impl-block method).
///
/// # Errors
///
/// Errors if the file does not parse as Rust, if the named fn is not found,
/// or if multiple fns share the same simple name (rationale must qualify).
///
/// # Panics
///
/// Panics only on internal invariant violation: when the post-collection
/// `total > 0` count is non-zero but neither the free-fn vec nor the impl-fn
/// vec yields an item. This is unreachable by construction.
pub fn test_body_hash(file: &Path, fn_name: &str) -> Result<String> {
    let raw = fs::read_to_string(file)
        .with_context(|| format!("reading test file {}", file.display()))?;
    let parsed: syn::File =
        syn::parse_file(&raw).with_context(|| format!("parsing {} as Rust", file.display()))?;

    let mut collector = FnCollector {
        target: fn_name,
        free_hits: Vec::new(),
        impl_hits: Vec::new(),
    };
    collector.visit_file(&parsed);

    let total = collector.free_hits.len() + collector.impl_hits.len();
    if total == 0 {
        return Err(anyhow!("fn `{fn_name}` not found in {}", file.display()));
    }
    if total > 1 {
        return Err(anyhow!(
            "multiple fns named `{fn_name}` found in {} (count={total}); qualify with impl-block path in rationale",
            file.display()
        ));
    }

    let token_string = if let Some(f) = collector.free_hits.into_iter().next() {
        f.to_token_stream().to_string()
    } else {
        collector
            .impl_hits
            .into_iter()
            .next()
            .expect("total > 0 with no free hit implies an impl hit")
            .to_token_stream()
            .to_string()
    };

    let mut h = Sha256::new();
    h.update(token_string.as_bytes());
    Ok(hex::encode(h.finalize()))
}

struct FnCollector<'a> {
    target: &'a str,
    free_hits: Vec<ItemFn>,
    impl_hits: Vec<ImplItemFn>,
}

impl<'ast> Visit<'ast> for FnCollector<'_> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if node.sig.ident == self.target {
            self.free_hits.push(node.clone());
        }
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        if node.sig.ident == self.target {
            self.impl_hits.push(node.clone());
        }
        visit::visit_impl_item_fn(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_tmp(dir: &tempfile::TempDir, name: &str, body: &str) -> std::path::PathBuf {
        let p = dir.path().join(name);
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        p
    }

    #[test]
    fn source_hash_inclusive_range() {
        let dir = tempfile::tempdir().unwrap();
        let p = write_tmp(&dir, "doc.md", "alpha\nbeta\ngamma\ndelta\n");
        let h1 = source_hash(&p, 2, 3).unwrap();
        // "beta\ngamma" hash should be deterministic across calls.
        let h2 = source_hash(&p, 2, 3).unwrap();
        assert_eq!(h1, h2);
        // Different range -> different hash.
        let h3 = source_hash(&p, 1, 3).unwrap();
        assert_ne!(h1, h3);
    }

    #[test]
    fn source_hash_out_of_bounds_errors() {
        let dir = tempfile::tempdir().unwrap();
        let p = write_tmp(&dir, "doc.md", "a\nb\n");
        assert!(source_hash(&p, 1, 99).is_err());
        assert!(source_hash(&p, 0, 1).is_err());
        assert!(source_hash(&p, 3, 2).is_err());
    }

    #[test]
    fn test_body_hash_finds_free_fn() {
        let dir = tempfile::tempdir().unwrap();
        let body = "fn alpha() { let x = 1; assert_eq!(x, 1); }\nfn beta() {}\n";
        let p = write_tmp(&dir, "t.rs", body);
        let h = test_body_hash(&p, "alpha").unwrap();
        assert_eq!(h.len(), 64);
    }

    #[test]
    fn test_body_hash_finds_impl_method() {
        let dir = tempfile::tempdir().unwrap();
        let body = "struct S; impl S { fn alpha(&self) { let _ = 1; } }\n";
        let p = write_tmp(&dir, "t.rs", body);
        let h = test_body_hash(&p, "alpha").unwrap();
        assert_eq!(h.len(), 64);
    }

    #[test]
    fn test_body_hash_missing_errors() {
        let dir = tempfile::tempdir().unwrap();
        let p = write_tmp(&dir, "t.rs", "fn other() {}\n");
        let err = test_body_hash(&p, "alpha").unwrap_err();
        assert!(err.to_string().contains("not found"), "got: {err}");
    }

    #[test]
    fn test_body_hash_ambiguous_errors() {
        let dir = tempfile::tempdir().unwrap();
        let body = "fn alpha() {} fn beta() {} struct S; impl S { fn alpha(&self) {} }\n";
        let p = write_tmp(&dir, "t.rs", body);
        let err = test_body_hash(&p, "alpha").unwrap_err();
        assert!(err.to_string().contains("multiple"), "got: {err}");
    }

    #[test]
    fn file_hash_round_trips_byte_identical_content() {
        let dir = tempfile::tempdir().unwrap();
        let body = "#!/usr/bin/env bash\nset -euo pipefail\necho hello\n";
        let p1 = write_tmp(&dir, "script-a.sh", body);
        let p2 = write_tmp(&dir, "script-b.sh", body);
        let h1 = file_hash(&p1).unwrap();
        let h2 = file_hash(&p2).unwrap();
        assert_eq!(h1, h2, "byte-identical content -> identical hashes");
        assert_eq!(h1.len(), 64, "sha256 hex digest is 64 chars");

        // Mutating one file produces a different hash.
        let body2 = "#!/usr/bin/env bash\nset -euo pipefail\necho goodbye\n";
        let p3 = write_tmp(&dir, "script-c.sh", body2);
        let h3 = file_hash(&p3).unwrap();
        assert_ne!(h1, h3, "different content -> different hash");

        // Missing file errors.
        assert!(file_hash(&dir.path().join("nope.sh")).is_err());
    }
}
