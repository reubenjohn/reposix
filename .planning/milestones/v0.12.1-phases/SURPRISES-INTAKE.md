# v0.12.1 Surprises Intake (P76 source-of-truth)

> **Append-only intake for surprises discovered during P72-P75 execution.**
> Each entry is something the discovering phase chose NOT to fix eagerly because it was massively out-of-scope. P76 drains this file.
>
> **Eager-resolution preference:** if a surprise can be closed inside its discovering phase without doubling the phase's scope (rough heuristic: < 1 hour incremental work, no new dependency introduced, no new file created outside the phase's planned set), do it there. The intake file is for items that genuinely don't fit.
>
> **Distinction from `GOOD-TO-HAVES.md`:** entries here fix something that's BROKEN, RISKY, or BLOCKING. Improvements/polish go in `GOOD-TO-HAVES.md` (P77).

## Entry format

```markdown
## YYYY-MM-DD HH:MM | discovered-by: P<N> | severity: BLOCKER|HIGH|MEDIUM|LOW

**What:** One-paragraph description of what was found.

**Why out-of-scope for P<N>:** Why eager-resolution wasn't possible (scope, time, dependency).

**Sketched resolution:** One paragraph proposing how P76 should resolve.

**STATUS:** OPEN  (← P76 updates to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA)
```

---

## Entries

## 2026-04-29 19:58 | discovered-by: P72 | severity: LOW

**What:** Two pre-existing BOUND rows flipped to `STALE_DOCS_DRIFT` during the P72 post-bind `doc-alignment walk`. The walker hashes prose AND test bodies; these two rows have drift NOT caused by P72:
- `planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish-03-mermaid-render` cites `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:85` (untouched by P72).
- `docs/decisions/009-stability-commitment/cli-subcommand-surface` cites `crates/reposix-cli/src/main.rs:37-299` (untouched by P72).

Both rows surfaced now because P72 was the first walk after their referenced source drift landed. Net effect on summary: +9 P72 rows BOUND but only +7 net `claims_bound` — 2 absorbed by the new STALE_DOCS_DRIFT state.

**Why out-of-scope for P72:** Both rows cite files outside P72's planned modification set (P72 modifies only README.md, docs/development/contributing.md, CLAUDE.md, and quality/* artifacts). Refreshing the source/test hashes for these rows requires recomputing claims and possibly re-grading — not an in-place hash update. SCOPE BOUNDARY honoured per executor.md.

**Sketched resolution:** P76 runs `/reposix-quality-refresh .planning/milestones/v0.11.0-phases/REQUIREMENTS.md` and `/reposix-quality-refresh docs/decisions/009-stability-commitment.md` (or rebinds the cli-subcommand-surface row against the current `crates/reposix-cli/src/main.rs` shape). If either claim has genuinely diverged from current code, propose-retire or rebind with updated source-line range. Ship in a single P76 commit.

**STATUS:** RESOLVED | row 1a: 0467373 (rebind, source_hash c88cd0f9 -> 6ec37650, last_verdict STALE_DOCS_DRIFT -> BOUND) | row 1b: fbc3caa (rebind, source_hash b9700827 -> 89b925f5, last_verdict STALE_DOCS_DRIFT -> BOUND). Both claims verified against live source via `sed`; both still describe current code verbatim. No propose-retire needed. Live walk post-action shows zero STALE_DOCS_DRIFT rows in catalog.

---

## 2026-04-29 20:55 | discovered-by: P74 | severity: LOW

**What:** P74's PROSE-FIX-01 edited `docs/social/linkedin.md:21` (FUSE filesystem -> git-native partial clone). The existing BOUND row `docs/social/linkedin/token-reduction-92pct` at line 21 (`Source::Single`) tipped to `STALE_DOCS_DRIFT` on the post-edit `walk`. A second `walk` did NOT auto-heal the row — `last_verdict` remained `STALE_DOCS_DRIFT` even though `source_hash` was refreshed to `1a19b86e19e7b9730b93fef81cc4ba09fe1338052f157a4ec83fa5c988f11476`. CONTEXT.md D-08 predicted "transient STALE_DOCS_DRIFT then heals on the next walk" — that didn't happen. This is consistent with the P75 hash-overwrite bug noted in HANDOVER §4 (the bug is described there as `Source::Multi`-specific but appears to surface on `Source::Single` rows too).

**Why out-of-scope for P74:** P75 is scoped specifically to fix the walker's hash/state machine bug. Fixing it inside P74 would mean editing `crates/reposix-quality/src/commands/doc_alignment.rs` walker code, which is exactly P75's planned modification set. SCOPE BOUNDARY honoured.

**Sketched resolution:** P75 fixes the walker so a second `walk` after `source_hash` refresh transitions `STALE_DOCS_DRIFT` -> `BOUND` when the bound test still passes. After P75 ships, this row should heal on the next walk without a fresh bind. If P75's fix doesn't cover `Source::Single`, broaden the fix scope.

**STATUS:** RESOLVED | healed by P75 commit 9e07028 (verbs::bind hash-overwrite fix landed in 69a30b0; an explicit re-bind at 9e07028 refreshed source_hash STALE_DOCS_DRIFT(1a19b86e…) -> BOUND(7a1d7a4e…)). P75 SUMMARY clarified the procedural finding: walks NEVER auto-refresh source_hash — only binds do, by design (walker docstring at crates/reposix-quality/src/commands/doc_alignment.rs:802-804). The "didn't auto-heal on second walk" observation was confirmed-not-a-bug; the heal path is `bind`. P76 confirms via live catalog query: `jq '.rows[] | select(.id == "docs/social/linkedin/token-reduction-92pct") | .last_verdict' == "BOUND"`. No code change required in P76; pure audit-trail update per CLAUDE.md OP-3.

---

## 2026-04-29 20:56 | discovered-by: P74 | severity: LOW

**What:** P74's `connector-matrix-on-landing.sh` verifier (D-06) was widened from CONTEXT.md's literal `^## .*[Cc]onnector` regex to `^## .*([Cc]onnector|[Bb]ackend)`. The actual heading on `docs/index.md:95` reads "## What each backend can do" (no "connector" word) — same matrix table at lines 102-107, synonym mismatch only. The capability matrix IS on landing; the noun differs from the catalog row's claim text ("Connector capability matrix added to landing page"). Widening the regex preserves the failure-mode the verifier cares about ("matrix accidentally deleted from landing") while being honest about live prose.

**Why out-of-scope for P74:** This is a Rule-2 micro-deviation eager-fixed inside the verifier (per OP-8: < 5 min, no new dep). Logged here for traceability so a future agent doesn't re-investigate.

**Sketched resolution:** Either (a) keep the widened regex as-is (current state — works), or (b) rename the heading to "## Connector capability matrix" in a future docs polish phase to make claim+heading literal-match. P77 GOOD-TO-HAVES candidate.

**STATUS:** OPEN
