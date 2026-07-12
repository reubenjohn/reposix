# P106 PROVE-BEFORE-FIX Diagnosis — Waived tutorials/examples reproduce

**Wave 1 diagnosis. Author: gsd-executor. Date: 2026-07-12.**
No non-trivial fixes applied — diagnosis only, per the wave charter. Evidence below is
from live gate + on-host runs against a real simulator on the box (git 2.25.1, docker up).

## Test harness used for the repro

- One cargo invocation total: `tutorial-replay.sh` did a (warm, 0.16s) `cargo build -p
  reposix-cli -p reposix-sim -p reposix-remote`. No second concurrent cargo process.
- For the `:7878` example rows I started the **built** binary (not cargo)
  `target/debug/reposix-sim --bind 127.0.0.1:7878 --seed-file
  crates/reposix-sim/fixtures/seed.json --ephemeral` on the host, ran each example's
  `run.sh`/`run.py` directly on host, then stopped the sim.
- Leaf isolation: every `reposix init` / `git commit` for the manual repro ran inside a
  `/tmp/p106-*` clone with `cd /tmp/...` in the same invocation; the example scripts
  self-target `/tmp/reposix-example-0N`.

## Headline findings (correct the waiver/ROADMAP hypotheses)

1. **QL-001 push path bug is CONFIRMED ALREADY-LANDED (P91-02) and works on git 2.25.1.**
   Manual round-trip in `/tmp/p106-full`: edited `issues/1.md`, committed, `git push
   origin main` → `bd848c1..87141f5  main -> main`, exit 0, secret-scan clean.
   `audit_events_cache` shows `helper_push_accepted`, `helper_push_sanitized_field`,
   `helper_push_started`. The `agent-ux/ql-001-canonical-path-shape` catalog row is PASS.
   → **NO Rust code fix is needed for any of the 5 rows.**

2. **The canonical record-path shape is `issues/<unpadded-id>.md`** (QL-001), e.g.
   `issues/1.md` … `issues/6.md`. The tutorials/examples predate that migration and still
   assume the OLD flat, zero-padded shape (`0001.md` at repo root). This is the *unnamed*
   root cause the waivers never mentioned.

3. **The tutorial-replay verifier runs ON HOST, not in a fresh container.** Despite the
   catalog `verifier.container: ubuntu:24.04`, `tutorial-replay.sh` reuses host binaries
   and never enters docker. So the ROADMAP's "cold cargo build inside a fresh container
   exceeds the 5-min budget" cause is **MOOT for the current gate** — the build was warm
   (0.16s). Do not spend the fix wave on container-budget pre-warming for this row.

## Per-row REAL results

### docs-repro/tutorial-replay — FAIL (harness + doc/path drift; NO Rust)
Two distinct harness/doc bugs, both in `quality/gates/docs-repro/tutorial-replay.sh` (and
the doc it mirrors, `docs/tutorials/first-run.md`):

- **Bug T1 — port mismatch.** The harness starts its sim on `127.0.0.1:7780`, but
  `reposix init sim::demo` targets the **default `127.0.0.1:7878`** (init configures
  `remote.origin.url = reposix::http://127.0.0.1:7878/...` — it does not read the 7780
  port). With no sim on 7878, step 3 (init) fails:
  - Run A (only the 7780 sim up): artifact `asserts_failed: ["step 3: reposix init
    failed"]`, exit 1.
  - Run B (a 7878 sim also up): step 3 + step 4 PASS — proving init needs a sim on the
    **default** port the harness never binds.
- **Bug T2 — stale path assertion.** With a reachable sim, step 5 asserts `-f
  $clone/0001.md`. The materialized tree is `issues/1.md … issues/6.md`; `0001.md` at root
  does not exist. Artifact: `asserts_failed: ["step 5: 0001.md missing or has no title:
  frontmatter"]`, exit 1.
- Steps 7 (push `main -> main`) and 8 (`helper_push_%` audit row) **do** succeed once the
  earlier steps are unblocked (proven by the manual round-trip, finding #1).

**Fix class:** shell/harness (bind sim on the default 7878 or teach init the port) + doc
path refresh (`0001.md` → `issues/1.md`). NO Rust.

### docs-repro/example-01-shell-loop — FAIL via harness only; example CODE PASSES on host
- `container-rehearse.sh docs-repro/example-01-shell-loop` → exit 1, artifact stderr:
  `curl: (7) Failed to connect to 127.0.0.1 port 7878 … FAIL: sim not reachable`. The
  driver never starts a sim, and even a host sim on 7878 is unreachable from the
  container's isolated loopback.
- **Example code works on host** (sim on 7878): `bash examples/01-shell-loop/run.sh` →
  push `61d2dcb..09adc73  main -> main`. It survives the path migration only because it
  uses a **recursive** predicate (`grep -lr '^status: open' . --include='*.md'`).

**Fix class:** shell/harness (make a sim reachable inside/around the container run). NO
example-code change strictly required, NO Rust.

### docs-repro/example-02-python-agent — FAIL: example CODE drift + harness gap
- On host with a reachable 7878 sim: `python3 examples/02-python-agent/run.py` → `matched
  0 issue(s) mentioning 'database': []` … `nothing to commit`, **exit 0**.
- **Root cause = code drift:** `run.py:90` globs `WORK.glob("*.md")` (repo root, NON-
  recursive). The records live at `issues/*.md`, so zero match. The seed *does* contain
  `database` (1 hit) — the miss is purely the flat-root glob.
- **Honesty hazard (NOTICING):** run.py exits **0** while doing nothing, so the exit-code-
  driven `container-rehearse.sh` would score this GREEN even though the asserted behavior
  ("finds 'database', applies severity label, push succeeds") never happened. The fix must
  make the example fail loudly when it matches nothing, or the row can lie GREEN.

**Fix class:** example-code (Python: recurse or `issues/` prefix + non-empty guard) +
harness gap. NO Rust.

### docs-repro/example-04-conflict-resolve — FAIL: example CODE drift + harness gap
- On host with a reachable 7878 sim: `bash examples/04-conflict-resolve/run.sh` →
  `ls: cannot access '/tmp/reposix-example-04-A/*.md': No such file or directory`, aborts
  at step 1.
- **Root cause = code drift:** `run.sh:38` `target="$(ls "$WORK_A"/*.md | …)"` globs the
  repo root; canonical paths are `issues/*.md`. The conflict/`fetch first`/`pull --rebase`
  logic is never reached, so it is UNVERIFIED (not disproven).

**Fix class:** example-code (bash path glob) + harness gap. NO Rust.

### docs-repro/example-05-blob-limit-recovery — FAIL: example CODE drift + harness gap
- On host with a reachable 7878 sim: `bash examples/05-blob-limit-recovery/run.sh` →
  `error: Sparse checkout leaves no entry on working directory`, aborts at step 3.
- **Root cause = code drift (double):** `run.sh:46,54` `git sparse-checkout set '0001.md'
  '0002.md' …` uses BOTH the old zero-padding AND the flat-root bucket; canonical is
  `issues/1.md` (unpadded, bucketed). No pathspec matches → empty working tree. The
  blob-limit teaching string / `git sparse-checkout` recovery is never actually exercised.

**Fix class:** example-code (bash sparse-checkout pathspecs `issues/1.md …` + the `ls`
lines) + harness gap. NO Rust.

### docs-repro/example-03-claude-code-skill — PASS (confirmed once; out of scope)

## Firm call on remaining work

**(c) BOTH — but "both" = shell/harness + example-script (Python/bash) path fixes, with
ZERO Rust.** Concretely:

| Row | Harness gap (sim reachability / port) | Example/doc path-drift fix | Rust fix |
|---|---|---|---|
| tutorial-replay | YES (T1 port 7780 vs 7878) | YES (T2 `0001.md`→`issues/1.md`) | no |
| example-01 | YES | no (recursive grep survives) | no |
| example-02 | YES | YES (`run.py` root glob → `issues/*.md` + fail-loud) | no |
| example-04 | YES | YES (`run.sh` root glob → `issues/*.md`) | no |
| example-05 | YES | YES (`run.sh` sparse pathspecs → `issues/<unpadded>.md`) | no |

The QL-001 Rust fix is **confirmed already-landed**; the waiver's "QL-001 push path-shape
bug" cause for tutorial-replay is **stale/resolved**. The ROADMAP's "cold container build
budget" cause is **moot** for the host-run tutorial-replay gate as written.

## Noticing (OD-3 deliverable)

- **Waivers under-reported the failure class.** Examples 02/04/05 fail on stale
  pre-QL-001 path assumptions *independent of* sim reachability; the renewed P90 waivers
  attribute 100% of the failure to "container never brings up sim." A fix wave that only
  starts a sim would flip example-01 GREEN and leave 02/04/05 RED (or, worse for 02,
  falsely GREEN — see honesty hazard).
- **example-02 exits 0 on a no-op.** Exit-code-only grading (`container-rehearse.sh`) can
  score a lying GREEN. Recommend the fix add a non-empty-match guard AND that the catalog
  row / verifier assert the `helper_push_%` audit row + the label, not just exit 0.
- **Catalog `verifier.container: ubuntu:24.04` is misleading for tutorial-replay** — the
  script runs on host and never enters docker. Either wire the container or correct the
  field so the next reader is not misled about where the budget applies.
- **STATE.md staleness:** `last_updated: 2026-07-11`; frontmatter still carries a
  `milestone: v0.13.0 status: closed-green` block and a `v0.13.2 status: queued` block
  alongside the live v0.14.0 wave-2 cursor. Not blocking, but the multi-milestone
  frontmatter is easy to misread; the live cursor is the `last_activity` line (P102 hard
  gate, then P103–P109).

## Evidence index (commands, all reproducible)

- Push round-trip: `/tmp/p106-full`, `git push origin main` → `main -> main`, exit 0.
- Audit rows: `sqlite3 ~/.cache/reposix/sim-demo.git/cache.db "SELECT op FROM
  audit_events_cache WHERE op LIKE 'helper_push%'"` → 3 rows.
- tutorial-replay artifact: `quality/reports/verifications/docs-repro/tutorial-replay.json`
  (two captured runs: step-3 fail, then step-5 fail).
- example-01 container artifact:
  `quality/reports/verifications/docs-repro/example-01-shell-loop.json` (`sim not
  reachable`).
- Tree shape: `git ls-files` in `/tmp/p106-full` → `issues/1.md … issues/6.md`.
