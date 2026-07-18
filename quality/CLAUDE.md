# quality/CLAUDE.md — quality-gates rules (auto-loaded under quality/)

Extends root `CLAUDE.md`. **Read `quality/PROTOCOL.md` first** for the runtime contract.

## Catalog-first rule

Every phase's FIRST commit writes the catalog rows defining that phase's GREEN contract;
later commits cite the row id. The unbiased verifier subagent reads catalog rows that
existed BEFORE the implementation landed.

**Mint `minted_at` in that same first commit.** Any row minted with `last_verified` >=
2026-07-05 but no `minted_at` is rejected at catalog load by
`quality/runners/_audit_field.py::validate_row` (invoked from `run.py:121`) — add it
up front rather than discovering the rejection on your first pre-push run.

**`doc-alignment.json` (and every catalog under `quality/catalogs/`) rows are mint-only —
never hand-edit.** Rows are minted via `reposix-quality doc-alignment <verb>` (`bind` /
`mark-missing-test` / `propose-retire`) or the `/reposix-quality-refresh` fan-out, never
typed directly into the JSON. Hand-editing an enum field (`next_action`, `last_verdict`)
or a row's shape breaks `reposix-quality` at load time — and it breaks in serde/validation
layers, NOT JSON-syntax validation, so a hand-edit that "looks fine" (valid JSON) can still
be structurally wrong in ways `jq`/`python -m json.tool` will never catch: (1) an enum
field carrying a token outside the real variant set (`NextAction` is exactly
`WRITE_TEST`/`FIX_IMPL_THEN_BIND`/`UPDATE_DOC`/`RETIRE_FEATURE`/`BIND_GREEN` —
`crates/reposix-quality/src/catalog.rs:373-386`; a hand-typed placeholder like `BIND` is
not a member and fails `unknown variant`), (2) a required field silently omitted
(`last_verdict: RowState` has no `serde(default)` — a hand-added row missing it fails
`missing field`), (3) the `tests`/`test_body_hashes` and `source`/`source_hashes`
parallel-array invariants (`Row::validate_parallel_arrays`) going out of sync. Because
every `reposix-quality` invocation loads the whole catalog, ONE corrupted row blocks
every doc-alignment command for every row, not just the corrupted one — filed as
`SURPRISES-INTAKE.md` 2026-07-16 (P117 W2 push-blocker: a hand-typed `BIND` token plus a
missing `last_verdict` plus a parallel-array mismatch, stacked in the same hand-mint row).
The fix path for a corrupted row is re-mint via `/reposix-quality-refresh` (or the
specific `reposix-quality doc-alignment <verb>`), not a hand-edit of the JSON — a hand-edit
that happens to unblock one serde error commonly just exposes the next one underneath.

## Verifier-subagent dispatch

Phase close MUST dispatch an unbiased subagent that grades catalog rows from committed
artifacts with zero session context — the executing agent does not grade itself. Verbatim
prompt: `quality/PROTOCOL.md` § "Verifier subagent prompt template". The milestone-close
9th probe (`run.py --cadence pre-release-real-backend`, exit 0) is non-skippable and
never carries a waiver.

## Taxonomy — 9 dimensions / 8 cadences / 6 kinds

Adding a gate = one catalog row + one verifier in `quality/gates/<dim>/`; the runner
discovers + composes by tag. No new top-level script, no new pre-push wiring. Rows carry
`cadences: list[str]` (a gate may fire at multiple triggers). Full schema:
`quality/catalogs/README.md`. Pivot journal: `quality/SURPRISES.md` (append-only, ≤200
lines). When an owner catches a quality miss: fix it, update the relevant CLAUDE.md /
ORCHESTRATION.md, AND tag the dimension (routes to the right catalog + `gates/<dim>/`).

**Dimensions:**

| Dimension | Checks |
|---|---|
| code | clippy, fmt, cargo nextest, shell-coverage (kcov aggregate, ratchet floor) |
| docs-alignment | claims have tests; hash drift detection |
| docs-build | mkdocs strict, mermaid renders, link resolve, badges resolve |
| docs-repro | snippet extract, container rehearse, tutorial replay |
| release | gh assets present, brew formula current, crates.io max version, installer bytes |
| structure | freshness invariants, banned words, top-level scope |
| agent-ux | dark-factory (sim + DVCS third arm, both `mechanical`) + reposix-attach + bus URL prechecks + webhook YAML + test-name-vs-asserts honesty gate |
| perf | latency, token economy |
| security | allowlist enforcement, audit immutability |

**Shell-coverage ratchet:** `code/shell-coverage` (kcov, `gates/code/shell-coverage.sh`)
grades an *aggregate* line-coverage % over the whole in-scope shell corpus (never-run
scripts counted at 0%) against the floor in `quality/shell-coverage-floor.txt` — raise it
over time, never above the measured aggregate to force-pass. Needs `kcov` — on modern
distros (noble dropped it from apt) install the prebuilt release binary or build from
source (see the `shell-coverage` job in `.github/workflows/ci.yml`); `sudo apt-get install
-y kcov` still works on focal. The CI `shell-coverage` job uploads a `shell`-flag cobertura to Codecov.
Follow-up (documented, left at 0%): the cargo/sim-dependent gates are unexercised by the
harnesses, so they sit at 0% and drag the aggregate down — closing them is deferred.

**CI-green-on-main required-workflow list (P123/SC5a, DRAIN-01):**
`code/ci-green-on-main` (`gates/code/ci-green-on-main.sh`) watches a required-workflow
LIST, not one hardcoded workflow — `WORKFLOWS=(ci.yml release-plz.yml)`, the only two
confirmed (by reading the workflow files) to fire unconditionally on `push: branches:
[main]` with no path filter. Before adding a THIRD workflow, re-verify its trigger shape
first — the script's header comment names why `audit.yml` (path-filtered) and
`docs.yml`/`quality-post-release.yml` (`workflow_run`-triggered, not `push`) are
deliberately excluded. Aggregation: NOT-VERIFIED (any watched workflow unknowable — gh
error, no run found, still in-progress) always outranks FAIL (a red workflow) when
rolling up the per-workflow verdicts to one row grade.

**Cadences:** `pre-commit` (<2s) · `pre-push` (<60s) · `pre-pr` (PR CI, <10min) ·
`weekly` (cron, alerting) · `pre-release` (on tag, <15min) · `post-release` (alerting) ·
`on-demand` · `pre-release-real-backend` (local + milestone-close, env-gated, mandatory
at tag-time; default-skips to NOT-VERIFIED when `REPOSIX_ALLOWED_ORIGINS` + creds unset —
never skip-counts-as-pass, per PROTOCOL.md OD-2).

**Runtime does NOT scale with diff size.** Both budgets are fixed whole-repo costs, not
per-changed-file costs — measured 2026-07-12: pre-commit ≈0.7s (fmt --check on staged
`.rs` only, ~0.5s + runner ~0.2s — the only piece that scales at all, and only with
staged file *count*, not diff size). pre-push ≈55s, dominated by fixed-cost gates that
walk the whole tree regardless of what changed: `code/shell-coverage` (kcov aggregate,
~29s), `agent-ux/rebase-recovery-reconciles` (~9s), `docs-build/mkdocs-strict` (~2s),
full-workspace clippy/fmt (~1s combined). A one-line commit and a 500-file commit pay
the same pre-push tax. If pre-push ever creeps meaningfully past 60s, suspect a new
whole-repo gate (another kcov-style full-corpus walk), not diff growth — profile with
`python3 quality/runners/run.py --cadence pre-push` (per-row timing above) before
adding budget.

**Kinds:** `mechanical` · `container` · `asset-exists` · `subagent-graded` · `manual`
(TTL freshness) · `shell-subprocess` (real subprocess vs a real binary/backend + a
transcript at `quality/reports/transcripts/<row-slug>-<RFC3339>.txt` recording argv +
env_keys [NAMES only — no `=value`] + cwd + exit_code + stdout/stderr; transport-claim
rows MUST invoke a real reposix binary or backend endpoint). Grading contract:
PROTOCOL.md § "For rows with `kind: shell-subprocess`".

## Honesty rules (P90 RBF-FW-06..12)

Full detail: `quality/PROTOCOL.md` § "New runner/validator semantics" + "Verifier
exit-code conventions". In brief: `minted_at` (RFC3339, write-once) anchors the
audit-cutoff for `claim_vs_assertion_audit` (legacy pre-P90 rows fall back to
`last_verified`; exemption retires P95). Transport/perf rows carry `coverage_kind`
(`real-backend` or `WAIVED + until_date`, never bare PASS-with-comment). A missing
verifier script flips a row to `NOT-VERIFIED` unconditionally (never preserves prior
PASS; `error: verifier-not-found` marker). Env-gated skip fails closed to `NOT-VERIFIED`
but preserves `last_real_grade` + `skip_reason: env-missing`. `agent-ux/test-name-vs-asserts`
requires every `expected.asserts` entry to map to an `asserts_passed` string (closes the
"test name lies" class). Milestone-close dispatches two independent honesty spot-checks
from `quality/dispatch/`: `absorption-honesty-spot-check.md` (author ≠ orchestrator) and
`milestone-adversarial.md` (fresh subagent grades whether each row's assertion falsifies
its own description).

**Marker placement window (test-name-honesty).** The `// test-name-honesty: ok — <reason>`
marker (and the `#[test]`/`#[ignore = "real-backend..."]` gate) MUST sit within a **6-line
lookback window ending at the fn signature** — i.e. the 6 lines immediately ABOVE the
`fn ...(` line, or a trailing comment on the signature line itself. `test-name-vs-asserts.sh`
uses a fixed `CONTEXT_LINES=6` lookback; a marker (or the `#[test]` attribute) pushed farther
than 6 lines above the fn — e.g. separated by a long `///` doc-comment block — is **silently
ignored** (and if the `#[test]` attribute itself drifts out of the window the fn is skipped as
a non-test, so a dishonest name passes with no RAISE). Keep the marker hugging the signature.
Tight placement is deliberate; a preamble-anchored scan that removes the distance constraint is
a filed GOOD-TO-HAVE (v0.13.0), not current behavior. Full example + rationale: the gate's
header comment.

## Structure-dimension gates (P89)

`banned-production-tokens.sh` + `deferral-pointer-linter.sh` — both catalogued in
`quality/catalogs/freshness-invariants.json` (wrapper `"dimension": "structure"`; there
is no `structure.json`). Rule + regex scope: `crates/CLAUDE.md` § code conventions.

### File-size limits

`structure/file-size-limits.sh` (row `structure/file-size-limits` in
`freshness-invariants.json`) is the progressive-disclosure/readability gate. It walks
`git ls-files`, skips exclusions (`Cargo.lock`, `quality/catalogs/*.json`,
`quality/reports/`, `crates/*/tests/fixtures/`, `CHANGELOG.md`, `crates/*.rs`), and
assigns a per-extension ceiling:

| Path / extension | Ceiling |
|---|---|
| `CLAUDE.md` (any dir) | 40000 |
| `.claude/skills/**` | 10000 |
| `*.md` | 20000 |
| `*.rs` (outside `crates/`) | 20000 |
| `*.py` | 15000 |
| `*.sh` / `*.bash` | 10000 |

`pct = size*100/limit` (integer) sorts each file into one of two tiers:

- **75 ≤ pct < 100 — EARLY-WARNING (print-only).** A non-blocking WARN summary to
  stderr: a header naming the band count, then band files sorted by pct DESC (top-12,
  then `… and N more at ≥75%`). ALWAYS emitted, independent of `--warn-only`/the waiver,
  and NEVER touches the exit code. There is **no WARN verdict status** in `verdict.py` —
  this is print-only (precedent: the `.githooks/pre-push` timing tripwire's stderr `WARN`
  that never affects exit). It is the early-warning gate the owner asked for: files
  approaching the ceiling, not yet over it.
- **pct ≥ 100 (size > limit) — OVER-BUDGET (blocking).** `exit 1` by default; only this
  tier's exit is governed by `--warn-only` (flips to `exit 0`). The row is **WAIVED until
  2026-08-08** with verifier args `["--warn-only"]`, so today the over-budget tier warns
  instead of blocking; when the flag/waiver lapse the block-later contract reactivates.
  Live residual list + drain plan: the waiver `reason` on the row.

Self-test (both tiers + `--warn-only` independence + overflow): `bash
quality/gates/structure/file-size-limits.selftest.sh` (builds a throwaway `/tmp` repo —
never the shared repo).

### Verifier-script existence

`structure/verifier-script-exists.sh` (row `structure/verifier-script-exists` in
`freshness-invariants.json`, P123/SC4, DRAIN-06, closes GTH-V15-03) is the
framework-integrity gate: a catalog row could otherwise mint a graded verdict backed by a
`verifier.script` that doesn't exist on disk (or exists but lacks the executable bit),
and nothing structurally caught it. It scans every `quality/catalogs/*.json` row
(real `python3 -c` JSON parsing, not grep), EXCLUDING files named `*-allowlist.json`
(a different, non-row schema) and any catalog whose wrapper `dimension ==
"docs-alignment"` (that dimension's rows carry no `verifier.script` field at all — see
"Docs-alignment dimension" below). Three violation classes, each printed on its own line
with catalog + row id + path + a concrete fix: MISSING-FILE (the path doesn't resolve to
a file), NON-EXECUTABLE (the file exists but lacks `+x`), and UNBACKED-PASS (a `status:
PASS` row that declares NO `verifier.script` at all — a graded green with no verifier that
could have produced it).

**Scope = GRADED OUTCOMES only** (refined 2026-07-18, P123 close, coordinator design call
per DP-5 — see `claim_vs_assertion_audit`). The check fires ONLY for a row whose `status`
asserts a run result — `{PASS, FAIL, PARTIAL}` (the canonical graded-outcome triple,
mirror of `run.py`'s real-grade set). A row that asserts **no green** is EXEMPT: status
`WAIVED`/`NOT-VERIFIED` (the `STALE` display-flavor persists as `NOT-VERIFIED`), OR a
`FAIL`/`PARTIAL` row with `verifier.script: null` (it asserts no green, so a missing
verifier is not a false-green). **But a `status: PASS` with `verifier.script: null` is
FLAGGED** (UNBACKED-PASS, refined 2026-07-18 P123-close code review): the row asserts a
green with no verifier that could ever have produced it — exactly GTH-V15-03. This gate
flags it DIRECTLY and cadence-agnostically; do **not** rely on the runner's NOT-VERIFIED
flip as the backstop, because `run.py` only grades rows in-scope for the RUNNING cadence,
so a `PASS`+null-script row scoped to (e.g.) `weekly` is never re-graded at pre/post-push
and its unbacked PASS rides green. The runner flip is a SECONDARY, cadence-scoped defense,
never a substitute for this static check. **Why:** GTH-V15-03's hazard is an *unbacked
graded claim* (a false-green riding on a verifier that can't run); WAIVED/NOT-VERIFIED and
FAIL/PARTIAL null-script rows make no green claim, so a missing verifier there is not a
false-green. This
refinement (path (b) of the filed intake) resolved the 5 pre-existing catalog-first stubs
(2 WAIVED cross-platform rehearsals, `docs-build/animation-renders`, 2
`docs-repro/benchmark-claim/*`) — all now EXEMPT because none claims a graded outcome — so
the row grades **PASS for real** (155 in-scope graded rows, 0 violations, 17 exempt) and is
now tagged **`[pre-commit, pre-push, pre-pr]`** (a clean P1 row is safe repo-wide). The 32
chmod-+x fixes on graded rows remain in-scope and enforced. The 5 deferred verifiers stay
independently tracked (P97 / 117-07 W5 / GOOD-TO-HAVES-04); building them is now purely
additive.

Self-test (full truth table — PASS/FAIL/PARTIAL + missing/non-exec → violation, each
individually named; WAIVED/NOT-VERIFIED + missing → exempt; PASS + null-script → violation
(UNBACKED-PASS); FAIL/PARTIAL + null-script → exempt; all-good → pass): `bash
quality/gates/structure/verifier-script-exists.selftest.sh` (throwaway `/tmp` fixture repo
— never the shared repo, never the real catalogs).

## Docs-alignment dimension

Binary: `reposix-quality doc-alignment {bind, propose-retire, confirm-retire,
mark-missing-test, plan-refresh, plan-backfill, merge-shards, walk, status}`. Run
`status --top 10` for gap targeting. Pre-push gate: `gates/docs-alignment/walk.sh`. Two
axes: `alignment_ratio` (bound / non-retired) + `coverage_ratio` (lines_covered /
total_eligible); the walker BLOCKs when either drops below floor. Recovery:
`/reposix-quality-backfill` (full extraction) or `/reposix-quality-refresh <doc>` (single
doc). Both slash commands are top-level only (depth-2 fan-out unreachable inside
`gsd-executor`). Full spec: `quality/catalogs/README.md` § "docs-alignment dimension".

**Verification walks MUST use the wrapper.** Always run `bash
quality/gates/docs-alignment/walk.sh` (grades against a `/tmp` copy) — never invoke the
raw `reposix-quality doc-alignment walk` subcommand directly, which mutates the committed
catalog's coverage/summary counters as a side effect.
