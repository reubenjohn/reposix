---
phase: 124-container-rehearse-harness-hardening
plan: 124
type: execute
wave: 0
depends_on: []
autonomous: true
requirements: [DRAIN-13, DRAIN-14, DRAIN-22, DRAIN-23, DRAIN-24]
files_modified:
  # Wave 0 — catalog-first (GREEN contract)
  - quality/catalogs/docs-reproducible.json
  - quality/catalogs/freshness-invariants.json
  # Wave 1 — SC1 (congruence earned + example-05 real runtime)
  - quality/gates/docs-repro/container-rehearse.sh
  - examples/01-shell-loop/run.sh
  - examples/02-python-agent/run.py
  - examples/04-conflict-resolve/run.sh
  - examples/05-blob-limit-recovery/run.sh
  - examples/05-blob-limit-recovery/expected-output.md
  - examples/05-blob-limit-recovery/RUN.md
  - quality/gates/docs-repro/container-congruence-earned.sh
  - quality/gates/docs-repro/container-congruence-earned.selftest.sh
  # Wave 2 — SC2 (SIGKILL-proof teardown + orphan detection)
  - quality/gates/docs-repro/container-rehearse-sigkill-safe.sh
  # Wave 3 — SC3 (binary provenance in post-release workflow)
  - .github/workflows/quality-post-release.yml
  - quality/gates/structure/container-rehearse-binary-provenance.sh
  # Wave 4 — SC4 (exit-from-artifact + .sim-*.log gitignore)
  - quality/gates/docs-repro/container-rehearse-exit-from-artifact.sh
  - .gitignore

must_haves:
  truths:
    - "Container-row congruence is EARNED: container-rehearse.sh harvests per-step `ASSERT-PASS:` lines from the container's own stdout; a no-op `exit 0` example script FAILS congruence (the F-K4b tautology is closed)."
    - "example-05 drives a REAL runtime `BLOB_LIMIT_EXCEEDED_FMT` stderr error under `REPOSIX_BLOB_LIMIT` and recovers via `git sparse-checkout set` — not the pre-emptive source-constant stand-in it exercises today."
    - "Sim teardown survives an outer SIGKILL (internal `timeout` < the row's `timeout_s`, and/or a killed process group); a stale sim already on port 7878 is detected fail-loud, never silently reused."
    - "The harness exit code is derived strictly from the persisted artifact `exit_code`, so a docker rc=0 can never mask an artifact `exit_code=1`."
    - "`target/debug/reposix`'s provenance on the post-release runner is guaranteed by an explicit build/artifact step (or documented inline if one already exists)."
    - "`.sim-*.log` files under quality/reports/verifications/docs-repro/ are git-ignored."
  artifacts:
    - path: quality/catalogs/docs-reproducible.json
      provides: "example-05 expected.asserts rewritten to the real-runtime-error contract; 3 NEW docs-repro rows (congruence-earned, sigkill-safe, exit-from-artifact) NOT-VERIFIED"
      contains: "container-congruence-earned"
    - path: quality/catalogs/freshness-invariants.json
      provides: "1 NEW structure row (container-rehearse-binary-provenance) NOT-VERIFIED"
      contains: "container-rehearse-binary-provenance"
    - path: quality/gates/docs-repro/container-rehearse.sh
      provides: "ASSERT-PASS harvesting, SIGKILL-proof teardown, port-free orphan gate, exit-from-artifact"
      min_lines: 150
    - path: examples/05-blob-limit-recovery/run.sh
      provides: "real runtime blob-limit-exceeded observe-then-recover cycle emitting ASSERT-PASS lines"
    - path: quality/gates/docs-repro/container-congruence-earned.sh
      provides: "SC1 fail-loud meta-check that a no-op exit-0 script fails congruence"
    - path: quality/gates/docs-repro/container-rehearse-sigkill-safe.sh
      provides: "SC2 selftest: SIGKILL harness leaves no orphan on 7878; stale sim fails loud"
    - path: quality/gates/structure/container-rehearse-binary-provenance.sh
      provides: "SC3 grep gate over quality-post-release.yml for an explicit reposix build/artifact step"
    - path: quality/gates/docs-repro/container-rehearse-exit-from-artifact.sh
      provides: "SC4 selftest: harness exit derives from persisted artifact exit_code; .sim-*.log ignored"
  key_links:
    - from: "examples/{01,02,04,05}/run.{sh,py}"
      to: "quality/gates/docs-repro/container-rehearse.sh"
      via: "machine-parseable `ASSERT-PASS: <text>` lines on container stdout, harvested (not copied from expected.asserts)"
      pattern: "ASSERT-PASS:"
    - from: "quality/gates/docs-repro/container-rehearse.sh"
      to: "the persisted artifact exit_code"
      via: "harness re-reads exit_code from the just-written artifact and exits with it"
      pattern: "exit_code"
    - from: ".github/workflows/quality-post-release.yml"
      to: "target/debug/reposix on the runner"
      via: "an explicit cargo build -p reposix-cli (or artifact-download) step before the post-release gates run"
      pattern: "cargo build -p reposix-cli"
---

<objective>
Harden `quality/gates/docs-repro/container-rehearse.sh` so its `kind: container` docs-repro rows are
**provenance-guaranteed** and immune to SIGKILL orphaning, exit-code disagreement, and tautological
assertion-congruence.

Purpose: today a container row's congruence is a TAUTOLOGY (a no-op `exit 0` script passes identically
to a real one — `container-rehearse.sh:96-107`), the ephemeral sim leaks on an outer SIGKILL (an
un-firable `trap ... EXIT` at `:126`), a stale orphan on 7878 is silently reused (`:135-141` curls
whatever answers), the harness masks an artifact `exit_code=1` behind docker rc, and `target/debug/reposix`
reaches the post-release runner via an UNCONFIRMED implicit cache (`quality-post-release.yml` has no
build step). This phase closes all four.

Output: EARNED per-step congruence; a real runtime blob-limit exercise in example-05; SIGKILL-proof
teardown + fail-loud orphan detection; artifact-derived exit; a confirmed binary-provenance step; a
`.sim-*.log` gitignore pattern. Four new machine-checkable catalog gates grade it.
</objective>

<phase_context>
## Success criteria (VERBATIM from ROADMAP.md P124; the phase-close gate — waves map 1:1)

- **SC1 (DRAIN-22):** "Container-row congruence is EARNED via per-step `ASSERT-PASS:` harvesting, not
  emitted verbatim from `expected.asserts`; example-05 exercises a real runtime blob-limit-exceeded error
  + `git sparse-checkout` recovery cycle."
- **SC2 (DRAIN-23):** "Sim teardown survives an outer SIGKILL (an internal `timeout` shorter than the
  row's `timeout_s`, and/or a killed process group); a stale orphaned sim on port 7878 is detected
  fail-loud, not silently reused."
- **SC3 (DRAIN-24):** "`target/debug/reposix`'s provenance on the `quality-post-release.yml` runner is
  confirmed via an explicit build/artifact-download step, or documented inline if one already exists."
- **SC4 (DRAIN-13 + DRAIN-14):** "The harness exit code is derived strictly from the persisted
  `exit_code` (not a possibly-disagreeing rc=0), and a `.sim-*.log` `.gitignore` pattern is added under
  `quality/reports/verifications/docs-repro/`."

## Verbatim requirement definitions (from .planning/REQUIREMENTS.md — the precise contract)

- **DRAIN-13** *(MED, GTH-V15-10)*: "`container-rehearse.sh`'s harness return code and its persisted
  artifact `exit_code` can disagree (rc=0 masking artifact exit_code=1); a sim-readiness race between
  rapid sequential runs produces transient 'sim not reachable' flakes — derive the harness exit strictly
  from the persisted `exit_code`; add a pre-`docker run` port-7878-free + sim-reachability readiness gate."
- **DRAIN-14** *(LOW, GTH-V15-11)*: "Add a `.sim-*.log` pattern to `.gitignore` scoped to
  `quality/reports/verifications/docs-repro/`."
- **DRAIN-22** *(MED)*: "F-K4b container-class congruence tautology. Because `container-rehearse.sh` now
  emits each `kind: container` row's `expected.asserts` verbatim as `asserts_passed` on container exit 0,
  the per-expected-assert congruence gate (`_audit_field.py::asserts_congruent`) is a TAUTOLOGY — a no-op
  `exit 0` script would pass identically to a real one. Make container-row congruence EARNED, not emitted:
  prefer per-step-earned emission mirroring `tutorial-replay.sh` (each example script prints a
  machine-parseable `ASSERT-PASS: <text>` line only after the step that establishes that specific assert
  actually succeeds; harvest those instead of copying `expected.asserts`). Also give example-05 a real
  runtime blob-limit-exceeded exercise (drive the actual `BLOB_LIMIT_EXCEEDED_FMT` error +
  `git sparse-checkout` recovery cycle) rather than only the pre-emptive sparse-checkout + source-constant
  stand-in it exercises today."
- **DRAIN-23** *(MED)*: "SIGKILL sim-leak / EXIT-trap orphan in `container-rehearse.sh` — the harness
  backgrounds the ephemeral sim and tears it down via a bash `EXIT` trap, which never fires when the
  runner's `subprocess.run(timeout=...)` SIGKILLs the harness (reproduced in the b773c04 CI failure,
  orphaned pid on host port 7878). Make sim teardown SIGKILL-proof: wrap `docker run` in an internal
  `timeout` strictly shorter than the row's catalog `timeout_s` so the harness reaps its own children
  before the outer SIGKILL fires, and/or start the sim in its own process group (`setsid`/`set -m`) and
  kill the group on teardown; add a pre-`docker run` port-7878-free readiness check so a stale orphaned
  sim is detected fail-loud instead of silently reused."
- **DRAIN-24** *(MED, verify)*: "Confirm `target/debug/reposix`'s provenance on the
  `quality-post-release.yml` runner that `container-rehearse.sh` needs host-mounted — trace whether it
  reaches the runner via an explicit `cargo build -p reposix-cli` step, an artifact download, or an
  unconfirmed implicit cache hit. If implicit, add an explicit build or artifact-download step as a hard
  dependency of the container-rehearse job so `kind: container` docs-repro rows are provenance-guaranteed
  on a cold runner rather than silently degrading to NOT-VERIFIED; if an explicit step already exists,
  document it inline so the question isn't re-opened."

## Charter's DRAIN labels were scrambled — content-correct mapping used above (see NOTICED)

The dispatch charter parenthetically labelled SC1=DRAIN-13, SC2=DRAIN-14, SC3=DRAIN-22,
SC4=DRAIN-23+24. That is scrambled vs REQUIREMENTS.md. By CONTENT the true mapping is SC1=DRAIN-22,
SC2=DRAIN-23, SC3=DRAIN-24, SC4=DRAIN-13+14 (used throughout this plan). All five DRAIN IDs are covered
regardless; the ROADMAP `Requirements:` line lists all five and does not itself parenthesize the SCs.

## Ground-truth investigation findings (verified against reality, HEAD bc4decf3)

- Harness: `quality/gates/docs-repro/container-rehearse.sh` (199 lines). Congruence tautology at
  `:96-107` + `:187-193` (copies `expected.asserts` verbatim on exit 0). Un-firable teardown:
  `trap '... kill "$SIM_PID" ...' EXIT` at `:126`. Silent-orphan-reuse readiness at `:135-141` (curls
  whatever answers on 7878 — a stale sim passes). Exit == docker rc at `:164/:199`.
- Container example rows (all `post-release` cadence, 3 asserts each): `example-01-shell-loop`
  (`bash examples/01-shell-loop/run.sh`), `example-02-python-agent`
  (`python3 examples/02-python-agent/run.py`), `example-04-conflict-resolve`
  (`bash examples/04-conflict-resolve/run.sh`), `example-05-blob-limit-recovery`
  (`bash examples/05-blob-limit-recovery/run.sh`). example-03 is `manual` (no command — unaffected).
- example-05 today: `run.sh` step [1/4] GREPS the source constant, steps [3/4]-[4/4] do PRE-EMPTIVE
  `git sparse-checkout` — it provably NEVER reads a runtime blob-limit stderr (its own header + the row
  `claim_vs_assertion_audit` + `examples/05-blob-limit-recovery/expected-output.md` say so).
- Runtime error source: `crates/reposix-remote/src/stateless_connect.rs:59-60`
  `BLOB_LIMIT_EXCEEDED_FMT` — written to STDERR in `proxy_one_rpc` on a `command=fetch` that would
  materialize > `REPOSIX_BLOB_LIMIT` blobs (STDERR-only; NEVER on the protocol-v2 stdout). The
  fast-import fetch path bypasses the per-RPC check; the modern-git (2.34+) stateless-connect
  protocol-v2 fetch path is where it fires. ubuntu:24.04 ships git 2.43 → the real trigger IS reachable
  in-container.
- Congruence gate: `quality/runners/_audit_field.py::asserts_congruent` (`:188`) — every
  `expected.assert` must token-map (>=2 shared significant tokens, or >=1 for terse) to some
  `asserts_passed` entry. Switching the harness to harvest `ASSERT-PASS:` lines means EVERY container
  example must emit one such line per expected.assert or its row's congruence breaks.
- `tutorial-replay.sh` (the DRAIN-22-cited exemplar) uses an in-process `PASSED[]` array because it runs
  ON THE HOST; the container case needs the stdout-line protocol because the example runs in an isolated
  container the harness can only observe via captured stdout (see NOTICED).
- `.github/workflows/quality-post-release.yml`: runs `python3 quality/runners/run.py --cadence
  post-release` (the cadence that grades every container row) but has **NO** `cargo build -p reposix-cli`
  step and no artifact download → `target/debug/reposix` provenance is the unconfirmed-implicit-cache case
  DRAIN-24 flags. Confirmed real gap.
- `.gitignore`: no `.sim-*.log` pattern (SC4/DRAIN-14 gap confirmed).
</phase_context>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@quality/PROTOCOL.md
@quality/catalogs/README.md

Harness under change (read fully before editing):
@quality/gates/docs-repro/container-rehearse.sh

Per-step-earned-emission exemplar (DRAIN-22-cited; note the host `PASSED[]` array vs the container
stdout-line protocol this plan introduces):
@quality/gates/docs-repro/tutorial-replay.sh

Congruence gate the harvested lines must satisfy:
@quality/runners/_audit_field.py

example-05 rework surface (source of the honesty caveats to reverse):
@examples/05-blob-limit-recovery/run.sh
@examples/05-blob-limit-recovery/expected-output.md

Runtime error the rework must drive:
@crates/reposix-remote/src/stateless_connect.rs  (BLOB_LIMIT_EXCEEDED_FMT ~L59-60; proxy_one_rpc)

Provenance gap:
@.github/workflows/quality-post-release.yml

House-format catalog-first exemplar (mirror this shape for the new NOT-VERIFIED rows):
@.planning/phases/123-quality-runner-catalog-integrity-hardening/123-01-PLAN.md
</context>

<interfaces>
Unified catalog row schema (quality/catalogs/README.md) — every NEW row needs: `id`, `dimension`,
`kind`, `comment`, `sources`, `command` (null for a mechanical gate), `expected.asserts`,
`verifier.{script,args,timeout_s,container}`, `artifact`, `status`, `last_verified` (null until first
grade), `minted_at` (RFC3339, WRITE-ONCE — REQUIRED for a row landing now), `freshness_ttl` (null for
mechanical rows), `blast_radius`, `owner_hint`, `waiver` (null), `claim_vs_assertion_audit` (>=50
chars, REQUIRED — every row minted here post-dates the audit cutoff), `cadences`.
`_audit_field.validate_row` runs on EVERY `load_catalog()` (`run.py:121`) and raises `SystemExit`
— breaking EVERY row in the file — if `claim_vs_assertion_audit` or `minted_at` is missing.

The `ASSERT-PASS:` line protocol this plan introduces (harness ⇄ example contract):
```
# In each container example, AFTER the load-bearing step succeeds under `set -euo pipefail`:
echo "ASSERT-PASS: <text that token-maps to the matching expected.asserts entry>"
# container-rehearse.sh greps captured container STDOUT for '^ASSERT-PASS: ' and puts the
# captured text into asserts_passed[] — it NO LONGER copies row.expected.asserts verbatim.
```
`_audit_field.asserts_congruent(expected, passed)` then requires each expected.assert to share >=2
significant tokens (>=1 if the expected assert has <=2 significant tokens) with some harvested line.
</interfaces>

<execution_constraints>
## Binding constraints (embedded verbatim — apply to EVERY task in EVERY wave)

- **ONE cargo invocation machine-wide** (hook-enforced `cargo-mutex.sh`, exit 2). Cargo is
  **FOREGROUND-only**, NEVER `run_in_background`/detached (orphans the build + OOM risk — this VM
  OOM-crashed 3×). Prefer `cargo build -p reposix-cli` (jobs=2), never `--workspace` unless required.
  This phase needs `target/debug/reposix` built ONCE for local container/sim rehearsal — build it in a
  single foreground invocation and reuse.
- **One tree-writer at a time.** Waves are SERIAL (0→1→2→3→4); a wave's push/commit lands before the
  next opens. No two lanes edit the shared tree concurrently.
- **Leaf test isolation** (hook-enforced, exit 2): any `reposix init` / sim-seed / `git config` /
  `git commit` done for TEST SETUP runs in a throwaway `/tmp` clone with the `cd /tmp/...` in the SAME
  Bash invocation — NEVER the shared repo. The container rehearsal already runs the example in a
  `/tmp`-scoped `$WORK`; keep every selftest fixture under `/tmp`.
- **Sim is the default backend. Do NOT touch real backends (GitHub/Confluence/JIRA) in P124.** The
  runtime blob-limit exercise runs against the ephemeral SIM on 127.0.0.1:7878.
- **Uncommitted = didn't happen.** Commit before ending any turn; mid-phase commits stay LOCAL until
  the phase-close push. **No `--no-verify`.** Targeted staging only (never `git add -A`/`.`). No tag push.
- **Enter through GSD.** Never hand-edit outside this phase. Never silently downgrade a gate.
- **Never hand-pick gates before a push** — run `python3 quality/runners/run.py --cadence pre-push`
  (FULL). A docs edit must pass BOTH `docs-build/*` AND `structure/banned-words`.
- **Docker dependency:** the container waves need docker. Where docker is absent the harness writes
  NOT-VERIFIED (never a false green); the SC1/SC2/SC4 selftests must degrade to NOT-VERIFIED (not FAIL)
  when docker/kcov/sim substrate is unavailable, per OP-2 (skip is never pass).

## CHARTER BLOCK — carry verbatim into every executor lane

1. You own what you touch. Acceptance criteria are the floor, not the ceiling.
2. Noticing is a deliverable — report lying doc claims, tests that don't assert what their names
   promise, teaching-free errors, dead code, missing edge cases. An empty noticing section from
   code-touching work is a red flag.
3. Eager-fix or file — never silently skip. <1h + no new dependency → fix in place; else →
   SURPRISES-INTAKE / GOOD-TO-HAVES with severity + sketch.
4. Verify against reality — run it, render it, hit the backend. A claim without an artifact is not done.
5. North star: polish for adoption — would a skeptical first-time dev come away impressed?
</execution_constraints>

<tasks>

<!-- ==================================================================== -->
<!-- WAVE 0 — CATALOG-FIRST: author the GREEN-contract rows the verifier   -->
<!-- will grade. FIRST commit of the phase. No implementation code.        -->
<!-- ==================================================================== -->

<task type="auto" wave="0">
  <name>Wave 0 · Task 1: Author 4 catalog rows + rewrite example-05's contract (catalog-first)</name>
  <files>quality/catalogs/docs-reproducible.json, quality/catalogs/freshness-invariants.json</files>
  <action>
Author the phase's GREEN contract in the SAME first commit (quality/PROTOCOL.md Step 3, catalog-first).
Hand-author directly in JSON (accepted for non-doc-alignment dimensions). `minted_at` = current UTC
RFC3339 on every NEW row. Each `claim_vs_assertion_audit` >=50 chars, stating how the verifier's
asserts falsify the claim if false. Each new row `status: "NOT-VERIFIED"`, `last_verified: null`,
`waiver: null`. `sources` cite this plan file + the DRAIN id each row closes.

**In `quality/catalogs/docs-reproducible.json`:**

(a) REWRITE the existing `docs-repro/example-05-blob-limit-recovery` row's `expected.asserts` to the
new real-runtime-error contract (do NOT delete `comment` history — APPEND a P124 note that the
2026-07-13 honesty reword is now SUPERSEDED because example-05 drives the real error). New asserts
(replace all 3):
  - "bash examples/05-blob-limit-recovery/run.sh exits 0 in an ubuntu:24.04 container after driving a
    REAL runtime blob-limit-exceeded refusal under REPOSIX_BLOB_LIMIT and recovering"
  - "the helper emits the BLOB_LIMIT_EXCEEDED_FMT stderr refusal (naming `git sparse-checkout set`)
    when a blob-materializing fetch/checkout of refs/reposix/origin/main would exceed REPOSIX_BLOB_LIMIT"
  - "after `git sparse-checkout set` narrows the scope, the retried checkout materializes the narrowed
    issues/*.md record set and completes 0 — the observe-error-then-recover cycle"
  Flip `status` to `"NOT-VERIFIED"` (a real re-grade re-mints it; do not leave a stale PASS describing
  the old pre-emptive-only behavior). Keep `cadences: ["post-release"]`, `blast_radius: "P1"`.

(b) ADD `docs-repro/container-congruence-earned` (SC1 fail-loud meta-check, kind `mechanical`):
  - comment: closes the F-K4b container tautology (DRAIN-22) — container-rehearse.sh must HARVEST
    per-step `ASSERT-PASS:` lines from container stdout, never copy `expected.asserts` verbatim; a no-op
    `exit 0` script must FAIL congruence.
  - expected.asserts: ["container-rehearse.sh contains no code path that copies row.expected.asserts
    into asserts_passed (grep-proven absence of the verbatim-copy line)", "a fixture example that exits
    0 but prints ZERO `ASSERT-PASS:` lines produces an artifact whose asserts_passed does NOT cover the
    row's expected.asserts, so asserts_congruent() returns False", "a fixture example that prints one
    `ASSERT-PASS:` line per expected.assert produces a congruent artifact"]
  - verifier.script `"quality/gates/docs-repro/container-congruence-earned.sh"`, args [], timeout_s 60,
    container null. artifact `"quality/reports/verifications/docs-repro/container-congruence-earned.json"`.
    blast_radius `"P0"` (a false green rides on this). cadences `["pre-push","pre-pr"]`.

(c) ADD `docs-repro/container-rehearse-sigkill-safe` (SC2, kind `mechanical`):
  - comment: closes DRAIN-23 (b773c04 orphan) — teardown survives an outer SIGKILL and a stale sim on
    7878 is fail-loud, never reused.
  - expected.asserts: ["container-rehearse.sh wraps its `docker run` in an internal `timeout` strictly
    shorter than the row's catalog timeout_s so it reaps children before an outer SIGKILL", "the
    ephemeral sim runs in its own process group and teardown kills the GROUP (setsid/set -m), not just
    the leader pid", "a pre-`docker run` check FAILS LOUD (NOT-VERIFIED/error, not silent reuse) when
    port 7878 is already occupied by a stale sim", "a selftest that SIGKILLs the harness mid-run leaves
    no listener on 127.0.0.1:7878 afterward"]
  - verifier.script `"quality/gates/docs-repro/container-rehearse-sigkill-safe.sh"`, args [],
    timeout_s 120, container null. artifact
    `"quality/reports/verifications/docs-repro/container-rehearse-sigkill-safe.json"`. blast_radius
    `"P1"`. cadences `["pre-push","pre-pr"]`.

(d) ADD `docs-repro/container-rehearse-exit-from-artifact` (SC4/DRAIN-13, kind `mechanical`):
  - comment: closes DRAIN-13 — the harness exit is derived STRICTLY from the persisted artifact
    exit_code so a docker rc=0 cannot mask an artifact exit_code=1.
  - expected.asserts: ["container-rehearse.sh writes the artifact FIRST, then re-reads exit_code from it
    and exits with THAT value", "a forced artifact exit_code=1 while docker rc would be 0 makes the
    harness exit 1 (selftest-proven)", "the pre-`docker run` readiness gate also requires sim
    reachability, not merely a bound port"]
  - verifier.script `"quality/gates/docs-repro/container-rehearse-exit-from-artifact.sh"`, args [],
    timeout_s 90, container null. artifact
    `"quality/reports/verifications/docs-repro/container-rehearse-exit-from-artifact.json"`. blast_radius
    `"P1"`. cadences `["pre-push","pre-pr"]`.

**In `quality/catalogs/freshness-invariants.json`** (dimension structure):

(e) ADD `structure/container-rehearse-binary-provenance` (SC3/DRAIN-24, kind `mechanical`):
  - comment: closes DRAIN-24 — `target/debug/reposix` must reach the post-release runner via an EXPLICIT
    build/artifact step, not an unconfirmed implicit cache.
  - expected.asserts: [".github/workflows/quality-post-release.yml contains an explicit
    `cargo build -p reposix-cli` (or artifact-download) step that runs BEFORE `run.py --cadence
    post-release`", "the step is a hard dependency of the job that grades container rows — no container
    docs-repro row can silently degrade to NOT-VERIFIED for a missing host binary on a cold runner",
    "the provenance decision is documented inline in the workflow (a comment naming why the step exists)"]
  - verifier.script `"quality/gates/structure/container-rehearse-binary-provenance.sh"`, args [],
    timeout_s 30, container null. artifact
    `"quality/reports/verifications/structure/container-rehearse-binary-provenance.json"`. blast_radius
    `"P1"`. cadences `["pre-push","pre-pr"]`.

The 4 new NOT-VERIFIED rows name verifier scripts that don't exist yet — that is CORRECT catalog-first
and SAFE: `structure/verifier-script-exists` exempts NOT-VERIFIED rows (P123 ruling), and a
NOT-VERIFIED row at pre-push does not block (proven in P123-01). Confirm this in the verify step.
  </action>
  <verify>
    <automated>python3 -c "import sys; sys.path.insert(0,'quality/runners'); import run; [run.load_catalog(p) for p in run.discover_catalogs()]; print('LOADS_OK')" && python3 quality/runners/run.py --cadence pre-push >/dev/null 2>&1; echo "pre-push rc=$?"</automated>
  </verify>
  <done>example-05's expected.asserts describe the real-runtime-error contract; 4 new NOT-VERIFIED rows
  (3 docs-repro + 1 structure) exist, each with minted_at + a >=50-char claim_vs_assertion_audit; every
  catalog loads without SystemExit; `run.py --cadence pre-push` still exits 0 (the NOT-VERIFIED rows with
  not-yet-written verifiers do not self-block). COMMIT this as the phase's first commit.</done>
</task>

<!-- ==================================================================== -->
<!-- WAVE 1 — SC1 (DRAIN-22): earned congruence + example-05 real runtime  -->
<!-- HIGHEST-RISK WAVE. See return note: recommend splitting 1a/1b.        -->
<!-- ==================================================================== -->

<task type="auto" wave="1" tdd="true">
  <name>Wave 1a · Task 1: Harvest ASSERT-PASS lines (harness + examples 01/02/04)</name>
  <files>quality/gates/docs-repro/container-rehearse.sh, examples/01-shell-loop/run.sh, examples/02-python-agent/run.py, examples/04-conflict-resolve/run.sh, quality/gates/docs-repro/container-congruence-earned.sh, quality/gates/docs-repro/container-congruence-earned.selftest.sh</files>
  <behavior>
    - A container example that prints one `ASSERT-PASS: <text>` line per expected.assert (after the
      load-bearing step, under `set -euo pipefail`) → harness harvests them → asserts_congruent() True.
    - A no-op `sh -c "exit 0"` example (zero ASSERT-PASS lines) → asserts_passed does NOT cover
      expected.asserts → asserts_congruent() False (the tautology is closed).
    - The harness NO LONGER contains the verbatim `expected.asserts → asserts_passed` copy path.
  </behavior>
  <action>
Rewrite the harvesting half of `container-rehearse.sh`:
  1. DELETE the tautology block (`:96-107` EXPECTED_ASSERTS resolution + `:187-193` verbatim copy). Keep
     the generic `"container <c> ran command and exited 0"` line ONLY as diagnostic context, NOT as a
     congruence source.
  2. After capturing container stdout to `$STDOUT_TMP`, grep `^ASSERT-PASS: ` lines and put the
     trailing text into `asserts_passed[]`. Harvest from stdout only (the `[RPX-xxxx]` stderr-only
     constraint means teaching strings never collide). Tempfile-then-grep (P56 SIGPIPE lesson — never
     pipe-into-head).
  3. Update the artifact-writing python block to emit the harvested list.
Then add `ASSERT-PASS:` emission to the three examples so each row's 3 expected.asserts each token-map
to a harvested line (>=2 shared significant tokens per `_audit_field._sig_tokens`):
  - `examples/01-shell-loop/run.sh` (bash): `echo "ASSERT-PASS: <text>"` after each of the 3 steps.
  - `examples/02-python-agent/run.py` (PYTHON — not bash): `print("ASSERT-PASS: <text>", flush=True)`
    after each assertion succeeds.
  - `examples/04-conflict-resolve/run.sh` (bash): same, after the fetch-first-then-recover step lands.
Word each `ASSERT-PASS:` text to token-overlap its matching `expected.asserts` entry (open the catalog
row, mirror its salient nouns). Do NOT weaken any example's real assertions — the ASSERT-PASS line rides
AFTER the real check, it does not replace it.
Author the SC1 meta-check `container-congruence-earned.sh` + `.selftest.sh`: the selftest builds two
throwaway `/tmp` fixture "examples" (one no-op `exit 0`, one printing N ASSERT-PASS lines), runs the
harness's harvest logic against a stub catalog row, and asserts congruent==False for the no-op and
True for the real one; the gate ALSO greps container-rehearse.sh proving the verbatim-copy path is
GONE. Degrade to NOT-VERIFIED (not FAIL) if docker is absent — the grep-absence leg still runs
docker-free, so the gate can PASS its static leg and NOT-VERIFIED only the dynamic leg.
  </action>
  <verify>
    <automated>bash quality/gates/docs-repro/container-congruence-earned.selftest.sh && bash quality/gates/docs-repro/container-congruence-earned.sh; echo "rc=$?"</automated>
  </verify>
  <done>container-rehearse.sh harvests `ASSERT-PASS:` lines and contains NO verbatim expected.asserts
  copy; examples 01/02/04 each emit one ASSERT-PASS line per expected.assert; the selftest proves a
  no-op script FAILS congruence and a real one passes. If docker is present locally, also run
  `quality/gates/docs-repro/container-rehearse.sh docs-repro/example-01-shell-loop` and confirm the
  artifact's asserts_passed are HARVESTED (not the row's verbatim asserts).</done>
</task>

<task type="auto" wave="1" tdd="true">
  <name>Wave 1b · Task 2: example-05 drives the REAL runtime blob-limit error + recovery</name>
  <files>examples/05-blob-limit-recovery/run.sh, examples/05-blob-limit-recovery/expected-output.md, examples/05-blob-limit-recovery/RUN.md</files>
  <behavior>
    - Under `REPOSIX_BLOB_LIMIT=<small>`, a blob-materializing `git checkout`/`git fetch` of
      refs/reposix/origin/main (whose tree references MORE than the limit of unmaterialized blobs)
      triggers the helper's real `BLOB_LIMIT_EXCEEDED_FMT` stderr refusal naming `git sparse-checkout set`.
    - The script CAPTURES that stderr (proves it fired — grep for the literal `git sparse-checkout`
      token / `[RPX-0503]`), then runs `git sparse-checkout set issues/1.md issues/2.md ...` to narrow
      under the limit, retries the checkout, and it completes 0.
    - The script exits 0 and prints one `ASSERT-PASS:` line per (rewritten) expected.assert.
  </behavior>
  <action>
Rework `examples/05-blob-limit-recovery/run.sh` to exercise the REAL cycle (reverse the 2026-07-13
"provably never reads helper stderr" caveat — it now DOES):
  1. `reposix init sim::demo "$WORK"` with `REPOSIX_BLOB_LIMIT` set SMALL (the demo seed has 6 issues;
     a limit of 3 refuses a full-tree materialization). Keep `set -euo pipefail`.
  2. Attempt a blob-materializing checkout WITHOUT pre-narrowing (e.g. `git -C "$WORK" checkout -B main
     refs/reposix/origin/main` after a `--filter=blob:none` fetch, so the checkout lazy-fetches all 6
     blobs through the helper's `command=fetch` → exceeds the limit). Capture combined stderr to a file.
     This is the RUNTIME trigger the fast-import path bypassed — it MUST go through the modern-git
     (2.34+) stateless-connect protocol-v2 fetch. Assert (fail-loud under set -e) the captured stderr
     contains the `git sparse-checkout` token AND `[RPX-0503]` → emit its ASSERT-PASS line.
  3. Recover: `git -C "$WORK" sparse-checkout init --no-cone && sparse-checkout set issues/1.md
     issues/2.md issues/3.md`, retry the checkout, assert it exits 0 and `ls "$WORK"/issues/*.md`
     matches the narrowed set → emit its ASSERT-PASS line.
  4. Emit the final exits-0 ASSERT-PASS line.
Rewrite `expected-output.md` + `RUN.md` to describe the REAL observe-then-recover flow and DELETE the
"does NOT observe a runtime blob-limit stderr error" caveat (it is now false). Keep the audit-inspection
footer.
VERIFY AGAINST REALITY (mandatory): build `target/debug/reposix` ONCE (`cargo build -p reposix-cli`,
foreground), start the ephemeral sim, and run the example BOTH bare-on-host and (if docker present) via
`quality/gates/docs-repro/container-rehearse.sh docs-repro/example-05-blob-limit-recovery`. Confirm the
real `BLOB_LIMIT_EXCEEDED_FMT` line actually appears in captured stderr — do NOT declare done on an
assumed trigger. If the runtime error does NOT fire through the expected path (e.g. checkout batches
the lazy fetch differently than assumed), STOP and FILE the exact observed behavior to
SURPRISES-INTAKE with the captured transcript rather than faking the assertion — the whole point of
DRAIN-22 is that the error is EARNED.
  </action>
  <verify>
    <automated>cargo build -p reposix-cli 2>&1 | tail -3 && ( ./target/debug/reposix sim --bind 127.0.0.1:7878 --ephemeral >/tmp/e05-sim.log 2>&1 & SP=$!; sleep 2; PATH="$PWD/target/debug:$PATH" bash examples/05-blob-limit-recovery/run.sh; RC=$?; kill $SP 2>/dev/null; echo "example05 rc=$RC" )</automated>
  </verify>
  <done>example-05 run.sh drives a REAL BLOB_LIMIT_EXCEEDED_FMT stderr refusal (captured + asserted),
  recovers via `git sparse-checkout set`, retries clean, exits 0, and prints one ASSERT-PASS line per
  rewritten expected.assert; expected-output.md/RUN.md describe the real cycle with the stale
  "never observes the error" caveat removed; the on-host run exits 0 with the real error proven in the
  captured transcript.</done>
</task>

<!-- ==================================================================== -->
<!-- WAVE 2 — SC2 (DRAIN-23): SIGKILL-proof teardown + fail-loud orphan     -->
<!-- ==================================================================== -->

<task type="auto" wave="2">
  <name>Wave 2 · Task 1: SIGKILL-proof teardown + pre-run port-7878-free fail-loud gate</name>
  <files>quality/gates/docs-repro/container-rehearse.sh, quality/gates/docs-repro/container-rehearse-sigkill-safe.sh</files>
  <action>
Harden `container-rehearse.sh` teardown (DRAIN-23):
  1. Start the ephemeral sim in its OWN process group (`setsid` or `set -m` + a subshell) so teardown
     can `kill -TERM -- -"$SIM_PGID"` the whole group, not just the leader pid — a child left by the sim
     is reaped too.
  2. Wrap `docker run` in an internal `timeout <T> docker run ...` where `<T>` is STRICTLY shorter than
     the row's catalog `timeout_s` (read `timeout_s` from the row JSON already parsed; subtract a margin,
     e.g. `T = timeout_s - 30`, floor 30). This guarantees the harness regains control and reaps its
     children BEFORE the runner's outer `subprocess.run(timeout=...)` SIGKILLs it.
  3. Keep the existing `trap ... EXIT` for the graceful path, but the process-group kill + internal
     timeout are the SIGKILL-proof backstop (the EXIT trap alone never fires on SIGKILL — that is the
     b773c04 bug).
  4. Add a PRE-`docker run` gate: if `127.0.0.1:7878` is ALREADY bound before this harness starts its own
     sim, FAIL LOUD — write a NOT-VERIFIED artifact with a teaching error ("stale sim/orphan on 7878;
     free it with `kill $(lsof -ti:7878)` and re-run") and exit non-zero. Do NOT curl-and-reuse whatever
     answers (the current `:135-141` silently reuses a stale orphan — that is the exact DRAIN-23 miss).
Author `container-rehearse-sigkill-safe.sh` (SC2 selftest): in a throwaway `/tmp` scope, start the
harness against a fixture row, SIGKILL it mid-run (or SIGKILL the process the harness backgrounded via
the runner's timeout path), then assert `lsof -ti:7878` (or `ss`/`curl`) shows NO surviving listener;
AND assert a second harness invocation with a deliberately-pre-bound 7878 fails LOUD (non-zero +
teaching error), not silent reuse. Degrade to NOT-VERIFIED where docker/lsof substrate is absent.
  </action>
  <verify>
    <automated>bash quality/gates/docs-repro/container-rehearse-sigkill-safe.sh; echo "rc=$?"</automated>
  </verify>
  <done>container-rehearse.sh starts the sim in its own process group, kills the GROUP on teardown,
  wraps `docker run` in an internal `timeout` < the row's timeout_s, and FAILS LOUD on a pre-bound 7878;
  the selftest proves a SIGKILLed harness leaves no listener on 7878 and a stale-sim precheck is
  fail-loud. No orphan `reposix sim` process survives the selftest (`ps aux | grep 'reposix sim'`
  clean afterward).</done>
</task>

<!-- ==================================================================== -->
<!-- WAVE 3 — SC3 (DRAIN-24): confirm/guarantee binary provenance on runner -->
<!-- ==================================================================== -->

<task type="auto" wave="3">
  <name>Wave 3 · Task 1: Explicit reposix-build step in quality-post-release.yml + provenance gate</name>
  <files>.github/workflows/quality-post-release.yml, quality/gates/structure/container-rehearse-binary-provenance.sh</files>
  <action>
DRAIN-24 is confirmed: `quality-post-release.yml` runs `run.py --cadence post-release` (the cadence that
grades every `kind: container` docs-repro row, which host-mounts `target/debug/reposix`) but has NO
build/artifact step — the binary's presence is an unconfirmed implicit cache. Close it:
  1. Add an explicit step BEFORE "Run post-release cadence gates":
     `- name: Build reposix (host binary the container rows mount)` running `cargo build -p reposix-cli`
     (the toolchain is already set up via `dtolnay/rust-toolchain@stable`). Add an INLINE comment naming
     WHY (container docs-repro rows mount `target/debug/reposix`; without this they degrade to
     NOT-VERIFIED on a cold runner — DRAIN-24). Respect the ONE-cargo rule conceptually (CI runner is a
     separate machine; this is a single invocation there).
  2. Author `quality/gates/structure/container-rehearse-binary-provenance.sh`: parse the workflow YAML
     (python3, not brittle grep-only) and assert an explicit `cargo build -p reposix-cli` (or an
     artifact-download) step exists AND precedes the `run.py --cadence post-release` step, AND that an
     inline provenance comment is present. Print a teaching failure (name the file + the missing step +
     the fix) if absent.
Do NOT re-add `build-essential`/compiler toolchain to the CONTAINER (fix-it-twice ruling b773c04: the
examples run the pre-built host-mounted binary; there is no in-container cargo build). This step builds
on the RUNNER host, not in the container.
  </action>
  <verify>
    <automated>bash quality/gates/structure/container-rehearse-binary-provenance.sh; echo "rc=$?"</automated>
  </verify>
  <done>quality-post-release.yml has an explicit `cargo build -p reposix-cli` step (with an inline
  provenance comment) before the post-release gates run; the provenance gate PASSES and would FAIL if
  the step were removed. `bash quality/gates/docs-build/*.sh` and `structure/banned-words.sh` still pass
  for the workflow edit (it is under `.github/`, but run the full pre-push to be safe).</done>
</task>

<!-- ==================================================================== -->
<!-- WAVE 4 — SC4 (DRAIN-13 + DRAIN-14): exit-from-artifact + gitignore     -->
<!-- Depends on Wave 2's readiness gate + Wave 1's harvested artifact.      -->
<!-- ==================================================================== -->

<task type="auto" wave="4">
  <name>Wave 4 · Task 1: Derive harness exit from persisted artifact exit_code + .sim-*.log gitignore</name>
  <files>quality/gates/docs-repro/container-rehearse.sh, .gitignore, quality/gates/docs-repro/container-rehearse-exit-from-artifact.sh</files>
  <action>
DRAIN-13 exit-derivation: today the harness `exit "$EXIT_CODE"` where `$EXIT_CODE` == docker rc
(`:164/:199`). After Wave 1, a docker rc=0 can co-exist with an artifact `exit_code=1` (e.g. a missing
required `ASSERT-PASS:` line → congruence fail recorded in the artifact). Change the harness so it:
  1. Determines the artifact's `exit_code` as the AUTHORITATIVE value: rc=0 AND all expected asserts
     harvested → 0; otherwise 1 (a clean docker exit with missing/uncongruent asserts is a FAIL, not a
     pass). Write that into the artifact.
  2. Re-READ `exit_code` from the just-written artifact (python one-liner) and `exit` with THAT — the
     harness rc now provably equals the persisted exit_code (close the rc-masks-artifact gap).
  3. Extend the pre-`docker run` readiness gate (added in Wave 2) to also require sim REACHABILITY (a
     successful `curl` against the sim the harness itself started), not merely a bound port — DRAIN-13's
     sim-readiness-race leg.
DRAIN-14 gitignore: add to `.gitignore` a pattern ignoring `.sim-*.log` scoped to
`quality/reports/verifications/docs-repro/` (mirror the existing scoped-log convention in `.gitignore`,
e.g. the `p94-git243-fallback-sentinel-*.log` precedent). Confirm `git check-ignore` matches a sample
`quality/reports/verifications/docs-repro/.sim-example-01.json`-adjacent `.sim-*.log`.
Author `container-rehearse-exit-from-artifact.sh` (SC4 selftest): force an artifact with `exit_code: 1`
while simulating docker rc=0 (a stub row / fixture), run the harness's exit-derivation path, assert the
harness process exits 1; AND assert `git check-ignore quality/reports/verifications/docs-repro/.sim-x.log`
succeeds. Degrade to NOT-VERIFIED where docker substrate is absent (the gitignore leg still runs).
  </action>
  <verify>
    <automated>bash quality/gates/docs-repro/container-rehearse-exit-from-artifact.sh && git check-ignore quality/reports/verifications/docs-repro/.sim-example-01-shell-loop.log; echo "ignore-rc=$?"</automated>
  </verify>
  <done>container-rehearse.sh derives its exit strictly from the persisted artifact exit_code (a forced
  artifact exit_code=1 with docker rc=0 → harness exits 1, selftest-proven); the readiness gate requires
  sim reachability; `.gitignore` ignores `.sim-*.log` under quality/reports/verifications/docs-repro/;
  the SC4 selftest passes.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| container stdout → harvested `asserts_passed` | The example runs in a docker container seeded (indirectly) by attacker-influenced sim data; its stdout is Tainted. The harness harvests `ASSERT-PASS:` lines from it and grades a gate on them. |
| ephemeral sim on 127.0.0.1:7878 | A background process the harness owns; an orphan from a prior run is an untrusted pre-existing listener. |
| workflow YAML → provenance gate | The gate reads `.github/workflows/quality-post-release.yml`; a silently-removed build step must be caught, not assumed. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-124-01 | Spoofing | A compromised/lazy example script prints a fake `ASSERT-PASS:` line without the real work | mitigate | ASSERT-PASS lines are emitted ONLY after the load-bearing step under `set -euo pipefail`; the `container-congruence-earned` meta-check proves a no-op `exit 0` FAILS congruence, so a script that skips the work but prints exit 0 without lines cannot pass. Wording token-maps the real expected.assert, not a generic string. |
| T-124-02 | Tampering | Tainted container stdout routed onward | accept/mitigate | The harvested lines are used ONLY to grade the harness's own artifact — never routed to an outbound side-effect (OP-2); they are diagnostic text, not executed. |
| T-124-03 | DoS / resource-leak | Orphaned sim on 7878 from a SIGKILLed harness blocks/poisons the next run | mitigate | Process-group teardown + internal `timeout` < `timeout_s` (SC2); pre-run port-7878-free FAIL-LOUD gate refuses to reuse a stale listener. |
| T-124-04 | Repudiation | A false-green container row on a cold runner with no `target/debug/reposix` | mitigate | SC3 explicit build step + provenance gate; a missing binary is a fail-loud NOT-VERIFIED skip artifact, never a silent pass. |
| T-124-05 | Elevation-of-privilege | rc=0 masks a real assertion failure recorded in the artifact | mitigate | SC4 derives harness exit strictly from the persisted artifact exit_code. |
</threat_model>

<verification>
## Litmus / phase-close verification (exact commands the gsd-verifier will run)

Run from repo root. Where docker is present, the container legs run for real; where absent, the gate
degrades to NOT-VERIFIED (never a false PASS — OP-2).

- **Catalog integrity (Wave 0):**
  `python3 -c "import sys; sys.path.insert(0,'quality/runners'); import run; [run.load_catalog(p) for p in run.discover_catalogs()]; print('LOADS_OK')"`
  then `python3 quality/runners/run.py --cadence pre-push` → exit 0.

- **SC1 — earned congruence + example-05 real runtime:**
  `bash quality/gates/docs-repro/container-congruence-earned.selftest.sh` (no-op FAILS, real passes) →
  `bash quality/gates/docs-repro/container-congruence-earned.sh` → exit 0;
  `grep -nc 'expected.asserts' quality/gates/docs-repro/container-rehearse.sh` shows NO verbatim-copy
  path (only the deletion is acceptable); example-05 on-host run drives a REAL `[RPX-0503]` /
  `git sparse-checkout` stderr line in its captured transcript (build once:
  `cargo build -p reposix-cli`, foreground).

- **SC2 — SIGKILL-proof teardown + orphan fail-loud:**
  `bash quality/gates/docs-repro/container-rehearse-sigkill-safe.sh` → exit 0; `ps aux | grep -v grep |
  grep 'reposix sim'` clean afterward; `grep -n 'setsid\|kill -.*-\|timeout ' quality/gates/docs-repro/container-rehearse.sh`
  shows the process-group + internal-timeout mechanism.

- **SC3 — binary provenance:**
  `bash quality/gates/structure/container-rehearse-binary-provenance.sh` → exit 0;
  `grep -n 'cargo build -p reposix-cli' .github/workflows/quality-post-release.yml` present with an
  inline provenance comment.

- **SC4 — exit-from-artifact + gitignore:**
  `bash quality/gates/docs-repro/container-rehearse-exit-from-artifact.sh` → exit 0;
  `git check-ignore quality/reports/verifications/docs-repro/.sim-example-01-shell-loop.log` → exit 0.

- **Phase-close cadence + push:** `python3 quality/runners/run.py --cadence pre-push` exit 0;
  `git push origin main` lands BEFORE the verifier subagent; then
  `python3 quality/runners/run.py --cadence post-push --persist` — `code/ci-green-on-main` (P0) asserts
  main's newest `ci.yml` + `release-plz.yml` runs concluded success. Never open P125 over a red main.
</verification>

<success_criteria>
All four ROADMAP SCs verified against reality (not asserted): SC1 congruence EARNED (no-op exit-0
FAILS) + example-05 drives the real runtime error; SC2 teardown SIGKILL-proof + orphan fail-loud; SC3
`target/debug/reposix` provenance guaranteed by an explicit workflow step + gate; SC4 harness exit
derived from persisted artifact exit_code + `.sim-*.log` gitignored. Every requirement ID
(DRAIN-13/14/22/23/24) maps to a committed catalog row minted BEFORE its implementation. `run.py
--cadence pre-push` exit 0; main CI GREEN after push.
</success_criteria>

<output>
After completion, create
`.planning/phases/124-container-rehearse-harness-hardening/124-SUMMARY.md`.
</output>
