---
phase: 14
wave: C
runner: gsd-executor
timestamp: 2026-04-14T16:38Z
verdict: PASS
---

# Phase 14 VERIFICATION

> Runner: Wave C executor, 2026-04-14T16:38Z (local 09:38 PDT).
> Scope: integration-verify — read-only against production code; compose the
> verification doc that Wave D's CHANGELOG will cite.

## Summary

Phase 14 met its goal: every write through FUSE (`release`, `create`) and every
write through `git-remote-reposix` now flows through `IssueBackend` trait methods,
not through the deleted `fetch.rs` / `client.rs` modules. The refactor is
wire-compatible with the sim — workspace tests stay at 274 passing (≥272 floor per
LD-14-08), clippy clean with `-D warnings`, green-gauntlet --full 6/6 gates green,
smoke demos 4/4, and the Tier-1 live write-demo (`01-edit-and-push.sh`) exits 0
with "DEMO COMPLETE" after a full FUSE edit → sim PATCH → git clone → git commit
→ git push → sim PATCH round-trip. Sqlite spot-check of the sim's `audit_events`
table confirms the R2 behaviour change: all writes now carry
`agent_id = reposix-core-simbackend-<pid>-{fuse,remote}`; the old
`reposix-fuse-<pid>` / `git-remote-reposix-<pid>` attribution strings are absent.
R1 (assignee-clear on untouched PATCH) is a documented semantic drift not exercised
by demo 01 (issue 1's seed has `assignee: null`, so clear-vs-preserve is behaviour-
equivalent here) — Wave D owns the CHANGELOG entry. No regressions surfaced. Wave D
is cleared to proceed.

## Pre-flight (Task C.1)

```
$ git log --oneline -10 | grep -E '\(14-B[12]\)'
bdad951 refactor(14-B1): route fs.rs write path through IssueBackend trait
cd50ec5 test(14-B1): re-home SG-03 egress-sanitize proof onto SimBackend
938b8de refactor(14-B2): route reposix-remote through IssueBackend trait

$ git status --short
(clean)

$ cargo check --workspace --locked
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.26s
```

All three prior-wave commits present on `main`. Working tree clean. R11 lockfile
sanity check passes.

## SC verification table

| SC      | Verdict | Evidence (section)                                      |
| ------- | ------- | ------------------------------------------------------- |
| SC-14-01 | PASS   | §Grep proofs — zero `patch_issue` code refs in fs.rs   |
| SC-14-02 | PASS   | §Grep proofs — zero `post_issue` code refs in fs.rs    |
| SC-14-03 | PASS   | §Grep proofs — fetch.rs + tests/write.rs gone; no mod  |
| SC-14-04 | PASS   | §Grep proofs — zero `api::` refs anywhere in remote/   |
| SC-14-05 | PASS   | §Grep proofs — client.rs gone; no `mod client`          |
| SC-14-06 | PASS   | Wave B1 commit cd50ec5 re-homed SG-03; tests still green |
| SC-14-07 | PASS   | §Workspace tests + clippy + §green-gauntlet            |
| SC-14-08 | PASS   | §smoke (4/4) + §live write-demo                         |
| SC-14-09 | PASS   | §live write-demo — conflict pathway not exercised live, but `update_issue_with_timeout_times_out_within_budget` + `patch_issue_409_returns_conflict` re-home tests cover the error-surface contract (see B1 test log, 274 passing) |
| SC-14-10 | N/A    | Docs sweep is Wave D scope — not re-verified here      |

## Raw evidence

### Workspace tests (Task C.2)

`cargo test --workspace --locked` — aggregated per-binary totals:

```
Passed: 274  Failed: 0  Ignored: 11
```

Per-binary breakdown (from `/tmp/phase14-c-test.log`, last line of each group):

```
ok.   3 passed (confluence)
ok.   5 passed,  1 ignored (core)
ok.  28 passed (fuse unittests)       <— includes the Phase-14 re-homed tests
ok.   5 passed,  2 ignored (bulk_delete_cap)
ok. 101 passed (sim unittests)        <— includes the SG-03 re-home + sim 409 pins
ok.   8 passed (sim integration/api)
ok.   3 passed,  1 ignored (github — one live-creds-required)
ok.   9 passed,  1 ignored (mounted_fs)
ok.  36 passed (fuse integration)     <— includes the --ignored FUSE mount tests
ok.   8 passed,  1 ignored (reposix-cli)
... (remaining small binaries 0-15 each, see /tmp/phase14-c-test.log)

Totals: 274 passed, 0 failed, 11 ignored
```

LD-14-08 floor ≥ 272 satisfied (+2 over floor).

Exit code: 0. Duration: ~9s under cached state, ~42s cold in the gauntlet.

### Clippy (Task C.2)

`cargo clippy --workspace --all-targets --locked -- -D warnings`

Tail:

```
    Checking reposix-core v0.4.0 (/home/reuben/workspace/reposix/crates/reposix-core)
    Checking reposix-sim v0.4.0 (/home/reuben/workspace/reposix/crates/reposix-sim)
    Checking reposix-github v0.4.0 (/home/reuben/workspace/reposix/crates/reposix-github)
    Checking reposix-confluence v0.4.0 (/home/reuben/workspace/reposix/crates/reposix-confluence)
    Checking reposix-swarm v0.4.0 (/home/reuben/workspace/reposix/crates/reposix-swarm)
    Checking reposix-remote v0.4.0 (/home/reuben/workspace/reposix/crates/reposix-remote)
    Checking reposix-fuse v0.4.0 (/home/reuben/workspace/reposix/crates/reposix-fuse)
    Checking reposix-cli v0.4.0 (/home/reuben/workspace/reposix/crates/reposix-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 11.24s
```

Zero warnings. Exit 0. (Rebuilt after `cargo clean -p reposix-fuse -p reposix-remote -p reposix-core` to prove the lint actually fires fresh.)

### fmt (Task C.2)

`cargo fmt --all --check` — exit 0, no output.

### Grep proofs (Task C.3)

#### SC-14-01 — FUSE PATCH through trait

```
$ git grep -n 'patch_issue\b' crates/reposix-fuse/src/fs.rs
crates/reposix-fuse/src/fs.rs:1423:        // defence-in-depth the old `fetch::patch_issue` provided on top of
```

Only hit is inside a doc-comment block inside a re-homed test function at
`fs.rs:1418-1424`; it cites the origin of the test verbatim for grounding.
Plan expressly permits prose-only references ("doc-comment mentions are allowed").

Positive check — the new trait path is present:

```
$ git grep -n 'backend\.update_issue\|update_issue_with_timeout' crates/reposix-fuse/src/fs.rs
crates/reposix-fuse/src/fs.rs:58:   //! `update_issue_with_timeout` / `create_issue_with_timeout`. On timeout
crates/reposix-fuse/src/fs.rs:225:  async fn update_issue_with_timeout(...)
crates/reposix-fuse/src/fs.rs:234:          backend.update_issue(project, id, patch, expected_version),
crates/reposix-fuse/src/fs.rs:1139: let result = self.rt.block_on(update_issue_with_timeout(
... (9 more hits in re-homed tests)
```

Verdict: PASS.

#### SC-14-02 — FUSE POST through trait

```
$ git grep -n 'post_issue\b' crates/reposix-fuse/src/fs.rs
(zero hits)
```

Positive check:

```
$ git grep -n 'backend\.create_issue\|create_issue_with_timeout' crates/reposix-fuse/src/fs.rs
crates/reposix-fuse/src/fs.rs:58:   //! `update_issue_with_timeout` / `create_issue_with_timeout`. On timeout
crates/reposix-fuse/src/fs.rs:247:  async fn create_issue_with_timeout(...)
crates/reposix-fuse/src/fs.rs:252:          match tokio::time::timeout(READ_GET_TIMEOUT, backend.create_issue(project, issue)).await {
crates/reposix-fuse/src/fs.rs:1218: let result = self.rt.block_on(create_issue_with_timeout(
... (re-homed test hits)
```

Verdict: PASS.

#### SC-14-03 — `fetch.rs` + `tests/write.rs` deleted, `EgressPayload` gone

```
$ test ! -e crates/reposix-fuse/src/fetch.rs && echo "fetch.rs gone"
fetch.rs gone
$ test ! -e crates/reposix-fuse/tests/write.rs && echo "write.rs gone"
write.rs gone
$ git grep -n 'pub mod fetch' crates/reposix-fuse/src/lib.rs
(zero hits)
$ git grep -n 'EgressPayload' crates/reposix-fuse/
(zero hits)
```

One residual mention surfaces workspace-wide in `crates/reposix-core/src/backend/sim.rs:742`, inside a comment that
explains why the sim-side `render_patch_body` was authored the way it was —
prose-only, documenting the old shape for future readers. Acceptable per LD-14-07
intent (no live code path, no dead #[allow(dead_code)], just one breadcrumb in a
comment).

Verdict: PASS.

#### SC-14-04 — remote helper through trait

```
$ git grep -n 'api::\(list_issues\|patch_issue\|post_issue\|delete_issue\)' crates/reposix-remote/src/main.rs
(zero hits)
$ git grep -n 'api::' crates/reposix-remote/src/
(zero hits)
```

Positive check (`state.backend.*` call sites ≥ 4):

```
$ git grep -n 'state\.backend\.' crates/reposix-remote/src/main.rs
crates/reposix-remote/src/main.rs:176:  let issues = match state.rt.block_on(state.backend.list_issues(&state.project)) {
crates/reposix-remote/src/main.rs:218:  let prior  = match state.rt.block_on(state.backend.list_issues(&state.project)) {
crates/reposix-remote/src/main.rs:293:          .block_on(state.backend.create_issue(&state.project, untainted))
crates/reposix-remote/src/main.rs:311:          .block_on(state.backend.update_issue(
crates/reposix-remote/src/main.rs:323:          .block_on(state.backend.delete_or_close(
```

5 call sites. Verdict: PASS.

#### SC-14-05 — `client.rs` deleted

```
$ test ! -e crates/reposix-remote/src/client.rs && echo "client.rs gone"
client.rs gone
$ git grep -n '^mod client\b' crates/reposix-remote/src/
(zero hits)
$ git grep -n 'mod client\|use crate::client' crates/reposix-remote/src/main.rs
(zero hits)
```

Verdict: PASS.

#### R7 — residual `fetch::` imports in reposix-fuse

```
$ git grep -n 'fetch::' crates/reposix-fuse/src/
crates/reposix-fuse/src/fs.rs:1423:        // defence-in-depth the old `fetch::patch_issue` provided on top of
```

Same single doc-comment hit flagged under SC-14-01. Zero live code references.

### green-gauntlet --full (Task C.4)

```
$ bash scripts/green-gauntlet.sh --full
== [running] fmt
✓ fmt ok 0s
== [running] clippy
✓ clippy ok 1s
== [running] test
✓ test ok 9s
== [running] smoke
✓ smoke ok 9s
== [running] mkdocs-strict
✓ mkdocs-strict ok 2s
== [running] fuse-ignored
✓ fuse-ignored ok 2s

✓ green gauntlet passed
```

Exit 0. Wall-clock duration: ~24s on second pass (cached), ~55s on first pass
against cold compile artifacts. 6/6 gates green, including `fuse-ignored` which
exercises `cargo test --release -p reposix-fuse --locked -- --ignored
--test-threads=1` (the nested_layout, sim_death_no_hang, readdir mounted-FS
integration tests).

### smoke.sh (Task C.5)

```
$ bash scripts/demos/smoke.sh
================================================================
  reposix demos — smoke suite (4 demos)
================================================================

>>> 01-edit-and-push.sh
== assert.sh: demo exited with rc=0
== assert.sh: PASS (01-edit-and-push.sh)

>>> 02-guardrails.sh
== assert.sh: PASS (02-guardrails.sh)

>>> 03-conflict-resolution.sh
== assert.sh: PASS (03-conflict-resolution.sh)

>>> 04-token-economy.sh
== assert.sh: PASS (04-token-economy.sh)

================================================================
  smoke suite: 4 passed, 0 failed (of 4)
================================================================
```

Exit 0. 4/4.

### Live write-demo (Task C.6)

```
$ PATH="$PWD/target/release:$PATH" bash scripts/demos/01-edit-and-push.sh
...
[3/6] edit through FUSE (sed-style in-memory write)
  before: status = open
  after FUSE write, frontmatter head:
  ---
  id: 1
  title: database connection drops under load
  status: in_progress
  ...
  server confirms:
  {
    "id": 1,
    "status": "in_progress",
    "version": 2
  }

[4/6] git clone via reposix:: remote
[5/6] edit + commit + push
  pushing...
      To reposix::http://127.0.0.1:7801/projects/demo
       * [new branch]      main -> main

[6/6] verify server state reflects the push
  issue 1 status after push (expect in_review):
  in_review

== DEMO COMPLETE ==
```

Exit 0. All three of the demo's asserted markers (`DEMO COMPLETE`, `status: in_progress`, `in_review`) present. Both the FUSE write path (step 3) and the git-remote-reposix push path (step 5) round-tripped a PATCH through the trait into the sim and the sim's stored state reflects both mutations.

### Audit attribution spot-check (Task C.7)

The demo's `SIM_DB` at `/tmp/reposix-demo-01-sim.db` was polled in a concurrent
sidecar while the demo ran; the last snapshot taken immediately before the demo's
cleanup trap unlinked the file:

```
$ sqlite3 /tmp/reposix-demo-01-sim.db "SELECT agent_id, COUNT(*) FROM audit_events GROUP BY agent_id ORDER BY agent_id;"
anonymous|4
reposix-core-simbackend-1874435-fuse|8
reposix-core-simbackend-1874544-remote|1
reposix-core-simbackend-1874574-remote|2

$ sqlite3 /tmp/reposix-demo-01-sim.db "SELECT agent_id, method, path, status FROM audit_events ORDER BY id DESC LIMIT 15;"
anonymous                             |GET  |/projects/demo/issues/1|200
reposix-core-simbackend-1874574-remote|PATCH|/projects/demo/issues/1|200
reposix-core-simbackend-1874574-remote|GET  |/projects/demo/issues  |200
reposix-core-simbackend-1874544-remote|GET  |/projects/demo/issues  |200
anonymous                             |GET  |/projects/demo/issues/1|200
reposix-core-simbackend-1874435-fuse  |PATCH|/projects/demo/issues/1|200
anonymous                             |GET  |/projects/demo/issues/1|200
reposix-core-simbackend-1874435-fuse  |GET  |/projects/demo/issues/1|200
reposix-core-simbackend-1874435-fuse  |GET  |/projects/demo/issues  |200
reposix-core-simbackend-1874435-fuse  |GET  |/projects/demo/issues  |200
reposix-core-simbackend-1874435-fuse  |GET  |/projects/demo/issues  |200
reposix-core-simbackend-1874435-fuse  |GET  |/projects/demo/issues  |200
reposix-core-simbackend-1874435-fuse  |GET  |/projects/demo/issues  |200
reposix-core-simbackend-1874435-fuse  |GET  |/projects/demo/issues  |200
anonymous                             |GET  |/healthz              |200
```

Verdict:

- FUSE-originated writes (PATCH on step 3) tag as `reposix-core-simbackend-1874435-fuse`. PASS (R2 new-suffix attribution confirmed).
- Remote-helper writes (PATCH on step 5 via git push, plus the GET on step 4's git fetch) tag as `reposix-core-simbackend-{1874544,1874574}-remote`. PASS. (Two distinct PIDs because `git fetch` and `git push` each spawn their own `git-remote-reposix` helper process — expected.)
- No `reposix-fuse-<pid>` or `git-remote-reposix-<pid>` rows appear. PASS.
- `anonymous` rows are the demo's own `curl` probes (steps 3, 6) — these are out-of-scope for the attribution check; `curl` does not set `X-Reposix-Agent`.

Note on schema: the sim table is `audit_events` with a column named `agent_id` (not `audit` / `agent` as the task preface sketched). The plan itself (Task C.7 snippet) matches the actual schema (`SELECT agent, action FROM audit`); I adjusted the query to `agent_id`, `method`, `path`, `status` against the real table after discovery — the spot-check's purpose (confirm new-suffix attribution) is unaffected.

## Behavior-change observations

### R1 — assignee-clear on untouched PATCH

The refactor's `SimBackend::render_patch_body` emits `"assignee": null` on a
PATCH when the untainted `Issue` has `assignee: None`; the pre-refactor
`EgressPayload` omitted the key entirely via `#[serde(skip_serializing_if = "Option::is_none")]`. The sim's `FieldUpdate<String>` deserializer treats absent
as `Unchanged` and `null` as `Clear`, so the new wire body explicitly clears the
assignee on every PATCH from FUSE `release`.

As exercised by `01-edit-and-push.sh`: issue 1's seed has `assignee: null` (see
`crates/reposix-sim/fixtures/seed.json`). Both the FUSE PATCH (step 3) and the
remote-helper PATCH (step 5) round-trip cleanly and the final server state at
step 6 shows `status: in_review` — meaning the write landed. Neither PATCH
materially altered the assignee (the field was already null and the file-backed
frontmatter never introduced a non-null value).

**Live symptom:** none. The demo exercises the neutral case where assignee is
already null and the file is not setting an alternative.

**Documentation action:** Wave D must call this out in CHANGELOG `### Changed`
as the intended, accepted semantic shift per R1's Option (A) "accept the
cleared-on-null behaviour — file is the source of truth."

### R2 — X-Reposix-Agent attribution suffix

Captured live above in §Audit attribution spot-check. All writes through
`IssueBackend` impls now tag with `reposix-core-simbackend-<pid>-{fuse,remote}`
instead of `reposix-fuse-<pid>` / `git-remote-reposix-<pid>`. The `-{fuse,remote}`
suffix is the per-call-site discriminator supplied by
`SimBackend::with_agent_suffix(origin, Some("fuse"|"remote"))`. The `<pid>` is
`std::process::id()` captured at `SimBackend::new` time — stable within one
process, fresh across spawns (hence the two distinct remote PIDs in the fetch +
push path).

**Live symptom:** visible in any `SELECT agent_id, COUNT(*) FROM audit_events
GROUP BY agent_id` query post-v0.4.1. Operators/monitoring jobs that grouped or
filtered on the old prefix strings will need to update their queries.

**Documentation action:** Wave D CHANGELOG `### Changed` entry with the old → new
mapping and a note that the pid suffix is per-process (not per-connection).

## Wave-C-discovered issues

### Issue C-1 (self-discovered) — stale release binaries masked smoke output on first pass

**Symptom:** My first pass of `smoke.sh` + `01-edit-and-push.sh` after B1/B2
landed produced the audit-attribution output
`git-remote-reposix-1869494|PATCH|…` instead of the expected
`reposix-core-simbackend-<pid>-remote`. This initially looked like a remote-
helper regression — a code path bypassing `state.backend`.

**Root cause:** `target/release/git-remote-reposix` had mtime
`2026-04-14 04:28` — predating the B1 commit at `09:24` and the B2 commit at
`09:06`. `scripts/green-gauntlet.sh` does NOT build release binaries; it uses
`target/release/*` if present and falls back to `target/debug/*` only if the
release dir is absent (gauntlet.sh:78-85). The smoke gate inside the gauntlet
had been running against stale binaries — but it passed because the old smoke
markers (`DEMO COMPLETE`, `status: in_progress`, `in_review`) are still emitted
by the pre-refactor code path. The R2 agent-suffix change is not asserted in
the smoke suite.

**Fix:** Rebuilt with `cargo build --release --workspace --bins --locked` (took
1m58s). Re-ran the full gate matrix. Fresh-binary run produced the expected
attribution (`reposix-core-simbackend-<pid>-{fuse,remote}`).

**Disposition:** Not a Phase 14 regression. This is a grounding-infrastructure
gap (per user OP #4): green-gauntlet trusts the freshness of binaries it did
not build. Two viable mitigations for future phases:

1. Add a gate that asserts `target/release/*` mtime > last-relevant-commit mtime,
   and fails loud if stale. Cheap, ~10 lines in the gauntlet driver.
2. Have the gauntlet do a `cargo build --release --workspace --bins --locked` as
   the first gate, regardless of cached state. Slower (~2min cold, <5s warm),
   honest.

I recommend option 2 as the default behaviour of `--full` mode, with a
`--skip-build` escape hatch for humans who just rebuilt. Option 1 is a cheap
complement regardless.

**Flagged to Wave D / orchestrator:** this is outside Wave C's scope; filing as
a deferred item for a follow-up small QoL fix.

### Issue C-2 (self-discovered) — audit_events vs audit naming

The task preface for C.7 referenced `SELECT agent, COUNT(*) FROM audit_events
GROUP BY agent ORDER BY agent` and `SELECT agent, action FROM audit ORDER BY
id DESC LIMIT 10`. The first is correct on the table (`audit_events` ✓) but
wrong on the column (`agent_id` not `agent`); the second uses a table alias
that does not exist (`audit`). The PLAN.md Task C.7 snippet says `SELECT agent,
action FROM audit ORDER BY id DESC LIMIT 10;` — it's aspirational, not against
the real schema.

No impact on verification — I discovered the real schema via
`src/middleware/audit.rs:118` (INSERT INTO audit_events ...) and used the
correct query. Flagging so future wave planners don't copy the snippet
verbatim. The actual schema has no `action` column either; the closest proxies
are `method` (GET/PATCH/…) and `path`. If the intent of the spot-check is a
one-liner rollup by actor and verb, it should be
`SELECT agent_id, method, COUNT(*) FROM audit_events GROUP BY agent_id, method ORDER BY agent_id, method;`.

## Verdict

**Phase 14 integration verified: PASS.**

- SC-14-01..05 satisfied by grep proofs (Task C.3).
- SC-14-06 satisfied by the B1 commit `cd50ec5` re-homing SG-03 onto `SimBackend`;
  the sim crate's 101 passing unit tests include the re-homed coverage.
- SC-14-07 satisfied by `cargo test --workspace --locked` 274 passing / 0 failing
  (LD-14-08 floor ≥272 met +2), clippy `-D warnings` clean, and
  `green-gauntlet --full` 6/6 gates green.
- SC-14-08 satisfied by smoke 4/4 and demo-01 exit 0 with all markers.
- SC-14-09 error-surface satisfied by the re-homed timeout/conflict tests under
  the workspace test suite (covered within the 274).
- SC-14-10 deferred to Wave D per its owner split.
- R1 (assignee clear) and R2 (agent-suffix) documented above as observed behaviour
  changes — Wave D CHANGELOG `### Changed` owns the write-up.

**Wave D is cleared to proceed.**
