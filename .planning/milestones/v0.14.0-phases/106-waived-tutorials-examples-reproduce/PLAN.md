---
phase: 106-waived-tutorials-examples-reproduce
plan: overview
type: execute
wave: 1
depends_on: [102]
requirements: [D2-TUTORIAL-REPLAY-CONTAINER-BUDGET-01, D2-TUTORIAL-REPLAY-QL001-01, D2-EXAMPLES-SIM-REACHABLE-01]
autonomous: true
---

<objective>
Drive the five WAIVED `docs-repro` catalog rows — `docs-repro/tutorial-replay`,
`example-01-shell-loop`, `example-02-python-agent`, `example-04-conflict-resolve`,
`example-05-blob-limit-recovery` — from WAIVED to catalog-row PASS before their HARD
DEADLINE 2026-09-15. An unbiased verifier subagent grades the flip; this plan does NOT
self-declare PASS.

Wave 1 (this artifact + `106-DIAGNOSIS.md`) is a PROVE-BEFORE-FIX lane: it ran every gate
for real against a live simulator and pinned the ACTUAL root causes, which differ from the
renewed-P90 waiver text. The fix is delivered by the downstream waves specced below.
</objective>

<scope-correction>
The renewed-P90 waivers and the v0.14.0 ROADMAP § "Phase 106" attribute the failures to
(a) "container never brings up the sim" and (b) a "QL-001 push path-shape bug" (+ a "cold
container cargo-build budget" for tutorial-replay). Wave-1 live re-check (evidence:
`106-DIAGNOSIS.md`) corrects this:

- **QL-001 is confirmed ALREADY-LANDED (P91-02) and works on git 2.25.1.** A manual push
  round-trip pushes `main -> main` (exit 0) and writes `helper_push_accepted` to
  `audit_events_cache`. The `agent-ux/ql-001-canonical-path-shape` row is PASS. **No Rust
  fix is in scope for this phase.**
- **The tutorial-replay verifier runs ON HOST, never in docker** (warm build 0.16s), so
  the "cold container build budget" cause is MOOT for the gate as written.
- **The unnamed real root cause is path drift:** the tutorials/examples predate the QL-001
  canonical shape and still assume flat, zero-padded `0001.md` at repo root. Canonical is
  `issues/<unpadded-id>.md` (`issues/1.md … issues/6.md`). Examples 02/04/05 fail on this
  *even with a reachable sim*; example-01 survives only because it uses a recursive grep.

Do NOT plan a Rust change or a container-prewarm lane. The remaining work is (c) BOTH:
shell/harness (sim reachability + tutorial-replay port) AND example-script (Python/bash)
path fixes.
</scope-correction>

<catalog-first-note>
The five GREEN-contract rows ALREADY EXIST in `quality/catalogs/docs-reproducible.json`
(minted long before this phase; their `expected.asserts` predate the fix). This phase does
NOT mint new rows — it removes the `waiver` block and flips `status` WAIVED → PASS on the
existing rows once the fix waves land and the verifier grades them GREEN. The verifier
reads the pre-existing `expected.asserts`, so those asserts are the frozen contract:
tighten the *code/harness* to meet them, do not weaken the asserts to meet the code
(example-02 exception below).
</catalog-first-note>

<green-contract>
Row-by-row PASS definition the verifier will grade (from the live catalog asserts):

1. **docs-repro/tutorial-replay** — `bash quality/gates/docs-repro/tutorial-replay.sh`
   exits 0; sim listens; `reposix init` exits 0; `git checkout -B main
   refs/reposix/origin/main` exits 0; the seeded record file exists with `title:`
   frontmatter; `git push` reports `main -> main`; `audit_events_cache` has a
   `helper_push_%` row.
2. **docs-repro/example-01-shell-loop** — `bash examples/01-shell-loop/run.sh` exits 0
   with a sim reachable; partial clone at `/tmp/reposix-example-01`; push succeeds; audit
   shows `helper_push_%`.
3. **docs-repro/example-02-python-agent** — `python3 examples/02-python-agent/run.py`
   exits 0; the agent MATCHES ≥1 issue mentioning `database`, applies `severity:`, and
   `git push` succeeds (must FAIL LOUD on zero matches — see honesty note).
4. **docs-repro/example-04-conflict-resolve** — `bash examples/04-conflict-resolve/run.sh`
   exits 0; second agent observes `fetch first`, recovers via `git pull --rebase`; both
   edits merge.
5. **docs-repro/example-05-blob-limit-recovery** — `bash
   examples/05-blob-limit-recovery/run.sh` exits 0; agent observes the blob-limit error,
   recovers via `git sparse-checkout`; targeted blob materialized, siblings sparse.

(example-03 is already PASS — out of scope; do not touch.)
</green-contract>

<deliverables>

## D0 — Wave-1 diagnosis (DONE, this wave)
`106-DIAGNOSIS.md` — per-row real results + firm call. Committed alongside this PLAN.

## D1 — Harness: sim reachability for the container-rehearse example rows (shell)
**Rows:** example-01/02/04/05. **File:** `quality/gates/docs-repro/container-rehearse.sh`
(+ optionally the example `run.sh`/`run.py` sim-reachability preambles).
**Root cause:** the driver runs `row.command` inside `docker run … -w /workspace` with the
container's isolated loopback; nothing binds a sim on `127.0.0.1:7878` reachable from
inside. **Fix options (executor picks, documents tradeoff):** start an ephemeral
`reposix-sim` and expose it to the container run (`--network host`, or a sidecar +
`host.docker.internal`, or run the example against a sim on the shared docker network),
OR — if the row's intent is host-rehearsal — run the example on host with a driver-managed
ephemeral sim and correct the misleading `verifier.container` field. Whichever path: the
example must reach a seeded sim and the audit `helper_push_%` row must be assertable.

## D2 — Example-script path drift → canonical `issues/<unpadded-id>.md` (Python + bash)
**Rows:** example-02/04/05. NO Rust.
- `examples/02-python-agent/run.py:90` — `WORK.glob("*.md")` misses `issues/*.md`. Switch
  to the canonical bucket (`WORK.glob("issues/*.md")` or a recursive walk) AND add a
  non-empty-match guard so a zero-match run exits NON-ZERO (closes the exit-0-on-no-op
  honesty hazard). Downstream `git add p.name` must use the `issues/`-relative path.
- `examples/04-conflict-resolve/run.sh:38` — `ls "$WORK_A"/*.md` → `ls
  "$WORK_A"/issues/*.md`; propagate the `issues/` prefix through the edit/commit steps.
- `examples/05-blob-limit-recovery/run.sh:46,54` — `sparse-checkout set '0001.md' …` →
  `issues/1.md issues/2.md …` (unpadded, bucketed); fix the `ls "$WORK"/*.md` inspection
  lines to `issues/*.md`.
- Refresh each example's `expected-output.md` / `RUN.md` to the new paths so the docs and
  the script agree (docs-repro honesty).

## D3 — tutorial-replay harness + doc path refresh (shell + doc)
**Row:** tutorial-replay. **Files:** `quality/gates/docs-repro/tutorial-replay.sh`,
`docs/tutorials/first-run.md`. NO Rust.
- **T1 port mismatch:** the harness binds its sim on `:7780` but `reposix init sim::demo`
  targets the default `:7878`. Bind the ephemeral sim on the default `127.0.0.1:7878`
  (matching the documented front door in first-run.md), OR teach the init step the port.
  Keep it leaf-isolated (its own `/tmp` clone + ephemeral in-mem sim).
- **T2 stale path assertion:** step 5 checks `-f $clone/0001.md`; the real file is
  `issues/1.md`. Update the assertion and the step-6 edit target to the canonical path,
  and refresh `docs/tutorials/first-run.md` (steps 5–8) to `issues/1.md`.
- Correct the misleading catalog `verifier.container: ubuntu:24.04` field (the script runs
  on host) OR genuinely containerize + pre-warm — document the choice.

## D4 — Catalog flip + verify
Remove the `waiver` blocks and set `status: PASS` on the five rows in
`quality/catalogs/docs-reproducible.json` ONLY after D1–D3 land and pass on a real run.
Phase close: `git push origin main` BEFORE the verifier subagent; unbiased verifier grades
the five rows from committed artifacts; verdict at `quality/reports/verdicts/p106/VERDICT.md`.
</deliverables>

<constraints>
- **One cargo invocation machine-wide.** Never run two cargo commands at once; prefer
  `-p <crate>`. The gate scripts build/`reposix-sim` themselves — do not launch a parallel
  cargo. Reuse `target/debug/` binaries where possible.
- **Leaf isolation (fail-closed hook, exit 2).** Every `reposix init/attach/sync`,
  sim-seed, `git init/commit/config` for TEST SETUP runs in a throwaway `/tmp` clone with
  `cd /tmp/...` in the SAME Bash invocation. NEVER mutate the shared repo's git state or
  `.git/config`. The example/gate scripts already self-target `/tmp`; preserve that.
- **No Rust.** QL-001 is landed; this phase is shell + Python + docs + catalog only. If a
  genuine Rust regression surfaces, STOP and route it (Rule 4 architectural) — do not
  quietly patch core here.
- **Commit explicit paths only** — never `git add .`/`-A`, never `git clean`/`stash`.
  Foreign untracked dirs (`.planning/phases/21-*`, `22-*`, `scripts/demos/`,
  `scripts/dev/`) and any foreign stash are NOT ours — leave them.
- **Push cadence:** phase close pushes `origin main` BEFORE the verifier; do NOT push in
  the diagnosis wave.
- **No `--no-verify`.**
</constraints>

<success-criteria>
1. Five `docs-repro` rows flip WAIVED → PASS in `quality/catalogs/docs-reproducible.json`;
   waiver blocks removed; 2026-09-15 deadline met with margin.
2. `tutorial-replay.sh` exits 0 end-to-end on host (sim reachable, canonical path
   assertion, push `main -> main`, audit `helper_push_%` row).
3. `examples/01,02,04,05` `run.sh`/`run.py` exit 0 against a reachable sim, exercise their
   asserted behavior (example-02 matches ≥1 `database` issue + labels + pushes; example-04
   conflict-recovers; example-05 blob-limit-recovers), and the container-rehearse driver
   reaches a seeded sim.
4. example-02 no-op run exits NON-ZERO (honesty guard); example docs/`expected-output.md`
   agree with the scripts' canonical paths.
5. No Rust changed. Phase pushed; unbiased verifier GREEN; verdict at
   `quality/reports/verdicts/p106/VERDICT.md`; main CI green after push.
</success-criteria>
