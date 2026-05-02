← [back to index](./index.md)

# POC findings → planner re-engagement protocol

The POC's whole purpose is to surface decisions the architecture sketch
didn't anticipate. The orchestrator MUST treat `POC-FINDINGS.md` as a
potential plan-revision input:

1. **At 79-01 close** (after POC commits land + push), orchestrator reads
   `research/v0.13.0-dvcs/poc/POC-FINDINGS.md` from origin/main.
2. **Decision gate** — orchestrator routes to one of:
   - **a) No revision needed.** Findings are informational only (e.g.,
     "frontmatter parse via `serde_yaml::from_str` is straightforward;
     no algorithm change"). Proceed directly to 79-02 → 79-03 execution
     as-drafted.
   - **b) In-place revision.** Findings warrant a tweak to 79-02 or 79-03
     (e.g., "reconciliation needs a `--orphan-policy=defer` option not yet
     enumerated"). Orchestrator dispatches a planner subagent with
     `<revision_context>` containing the POC findings + the existing
     plan(s). Planner produces a revised PLAN.md (NEW commit; no
     `--amend`; commit message cites the finding source).
   - **c) Phase split.** Findings reveal scope > combined 79-02 + 79-03
     budget (e.g., "reconciliation cases need >5 distinct rules" — exact
     early-signal trigger from `vision-and-mental-model.md` § "Risks").
     Orchestrator surfaces split options to owner: continue P79 with
     reduced scope + defer extras to P79.5 or P88, OR split inline. Owner
     approves a path.
3. **Artifact contract** — `POC-FINDINGS.md` MUST contain a top-level
   subsection `## Implications for 79-02` (the section name retained from
   the original draft for continuity; treat as "Implications for 79-02
   AND/OR 79-03") listing 0-N items, each tagged `INFO | REVISE | SPLIT`.
   Orchestrator routes by the highest-severity tag present.

This checkpoint is BLOCKING — 79-02 execution does not begin until the
checkpoint is resolved. The check costs ~5 minutes of orchestrator
read-then-decide; saves the >5h cost of executing a stale plan.
