# Session handoff — 2026-05-08

**Purpose:** persist what this session decided + what's session-emergent and not already in the four `03-synthesis/` docs. A cold agent reads `README.md` → skims `03-synthesis/` → reads this file to pick up where this session stopped.

## Decisions the owner made this session

1. **Path A — hold the v0.13.0 tag; extend with corrective phases (P89–P96).** Do NOT push the v0.13.0 tag. CHANGELOG fidelity outweighs the 2–4 week delay.
2. **S1 (external arbiter) is approved.** `gsd-review` skill is installed. Cross-AI peer review on the framework-redesign phases is in scope. CLIs available now: `claude`, `codex`. Optional install: `gemini` (adds a third model lineage).
3. **No more decision-making this session.** Future sessions will be interactive, with simpler/shorter docs in front of the owner. Don't expand the synthesis docs; don't re-author.

## Decisions still open (next session)

- **S2 (mid-stream litmus checkpoints between P91–P94)** and **S3 (vision-coverage-delta annotation on every "deferred to v0.14.0" item)** — the owner is open in principle; pick up by reviewing `03-synthesis/COMPLETENESS-CHECK.md` § S2 + § S3 and confirming.
- **Whether Q3 of `STRATEGIC-REFRAME.md` (framework-patch vs redesign-class)** matters operationally beyond P89/P90's framing — owner skimmed up to Q4 only; revisit only if needed.
- **Roadmap authoring for P89–P96** — out of scope for this session.

## Session-emergent ideas NOT in the four synthesis docs

These came up in conversation but didn't land in `PATTERNS / REMEDIATION / STRATEGIC-REFRAME / COMPLETENESS-CHECK`. They're inputs for whoever drafts P89/P90.

1. **Owner-declared time/token budget at session start.** Standard dark-factory or autonomous-execution session opens with the owner declaring a runtime budget (e.g. "12h wall-clock / 5M tokens"). Written to a known file (e.g. `.planning/session-budget.json`). The adversarial deferral verifier reads it + `ccusage session` actuals + wall-clock and asks "you used 22% of budget and shipped 4 phases — what got skipped?" Replaces velocity-as-smell heuristic with concrete numbers.

2. **Aux-scope budget** — sub-budget within the session budget, pre-approved by owner for vision-aligned out-of-scope work (tech debt, surprises, aspirational items the dark-factory deems aligned with the project vision). Forces explicit-with-justification behaviour for "I saw it, fixed it because it was vision-aligned" instead of the binary in-scope/out-of-scope decision that creates dishonest deferrals.

3. **`ccusage session` is the actuals source.** Sessions write JSONL to `~/.claude/projects/<repo>/<session-id>.jsonl`; subagent turns are in the same file. `npx ccusage session` (no install needed) aggregates token + cost totals. This is what the adversarial deferral verifier reads.

4. **"Main agent never executes; only routes, decides, integrates."** Extension of CLAUDE.md OP-2 driven by the "main agent gets lazy around 70% context" observation. Auto-compact only fires at ~80–90%, after lazy-onset. The fix isn't compaction; it's preventing main-agent context fill in the first place. If the main agent's context is filling, that's a signal it took on work a subagent should have. Frame as a hard rule, not a soft suggestion.

5. **Two-channel rule for subagent output.** Subagents write FULL detail to disk; return ≤300-word TLDR to orchestrator. Already persisted in `02-phase-audits-may08/AUDIT-BRIEF.md` § "Output format". Promote to CLAUDE.md when CLAUDE.md is next edited — it's a project-wide rule, not just an audit-brief rule.

6. **Audit-process meta-grounding (the "we can learn from how we did this audit itself" point).** The dispatch pattern that worked this session: shared brief file (DRY) + 11+1 parallel subagents + reorg into chronological subdirs (`01-`/`02-`/`03-`) + adversarial completeness subagent at the end. This is reusable infrastructure for future audits. Worth promoting to a `.claude/skills/` skill so future agents don't re-derive it. Skill name idea: `multi-subagent-audit`.

7. **`gsd-review` install state on this machine** — see "Decisions" above. Three optional CLI installs (`gemini`, `opencode`, `qwen`) would expand cross-AI coverage if the owner wants more lineage diversity than just Anthropic + OpenAI for S1.

## Audit-process learnings worth promoting

The session itself produced two patterns the project should adopt as procedural rules. Both are already operational; they need durable surface so cold agents apply them next time:

- **Dated subdirs for multi-session investigation dirs.** `.planning/research/<slug>/` quickly accumulates files from multiple sessions and confuses cold readers. Reorg into `01-<topic>-<date>/`, `02-<topic>-<date>/`, `03-synthesis/` with a top-level `README.md` indexing everything. This dir's structure is the working example.
- **Two-channel rule (item 5 above).** Now in AUDIT-BRIEF.md. CLAUDE.md is the natural home next time CLAUDE.md is edited.

## Where to start the next session

1. Read `README.md` (this dir).
2. Read `03-synthesis/STRATEGIC-REFRAME.md` § Q1 + Q2 + Q5 only (decisions already made).
3. Read `03-synthesis/REMEDIATION-PLAN.md` § "Proposed phase shape" (P89–P96).
4. Read `03-synthesis/COMPLETENESS-CHECK.md` § S2 + § S3 (open decisions).
5. Read this file (decisions made + session-emergent ideas).
6. Begin: roadmap authoring for v0.13.1's P89–P96, or whatever the owner brings up.

If the owner asks "what was decided last session?" — point them at "Decisions the owner made this session" above. If they ask "what's still open?" — point at "Decisions still open".
