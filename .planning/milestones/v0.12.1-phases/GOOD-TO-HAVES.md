# v0.12.1 Good-to-haves Intake (P77 source-of-truth)

> **Append-only intake for nice-to-haves discovered during P72-P76 execution.**
> Each entry is an improvement that would help the next maintainer but doesn't fix a bug. P77 drains this file (XS first, then S; M items defer to v0.13.0).
>
> **Eager-resolution preference:** if it's genuinely XS (< 15 min) and scope-local, fold into the discovering phase. The intake captures items that need their own focused attention.
>
> **Distinction from `SURPRISES-INTAKE.md`:** entries here are POLISH or CLARITY improvements. Things that are broken, risky, or blocking go in `SURPRISES-INTAKE.md` (P76).

## Entry format

```markdown
## discovered-by: P<N> | size: XS|S|M | impact: clarity|perf|consistency|grounding

**What:** One-paragraph description.

**Proposed fix:** One line.

**STATUS:** OPEN  (← P77 updates to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA)
```

## Size labels

- **XS** — 5-15 min: typo, error message clarification, single-file cross-ref, comment-only update.
- **S** — 15-60 min: helper extraction, test consolidation, single-file refactor, doc cross-ref sweep on one page.
- **M** — 1-3 hours: multi-file refactor, naming sweep, doc reorganization. Default DEFERRED to v0.13.0.

---

## Entries

## discovered-by: P74 | size: XS | impact: clarity

**What:** `docs/index.md:95` heading reads "## What each backend can do" — synonym for the catalog row claim "Connector capability matrix added to landing page" (row `polish2-06-landing`). P74 widened the verifier regex to accept either noun (`[Cc]onnector|[Bb]ackend`) per the eager-fix decision logged in P74's SUMMARY (commit c8e4111). Renaming the heading to "## Connector capability matrix" would let the verifier regex narrow back to a literal claim+heading match — a small clarity win, no behaviour change.

**Proposed fix:** Edit `docs/index.md:95` heading to `## Connector capability matrix`. Optionally narrow `quality/gates/docs-alignment/connector-matrix-on-landing.sh` regex back to `[Cc]onnector` (single-noun match) once the rename lands.

**STATUS:** OPEN
