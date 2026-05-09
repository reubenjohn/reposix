# v0.13.2 Surprises Intake (P107 source-of-truth)

> **Append-only intake for surprises discovered during P98–P106 execution.**
> Each entry is something the discovering phase chose NOT to fix eagerly because it was massively out-of-scope. P107 drains this file (per CLAUDE.md OP-8 — the +2 reservation phase absorbs both surprises and good-to-haves duties at v0.13.2's close).
>
> **Eager-resolution preference:** if a surprise can be closed inside its discovering phase without doubling the phase's scope (rough heuristic: < 1 hour incremental work, no new dependency introduced, no new file created outside the phase's planned set), do it there. The intake file is for items that genuinely don't fit.
>
> **Distinction from `GOOD-TO-HAVES.md`:** entries here fix something that's BROKEN, RISKY, or BLOCKING. Improvements/polish go in `GOOD-TO-HAVES.md` (also drained by P107).

## Entry format

```markdown
## YYYY-MM-DD HH:MM | discovered-by: P<N> | severity: BLOCKER|HIGH|MEDIUM|LOW

**What:** One-paragraph description of what was found.

**Why out-of-scope for P<N>:** Why eager-resolution wasn't possible (scope, time, dependency).

**Sketched resolution:** One paragraph proposing how P107 should resolve.

**STATUS:** OPEN  (← P107 updates to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA)
```

---

## Entries

_No entries yet — phases P98–P106 have not started execution. Append below this line as surprises surface._
