# Phase 89 — Discussion Log

**Date:** 2026-05-08
**Mode:** autonomous (per owner directive)
**Subagent:** discuss-phase research subagent (single pass)

## Overall Disposition

Discussion ran in autonomous mode per owner directive (2026-05-08 session). No gray areas escalated to user — all six REQ-IDs were already locked by `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md` § P89 + § F-K1..K8 + COMPLETENESS-CHECK § Decision 3, and re-affirmed verbatim in `.planning/milestones/v0.13.0-phases/ROADMAP.md` § Phase 89.

Implementation defaults captured in `89-CONTEXT.md` § `<decisions>`. The planner consumes those defaults; they may be overridden if research surfaces evidence (each carries a citation to its canonical source).

## Per-REQ-ID — default chosen + 1-line rationale

| REQ-ID | Default chosen | 1-line rationale |
|---|---|---|
| **RBF-FW-01** | Extend `VALID_CADENCES` tuple in `quality/runners/run.py:45`; gate via env-check in `run_row()` mirroring the existing `is_stale()` short-circuit pattern; CI auto-skips because creds aren't set | Single-edit + mirrors established short-circuit pattern; no architectural rewrite of runner |
| **RBF-FW-02** | New `kind` enum value in `quality/catalogs/README.md` schema; verifier scripts use existing `.sh` dispatch path; transcript artifact at `quality/reports/transcripts/<row-slug>-<ts>.txt` with env KEYS (not values) | Kind is a convention enforced by schema + verifier subagent grading, not by runner subprocess shape; security-first transcript per CLAUDE.md threat model |
| **RBF-FW-03** | Ship BOTH a verdict-template entry AND a catalog row at `agent-ux/milestone-close-vision-litmus-real-backend`; verifier script exists at P89 close but legitimately NOT-VERIFIED until P91+P92+P93+P94+P95 land substrate | Catalog row makes the probe a runtime artifact (Decision 3 + COMPLETENESS-CHECK S2); SLOT pattern explicitly defends against PATTERNS C7 self-licensing-deferral-loop |
| **RBF-FW-04** | NEW script `quality/gates/structure/banned-production-tokens.sh` (NOT extension of `scripts/banned-words-lint.sh` which is `docs/`-only); pattern `\bP[0-9]+-[0-9]+\b`; excludes `tests/` + allowlist-marker lines | Existing layered docs-banned-words model breaks if extended to `crates/`; sibling script preserves both scopes |
| **RBF-FW-05** | New `quality/gates/structure/deferral-pointer-linter.sh`; greps three patterns over `crates/`; cross-references named-phase number against `.planning/phases/N-*/PLAN*.md` EXISTENCE only (content cross-ref deferred to P95 polish) | Existence check covers the dominant failure mode (named phase doesn't exist) without over-engineering on first pass |
| **RBF-FW-11** | New required schema field `claim_vs_assertion_audit` (string ≥50 chars); runner cross-check date-gated on rows minted ≥ 2026-05-08 (legacy 388 rows continue to validate; backfill is P95 RBF-D-06); `claim_vs_assertion_audit_hash` artifact field for forensic drift detection | Date-cutoff prevents breaking legacy rows; hash field future-proofs adversarial pass (P90 RBF-FW-12) |

## Findings surfaced during scout (informed defaults)

1. **`scripts/banned-words-lint.sh` is `docs/`-only with a layered model.** Extending it to `crates/` would conflate scopes. Drives D-04a (NEW script).
2. **`quality/runners/run.py` is at 376 lines vs ≤350 anti-bloat cap.** RBF-FW-01 + RBF-FW-11 will push it further; default factoring into `_realbackend.py` + `_audit_field.py` siblings per the established `_freshness.py` precedent at line 34.
3. **Runner has no exit-code → NOT-VERIFIED mapping today** (verifier scripts must write the artifact directly with status NOT-VERIFIED). Drives Q-EXIT-1 + the worked-example shape in `<specifics>`.
4. **`tag-v0.13.0.sh` is currently `.disabled`** per Path A/Option B (v0.13.0 tag is held). P89 documents the future 9th-probe guard contract in PROTOCOL.md so P97's plan-checker catches it; P89 does NOT modify the tag script (out-of-phase scope).
5. **CLAUDE.md "9 dimensions" does NOT list `framework`** as a dimension. ROADMAP P89 SC #5 mentions `quality/catalogs/{agent-ux,framework}.json` literally — interpreted as conventional naming, not contractual. Default avoids creating a new dimension; planner can override (Q-CAT-1).
6. **Existing milestone-close verdict format** at `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` is an 8-probe table (re-verification context). The 9th probe extension is shape-compatible.
7. **CLUSTER C7 ("self-licensing-deferral-loop") risk on P89 itself** per COMPLETENESS-CHECK S1 — explicitly defended against in D-03c (the SLOT pattern: `blast_radius: P0` + verifier script EXISTS but returns NOT-VERIFIED + no WAIVER mechanism in P89 close).

## Blockers / escalations

None. Owner directive was "run autonomously and surface only crucial decisions." No decisions met that bar — every REQ-ID was already locked, and every implementation default has a citation to a canonical doc.

## Handoff to planner

`89-CONTEXT.md` is complete and self-sufficient for `gsd-planner` to produce `89-PLAN-OVERVIEW.md` + per-task `89-PLAN-T<N>.md` files without further user input. The 8 open questions in `<open_questions>` are research-validation items, not user-decision items.

---

*End of discussion log.*
