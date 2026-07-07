# v0.13.1 Requirements — Front door actually works (onboarding hotfix)

**Milestone status:** PLANNING (Phases P98–P101; scaffolded 2026-07-07).

**Milestone goal:** Make the documented `reposix-sim` getting-started flow actually work
for a fresh zero-shot agent on a locally-built binary — read + write + push with ZERO
manual fixups — fix the one code lie that hides the failure (`reposix init` exit-0 on an
unreachable backend), resolve the `pull --rebase && push` crash honestly, and
institutionalize a standing zero-shot-human-simulation gate (D3) so the front door can
never silently rot again. This is a **hotfix, not discovery**: all seven acceptance items
are already root-caused (D1, `.planning/STATE.md` 2026-07-07). Sequenced BEFORE the
v0.14.0 pivot.

**The litmus test:** at v0.13.1 close, a fresh agent given only the shipped docs
copy-pastes the sim getting-started flow on a locally-built binary and completes
read+write+push with zero manual fixups; `reposix init` exits non-zero on an unreachable
backend; no documented command errors on copy-paste; B5 is resolved honestly; the tree is
clean on `origin/main`; and an unbiased gsd-verifier confirms the P98–P101 catalog rows
PASS.

**Root-cause bundle (source of truth):**
- `.planning/STATE.md` `last_activity` 2026-07-07 (D1 / D2 / D3 decisions; zero-shot
  reproduction findings).
- `docs/tutorials/first-run.md` + `docs/index.md` — the drifted getting-started prose (B1, B2, B3-docs).
- `crates/reposix-cli/src/init.rs` (~141–144 doc-comment, ~216–253 exit-masking, WARN naming) — B4.
- `crates/reposix-sim/` default seed — B3-sim ground truth (`issues/1.md`..`6.md`, issue 1 = `A-edit-conflict-test`).
- `docs/how-it-works/trust-model.md` — `helper_push_sanitized_field` audit reality (B2).
- ADR-010 §3 + `docs/guides/troubleshooting.md` — RBF-LR-03 known-limitation precedent (B5).
- `quality/catalogs/doc-alignment.json:1934,1959` — the two drifted binstall rows (B7); PR #70 (`4b564e4`).
- `quality/catalogs/agent-ux.json` + `quality/gates/agent-ux/` — home for the new zero-shot row (B6).

**Operating-principle hooks (non-negotiable, per project CLAUDE.md):**
- **OP-1 Simulator-first.** Every phase targets the sim onboarding path; sim is the only backend exercised.
- **OP-2 Tainted-by-default.** No new egress surface in this hotfix; seed data stays attacker-influenced by assumption.
- **OP-3 Audit log non-optional.** B2's fix corrects the audit narrative to match the real `helper_push_sanitized_field` row — it does not remove or weaken any audit write.
- **OP-7 Verifier subagent on every phase close.** P101 dispatches the milestone-close verifier; every phase gets a per-phase verdict.
- **OP-8 +2 phase practice (adapted).** This 4-phase hotfix folds surprises-absorption into P101 (milestone-close) rather than reserving a standalone slot; eager-resolution still applies per phase.
- **OP-9 Milestone-close: distill before archiving.** P101 distills into `.planning/RETROSPECTIVE.md` before archive.
- **Per-phase push cadence.** `git push origin main` BEFORE verifier-subagent dispatch.
- **Enter through GSD.** Phases are planned via `/gsd-plan-phase <n>` and executed via `/gsd-execute-phase <n>`; no hand-editing outside a GSD-tracked phase.

### Active

#### Doc-truth — getting-started prose matches reality (P98)

- [ ] **ONB-DOC-ISSUES-01** (B1): `docs/tutorials/first-run.md` + `docs/index.md` name the real seed records — `issues/1.md`..`issues/6.md` (unpadded), 6 issues, seed issue 1 title `A-edit-conflict-test`. Verified against the actual default-seed output, not asserted.
- [ ] **ONB-DOC-AUDIT-SQL-01** (B2): Tutorial step-8 audit SQL uses only real columns (`ts, op, backend, project, issue_id, oid, bytes, reason`); the nonexistent `decision` column is removed; the narrative no longer falsely claims there is no `helper_push_sanitized_field` row (the row IS produced — narrative corrected).
- [ ] **ONB-DOC-SEED-DOCS-01** (B3-docs): Tutorial step-2 no longer instructs a source-tree `--seed-file crates/.../seed.json` (absent from release archives); prose aligns to the built-in default seed that ships in release archives.
- [ ] **ONB-PULL-REBASE-TRIAGE-01** (B5): `git pull --rebase && git push` crash after a concurrent external REST write is triaged — a dated entry in `.planning/CONSULT-DECISIONS.md` records known-limitation (RBF-LR-03-family; honest troubleshooting prose + v0.14.0 follow-up filed) vs regression (hand-off to P99 for a fix). No silent skip.

#### Code-honesty — built-in seed is the real path + `init` fails loudly (P99)

- [ ] **ONB-SIM-SEED-DEFAULT-01** (B3-sim): The built-in default seed is the documented onboarding path; a locally-built binary reproduces the exact records P98's docs describe with no `--seed-file` flag.
- [ ] **ONB-SIM-BANNER-01** (B3-sim): Sim startup banner reconciled with the docs — a tiny banner is added OR the doc claim is dropped (consistent with the P98 decision); no residual doc↔binary startup mismatch.
- [ ] **ONB-INIT-EXIT-HONEST-01** (B4): `reposix init` against an unreachable/failed-fetch backend exits **non-zero** with an honest teaching error (`crates/reposix-cli/src/init.rs` ~216–253); an integration test asserts the non-zero exit + message; the best-effort-always-`Ok` doc-comment at init.rs:141–144 is removed; the misleading WARN that names `refs/reposix/main` is corrected to name the real condition.

#### Quality — institutionalize the zero-shot gate + refresh drifted rows (P100)

- [ ] **ONB-ZEROSHOT-CATALOG-01** (B6): New agent-ux catalog row "zero-shot human-simulation onboarding" lands in `quality/catalogs/agent-ux.json` (unified row schema per `quality/catalogs/README.md`; `cadence: milestone-close`; front-door `blast_radius`). Catalog-first — the row predates its verifier.
- [ ] **ONB-ZEROSHOT-VERIFIER-01** (B6): A verifier under `quality/gates/agent-ux/` implements the row — it drives (or honestly NOT-VERIFIED-gates) a fresh zero-shot sim onboarding run on a locally-built binary; discovered cleanly by the runner via tag. `quality/CLAUDE.md` / `quality/PROTOCOL.md` updated in the same PR if a convention is introduced.
- [ ] **ONB-BINSTALL-DRIFT-01** (B7): The two binstall rows in `quality/catalogs/doc-alignment.json` (`docs/index/install-cargo-binstall`, `planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish-07-binstall`) refreshed honestly post-PR-#70 — no `STALE_TEST_DRIFT`; the working-tree modification to `doc-alignment.json` present at scaffold time is resolved as part of this refresh. Tree clean.

#### Zero-shot gate run + milestone verify (P101)

- [ ] **ONB-ZEROSHOT-GATE-01**: A fresh zero-shot human-sim agent (given only shipped docs) completes read + write + push against the sim on a locally-built binary with ZERO manual fixups; `reposix init` non-zero-on-unreachable demonstrated live; no documented command errors on verbatim copy-paste; B5 confirmed resolved honestly; clean tree on `origin/main`; run transcript captured under `quality/reports/verifications/agent-ux/`.
- [ ] **ONB-MILESTONE-VERIFY-01**: Unbiased gsd-verifier subagent confirms P98–P101 catalog rows PASS (incl. the new zero-shot agent-ux row GREEN); milestone verdict at `quality/reports/verdicts/milestone-v0.13.1/VERDICT.md`; RETROSPECTIVE v0.13.1 section distilled per OP-9 BEFORE archive.

### Out of Scope (deferred to v0.14.0 or later)

- **The v0.14.0 pivot** (RBF-LR-03 slug/commit-sequence redesign, observability/multi-repo). v0.13.1 is sequenced strictly BEFORE it; if B5 is triaged as a known-limitation, its real fix carries forward here.
- **Worktree-isolation + reject-`t@t`-identity hook** (D2). P0 for v0.14.0 hardening, not this hotfix.
- **Shipping the sim in a prebuilt distribution.** v0.13.1 documents the locally-built path honestly; whether the sim gets its own prebuilt artifact is a v0.14.0 packaging decision.
- **The `crlf` CI flake** (S-260707-rbf-01, wiremock harness artifact under CI CPU starvation). OPEN as a monitor, not a v0.13.1 deliverable.

### Traceability

Drafted 2026-07-07 (hotfix scaffold; phases planned per `/gsd-plan-phase` at execution
time). Coverage: **12/12 v0.13.1 REQ-IDs mapped to exactly one phase** (no orphans, no
duplicates). Phases P98–P101, inserted after v0.13.0/P97 and before the v0.14.0 pivot.

| REQ-ID | B-item | Phase | Kind | Status |
|--------|--------|-------|------|--------|
| ONB-DOC-ISSUES-01 | B1 | P98 | doc-lie | planned |
| ONB-DOC-AUDIT-SQL-01 | B2 | P98 | doc-lie | planned |
| ONB-DOC-SEED-DOCS-01 | B3-docs | P98 | doc-lie | planned |
| ONB-PULL-REBASE-TRIAGE-01 | B5 | P98 | triage | planned |
| ONB-SIM-SEED-DEFAULT-01 | B3-sim | P99 | code | planned |
| ONB-SIM-BANNER-01 | B3-sim | P99 | code | planned |
| ONB-INIT-EXIT-HONEST-01 | B4 | P99 | code | planned |
| ONB-ZEROSHOT-CATALOG-01 | B6 | P100 | quality | planned |
| ONB-ZEROSHOT-VERIFIER-01 | B6 | P100 | quality | planned |
| ONB-BINSTALL-DRIFT-01 | B7 | P100 | housekeeping | planned |
| ONB-ZEROSHOT-GATE-01 | DoD | P101 | verify | planned |
| ONB-MILESTONE-VERIFY-01 | DoD | P101 | verify | planned |

### Definition of Done (milestone) — restated for the verifier

v0.13.1 closes ONLY when ALL hold, each proven by an artifact (not asserted):
1. Fresh zero-shot human-sim agent completes read+write+push on a locally-built binary with ZERO manual fixups.
2. `reposix init` exits non-zero on an unreachable/failed-fetch backend.
3. No documented getting-started command errors on verbatim copy-paste.
4. B5 resolved honestly (decision in `.planning/CONSULT-DECISIONS.md`; known-limitation ⇒ v0.14.0 follow-up filed + honest troubleshooting prose).
5. Clean tree pushed to `origin/main`.
6. Unbiased gsd-verifier confirms P98–P101 catalog rows PASS; new zero-shot agent-ux row GREEN at milestone close.
