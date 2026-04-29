# P76 Triage Table — SURPRISES-INTAKE.md (3 LOW entries)

Per CONTEXT.md D-01 (triage-by-severity-then-execute) and D-07 (executor mode for ≤5 LOW entries).
All 3 entries are severity LOW; executor mode is the right shape.

| Entry | Severity | Disposition | Rationale | Action commit (Wave 2) |
|-------|----------|-------------|-----------|------------------------|
| 1a polish-03-mermaid-render | LOW | REBIND | `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:85` still describes mermaid-render hygiene; claim text matches the line verbatim. Stored `source_hash=c88cd0f9…` is stale; current bytes hash differently. Rebind preserves citation, refreshes hash. | `fix(p76): RESOLVED entry-1a` |
| 1b cli-subcommand-surface | LOW | REBIND | `crates/reposix-cli/src/main.rs:37` still opens `enum Cmd { ... Sim { ...` and `:299` still ends `Version,` block (file is 420 lines, range 37-299 still anchors the subcommand surface). Subcommand list (init|sim|list|refresh|spaces|log|history|tokens|cost|gc|doctor|version) is unchanged. Stored `source_hash=b9700827…` drifted under the 263-line range; rebind refreshes. | `fix(p76): RESOLVED entry-1b` |
| 2 linkedin Source::Single | LOW | RESOLVED (annotation only) | P75 commit `9e07028` already healed the row: live catalog confirms `last_verdict == BOUND`, `source_hash == 7a1d7a4e…` (matches P75 SUMMARY's heal-bind narrative). No code change, no row mutation in P76 — pure audit-trail update. | `fix(p76): RESOLVED entry-2` (annotation-only) |
| 3 connector-matrix synonym | LOW | WONTFIX + new GOOD-TO-HAVE | The connector-matrix regex widening (P74 commit `c8e4111`: `[Cc]onnector` → `[Cc]onnector\|[Bb]ackend`) is a complete fix — the verifier asserts the failure mode P74 cared about ("matrix accidentally deleted from landing") against the live heading "What each backend can do." Renaming the heading to "Connector capability matrix" to literal-match the catalog row claim text is purely cosmetic; filed as a P77 GOOD-TO-HAVE (size XS, impact clarity). | `fix(p76): WONTFIX entry-3` + GOOD-TO-HAVES.md filing |

## Inline `sed` evidence (entry 1)

### Row 1a: `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:85`

```
- [shipped] **POLISH-03**: All mermaid diagrams render without console errors on the live site (Phase 52, F1+F2+F3 from `.planning/research/v0.11.0/mkdocs-site-audit.md`).
```

Catalog claim text: "All mermaid diagrams render without console errors on the live site"

**Match: yes (verbatim).** Decision: **REBIND**.

### Row 1b: `crates/reposix-cli/src/main.rs:37-299`

`sed -n '37,40p'`:
```
#[derive(Debug, Subcommand)]
enum Cmd {
    /// Run the in-process REST simulator (delegates to `reposix-sim`).
    Sim {
```

`sed -n '295,305p'`:
```
        chars_per_token: Option<f64>,
    },
    /// Print the version.
    Version,
}

#[tokio::main]
// Top-level dispatcher; line count grows linearly with subcommand count
// and the match-on-clap-derive structure does not benefit from extraction.
#[allow(clippy::too_many_lines)]
async fn main() -> Result<()> {
```

Catalog claim text: "CLI subcommand surface (init|sim|list|refresh|spaces|log|history|tokens|cost|gc|doctor|version) is locked under semver"

**Match: yes — `enum Cmd` opens at 37, closes at 299 (line 299 is `}` after `Version,`); the cited 12-subcommand list is exhaustively enumerated within. The end-of-range needs a 1-line tweak to point at the closing `}` properly: lines 37–299 already anchor the enum precisely (line 299 is the closing brace; the empty line + `#[tokio::main]` start at 300+).**

Decision: **REBIND** with the same 37-299 range.

## Live catalog state pre-action

```
polish-03-mermaid-render        → last_verdict=STALE_DOCS_DRIFT  source_hash=c88cd0f9…
cli-subcommand-surface          → last_verdict=STALE_DOCS_DRIFT  source_hash=b9700827…
linkedin/token-reduction-92pct  → last_verdict=BOUND             source_hash=7a1d7a4e…  (already healed by P75 9e07028)
```

## Honesty check (D-09 pre-flight)

P76 is itself NOT permitted to populate SURPRISES-INTAKE.md (forbidden recursion per D-09).
Any new findings during P76 execution will append to GOOD-TO-HAVES.md instead. As of triage,
no NEW findings observed beyond the planned scope.
