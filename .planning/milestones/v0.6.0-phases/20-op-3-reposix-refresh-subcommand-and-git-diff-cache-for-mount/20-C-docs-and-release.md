---
phase: 20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount
plan_id: 20-C
wave: 3
goal: >
  Update CHANGELOG, STATE.md, ROADMAP.md, and the workspace Cargo.toml version
  to v0.6.0. Write the phase SUMMARY. Ensure CI artifacts are consistent with
  the shipped feature.
depends_on:
  - 20-A
  - 20-B
type: execute
autonomous: true
requirements:
  - REFRESH-01
  - REFRESH-02
  - REFRESH-03
  - REFRESH-04
  - REFRESH-05

must_haves:
  truths:
    - "CHANGELOG has a `[v0.6.0]` section describing `reposix refresh`"
    - "STATE.md `current_phase` is updated to reflect Phase 20 complete"
    - "ROADMAP.md Phase 20 entry is marked done"
    - "Workspace version in Cargo.toml is bumped to 0.6.0"
    - "`cargo test --workspace --quiet` passes with the version bump applied"
  artifacts:
    - path: "CHANGELOG.md"
      provides: "[v0.6.0] entry with reposix refresh feature description"
      contains: "v0.6.0"
    - path: ".planning/STATE.md"
      provides: "Phase 20 marked complete, Phase 21 set as next"
      contains: "Phase 20"
    - path: "Cargo.toml"
      provides: "workspace version = \"0.6.0\""
      contains: "0.6.0"
  key_links:
    - from: "CHANGELOG.md"
      to: "REFRESH-01..REFRESH-05"
      via: "human-readable feature summary"
      pattern: "reposix refresh"
---

<objective>
Ship Phase 20: bump workspace version to v0.6.0, write the CHANGELOG entry,
update STATE.md and ROADMAP.md to mark Phase 20 complete, verify the full
workspace builds clean at the new version, and record the phase SUMMARY.

Purpose: Close the release loop — the feature is implemented and tested in
Waves A+B; this wave makes the release artifacts coherent so future phases
start from a clean baseline.

Output:
- `CHANGELOG.md` updated with `[v0.6.0]` section
- `Cargo.toml` workspace version bumped to `0.6.0`
- `.planning/STATE.md` updated (Phase 20 complete, Phase 21 next)
- `.planning/ROADMAP.md` Phase 20 entry updated (plans listed, goal finalized)
- `cargo test --workspace --quiet` green at v0.6.0
- Phase SUMMARY written
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount/20-A-SUMMARY.md
@.planning/phases/20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount/20-B-SUMMARY.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Bump version to v0.6.0 and write CHANGELOG entry</name>
  <files>
    Cargo.toml,
    CHANGELOG.md
  </files>
  <action>
**1. Bump workspace version in `Cargo.toml`:**

Find the `[workspace.package]` section and change:
```toml
version = "0.5.x"   # whatever the current value is
```
to:
```toml
version = "0.6.0"
```

All crates use `version.workspace = true` so this single change propagates.
After editing, run `cargo check --workspace` to confirm the version propagates
without error. Do NOT edit individual crate Cargo.toml files.

**2. Update CHANGELOG.md:**

Read the current CHANGELOG.md to understand the format. Then prepend a new
section at the top (above the most recent existing entry). Use the same heading
style already in the file.

Content for the `[v0.6.0]` entry:

```markdown
## [v0.6.0] — 2026-04-15

### Added

- **`reposix refresh` subcommand** (OP-3, REFRESH-01 through REFRESH-05):
  Re-fetches all issues/pages from the configured backend, writes deterministic
  `.md` files into the mount directory, and creates a git commit so `git log`
  and `git diff HEAD~1` inside the mount show the history of backend snapshots.

  Usage:
  ```
  reposix refresh --mount <path> --project demo [--backend sim|github|confluence]
  ```

  - Writes `<mount>/issues/<padded-id>.md` using the same `frontmatter::render`
    function the FUSE read path uses — guarantees byte-identical output so
    consecutive refreshes with no backend changes produce an empty `git diff`.
  - Creates `<mount>/.reposix/fetched_at.txt` with an ISO-8601 UTC timestamp.
  - Git commits are authored as `reposix <backend@project>` for clear
    attribution in `git log`.
  - Errors with a clear message if a FUSE mount is active at the target path
    (check `<mount>/.reposix/fuse.pid`); unmount first to avoid routing writes
    through the FUSE write callbacks.
  - `--offline` flag is declared (forward-compat); offline read path ships in
    Phase 21.

- **`cache_db` module** (`crates/reposix-cli/src/cache_db.rs`): minimal SQLite
  metadata store at `<mount>/.reposix/cache.db` (mode 0600) recording last
  fetch time, backend name, and project. SQLite WAL + EXCLUSIVE locking prevents
  concurrent refresh races. The DB is gitignored — only `.md` files and
  `fetched_at.txt` are committed.
```

If CHANGELOG.md does not exist, create it with the above entry as the sole
content (plus a standard preamble "# Changelog\n\n").

**3. Run workspace check:**
```
cargo check --workspace
```
If this fails due to a version-related issue (e.g. Cargo.lock needs update),
run `cargo update -p reposix-core --precise 0.6.0` or simply `cargo build -p reposix-cli`
to trigger Cargo.lock regeneration. The version bump is in workspace package
metadata only — no code changes.
  </action>
  <verify>
    <automated>cargo check --workspace --quiet 2>&1 | tail -10 && grep -c "v0.6.0" CHANGELOG.md</automated>
  </verify>
  <done>
    `cargo check --workspace` exits 0. `CHANGELOG.md` contains a `[v0.6.0]`
    section describing `reposix refresh`. Workspace `Cargo.toml` has
    `version = "0.6.0"`. `cargo test --workspace --quiet` still passes (run
    to confirm no version-sensitive test broke).
  </done>
</task>

<task type="auto">
  <name>Task 2: Update STATE.md, ROADMAP.md, run final gate, write SUMMARY</name>
  <files>
    .planning/STATE.md,
    .planning/ROADMAP.md
  </files>
  <action>
**1. Update `.planning/STATE.md`:**

Read the current STATE.md. Update the following fields:
- `current_phase`: set to `21` (or `next` if Phase 21 is not yet named — use
  the name from ROADMAP.md if a Phase 21 entry exists, otherwise write
  "TBD — Phase 21 (offline FUSE read path)").
- Add Phase 20 to the completed phases list with a one-line summary:
  "Phase 20: `reposix refresh` subcommand + git-diff cache (v0.6.0)".
- Update `last_completed_phase` to `20`.
- Increment the `version` field to `0.6.0` if such a field exists.
- Update `updated_at` to `2026-04-15`.

Do NOT rewrite unrelated fields — surgical edits only.

**2. Update `.planning/ROADMAP.md`:**

Find the Phase 20 entry (search for "Phase 20" or "op-3-reposix-refresh").
Update:
- Mark the phase as `[x]` (done) in the checkbox if it has one.
- Ensure the `**Plans:**` line reflects 3 plans: `20-A`, `20-B`, `20-C`.
- Ensure the plan list is present:
  ```
  Plans:
  - [x] 20-A-refresh-cmd.md — refresh.rs + cache_db.rs + main.rs wiring
  - [x] 20-B-tests-and-polish.md — integration tests + workspace gate
  - [x] 20-C-docs-and-release.md — CHANGELOG + version bump + STATE update
  ```

**3. Final workspace gate:**

Run:
```
cargo test --workspace --quiet
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

If any step fails, fix before proceeding. This is the phase-exit gate.

**4. Write phase SUMMARY:**

Create `.planning/phases/20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount/20-C-SUMMARY.md`
using the standard summary template. Include:
- Phase goal achieved
- Files created/modified: refresh.rs, cache_db.rs, main.rs, tests/refresh_integration.rs,
  CHANGELOG.md, Cargo.toml, STATE.md, ROADMAP.md
- Key decisions made: sub-directory `.reposix/.gitignore` for cache.db WAL
  ignores (avoided modifying FUSE GITIGNORE_BYTES); `run_refresh_inner` split
  for testability; rustix signal-0 or /proc fallback for FUSE-active detection
- Test results: N unit tests + 4 integration tests passing
- Version shipped: v0.6.0
- Deferred: offline FUSE read path (Phase 21), full SQLite issue cache replacing
  DashMap (Phase 21+), `reposix-cache` as its own crate (Phase 21+)
  </action>
  <verify>
    <automated>cargo test --workspace --quiet 2>&1 | tail -5 && grep "current_phase\|last_completed" .planning/STATE.md | head -5</automated>
  </verify>
  <done>
    `cargo test --workspace --quiet` — 0 failures.
    STATE.md `last_completed_phase` = 20.
    ROADMAP.md Phase 20 entry shows 3 plans and is marked done.
    CHANGELOG.md has `[v0.6.0]` entry.
    Phase SUMMARY written at `20-C-SUMMARY.md`.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| CHANGELOG content | Human-authored; no external input. No threat surface. |
| Version bump propagation | Cargo workspace version is a build-time constant. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-20C-01 | Tampering | Cargo.toml version field | accept | Version bumps are committed to git; any unauthorized change is visible in `git log`. Build artifacts are reproducible. |
| T-20C-02 | Repudiation | CHANGELOG authorship | accept | CHANGELOG is committed to git with the standard author. `git log --follow CHANGELOG.md` provides audit trail. |
</threat_model>

<verification>
1. `cargo test --workspace --quiet` — 0 failures
2. `cargo clippy --workspace --all-targets -- -D warnings` — 0 warnings
3. `cargo fmt --all --check` — 0 diffs
4. `grep "0.6.0" Cargo.toml` — version present
5. `grep "\[v0.6.0\]" CHANGELOG.md` — entry present
6. `grep "last_completed_phase.*20" .planning/STATE.md` — phase marked done
</verification>

<success_criteria>
- Workspace version = `0.6.0` in Cargo.toml
- CHANGELOG.md `[v0.6.0]` section present with `reposix refresh` description
- STATE.md reflects Phase 20 complete
- ROADMAP.md Phase 20 has 3 plans listed and is marked done
- Full workspace CI gate green: test + clippy + fmt
- Phase SUMMARY at 20-C-SUMMARY.md
</success_criteria>

<output>
After completion, create `.planning/phases/20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount/20-C-SUMMARY.md`
with the standard summary template fields filled in.
</output>
