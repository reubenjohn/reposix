← [back to index](./index.md)

# Task 01-T03 — Read `REPOSIX_BLOB_LIMIT` once into `OnceLock` + define verbatim message constant

<read_first>
- `crates/reposix-remote/src/main.rs` (entire file)
- `crates/reposix-remote/src/stateless_connect.rs` (entire file)
- `crates/reposix-remote/Cargo.toml` (confirm `once_cell` is NOT a dep — std `OnceLock` ships in 1.70+, our toolchain is 1.82+)
</read_first>

<action>
Edit `crates/reposix-remote/src/stateless_connect.rs`. Near the top of the file (after the `use` block, before `RpcStats`):

```rust
/// Default upper bound on `want` lines per `command=fetch` RPC turn.
/// Configurable via `REPOSIX_BLOB_LIMIT` env var. `0` means "unlimited"
/// (explicit opt-out for very-large bulk operations).
pub const DEFAULT_BLOB_LIMIT: u32 = 200;

/// Verbatim stderr message for blob-limit refusal. Backticks around
/// `git sparse-checkout set <pathspec>` are LITERAL — they render as code
/// formatting in agent terminals that support it, and look correct in
/// plaintext. The literal string `git sparse-checkout` is the
/// dark-factory teaching mechanism (REQUIREMENTS.md ARCH-09): an
/// unprompted agent reads the error, runs the named command, and
/// self-corrects with no in-context system-prompt instructions.
pub const BLOB_LIMIT_EXCEEDED_FMT: &str =
    "error: refusing to fetch {N} blobs (limit: {M}). Narrow your scope with `git sparse-checkout set <pathspec>` and retry.";

/// Process-wide cache for `REPOSIX_BLOB_LIMIT`. Read once at first
/// access; subsequent calls are lock-free.
static BLOB_LIMIT: std::sync::OnceLock<u32> = std::sync::OnceLock::new();

/// Resolve the configured blob limit. Reads `REPOSIX_BLOB_LIMIT` once;
/// invalid values (non-numeric, overflow) fall back to
/// [`DEFAULT_BLOB_LIMIT`] with a `tracing::warn!`. `0` is the explicit
/// opt-out and is preserved verbatim.
pub fn configured_blob_limit() -> u32 {
    *BLOB_LIMIT.get_or_init(|| match std::env::var("REPOSIX_BLOB_LIMIT") {
        Ok(s) => match s.trim().parse::<u32>() {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(
                    raw = %s,
                    error = %e,
                    "invalid REPOSIX_BLOB_LIMIT, using default"
                );
                DEFAULT_BLOB_LIMIT
            }
        },
        Err(_) => DEFAULT_BLOB_LIMIT,
    })
}

/// Format the verbatim refusal message with concrete `N` (want count)
/// and `M` (limit) substituted.
pub(crate) fn format_blob_limit_message(want_count: u32, limit: u32) -> String {
    BLOB_LIMIT_EXCEEDED_FMT
        .replace("{N}", &want_count.to_string())
        .replace("{M}", &limit.to_string())
}
```

Inline tests in the same file's `mod tests` block:

```rust
#[test]
fn blob_limit_message_contains_literal_git_sparse_checkout() {
    let msg = format_blob_limit_message(250, 200);
    assert!(msg.contains("git sparse-checkout"),
        "verbatim error message MUST literally contain `git sparse-checkout`; got: {msg}");
    assert!(msg.contains("250"), "want_count substituted");
    assert!(msg.contains("200"), "limit substituted");
    assert!(msg.starts_with("error: refusing to fetch "),
        "exact prefix per ARCH-09: {msg}");
    assert!(msg.contains("`git sparse-checkout set <pathspec>`"),
        "backticks-and-pathspec-template preserved verbatim: {msg}");
}

#[test]
fn configured_blob_limit_default_is_200() {
    // Note: this test runs in the same process as others. To avoid
    // OnceLock state collision, gate by `REPOSIX_BLOB_LIMIT` being
    // unset at test-binary start. The cargo test harness invokes this
    // before any test calls `configured_blob_limit()`. If a sibling
    // test sets the env var, that test must scope its assertion to the
    // raw parser (e.g. via a private helper).
    if std::env::var("REPOSIX_BLOB_LIMIT").is_err() {
        assert_eq!(configured_blob_limit(), 200);
    }
}
```

For env-dependent tests, factor the parsing into a separate pure function `parse_blob_limit(raw: Option<&str>) -> u32` so tests can exercise zero/invalid/large/default paths without OnceLock interference. Refactor `configured_blob_limit` to call the pure helper:

```rust
fn parse_blob_limit(raw: Option<&str>) -> u32 {
    match raw.map(str::trim).filter(|s| !s.is_empty()) {
        None => DEFAULT_BLOB_LIMIT,
        Some(s) => match s.parse::<u32>() {
            Ok(v) => v,
            Err(_) => DEFAULT_BLOB_LIMIT,
        },
    }
}
```

Add tests:

```rust
#[test]
fn parse_blob_limit_default_when_absent() {
    assert_eq!(parse_blob_limit(None), DEFAULT_BLOB_LIMIT);
}

#[test]
fn parse_blob_limit_zero_means_unlimited_value() {
    // `0` is preserved verbatim — the *enforcement* code interprets it
    // as "unlimited"; the parser just returns the raw value.
    assert_eq!(parse_blob_limit(Some("0")), 0);
}

#[test]
fn parse_blob_limit_falls_back_on_garbage() {
    assert_eq!(parse_blob_limit(Some("not-a-number")), DEFAULT_BLOB_LIMIT);
    assert_eq!(parse_blob_limit(Some("")), DEFAULT_BLOB_LIMIT);
    assert_eq!(parse_blob_limit(Some("   ")), DEFAULT_BLOB_LIMIT);
}

#[test]
fn parse_blob_limit_accepts_5() {
    assert_eq!(parse_blob_limit(Some("5")), 5);
}
```
</action>

<acceptance_criteria>
- `grep -n "BLOB_LIMIT_EXCEEDED_FMT" crates/reposix-remote/src/stateless_connect.rs` matches once at the const definition.
- `grep -n "git sparse-checkout" crates/reposix-remote/src/stateless_connect.rs` matches at least once (the const).
- `cargo test -p reposix-remote stateless_connect::tests::blob_limit_message_contains_literal_git_sparse_checkout` exits 0.
- `cargo test -p reposix-remote stateless_connect::tests::parse_blob_limit_default_when_absent` exits 0.
- `cargo test -p reposix-remote stateless_connect::tests::parse_blob_limit_zero_means_unlimited_value` exits 0.
- `cargo test -p reposix-remote stateless_connect::tests::parse_blob_limit_falls_back_on_garbage` exits 0.
- `cargo build -p reposix-remote` exits 0.
</acceptance_criteria>

<threat_model>
Env var is read once at process start; race between threads is impossible because the helper is single-threaded (one stdin/stdout pair per git invocation). `parse::<u32>()` rejects negative numbers and overflow. Garbage input degrades to the default rather than panicking — the helper never silently disables the guardrail on a typo. `tracing::warn!` is stderr-only, no exfil.
</threat_model>
