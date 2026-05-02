# Vision, Strategy, and Owner Decisions

← [back to index](./index.md)

## 2. Five-year vision (3-5 bullets)

Concrete future states. Picked for credibility, not aspiration:

1. **The dark-factory pattern is a recognised industry term, with reposix cited as the canonical reference implementation.** Simon Willison's `agentic-engineering-reference.md` distillation already names it. By 2031, "ran a thousand agents against a simulator overnight" should be a job description, and `reposix-sim` should be the example people teach from. Measurable proxy: ≥3 conference talks (workshop, programming-language venue, or industry track) cite reposix when discussing simulator-first agent infrastructure.

2. **Every coding agent in the top 10 by usage offers reposix as a default issue-tracker integration alongside MCP.** Not exclusive of MCP — alongside. The model is GitHub Copilot offering reposix as a one-click "treat your tracker as a git repo" option. Measurable proxy: Claude Code skill registry, Cursor recipe library, Aider extension docs, and Continue's tool catalog all ship a `reposix-init`-flavoured guide.

3. **There are >25 community `BackendConnector` crates covering tools beyond the original three (GitHub, Confluence, JIRA).** Not 50 — that's vanity. 25 is the credibility line; it means Linear, Notion, Asana, ClickUp, Phabricator, Redmine, Trac, Bugzilla, Pivotal, GitLab Issues, Bitbucket, ServiceNow, Zendesk Support, Salesforce Cases, FreshDesk, Plane, Trello, monday.com, plus a long tail. Measurable proxy: `reposix-connector` crates.io tag has ≥25 published crates by 2031.

4. **reposix's token-economy claim is a measured CS-publishable result.** Not a marketing slide — a paper at a programming languages or systems workshop with reproducible benchmarks comparing dark-factory POSIX-over-REST against MCP and raw SDK loops on a fixed task suite, cost per task in dollars, latency CDFs, and an honest discussion of where the abstraction breaks. Measurable proxy: at least one peer-reviewed citation by 2030.

5. **`reposix gc` and `reposix doctor` are the Rust-equivalents of `git fsck` for the agent era.** When something is broken in an agent's filesystem-based tracker integration, the first instinct should be to run `reposix doctor`. This requires the diagnostics layer to be cared-for, not bolted-on. Measurable proxy: ≥80% of GitHub issues filed against `reubenjohn/reposix` include `reposix doctor` output without prompting.

Rejected because not credible inside five years: a managed cloud service business; an enterprise-RBAC differentiator; a Windows/macOS native filesystem rewrite. See §4.

---

## 4. Cuts and trade-offs (be honest)

Things explicitly rejected and why:

1. **"reposix as a managed cloud service in 2026."** Premature. OSS adoption first. Without ≥1000 GitHub stars and ≥3 paying customer interviews, a managed offering is a distraction. Re-evaluate at v1.0 if the OSS metrics warrant it.

2. **"Reinvent git for partial-clone improvements."** Out of scope. Git already does what we need (>2.34); fighting upstream is a multi-year sink. If git's partial-clone GC is bad, write a `reposix gc` (§3j) that works *with* git, not against it.

3. **"Compete with GitHub / Atlassian on tracker functionality."** Orthogonal. reposix is a substrate; if a backend lacks a feature, that's the backend's choice. Do not build a reposix-native sprint planner.

4. **"Polyglot connectors via subprocess."** Tempting (Python connectors are easier to write) but a serious complication for the security model. Subprocess connectors blow up the egress allowlist enforcement story unless the subprocess inherits the same `reposix_core::http::client()` factory — which means *re-implementing it in Python*. Defer to v0.14.0 plugin-registry phase, decide then. Recommend: Rust-only for stability commitment; subprocess connectors only as a stretch goal once Rust-only ecosystem proves out.

5. **Windows / macOS native filesystem rewrite.** Pre-v0.9.0 pivot this was a real concern (FUSE-on-Windows is a different VFS). Post-pivot it's moot — git works on every platform. This stays cut forever; don't reopen.

6. **"Real-time collaboration" milestone.** Cut, correctly, in milestone-plan.md §"What was cut": git already does this. The branch-per-draft pattern from `initial-report.md` §"Confluence Hierarchies and Draft Lifecycles" is the answer.

7. **A "reposix-native" agent harness.** Tempting (control the full stack), but it would invert the value proposition: reposix's whole point is to be invisible to whatever harness the user picked. Build *recipes* (milestone-plan.md §v0.12.0) for existing harnesses, not a competing harness.

---

## 5. Recommended next-3-month plan

Sequenced by value × reachability against the v0.11.0 "Performance & Sales Assets" milestone in `.planning/PROJECT.md`. The owner already has v0.11.0 scaffolded around bench harness + doc-polish backlog + helper-multi-backend-dispatch fix + `IssueId→RecordId` rename.

**Month 1 (v0.11.0 ship — already in flight).**
- Land what's planned: bench harness (Phase 46), MCP-equivalent baseline (Phase 47), recorded asciinema + blog draft (Phase 48), coverage ratchet (Phase 49), 9 major + 17 minor doc-clarity findings, helper-multi-backend-dispatch carry-forward, `IssueId→RecordId` rename.
- **Add to v0.11.0:** §3c **token-cost ledger** (~2 days). It folds naturally into Phase 46/47's bench work — once we can count tokens for the bench, we can count them for every helper invocation. Promotes the `reposix-vs-mcp` table cells from "characterized" to "measured" without any docs work, which is OP-1 ground-truth gold.

**Month 2 (v0.11.5 or early v0.12.0).** Pick **two** innovations:
- §3a **`reposix doctor`**. Lowest-risk, highest leverage. Every confused-user bug report becomes a doctor invocation; every doctor finding teaches reposix something it didn't know. ~2 days. Lands as a new phase in v0.11.5 or first phase of v0.12.0.
- §3b **time-travel via git tags**. Highest originality, ~3 days, no breaking changes. Differentiates reposix from every MCP server out there. Lands as second phase of v0.12.0.

Combined: ~5 engineering days, two surprising features, both compose with everything that came before.

**Month 3 (v0.12.0 second half).**
- Begin agent-SDK integration recipes (milestone-plan.md §v0.12.0). Dogfood the doctor + time-travel work against the recipe CI jobs.
- Assess: if doctor is generating ≥1 fix-this-finding per week, it's earning its keep; if not, prune diagnostics. If time-travel-via-tags is in use by ≥1 external user, prioritise the `reposix log --time-travel` UI; if not, leave it as a power-user feature.

**Why these two innovations specifically for month 2:** Doctor de-risks adoption (§2 future state #5). Time-travel-via-tags creates a pithy talk-track (§2 future state #1, #4) that no competitor has. Together they're cheap, original, and synergistic with the v0.11.0 work already in flight. Other innovations (multi-project helper, OTel, conflict UI, plugin registry) are correct candidates for v0.13.0+ and don't need to be pulled forward.

---

## 6. Originality audit

Honest call on each §3 idea. Categories: **Novel (original to reposix or near-original)**, **Hybrid (well-known idea, novel application)**, **Well-known (table stakes)**.

| Innovation | Category | Why |
|---|---|---|
| §3a `reposix doctor` | Hybrid | Doctor commands are common (`brew doctor`, `flutter doctor`, `docker doctor`). Novel application: agent-era diagnostics where the *fixes* are what matter, not the findings, because an autonomous agent reads the fix string and runs it. |
| §3b time-travel via git tags | **Novel (highest)** | I cannot find prior art for "tag every external sync as a git ref." git-bug uses Lamport timestamps; jirafs has snapshots. Neither exposes per-sync points as first-class git refs an agent can `checkout`. |
| §3c token-cost ledger | Hybrid | Cost telemetry is well-known (LangSmith, Helicone). Novel application: built-in to a git remote helper, persisted in the same SQLite as the audit log, queryable by agents themselves. |
| §3d multi-project helper process | Well-known | Daemon-with-shared-state is standard server design. Listed because the *application* (one git remote helper serving N partial clones) is what milestone-plan.md already commits to; documenting the rationale matters. |
| §3e OpenTelemetry tracing | Hybrid | OTel itself is table-stakes. Novel application: spans carry `taint.origin` + `agent.harness_hint` attributes that make dark-factory queries first-class — no other tracing setup formalises that. |
| §3f conflict resolution UI | Well-known | 3-way merge UIs are old. Field-level diffing of YAML frontmatter is mildly novel but not breakthrough. |
| §3g plugin registry | Well-known | Cargo, npm, pip have registries; this is the same idea. Novelty is the BackendConnector subprocess protocol if we go polyglot. |
| §3h swarm replay | Hybrid | Chaos engineering and load replay are well-known. Novel application: agent-shaped operations as the unit of replay, with deterministic timing for bug reproduction. |
| §3i capability negotiation | Hybrid | API capabilities are well-known. Novel application: injecting valid transitions as YAML comments in the frontmatter (this is from `initial-report.md` §"Validating Workflow Transitions" — credit there). |
| §3j `reposix gc / archive` | Well-known | Cache eviction with LRU/TTL is standard. Listed because users will demand it. |
| §3k streaming push | Well-known | Streaming parsers are standard performance engineering. |
| §3l research paper | **Novel (process)** | The *paper* isn't novel; the *act of formalising the dark-factory pattern as a publishable claim with measurable comparisons* is novel for this project category. Most OSS agent-infra ships marketing slides; almost none submit to peer review. |

**Tally:** 2 Novel · 5 Hybrid · 5 Well-known. The novel ones (§3b, §3l) are the ones to lead with publicly. The hybrids are the ones that compound. The well-knowns are operational must-do — table stakes for the §2 future state where reposix is a credible production substrate.

**Self-check:** is this audit honest? §3b is the call I'd most expect to be wrong about — there may be prior art I haven't found. Recommend a 1-hour literature search before committing publicly to "first to do this." Search terms: "git remote tag every fetch", "promisor remote temporal snapshot", "external-sync git ref."

---

## 7. Open questions for the owner

Five decisions only the owner can make. Pick before v0.12.0 planning starts.

1. **Research paper, yes or no?** §3l. If yes, allocate 4 weeks of writing + figure work after v0.11.0 numbers stabilise. Co-author Simon Willison? If no, deprecate this from §2 future state #4 and rely on conference talks instead.

2. **Plugin registry: Rust-only or polyglot?** §3g + §4 cut #4. Rust-only is safer for the security model (egress allowlist enforcement), polyglot is friendlier for ecosystem growth. Recommendation: Rust-only for v0.14.0; revisit polyglot at v1.0 if ≥3 community members ask for Python connectors. Owner decides timing.

3. **Managed-service future state on the table?** §4 cut #1. Recommendation: not in 2026, re-evaluate at v1.0 with concrete OSS metrics. Owner confirms the cut.

4. **Public roadmap: who owns it once contributors arrive?** Today the owner is `reubenjohn`; `.planning/` is single-author. At ≥10 community contributors, this stops working. Recommendation: split `.planning/` (private, owner-only) from `ROADMAP.md` (public, milestone-level). Owner decides when to flip.

5. **Branding vs MCP — sit alongside, replace, or position above?** v0.10.0 docs commit to "complement, not replace" (banned word: "replace"). The §2 future state #2 ("alongside MCP") is consistent with that. But §3l's research paper would imply *above* MCP for the 80% of operations. Pick one frame and hold it across docs, social, and the paper if it happens. Recommendation: "complement for the 80%, replace nothing." Owner ratifies.

---

## 8. Owner decisions (2026-04-25)

Resolutions on the §7 open questions. These ratify scope and unblock v0.11.0 planning.

1. **Research paper — out of scope for now.** §3l deferred indefinitely; downgrade §2 future-state #4 ("CS-publishable result") to a stretch goal. Conference-talk track stays viable.
2. **Plugin registry — Rust-only for now.** §3g + §4 cut #4 ratified. Subprocess/polyglot connectors stay parked; revisit at v1.0 only if a real ecosystem signal emerges. The egress-allowlist enforcement story remains intact because every connector links the same `reposix_core::http::client()` factory.
3. **Managed service — deferred (may revisit in 2026).** §4 cut #1 stays the default posture. Owner reserves the right to revisit later in 2026 if OSS adoption metrics warrant; until then no architectural decisions assume a hosted offering.
4. **`.planning/` split at >10 contributors — agreed.** Trigger condition stays "≥10 active community contributors." Action when triggered: split `.planning/` (private, owner-only state) from a public `ROADMAP.md` (milestone-level commitments). No work needed today.
5. **Branding posture — "complement for the 80%, replace nothing."** Ratified. This is the canonical phrasing across docs, social, and any future paper. The banned-words linter (`scripts/banned-words-lint.sh` + `.banned-words.toml`) keeps this honest at the doc layer.

These decisions are roadmap inputs, not code changes. Acted on in v0.11.0 planning by:
- Removing §3l from any active phase scope (was already deferred to v0.15.0+).
- Promoting the §3c token-cost ledger and §3a `reposix doctor` to v0.11.0 candidates per §5.
- Holding §3g plugin-registry / subprocess polyglot work parked behind v0.14.0+ gating.
- Adding "complement for the 80%, replace nothing." to the banned-words config as the canonical positioning anchor.

---

*End of brainstorm. ~590 lines + owner decisions. Not roadmap-mutating; the v0.11.0 ROADMAP entry is the binding artifact.*
