//! Rust-compiler-grade, 3-part teaching-error bodies (Phase 120 / P120).
//!
//! Every user-facing `reposix` CLI subcommand error and every `reposix-remote`
//! git-helper error is meant to meet the same bar the project already claims in
//! `crates/CLAUDE.md` and demonstrates in
//! `reposix-cli/src/init.rs::refuse_existing_repo_root`:
//!
//! 1. **teach the fix** (`Fix: …`),
//! 2. **suggest the alternative** (`Alternative: …`, omitted when there is none),
//! 3. **give a copy-paste recovery command** (an indented `Recovery:` block).
//!
//! Rather than hand-roll ~40 prose strings, both crates route their error sites
//! through the [`Teach`] builder (or the [`teach`] convenience fn). The rendered
//! SHAPE is a mechanical contract — `quality/gates/agent-ux/teach_scan.py` and the
//! `errors_teach_recovery` integration suites grep for the `Fix:` / `Recovery:`
//! anchors — so it must stay stable.
//!
//! This module is PURE string formatting: no `anyhow` (callers wrap with
//! `bail!`/`anyhow!` at the binary boundary, per `crates/CLAUDE.md`), no new
//! dependency, `forbid(unsafe_code)`-clean.
//!
//! # Rendered shape
//!
//! ```text
//! <headline>
//! Fix: <fix>
//! Alternative: <alternative>      // OMITTED when the alternative is unset/empty (FLAG 1)
//! Recovery:                       // OMITTED when the recovery slice is empty
//!   <recovery[0]>
//!   <recovery[1]>
//! ```
//!
//! # Two baked resolutions
//!
//! - **FLAG 1 — empty alternative suppresses its line.** An error with no genuine
//!   alternative (e.g. an unexpected-EOF protocol desync) passes `""` and no hollow
//!   `Alternative:` line is emitted. The recovery slice is likewise suppressible.
//! - **FLAG 2 — a forward-compat `.code()` slot.** [`Teach::code`] is wired NOW and
//!   renders NOTHING this phase. P121 will own the `RPX-xxxx` render format and add
//!   `.code(...)` only to the individual sites that need a code — the other ~40 call
//!   sites are untouched by that change. [`tests::code_slot_renders_nothing`] pins the
//!   current byte-output so P121's addition is a visible, reviewed diff.

/// Return `s` only when it is `Some` and non-empty; used to suppress the
/// `Fix:` / `Alternative:` lines for empty inputs (FLAG 1).
fn non_empty(s: Option<&str>) -> Option<&str> {
    s.filter(|v| !v.is_empty())
}

/// Builder for a Rust-compiler-grade, 3-part teaching-error body.
///
/// The rendered shape (see the [module docs](self)) is a contract the agent-ux
/// gate and the integration tests grep for. Construct with [`Teach::new`], layer
/// on `.fix()` / `.alternative()` / `.recovery()` (and, for P121, `.code()`), then
/// render via [`Display`](std::fmt::Display) — typically inside a `bail!("{}", …)`:
///
/// ```
/// use reposix_core::errmsg::Teach;
/// let msg = Teach::new("the widget is jammed")
///     .fix("clear the jam and re-seat the widget")
///     .recovery(&["reposix widget --reset"])
///     .to_string();
/// assert!(msg.contains("Fix:"));
/// assert!(msg.contains("Recovery:"));
/// // No `.alternative()` → no `Alternative:` line (FLAG 1).
/// assert!(!msg.contains("Alternative:"));
/// ```
#[derive(Debug, Clone)]
pub struct Teach<'a> {
    headline: &'a str,
    fix: Option<&'a str>,
    /// `None` (or an empty `.alternative("")`) omits the `Alternative:` line (FLAG 1).
    alternative: Option<&'a str>,
    recovery: &'a [&'a str],
    /// P121 error-code slot (FLAG 2): wired now, renders nothing this phase.
    code: Option<&'a str>,
}

impl<'a> Teach<'a> {
    /// Start a teaching-error body with its headline (the one-line summary of
    /// what went wrong). Layer `.fix()` / `.alternative()` / `.recovery()` on top.
    #[must_use]
    pub fn new(headline: &'a str) -> Self {
        Self {
            headline,
            fix: None,
            alternative: None,
            recovery: &[],
            code: None,
        }
    }

    /// Set the `Fix: …` line — what the user should change. An empty `fix`
    /// suppresses the line.
    #[must_use]
    pub fn fix(mut self, fix: &'a str) -> Self {
        self.fix = Some(fix);
        self
    }

    /// Set the `Alternative: …` line — a different command/approach that also
    /// solves the user's goal. An empty `alt` (`""`) is treated as UNSET and omits
    /// the line entirely (FLAG 1) — use it for errors with no genuine alternative.
    #[must_use]
    pub fn alternative(mut self, alt: &'a str) -> Self {
        self.alternative = Some(alt);
        self
    }

    /// Set the `Recovery:` block — one or more copy-paste-runnable command lines,
    /// each rendered 2-space-indented. An empty slice omits the block.
    #[must_use]
    pub fn recovery(mut self, recovery: &'a [&'a str]) -> Self {
        self.recovery = recovery;
        self
    }

    /// Attach a forward-compat error code (FLAG 2 / P121 slot). Wired now, renders
    /// NOTHING this phase: a `.code("RPX-0001")` body is byte-identical to no code.
    /// P121 owns the render format and changes only [`Display`](std::fmt::Display).
    #[must_use]
    pub fn code(mut self, code: &'a str) -> Self {
        self.code = Some(code);
        self
    }
}

impl std::fmt::Display for Teach<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.headline)?;
        if let Some(fix) = non_empty(self.fix) {
            write!(f, "\nFix: {fix}")?;
        }
        if let Some(alt) = non_empty(self.alternative) {
            write!(f, "\nAlternative: {alt}")?;
        }
        if !self.recovery.is_empty() {
            f.write_str("\nRecovery:")?;
            for cmd in self.recovery {
                write!(f, "\n  {cmd}")?;
            }
        }
        // FLAG 2 / P121 forward-compat: the error-code slot (`self.code`) is WIRED
        // but renders NOTHING this phase — a `.code("RPX-0001")` body is
        // byte-identical to no code (pinned by `tests::code_slot_renders_nothing`).
        // P121 owns the `RPX-xxxx` render format and adds its render HERE, e.g.
        // `if let Some(code) = self.code { write!(f, " [{code}]")?; }`, leaving the
        // ~40 call sites untouched. The field stays live (no dead_code) via the
        // derived `Debug` impl, which reads every field.
        Ok(())
    }
}

/// Convenience wrapper for the common `(headline, fix, alternative, recovery)`
/// teaching shape — most of the ~40 call sites use this fn. An EMPTY `alternative`
/// (`""`) omits the `Alternative:` line (FLAG 1); an empty `recovery` slice omits
/// the `Recovery:` block. Delegates to [`Teach`]; when a site needs the P121
/// `.code(...)` slot, build with [`Teach`] directly.
///
/// ```
/// use reposix_core::errmsg::teach;
/// let msg = teach(
///     "spec `foo` is malformed",
///     "specs are `<backend>::<project>`",
///     "start with the simulator: sim::demo",
///     &["reposix init sim::demo /tmp/demo"],
/// );
/// assert_eq!(
///     msg,
///     "spec `foo` is malformed\n\
///      Fix: specs are `<backend>::<project>`\n\
///      Alternative: start with the simulator: sim::demo\n\
///      Recovery:\n  reposix init sim::demo /tmp/demo"
/// );
/// ```
#[must_use]
pub fn teach(headline: &str, fix: &str, alternative: &str, recovery: &[&str]) -> String {
    Teach::new(headline)
        .fix(fix)
        .alternative(alternative)
        .recovery(recovery)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{teach, Teach};

    #[test]
    fn full_three_part_render_is_byte_exact() {
        let got = teach(
            "the sim at `127.0.0.1:7878` is unreachable",
            "start the simulator, then re-run",
            "point --origin at the right host/port",
            &["reposix sim", "reposix list --origin http://127.0.0.1:7878"],
        );
        assert_eq!(
            got,
            "the sim at `127.0.0.1:7878` is unreachable\n\
             Fix: start the simulator, then re-run\n\
             Alternative: point --origin at the right host/port\n\
             Recovery:\n  reposix sim\n  reposix list --origin http://127.0.0.1:7878"
        );
        // Greppable anchors the gate + integration suites key on.
        assert!(got.contains("Fix:") && got.contains("Alternative:") && got.contains("Recovery:"));
    }

    #[test]
    fn recovery_lines_are_two_space_indented() {
        let got = teach("hd", "f", "a", &["one", "two"]);
        assert!(got.contains("\n  one\n  two"), "got:\n{got}");
    }

    #[test]
    fn empty_alternative_suppresses_the_line() {
        // FLAG 1: an error with no genuine alternative emits NO hollow line.
        let got = teach("hd", "do the fix", "", &["run this"]);
        assert_eq!(got, "hd\nFix: do the fix\nRecovery:\n  run this");
        assert!(
            !got.contains("Alternative:"),
            "empty alt must omit the line: {got}"
        );
    }

    #[test]
    fn empty_recovery_omits_the_block_no_dangling_header() {
        // headline + Fix + Alternative but NO dangling `Recovery:` with no command.
        let got = teach("hd", "do the fix", "or do that", &[]);
        assert_eq!(got, "hd\nFix: do the fix\nAlternative: or do that");
        assert!(
            !got.contains("Recovery:"),
            "empty recovery must omit the header: {got}"
        );
    }

    #[test]
    fn builder_without_alternative_matches_empty_alternative() {
        // `Teach::new(hd).fix(f).recovery(&[r])` (no `.alternative`) == empty alt.
        let builder = Teach::new("hd").fix("f").recovery(&["r"]).to_string();
        let free_fn = teach("hd", "f", "", &["r"]);
        assert_eq!(builder, free_fn);
        assert_eq!(builder, "hd\nFix: f\nRecovery:\n  r");
    }

    #[test]
    fn code_slot_renders_nothing() {
        // FLAG 2 / P121: the `.code(...)` slot is wired but renders NOTHING this
        // phase — byte-identical to no code. This pins the baseline so P121's
        // render is a visible, reviewed diff (not a silent behavior change).
        let with_code = Teach::new("hd")
            .fix("f")
            .recovery(&["r"])
            .code("RPX-0001")
            .to_string();
        let without = Teach::new("hd").fix("f").recovery(&["r"]).to_string();
        assert_eq!(
            with_code, without,
            "code slot must render nothing this phase"
        );
        assert!(
            !with_code.contains("RPX-0001"),
            "code must not leak into output yet: {with_code}"
        );
    }

    #[test]
    fn headline_only_is_just_the_headline() {
        assert_eq!(Teach::new("just a headline").to_string(), "just a headline");
    }

    #[test]
    fn empty_fix_is_suppressed() {
        // A builder with an empty fix omits the Fix line (symmetry with alt).
        let got = Teach::new("hd").fix("").recovery(&["r"]).to_string();
        assert_eq!(got, "hd\nRecovery:\n  r");
    }
}
