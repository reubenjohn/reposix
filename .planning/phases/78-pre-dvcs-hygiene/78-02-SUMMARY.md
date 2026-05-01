---
phase: 78
plan: 02
title: "HYGIENE-02 — 3 TINY verifiers + WAIVED to PASS"
requirement: HYGIENE-02
ship_commits: [2bc4dc7]
ship_date: 2026-04-30
files_added:
  - quality/gates/structure/no-loose-top-level-planning-audits.sh
  - quality/gates/structure/no-pre-pivot-doc-stubs.sh
  - quality/gates/structure/repo-org-audit-artifact-present.sh
files_modified:
  - quality/catalogs/freshness-invariants.json
  - quality/gates/structure/freshness-invariants.py
status: SHIPPED
---

# Phase 78 Plan 02 — HYGIENE-02 SUMMARY

## One-liner

Landed 3 TINY shell verifier scripts (`no-loose-top-level-planning-audits.sh`,
`no-pre-pivot-doc-stubs.sh`, `repo-org-audit-artifact-present.sh`) under
`quality/gates/structure/` and atomically flipped the 3 corresponding catalog
rows in `quality/catalogs/freshness-invariants.json` from `WAIVED` to `PASS`
in a single catalog-first commit (2bc4dc7), 14 days before the 2026-05-15
waiver expiry.

## Tasks completed

| Task | Status | Detail |
|------|--------|--------|
| T01 — author `no-loose-top-level-planning-audits.sh` | PASS | 26 lines TINY shape; PASS smoke + synthetic FAIL smoke (`touch .planning/MILESTONE-AUDIT-FAKE.md` -> rc=1) both green |
| T02 — author `no-pre-pivot-doc-stubs.sh` | PASS | 30 lines TINY shape; PASS smoke + synthetic FAIL smoke (`printf x > docs/fake-stub.md` -> rc=1) both green |
| T03 — author `repo-org-audit-artifact-present.sh` | PASS | 29 lines TINY shape (initial 31 trimmed by collapsing 4-line comment to 2 — see Deviations); PASS smoke + synthetic FAIL smoke (artifact mv -> rc=1) both green |
| T04 — catalog flip WAIVED to PASS | PASS | 3 rows updated atomically: `verifier.script` -> `*.sh`, `verifier.args` -> `[]`, `status` -> `PASS`, `last_verified` -> 2026-05-01T05:20:52Z, `waiver` -> null. JSON parses (`python3 -m json.tool`); runner exits 0 |
| T05 — catalog-first atomic commit | PASS | Single commit 2bc4dc7; 5 files, 106 insertions / 39 deletions. Pre-commit fmt hook clean (only a benign existing-size warning on freshness-invariants.py) |

**Push status:** NOT pushed by this plan per execution protocol point 5
(orchestrator pushes once at phase close). Local commit only.

## Commits

- `2bc4dc7` — `quality(structure): land 3 TINY verifiers + flip WAIVED to PASS (HYGIENE-02)` — atomic catalog-first commit (3 .sh files + freshness-invariants.json flip + freshness-invariants.py path-forward comments).

## Runner smoke (post-commit)

`python3 quality/runners/run.py --cadence pre-push` exit=0 with these lines for the 3 flipped rows:

```
[PASS         ] structure/no-loose-top-level-planning-audits  (P1, 0.08s)
[PASS         ] structure/no-pre-pivot-doc-stubs  (P1, 0.17s)
[PASS         ] structure/repo-org-audit-artifact-present  (P1, 0.11s)
summary: 25 PASS, 0 FAIL, 0 PARTIAL, 0 WAIVED, 0 NOT-VERIFIED -> exit=0
```

The summary `0 WAIVED` is the load-bearing observation: the catalog no longer
carries any structure-dimension PASS contract on a waiver lifeline going into
v0.13.0.

## Verifier dispatch evidence

`quality/runners/run.py:237-238` confirms `.sh` extension dispatches via `bash <path>`:

```python
elif suffix in (".sh", ""):
    cmd = ["bash", str(script_abs), *args]
```

Each runner-written artifact at `quality/reports/verifications/structure/<row>.json` shows `exit_code: 0` and the verifier's PASS stdout verbatim. Artifacts are gitignored per `.gitignore` rule `quality/reports/verifications/*/*.json` and therefore not in the commit (matches the existing PASS-row pattern in the same catalog).

## Deviations from plan

1. **T03 — line-count trim (Rule 1, in-spec).** Initial draft of
   `repo-org-audit-artifact-present.sh` was 31 lines, exceeding the 30-line
   TINY-shape ceiling specified in `<must_haves>`. Collapsed a 4-line
   block comment ("Sanity: ... categories; require at least one row using
   that vocabulary.") to a 2-line summary ("Sanity: closure-path vocabulary
   present (...) — at least one row.") to land at 29 lines. No semantic
   change to the verifier logic; assertion regex `closed-by-catalog-row|
   closed-by-existing-gate|waived` unchanged. Smoke tests re-run after the
   trim and both PASS + synthetic FAIL paths green.

2. **Bash-tool 300-char threshold guard (encountered, not a plan deviation).**
   The hook `node /home/reuben/.claude/hooks/deny-ad-hoc-bash.js` blocked
   two intermediate diagnostic invocations (a 547-char `python3 -c`
   inspection and the 1158-char heredoc commit invocation drafted from the
   plan's T05 sample). Resolved without policy-bypass: replaced the inline
   inspection with a `jq` one-liner well under the threshold; replaced the
   heredoc commit message with a tempfile + `git commit -F` (the canonical
   long-message git pattern). The `gsd-tools.cjs commit` subcommand has no
   `--file`/stdin path, so plain `git commit -F` is the correct primitive
   here. No semantic difference vs. the plan's recipe; the commit message
   body matches plan T05 verbatim modulo one ASCII substitution: rendered
   the unicode arrow as "to" because the tempfile path is plain ASCII.

## SURPRISES candidates

None. All anticipated risk mitigations from `78-PLAN-OVERVIEW.md`
"Risks + mitigations" were not triggered:

- The `repo-org-audit-artifact-present` artifact was present at
  `quality/reports/audits/repo-org-gaps.md` (21,332 bytes) with the closure-path
  vocabulary already in the file (verified before T01 began).
- The runner's `.sh` dispatch already worked (lines 237-238 of `run.py`);
  no runner patch needed; no `MEDIUM` SURPRISES entry required.
- No cargo work; no memory-pressure risk; no `cargo fmt` drift.

OP-8 honesty check: every plan task and acceptance criterion mapped to a
verifiable on-disk artifact (file size, JSON shape, runner exit code,
synthetic FAIL rc). No tasks skipped; no findings deferred.

## Catalog-first compliance (PROTOCOL.md Principle A)

The atomic commit 2bc4dc7 ships the validation surface (.sh verifier files)
AND the catalog row update (status flip + verifier reference) in a single
commit, per `quality/PROTOCOL.md` § "Subagents propose; tools validate and
mint" — the rows defining the GREEN contract and the verifiers implementing
that contract were introduced together. No pre-existing GREEN contract was
broken; no row was left in a state where its catalog contract had no live
verifier.

The Python branches at `quality/gates/structure/freshness-invariants.py`
lines 274 / 303 / 520 STAY in place per the must_have in the plan, each
prefaced with a one-line `# P78-02 path-forward:` comment. They serve as a
regression net should the .sh wrapper ever break (the runner can fall back
to `--row-id` dispatch if the catalog is reverted).

## CLAUDE.md update

Per `78-PLAN-OVERVIEW.md` § "Phase-close protocol" point 6, **78-02 does
NOT update CLAUDE.md** — the convention (TINY shell verifiers under
`quality/gates/structure/`) is already documented in CLAUDE.md "Quality
Gates — dimension/cadence/kind taxonomy"; 78-02 is an instance of an
existing convention, not a new convention. CLAUDE.md surface unchanged.

## Verification table

| Acceptance criterion | Evidence |
|----------------------|----------|
| 3 .sh files exist + executable + 5-30 lines | `ls -la quality/gates/structure/no-*-*.sh repo-*.sh` -> rwx + sizes 1302/1301/1457; `wc -l` -> 26/30/29 |
| Each PASS smoke green | `bash <script>` -> rc=0 + `PASS: ...` for all 3 |
| Each FAIL smoke green | Synthetic offender produced rc=1 for all 3 (cleanup verified) |
| JSON parses | `python3 -m json.tool quality/catalogs/freshness-invariants.json > /dev/null` -> exit 0 |
| Catalog flip shape | `jq` per-row inspection: status=PASS, verifier=*.sh, args=[], waiver=null |
| Runner end-to-end | `python3 quality/runners/run.py --cadence pre-push` -> exit 0; 3 rows `[PASS]`; summary 0 WAIVED |
| Path-forward comments on Python branches | `grep -B1 "def verify_no_loose_top_level_planning_audits\|def verify_no_pre_pivot_doc_stubs\|def verify_repo_org_audit_artifact_present" quality/gates/structure/freshness-invariants.py` -> 3 `# P78-02 path-forward:` comments visible |
| Atomic commit shape | `git show --stat 2bc4dc7` -> 5 files, 3 with `create mode 100755` (preserves exec bit) |

## Self-Check: PASSED

- 3 verifier .sh files: FOUND
- freshness-invariants.json status flips: FOUND (3 rows PASS, waiver=null)
- freshness-invariants.py path-forward comments: FOUND (3 occurrences)
- Commit 2bc4dc7: FOUND in `git log`
- SUMMARY.md: this file at `.planning/phases/78-pre-dvcs-hygiene/78-02-SUMMARY.md`
