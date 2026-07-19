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
This bites LEGACY rows too, not just new mints: a pre-P90 row with **no `minted_at`**
whose verifier re-grades for real (e.g. an env/git-version precondition that was
short-circuiting now passes) stamps a fresh `last_verified` >= the cutoff, and the reject
is an **uncaught `SystemExit` in `load_catalog`** — it crashes ALL of `run.py` before
grading, for every cadence touching that catalog (incl. the milestone-close 9th
`pre-release-real-backend` probe), not just that one row. So proactively add `minted_at`
to any legacy row whose verifier can start executing (P126 W1 defused
`agent-ux/real-git-push-e2e` this way; root cause: `surprises-intake/part-08.md` #2). The
whole-corpus invariant is CI-guarded by `test_run.py::TestNoArmedMintedAtLandmine` — it
FAILs if any row carries `last_verified` >= the P90 cutoff without `minted_at`.

**Validate/read cadences are byte-for-byte read-only — STRUCTURALLY, not by convention.**
Only an explicit `--persist` MINT run may write `quality/catalogs/`; a bare cadence GATE
run (pre-commit/pre-push/pre-pr) grades in memory and writes per-row artifacts but never
the catalog (`structure/catalog-immutable-on-read`). As of P126 W1 the contract lives at
the WRITE BOUNDARY: `run.py::save_catalog` takes a **required `persist=` keyword** and
raises `RuntimeError` on a `persist=False` write, so a future refactor that reaches the
write in a non-persist path fails loud instead of silently round-tripping a catalog (the
class that staled + un-waived `subjective-rubrics.json`'s `headline-numbers-sanity` row
during a validate-only commit-hook run). Never write a catalog with raw
`json.dumps(indent=2)` (no `ensure_ascii=False`) — it `\u`-escapes em-dashes and reformats
the bytes; go through `save_catalog(..., persist=True)`. Regression:
`test_run.py::TestSaveCatalogPersistGuard` + `TestValidateOnlyMultiCatalogByteIdentical`
(dimension: catalog-integrity / structure).

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

**Two honesty layers (why a >15% counter-drift can be legitimate, not gaming):** the gate
runs (1) **kcov's runtime line instrumentation** — the coverage metric that grades the
aggregate floor — alongside (2) an independent **anti-gaming counter**, `scripts/
shell_coverage.py`'s static bash-aware `coverable_line_count`, cross-checked to stay within
15% of kcov's own coverable-line total (so a script can't be restructured to make kcov see
artificially few coverable lines). Both estimate the SAME "coverable lines" quantity by
different heuristics (the static parser's heredoc / `case`-arm / multi-line-continuation skip
rules vs kcov's interpreter-driven instrumentation), so on a SMALL script a few
structurally-ambiguous lines are a large PERCENT on a tiny absolute gap — e.g. `transcript.sh`
at counter=34 vs kcov=27 is a 7-line gap that reads as 25.9%. That drift **WARNs and flips
only the P2 counter-validation assert, never the aggregate-floor pass/fail**, so it is
non-blocking; a >15% drift on a small script is expected divergence, not evidence of a gamed
denominator (this is a *clarification of the mechanism*, not a fix of the drift — the
reconcile/threshold decision is tracked in `SURPRISES-INTAKE.md`'s `code/shell-coverage`
drift entries).

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

**`.env` self-sourcing (P123/SC1, DRAIN-03):** `quality/runners/run.py` conditionally
self-sources `./.env` (via the sibling `quality/runners/_env_load.py`) before grading,
present-only and non-clobbering (`os.environ.setdefault` per key — an operator/CI-exported
credential always wins over a `.env` value). This closes the false-green-preflight /
silent-skip gap where `scripts/preflight-real-backends.sh` sourced `.env` and reported
backends reachable, but `run.py` itself did not and silently skipped every real-backend
row to `NOT-VERIFIED`. Parsing is a deliberately minimal subset of shell `source`
(`KEY=value` and `export KEY=value` lines, `#`-comments, blank lines, matched quotes —
the leading `export ` token is stripped so an exported credential loads under its bare
key, not `"export KEY"`); it does not interpret expansion/substitution/multi-line values.
OP-1 fail-closed is unchanged — sourcing `.env` only makes creds-in-`.env` effective; a
real backend is still hit only when creds are present AND `REPOSIX_ALLOWED_ORIGINS` is
non-default. Because a `.env`-sourced `GITHUB_TOKEN`/`GH_TOKEN` can shadow an operator's
`gh auth login` keyring session, any LOCAL/on-demand gate that shells out to `gh` should
run it under `env -u GH_TOKEN -u GITHUB_TOKEN` (precedent: `code/ci-green-on-main.sh`) —
never strip those vars from a gate that runs only in a CI/cron cadence, where
`GITHUB_TOKEN` is the sole available credential (see `SURPRISES-INTAKE.md`'s P123-close
gh-auth audit entry for the per-gate classification).

**Cadences:** `pre-commit` (<2s) · `pre-push` (~90-120s, see § Runtime below) · `pre-pr` (PR CI, <10min) ·
`weekly` (cron, alerting) · `pre-release` (on tag, <15min) · `post-release` (alerting) ·
`on-demand` · `pre-release-real-backend` (local + milestone-close, env-gated, mandatory
at tag-time; default-skips to NOT-VERIFIED when `REPOSIX_ALLOWED_ORIGINS` + creds unset —
never skip-counts-as-pass, per PROTOCOL.md OD-2).

**Runtime does NOT scale with diff size.** Both budgets are fixed whole-repo costs, not
per-changed-file costs — measured 2026-07-12: pre-commit ≈0.7s (fmt --check on staged
`.rs` only, ~0.5s + runner ~0.2s — the only piece that scales at all, and only with
staged file *count*, not diff size). pre-push ≈90-120s (re-measured 2026-07-18/P124: 122s;
corroborated by P121 ~92s and P123 109-121s — the original ≈55s budget has been outgrown by
kcov-corpus growth + new whole-repo subprocess gates, NOT by diff size). Dominated by
fixed-cost gates that walk the whole tree regardless of what changed: `code/shell-coverage`
(kcov aggregate, ~65s — up from ~29s as the shell corpus grew, and the single largest cost
by far), `agent-ux/rebase-recovery-reconciles` (~15s), `docs-repro/container-rehearse-sigkill-safe`
(~13s — added P124, bumped from ~11s when the P124-review de-flake widened the SIGKILL control
budget: outer 4s→8s + a bounded 10s post-SIGKILL bind retry, so a loaded VM stops flaky
NOT-VERIFYing the P1 row), `docs-build/mkdocs-strict` (~4s), full-workspace clippy/fmt (~2s combined).
A one-line commit and a 500-file commit pay the same pre-push tax. The `.githooks/pre-push`
timing tripwire prints a stderr `WARN` (never blocks) once a run exceeds 90s; if pre-push
creeps meaningfully past ~120s, suspect a NEW whole-repo gate (another kcov-style full-corpus
walk), not diff growth — profile with `python3 quality/runners/run.py --cadence pre-push`
(per-row timing above) before adding budget.

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

**`minted_at` also arms the GRADE-TIME F-K4b congruence gate — not only the load-time
crash (P126 dc60cc21 CI regression).** Adding a write-once `minted_at` to a legacy
`kind: mechanical` row that emits `asserts_passed` (via a shell verifier) does two things
at once: it defuses the `validate_row` load crash (W1) AND it activates
`apply_pass_gates`'s F-K4b `asserts_congruent` check (`_audit_field.py`, gated on
`if row.get("minted_at")`). F-K4b runs ONLY on the PASS path (verifier exit 0 → PASS) and
requires **every** `expected.asserts` entry to map to an `asserts_passed` entry. So an
`expected.asserts` entry describing a **mutually-exclusive / environment branch the PASS
run never executes** — e.g. a `git < 2.34 → NOT-VERIFIED (exit 75)` skip — is structurally
UNCOVERABLE on the git≥2.34 PASS path and **silently demotes the real PASS to FAIL** the
instant `minted_at` lands (agent-ux/real-git-push-e2e FAILed CI run 29687639465 at 0.68s
this way). **Rule:** on a `minted_at` + `asserts_passed`-emitting mechanical row,
`expected.asserts` must be the **PASS-path claim set ONLY**; a skip/NOT-VERIFIED alternate
outcome belongs in the row's `comment`/`owner_hint`/`claim_vs_assertion_audit` (and is
enforced live by the verifier's own exit-75 branch), never in `expected.asserts`.
Regression-locked at **pre-push** by `test_audit_field.py::TestFK4bMutuallyExclusiveBranch`
(RED demote + GREEN row-shape + a live-catalog guard, run via
`structure/asserts-congruence-grade-time`). `kind: shell-subprocess` rows are exempt
(graded on the regenerated transcript, `apply_pass_gates` returns early).

**Coverage gap (why W6's local pre-push passed while CI caught it):**
`agent-ux/real-git-push-e2e` (P0) is cadence **`pre-pr`/`pre-release`/`on-demand` — NOT
`pre-push`** (it is cargo/sim-dependent, deliberately excluded from the local battery).
Before any phase-close push, run it locally with the SAME condition CI uses:
`python3 quality/runners/run.py --cadence pre-pr` (validate-only, no `--persist`; needs
git ≥ 2.34 + prebuilt bins) — or the single row directly via
`bash quality/gates/agent-ux/real-git-push-e2e.sh`. The pre-push battery alone will NOT
re-grade this P0. The structural F-K4b half of the hazard is now caught locally at
pre-push by the regression above, but the full round-trip stays pre-pr/CI-only.

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

### Hermetic test convention

**Any test that reaches a live network resource — directly, or transitively via a
subprocess it drives — must pass deterministically OFFLINE.** Mock, stub, or
short-circuit the probe before it fires; a test that depends on a real socket succeeding
is a flake generator, not a regression test. This was codified 2026-07-19 (Cycle-2 task
(d)) after `quality/runners/test_freshness_synth.py` — which exercises the runner
end-to-end by invoking `run.py --cadence weekly` as a subprocess — turned out to
transitively fan out into 20+ live HTTP calls (crates.io API, GitHub API) via unrelated
weekly-cadence `release-assets.json` rows, plus a deterministic `verifier not found at
None` leakage from two null-script `docs-reproducible.json` stub rows. A flaky/offline
network turned a P1 crates.io row FAIL and nondeterministically broke the test's exit-code
assertion (the "stale-P2 flake," PR#77 family) — invisible in code review because the test
itself contains no literal network call; the dependency was entirely transitive through
`--cadence weekly`'s catalog fan-out.

**The fix pattern (reusable for any test that subprocess-invokes `run.py`, not just this
one):** before the subprocess call, give every catalog row that could reach a live network
call (or any other non-hermetic side effect) OTHER than the row genuinely under test a
temporary near-future `waiver` — the runner's own WAIVED short-circuit in
`run_row` returns BEFORE the network-call branch and BEFORE the verifier-missing branch
(see `run.py::run_row`, `waiver_active` check order). Back up every catalog file touched
before mutating, restore all of them after (pass or fail) — see
`test_freshness_synth.py`'s `backup_catalogs` fixture + `_neutralize_other_weekly_rows`
for the reference implementation.

**Regression lock:** `structure/hermetic-test-network-isolation`
(`quality/gates/structure/hermetic-test-network-isolation.sh`,
`freshness-invariants.json`) re-runs `test_freshness_synth.py` under a poisoned
HTTP(S)/HTTPS proxy (`http_proxy=http://127.0.0.1:1 https_proxy=http://127.0.0.1:1
no_proxy=`) — any unmocked/un-neutralized `urllib.request` call routes through the
unreachable proxy and fails fast with `ECONNREFUSED` instead of reaching the real
network. Poisoned-proxy denial was chosen over `unshare -rn`/`firejail --net=none` for
the COMMITTED gate specifically because it needs no `CAP_SYS_ADMIN` / unprivileged user
namespace (some CI sandboxes restrict those); both mechanisms were manually verified to
deny the network and both pass the test deterministically — see the gate's header
comment for the manual `unshare -rn` cross-check.

### Backgrounded-process fd convention (structure / gate-authoring)

A gate that backgrounds a process (`… &` — a sim, a helper) MUST redirect its
stdout AND stderr on the SAME line as the `&` (`… >"${RUN_DIR}/sim.log" 2>&1 &`),
never a bare `… &`. `run.py` runs every gate under
`subprocess.run(capture_output=True)`; a backgrounded child that inherits that
capture pipe holds the write end open, so the runner's cleanup `communicate()`
blocks until the per-row timeout — and if a surviving grandchild keeps the pipe
open, run.py wedges INDEFINITELY (no timeout on the cleanup drain). That was the
2026-07-19 pre-pr CI hang: the job ran to its 28-min cap and was CANCELLED, and
cancelled-job logs are purged, so it was undiagnosable. Redirect to a per-run
log and `cat` that log on any startup-failure path so bind/seed errors stay
visible (they are no longer inherited to the gate's own stderr). Enforced by
convention + code review, not a gate; the reference fixes are the redirects
across `quality/gates/agent-ux/*` + `quality/gates/perf/latency-bench.sh`.

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

**Reading a walk BLOCK (DRAIN-17, P126 W2).** When a walk BLOCKs on per-row states it now
LEADS with a summary — `docs-alignment BLOCK: N row(s) blocking across M state(s):` then a
line per distinct blocking `RowState` naming its count + the exact row id(s)
(`MISSING_TEST x2 -- rows: [id-a, id-b]`), followed by the three-part fix / alternative /
copy-paste recovery teaching — so the FIRST line names WHICH row-STATE(s) to fix, not just
an alignment/coverage ratio. The per-row detail lines and any floor-ratio lines still
follow as context; a floor-ONLY block (no per-row state) prints just its ratio line,
unchanged. Regression: `quality/gates/docs-alignment/walk-block-summary.selftest.sh`
(exercises the real binary through `walk.sh`); state summary source is `walk()` /
`block_state_summary()` in `crates/reposix-quality/src/commands/doc_alignment.rs`.
