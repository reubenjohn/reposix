---
phase: 123-quality-runner-catalog-integrity-hardening
milestone: v0.15.0
verifier: independent phase-close verifier (zero session context beyond the charter)
verified_at: 2026-07-18T14:00:00Z
head: e9df5d7ca9fb155d23f8a2c74a0f9f1401afafdc
overall: GREEN
score: 5/5 success criteria PASS (SC5b real-backend LEG NOT-VERIFIED by-design)
post_push_cadence: PASS — 1 PASS / 0 FAIL / 0 P0-P1 RED, exit 0
requirements: DRAIN-01, DRAIN-03, DRAIN-04, DRAIN-05, DRAIN-06, DRAIN-10 — all SATISFIED
---

# Phase 123 — Quality-runner & catalog integrity hardening: independent phase-close verdict

**Goal:** `quality/runners/run.py` and the catalog it persists resist false-greens, silent
corruption, and misleading errors.

**Ground truth verified:** `HEAD == origin/main == e9df5d7c`, working tree clean. CI GREEN
on `e9df5d7c` (`ci.yml` run `29647943849` = success, `release-plz.yml` run `29647943945` =
success — both re-confirmed live via `gh run list`).

**Method:** ran every gate / selftest / unittest against reality; did not trust SUMMARY
claims. All commands + observed output below.

---

## Overall: GREEN — 5/5 SC PASS

| SC | Verdict | One-line basis |
|----|---------|----------------|
| SC1 | **PASS** | `run.py` self-sources `./.env` (present-only, non-clobbering); confirmed LIVE (sourced 11 vars, KEY-NAMES-only) |
| SC2 | **PASS** | `--persist` refuses committed-GREEN → FAIL/PARTIAL without `--allow-downgrade`; end-to-end test + live gate |
| SC3 | **PASS** | advisory `flock` serializes concurrent `--persist`; real two-process no-lost-update proof (~4.3s wall) |
| SC4 | **PASS** | `structure/verifier-script-exists.sh` gate; full-truth-table selftest + live gate (155 in-scope, 0 violations). **Graded-outcome scope: SOUND** |
| SC5 | **PASS** | `code/ci-green-on-main` watches `(ci.yml release-plz.yml)`; t4 surfaces real oid-drift stderr. SC5b real-backend LEG NOT-VERIFIED by-design |

---

## Per-SC evidence

### SC1 — `run.py` self-sources `.env` (DRAIN-03) → PASS

- **Code path (wired):** `run.py:53` imports `_env_load`; `run.py:466`
  `_env_load.load_dotenv_if_present(REPO_ROOT)` runs FIRST in `main()`, before any catalog
  load / real-backend gating. Impl `quality/runners/_env_load.py`: `os.environ.setdefault`
  per key (existing env wins), `export ` whole-token strip, matched-quote strip, present-only
  no-op when absent, ONE stderr diagnostic naming KEY NAMES only.
- **Unit proof:** `TestEnvSelfSourcing` — `Ran 5 tests ... OK`
  (`test_already_set_env_wins`, `test_export_prefixed_line_loads_bare_key` [loaded
  `GITHUB_TOKEN, BARE_KEY, exportFOO` — confirms `export KEY=` → `KEY` while `exportFOO=`
  stays `exportFOO`], `test_missing_env_is_silent_noop`, `test_no_secret_value_reaches_stderr`,
  `test_unset_keys_are_loaded`).
- **LIVE against reality:** my `run.py --cadence post-push --persist` printed
  `run.py: sourced 11 var(s) from ./.env: ATLASSIAN_API_KEY, ..., GITHUB_TOKEN,
  REPOSIX_ALLOWED_ORIGINS, ...` — KEY NAMES only, no values (secret hygiene confirmed).
- **gh-shadow residual (charter item):** `.env`-sourced `GITHUB_TOKEN` was present yet
  `code/ci-green-on-main` still graded **PASS**, because `ci-green-on-main.sh:121` runs `gh`
  under `env -u GH_TOKEN -u GITHUB_TOKEN` — the SC1 sharp-edge mitigation working live. The
  **broader per-gate gh-auth audit is FILED** in
  `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md:1059+` (7 gh-shelling gates
  classified; 2 local/on-demand candidates for `env -u`, 1 CI-cron gate that must NOT strip)
  as a **P127 candidate** — confirmed present. Acceptable disposition, not a P123 blocker.

### SC2 — `--persist` refuses committed-GREEN downgrade (DRAIN-04) → PASS

- **Code path (wired):** `run.py:54` imports `_persist_guard`; `run.py:564-585` reads the
  committed baseline via `committed_head_statuses` (`git show HEAD:<catalog>`, NOT the dirty
  worktree), calls `refuse_downgrade_without_flag`, and on violation without
  `--allow-downgrade` prints a teaching refusal (row id + old→new + copy-paste recovery) and
  skips `save_catalog`; `run.py:617-618` forces `exit_code = max(exit_code, 1)` so a blocked
  downgrade can never ride a green exit. `_GREEN={PASS,WAIVED}`, `_REGRESSION={FAIL,PARTIAL}`;
  NOT-VERIFIED deliberately excluded (never a downgrade).
- **Exercised end-to-end (the documented test):** `TestPersistDowngradeGuard` —
  `Ran 9 tests ... OK`. These drive the REAL `run.main(--persist)` over a throwaway `/tmp`
  git repo with a real `git show HEAD:` baseline: `test_1` (PASS→FAIL refused, disk stays
  PASS, refusal names id/PASS/FAIL/`--allow-downgrade`, exit 1), `test_2`
  (`--allow-downgrade` persists FAIL with a loud notice), `test_3` (WAIVED→FAIL refused),
  `test_4` (brand-new row absent from HEAD mints freely), `test_6` (PASS→NOT-VERIFIED always
  allowed — deadlock-prevention), `test_7` (blocked P2 downgrade still forces non-zero exit).
- **Live gate:** `structure/persist-refuses-downgrade.sh` → `PASS ... EXIT=0`.

### SC3 — concurrent `--persist` cannot race-corrupt catalog (DRAIN-05) → PASS

- **Code path (wired):** `run.py:496-500` wraps the WHOLE per-catalog read-modify-write
  (`load_catalog` → grade → `save_catalog`) in `_persist_guard.catalog_persist_lock` on the
  `--persist` path, `contextlib.nullcontext()` otherwise. Impl: `fcntl.flock(LOCK_EX)` on
  `quality/reports/.persist.lock` (gitignored), no timeout (kernel releases on exit/SIGKILL).
- **Real concurrency proof:** `TestPersistCatalogLock` — `Ran 4 tests in 4.274s ... OK`.
  The ~4.3s wall-clock confirms genuine cross-process blocking (a mock would be ~0s):
  `test_1` a second REAL subprocess blocks ≥1.8s on the held flock; `test_2` validate-only
  never opens/contends the lock; `test_4` **two concurrent `run.py --persist` interpreters on
  the SAME catalog each flip a different row and BOTH flips survive** (the GTH-V15-01
  lost-update hazard, closed). Test asserts exactly what its name promises.
- **Live gate:** `structure/persist-catalog-write-locked.sh` → `PASS ... EXIT=0`.

### SC4 — `verifier-script-exists.sh` gate (DRAIN-06, GTH-V15-03) → PASS · scope SOUND

- **Selftest (full truth table):** `verifier-script-exists.selftest.sh` →
  `RESULT: 15 passed, 0 failed` (exit 0). Asserts the exact tightened contract:
  PASS/FAIL/PARTIAL + missing-file → VIOLATION; PASS + non-exec → VIOLATION;
  **PASS + null-script → VIOLATION (unbacked-PASS)**; WAIVED/NOT-VERIFIED + missing → EXEMPT;
  **FAIL/PARTIAL + null-script → EXEMPT**; exactly `5 violation(s) ... 9 rows seen, 3 exempt`.
- **Live gate (real catalogs):** `PASS: verifier-script-exists — 155 in-scope graded-outcome
  rows across 11 catalogs (172 rows seen, 17 exempt ...)`, exit 0. Zero live violations.

**SC4 / GTH-V15-03 graded-outcome soundness ruling (the load-bearing, second-independent grade):
SOUND.** Reasoning, backed by an *orthogonal* cross-check (a scan I wrote that does NOT reuse
the gate's logic and scans ALL catalogs including the two the gate skips):

- The named worry — a cadence-scoped backstop that a `PASS, null-script, cadences:["weekly"]`
  row could ride through — **does not apply**: the gate flags PASS+null-script **directly and
  cadence-agnostically** (`verifier-script-exists.sh` scans every row in every
  `quality/catalogs/*.json` regardless of the row's own cadence; the gate itself fires at
  `[pre-commit, pre-push, pre-pr]`). It does NOT rely on `run.py`'s cadence-scoped
  NOT-VERIFIED flip as the backstop — that flip is documented (gate header + `quality/CLAUDE.md`)
  as a SECONDARY defense only.
- **Independent scan result:** `0` PASS rows with null/absent `verifier.script` anywhere;
  `0` PASS rows with a missing/non-exec declared script; **`0` PASS+null-script rows scoped to
  weekly/post-release/on-demand**. The hole does not exist live, and would be caught if it did.
- The two exempted catalogs are genuinely a different schema, not a smuggling channel:
  `doc-alignment.json` (dim=`docs-alignment`) rows carry NO `status` field at all
  (distinct status = `{None}`; they use `last_verdict`/`next_action`, enforced independently
  by the `reposix-quality` binary + `docs-alignment/walk.sh`); `docs-reproducible-allowlist.json`
  is the `{ids, reasons}` non-row schema (no `rows`). Neither can carry a `status: PASS` green
  claim through this gate's schema.
- **Residual exemptions are correct, not holes:** WAIVED/NOT-VERIFIED and FAIL/PARTIAL+null
  rows assert *no green*, so a missing verifier there is not a false-green — precisely the
  GTH-V15-03 boundary (an unbacked *graded* green). Latent-detection note: a WAIVED row with
  a broken declared script is only caught the moment it flips to PASS (see NOTICED #2) — by
  design, acceptable.

### SC5 — required-workflow list + real oid-drift stderr (DRAIN-01/10) → PASS

- **SC5a (required-workflow LIST):** `ci-green-on-main.sh:68` `WORKFLOWS=("ci.yml"
  "release-plz.yml")` — no longer a single hardcoded `ci.yml`. Aggregation is sound:
  NOT-VERIFIED (unknowable) outranks FAIL (red) outranks PASS; tainted `gh` conclusion bytes
  routed through `json_object`/`json_array` (`python3 json.dumps`), never hand-interpolated.
  **Proven LIVE:** my post-push run graded the P0 row **PASS** — which REQUIRES *both* watched
  workflows' latest main runs = success (`ci.yml` `29647943849`, `release-plz.yml`
  `29647943945`, both re-confirmed `success` via direct `gh run list`).
- **SC5b (real oid-drift stderr):** `lib/t4-real-backend-flow.sh:26-41` `_t4_checkout_or_fail`
  captures ONLY stderr (`2>&1 >/dev/null`) and passes the REAL git error as the fail detail,
  replacing the misleading hardcoded "requires git >= 2.34 stateless-connect fetch" fallback
  (misleading by construction — the caller already verified git ≥ 2.34). Hermetic selftest
  `t4-real-backend-flow.selftest.sh` → `RESULT: 3 passed, 0 failed`; observed captured detail
  = `fatal: 'refs/reposix/origin/main' is not a commit and a branch 'main' cannot be created
  from it` (real stderr), and asserted to NOT contain the version fallback.
- **SC5b real-backend LEG:** graded **NOT-VERIFIED by-design** — the
  `agent-ux/t4-conflict-rebase-ancestry-real-backend` execution against TokenWorld is
  env-gated / milestone-close-scoped, not a phase-close gate. The gradeable P123 deliverable
  (the structural stderr fix) is proven hermetically. NOT-VERIFIED, not FAIL — correct.

---

## Post-push cadence (PROTOCOL.md Step 6/7 — the verifier's primary run)

```
$ python3 quality/runners/run.py --cadence post-push --persist
run.py: sourced 11 var(s) from ./.env: ATLASSIAN_API_KEY, ... , GITHUB_TOKEN, REPOSIX_ALLOWED_ORIGINS, ...
  catalog: code.json (9 rows; 1 in scope)
    [PASS         ] code/ci-green-on-main  (P0, 1.54s)
summary: 1 PASS, 0 FAIL, 0 PARTIAL, 0 WAIVED, 0 NOT-VERIFIED -> exit=0
```

- **P0 `code/ci-green-on-main` = PASS.** Latest `ci.yml` run on main (`29647943849`) concluded
  `success`, AND latest `release-plz.yml` (`29647943945`) concluded `success`.
- **Pass count:** 1 PASS / 0 FAIL / 0 PARTIAL. **No P0/P1 FAIL. Exit 0.**
- Tree stayed **clean** after the `--persist` run (unchanged-status rows roll back
  `last_verified`, so no catalog write / no dirty tree).

---

## Requirements coverage

| Req | SC | Status |
|-----|----|--------|
| DRAIN-03 | SC1 | SATISFIED |
| DRAIN-04 | SC2 | SATISFIED |
| DRAIN-05 | SC3 | SATISFIED |
| DRAIN-06 | SC4 | SATISFIED |
| DRAIN-01 | SC5a/SC5b | SATISFIED (SC5b real-backend leg NOT-VERIFIED by-design) |
| DRAIN-10 | SC5a | SATISFIED |

**QG-07 (per-phase CLAUDE.md update):** SATISFIED — `quality/CLAUDE.md` documents all of
CI-green-required-list, `.env` self-sourcing, verifier-script-existence scope; `quality/PROTOCOL.md`
documents `.env` self-sourcing, downgrade guard, concurrency lock. SURPRISES-INTAKE carries the
P127 gh-auth entry.

---

## NOTICED (OD-3 deliverable)

1. **INFO — `.env` real creds now effective in every `run.py` process.** SC1 (correctly) makes
   the operator's real backend creds (`ATLASSIAN_API_KEY`, `GITHUB_TOKEN`, `JIRA_API_TOKEN`,
   `REPOSIX_ALLOWED_ORIGINS`, `CARGO_REGISTRY_TOKEN`, …) live in `run.py`'s env whenever `./.env`
   is present. Not a new attack surface (creds already on disk; gates are trusted first-party),
   and OP-1 fail-closed is unchanged. The gh-shadow slice is audited (P127). The broader
   "run.py now hydrates real creds for all downstream gates" property is documented in the
   `_env_load.py` docstring + intake — worth a one-line security sign-off at milestone-close,
   not a blocker.
2. **INFO — SC4 latent (not eager) detection for WAIVED/NOT-VERIFIED rows.** A WAIVED row with a
   missing/non-exec declared `verifier.script` is exempt until it flips to PASS, at which point
   the gate catches it. Sound per the graded-outcome scope (a WAIVED row asserts no green), but
   a broken script can sit undetected across the waiver window. By design; acceptable.
3. **INFO — `ci-green-on-main` `none` verdict path unexercised live.** Both watched workflows
   currently have runs, so the "workflow never fired → NOT-VERIFIED" branch (release-plz race)
   isn't hit in practice; its logic + comment are sound and defensively correct.

No BLOCKER or WARNING findings. No gaps. STATE may advance.

---

_Independent verifier — verified against reality (ran every gate, hit the live GitHub API,
exercised the guards in throwaway repos). Verdict artifact only; STATE-advance + final push
left to the coordinator's separate executor._
