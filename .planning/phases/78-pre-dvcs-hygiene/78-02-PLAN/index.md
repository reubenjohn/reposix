---
phase: 78
plan: 02
title: "HYGIENE-02 — land 3 TINY shell verifiers + flip WAIVED → PASS before 2026-05-15"
wave: 1
depends_on: []
requirements: [HYGIENE-02]
files_modified:
  - quality/gates/structure/no-loose-top-level-planning-audits.sh
  - quality/gates/structure/no-pre-pivot-doc-stubs.sh
  - quality/gates/structure/repo-org-audit-artifact-present.sh
  - quality/catalogs/freshness-invariants.json
autonomous: true
mode: standard
---

# Phase 78 Plan 02 — WAIVED-row verifiers (HYGIENE-02)

<objective>
Land three TINY-shape shell verifiers under `quality/gates/structure/` that
implement the assertions of three currently-WAIVED rows in
`quality/catalogs/freshness-invariants.json`. Flip each row's status from
`WAIVED` → `PASS` (deleting the `waiver` block) AS PART OF THE SAME COMMIT that
adds the verifier scripts — this is the **catalog-first** atomic flip per
`quality/PROTOCOL.md` § "Subagents propose; tools validate and mint" + the
recurring success criterion in REQUIREMENTS.md "Catalog-first: phase's first
commit writes catalog rows BEFORE implementation."

Waivers expire 2026-05-15 (~15 days from kickoff 2026-04-30). Waiver
auto-renewal would defeat the catalog-first principle (rows defining a green
contract whose verifier never lands). HYGIENE-02 closes that gap before the
expiry deadline so the v0.13.0 milestone never carries a dimension's PASS
contract on a waiver lifeline.

This plan touches **shell + JSON only** — no cargo. It runs in parallel with
78-01 (gix bump cargo work) per CLAUDE.md "doc-only / planning-only subagents
can still run in parallel with one cargo subagent."

**Important pre-existing state:** the verifier *Python branches* for these
three rows already exist inside `quality/gates/structure/freshness-invariants.py`
(functions `verify_no_loose_top_level_planning_audits`,
`verify_no_pre_pivot_doc_stubs`, `verify_repo_org_audit_artifact_present` at
lines 274 / 303 / 520). The WAIVED status is a **process-level waiver** of
"verifier exists per the docs-alignment dimension precedent shape (TINY .sh
under quality/gates/<dim>/, mirroring jira-adapter-shipped.sh)" — NOT a waiver
of "any verifier exists at all." The CARRY-FORWARD acceptance and the ROADMAP
P78 success criterion both name `quality/gates/structure/<row>.sh` shell files
explicitly. Path forward: author 3 .sh wrappers that EITHER (a) call the
existing Python branch directly OR (b) implement the assertion natively in
shell. Choose (b) for the TINY-shape precedent — `jira-adapter-shipped.sh`
is 23 lines of pure shell. Each verifier here is ≤ 30 lines.

After verifier scripts exist, update each row's `verifier.script` field to
point at the new `.sh` AND delete the `waiver` block AND set `status: PASS`.
The three changes land in one commit so the row's GREEN contract and its
verifier are introduced atomically.
</objective>

<must_haves>
- Three new files exist under `quality/gates/structure/`:
  - `no-loose-top-level-planning-audits.sh`
  - `no-pre-pivot-doc-stubs.sh`
  - `repo-org-audit-artifact-present.sh`
- Each script is **5–30 lines** of `bash` (TINY shape per docs-alignment
  dimension precedent — `quality/gates/docs-alignment/jira-adapter-shipped.sh`
  is the canonical reference; 23 lines).
- Each script:
  - Starts with `#!/usr/bin/env bash` + `set -euo pipefail`.
  - Resolves repo root via `SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd); REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"`.
  - Implements the row's assertion verbatim from the catalog `expected.asserts`
    field (see canonical_refs for each row's exact assertion text).
  - On PASS: prints `PASS: <one-line reason>`; exits 0.
  - On FAIL: prints `FAIL: <one-line reason>` to stderr; exits 1.
  - Has `chmod +x` set (Bash + git-tracked exec bit).
- Each catalog row's `verifier.script` field updates from
  `quality/gates/structure/freshness-invariants.py` to
  `quality/gates/structure/<row-id-tail>.sh`. The `verifier.args` field becomes
  `[]` (the row no longer dispatches by `--row-id`; the .sh implements one
  row).
- Each catalog row's `status` flips `WAIVED` → `PASS`.
- Each catalog row's `waiver` field is deleted (set to `null` per the schema —
  the existing PASS rows in the same file use `"waiver": null`, follow that
  precedent).
- Each catalog row's `last_verified` updates to the current ISO-8601 UTC
  timestamp at the moment of the flip-commit.
- Each catalog row's `artifact` field stays the same path
  (`quality/reports/verifications/structure/<row-id-tail>.json`) — the runner
  writes there.
- The runner accepts the new shape: `python3 quality/runners/run.py --cadence
  pre-push` exits 0 for the three rows AFTER the flip.
- The Python verifier *functions* for these three rows in
  `freshness-invariants.py` (lines 274, 303, 520) STAY IN PLACE — do NOT
  delete. They serve as a regression net + provide the assertion logic the
  shell verifiers translate from. Mark them with a one-line code comment:
  `# Path-forward (P78-02): shell wrapper at quality/gates/structure/<row>.sh
  is now the catalog verifier; this Python branch retained as regression net.`
  This is an Eager-resolution choice — leaves the Python intact so the runner
  can fall back to `--row-id` dispatch if the shell wrapper breaks; cheap (3
  one-line comments) and improves resilience.
</must_haves>

<canonical_refs>
- `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` § "WAIVED-STRUCTURE-ROWS-03"
  — verbatim acceptance criteria + assertion text per row.
- `.planning/research/v0.13.0-dvcs/decisions.md` § "WAIVED structure rows" —
  owner ratification 2026-04-30 ("rows flip from WAIVED → PASS without
  renewing").
- `.planning/REQUIREMENTS.md` HYGIENE-02 — verbatim acceptance.
- `quality/PROTOCOL.md` § "Principle A — Subagents propose with citations;
  tools validate and mint" — 3 sub-points; the .sh files are the validation +
  mint surface here.
- `quality/PROTOCOL.md` § "Verifier subagent prompt template" — for the
  phase-close verifier-subagent dispatch (P78 verdict).
- `quality/catalogs/README.md` § catalog row schema — `verifier.script`,
  `verifier.args`, `status`, `waiver`, `last_verified`, `artifact` field
  semantics.
- `quality/gates/docs-alignment/jira-adapter-shipped.sh` (23 lines) — TINY
  shape precedent. Mirror this structure.
- `quality/catalogs/freshness-invariants.json` rows at lines 324–435:
  - `structure/no-loose-top-level-planning-audits` (lines 324–360)
  - `structure/no-pre-pivot-doc-stubs` (lines 361–397)
  - `structure/repo-org-audit-artifact-present` (lines 398–435)
- `quality/gates/structure/freshness-invariants.py` lines 274 / 303 / 520 —
  existing Python implementations of each assertion (translation source for
  the .sh).
- `CLAUDE.md` "Quality Gates — dimension/cadence/kind taxonomy" — confirms
  TINY shell verifiers belong under `quality/gates/<dim>/`.
- `CLAUDE.md` § Operating Principles #4 (Self-improving infrastructure) +
  "Ad-hoc bash is a missing-tool signal" — these verifiers are committed
  named scripts, not ad-hoc pipelines.

This plan does not introduce new threat-model surface. Each verifier is a
read-only filesystem assertion against catalog-named paths. No `<threat_model>`
delta required.
</canonical_refs>

---

## Chapters

- [T01 — Author `no-loose-top-level-planning-audits.sh`](./t01-no-loose-top-level-planning-audits.md)
- [T02 — Author `no-pre-pivot-doc-stubs.sh`](./t02-no-pre-pivot-doc-stubs.md)
- [T03 — Author `repo-org-audit-artifact-present.sh`](./t03-repo-org-audit-artifact-present.md)
- [T04 — Catalog flip: WAIVED → PASS for all three rows](./t04-catalog-flip.md)
- [T05 — Catalog-first atomic commit + per-phase push](./t05-commit-push.md)
