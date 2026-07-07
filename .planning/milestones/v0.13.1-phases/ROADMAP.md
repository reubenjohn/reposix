## v0.13.1 Front door actually works — onboarding hotfix (PLANNING)

> **Status:** scaffolded 2026-07-07. Hotfix milestone, NOT discovery — all seven
> acceptance items are already root-caused (D1 decision, `.planning/STATE.md`
> `last_activity` 2026-07-07). Zero-shot human-simulation testing (3 independent
> fresh-agent reproductions) found `reposix-sim` onboarding **100% broken on the shipped
> binary**: the sim ships in no prebuilt distribution, the documented getting-started flow
> mis-states filenames/counts/columns/flags, and `reposix init` masks the underlying
> fetch failure behind exit 0. This milestone makes the front door actually work and
> institutionalizes the zero-shot check so it can never silently rot again. Sequenced
> BEFORE the v0.14.0 pivot per D1.

> **Phase numbering (2026-07-07).** v0.13.0 (extension) closed at **P97**. This hotfix
> claims **P98–P101** as the four onboarding-repair phases inserted after P97 and before
> the v0.14.0 pivot. The queued v0.13.2 "Cross-link fidelity" milestone was scoped at
> P98–P107 while v0.13.1 did not yet exist; when v0.13.2 is (re)planned it shifts to
> **P102–P111** to make room for this hotfix (same renumber-on-insertion convention the
> v0.12.1 ROADMAP used for P66). This scaffold does NOT edit v0.13.2's files — the
> coordinator applies the shift at v0.13.2 plan time. **RAISED for owner/coordinator
> confirmation before P98 executes.**

**Thesis.** A launch that a fresh agent cannot complete by copy-pasting the documented
flow is not launched. v0.13.1 closes the gap between what the getting-started docs claim
and what a locally-built binary actually does, fixes the one code lie that hides the
failure (`reposix init` exit-0 on unreachable backend), and adds a standing
zero-shot-human-simulation gate so the front door is re-verified at every milestone close
(D3).

**Seven verified-broken acceptance items** (each already root-caused; grouped into 4 phases):

| # | Kind | One-line | Phase |
|---|------|----------|-------|
| B1 | doc-lie | issue filenames/counts/titles wrong in `docs/tutorials/first-run.md` + `docs/index.md` (real: `issues/1.md`..`6.md` unpadded, 6 issues, seed issue 1 title `A-edit-conflict-test`) | P98 |
| B2 | doc-lie | tutorial step-8 audit SQL uses nonexistent column `decision` (real columns: `ts,op,backend,project,issue_id,oid,bytes,reason`); narrative wrongly claims no `helper_push_sanitized_field` row | P98 |
| B3-docs | doc-lie | tutorial step-2 uses source-tree `--seed-file crates/.../seed.json` absent from release archives; align docs to built-in default seed | P98 |
| B5 | TRIAGE | `git pull --rebase && git push` crash after concurrent external REST write — decide known-limitation (RBF-LR-03-family, honest docs + file for v0.14.0) vs regression (fix now); record in `.planning/CONSULT-DECISIONS.md` | P98 |
| B3-sim | code | make the built-in default seed the documented path; sim startup banner mismatch (add tiny banner OR drop the doc claim) | P99 |
| B4 | code | `reposix init` masks unreachable/failed-fetch backend as exit-0 success (`crates/reposix-cli/src/init.rs` ~216–253) — must exit non-zero honestly; remove best-effort-always-`Ok` doc-comment (init.rs:141–144); fix misleading WARN naming `refs/reposix/main` | P99 |
| B6 | quality | new agent-ux catalog row "zero-shot human-simulation onboarding" in `quality/catalogs/agent-ux.json` + verifier under `quality/gates/agent-ux/`; milestone-close cadence | P100 |
| B7 | housekeeping | two binstall rows in `quality/catalogs/doc-alignment.json` drift to `STALE_TEST_DRIFT` after PR #70; refresh honestly; clean tree | P100 |

**Definition of Done (milestone-level).** v0.13.1 closes ONLY when ALL of:

1. A fresh **zero-shot human-sim agent** copy-pastes the documented sim getting-started
   flow on a **locally-built binary** and completes **read + write + push with ZERO
   manual fixups**.
2. `reposix init` exits **non-zero** on an unreachable/failed-fetch backend (no more
   exit-0 masking).
3. **No documented command errors on copy-paste** — every command in
   `docs/tutorials/first-run.md` + `docs/index.md` getting-started runs verbatim.
4. **B5 resolved honestly** — either fixed, or documented as a known limitation with a
   filed v0.14.0 follow-up and honest troubleshooting prose; decision recorded in
   `.planning/CONSULT-DECISIONS.md`.
5. **Clean tree pushed to `origin/main`** (per-phase push cadence; no dirty carry).
6. An **unbiased gsd-verifier subagent confirms the catalog rows PASS** (P98–P101), and
   the new zero-shot agent-ux row is GREEN at milestone close.

**Depends on:** v0.13.0 shipped (HEAD == origin/main == `5fd4731` at scaffold time).

---

### Phase 98: Doc-truth — getting-started prose matches reality (B1, B2, B3-docs, B5-honesty)

**Goal:** Make every claim in the sim getting-started path true. Correct issue
filenames/counts/titles (B1), the audit-SQL column + the false "no sanitized-field row"
narrative (B2), and the source-tree `--seed-file` command that does not exist in release
archives (B3-docs). Triage B5 (the `pull --rebase && push` crash after a concurrent
external REST write): decide **known-limitation** (RBF-LR-03-family — make docs honest +
file a v0.14.0 follow-up) vs **regression** (fix now), and record the decision in
`.planning/CONSULT-DECISIONS.md`. This phase is DOCS + a decision record only — no crate
code changes (those are P99).

**Requirements:** ONB-DOC-ISSUES-01, ONB-DOC-AUDIT-SQL-01, ONB-DOC-SEED-DOCS-01, ONB-PULL-REBASE-TRIAGE-01 · **Depends on:** v0.13.0 shipped · **Plan:** TBD (`/gsd-plan-phase 98`)

**Success criteria:**
1. `docs/tutorials/first-run.md` + `docs/index.md` name the real seed records: `issues/1.md`..`issues/6.md` (unpadded), 6 issues, seed issue 1 title `A-edit-conflict-test` — verified against the actual default-seed output.
2. Tutorial step-8 audit SQL uses only real columns (`ts, op, backend, project, issue_id, oid, bytes, reason`); the `decision` column reference is gone; the narrative no longer claims there is no `helper_push_sanitized_field` row (the row IS produced — narrative corrected to match).
3. Tutorial step-2 no longer instructs a source-tree `--seed-file crates/.../seed.json`; prose aligns to the built-in default seed that ships in release archives.
4. **B5 triage recorded** in `.planning/CONSULT-DECISIONS.md` with a dated `[SELF|FABLE|OWNER]` entry: verdict (known-limitation vs regression), rationale, and — if known-limitation — the honest troubleshooting prose landed + a v0.14.0 follow-up filed (`SURPRISES-INTAKE.md`); if regression, hand-off to P99.
5. Phase close: `git push origin main`; verifier subagent GREEN; verdict at `quality/reports/verdicts/p98/VERDICT.md`.

**Context anchor:** `docs/tutorials/first-run.md`; `docs/index.md`; the default-seed source; `docs/how-it-works/trust-model.md` (`helper_push_sanitized_field`); ADR-010 §3 + `docs/guides/troubleshooting.md` (RBF-LR-03 known-limitation precedent); `.planning/CONSULT-DECISIONS.md`.

### Phase 99: Code-honesty — built-in seed is the real path + `init` fails loudly (B3-sim, B4)

**Goal:** Make the binary behave the way P98's corrected docs now promise. Ensure the
built-in default seed is the documented, archive-shipped path (B3-sim), and reconcile the
sim startup banner with the docs (add a tiny banner OR drop the doc claim — whichever
P98 chose). Fix the load-bearing code lie (B4): `reposix init` must exit **non-zero** when
the backend is unreachable or the fetch fails, instead of masking it as exit-0 success
(`crates/reposix-cli/src/init.rs` ~216–253); remove the best-effort-always-`Ok`
doc-comment (init.rs:141–144); fix the misleading WARN that names `refs/reposix/main`.

**Requirements:** ONB-SIM-SEED-DEFAULT-01, ONB-SIM-BANNER-01, ONB-INIT-EXIT-HONEST-01 · **Depends on:** P98 GREEN · **Plan:** TBD (`/gsd-plan-phase 99`)

**Success criteria:**
1. Built-in default seed is the documented onboarding path; a locally-built binary reproduces the exact records P98's docs describe (`issues/1.md`..`6.md`, issue 1 = `A-edit-conflict-test`) with no `--seed-file` flag.
2. Sim startup banner matches the docs (banner added, or the doc claim dropped — consistent with the P98 decision); no residual doc↔binary mismatch on startup output.
3. `reposix init` against an unreachable/failed-fetch backend exits **non-zero** with an honest, teaching error; an integration test asserts the non-zero exit + message.
4. The best-effort-always-`Ok` doc-comment at `init.rs:141–144` is removed; the WARN that misleadingly names `refs/reposix/main` is corrected to name the real condition.
5. Cargo discipline respected (one invocation at a time, prefer `-p reposix-cli`); tests green.
6. Phase close: `git push origin main`; verifier subagent GREEN; verdict at `quality/reports/verdicts/p99/VERDICT.md`.

**Context anchor:** `crates/reposix-cli/src/init.rs` (~141–144 doc-comment, ~216–253 exit-masking, WARN naming); the default-seed source in `crates/reposix-sim/`; `crates/CLAUDE.md` (build memory budget); P98 banner/seed decision.

### Phase 100: Quality — institutionalize the zero-shot gate + refresh drifted rows (B6, B7)

**Goal:** Turn the D3 decision ("zero-shot human-simulation testing becomes a standing
milestone-close gate") into a real catalog row + verifier (B6), and clean up the two
binstall doc-alignment rows that drifted to `STALE_TEST_DRIFT` after PR #70 (B7).
Catalog-first: the row lands BEFORE the verifier it grades. This phase leaves the tree
clean across the quality dimension.

**Requirements:** ONB-ZEROSHOT-CATALOG-01, ONB-ZEROSHOT-VERIFIER-01, ONB-BINSTALL-DRIFT-01 · **Depends on:** P99 GREEN · **Plan:** TBD (`/gsd-plan-phase 100`)

**Success criteria:**
1. New agent-ux catalog row "zero-shot human-simulation onboarding" lands in `quality/catalogs/agent-ux.json`, conforming to the unified row schema in `quality/catalogs/README.md`, with `cadence: milestone-close` and a `blast_radius` reflecting front-door criticality.
2. A verifier under `quality/gates/agent-ux/` implements the row — it drives (or honestly NOT-VERIFIED-gates) a fresh zero-shot sim onboarding run on a locally-built binary; discovered cleanly by the runner via tag.
3. The two binstall rows in `quality/catalogs/doc-alignment.json` (`docs/index/install-cargo-binstall`, `planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish-07-binstall`) are refreshed honestly post-PR-#70 — no `STALE_TEST_DRIFT`; the working-tree modification to `doc-alignment.json` present at scaffold time is resolved as part of this row refresh (RAISED below).
4. `quality/CLAUDE.md` / `quality/PROTOCOL.md` updated in the same PR if the new row introduces a convention (per the fix-it-twice meta-rule).
5. Tree clean; phase close: `git push origin main`; verifier subagent GREEN; verdict at `quality/reports/verdicts/p100/VERDICT.md`.

**Context anchor:** `quality/catalogs/agent-ux.json`; `quality/gates/agent-ux/`; `quality/catalogs/README.md` (row schema); `quality/catalogs/doc-alignment.json:1934,1959` (binstall rows); PR #70 (`4b564e4`); `quality/CLAUDE.md`; STATE.md D3.

### Phase 101: Zero-shot gate run + milestone verify (Definition-of-Done proof)

**Goal:** Prove the milestone Definition of Done against reality. Dispatch a fresh
zero-shot human-sim agent that copy-pastes the documented sim getting-started flow on a
locally-built binary and completes read + write + push with ZERO manual fixups. Confirm
`reposix init` non-zero on unreachable backend. Confirm B5 resolved honestly. Ensure the
tree is clean and pushed. Dispatch the unbiased gsd-verifier to confirm P98–P101 catalog
rows PASS (incl. the new zero-shot agent-ux row). This is the milestone-close phase.

**Requirements:** ONB-ZEROSHOT-GATE-01, ONB-MILESTONE-VERIFY-01 · **Depends on:** P98 + P99 + P100 ALL GREEN · **Plan:** TBD (`/gsd-plan-phase 101`) · **Execution mode:** top-level (fresh-agent zero-shot dispatch is orchestration-shaped, not `gsd-executor`)

**Success criteria:**
1. A fresh zero-shot human-sim agent (no prior context, given only the shipped docs) completes read + write + push against the sim on a locally-built binary with ZERO manual fixups; the run transcript is captured as evidence under `quality/reports/verifications/agent-ux/`.
2. `reposix init` non-zero-on-unreachable is demonstrated live (not just unit-tested).
3. No documented command in the getting-started path errors on verbatim copy-paste.
4. B5 confirmed resolved honestly (decision in `.planning/CONSULT-DECISIONS.md`; if known-limitation, the v0.14.0 follow-up is filed and the troubleshooting prose is honest).
5. Clean tree on `origin/main`; the new zero-shot agent-ux catalog row is GREEN.
6. Unbiased gsd-verifier subagent confirms P98–P101 catalog rows PASS; milestone verdict at `quality/reports/verdicts/milestone-v0.13.1/VERDICT.md`. RETROSPECTIVE distilled per OP-9 before archive.
7. Phase close: `git push origin main`; verifier GREEN; verdict at `quality/reports/verdicts/p101/VERDICT.md`.

**Context anchor:** the milestone Definition of Done (above); `quality/reports/verifications/agent-ux/`; `.planning/ORCHESTRATION.md` (orchestration-shaped phases run top-level); `quality/dispatch/`; `.planning/RETROSPECTIVE.md` (OP-9).

### Recurring success criteria for EVERY phase (P98–P101)

Non-negotiable per CLAUDE.md Operating Principles + the autonomous-execution protocol; NOT separate REQ-IDs:

1. **Catalog-first** — the quality phase (P100) writes catalog rows BEFORE the verifier that grades them (per `quality/CLAUDE.md`).
2. **CLAUDE.md updated in the same PR** — any phase that introduces a new file/convention/gate revises the relevant CLAUDE.md section in the same PR (fix-it-twice meta-rule).
3. **Per-phase push** — `git push origin main` BEFORE the verifier-subagent dispatch; the verifier grades RED if the phase shipped without the push landing.
4. **Phase close = unbiased verifier subagent (OP-7)** — verdict at `quality/reports/verdicts/p<N>/VERDICT.md`; the phase does not close on RED.
5. **Eager-resolution (OP-8)** — items <1h / no new dependency fixed in-phase; else filed to `SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md` (never silently skipped).
6. **Simulator-first (OP-1)** — the entire hotfix targets the sim onboarding path; the sim is the default and only backend exercised here.
7. **Verify against reality** — the DoD is proven by a real fresh-agent run (P101), not asserted.
