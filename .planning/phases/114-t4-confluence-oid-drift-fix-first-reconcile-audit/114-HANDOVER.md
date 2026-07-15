# 114-HANDOVER.md — P114 C1 relief, Wave-2-done / verification-tail boundary, 2026-07-15

Written by the outgoing P114 C1 phase-coordinator, relieving at a clean
Wave-2-code-done / verification-tail boundary (Wave 2 is DONE, PUSHED, CI GREEN — this
is NOT a mid-wave relief). Successor is a **fresh C1** coordinator identity dispatched
by L0 (this is a single-phase milestone slice; no C2 is in play). This file REPLACES
the prior (now-stale) Wave-1/Wave-2-boundary handover at this same path — that version
described Wave 2 as "not started"; it is now done. **Do not** touch
`.planning/MANAGER-HANDOVER.md` or `.planning/SESSION-HANDOVER.md` — those belong to
the concurrent w1:p7 manager session sharing this tree.

**Read order:** this file in full → `114-02-SUMMARY.md` (Wave-2 close, code-review
verdict) → `114-01-SUMMARY.md` (Wave-1 close, for the fold-in context) →
`114-RESEARCH.md` OQ1 (pre-ADF risk to SC1, still open — resolvable only by the live
SC1 run) → `.planning/ORCHESTRATION.md` §3 if the template itself is in doubt.

## 1. Ground truth (git)

`origin/main` = local HEAD = `12a0f57`, 0 ahead / 0 behind, working tree **CLEAN**
(verified via `git status` at relief time — this handover commit will be the first
thing to land on top).

Newest `ci.yml` run on main = `29435883070` on `12a0f57` → **conclusion=success**
(verified via `gh run view 29435883070`: all 15 jobs green — shell-coverage,
gitleaks, quality-gates pre-pr, test, rustfmt, clippy, runner unit tests, both v09 and
current real-backend contract integrations for github/jira/confluence, dark-factory
regression, coverage, bench-latency). `Docs` workflow_run `29436262574` on the same
push = success. `release-plz` run `29435883119` = success. `CodeQL` `29435881932` =
success. `code/ci-green-on-main` (P0) post-push probe = **PASS** (this is the state
the prior coordinator's step-4 leaves the successor to re-confirm, not re-derive from
scratch — it is already confirmed as of this handover).

Commits since the last known-clean sha, in order (verified via `git log --oneline`):

| Commit | Summary |
|---|---|
| `276beb8` | docs(planning): manager watch switches to polling model, ≤1h cap (owner directive) — **MANAGER's commit, not this coordinator's** |
| `eaf24d9` | docs(planning): polling-model doctrine moves to /herdr-manager skill — **MANAGER's commit** |
| `57362ff` | docs(114): C1 relief handover — Wave-1 done+CI-green, Wave-2 not started (the STALE handover this file replaces) |
| `09f2739` | docs(planning): L0 relief handover #26→#27 — v0.15.0 P114 Wave-1 done+CI-green, Wave-2 opens next — **prior L0's commit** |
| `6922f24` | docs(planning): poll-script pointer moves to skill-bundled canonical copy — **not this coordinator's/manager's** |
| `9915953` | test(114-02): `oid_drift_reconcile.rs` — `DriftingMock` + 3 tests |
| `0e200f9` | docs(114-02): error.rs/sync.rs/main.rs doc-precision + 114-01-SUMMARY §5 fold-in |
| `de87650` | docs(114-02): 114-02-SUMMARY.md |
| `9bed65a` | docs(114-02): builder.rs `read_blob` `# Errors` OidDrift framing aligned (code-review nit absorbed) |
| `12a0f57` | docs(114-02): re-bind cli-subcommand-surface doc-alignment hash (benign doc-comment drift, verified safe) |

`276beb8` + `eaf24d9` (and `09f2739`, `6922f24`) are the MANAGER's / prior L0's
doctrine/handover commits sitting below the Wave-2 stack — do **NOT** amend/rebase past
them; all future commits go strictly ON TOP.

## 2. Wave/cycle state

| Wave | Plan | State | Commits |
|---|---|---|---|
| 1 — FIX-01 (Confluence list/get render-parity) | `114-01-PLAN.md` | **DONE, PUSHED, CI GREEN** | `47fa803`, `9908fcc`, `bf005bc`, `db12187`, `6f15138` |
| 2 — FIX-02 (reconcile-audit: prove `--reconcile` does NOT heal systematic oid-drift + doc corrections) | `114-02-PLAN.md` | **CODE DONE, REVIEWED, PUSHED, CI GREEN** | `9915953`, `0e200f9`, `de87650`, `9bed65a`, `12a0f57` |

Wave 1: `cache.rs` confirmed accurate, untouched (no post-mortem needed — closed
clean, per the prior handover).

Wave 2 code-review verdict: **APPROVE-WITH-NITS**, all 5 charter items PASS. Test (b)
`reconcile_does_not_clear_stale_list_oid_while_bodies_diverge` proves BOTH halves of
reconcile-non-recovery (stale oid unchanged after a 2nd `build_from` AND `read_blob`
still errors `OidDrift`) — airtight SC4 backing. Doc precision correct; Pitfall 3
(over-correcting to "reconcile never heals oid-drift") was avoided — the
eventual-consistency-race class is still documented as reconcile-recoverable. The one
NIT (`builder.rs::read_blob` `# Errors` framing) was absorbed inline in `9bed65a`.

`12a0f57` is a benign doc-comment-drift re-bind: editing the `main.rs` `Sync` clap doc
in `0e200f9` moved prose inside the bound `cli-subcommand-surface` doc-alignment
region (L39–319), which drifted that row's stored `source_hash` and blocked the
pre-push `docs-alignment/walk` P0 gate (`STALE_DOCS_DRIFT`). Verified the drift is
benign — the semver-locked subcommand enum is unchanged, only a doc comment moved,
`test_body_hash` stable — then rebound via `reposix-quality doc-alignment bind --grade
GREEN` (hashes file bytes only, no network, OP-1-respected). Matches the row's own
prior rebind rationale.

**The phase is NOT closed.** No named-incident post-mortem exists for either wave —
both closed clean. What remains is the verification tail: gsd-verifier, real-backend
SC1/SC2 acceptance, intake filing, and the STATE.md cursor advance.

## 3. Binding constraints (unchanged)

- One tree-writer at a time (single-writer discipline, ORCHESTRATION §2).
- **ONE cargo invocation machine-wide** — prefer `-p reposix-core` / `-p reposix-cache`
  over `--workspace`; `cargo-nextest` is **not installed** in this environment
  (filed as GTH-10, do not re-file) — use `cargo test`.
- Leaf reposix/sim/git test setup MUST run in a throwaway `/tmp` clone with `cd` in the
  SAME Bash invocation — mechanically enforced by `.claude/hooks/leaf-isolation-guard.sh`
  (fail-closed, exit 2).
- No `--no-verify`, ever.
- No tag push — manager/L0 cuts tags, not this coordinator.
- **TARGETED staging only** (`git add <specific path>`) — never `git add -A`/`.`; never
  touch `.planning/MANAGER-HANDOVER.md` / `.planning/SESSION-HANDOVER.md`.
- Push cadence: `git push origin main` BEFORE the verifier subagent; if push is
  REJECTED because origin advanced, `git pull --rebase origin main && git push` —
  NEVER force, NEVER stash the manager's concurrent work.
- CI-confirmation is **inline bounded polling only, ≤1h cap** — **no background
  self-resume watchers** (owner directive 2026-07-15; a background watcher died
  silently last rotation and stalled ~2h — do not repeat that pattern). Confirm
  `code/ci-green-on-main` (P0, via `python3 quality/runners/run.py --cadence
  post-push --persist`) GREEN before ever opening the verifier or the next step.
- Commit-trailer format: `Co-Authored-By: Claude <model> <noreply@anthropic.com>` +
  `Claude-Session: <tag>`.
- Model tiering: dispatch gsd-verifier next, then gsd-executor (sonnet default) for
  SC1/SC2 runs / STATE.md advance / intake filing; opus only if a follow-up fix is
  genuinely complex (e.g. the pre-ADF list-path fallback in §5/§6 item 5, if needed).

## 4. Litmus / gate / REOPEN state

- No litmus gate is currently in a REOPEN state for P114.
- Wave-1 and Wave-2 both closed with clean CI-green pushes; no open waiver on either.
- Post-push `code/ci-green-on-main` (P0) probe: **CONFIRMED GREEN** against `12a0f57`
  (run `29435883070`, success) — this is settled, the successor does not need to
  re-derive it, only spot-re-confirm before dispatching the verifier (main could in
  theory move again before pickup).
- Real-backend acceptance gates (SC1 live TokenWorld checkout, SC2
  `quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh`) have **not yet
  been run this phase** — they gate phase-close, after the verifier, not before.
- The pre-push timing "regression" flagged in the prior (stale) handover §4/§5 is
  **RESOLVED as a non-issue** — see §5 RESOLVED-reframe below; do not re-raise it as
  open, but DO file the underlying stale-budget-doc GTH (§5 below).

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

**Batched — 4 GTH candidates, verified NOT yet filed in `.planning/GOOD-TO-HAVES.md`
(confirmed via grep at relief time) — do NOT drop:**

1. **`cargo doc` not warning-clean** in the reposix-cache/reposix-cli family
   (pre-existing, not FIX-02's own files): broken intra-doc links at `attach.rs:23`,
   `init.rs:33-34`, `cache.rs:596`, `sink_egress`; unclosed HTML tags at
   `main.rs:250`, `sync_tag.rs:8`. Severity LOW; sketch = doc-warning sweep.
   (Executor-1 NOTICED #2.)
2. **`oid_map` SQL-lookup idiom duplicated** across ≥4 test files; extract a shared
   `tests/common/mod.rs::oid_hex_for` helper. Severity LOW. (Executor-1 NOTICED #3.)
3. **Re-baseline the pre-push budget doc** — `quality/CLAUDE.md` § Cadences claims
   pre-push ≈55s / WARNs at ~60s, but real runs measured 104s and 191s, dominated by
   `code/shell-coverage` kcov (68–100s). The 60s WARN is always-firing noise and its
   "suspect a new whole-repo gate" advice misdirects (kcov IS the standing cost).
   Sketch = re-baseline the documented figure + WARN threshold to observed
   kcov-dominated ~100s. Severity LOW-MED. (Push-executor NOTICED #2.)
4. **`/reposix-quality-refresh` recovery hint** (printed by the docs-alignment
   walker) names a top-level-only command unreachable inside `gsd-executor`; the
   reachable primitive is the `doc-alignment bind` verb. Mild UX seam — the walker's
   printed recovery hint should name the subagent-reachable primitive. Severity LOW.
   (Push-executor NOTICED #3.)

**Already filed, verified present in `.planning/GOOD-TO-HAVES.md` — do NOT re-file:**
GTH-10 (cargo-nextest not installed), GTH-11 (cursor `body-format=` guard could
false-skip on a foreign substring).

**RESOLVED reframe (record so it isn't re-raised as open):** the prior (stale)
handover flagged a pre-push timing "regression" (Wave-1 127s vs ~91s baseline).
RESOLVED — NOT a regression: Wave-2's push landed ~104s (below the ~120s line); the
one-off 191s first-push was kcov variance PLUS a transiently-blocked walk gate, not
diff-driven. The real, still-open finding is the stale budget doc, filed as GTH
candidate #3 above.

**Process note (blocker pattern the successor may re-hit):** editing a doc-comment
INSIDE a bound doc-alignment region (e.g. `main.rs` `Sync` clap doc, L39–319, bound by
row `docs/decisions/009-stability-commitment/cli-subcommand-surface`) drifts that
row's stored `source_hash` and the pre-push `docs-alignment/walk` P0 gate REJECTS the
push (`STALE_DOCS_DRIFT`). Recovery: verify the drift is benign (semver-locked content
unchanged, only prose moved), then `reposix-quality doc-alignment bind --grade GREEN`
(hashes file bytes, no network, OP-1-safe) and commit the updated
`quality/catalogs/doc-alignment.json`. This is the documented benign-drift→rebind
pattern already exercised once this phase (`12a0f57`).

**Key open risk, not yet resolved (carried from Wave-1, still live):** if live
TokenWorld page `7766017` turns out to be pre-ADF (Confluence storage format, not
ADF-native), the Wave-1 fix (`body-format=atlas_doc_format` on the list path, no
storage fallback) does NOT cover it → `OidDrift` persists → SC1 goes RED. This is a
NAMED/EXPECTED risk (`114-RESEARCH.md` OQ1), not a surprise — resolvable only by
actually running the live SC1 checkout (§6 step 4).

## 6. Precise next steps (successor runbook) — the phase is NOT closed

1. Confirm main is still GREEN (`gh run list --branch main --workflow ci.yml --limit
   1`) before touching anything — expect `12a0f57` success. Note: this handover
   commit will be a LOCAL commit ahead of origin until the successor's first push;
   carry it forward, never drop it.
2. Dispatch **gsd-verifier** → `VERIFICATION.md`, goal-backward catalog-row grading
   (SC4 = reconcile-non-recovery, backed by the `DriftingMock` test + the corrected
   docs; all artifact-based rows). RED loops back to the offending wave, not to L0.
3. Push the verifier's `VERIFICATION.md` (targeted staging), inline-poll CI to green
   (≤1h cap, NO background self-resume watcher — hold the wait synchronously; a
   background watcher died silently last rotation and stalled ~2h), re-run
   `python3 quality/runners/run.py --cadence post-push --persist`, confirm
   `code/ci-green-on-main` P0 GREEN.
4. **Real-backend SC1/SC2 acceptance (only after verifier GREEN):** `source .env` in
   the SAME Bash invocation as the gate/run.py call; run
   `scripts/refresh-tokenworld-mirror.sh` FIRST as a pre-step (else mirror-lag
   false-negative). SC1 = live TokenWorld `git checkout -B main` INCLUDING page
   `7766017` → zero oid-drift abort. SC2 =
   `quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh` GREEN.
   Protected fixture pair `7766017`/`7798785` — NEVER delete (repair tool
   `scripts/confluence_tokenworld.py`, refuses to delete those two ids). If
   creds/substrate absent (no `.env` creds / default allowlist), report NOT-VERIFIED
   HONESTLY — never fake or skip-as-pass, never hit a real backend without creds.
5. **KEY SC1 RISK (research OQ1) — flag loud, do NOT hack around:** if page
   `7766017` is PRE-ADF (Confluence storage format, not ADF-native), the Wave-1 fix
   (`body-format=atlas_doc_format` on the list path, no storage fallback) does NOT
   cover it → `OidDrift` persists → SC1 goes RED. That is a NAMED/EXPECTED risk, not
   a surprise — report honestly and file a follow-up (list-path storage-fallback fix)
   as a phase/surprise; DO NOT silent-patch or hack around. Resolvable only by the
   live run.
6. Advance `.planning/STATE.md` (cursor + `completed_phases` → P114 complete) as the
   FINAL committed step, only after steps 2–5 confirm GO.
7. File the 4 batched GTH candidates from §5 (into `.planning/GOOD-TO-HAVES.md`,
   targeted staging) as part of intake disposition at phase-close — they are noticed
   but not yet routed, per OP-8's "eager-fix or file, never silently skip."
8. Report to L0: final verdict, this handover's commit SHA for provenance, and the
   intake disposition (4 new GTHs filed, GTH-10/GTH-11 confirmed already filed — do
   not re-file).
