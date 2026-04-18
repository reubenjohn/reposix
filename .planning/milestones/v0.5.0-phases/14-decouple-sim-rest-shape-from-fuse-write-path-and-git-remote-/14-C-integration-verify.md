---
phase: 14
wave: C
slug: integration-verify
serial: true
depends_on_waves: [B1, B2]
blocks_waves: [D]
estimated_wall_clock: 30m
executor_role: gsd-executor
---

# Wave C — integration verify

> Read `14-CONTEXT.md` and `14-RESEARCH.md` (whole file) before executing.
> This wave runs **after** both B1 and B2 have merged to the main branch.
> C's job is to prove the two refactored crates together satisfy SC-14-01..09.

## Scope in one sentence

Run the full verification matrix (workspace tests, clippy, `green-gauntlet.sh --full`,
`smoke.sh`, the live write-demo, and grep proofs for SC-14-01..05), then write
`14-VERIFICATION.md` recording results.

## What NOT to touch

- **Do not edit code in Wave C**, except to fix a genuine regression the verification
  surfaces. If a fix is needed, it returns to the responsible wave (B1 or B2) for a
  follow-up commit, and Wave C re-runs from the top. Wave C never ships a "quick patch"
  commit of its own against production code.
- The one exception: if a test is genuinely flaky (unrelated to Phase 14) and the
  investigation proves it, leave a note in `14-VERIFICATION.md`. Do not retro-silence
  flakiness.

## Files to touch

- `.planning/phases/14-decouple-sim-rest-shape-from-fuse-write-path-and-git-remote-/14-VERIFICATION.md`
  — created in this wave with the full output of every check below.

## Files to delete

None.

## Tasks

### Task C.1 — Pre-flight sanity

```bash
# Confirm B1 and B2 both landed.
git log --oneline -10 | grep -E '\(14-B[12]\)'
# Expect at least two lines.

git status
# Expect clean (no staged/unstaged changes).

cargo check --workspace --locked
# Expect green. This is R11 lockfile-sanity check.
```

If `git status` is not clean, stop and triage — something is wrong with the
branch state.

### Task C.2 — Workspace-wide tests + clippy

```bash
cargo test --workspace --locked 2>&1 | tee /tmp/phase14-c-test.log
cargo clippy --workspace --all-targets --locked -- -D warnings 2>&1 | tee /tmp/phase14-c-clippy.log
```

Acceptance:

- `cargo test --workspace --locked` — all tests pass. Test count (per LD-14-08)
  must be **≥ 272**. If below, halt Wave C and return to B1 for a test-add round.
- `cargo clippy ...` — clean, no warnings that trigger `-D warnings`.

Record both exit codes + tail of each log in `14-VERIFICATION.md`.

### Task C.3 — Grep proofs for SC-14-01..05

Run each grep below. Record exact command + output in `14-VERIFICATION.md`.

```bash
# SC-14-01 (FUSE PATCH through trait)
git grep -n 'patch_issue\b' crates/reposix-fuse/src/fs.rs        # expect: zero hits
git grep -n 'backend\.update_issue\|update_issue_with_timeout' crates/reposix-fuse/src/fs.rs
                                                                  # expect: at least one hit in release

# SC-14-02 (FUSE POST through trait)
git grep -n 'post_issue\b' crates/reposix-fuse/src/fs.rs         # expect: zero hits
git grep -n 'backend\.create_issue\|create_issue_with_timeout' crates/reposix-fuse/src/fs.rs
                                                                  # expect: at least one hit in create

# SC-14-03 (fetch.rs deleted)
test ! -e crates/reposix-fuse/src/fetch.rs && echo "fetch.rs gone"   # expect: prints
test ! -e crates/reposix-fuse/tests/write.rs && echo "write.rs gone" # expect: prints
git grep -n 'pub mod fetch' crates/reposix-fuse/src/lib.rs       # expect: zero hits
git grep -n 'EgressPayload' crates/reposix-fuse/                 # expect: zero hits

# SC-14-04 (remote helper through trait)
git grep -n 'api::\(list_issues\|patch_issue\|post_issue\|delete_issue\)' \
    crates/reposix-remote/src/main.rs                            # expect: zero hits
git grep -n 'state\.backend\.' crates/reposix-remote/src/main.rs # expect: ≥ 4 hits

# SC-14-05 (client.rs deleted)
test ! -e crates/reposix-remote/src/client.rs && echo "client.rs gone"  # expect: prints
git grep -n 'mod client\|use crate::client' crates/reposix-remote/src/main.rs
                                                                  # expect: zero hits

# R7 sanity — no surviving fetch:: imports
git grep -n 'fetch::' crates/reposix-fuse/src/                   # expect: zero hits
```

If any expectation fails, halt Wave C. Return to responsible wave (B1 for FUSE,
B2 for remote) with a diff-patch describing the missing cleanup. Wave C re-runs after
the fix commit lands.

### Task C.4 — green-gauntlet --full

```bash
bash scripts/green-gauntlet.sh --full 2>&1 | tee /tmp/phase14-c-gauntlet.log
```

The `--full` flag includes `cargo test --workspace --release -- --ignored`, which runs
the FUSE integration tests that actually mount a real FUSE filesystem. This is where
Phase 13 Wave C's nested-layout tests and Phase 10's read-path tests exercise end-to-end
behavior.

Acceptance: exit 0. Record the tail of the log (last ~40 lines) in
`14-VERIFICATION.md`.

If gauntlet fails:

- If the failure is in a FUSE `--ignored` integration test, suspect B1 — either
  `update_issue_with_timeout` / `create_issue_with_timeout` has a subtle bug, or the
  sanitizer test caught a regression. Return to B1.
- If the failure is in a `protocol.rs` or `bulk_delete_cap.rs` test, suspect B2 — the
  wire-level behavior drifted. Return to B2.
- If the failure is elsewhere (unrelated crate), investigate — possibly a Phase 13
  regression surfaced by chance. Do NOT silently carry.

### Task C.5 — smoke demos

```bash
bash scripts/demos/smoke.sh 2>&1 | tee /tmp/phase14-c-smoke.log
```

Acceptance: exit 0, **4 of 4** passing. Record the summary line from the log in
`14-VERIFICATION.md`.

### Task C.6 — Live write-demo (R8 mitigation)

This is the authoritative end-to-end proof that the write path through FUSE lands on
the sim and the audit log.

```bash
bash scripts/demos/01-edit-and-push.sh 2>&1 | tee /tmp/phase14-c-writedemo.log
```

The demo's built-in assertions (see its header: `ASSERTS: "DEMO COMPLETE"
"status: in_progress" "in_review"`) prove the PATCH landed on the sim via both the
FUSE mount path and the git-remote-reposix push path.

Acceptance: exit 0, `"DEMO COMPLETE"` present in stdout. Record in
`14-VERIFICATION.md`.

### Task C.7 — Manual audit-attribution spot check (R2 proof)

After C.6 runs, the sim's runtime audit database has rows from the demo. Verify the
new attribution strings:

```bash
# Find the sim's DB (demo script seeds it in runtime/).
ls runtime/*.db 2>/dev/null || ls /tmp/reposix-sim-*.db 2>/dev/null | head -3

# Query the agent column on the most recent rows.
sqlite3 <path-to-sim-db> "SELECT agent, action FROM audit ORDER BY id DESC LIMIT 10;"
```

Acceptance:

- The FUSE-originated writes (PATCH / POST) show `agent LIKE 'reposix-core-simbackend-%-fuse'`.
- The remote-helper-originated writes (if the demo exercises them — 01-edit-and-push.sh
  does) show `agent LIKE 'reposix-core-simbackend-%-remote'`.
- No rows have the old `reposix-fuse-%` or `git-remote-reposix-%` prefix.

Record the output in `14-VERIFICATION.md`. This is evidence for Wave D's CHANGELOG entry.

**Note:** if the demo script spins up its own sim with a per-run DB path, find the
actual path from its stdout logs. `scripts/demos/_lib.sh` likely prints it.

### Task C.8 — Optional: write `14-VERIFICATION.md`

Compose the verification doc now:

```markdown
# Phase 14 VERIFICATION

> Runner: Wave C executor, <timestamp>.

## Pre-flight (C.1)

<git log + git status + cargo check output>

## Workspace tests + clippy (C.2)

- `cargo test --workspace --locked`: <N passing>, <0 failing>, <time>
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: clean

## Grep proofs (C.3)

### SC-14-01
<command + output>

### SC-14-02
<command + output>

... (one block per SC)

## green-gauntlet --full (C.4)

<tail of log, ~40 lines>

Exit code: 0

## smoke.sh (C.5)

<summary line>

4/4 PASSING

## 01-edit-and-push.sh live write-demo (C.6)

<"DEMO COMPLETE" line>

## Audit attribution (C.7)

<sqlite output>

Confirmed: agent strings now use `reposix-core-simbackend-<pid>-{fuse,remote}`.

## Verdict

Phase 14 integration verified. SC-14-01..09 all satisfied. Wave D may proceed.
```

### Task C.9 — Commit `14-VERIFICATION.md`

```
docs(14-C): record phase 14 integration verification results

All SC-14-01..09 verified. green-gauntlet --full green. smoke 4/4.
01-edit-and-push.sh demo exits 0. Audit attribution confirmed.

Wave D cleared to proceed.
```

## Tests to pass before commit

- All checks above green.
- `14-VERIFICATION.md` exists, is non-empty, and records each task's output.
- `git status` clean after the verification doc commit.

## Acceptance criteria

- [ ] `cargo test --workspace --locked` green, ≥ 272 passing.
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings` clean.
- [ ] `scripts/green-gauntlet.sh --full` exits 0.
- [ ] `scripts/demos/smoke.sh` 4/4.
- [ ] `scripts/demos/01-edit-and-push.sh` exits 0.
- [ ] All grep proofs (SC-14-01..05 + R7 sanity) pass.
- [ ] Sqlite spot-check confirms new agent-attribution strings.
- [ ] `14-VERIFICATION.md` committed with all raw outputs.

## Non-scope (reserved for Wave D)

- CHANGELOG entry under `### Changed` — D owns.
- CLAUDE.md sweep of v0.3 deferral prose — D owns.
- `docs/architecture.md` diagram line updates — D owns.
- Tag-push (v0.4.1) — outside the phase; user-gated.

## If verification fails

**Do not ship a "Wave C quick fix" commit against production code.** The discipline is:

1. Record the failure in `14-VERIFICATION.md` as a blocker.
2. Return to the responsible wave (B1 for FUSE regressions, B2 for remote regressions).
3. That wave ships a follow-up commit with a `fix(14-B1)` or `fix(14-B2)` prefix.
4. Wave C re-runs the full matrix from Task C.1.
5. Only when all checks pass does Wave C commit `14-VERIFICATION.md`.

This preserves the audit trail (what broke, where it was fixed, what re-verified it)
and keeps phase prefixes honest — no `fix(14-C)` commits that hide a B1/B2 defect.

## References

- `14-CONTEXT.md` SC-14-07, SC-14-08.
- `14-RESEARCH.md` R7 (read-path grep), R8 (gauntlet coverage check).
- `14-PLAN.md` verification_plan section.
- `scripts/green-gauntlet.sh`, `scripts/demos/smoke.sh`, `scripts/demos/01-edit-and-push.sh`.
