# P89 Cross-AI Review — IMPLEMENTED framework changes (OD-1 as amended by OD-3)

**Date:** 2026-07-04 · **Scope:** the implemented P89 deliverables (commits 79a46be..3b38117), NOT the plans (plan-time cross-AI review was 11c6dda).

## Reviewers (three independent lineages)

| Leg | Tool | Notes |
|---|---|---|
| Codex | `codex exec -m gpt-5.5 --sandbox read-only` | gpt-5.3/5.2/5.1-codex rejected by account tier; gpt-5.5 used |
| Claude | zero-context opus subagent, adversarial brief, read-only | distinct from the orchestrator session per F-K5 |
| Independent #2 | zero-context sonnet subagent, adversarial brief, read-only | **substitutes for Gemini** — `gemini` CLI is unusable on this machine (`IneligibleTierError: no longer supported for Gemini Code Assist for individuals`). Recorded honestly per OD-3 close-report requirement. |

All three received the same brief (attack the gap between what catalog rows claim and what verifiers assert); raw reports archived in the session scratchpad (`review-codex.md`, `review-claude.md`, `review-independent2.md`); consolidated findings below are the committed record.

## Consolidated findings + dispositions

| ID | Sev | Finding (sources) | Disposition |
|---|---|---|---|
| H1 | HIGH | Env-gate accepted `localhost`/`[::1]`/`0.0.0.0`/`127.0.0.2` as "real backend" — fake env un-skips the cadence (Claude H1, Codex M5, Ind2) | **FIXED in-phase** (this commit): `_realbackend._has_non_local_origin` rejects all loopback/unspecified spellings, case-insensitive, multi-origin aware; 9 regression tests. **Residual** (sanctioned-target MEMBERSHIP + cred↔origin correspondence not checked): the env-gate is a skip heuristic, not the proof — the proof is the litmus verifier actually executing against the sanctioned target (P91). Filed to SURPRISES-INTAKE (MED, home P91). |
| H2 | HIGH | `claim_vs_assertion_audit` cutoff anchors on mutable `last_verified`; a backdated row dodges the check durably (Ind2 H1, Claude H2, Codex M4) | **DEFERRED with designed counters + intake HIGH.** Counters already in the design: (a) phase-close verifier subagent spot-checks hand-backdated `last_verified` (89-CONTEXT watchout, active TODAY); (b) P90 RBF-FW-12 adversarial dispatch grades audit text; (c) P95 RBF-D-06 backfills ALL legacy rows, after which the check becomes unconditional and the exemption class (and the bypass) disappears. Filed to SURPRISES-INTAKE (HIGH) proposing an immutable `minted_at` field in P90 so the window between now and P95 is also closed mechanically. |
| H3 | HIGH | OD-2 "no waiver" was prose-only: a `waiver` block on the P0 vision-litmus row would grade it green via the WAIVED path (Codex H1) | **FIXED in-phase** (this commit): `_audit_field.validate_row` refuses to LOAD any row tagged cadence `pre-release-real-backend` that carries a waiver (SystemExit before any verifier runs); 3 regression tests; PROTOCOL.md OD-2 block now names the mechanical enforcement. |
| H4 | HIGH | Missing verifier script preserves prior PASS/FAIL/PARTIAL (`run.py` verifier-not-found branch) — delete/typo a verifier path and a PASS row stays green (Codex H3) | **DEFERRED — pre-existing** (landed dd458bd, P57/v0.12.0; not introduced by P89) and squarely P90 RBF-FW-07 territory (runner-wide status preservation). Filed to SURPRISES-INTAKE (HIGH). Not fixed in-phase: changing the branch flips ~all rows NOT-VERIFIED on any deploy-time path glitch — the P57 rationale needs a deliberate P90 design decision, not a drive-by edit. |
| H5 | HIGH | OD-2 at actual release-cut is paperwork: `tag-v0.13.0.sh.disabled` is disabled and its guard greps a human-authored verdict, never the catalog JSON (Ind2 H2) | **DEFERRED by design** — P97 RBF-G-04 owns tag-script re-enable + 9th-probe guard (D-03d, documented in PROTOCOL.md). This finding sharpens P97's acceptance criteria: the guard MUST invoke `run.py --cadence pre-release-real-backend` and require exit 0 against catalog state, not grep a verdict file. Recorded here for P97's planner. |
| M6 | MED | `kind: shell-subprocess` transcript contract satisfiable by the word "transcript" in asserts prose; runner PASSes exit-0 without requiring transcript existence (Codex H2, Claude M3, Ind2) | **DEFERRED to P90 RBF-FW-08 — pre-documented boundary** (89-CONTEXT: "kind is doc-convention not runtime-enforced... Real structural enforcement is a P90 concern"). The asserts-mention fallback is explicitly transitional (`_audit_field` docstring). P90 should make the runner require `transcript_path` + file existence before PASS on this kind. |
| M7 | MED | `claim_vs_assertion_audit_hash` drift detection has no stored baseline to diff (Ind2) | **WORKS-AS-DESIGNED, thin by intent:** the baseline is the row's current text at read time — a verifier subagent recomputes sha256 from the row and compares to the artifact hash from grade time; P90 RBF-FW-12's adversarial dispatch is the designed consumer. No change. |
| M8 | MED | An env-scrubbed `--cadence pre-release-real-backend` run DEMOTES a previously-PASS row to NOT-VERIFIED and persists it, with no record of why (Ind2; reproduced live during 89-08 when a verification re-run demoted the cadence wiring row) | **DEFERRED + intake MED** (home P90 RBF-FW-07 status-preservation design): whether skip-events should re-grade rows is a deliberate design call. Coordinator reverted the live churn (test artifact, not ground truth). |
| M9 | MED | SLOT stub has no mechanical trigger forcing un-stubbing once P91–P95 land (Ind2) | **MITIGATED BY DESIGN:** `blast_radius: P0` + NOT-VERIFIED means milestone-close CANNOT grade GREEN while the stub exits 75 — the RED at close is the trigger. H3's fix closes the only bypass (waiver). No change. |
| L10 | LOW | Catalog row claims `**/CHANGELOG.md` exclusion the banned-tokens script "doesn't implement" (Codex L) | **NOT-A-BUG:** the scan is `find crates -name '*.rs'` — CHANGELOG.md files are never in scope; the assert's exclusion mention is redundant-but-true. No change. |
| L11 | LOW | Transcripts capture raw argv/stdout/stderr (tainted bytes) (Ind2) | **ACCEPTED:** transcripts are gitignored local artifacts; env VALUES are never captured (names only — verified by the worked example's own post-condition); consumption path is a read by the verifier subagent, no side-effectful sink. |
| L12 | LOW | No catalog file locking for concurrent runners (Ind2) | **ACCEPTED:** single-tree-writer discipline is a repo-wide operating rule (CLAUDE.md); not a P89 surface. |
| L13 | LOW | Regex/portability nits incl. pattern-2 intervening-words gap (Ind2, self-disclosed) | **ALREADY FILED** as GOOD-TO-HAVES-02 (17b687d). |

## Non-findings that held (all reviewers)

env_keys never leaks values; exit-75 is not a green soft-skip (P0/P1 NOT-VERIFIED ⇒ exit 1); deferral linter blocks bare no-PNN deferrals; transient flags (`_stale`, `_skipped_real_backend`, `_exit75_not_verified`) are stripped before catalog persistence; linters/worked-example/SLOT/unit suites all reproduce green from a cold context.

## Verdict synthesis

All three reviewers agree the mechanical skeleton is sound and that creds-missing-at-milestone-close is RED today. The convergent criticism — the framework's guarantees were softer than its prose at three points (H1/H3 fixed in-phase; H2/H4 deferred with designed counters and intake entries) — is exactly the class of finding OD-1 mandated this review to catch. P89's residual trust anchors are named above and each has a designated owner phase (P90 RBF-FW-07/-08/-12, P91 litmus, P95 RBF-D-06, P97 RBF-G-04).

## OD-1/OD-3 compliance record

- Cross-AI review of the IMPLEMENTED framework: **executed** (this artifact).
- OD-1 owner sign-off: **delegated per OD-3 (2026-07-03)** — "OD-3 delegation exercised" is recorded in the P89 verdict; the owner is notified in the session close report instead of blocking phase close.
