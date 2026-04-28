# 03 — Execution modes: top-level vs executor

## The depth-2 constraint

Claude Code limits subagent spawning depth: a subagent at depth 1 cannot spawn its own subagents. Combined with the fact that `gsd-executor`'s tool list does not include `Task`, this means **execution-style phases (under `/gsd-execute-phase`) cannot fan out to multiple subagents in parallel**.

This is not a bug to work around. It's a structural cue: some work is implementation-shaped (write code, run tests, commit), and some work is orchestration-shaped (fan out, gather, interpret, resolve). They want different containers.

## The two phase categories

We add a new ROADMAP marker per phase entry:

```
Execution mode: executor   # default — runs under /gsd-execute-phase
Execution mode: top-level  # runs from a top-level Claude Code session via slash command
```

### Execution mode: executor (default)

Implementation-shaped: write code, write tests, run cargo, commit. The orchestrator delegates to `gsd-executor` (depth 1, no `Task`). Internal parallelism is via wave-based plan splitting, not subagent fan-out. Examples in this milestone:

- **P64** — build the `crates/reposix-quality/` crate, the skill files, the hash binary, hook wiring, tests. All code-and-commit work. `gsd-executor` is the right container.
- All of P56–P63 ran this way.

### Execution mode: top-level

Orchestration-shaped: dispatch many subagents, aggregate results, resolve conflicts that need semantic judgment. The "executor" IS the top-level coordinator session because that session has `Task` and lives at depth 0. Examples:

- **P65** — backfill the doc-alignment catalog. The orchestrator runs `plan-backfill`, dispatches ~30 shard subagents in waves of 8, runs `merge-shards`, handles conflicts by editing shard JSON, re-runs. This cannot live inside `gsd-executor` — depth 2 isn't reachable, and `Task` isn't available.
- Future: any retroactive audit that fans out across docs / files / claims.

## Why not work around the constraint

Three workarounds were considered and rejected:

- **`claude -p` subprocess from inside `gsd-executor`.** Subscriptions don't have API keys; many users (including this owner) cannot use Path B. Rejected as primary surface; only acceptable as documented Path-B fallback for non-Claude-Code consumers.
- **One "manager" subagent that processes all 30 shards serially.** Violates the ≤3-files-per-agent rule we adopted to prevent recall degradation. Inconsistent with the design.
- **Cron / scheduled fan-out.** Each cron firing is a fresh top-level session with `Task`. Works at scale but adds machinery (state tracking which shards are done, recovery from partial runs) for no UX win at this scale. Reserved as the v0.13+ escape hatch if backfill grows past ~200 shards.

The straightforward answer wins: dispatch from a top-level session, accept the ~120-line orchestrator-context cost for a one-time backfill, mark P65 with `Execution mode: top-level`.

## What the autonomous orchestrator does for each mode

`/gsd-autonomous` reads phase entries and routes by mode:

```
For each phase in ROADMAP order:
  if phase.execution_mode == "executor":
    spawn gsd-discuss → gsd-planner → gsd-executor → verifier
    (the existing autonomous loop)

  if phase.execution_mode == "top-level":
    DO NOT delegate to gsd-executor.
    Read the phase's research brief (e.g. .planning/research/v0.12.0-docs-alignment-design/06-p65-backfill-brief.md).
    Execute the brief's protocol IN THE TOP-LEVEL SESSION:
      - run preparatory CLI commands (plan-backfill, etc.)
      - dispatch subagents via Task in waves
      - aggregate results
      - run finalizing CLI commands (merge-shards, etc.)
      - resolve conflicts via direct edits if merge-shards fails
    Then run the phase-close verifier subagent (same as executor mode).
    Commit the phase using the standard atomic-commit pattern.
```

If the autonomous orchestrator does NOT recognize the `Execution mode: top-level` marker, it MUST fail loud and refuse to proceed (do not silently fall back to executor delegation). The brief at `06-p65-backfill-brief.md` is normative.

## Routing for ad-hoc events

This generalizes beyond P65. After v0.12.0 ships:

- **Pre-push BLOCK with `STALE_DOCS_DRIFT`** mid-feature work → user invokes `/reposix-quality-refresh <doc>` from a fresh top-level session. Cannot run from inside an active `/gsd-execute-phase`.
- **Phase planner discovers the phase touches docs that affect existing rows** → planner emits a sub-task "after committing, user runs `/reposix-quality-refresh` from top-level."
- **Catastrophic catalog drift** (e.g. someone manually edited `doc-alignment.json`) → owner runs `/reposix-quality-backfill` from top-level to re-establish ground truth.

## CLAUDE.md addition (P64 deliverable)

A short note under the "Subagent delegation rules" section:

> **Orchestration-shaped phases run at top-level, not under `/gsd-execute-phase`.** When a phase's work shape is "fan out + gather + interpret + resolve," the top-level coordinator IS the executor. Mark such phases `Execution mode: top-level` in ROADMAP and provide a research brief the orchestrator follows verbatim. Do not attempt to dispatch subagents from inside `gsd-executor` (depth-2 unreachable, no `Task` tool).
