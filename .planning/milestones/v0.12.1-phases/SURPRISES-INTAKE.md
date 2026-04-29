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

**STATUS:** OPEN
