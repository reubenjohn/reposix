---
phase: 30
plan: 06
type: execute
wave: 1
depends_on: [30-01, 30-02]
files_modified:
  - docs/tutorial.md
  - scripts/test_phase_30_tutorial.sh
autonomous: true
requirements: [DOCS-06]
must_haves:
  truths:
    - "docs/tutorial.md is a 4-step runnable guide against the simulator (`reposix-sim`). Each command in the page is copy-pasteable and produces the documented output when run on a stock Linux dev host."
    - "The 'aha' moment (per Stripe quickstart pattern G) lands in step 4 — the reader `git push`es their edit and watches the server-side version bump from 1 to 2 via `curl | jq`."
    - "`scripts/test_phase_30_tutorial.sh` runs the 4-step tutorial end-to-end: starts reposix-sim, mounts via reposix CLI, edits an issue, `git push`es, asserts the version bumped, and tears down. Exits 0 on success, non-zero on any step failing."
    - "The tutorial prose uses zero Layer-3 terms in non-code (FUSE / daemon / kernel / syscall / mount as a noun) — all references appear only inside fenced code blocks."
    - "The word 'replace' does not appear (P1 scoped to index.md but we enforce discipline phase-wide)."
  artifacts:
    - path: "docs/tutorial.md"
      provides: "4-step runnable 5-minute tutorial against reposix-sim"
      min_lines: 80
    - path: "scripts/test_phase_30_tutorial.sh"
      provides: "End-to-end tutorial runner with cleanup trap"
      min_lines: 80
  key_links:
    - from: "docs/tutorial.md"
      to: "docs/how-it-works/index.md"
      via: "What just happened pointer"
      pattern: "how-it-works"
    - from: "scripts/test_phase_30_tutorial.sh"
      to: "target/release/reposix-sim"
      via: "spawns simulator"
      pattern: "reposix-sim"
---

<objective>
Author a working 5-minute tutorial that lands reposix's "aha" moment in step 4, and ship a test script that runs the tutorial end-to-end against the simulator. After this plan lands:

- `docs/tutorial.md` is a complete narrative walkthrough. Every code snippet in the page is a real command; every expected output is a real output. The reader can copy-paste and reproduce.
- `scripts/test_phase_30_tutorial.sh` spawns `reposix-sim`, mounts via `reposix` CLI, executes each tutorial command programmatically, asserts exit codes, and cleans up on exit. Running this script proves the tutorial is accurate.
- The structure follows Cloudflare Workers pattern H: 4 numbered steps with brief prose framing. The "aha" (server-side version bump visible via `curl | jq`) lands in step 4 — NOT saved for a recap (per Stripe pattern G in RESEARCH.md).
- All prose respects P2: non-code text uses "the reposix folder" / "the tracker as a folder" / "connect the tracker" — not "FUSE mount" / "daemon" / "kernel".

Purpose: DOCS-06 — the 5-minute first-run tutorial is the ONLY required Tutorial-category (Diátaxis) artifact. Plan 30-09 will playwright-screenshot each step. This plan authors the page and the test runner.

Output: 1 rewritten markdown file, 1 new bash script.

**Locked decisions honored:**
- Runs against `reposix-sim` (the default testing backend per CLAUDE.md OP #1 — simulator-first).
- 4 steps: start simulator → connect as folder → edit ticket → `git push` + verify.
- "Aha" in step 4 (Stripe G pattern). "What just happened" recap is 3 sentences, not another aha.
- Use `printf > file` (NOT `sed -i`) for the edit — per PATTERNS.md §docs/tutorial.md note carved from demo.md lines 125-136 (`sed -i` creates temp files the FUSE fs rejects with EINVAL).
- Prereqs listed as a bullet list, not prose paragraph.
- Cleanup commands at the end: `fusermount3 -u ...` and `pkill reposix-sim` (from demo.sh step 9).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-RESEARCH.md
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md
@.planning/notes/phase-30-narrative-vignettes.md

@docs/demo.md
@docs/tutorial.md

@scripts/demo.sh
@scripts/demos/full.sh
@scripts/hooks/test-pre-push.sh

<interfaces>
Existing demo script `scripts/demos/full.sh` contains the canonical 9-step flow. The tutorial is NOT a copy of it — it is a 4-step distillation of the simulator-only subset (steps 3, 4, 6, 7 from demo.md per PATTERNS.md line-mapping).

CLI contracts the tutorial relies on:

- `target/release/reposix-sim --bind 127.0.0.1:7878 --seed-file <path>` spawns simulator; default seed creates 6 issues.
- `curl -sf http://127.0.0.1:7878/healthz` returns "ok" when ready.
- `target/release/reposix mount <dir> --backend http://127.0.0.1:7878/projects/demo` mounts.
- `target/release/reposix umount <dir>` unmounts.
- `curl -s http://127.0.0.1:7878/projects/demo/issues/1 | jq .version` returns integer (1 before push, 2 after).

These contracts are proven by `scripts/demos/full.sh` and are stable as of Phase 29.

Test harness pattern from `scripts/hooks/test-pre-push.sh`:
- `cleanup() { ... }; trap cleanup EXIT` — unmount + kill sim unconditionally
- `run_and_check <label> <expected-exit> <command>` — tabular test output
</interfaces>
</context>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Tutorial commands -> reader's dev host | Commands spawn a local HTTP server (127.0.0.1:7878). Reader runs in their own shell; no remote calls. |
| Test script -> CI runner | The script spawns reposix-sim + mount; runs on ubuntu-latest in CI. Must tear down cleanly or subsequent jobs share state. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-30-06-01 | Information Disclosure | Tutorial example commands leak real endpoints/tokens | mitigate | All commands target `http://127.0.0.1:7878` (localhost). No auth. No creds. |
| T-30-06-02 | Denial of Service | Test script leaves reposix-sim running after failure | mitigate | `trap cleanup EXIT` unconditionally kills sim + unmounts. Mirrors `scripts/hooks/test-pre-push.sh` pattern. |
| T-30-06-03 | Elevation of Privilege | `fusermount3 -u` requires user to have mount access | accept | Tutorial is Linux-only for v0.9 (stated in Prereqs); fuse3 package is the prereq, same as demo.md. |
</threat_model>

## Chapters

- **[Task 1 — Write docs/tutorial.md](./task-1-tutorial-md.md)** — 4-step runnable tutorial with aha in step 4; Vale linting guidance; acceptance criteria.
- **[Task 2 — Write scripts/test_phase_30_tutorial.sh](./task-2-test-script.md)** — End-to-end runner with cleanup trap; verification; success criteria; phase output.
