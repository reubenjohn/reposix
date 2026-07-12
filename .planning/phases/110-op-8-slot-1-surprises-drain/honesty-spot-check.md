# Absorption Honesty Spot-Check — v0.14.0

**Spot-check author:** P110 drain executor — a dispatched GSD leaf subagent, spawned by the
v0.14.0 milestone coordinator with a single drain charter. **I am NOT the milestone
orchestrator** (clause b): I did not route or execute P102–P109; I received a fresh charter to
drain `SURPRISES-INTAKE.md` and grade honesty from committed artifacts only. This satisfies the
F-K5 clause-(b) process rule (`quality/dispatch/absorption-honesty-spot-check.md`): the v0.13.0
F2 finding was that the orchestrator authored its own spot-check — that failure mode does not
apply here.
**Date:** 2026-07-12
**Rubric (clause c):** for each sampled phase I walk ONE load-bearing example end-to-end and ask
only *"does this actually work as claimed?"* — NOT *"did the phase follow procedure?"*.
**Hash-binding (clause d):** the P110 verifier
(`quality/gates/agent-ux/p110-surprises-absorption.sh`, row `agent-ux/p110-surprises-absorption`)
asserts this file exists; a content-hash bind at grade-time is the intended clause-(d) mechanism
so a gutted-but-present report FAILs exactly as an absent one would.

## Sample (clause a — every zero-intake phase, plus a reasoned sample of filing phases)

Filing tally from `SURPRISES-INTAKE.md` `discovered-by:` tags — phases that DID file intake:
**P102** (5 rows), **P104** (2 rows), **P105** (3 rows). Non-phase lanes also filed
(health-triage ×2, C2-wave-2, two GSD-quicks, D2-re-seal Wave-1).

**Phases that closed with ZERO intake entries (clause-a mandatory sample):** **P103, P106, P107,
P108, P109.** An empty intake is exactly the phase most in need of a spot-check — it is either
genuinely clean or a phase that stopped looking. All five are walked below.

**Additional sampled filing phases:** P102 and P105 (walked because P105 is the source of the
HIGH push-correctness item and P102 is the serializing corruption gate the whole milestone
depends on).

## Deep walk — the one critical example (clause c)

**P105 — does the DOCUMENTED agent recovery `git pull --rebase && git push` actually reconcile
after a rejected push?** This is the single most load-bearing agent-UX claim in v0.14.0 (the
CLAUDE.md-documented recovery move). Walked end-to-end from committed source, not the executor's
word:

1. A push is rejected on remote drift → helper emits the standard "fetch first" error (per the
   push-conflict path). Agent runs `git pull --rebase`.
2. **The bug the intake filed (row 5, 08:35):** the `import` helper used to write
   `refs/reposix/origin/main` DIRECTLY in the fast-import stream AND advertise a refspec that
   made git ALSO update it → `cannot lock ref` → `git pull --rebase` exits 1 → the `&&`
   short-circuits → `git push` never runs → edit stranded.
3. **Verify against reality (committed source at HEAD):** `crates/reposix-remote/src/main.rs:202`
   now advertises `refspec refs/heads/*:refs/reposix-import/*` and `fast_import.rs:127-130` writes
   the PRIVATE, disjoint `refs/reposix-import/*` namespace — so `git fetch` is the SOLE writer of
   `refs/reposix/origin/main`. The double-write is gone. Fix commit: `bd5b9cb`.
4. **Phase-close proof:** `.planning/phases/105-rbf-lr-03-rebase-recovery/VERIFICATION.md` (GREEN,
   HEAD `8afb52d`) grades `agent-ux/rebase-recovery-reconciles` exit 0 (13/13 asserts) across
   peer-push drift (A), REST-PATCH drift (B), and record-deletion (C, no resurrection).

**Does it work end-to-end? YES — the single documented command now reconciles.**

**Honesty finding surfaced by this walk (the reason the walk matters):** the intake's row-5 STATUS
still read "OPEN / still-live HIGH bug" because it was filed at 08:35 BEFORE the same-phase
layer-2 fix (`bd5b9cb`) landed — and the coordinator's drain disposition inherited that stale
snapshot. The end-to-end walk (not a procedure check) is precisely what caught it. Row 5 is
corrected to **RESOLVED-in-P105** in this drain, with only the genuine residual (git >= 2.34
stateless-connect verification, PLAN §5, run on 2.25.1) carried forward as a DEFERRED coverage
extension. This is the F-K5 rubric doing its job: a phase can grade GREEN on procedure while a
downstream ledger says "broken" — the outcome-facing walk reconciles them to the truth.

## Per-phase walk (clause c rubric — one concrete example each)

| Phase | Critical example walked | Works end-to-end? | Notes |
|---|---|---|---|
| P103 (zero-intake) | doc-alignment grade/persist split + `file-size-limits` waiver OP-8 narrowing — does the split grade without re-persisting under NO-CARGO? | YES | Phase-close VERDICT `d0f23d5`/`d56d57a`: 3/3 GREEN, NO-CARGO honored; waiver narrowed to 56 residual files (`dad227e`). Empty intake consistent — a mechanical split, nothing to defer. |
| P106 (zero-intake) | do the waived tutorial/example snippets (01/02/04/05) actually reproduce against the sim? | YES | `quality/reports/verifications/docs-repro/` holds live per-example JSON + `.log` runs (example-01..05, tutorial-replay, snippet-coverage) — committed by THIS drain (T03). The diagnosis (`106-DIAGNOSIS.md`) reframed the renewed-P90 waiver; evidence shows real reproduction, not a paper waiver. |
| P107 (zero-intake) | does the RUSTSEC posture gate actually assert memmap2 + quinn-proto advisories are documented, not silently ignored? | YES | `evidence/P107-VERIFICATION.md` (GREEN, 4/4) + committed `p107-cargo-audit-2026-07-12.txt` (`7cfd165`). The cargo-audit artifact is the falsifying evidence, not a proxy claim. |
| P108 (zero-intake) | does `prune_oid_map` actually gate on a connector completeness signal so a paginated/truncated listing cannot delete live records? | YES | Impl shipped pre-P108 at `5cb9a14` (`gate prune_oid_map on connector completeness signal` + idempotent delete-NotFound); P108 closed the paperwork (`13de686`). The data-loss hazard is gated at the completeness signal, not on raw list length. |
| P109 (zero-intake) | does the RBF-FW-11 grandfather rule key off the LANDING COMMIT (not a null `last_verified`) so pre-existing rows aren't spuriously failed? | YES | Catalog-first `1cb9dd1` then fix `10bd508` (`grandfather keys off landing commit, not null last_verified`). Test-then-fix order visible in git log; the grandfather predicate is commit-anchored. |
| P102 (filed 5) | does the leaf-isolation guard BLOCK a canonical `cargo run -p reposix-cli -- init sim::demo .` at the shared tree while ALLOWing a `/tmp` redirect? | YES | `39a8500` hardening (canonical-form + realpath cwd + quoting + fail-closed) + `2ad2bf5` D2 re-seal (config-read false-positive / git-init-bare / cargo-sim-seed). CLAUDE.md § Non-negotiables documents all canonical forms as BLOCKING; the phase filed 5 adversarial-review holes AND fixed them in-lane — model use of the +2 framework. |
| P105 (filed 3) | (deep walk above) | YES | Filed 3 rows honestly; one (lost-update) → P113, one (fetch-ref-lock) fixed in-phase but ledger lagged (corrected here), one (resolve_import_parent) genuinely latent → DEFERRED. |

## Aggregate finding

**GREEN.** Every v0.14.0 phase sampled either filed honest intake (P102/P104/P105 — including
P102's adversarial self-review surfacing 5 of its own guard holes) or closed clean with a
verifiable working critical path (P103/P106/P107/P108/P109). No phase exhibits the
"found-it-but-skipped-it" failure mode. The intake is NON-empty and richly sourced (16 entries
across 3 phases + 5 non-phase lanes) — the opposite of the v0.13.0-F2 empty-intake-with-skipped-
findings RED pattern.

**One honesty correction was required and made** (not a RED, but a ledger-lag the outcome-facing
rubric caught): row 5's "still-live" status was stale — the fix (`bd5b9cb`) and its GREEN gate
prove the documented recovery works; the entry is corrected to RESOLVED-in-P105 with only the
modern-git verification residual deferred. That a procedure-facing check would have rubber-stamped
"phase GREEN, row OPEN, both fine" while the reality is "bug fixed, ledger stale" is exactly why
clause (c) mandates the end-to-end walk.

**Honesty contract: SIGNED.**

## Verdict

- [x] Every zero-intake phase sampled (clause a) — P103, P106, P107, P108, P109 all walked.
- [x] Spot-check author is not the milestone orchestrator (clause b) — dispatched drain leaf, stated.
- [x] Every sampled phase answers the "does it work end-to-end" rubric, not "did it follow procedure" (clause c) — per-phase table + one deep walk.
- [x] This report's content is intended for hash-binding by `agent-ux/p110-surprises-absorption` (clause d).

**Status: GREEN**

## Cross-reference for the verifier subagent

- `.planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md` — drained (0 OPEN; 16 terminal: 10 RESOLVED, 6 DEFERRED, 0 WONTFIX).
- `quality/gates/agent-ux/p110-surprises-absorption.sh` — fence-aware mechanical assertion of the drain.
- `quality/catalogs/agent-ux.json` row `agent-ux/p110-surprises-absorption` — FAIL until the verifier flips it.
- This file — narrative F-K5 honesty grade.
