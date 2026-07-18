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
//! <headline> [RPX-xxxx]          // ` [RPX-xxxx]` appended to the FIRST headline
//! Fix: <fix>                     //   line only when a code is set (FLAG 2)
//! Alternative: <alternative>      // OMITTED when the alternative is unset/empty (FLAG 1)
//! Recovery:                       // OMITTED when the recovery slice is empty
//!   <recovery[0]>
//!   <recovery[1]>
//! Explain: reposix explain RPX-xxxx  // trailing limb, only when a code is set (FLAG 2)
//! ```
//!
//! # Two baked resolutions
//!
//! - **FLAG 1 — empty alternative suppresses its line.** An error with no genuine
//!   alternative (e.g. an unexpected-EOF protocol desync) passes `""` and no hollow
//!   `Alternative:` line is emitted. The recovery slice is likewise suppressible.
//! - **FLAG 2 — the `.code()` render (P121).** [`Teach::code`] attaches a stable
//!   `RPX-xxxx` error code. When set, the code renders as a ` [RPX-xxxx]` tag on the
//!   headline's FIRST line and an `Explain: reposix explain RPX-xxxx` nudge trailing
//!   the body — the codified half of the Rust-compiler-grade UX north star. When
//!   UNSET the output is byte-identical to the no-code shape, so the ~40 call sites
//!   that never carry a code are untouched. Only the static code id is interpolated
//!   into these limbs — never a headline-derived or remote byte (OP-2). The extended
//!   explanation for each code lives once in [`crate::codes`]; this render carries
//!   only the id + the `explain` nudge, not the extended prose.

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
    /// P121 error-code slot (FLAG 2): renders a ` [RPX-xxxx]` headline tag + an
    /// `Explain:` nudge when `Some`; `None` is byte-identical to no code.
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

    /// Attach a stable `RPX-xxxx` error code (FLAG 2 / P121). When set, the code
    /// renders as a ` [RPX-xxxx]` tag on the headline's first line plus a trailing
    /// `Explain: reposix explain RPX-xxxx` limb; an UNSET code is byte-identical to
    /// no code. Prefer a `reposix_core::codes::ids` const (typo-proof) over a raw
    /// literal — the `agent-ux/rpx-codes-registry` gate cross-checks every emitted
    /// code against the [`crate::codes`] registry.
    #[must_use]
    pub fn code(mut self, code: &'a str) -> Self {
        self.code = Some(code);
        self
    }
}

impl std::fmt::Display for Teach<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Headline. When a code is set, the ` [RPX-xxxx]` tag rides the FIRST line
        // of the headline (never dangling on the last line of a multi-line headline
        // such as cache_build_error's `…\n(underlying: …)`). Only the static
        // `self.code` id is interpolated — never a headline-derived or remote byte
        // (OP-2 / T-121-01: the code + explain paths are static-only).
        match self.code {
            Some(code) => match self.headline.split_once('\n') {
                Some((first, rest)) => write!(f, "{first} [{code}]\n{rest}")?,
                None => write!(f, "{} [{code}]", self.headline)?,
            },
            None => f.write_str(self.headline)?,
        }
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
        // FLAG 2 / P121: the `Explain: reposix explain <code>` nudge — the codified
        // half of the north star, mirroring `rustc`'s pointer to `--explain`. It is
        // appended AFTER the recovery block, and only when a code is set; an unset
        // code leaves the output byte-identical to P120 (the tag above is likewise
        // gated on `self.code`). Static-only interpolation (OP-2).
        if let Some(code) = self.code {
            write!(f, "\nExplain: reposix explain {code}")?;
        }
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

/// Convenience wrapper for a CODED teaching error — the P121 sibling of [`teach`]
/// with a leading `RPX-xxxx` code. Renders identically to
/// `Teach::new(headline).fix(fix).alternative(alternative).recovery(recovery).code(code)`:
/// the ` [code]` tag rides the headline's first line and an
/// `Explain: reposix explain <code>` limb trails the body. Most coded call sites
/// use this for minimal churn over the P120 `teach(...)` shape. Prefer a
/// [`crate::codes::ids`] const for `code` (typo-proof) over a raw literal.
///
/// ```
/// use reposix_core::errmsg::teach_coded;
/// let msg = teach_coded(
///     "RPX-0001",
///     "invalid backend spec `foo`",
///     "a spec is `<backend>::<project>`",
///     "start with the simulator: sim::demo",
///     &["reposix init sim::demo /tmp/demo"],
/// );
/// assert!(msg.starts_with("invalid backend spec `foo` [RPX-0001]\n"));
/// assert!(msg.ends_with("Explain: reposix explain RPX-0001"));
/// ```
#[must_use]
pub fn teach_coded(
    code: &str,
    headline: &str,
    fix: &str,
    alternative: &str,
    recovery: &[&str],
) -> String {
    Teach::new(headline)
        .fix(fix)
        .alternative(alternative)
        .recovery(recovery)
        .code(code)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{teach, teach_coded, Teach};

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
    fn code_renders_tag_and_nudge() {
        // FLAG 2 / P121 (replaces the P120 `code_slot_renders_nothing` pin, now
        // false): a set code appends ` [RPX-xxxx]` to the headline's FIRST line and
        // a trailing `Explain:` limb — the codified north-star touch.
        let coded = Teach::new("hd")
            .fix("f")
            .recovery(&["r"])
            .code("RPX-0001")
            .to_string();
        assert_eq!(
            coded,
            "hd [RPX-0001]\nFix: f\nRecovery:\n  r\nExplain: reposix explain RPX-0001"
        );

        // The tag rides the FIRST line of a MULTI-LINE headline (never dangling on
        // the last line), e.g. cache_build_error's `<headline>\n(underlying: …)`.
        let multiline = Teach::new("could not sync\n(underlying: boom)")
            .fix("f")
            .code("RPX-0201")
            .to_string();
        assert_eq!(
            multiline,
            "could not sync [RPX-0201]\n(underlying: boom)\nFix: f\nExplain: reposix explain RPX-0201"
        );

        // An UNSET code still renders byte-identical to no code — the surviving half
        // of the old pinning invariant (backward-compat for the ~40 uncoded sites).
        let uncoded = Teach::new("hd").fix("f").recovery(&["r"]).to_string();
        assert_eq!(uncoded, "hd\nFix: f\nRecovery:\n  r");
        assert!(!uncoded.contains("RPX-"), "no code → no tag: {uncoded}");
        assert!(
            !uncoded.contains("Explain:"),
            "no code → no nudge: {uncoded}"
        );

        // `teach_coded` renders identically to the builder with `.code(...)`.
        assert_eq!(teach_coded("RPX-0001", "hd", "f", "", &["r"]), coded);
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
