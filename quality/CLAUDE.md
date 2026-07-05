# quality/CLAUDE.md — quality-gates rules (auto-loaded under quality/)

Extends root `CLAUDE.md`. **Read `quality/PROTOCOL.md` first** for the runtime contract.

## Catalog-first rule

Every phase's FIRST commit writes the catalog rows defining that phase's GREEN contract;
later commits cite the row id. The unbiased verifier subagent reads catalog rows that
existed BEFORE the implementation landed.

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

**Cadences:** `pre-commit` (<2s) · `pre-push` (<60s) · `pre-pr` (PR CI, <10min) ·
`weekly` (cron, alerting) · `pre-release` (on tag, <15min) · `post-release` (alerting) ·
`on-demand` · `pre-release-real-backend` (local + milestone-close, env-gated, mandatory
at tag-time; default-skips to NOT-VERIFIED when `REPOSIX_ALLOWED_ORIGINS` + creds unset —
never skip-counts-as-pass, per PROTOCOL.md OD-2).

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

## Structure-dimension gates (P89)

`banned-production-tokens.sh` + `deferral-pointer-linter.sh` — both catalogued in
`quality/catalogs/freshness-invariants.json` (wrapper `"dimension": "structure"`; there
is no `structure.json`). Rule + regex scope: `crates/CLAUDE.md` § code conventions.

## Docs-alignment dimension

Binary: `reposix-quality doc-alignment {bind, propose-retire, confirm-retire,
mark-missing-test, plan-refresh, plan-backfill, merge-shards, walk, status}`. Run
`status --top 10` for gap targeting. Pre-push gate: `gates/docs-alignment/walk.sh`. Two
axes: `alignment_ratio` (bound / non-retired) + `coverage_ratio` (lines_covered /
total_eligible); the walker BLOCKs when either drops below floor. Recovery:
`/reposix-quality-backfill` (full extraction) or `/reposix-quality-refresh <doc>` (single
doc). Both slash commands are top-level only (depth-2 fan-out unreachable inside
`gsd-executor`). Full spec: `quality/catalogs/README.md` § "docs-alignment dimension".
