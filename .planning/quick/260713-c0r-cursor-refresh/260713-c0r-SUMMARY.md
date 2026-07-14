---
quick_id: 260713-c0r
title: "Refresh GSD cursor post-b773c04 closure + fold 4 v0.15.0 noticings"
status: complete
completed: 2026-07-13
---

# Quick Task 260713-c0r — SUMMARY (GSD cursor refresh + intake fold)

Planning-docs-only. Reconciled the GSD cursor to reality (v0.14.0 public, b773c04
RED-main CLOSED, fix-first done) and routed 4 b773c04-arc noticings into the v0.15.0
intake. NO code / crates / gate scripts touched.

## What changed

### A. `.planning/STATE.md` (3 precise frontmatter edits)

1. **`status:`** — `v0.13.1-shipped-v0.14.0-fix-first-items-4-8-pending`
   → `v0.14.0-SHIPPED-public-b773c04-red-main-CLOSED-post-tag-queue-items-0-5-in-progress`.
2. **`last_activity:`** — rewrote the stale "v0.14.0 D2+B3 CLOSED GREEN at e11ba96; tag
   work = OWNER fix-first items 4-8 …" string to: b773c04 RED-main blocker CLOSED (fix @
   `8e2aae5`, CI run 29302967371 SUCCESS all 15 jobs + quality-post-release run 29302973970
   SUCCESS); v0.14.0 fully SHIPPED + public (crates.io 0.14.0, GitHub release "Latest");
   fix-first arc DONE; post-tag queue items 0-5 resumed by successor #17 (item 0 = this refresh).
3. **`workstream_a`** — `blocks_tag: true` → `false`; `next_phase` comment rewritten from
   "awaiting owner pre-tag actions + L0 tag push" to "v0.13.0 SHIPPED — tagged/released
   2026-07-07 (`3423b18`, 'chore: release v0.13.0 (#68)'); v0.13.1 hotfix shipped 2026-07-08
   (`04640d5`). No tag pending". `status: closed-green` kept. workstream_b / workstream_c
   left untouched (their "after v0.13.0 tag" references are a separate queue item, per charter).

STATE.md **body prose left unchanged by design** — see Noticing below; the body's "TAG
BLOCKED" / B1-B5 tag-remediation cursor is now stale vs the updated frontmatter but is a
larger de-stale job than this cursor-refresh quick.

### B. `.planning/PROJECT.md` — NO CHANGE (already accurate on the scoped facts)

Grepped for `v0.14|pending|unreleased|cursor|fix-first|shipped|current milestone`. PROJECT.md
makes **no** "v0.14.0 pending/unreleased" claim and **no** fix-first cursor — its v0.14.0
mentions are all v0.13.0-era scope-deferral notes ("multi-SoT is v0.14.0" L85; "defer to
v0.14.0" L89/L120; research-bundle ref L112). Per the task guardrail ("if already accurate
on these facts, change nothing"), left as-is. Broader structural staleness flagged below.

### C. Fold 4 noticings into the v0.15.0 intake (ROUTED, not fixed) — tagged v0.15.0

**Into `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` (2 appended, existing q0e entry undisturbed):**
- **Sim-leak-on-SIGKILL (MEDIUM).** `container-rehearse.sh` backgrounds the sim + cleans via
  EXIT trap; on `subprocess.run(timeout=…)` SIGKILL the trap never fires → orphaned sim on
  7878. Sketch: internal `timeout` shorter than the row's `timeout_s` and/or own process group.
- **Post-release-CI binary provenance (MEDIUM/verify).** `quality-post-release.yml` has no
  obvious `cargo build -p reposix-cli`, yet container-rehearse needs host-mounted
  `target/debug/reposix`; run 29302973970 passed but binary provenance is unconfirmed.
  Sketch: trace provenance (~10 min); add an explicit build/artifact dep if it's an incidental cache hit.

**Into `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` (2 appended as GTH-V15-10/-11, new provenance section):**
- **GTH-V15-10 — harness rc(0) vs artifact exit_code(1) mismatch + sim-readiness race.**
  example-02 flaked once on back-to-back local runs; reconcile the two success signals +
  the sim-readiness race between rapid sequential runs.
- **GTH-V15-11 — `.sim-*.log` gitignore gap.** `.sim-*.log` under
  `quality/reports/verifications/docs-repro/` not gitignored (siblings `*.json` /
  `*.cobertura.xml` are). One-line `.gitignore` fix.

Provenance on all 4: b773c04 RED-main arc, SESSION-HANDOVER successor #16 noticings.

## Push coupling

This commit rides with SESSION-HANDOVER commit `ffb9d25` (previously local-unpushed) in
**one** `git push origin main` → **one** CI trigger. Post-push `HEAD...origin/main` must read `0 0`.

## Noticing (OD-3 ownership charter — REPORTED, not fixed)

- **STATE.md body prose is now stale vs the refreshed frontmatter.** § "Workstream C" still
  reads "TAG BLOCKED (2026-07-13)", "READY-TO-TAG: NO", and carries the B1-B5 tag-remediation
  cursor describing "awaiting owner decision" on mirror-refresh / p93 — all superseded by
  v0.14.0 shipping + b773c04 closing. Out of scope for this precise-frontmatter quick; a fuller
  STATE.md body de-stale is a candidate follow-up queue item.
- **PROJECT.md is structurally behind (footer "Last updated: 2026-05-01").** Its "Current
  Milestone" sections still name v0.13.0 / v0.13.2 as in-flight; it never mentions v0.14.0 or
  v0.15.0 as milestones. Not touched (would be a wholesale rewrite, against the minimal-edit
  charter, and not the specific v0.14.0-pending/cursor staleness this task scoped) — flagged for the owner to route.

## Self-Check: PASSED

- STATE.md 3 edits applied (status / last_activity / workstream_a blocks_tag+next_phase).
- v0.15.0 SURPRISES-INTAKE.md: 2 new entries appended, q0e entry intact.
- v0.15.0 GOOD-TO-HAVES.md: GTH-V15-10 / GTH-V15-11 appended, prior rows + back-pointer note intact.
- PROJECT.md unchanged (already accurate on scoped facts).
- Quick record created under `.planning/quick/260713-c0r-cursor-refresh/`.
