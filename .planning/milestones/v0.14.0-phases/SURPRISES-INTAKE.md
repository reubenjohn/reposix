# v0.14.0 Surprises Intake (P110 source-of-truth)

> **Append-only intake for surprises discovered during P102–P109 execution.**
> Each entry is something the discovering phase chose NOT to fix eagerly because it was
> massively out-of-scope. P110 drains this file (per CLAUDE.md OP-8 — Slot 1 of the v0.14.0
> +2 reservation).
>
> **Eager-resolution preference:** if a surprise can be closed inside its discovering
> phase without doubling the phase's scope (rough heuristic: < 1 hour incremental work, no
> new dependency introduced, no new file created outside the phase's planned set), do it
> there. The intake file is for items that genuinely don't fit.
>
> **Distinction from `GOOD-TO-HAVES.md`:** entries here fix something that's BROKEN,
> RISKY, or BLOCKING. Improvements/polish go in `GOOD-TO-HAVES.md` (drained by P111, Slot
> 2).

## Entry format

```markdown
## YYYY-MM-DD HH:MM | discovered-by: P<N> | severity: BLOCKER|HIGH|MEDIUM|LOW

**What:** One-paragraph description of what was found.

**Why out-of-scope for P<N>:** Why eager-resolution wasn't possible (scope, time, dependency).

**Sketched resolution:** One paragraph proposing how P110 should resolve.

**STATUS:** OPEN  (← P110 updates to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA)
```

---

## Entries

_No entries yet — phases P102–P109 have not started execution. Append below this line as
surprises surface._
