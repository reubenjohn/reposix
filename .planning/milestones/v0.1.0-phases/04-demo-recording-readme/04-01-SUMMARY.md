---
phase: 04-demo-recording-readme
plan: 01
type: summary
completed: 2026-04-13T11:13:00Z
duration_min: 25
commits:
  - 1655815  # feat(04-01): extend demo seed to 6 issues
  - c1af7e4  # feat(04-01): add scripts/demo.sh
  - b6a2eb5  # docs(04-01): record script(1) typescript + transcript
  - fdeb211  # docs(04-01): add docs/demo.md walkthrough
files_created:
  - scripts/demo.sh
  - docs/demo.typescript
  - docs/demo.transcript.txt
  - docs/demo.md
files_modified:
  - crates/reposix-sim/fixtures/seed.json
  - crates/reposix-sim/src/seed.rs
  - crates/reposix-sim/src/routes/issues.rs
requirements:
  - FC-09  # Demo-ready by morning
  - SG-08  # Demo recording must show guardrails firing
  - FC-01  # Simulator-first architecture (re-validated)
  - FC-02  # Issues as Markdown + YAML (re-validated)
  - FC-03  # FUSE mount with read+write (re-validated)
  - FC-05  # Working CLI orchestrator (re-validated)
  - FC-06  # Audit log (re-validated)
---

# Phase 4 Plan 01: demo script + recording — SUMMARY

Idempotent 9-step demo driver, simulator seed extended to 6 issues for
the SG-02 bulk-delete cap demo, `script(1)` typescript recording, and a
walkthrough page a third party can paste verbatim. All four tasks
shipped as four separate commits.

## Tasks shipped

| # | Task                                                  | Commit    |
|---|-------------------------------------------------------|-----------|
| 1 | Extend demo seed to 6 issues                          | `1655815` |
| 2 | scripts/demo.sh — 9-step idempotent driver            | `c1af7e4` |
| 3 | Record docs/demo.typescript + transcript              | `b6a2eb5` |
| 4 | docs/demo.md walkthrough                              | `fdeb211` |

## Verification — `bash scripts/demo.sh`

**First run:** exit 0, ~25s wall clock (release build was already cached).
**Second consecutive run:** exit 0, ~25s wall clock — idempotent (trap
cleanup + clean-slate guard at the top).

## Recording verification (the SG-08 grep assertions)

Re-run against `docs/demo.typescript` immediately after recording at
2026-04-13 04:09 PDT:

```
grep -c "SG-02 fired as expected" docs/demo.typescript        => 1
grep -cE "allowlist refused|origin not allowlisted|EIO|ENOENT|Permission denied" docs/demo.typescript => 4
grep -c "== DEMO COMPLETE ==" docs/demo.typescript            => 1
grep -cE "EPERM|append-only|refusing|forbidden|allowlist" docs/demo.typescript => 5
```

Every required marker fires. The ROADMAP Phase 4 SC #3 grep
(`EPERM|append-only|refusing|forbidden|allowlist >= 1`) passes with 5
matches.

T-04-01 (info disclosure) check: `grep -iE "HOME=|/home/reuben|ssh|token|secret|password|api_key"` against the typescript returned no matches.

## docs/demo.md grep assertions

```
grep -c '## Walkthrough' docs/demo.md                  => 1
grep -cE 'ALLOWED_ORIGINS|allowlist' docs/demo.md      => 6
grep -c 'threat-model-and-critique' docs/demo.md       => 1
```

## Deviations from plan

1. **Step 6 (`sed -i` → `sed | printf >`).** The plan used `sed -i` on
   the FUSE mount, but the FUSE FS rejects filenames not matching
   `<id>.md` (SG-04). `sed -i` creates a temp file like `sed.XYZ` →
   `EINVAL`. **Rule 1 fix:** read, transform in memory, write back via
   `printf > file` (single open(O_TRUNC)+write).
2. **Step 7 (`git pull` → `git fetch + git checkout -B main`).** The
   plan used `git pull origin main --allow-unrelated-histories`, which
   fails on this v0.1 helper because git's post-fetch logic tries to
   read `refs/reposix/main` (the helper exposes the import as
   `refs/reposix/origin/main`). **Rule 1 fix:** explicit `git fetch ||
   true` (the spurious "fatal:" exit 128 is masked; the actual ref is
   verified with `git rev-parse --verify`) followed by `git checkout
   -B main refs/reposix/origin/main`. Documented as v0.1 wart in
   `docs/demo.md` "Limitations" section.
3. **`git init -b main` → `git init -q + git symbolic-ref`.** The plan
   used `git init -q -b main`, which requires git ≥ 2.28; this dev host
   is git 2.25. **Rule 3 fix:** portable `git symbolic-ref HEAD
   refs/heads/main`.
4. **Step 2 (`cargo test --workspace --quiet`).** The plan ran tests
   with `--quiet` and tail -8'd the output. With `--quiet`, cargo
   collapses progress dots but still emits a `test result:` line per
   binary; we summarise by counting the per-binary lines for a clean
   "passed=133 failed=0 ignored=3" header that's easier to read on the
   recording.
5. **Allowlist refusal manifests as `Permission denied` on `ls`,
   not `EIO`.** The FUSE daemon returns the error to the kernel; the
   kernel surfaces it through `ls` as `Permission denied`. The stderr
   from the daemon includes the exact `origin not allowlisted: ...`
   message. The recording captures both.

All five deviations are tracked above and in commit messages.

## Files created

- `scripts/demo.sh` (240 lines, executable)
- `docs/demo.typescript` (102 lines, 3.6 KB raw `script(1)` output)
- `docs/demo.transcript.txt` (102 lines, ANSI-stripped, ASCII text)
- `docs/demo.md` (278 lines, walkthrough with `## Walkthrough` heading)

## Files modified

- `crates/reposix-sim/fixtures/seed.json` (3 → 6 issues)
- `crates/reposix-sim/src/seed.rs` (test assertions: 3 → 6)
- `crates/reposix-sim/src/routes/issues.rs` (next-id assertion 4 → 7)

## Self-Check

- [x] `scripts/demo.sh` exists, executable, 240 lines (≥ 150 required).
- [x] `docs/demo.typescript` exists, non-empty, contains all 3 required
      markers.
- [x] `docs/demo.transcript.txt` exists, non-empty, ASCII text.
- [x] `docs/demo.md` exists, has `## Walkthrough`, mentions allowlist,
      links to threat-model.
- [x] `crates/reposix-sim/fixtures/seed.json` has 6 issues.
- [x] `cargo test -p reposix-sim` green (29 tests).
- [x] `cargo fmt --all --check` clean.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` clean.
- [x] Four atomic commits landed: 1655815, c1af7e4, b6a2eb5, fdeb211.

Plan 04-01 COMPLETE.
