---
quick_id: 260715-h1d
slug: scratch-repo-keep-policy
type: execute
status: complete
files_modified: [docs/reference/testing-targets.md]
commit: a165d4858976c4475a16f4e6719210f5bc6aa947
requirements: [QUICK-260715-h1d]
---

# Quick 260715-h1d: Scratch-repo KEEP-policy — Summary

One-liner: Recorded a durable KEEP-policy (never-delete / force-push-reset /
unarchive-before-reuse) for the `reposix-scope-test-DELETEME` GitHub scratch
remote-target as a new subsection in `docs/reference/testing-targets.md`, and
eager-fixed a stale Phase-36 cleanup-automation forward-reference in the same
GitHub section.

## What was done

- **Task 1 (as planned):** Inserted `### Scratch repo — `reposix-scope-test-DELETEME`
  (KEEP-policy)` into the GitHub section of `docs/reference/testing-targets.md`,
  placed exactly between the `### Cleanup` owner-permission blockquote and the `---`
  divider that precedes `## JIRA`. Content states: throwaway private scratch
  remote-target (not an issues target); NEVER delete — reset via `git push --force`
  to keep the URL/identity stable; currently ARCHIVED (`archived: true`,
  `private: true`, last pushed `2026-07-14`, per the PLAN's live-verified grounding —
  GitHub was NOT re-hit); unarchive before first reuse via
  `gh api -X PATCH repos/reubenjohn/reposix-scope-test-DELETEME -f archived=false`.
  Matched the doc's tight imperative voice and ~68-char wrapping. The nested ```bash
  fence renders as a normal fenced code block.

## Deviations from plan

None to the primary task — executed exactly as written.

## Noticing (ownership deliverable)

- **Phase-36 stale forward-reference — CONFIRMED STALE, EAGER-FIXED (authorized).**
  The GitHub `### Cleanup` subsection read: *"The Phase 36 cleanup automation will
  handle this; for now manual cleanup at ..."*. The project is at v0.15.0 / P115 —
  Phase 36 is long past. I verified no such automation shipped:
  - `git log --oneline --all -- '*cleanup*'` → only unrelated commits (seed, CATALOG
    split, phase-dir clears); no cleanup-automation feature.
  - `grep -ri "kind:test" scripts/ crates/` → nothing.
  - `grep -rl "label:kind:test\|bulk.close\|is:issue" scripts/ crates/ .github/` →
    nothing; `ls scripts/ | grep -i cleanup` → nothing.
  - `grep -rl "Phase 36" .planning/ docs/` → only historical planning/research files,
    never a shipped artifact.

  Confirmed absent, so I rewrote the sentence to present-tense reality in the SAME
  atomic commit, keeping the `kind:test` label guidance and the manual issues-URL:
  *"…and bulk-closed at session end. No cleanup automation exists yet — close them
  manually at <https://github.com/reubenjohn/reposix/issues>."* Dropped the false
  "Phase 36 will handle this" promise (a lying-doc claim).

- No other issues spotted near this work. The surrounding GitHub/JIRA/Confluence
  sections are byte-for-byte unchanged (diff is +25/-2, confined to the two touched
  regions of the one file).

## Verification against reality

- `git show --stat HEAD` → **exactly one file changed**:
  `docs/reference/testing-targets.md | 27 +++++++--`, `1 file changed, 25 insertions(+),
  2 deletions(-)`.
- Anchored-heading grep count == 1; `gh api ... archived=false` grep count == 1.
- One-file diff-scope guard: `git diff --name-only | wc -l` == 1 (pre-commit).
- `bash quality/gates/docs-build/mkdocs-strict.sh` → `OK: docs site clean`
  (built in 1.25s, no strict warnings).
- Pre-commit hook ran clean (no `--no-verify`): 1 PASS, 0 FAIL, 1 WAIVED (unrelated
  pre-existing file-size waiver), exit 0.

## Commit

- `a165d4858976c4475a16f4e6719210f5bc6aa947` —
  `docs(testing-targets): record reposix-scope-test-DELETEME KEEP-policy`
  (body notes the Phase-36 eager-fix; trailers `Co-Authored-By: Claude Sonnet 5` +
  `Claude-Session: gsd-executor`).

## Handoff notes for orchestrator

- NOT pushed (per contract — orchestrator owns push + STATE.md).
- No `.planning/` artifacts committed by this executor. The PLAN.md / this SUMMARY.md
  under `.planning/quick/260715-h1d-scratch-repo-keep-policy/` remain untracked for the
  orchestrator to commit separately.

## Self-Check: PASSED

- File exists and contains the KEEP-policy subsection: FOUND
  (`docs/reference/testing-targets.md`).
- Commit exists: FOUND (`a165d48`).
