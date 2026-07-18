//! `reposix explain <code>` — the `RPX-xxxx` code-lookup subcommand (Phase 121 /
//! P121), the codified, queryable half of the project's Rust-compiler-grade UX
//! north star.
//!
//! This mirrors `rustc --explain E0308`: a dev who hits `RPX-0201` on stderr
//! runs `reposix explain RPX-0201` and reads the EXTENDED cause / fix /
//! alternative / copy-paste recovery for that code.
//!
//! Two-tier by design (see [`reposix_core::codes`]): the inline error render
//! prints only the terse `[RPX-xxxx]` tag + an `Explain: reposix explain
//! RPX-xxxx` nudge; THIS command prints the registry entry's full extended
//! prose. All output is drawn from the static [`reposix_core::codes::REGISTRY`]
//! — never hardcoded here (single source of truth) and never a remote byte
//! (OP-2 / T-121-01: the registry is 100% `&'static str`).

use anyhow::{bail, Result};
use reposix_core::codes::{self, ids, ExplainEntry};
use reposix_core::errmsg::teach_coded;

/// Run `reposix explain`.
///
/// - `Some(code)` with `list == false` → print that one code's extended
///   explanation (SC3 shape), or teach the unknown-code path via `RPX-0900`.
/// - no code, or `--list` (with or without a code) → enumerate every registered
///   code + title, sorted, one per line.
///
/// # Errors
///
/// Returns an error (exit 1) only for an UNKNOWN code — a teaching `RPX-0900`
/// message that names `reposix explain --list`. The enumerate and known-code
/// paths always succeed.
pub fn run(code: Option<String>, list: bool) -> Result<()> {
    match code {
        Some(code) if !list => print_one(&code),
        _ => {
            print_list();
            Ok(())
        }
    }
}

/// Enumerate every registered code + title, sorted by code, one per line.
///
/// REGISTRY-DRIVEN: iterates [`codes::REGISTRY`] directly, so every entry ever
/// added (including future W3/W4 additions) shows up automatically — there is no
/// hardcoded list to drift.
fn print_list() {
    let mut entries: Vec<&ExplainEntry> = codes::REGISTRY.iter().collect();
    entries.sort_by_key(|e| e.code);
    println!(
        "reposix error codes ({} total) — run `reposix explain <code>` for the full explanation:\n",
        entries.len()
    );
    for entry in entries {
        println!("  {}  {}", entry.code, entry.title);
    }
}

/// Print one code's extended explanation, mirroring `rustc --explain`'s
/// "code header + extended body + how-to-fix" shape (SC3):
///
/// ```text
/// RPX-0201: <title>
///
/// <cause — extended prose>
///
/// Fix: <fix>
/// Alternative: <alternative>   # omitted when empty
/// Recovery:
///   <cmd1>
///   <cmd2>
/// ```
fn print_one(code: &str) -> Result<()> {
    let Some(entry) = codes::explain(code) else {
        // Dogfood the 3-part teaching bar for the unknown-code path via the
        // explain-meta code RPX-0900 — a teaching message that names how to list
        // valid codes, exits non-zero, and never panics or prints a bare "not
        // found". anyhow prints this to stderr and the process exits 1.
        bail!(
            "{}",
            teach_coded(
                ids::EXPLAIN_UNKNOWN_CODE,
                &format!("no such reposix error code: `{code}`."),
                "check the spelling — RPX codes are always four digits, e.g. `RPX-0201`. \
                 `reposix explain --list` prints every code reposix knows about with its title.",
                "browse the full code index in the docs at docs/reference/error-codes.md.",
                &["reposix explain --list     # every defined RPX code + title"],
            )
        );
    };

    println!("{}: {}", entry.code, entry.title);
    println!();
    println!("{}", entry.cause);
    println!();
    println!("Fix: {}", entry.fix);
    if !entry.alternative.is_empty() {
        println!("Alternative: {}", entry.alternative);
    }
    println!("Recovery:");
    for line in entry.recovery {
        println!("  {line}");
    }
    Ok(())
}
