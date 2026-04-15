---
phase: "19"
plan_id: "19-B"
type: execute
wave: 2
depends_on:
  - "19-A"
goal: "Green workspace gauntlet (fmt + clippy + test across all crates), CHANGELOG entry for Phase 19, STATE.md cursor advance, and phase SUMMARY."
files_modified:
  - CHANGELOG.md
  - .planning/STATE.md
  - .planning/ROADMAP.md
autonomous: true
requirements:
  - LABEL-01
  - LABEL-02
  - LABEL-03
  - LABEL-04
  - LABEL-05

must_haves:
  truths:
    - "`cargo test --workspace --quiet` passes with no test failures"
    - "`cargo clippy --workspace --all-targets -- -D warnings` emits zero warnings"
    - "`cargo fmt --all -- --check` exits 0"
    - "CHANGELOG.md has a `### Added — Phase 19` section documenting the labels/ overlay"
    - "ROADMAP.md Phase 19 entry is marked complete (`[x]`)"
    - "STATE.md cursor is advanced past Phase 19"
  artifacts:
    - path: "CHANGELOG.md"
      provides: "Phase 19 Added section under [Unreleased]"
      contains: "labels/"
    - path: ".planning/STATE.md"
      provides: "Phase 19 complete status"
---

<objective>
Run the full workspace green-gauntlet (fmt + clippy + test), write the CHANGELOG entry for Phase 19, mark Phase 19 complete in ROADMAP.md and STATE.md, and commit everything.

Purpose: Close the release cycle for Phase 19. No code changes — this plan only runs verification and updates documentation artifacts.

Output:
- Green CI-equivalent gauntlet across all workspace crates
- CHANGELOG.md `### Added — Phase 19` section
- ROADMAP.md Phase 19 marked `[x]`
- STATE.md cursor advanced
- `19-B-SUMMARY.md` committed
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/ROADMAP.md
@CHANGELOG.md
@.planning/phases/19-op-1-remainder-labels-and-spaces-directory-views-as-read-onl/19-A-SUMMARY.md
</context>

<tasks>

<task type="auto">
  <name>Task B-1: Full workspace green-gauntlet</name>
  <files></files>
  <action>
Run the four CI-equivalent commands in order. Each must exit 0 before proceeding to the next. Do not proceed to Task B-2 until all four are green.

```bash
# 1. Format check
cargo fmt --all -- --check

# 2. Clippy across the whole workspace, all targets, deny all warnings
cargo clippy --workspace --all-targets -- -D warnings

# 3. Unit tests (no ignored integration tests)
cargo test --workspace --quiet

# 4. Final check pass
cargo check --workspace --quiet
```

If any command fails:
- `cargo fmt` failure: run `cargo fmt --all` to fix, then re-check.
- `cargo clippy` failure: read the warning output carefully. Non-exhaustive match on `InodeKind` means a FUSE callback arm was missed in Task A-2 — add the missing arms in `fs.rs`, then re-run.
- `cargo test` failure: read the failing test name and module. Fix the root cause in the relevant source file, then re-run the test suite.

Do NOT mark this task done if any of the four commands is non-zero.
  </action>
  <verify>
    <automated>cargo fmt --all -- --check &amp;&amp; cargo clippy --workspace --all-targets -- -D warnings &amp;&amp; cargo test --workspace --quiet</automated>
  </verify>
  <done>
    All four commands exit 0. No test failures. No clippy warnings. Formatted code is unchanged.
  </done>
</task>

<task type="auto">
  <name>Task B-2: CHANGELOG + ROADMAP + STATE update</name>
  <files>
    CHANGELOG.md
    .planning/STATE.md
    .planning/ROADMAP.md
  </files>
  <action>
**Step 1 — CHANGELOG.md.**

Read `CHANGELOG.md`. In the `## [Unreleased]` section, add a new subsection immediately after the Phase 18 entry (or as the first entry if Phase 18 is already under a released version tag):

```markdown
### Added — Phase 19: OP-1 remainder — `labels/` symlink overlay

- **`labels/` read-only overlay (Phase 19):** `mount/labels/<label>/` lists all
  issues/pages carrying that label as symlinks pointing to the canonical bucket
  file (`../../<bucket>/<padded-id>.md`). Labels populated from `Issue::labels`
  (already present for sim and GitHub adapter; Confluence adapter defers labels
  to a later phase). Each `labels/<label>/` directory uses `slug_or_fallback` +
  `dedupe_siblings` for filesystem-safe, collision-free directory names.
  Requirements: LABEL-01, LABEL-02, LABEL-03, LABEL-04, LABEL-05.
- **Inode constants:** `LABELS_ROOT_INO = 0x7_FFFF_FFFF`,
  `LABELS_DIR_INO_BASE = 0x10_0000_0000`, `LABELS_SYMLINK_INO_BASE = 0x14_0000_0000` —
  disjoint from all existing ranges; const-assertions pin the ordering.
- **`.gitignore` update:** synthesized `.gitignore` now contains `/tree/\nlabels/\n`
  (was `/tree/\n`) so `git status` inside the mount stays clean.
- **`_INDEX.md` update:** `mount/_INDEX.md` now includes a `labels/` row with
  distinct label count.
- **`spaces/` deferred to Phase 20** — requires new `IssueBackend` trait surface
  (`list_spaces`) and Confluence-only API calls. See `19-RESEARCH.md §Scope Recommendation`.
```

**Step 2 — ROADMAP.md.**

Read `.planning/ROADMAP.md`. Find the Phase 19 entry. Change `- [ ]` to `- [x]`. Update the Plans list to show both plans checked:

```
Plans:
- [x] 19-A-labels-fuse-impl.md — labels/ inode constants, pure module, FUSE dispatch
- [x] 19-B-docs-and-release.md — green gauntlet, CHANGELOG, STATE update
```

**Step 3 — STATE.md.**

Read `.planning/STATE.md`. Update the current phase cursor from Phase 19 to Phase 20 (or "complete" if Phase 20 is not yet planned). Add Phase 19 to the completed phases list with a one-line summary: "Phase 19: `labels/` read-only symlink overlay — LABEL-01..05."

**Step 4 — Commit.**

Stage and commit:
```bash
git add CHANGELOG.md .planning/STATE.md .planning/ROADMAP.md \
  .planning/phases/19-op-1-remainder-labels-and-spaces-directory-views-as-read-onl/
git commit -m "docs(19): CHANGELOG + STATE + ROADMAP — Phase 19 labels/ overlay complete"
```
  </action>
  <verify>
    <automated>grep -q "labels/" CHANGELOG.md &amp;&amp; grep -q "\[x\].*Phase 19" .planning/ROADMAP.md &amp;&amp; echo "docs updated"</automated>
  </verify>
  <done>
    - CHANGELOG.md contains `### Added — Phase 19` section with labels/ documented
    - ROADMAP.md Phase 19 entry is `[x]`
    - STATE.md cursor is past Phase 19
    - All changes committed to git
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| docs-only plan | No new code or trust boundaries introduced in this plan. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-19-B-01 | — | No new threats | accept | This plan only runs CI commands and updates Markdown files. All security mitigations are in Plan 19-A. |
</threat_model>

<verification>
```bash
# Verify docs updated
grep -q "labels/" CHANGELOG.md && echo "CHANGELOG OK"
grep -q "\[x\]" .planning/ROADMAP.md && echo "ROADMAP OK"

# Final workspace green-check
cargo test --workspace --quiet
```
</verification>

<success_criteria>
- `cargo fmt --all -- --check` exits 0
- `cargo clippy --workspace --all-targets -- -D warnings` exits 0 with zero warnings
- `cargo test --workspace --quiet` exits 0 with no failures
- CHANGELOG.md `### Added — Phase 19` section present
- ROADMAP.md Phase 19 marked `[x]`
- STATE.md cursor advanced
- All changes committed
</success_criteria>

<output>
After completing both tasks, create `.planning/phases/19-op-1-remainder-labels-and-spaces-directory-views-as-read-onl/19-B-SUMMARY.md` following the template at `@$HOME/.claude/get-shit-done/templates/summary.md`.
</output>
