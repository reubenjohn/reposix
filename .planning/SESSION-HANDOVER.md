# SESSION-HANDOVER.md — top-level (L0) session rotation, 2026-07-05

Written for the incoming top-level orchestrator (L0). The prior L0 session ended
deliberately to **re-launch with a leaner bootstrap** — its P92 recon over-fanned
(6 recon agents fanned separate reports into L0, burning ~12% of L0 context before
any execution). Compact per `.planning/ORCHESTRATION.md` §3.

**Read-first order:** (1) `.planning/STATE.md`, (2) this file, (3)
`.planning/CONSULT-DECISIONS.md` (D-P92-01), (4) `.planning/ORCHESTRATION.md`
§11–12 (no-fable tiering, 10%/10x budgets, DP/valve, HCI), (5)
`.planning/milestones/v0.13.0-phases/ROADMAP.md` (P92 section) +
`SURPRISES-INTAKE.md`, (6) `quality/PROTOCOL.md` if touching gates.

**Guardrails:** do NOT run `tag-v0.13.0.sh` (`.disabled`) until P97 GREEN. Do NOT
touch PR #61 (release hold) until P97. Do NOT `git add .` — a `M CLAUDE.md` is
intentionally dirty (at the 40k cap, slimmed) and stays UNSTAGED; only touch it
through a GSD-tracked change.

## 1. Ground truth (git)

- HEAD (pre-this-commit) = `2bfccdf` on `main`, up to date with `origin/main`.
- This handover commit adds ONLY `.planning/STATE.md` + `.planning/SESSION-HANDOVER.md`
  (local, not pushed — owner said leave local). `M CLAUDE.md` left unstaged.
- `STATE.md` frontmatter: `phases_completed: 14`, `next_phase: P92` — P78–P91
  SHIPPED of 20 (P78–P97). **P92 is recon-complete but NOT executed.**
- Recent load-bearing commits: `2bfccdf` D-P92-01 ledger open; `9f1dd7d` hook-throttle
  gate; `e69b325` per-agent JIT throttle; `fe8c558`/`fe5e8f2` doctrine stale-sweep.

## 2. Wave/cycle state

| Item | State | Note |
|---|---|---|
| P91 | DONE GREEN | verdict `quality/reports/verdicts/p91/VERDICT.md` |
| P92 recon | DONE | → decision D-P92-01 no-split (2bfccdf) |
| P92 execution | NOT STARTED | residual scope below; next L0 dispatch |
| quality-weekly | GREEN | fixed; live run `28731753695` |

**D-P92-01 carry-forward (no-split).** Recon found the heavy P92 fixes ALREADY
LANDED before recon: `cb630e5` (GIT_DIR scrub) + `a0c84a3` (`.with_audit` chained
on Confluence + JIRA connectors). So P92 stays ONE phase — no P92a/P92b split.

**P92 residual scope (what execution must still deliver):**
1. **T4 rebase-ancestry regression test** — prove-before-fix (DP-2): write the
   failing test that reproduces the ancestry-drift before any fix.
2. **`bus_write_audit_completeness.rs`** — upgrade to assert OP-3 **dual-table**
   audit rows (both `audit_events` + `audit_events_cache`) on a real push.
3. **Behavioral no-retry verifier** — assert the push path does not silently retry.
4. **TokenWorld smoke** — real-backend litmus (cred-gated; see §4).

## 3. Binding constraints (unchanged)

- ONE cargo invocation machine-wide (`cargo-mutex.sh`, exit 2). Prefer `-p <crate>`.
- One tree-writer at a time; no `--no-verify`; commit-trailer format; model tiering.
- **Root `CLAUDE.md` is at the 40k cap** (~39,910 bytes) — any addition must be
  paid for by a deletion; push detail to a scoped `CLAUDE.md` or linked doc.
- **TokenWorld real-backend is cred-gated** — real Confluence/GitHub/JIRA litmus
  runs only when creds are in `.env` AND a non-default `REPOSIX_ALLOWED_ORIGINS`
  is set. Env-unset reads NOT-VERIFIED (never FAIL/skip-as-pass). Sim is default.

## 4. Litmus / gate / REOPEN state

- P92 litmus: T1 + **T4** on sim + TokenWorld; REOPENS on ≥1 HIGH (OD-2 unchanged).
- Milestone-close (P97) 9th probe `pre-release-real-backend` is non-skippable and
  never waivable (`.planning/CLAUDE.md`); env-gated rows read NOT-VERIFIED.
- No open REOPEN loops carried in. quality-weekly is GREEN as of run `28731753695`.

## 5. Mid-execution decisions + noticed, not yet filed

- **D-P92-01 (no-split)** formalized in `.planning/CONSULT-DECISIONS.md` (append-only
  ledger, opened this window at `2bfccdf`).
- **Owner-gated non-blockers (do NOT self-authorize; surface, don't act):**
  - **PR #62** — codecov-action dependabot bump (6→7), CI green. Not owner-named →
    no steward action (steward.md rule).
  - **~9 stale branches** (`bench/refresh-latency-*`, `release-plz-2026-*`, a few
    `fix/*`) — deletion blocked on owner-named-target approval.
  - **17 docs-alignment rows** flipped `STALE_TEST_DRIFT` → routed to **P95**
    (`/reposix-quality-refresh`); above the 0.5 alignment floor, non-blocking now.

## 6. Precise next steps (successor runbook)

1. Ground-truth first: `git log --oneline -8`, `git status`, confirm HEAD `2bfccdf`
   (+ this local handover commit) and that `M CLAUDE.md` is the only dirty path.
2. Read `.planning/CONSULT-DECISIONS.md` (D-P92-01) before touching P92.
3. **Re-launch LEAN — ONE consolidated recon lane, not 6.** Per ORCHESTRATION.md
   §11 report-only diet + 10% budget: dispatch a SINGLE recon/plan lane that returns
   ONE conclusion, not N separate report-bearing agents fanned into L0. Absorb a
   conclusion, not raw reports; children absorb the 10x blowup (pre-authorize the
   split in their charter).
4. Plan + execute **P92** on its residual scope (§2 items 1–4). Prove-before-fix on
   T4 (DP-2). Litmus T1+T4 sim+TokenWorld; REOPEN on ≥1 HIGH.
5. Per-phase close ritual: `git push origin main` BEFORE the verifier subagent;
   verifier grades RED on a missing push. Advance STATE.md cursor to P93.
6. Do NOT drain the owner-gated non-blockers (§5) without explicit owner naming.
