---
phase: 11-confluence-adapter
plan: F
subsystem: release-engineering
tags: [release, changelog, tag-script, morning-brief, v0.3.0]
requires:
  - 11-A (reposix-confluence crate)
  - 11-B (CLI dispatch)
  - 11-C (contract test)
  - 11-D (demos)
  - 11-E (docs + env)
provides:
  - MORNING-BRIEF-v0.3.md at repo root
  - CHANGELOG.md [v0.3.0] — 2026-04-14 with BREAKING callout
  - scripts/tag-v0.3.0.sh (executable, 6 safety guards)
  - Prior briefs (MORNING-BRIEF.md, PROJECT-STATUS.md) repointed at v0.3 brief
affects:
  - (Tag push itself NOT executed — gated on Task 4 human-verify per plan design)
tech-stack:
  added: []
  patterns: [release-script-with-safety-guards, changelog-keep-a-changelog, morning-brief-handoff-doc]
key-files:
  created:
    - MORNING-BRIEF-v0.3.md
    - scripts/tag-v0.3.0.sh
    - .planning/phases/11-confluence-adapter/11-F-SUMMARY.md
  modified:
    - CHANGELOG.md
    - MORNING-BRIEF.md
    - PROJECT-STATUS.md
decisions:
  - "Task 5 (tag push) deliberately NOT executed — per user scope note AND per plan's `autonomous: false` + `checkpoint:human-verify` gate. The v0.3.0 tag is a permanent widely-visible artifact and the single remaining human-gate step."
  - "scripts/tag-v0.3.0.sh has no --force override flag. All six safety guards are unconditional; if a guard trips, the operator fixes the root cause."
  - "Tag script extracts its message body from CHANGELOG [v0.3.0] via sed — single source of truth, prevents drift between tag and CHANGELOG."
  - "BREAKING callout added for TEAMWORK_GRAPH_API → ATLASSIAN_API_KEY rename (caught by plan author in Task 2 action spec)."
metrics:
  duration_minutes: 5
  tasks_completed: 3
  tasks_blocked_on_human: 2
  files_created: 3
  files_modified: 3
  commits: 3
  completed_date: "2026-04-14"
---

# Phase 11 Plan F: release Summary

Ship the three release artifacts (MORNING-BRIEF-v0.3.md, CHANGELOG [v0.3.0] promotion, scripts/tag-v0.3.0.sh) with all six safety guards on the tag script. Stop at the human-verify checkpoint — the `git push origin v0.3.0` is the single remaining step, gated on user review per the plan's `autonomous: false` frontmatter and the T-11F-06 mitigation.

## What shipped

| Artifact | Status | Commit |
|---|---|---|
| `MORNING-BRIEF-v0.3.md` (169 lines) | shipped | `c305e5c` |
| `MORNING-BRIEF.md` header pointer to v0.3 brief | shipped | `c305e5c` |
| `PROJECT-STATUS.md` header pointer to v0.3 brief | shipped | `c305e5c` |
| `CHANGELOG.md` [Unreleased] → [v0.3.0] + fresh empty [Unreleased] + BREAKING callout + compare-link for v0.3.0 | shipped | `477a7a4` |
| `scripts/tag-v0.3.0.sh` (93 lines, chmod +x, all 6 guards) | shipped | `d29cdcf` |
| `git push origin v0.3.0` | **NOT EXECUTED** (gated — see below) | — |

Tasks 1-3 auto; Task 4 is a blocking human-verify checkpoint; Task 5 runs only after "approved" signal per the plan.

## Task 4 checkpoint — status

**Not auto-approved.** Per the user's scope note for this session:

> CRITICAL: DO NOT push to origin. Create the tag locally only — `git tag -a v0.3.0 -m "..."` IS allowed (local op, reversible). But `git push origin v0.3.0` is the human-verify step the plan's frontmatter guards — leave it off entirely and document the exact command in MORNING-BRIEF-v0.3.md as "run this one command in the morning".
>
> **Also DO NOT run the actual tag script** — just ship it. The user runs it in the morning after reading the brief.

So: script shipped, tag NOT created, tag NOT pushed. The user runs `bash scripts/tag-v0.3.0.sh` in the morning after eyeballing the CHANGELOG.

Automated verification that DID run (so the morning handoff is on green):

```text
cargo fmt --all --check                                           PASS (exit 0)
cargo clippy --workspace --all-targets --locked -- -D warnings    PASS (clean)
cargo test --workspace --locked                                   PASS 191/191 (5 ignored)
cargo build --release --workspace --bins --locked                 PASS
PATH=target/release:$PATH bash scripts/demos/smoke.sh             PASS 4/4
bash -n scripts/tag-v0.3.0.sh                                     PASS (syntax clean)
grep -qE '^## \[v0\.3\.0\] — 2026-04-14' CHANGELOG.md             PASS
grep -qE '^## \[Unreleased\]$' CHANGELOG.md                       PASS
sed -n '/## \[v0.3.0\]/,/## \[v0.2.0-alpha\]/p' CHANGELOG.md \
    | grep -qE 'BREAKING|breaking change'                         PASS
test -f MORNING-BRIEF-v0.3.md && [ $(wc -l < ...) -ge 100 ]       PASS (169 lines)
```

`shellcheck` was not installed on the dev host; the `bash -n` parse check is the fallback per the user's scope note ("if the script supports --dry-run OR lint the script via shellcheck").

## Success criteria audit (from plan §success_criteria)

| # | Criterion | Result |
|---|---|---|
| 1 | `test -f MORNING-BRIEF-v0.3.md` | PASS |
| 2 | `wc -l MORNING-BRIEF-v0.3.md >= 100` | PASS (169) |
| 3 | `grep -q 'id.atlassian.com/manage-profile/security/api-tokens'` | PASS |
| 4 | `grep -q 'scripts/tag-v0.3.0.sh'` in brief | PASS |
| 5 | `grep -qE '(Known open gaps\|Known gaps)'` | PASS |
| 6 | `test -x scripts/tag-v0.3.0.sh` | PASS |
| 7 | `bash -n scripts/tag-v0.3.0.sh` | PASS |
| 8 | all six guards present in script | PASS (g1-g6 verified individually) |
| 9 | `grep -qE '^## \[v0\.3\.0\] — 2026-04-14'` | PASS |
| 10 | `grep -qE '^## \[Unreleased\]$'` | PASS |
| 11 | BREAKING callout in v0.3.0 block | PASS |
| 12 | `git tag -l v0.3.0` | N/A — Task 5 deferred |
| 13 | `git ls-remote --tags origin v0.3.0` | N/A — Task 5 deferred |
| 14 | `gh run list --limit 5` | N/A — Task 5 deferred |

11/11 in-scope criteria green. 3 criteria (12-14) explicitly gated on Task 5 per the plan.

## The one command the user runs in the morning

```bash
bash scripts/tag-v0.3.0.sh
```

That script wraps `git tag -a v0.3.0 -m "..."` + `git push origin v0.3.0` behind six safety guards (branch=main, clean tree, no local/remote tag collision, CHANGELOG has v0.3.0 section, cargo test green, smoke.sh green). If any guard trips, the script exits non-zero without tagging. There is no --force flag.

After the push succeeds, optionally: <https://github.com/reubenjohn/reposix/releases/new?tag=v0.3.0> + paste the CHANGELOG [v0.3.0] section as the body.

## Deviations from Plan

**None from the plan's task flow.** The three auto tasks (1, 2, 3) landed exactly per the plan's `<action>` blocks. The Task 4 human-verify checkpoint is reached and paused as designed.

## Deferred / pre-existing drift flagged to user

The worktree carried pre-existing uncommitted changes from prior Phase 11 work (NOT from 11-F):

```
 D .claude/scheduled_tasks.lock
 M benchmarks/RESULTS.md
 M crates/reposix-confluence/Cargo.toml       (adds url workspace dep)
 M crates/reposix-confluence/src/lib.rs       (WR-01/WR-02 hardening)
?? .planning/phases/11-confluence-adapter/11-REVIEW.md
```

Per Scope Boundary rule in the executor spec, NOT touched by this plan. But they WILL trip guard #2 (clean tree) of `scripts/tag-v0.3.0.sh` when the user runs it. MORNING-BRIEF-v0.3.md §"Cutting the tag" now has an explicit "deal with the drift first" block with three options (recommended: commit the hardening as a pre-release bundle). This note IS the handoff.

One non-deviation worth noting: the plan's Task 4 `<how-to-verify>` list included `env -u ATLASSIAN_API_KEY ... cargo test -p reposix-confluence --locked -- --ignored --nocapture 2>&1 | grep 'SKIP:'` and the two demo SKIP-clean checks. Those are part of the **user's** checkpoint review, not the executor's — they're documented in MORNING-BRIEF-v0.3.md §"Prove it works" so the user can run them on the morning walk-through.

## Authentication gates

None encountered. This plan had no backend-touching tasks.

## Known stubs

None. Every artifact is a concrete committed file with real content.

## Threat surface flags

None. The plan itself is a release-engineering plan that adds documentation + an executable script. The script is the new surface, and it's covered by the threat register (T-11F-01..06) with explicit mitigations in the guards.

## Self-Check: PASSED

- FOUND: MORNING-BRIEF-v0.3.md
- FOUND: scripts/tag-v0.3.0.sh (+x, 93 lines)
- FOUND: CHANGELOG.md contains `[v0.3.0] — 2026-04-14`
- FOUND commit: c305e5c (docs(11-F-1))
- FOUND commit: 477a7a4 (docs(11-F-2))
- FOUND commit: d29cdcf (chore(11-F-3))

Phase 11 Plan F in-scope work complete. v0.3.0 ready for human-gated tag push.
