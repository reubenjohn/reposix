---
session: 7
written: 2026-04-14
recovery_command: "cat .planning/SESSION-7-BRIEF.md"
---

# Session 7 Mission Brief — reposix autonomous execution

> **After any compaction, run the recovery command above first. This file is the source of truth.**

## Current state

- Version: v0.5.0 (277/277 tests green, tagged, pushed)
- Branch: main
- Next phase: 16 (not started)
- Milestones queued: v0.6.0 (Phases 16–20) + v0.7.0 (Phases 21–25)

## Phase table

| # | Name | Milestone | CONTEXT.md |
|---|------|-----------|------------|
| 16 | Confluence write path (create/update/delete + ADF↔MD) | v0.6.0 | phases/16-confluence-write-path-*/CONTEXT.md |
| 17 | Swarm --mode confluence-direct | v0.6.0 | phases/17-swarm-confluence-direct-*/CONTEXT.md |
| 18 | OP-2 remainder: tree-recursive + mount-root _INDEX.md | v0.6.0 | phases/18-op-2-remainder-*/CONTEXT.md |
| 19 | OP-1 remainder: labels/ + spaces/ symlink overlays | v0.6.0 | phases/19-op-1-remainder-*/CONTEXT.md |
| 20 | OP-3: reposix refresh + git-diff cache | v0.6.0 | phases/20-op-3-*/CONTEXT.md |
| 21 | OP-7 hardening bundle (contention, truncation, chaos, macFUSE) | v0.7.0 | phases/21-op-7-hardening-*/CONTEXT.md |
| 22 | OP-8 honest-tokenizer benchmarks | v0.7.0 | phases/22-op-8-honest-*/CONTEXT.md |
| 23 | OP-9a Confluence comments as pages/id.comments/*.md | v0.7.0 | phases/23-op-9a-*/CONTEXT.md |
| 24 | OP-9b whiteboards, attachments, folders | v0.7.0 | phases/24-op-9b-*/CONTEXT.md |
| 25 | OP-11 docs reorg: InitialReport.md + AgenticEngineeringRef → docs/research/ | v0.7.0 | phases/25-op-11-*/CONTEXT.md |

## Hard stops (NEVER cross without explicit user check-in)

- **OP-10** (eject 3rd-party adapter crates): user-gated. Stop and ask.
- **Phase 12** (subprocess/JSON-RPC ABI): user-gated. Stop and ask.
- **Tag push**: ALWAYS run `bash scripts/green-gauntlet.sh --full` (6/6 gates) then ask user before running `bash scripts/tag-vX.Y.Z.sh`.
- **Live Confluence writes (Phase 16)**: all wiremock contract tests must pass before touching real tenant.

## Per-phase process (GSD cycle)

For each phase N:
1. `cat .planning/phases/<N>-*/CONTEXT.md` — read design brief
2. `/gsd-plan-phase N` — creates PLAN.md (no Rust yet)
3. `/gsd-execute-phase N` — runs PLAN.md wave by wave with atomic commits
4. `/gsd-verify-work` — goal-backward verification
5. `/gsd-code-review` — post-ship code review
6. Update `.planning/STATE.md` cursor
7. If end of milestone: promote CHANGELOG [Unreleased] → [vX.Y.Z], bump version, run green-gauntlet, ask user for tag gate

## Dark-factory rules (non-negotiable)

1. **Simulator is default.** Never hit real backend unless `.env` has credentials AND `REPOSIX_ALLOWED_ORIGINS` includes that origin.
2. **Tainted by default.** Every byte from any remote (sim counts) is `Tainted<T>`. Never route into side-effect actions.
3. **Audit log required.** Every network-touching action → SQLite audit row. Feature is not done without it.
4. **No hidden state.** All state in committed artifacts. No "works in my shell" bugs.
5. **Test floor never drops.** `cargo test --workspace` count must not decrease after any wave. Clippy `-D warnings` must stay clean.

## New-phase protocol (if a phase reveals follow-up work)

```bash
/gsd-add-phase <description>
cat > .planning/phases/<N>-<slug>/CONTEXT.md << 'EOF'
# Phase N CONTEXT — <name>
## Goal
<one paragraph — what and why>
## What triggered this
Phase <M> revealed...
EOF
git add .planning/
git commit -m "chore(planning): add Phase N stub — triggered by Phase M"
```
Then continue the current phase without stopping.

## Recovery after compaction

If you have lost track of what you were doing:
```bash
cat .planning/SESSION-7-BRIEF.md     # re-read this file
cat .planning/STATE.md               # find stopped_at cursor
git log --oneline -10                # see what has shipped
ls .planning/phases/<N>-*/           # check if PLAN.md or SUMMARY.md exists
```
Then resume from the STATE.md cursor.
