# v0.13.0 Audit Brief — Plan vs. Reality Misalignment

**Read this first.** Shared briefing for all v0.13.0 phase-audit subagents and the bird's-eye vision-audit subagent. Findings get written under the per-agent paths in this same directory.

## Why this audit exists

A 4-subagent dark-factory exercise (2026-05-02; see `SUMMARY.md` + `T1-T4-*.md` in this directory) found 37 frictions (16 HIGH) on the v0.13.0 milestone *after* it was graded GREEN by the verifier subagent. Root cause: the quality framework structurally exempts real-backend end-to-end flows. We've confirmed CLUSTERS A–H from the dark-factory tests. We now suspect there are MORE failures the framework also missed.

**Your job is to find the failures we don't yet know about.**

## Project context (60 seconds)

**reposix** exposes REST issue trackers (Confluence, GitHub Issues, JIRA) as a git-native partial clone. Agents `git clone` and use `cat`, `grep`, `sed`, `git push` — zero custom CLI awareness. The runtime is three pieces: `reposix-cache` (bare git repo built from REST responses), `git-remote-reposix` (remote helper, push/fetch tunnel), and `reposix init`/`reposix attach` (bootstrap commands).

**v0.13.0 milestone vision: DVCS over REST.** Two-sentence summary:
> Confluence (or any one issues backend) remains the source of truth, but a plain-git mirror on GitHub becomes the universal-read surface; devs `git clone` vanilla, install reposix only to write back, and `git push` via a bus remote that fans out SoT-first then mirror-best-effort.

Three roles the milestone enables:
1. **SoT-holder** — reposix-equipped, writes via bus remote (`reposix::<sot>?mirror=<gh-url>`).
2. **Mirror-only consumer** — vanilla git, read-only.
3. **Round-tripper** — vanilla checkout, then `reposix attach <backend>::<project>`, then writes via bus.

**The pivotal claim that broke:** "Pure git after init/attach. No reposix CLI awareness needed beyond bootstrap." The dark-factory exercise found this is half-true on sim and structurally broken on real Confluence.

## What v0.13.0 promised — phase decomposition

```
P78 — Pre-DVCS hygiene (gix bump + WAIVED-row verifiers + walker schema migration)
P79 — POC + reposix attach core
P80 — Mirror-lag refs (refs/mirrors/<sot>-{head,synced-at})
P81 — L1 perf migration (list_changed_since-based conflict detection + sync --reconcile)
P82 — Bus URL parser + cheap prechecks + fetch dispatch
P83 — Bus write fan-out (SoT-first, mirror-best-effort, fault injection)
P84 — Webhook-driven mirror sync (GH Action workflow)
P85 — DVCS docs (topology + mirror setup + troubleshooting)
P86 — Dark-factory regression — DVCS third arm
P87 — Surprises absorption (+2 reservation slot 1)
P88 — Good-to-haves polish + milestone close
```

Per-phase artifacts you'll find:
- `.planning/phases/<n>-<slug>/` — PLAN files, RESEARCH, plan-check (PLAN.md is often split into `<n>-PLAN-OVERVIEW.md` + chapter files).
- `.planning/milestones/v0.13.0-phases/ROADMAP.md` — phase entry with goal + REQs + SC.
- `quality/reports/verdicts/p<n>/VERDICT.md` — what the verifier subagent said.
- `quality/reports/verifications/<phase or row>/` — per-row evidence files.
- Commit history: `git log --grep "P<n>\|p<n>"` or by date range.

## Failure shapes we already know about (look for MORE of these)

1. **Test name promises one thing, assertions deliver less.** Example: `dark_factory_real_confluence` is named for real-backend coverage. Its assertions stop at "URL has the right shape" — never runs `git fetch`/`git push`. That's a broken existing gate, not a missing one.
2. **"Substrate gap" / "deferred" deferrals masquerading as GREEN.** Example: P86 verdict GREEN despite the TokenWorld arm being explicitly deferred under "substrate gap" framing in catalog comments — public docs ("pure git after init") never qualified.
3. **Plan promises N items, ship delivers N-K, K silently dropped.** Example: P79-03 (real-backend `attach` for confluence/github/jira) appears not to have landed; production error message still leaks "P79-02 scaffold" / "P79-03" phase IDs.
4. **Project's own non-negotiable invariants violated silently.** Example: OP-3 audit log mandatory; every `git push` in the dark-factory tests writes ZERO `helper_push_*` rows because cache.db is never created on the helper-push path.
5. **Documented user-facing flow rejected by the implementation.** Example: bus push helper rejects `.github/workflows/*.yml` as "invalid records (no frontmatter)" — but the documented mirror-setup guide tells users to commit exactly that file.
6. **Velocity-as-skip-signal.** Phases that shipped faster than scoped → suspect scope was cut without intake-file logging.

## What to look for (per-phase auditor)

For your assigned phase, READ:
- The phase ROADMAP entry (in `.planning/milestones/v0.13.0-phases/ROADMAP.md`).
- The PLAN files (`.planning/phases/<n>-<slug>/<n>-PLAN-OVERVIEW.md` + any chapter files).
- The RESEARCH file if it exists.
- The VERDICT (`quality/reports/verdicts/p<n>/VERDICT.md`).
- The catalog rows the phase introduced (look in `quality/catalogs/*.json`; grep for the phase number or REQ-ID).
- The actual code/docs that shipped (use `git log --oneline` on the phase's commits, `git show` to see diffs, or read the files directly).

ASK these questions:
1. **Plan vs. ship gap.** Does each PLAN promise (success criterion, REQ, deliverable) have evidence in the shipped artifacts? List anything that was promised but isn't visible in code/docs/catalog.
2. **Catalog row honesty.** Do the catalog rows the phase introduced actually exercise the load-bearing claim? Do their assertions match the row's `description` / `claim`? Find any "URL-shape only" / "exists-only" / "structure-only" assertions where the description implies functional verification.
3. **Deferrals.** Does the phase carry deferrals? Where are they recorded? Is the deferral reasoning load-bearing for shipping ("we couldn't do X and X is required to claim Y")?
4. **Velocity smell.** Did the phase ship significantly faster than the plan suggested? If yes, what got cut?
5. **Real-backend vs. simulator coverage.** Does the phase's transport-layer / push-path / fetch-path claim depend on real-backend behavior? If yes, does any verifier exercise the claim against a real backend (look for `agent_flow_real.rs`, `--ignored` tests with real-backend env vars, or any `cadence: pre-release` row)?
6. **Doc claim vs. code reality.** If the phase shipped user-facing docs, walk one critical example end-to-end mentally: would the documented commands actually work? Note any contradictions between user-facing docs and the implementation.
7. **CLAUDE.md update.** Did the phase update CLAUDE.md (mandatory per QG-07) in a way that reflects the actual shipped state? Or does CLAUDE.md describe an aspirational state?

## What to look for (bird's-eye vision auditor)

You are NOT auditing individual phases. You are checking whether v0.13.0, *as a whole*, delivers the milestone vision.

READ:
- `.planning/milestones/v0.13.0-phases/ROADMAP.md` (full file)
- `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md` (the vision statement)
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` (the design contract)
- `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` (what was graded)
- `CHANGELOG.md` v0.13.0 entry
- `.planning/RETROSPECTIVE.md` v0.13.0 section
- The dark-factory findings in this directory's `SUMMARY.md` + T1–T4 files

ASK:
1. **Three-roles delivery.** Does v0.13.0 deliver each of the three roles end-to-end on a real backend? What's the gap between "graded GREEN" and "a vanilla cold reader could execute the documented flow"?
2. **Vision-to-ship leakage.** Are there parts of the vision that show up nowhere in the shipped milestone? Subtle stuff (e.g., the "agent has zero reposix awareness" claim — was it ever tested as a hypothesis?).
3. **Cross-phase coherence.** Do P78 → P88 stack into the vision, or do they read like 11 disconnected tasks that each pass their own verifier?
4. **Verdict honesty.** Read `milestone-v0.13.0/VERDICT.md`. Did the milestone-close verifier interrogate the vision, or did it check "are all 11 phase verdicts GREEN"? What's the strongest gap between the verdict's reasoning and reality?
5. **Process gaps you'd surface to a CTO.** If you had 5 minutes to brief an engineering exec on what the milestone *actually* shipped vs. what the verdict claims, what's the headline?

## Output format

> **Two-channel rule (load-bearing — read carefully).** Every audit subagent writes to TWO channels:
> 1. **The file** (full detail, one finding per F-entry, every claim cited with path + line range or SHA). Targets: per-phase auditors → `phase-audit-p<n>.md`; vision auditor → `vision-audit.md`. Both inside `02-phase-audits-may08/` (or the equivalent dated subdir for future audits).
> 2. **The return-message TLDR to the orchestrator** (≤ 300 words). Headline verdict + counts + 3–5 most load-bearing findings with one-line evidence each. Do NOT dump the full file body into the return message — it overflows the orchestrator's context. The orchestrator can `Read` the file when it needs detail.
>
> This rule exists because the May 8 audit's first run had subagents echo full findings back to the orchestrator, which filled context that should have stayed on disk. Future audits MUST follow this contract.

Use this structure:

```markdown
# Phase P<n> Audit — <slug>
**Auditor:** unbiased subagent (zero session context)
**Date:** 2026-05-08

## Verdict at a glance
- ALIGNED items: <count>
- MISALIGNED items: <count>
- SUSPECT items: <count>

## Findings

### F1 — <one-line title> [SEVERITY: HIGH | MED | LOW]
**Claim in plan:** <quote or paraphrase>
**Reality:** <what shipped>
**Evidence:** <file path, line range, or commit SHA>
**Why it matters:** <one sentence>

### F2 — ...
```

Severity guidance:
- **HIGH** — load-bearing claim is unsupported / a project invariant is violated / a documented user flow can't execute.
- **MED** — non-load-bearing gap or a quality-framework integrity issue.
- **LOW** — cosmetic, doc-drift, or future-work pointer.

## Hard rules

1. Do NOT modify any source file. Audit only.
2. If a probe is inconclusive (e.g. you'd need to run cargo to confirm), mark the finding **SUSPECT** and document what would settle it.
3. Cite a file path + line range or a commit SHA for every finding. No vibes.
4. Keep findings atomic — one issue per F-entry. Don't bundle.
5. Aim for completeness over brevity; this is the input to the v0.13.1 framework-fix phase.
