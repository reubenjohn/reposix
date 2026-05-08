# 06 — Decisions log

> **Purpose.** Every load-bearing design decision with options-considered, choice, rationale, and origin (chat turn or owner direction).
> **Read first.** [`02-architecture.md`](./02-architecture.md).
> **Read next.** [`08-open-questions.md`](./08-open-questions.md) for what's still TBD.

Format per ADR:
- **Decision.** One-sentence statement.
- **Options considered.** What we discussed.
- **Chosen.** Which one.
- **Rationale.** Why.
- **Origin.** Owner direction / chat turn / inferred.
- **Trade-off accepted.** What we give up.

---

## ADR-1 — Edges are the grading unit, not nodes

**Decision.** The atomic unit graded by this gate is an `(source, target)` edge, not a single file or directory.

**Options considered.**
- (a) File-level: "is `docs/README.md` adequate?"
- (b) Directory-level: "is the `docs/concepts/` folder well-introduced by its parent?"
- (c) Edge-level: "does the link from A to B match what B teaches?"

**Chosen.** (c) edge-level.

**Rationale.** Fidelity is a relationship between two artifacts. Asking "is this README good?" is too vague to grade reliably; asking "does this README's link to dvcs-topology.md match what dvcs-topology.md teaches?" is concrete. The unit is the assertion, not the asserter.

**Origin.** Chat turn — owner asked about cross-links: "Because the judgement of whether it needs to be updated and updating it is very similar to that of ancestor README files right?" — confirming edges are the unit, not files.

**Trade-off accepted.** A single file edit can stale many edges; the walker has to be efficient at re-classification. We accept the engineering cost.

---

## ADR-2 — Four-level scrutiny ladder

**Decision.** Edges have a `max_level` of L0 (link resolves), L1 (anchor resolves), L2 (hash-drift detection), or L3 (LLM-judged fidelity).

**Options considered.**
- (a) Binary: "this edge is fidelity-graded or not."
- (b) Three levels (mechanical / hash-drift / agentic).
- (c) Four levels: split mechanical into link-resolves + anchor-resolves.
- (d) More levels (5+): adding things like "structural similarity," "semantic similarity," etc.

**Chosen.** (c) four levels.

**Rationale.** Anchor-resolves is meaningfully separate from link-resolves: anchor failures are silent (mkdocs renders, link works, target section just doesn't exist). Splitting them lets adopters configure "I just want broken links caught" (L0 only) vs "I want anchor failures too" (L1) cheaply. More levels (5+) add complexity without clear benefit.

**Origin.** Chat turn — proposed in conversation as "tiered scrutiny ladder."

**Trade-off accepted.** Some adopters will find 4 levels excessive; mitigated by the `default` scope letting them set one and forget.

---

## ADR-3 — Scope-only configuration; no global config

**Decision.** All configurable knobs live inside named scopes. There's no global config block.

**Options considered.**
- (a) Global defaults + scope overrides + edge-level overrides.
- (b) Per-directory `.cross-link-fidelity` files merged on lookup.
- (c) Scope-only with a special `default` scope.

**Chosen.** (c).

**Rationale.** Owner-directed: "Lets make all configs non-global and scope specific and we just have a 'default' scope which includes all sub-files for convenience and reduced config boilerplate." Eliminates merge complexity (no directory walk; no global vs scope priority). Easier to extract as a portable tool.

**Origin.** Owner direction in this conversation.

**Trade-off accepted.** The `[project]` block exists for things like `walk_paths` and `max_l3_per_push` that genuinely don't fit a scope mental model. Pragmatic exception, narrow.

---

## ADR-4 — Last-match-wins scope ordering (gitignore semantics)

**Decision.** When an edge matches multiple scopes, the scope appearing latest in the config wins.

**Options considered.**
- (a) First-match-wins (most specific first).
- (b) Most-specific-wins (compute specificity heuristically).
- (c) Last-match-wins (gitignore semantics).

**Chosen.** (c).

**Rationale.** Users already know gitignore semantics. "Most-specific-wins" sounds clean but specificity-computation is contentious (length of glob? number of literal segments? presence of `**`?) and produces surprising orderings. Last-match-wins is deterministic and explicit.

**Origin.** Chat turn — proposed as part of the scope config sketch.

**Trade-off accepted.** Re-ordering a config file changes behavior. Acceptable; it's the cost of explicitness.

---

## ADR-5 — Tracker (machine state) and config (human policy) are separate files

**Decision.** State lives in `quality/catalogs/cross-link-fidelity.json` (machine-managed). Policy lives in `.cross-link-fidelity` (human-authored).

**Options considered.**
- (a) Single config file containing both policy and state.
- (b) Two files: config (human) + tracker (machine).
- (c) State in a hidden/non-tracked file (`.cross-link-fidelity-state`).

**Chosen.** (b).

**Rationale.** Owner-directed: "I think if we separate tracker files (machine managed and not meant to be human readible but CLI offers insights) from the config file, it would be better." Tracker is git-tracked (PR review wants the diff), but never hand-edited. Same separation as `Cargo.toml` (config) and `Cargo.lock` (tracker).

**Origin.** Owner direction.

**Trade-off accepted.** Tracker diffs can be noisy in PRs. Mitigated by `cross-link report-pr` summarizing the diff into a comment.

---

## ADR-6 — Three-flavor grading context (target ⊕ edge ⊕ source), namespaced

**Decision.** L3 grading context can be set on the target (frontmatter), the edge (config override), or the source (scope-default). Merged at grading time. All under the `cross_link_fidelity:` namespace.

**Options considered.**
- (a) Target-only (frontmatter on target doc).
- (b) Source-only (scope config).
- (c) Edge-only (per-edge in tracker).
- (d) All three with explicit merge order.

**Chosen.** (d) — three flavors, namespaced.

**Rationale.** Each flavor solves a different need. Target context "what reader/scope I serve" travels with the doc. Edge context handles unusual lensings ("from anchor README, give just enough to skip the full read"). Source context covers index pages with consistent linking style. Owner-flagged the namespace concern: bare `grading_context:` would collide with other tools' frontmatter conventions.

**Origin.** Chat turn — "For judge context, should it be owned by the source, target or link?" → my answer "all three, target-primary" → owner agreed and asked for namespace fix.

**Trade-off accepted.** Three levels of context = more places to look when debugging a judgement. Mitigated by `cross-link show <edge-id>` which renders the merged context.

---

## ADR-7 — An edge is an explicit markdown link in a markdown file

**Decision.** v1 edge discovery is limited to `[text](path#anchor?)` syntax in `.md` files. Excludes code comments, mkdocs nav, frontmatter `see_also`, HTML `<a>` tags.

**Options considered.**
- (a) Markdown links only.
- (b) Add HTML anchor tags in markdown.
- (c) Add references in code comments.
- (d) Add mkdocs nav entries.

**Chosen.** (a) — narrow at v1.

**Rationale.** Owner-directed: "What counts as an edge makes sense but the target can be any file not just md files. Maybe as a starting point, to simplify our design lets assume non-leaf files have to be md files." Narrow scope keeps the parser tractable. Broaden later.

**Origin.** Owner direction.

**Trade-off accepted.** mkdocs nav reachability isn't covered by this gate; stays in `structure/cross-link-audit.py`. HTML `<a>` tags in markdown rare in this project; revisit if it shows up.

---

## ADR-8 — Project default `max_level` is L3 (dark-factory ethos)

**Decision.** This project's `default` scope ships with `max_level: L3`. Other adopters can choose differently in their config.

**Options considered.**
- (a) Default L1 (mechanical-only, opt-in to L3).
- (b) Default L3 only on anchor-readme scopes.
- (c) Default L3 universally.

**Chosen.** (c) for this project; (a) for the standalone tool's shipped default.

**Rationale.** Owner-directed: "Because this is a dark factory project, I think edges should be L3 by default." For *this* project, drift in fidelity is a security-of-grounding concern, not a polish concern. For the *standalone tool*, default L1 is friendlier to new adopters.

**Origin.** Owner direction.

**Trade-off accepted.** Higher Sonnet cost for this project. Mitigated by edge-level downgrades (nav-only scopes → L1) and the cap.

---

## ADR-9 — Per-push L3 cap with explicit-out-of-band-refresh BLOCK on overage

**Decision.** Pre-push hook caps L3 calls at `max_l3_per_push` (default 10). Overage BLOCKs the push with a directive to run `cross-link plan-refresh` out-of-band.

**Options considered.**
- (a) No cap (let it run).
- (b) Auto-degrade on overage (run as many as possible, defer the rest).
- (c) Cap + BLOCK.
- (d) Cap + warn.

**Chosen.** (c).

**Rationale.** Auto-degrade is the dark-factory anti-pattern: silent skip is silent coverage erosion. Cap-and-block forces explicit human/agent action; the action is cheap (one named verb).

**Origin.** Inferred from owner emphasis on dark-factory ethos.

**Trade-off accepted.** Refactoring a concept doc requires an extra step (out-of-band refresh) before the push lands. Acceptable; refactors are infrequent.

---

## ADR-10 — Bootstrap is a separate CLI verb; pre-push enforcement requires bootstrap progress past floor

**Decision.** `cross-link bootstrap [--all|--target-coverage 0.5]` is a CI-only / owner-only operation. Pre-push enforcement past `block-broken` requires the scope's coverage @ L3 to be above its floor.

**Options considered.**
- (a) Bootstrap as part of pre-push.
- (b) Bootstrap as a one-shot, all-or-nothing.
- (c) Bootstrap as incremental, optional.

**Chosen.** (c).

**Rationale.** Cost asymmetry — pre-push must stay cheap; bootstrap is heavy ($12+, ~30min). Incremental bootstrap (`--batch N`) lets brownfield repos climb gradually.

**Origin.** Chat turn — owner brownfield concern.

**Trade-off accepted.** Adopters need to set up a CI cron for bootstrap. Worth the friction; it's owner-controllable cost.

---

## ADR-11 — Edge state taxonomy (UNGRADED/GRADED/STALE/BROKEN) is the brownfield primitive

**Decision.** Every edge is in exactly one of four states: UNGRADED, GRADED, STALE, BROKEN. UNGRADED is a legitimate baseline state, not a defect.

**Options considered.**
- (a) Binary: "graded yes/no" with subdivisions on top.
- (b) Three states (graded / stale / broken) — UNGRADED rolled into GRADED-with-empty-history.
- (c) Four states with UNGRADED first-class.

**Chosen.** (c).

**Rationale.** UNGRADED-first-class is the answer to "how does a 233-edge repo onboard without a megacommit?" Without UNGRADED, the gate has to either pretend everything is graded (lying) or block everything (unusable).

**Origin.** Owner-flagged in chat: "I think we need to make the design support brownfield repos."

**Trade-off accepted.** Edge state classification is more nuanced; CLI verbs need to filter by state.

---

## ADR-12 — Ratcheting coverage floor (monotonic, per-scope)

**Decision.** Each scope's coverage @ L3 floor only goes up over time. Pushes that would decrease coverage below the floor BLOCK.

**Options considered.**
- (a) Hard target (e.g., 90%) — fixed threshold.
- (b) Per-PR delta requirement (every PR raises coverage by ≥X%).
- (c) Monotonic floor (only goes up).
- (d) Hybrid: floor + optional per-PR delta.

**Chosen.** (d) — floor by default; per-PR delta opt-in.

**Rationale.** Owner-directed: "coverage metric cranking up is crucial." Floor-only is the friendly default; per-PR delta is the forcing-function for projects that want it.

**Origin.** Owner direction.

**Trade-off accepted.** Floor-reset requires a CLI verb with `--reason`. Acceptable; reset is rare and audit-worthy.

---

## ADR-13 — Phased enforcement modes

**Decision.** Five enforcement modes per scope: `warn` → `block-broken` → `block-stale` → `block-floor` → `block-newedge`. Adopters progress incrementally.

**Options considered.**
- (a) Binary on/off.
- (b) Three modes (warn / block-some / block-all).
- (c) Five modes mapped to specific failure classes.

**Chosen.** (c).

**Rationale.** Different failure classes have different "always wrong" status. BROKEN is always wrong. UNGRADED-NEW is wrong only at maximum rigor. Mapping modes to failure classes lets adopters opt into rigor at the right pace.

**Origin.** Inferred from brownfield concern.

**Trade-off accepted.** More modes = more documentation. Mitigated by the journey walkthrough in [`09-brownfield-and-onboarding.md`](./09-brownfield-and-onboarding.md).

---

## ADR-14 — New-edge contract asymmetry

**Decision.** Edges introduced in a PR are held to the scope's `max_level` immediately, regardless of repo brownfield state. Edges that existed pre-adoption stay UNGRADED until their natural turn.

**Options considered.**
- (a) All edges treated equally (brownfield baseline applies to new and old).
- (b) New edges held to higher standard than old.

**Chosen.** (b).

**Rationale.** New edges have no excuse for being ungraded — author is right there, can answer the judge prompt. Old edges deserve grace; nobody can graceful-fix 233 edges in a single PR.

**Origin.** Inferred during brownfield design.

**Trade-off accepted.** "What counts as new" requires git-aware logic (`introduced_in_pr` field). Acceptable; tracker already records `discovered_at`.

---

## ADR-15 — Bootstrap is CI-only / owner-only; pre-push runs only L0+L1+L2+drift-triggered-L3

**Decision.** Pre-push hook never bulk-grades; it only L3-grades on hash-drift, capped at `max_l3_per_push`. Bootstrap runs in CI cron or owner workstation.

**Options considered.**
- (a) Pre-push handles bootstrap (lazy bootstrap on push).
- (b) CI-only bootstrap.
- (c) CI cron incremental + owner one-shot.

**Chosen.** (c).

**Rationale.** Cost asymmetry — laptops pre-push must stay <2min; bootstrap is 30min+. Cron-driven amortization keeps daily cost ~$0.50.

**Origin.** Inferred from brownfield design.

**Trade-off accepted.** Adopters need to configure CI cron. Documented in onboarding journey.

---

## ADR-16 — Coverage badge is a first-class output

**Decision.** Tracker emits per-scope shields.io badge JSON files. Badges are the primary social-pressure mechanism for adopters in `warn` mode.

**Options considered.**
- (a) No badge — tracker JSON only.
- (b) One badge per scope.
- (c) Project-aggregate badge.

**Chosen.** (b) — one badge per scope.

**Rationale.** Codecov's badge mechanism is what drove its adoption. Per-scope badges let adopters surface "fidelity @ L3 for anchor READMEs: 92%" specifically, vs a coarse aggregate.

**Origin.** Inferred during brownfield design — owner emphasized "coverage metric cranking up."

**Trade-off accepted.** README clutter if many scopes. Mitigated by aggregate badge as opt-in (sketched in open questions).

---

## ADR-17 — Fail-closed on L3 dispatch failure

**Decision.** If Sonnet API call times out or returns error, edge is marked `STALE_REGRADE_FAILED` and pre-push BLOCKs. Emergency env var `REPOSIX_CLF_SKIP_L3=1` allows bypass with audit log entry.

**Options considered.**
- (a) Fail-open: timeout → assume PASS, push continues.
- (b) Fail-closed: timeout → BLOCK, require explicit bypass.
- (c) Retry-then-fail-open.

**Chosen.** (b).

**Rationale.** Dark-factory ethos. Silent skip on outage is silent coverage erosion.

**Origin.** Inferred from project ethos.

**Trade-off accepted.** Outages block pushes. Acceptable; emergency env var is the pressure-release valve.

---

## ADR-19 — Edge identity is path-derived

**Decision.** `edge_id = sha256(source_path || source_section_path || target_path || target_anchor)`. File renames produce new edge IDs; recovery is `cross-link rebind --auto`.

**Options considered.**
- (a) Path-derived (above).
- (b) Content-derived (hash of source link text + target body).
- (c) UUID assigned on first discovery, stored permanently.

**Chosen.** (a).

**Rationale.** Path-derived is deterministic and reproducible across walks without state. Content-derived breaks on any edit. UUID requires permanent storage and breaks under git rebase. Path-derived with `rebind --auto` preserves grade across content-stable refactors while treating genuine reorganization as new edges.

**Origin.** Surfaced by review-agent in 02-architecture review as load-bearing for ratchet semantics.

**Trade-off accepted.** Bulk file moves require `rebind --auto` to preserve grade. Acceptable.

---

## ADR-20 — File-vs-folder config priority

**Decision.** If both `.cross-link-fidelity` (file) AND `.cross-link-fidelity/` (folder) exist, folder wins and file is an error (`cross-link audit` BLOCKs).

**Origin.** Surfaced by review-agent.

**Trade-off accepted.** Migration requires explicit `git rm` of the file. Acceptable.

---

## ADR-21 — `max_l3_per_push` overage on new-edge contract = BLOCK with split-PR directive

**Decision.** A PR adding more new edges to an L3 scope than `max_l3_per_push` allows BLOCKs with directive to split the PR. Does not auto-batch or queue.

**Options considered.**
- (a) BLOCK with split-PR directive.
- (b) Batch first N, mark rest STALE_PENDING, BLOCK push.
- (c) Raise cap for PR-introduced edges only.

**Chosen.** (a).

**Rationale.** Smaller PRs are easier to review and align with the dark-factory ethos. A 15-new-edge PR likely does multiple things; splitting clarifies the atomic units.

**Origin.** Surfaced by review-agent.

---

## ADR-22 — Floor never silently lowers; explicit `reset-floor` verb required

**Decision.** Floor decreases require `cross-link reset-floor <scope> --reason "<text>"` with audit-log entry. Pure deletions of GRADED edges preserve floor at prior value. Bulk file moves preserve grade via `rebind --auto`.

**Origin.** Surfaced by review-agent — "floor only goes up" was naive without policy on edge-population-change events.

**Trade-off accepted.** Owner has to run a verb explicitly to lower a floor. That's the point.

---

## ADR-18 — Last-match-wins for scope, but per-edge override beats all scopes

**Decision.** Scope resolution is last-match-wins. Per-edge overrides in `[edge_overrides]` blocks beat any matching scope.

**Options considered.**
- (a) Edge override → most-specific scope → default.
- (b) Edge override always wins; among scopes, last-match-wins.

**Chosen.** (b).

**Rationale.** Per-edge overrides are explicit acts of human judgement. They should never be overridden by scope rules; that defeats the purpose.

**Origin.** Inferred from owner's question about per-edge ignore.

**Trade-off accepted.** Two resolution paths to remember. Mitigated by `cross-link show <edge-id>` rendering the resolved scope and override.

---

## ADR-23 — Tag-filter (framework) over cadence-baked (per-scope)

**Decision.** Scope schema carries `tags: list[string]`, NOT `cadence: enum`. The Rust CLI accepts `--tags <a,b>` and `--exclude-tags <c>`; the user wires the filter into whatever orchestration they prefer (pre-commit, prek, Claude Code hooks, GitHub Actions, Bazel, MCP tools, plain cron). The framework is orchestration-agnostic.

**Options considered.**
- (a) Each scope names exactly one `cadence` (pre-commit | pre-push | weekly | on-demand). CLI verbs dispatch by cadence. Tight integration with one orchestration model.
- (b) Each scope carries `tags: list[string]` (free-form). CLI takes a `--tags` filter; user wires the filter into whatever orchestration they have. Decouples *what to grade* from *when to grade it*.
- (c) Both — `cadence` as a default plus `tags` as overrides. More complex with no marginal value over (b).

**Chosen.** (b) — tags-only on scope blocks.

**Rationale.** The framework should not assume an orchestrator. Adopters using `prek` should not have to learn reposix's cadence taxonomy; adopters using Claude Code hooks should not be forced into pre-commit semantics. The user knows their orchestration; the gate provides a label-driven CLI filter and gets out of the way. This also matches `quality/PROTOCOL.md`'s precedent that catalog rows carry `cadences: list[str]` (the same lesson, applied at a different layer of the architecture).

The boundary inside reposix: `tags` is config-side (framework, user-facing); `cadences` is catalog-side (reposix runner adapter). The catalog row's `verifier` field translates user-friendly tags into the runner's typed cadence enum at integration time. See `03-schemas.md` § "Two layers — framework vs reposix integration".

**Origin.** Owner direction 2026-05-08 mid-flight ("instead of each scope defining a cadence, wouldn't a better design be to be how bazel does it with tags where the CLI allows a tag filter or some other way to directly specify the list of scopes to run, and then the user can integrate it into whatever hook or CI they want?"). Reinforced by owner's follow-up: "the orchestration is decoupled from the framework. So if some people want to use prek vs pre-commit, vs claude tool use hooks, etc, they can." Subagent design-scrutiny session ratified.

**Trade-off accepted.** Adopters of the standalone tool must wire orchestration themselves; the framework gives them no out-of-the-box "run on every push." Documentation in the standalone-tool README must compensate with worked examples for the four most common orchestrators (pre-commit, GitHub Actions, Claude Code hooks, plain cron).

---

## ADR-24 — Per-scope `max_l3_per_push`; `walk_paths` derived from scope globs

**Decision.** Two structural changes: (1) `max_l3_per_push` moves from `[project]` to per-scope (each scope can carry its own cap; budget exhaustion is per-scope, not project-wide). (2) The top-level `[project].walk_paths` allowlist is DELETED; discovery walks the union of all non-`ignore` scopes' `source_glob`s.

**Options considered for (1).**
- (a) Top-level project-wide cap (the original design). One bucket, simple mental model. A refactor in `docs/concepts/**` could exhaust the budget for `**/README.md`.
- (b) Per-scope caps. Each scope owns its budget. Budget exhaustion stays local; the split-PR directive (ADR-21) names which scope hit its cap.

**Chosen.** (b).

**Rationale.** Cost-asymmetry between scopes is real: `nav-only` should never spend L3 budget; `anchor-readme` is the highest-stakes scope and earns generous budget; `historical` archives skip L3 entirely. A project-wide cap conflates these. Per-scope caps also make the recovery hint actionable — "scope `anchor-readme` hit its cap of 5 L3 calls; split this PR" is more useful than "the project hit its cap."

**Options considered for (2).**
- (a) Keep `[project].walk_paths` allowlist. Filters discovery before scope resolution. Two ways to declare "where to look" (this allowlist + `source_glob`).
- (b) Delete it. Discovery = union of non-`ignore` scope globs. One way to declare what's in scope.

**Chosen.** (b).

**Rationale.** Two precedence rules competing silently is a footgun. The first design's `walk_paths = ["docs/**", "**/README.md", ...]` overlapped `source_glob = "**/*.md"` — a user staring at "why didn't this edge get discovered?" must mentally diff the two. Deleting the allowlist removes the question. Excluding a path entirely is still possible (give it a scope with `ignore = true`).

**Origin.** Owner direction 2026-05-08 mid-flight. Subagent design-scrutiny session ratified.

**Trade-off accepted.** Per-scope caps add a sum-vs-max question for total cost guards: if every scope sets `max_l3_per_push = 10` and there are 5 scopes, a single push could trigger 50 L3 calls. Mitigated by an OPTIONAL `[project].max_l3_per_push_total` for adopters who want a hard global ceiling — not in v1, GOOD-TO-HAVES candidate.

---

## ADR-25 — Catalog (runner-readable) and Tracker (gate-internal) live in different files

**Decision.** Two files, two purposes:
- `quality/catalogs/cross-link-fidelity.json` — runner-readable catalog (~4 rows) following the unified row schema (`id`, `dimension`, `kind`, `cadences`, `expected.asserts`, `verifier`, `artifact`, `status`, `blast_radius`). The reposix runner discovers it.
- `quality/state/cross-link-fidelity-tracker.json` — gate-internal per-edge tracker (per-edge state, last_graded_hash, last_verdict, etc.). The runner does NOT touch it; only the gate's CLI reads/writes it.

**Options considered.**
- (a) One file at `quality/catalogs/cross-link-fidelity.json` with both ~4 catalog summary rows AND ~400 per-edge entries. Compact but breaks the runner schema (`runners/run.py:62-69` discovers and validates against the unified row schema; would crash on edge rows).
- (b) Two files (catalog + tracker), separated by purpose.

**Chosen.** (b).

**Rationale.** The cross-gate design review (2026-05-08, second subagent scrutiny) flagged this as a hard architectural collision. The runner's discovery code at `quality/runners/run.py:62-69` expects every JSON in `quality/catalogs/` to match the unified row schema. A 400-edge tracker file at that path would either crash the runner or force schema-evolution on the runner side. Both are bad. Separating the files preserves the runner contract and gives the gate a clean place to put its high-cardinality state.

The catalog rows carry the four scrutiny levels (with L2 bundled into L3 because L2 only triggers L3) plus a floor-not-decreased invariant row. The tracker carries per-edge bookkeeping. Each file does one job.

**Origin.** Subagent cross-gate design review 2026-05-08 finding C-2.

**Trade-off accepted.** Two files mean two read paths in the gate's CLI; mitigated by a single `cross-link audit` verb that cross-checks consistency (every catalog row's verifier output references valid tracker entries).

---

## ADR-26 — L3 dispatch reuses `persist_artifact` + Path A/B precedent

**Decision.** L3 (LLM-judged fidelity) MUST dispatch via the existing subjective-rubric infrastructure shipped in `.claude/skills/reposix-quality-review/`. Specifically:
- L3 verdicts persist via `lib/persist_artifact.py:33-59` (`persist_artifact()`) using the canonical `{ts, score, verdict, rationale, evidence_files, dispatched_via, asserts_passed, asserts_failed, stale}` shape.
- Dispatch follows the **Path A / Path B split** pattern from `lib/dispatch_inline_subagent.sh:39-76`: Path A (in-session via Claude Code Task tool) is the unbiased grading path; Path B (subprocess stub returning FAIL) is the CI fallback when no API key + no in-session orchestrator is available.

**Options considered.**
- (a) New shell wrapper at `quality/gates/cross-link-fidelity/verifiers/l3-fidelity-judge.sh` invoking Anthropic SDK directly. Parallel to the rubric dispatcher.
- (b) Reuse the existing rubric dispatcher (Path A/B). One dispatch chain, one persistence helper.

**Chosen.** (b).

**Rationale.** The infrastructure already exists, is dogfooded by `quality/catalogs/subjective-rubrics.json`, and has a documented MIGRATE-03 lesson (runner sweep stomping fresh Path-A artifacts) that we'd hit again if we built a parallel system. Reusing it gets us the artifact shape, the Path A/B fallback, and the artifact-persistence safety net — all for free.

**Origin.** Subagent cross-gate design review 2026-05-08 finding C-4 + S-3.

**Trade-off accepted.** Coupling to the rubric infrastructure. If the rubric dispatcher changes shape, cross-link L3 changes with it. Acceptable: changes there will be reposix-wide and we want cross-link to inherit them.

---

## ADR-27 — Lift markdown walker to a shared module

**Decision.** Before P97 (P1) lands edge-walker code, lift the existing markdown-walking utilities from `crates/reposix-quality/src/coverage.rs` (specifically `eligible_files()` at line 46, `walk_md()` at line 73, and the related glob/path normalizers) into a shared module — either `crates/reposix-quality/src/md_walker.rs` (intra-crate) or a new `crates/reposix-md-walker/` crate (extraction-friendly per `07-extraction-plan.md`). Both `docs-alignment` and cross-link-fidelity consume the shared module.

**Options considered.**
- (a) Cross-link's walker is a separate fork of `coverage.rs::walk_md`. ~150 LOC duplicated. Two divergent walkers in one binary.
- (b) Lift to `src/md_walker.rs` inside `reposix-quality` (lower-risk; both gates inside the same crate). Cross-link calls the same `walk_md`.
- (c) Lift to a standalone `crates/reposix-md-walker/`. Strictly cleaner for extraction; adds a workspace member.

**Chosen.** (b) for v1, with (c) as the v0.13.3 extraction target.

**Rationale.** The markdown walker is the bottleneck path for both docs-alignment and cross-link. Two divergent walkers will diverge silently (different glob behavior, different ignore rules, different normalization). The lift is ~150 LOC and makes both gates cheaper to evolve. (c) is the right v0.13.3 move when the extraction plan crystallizes; v1 lifts inside `reposix-quality` to minimize blast radius.

**Origin.** Subagent cross-gate design review 2026-05-08 finding S-1.

**Trade-off accepted.** P97 (P1) gains a refactor task. The refactor must precede new edge-walker code so cross-link is built on the lifted module from day one.

---

## ADR-28 — Add `heading_subtree_hash` next to existing hash helpers

**Decision.** Add `pub fn heading_subtree_hash(file: &Path, slug: &str) -> Result<String>` to `crates/reposix-quality/src/hash.rs` next to the existing `source_hash` (line 29) and `test_body_hash` (line 92). Implementation: parse the markdown via the workspace's `pulldown-cmark = "0.13"` dep, locate the heading whose slug matches, sha256 the AST subtree from that heading until the next same-or-higher-level heading.

**Options considered.**
- (a) New file, new module, fresh hash implementation. Parallel to existing hash.rs.
- (b) Add to existing hash.rs next to `source_hash` + `test_body_hash`. Three hashes in one module.

**Chosen.** (b).

**Rationale.** Hash-drift detection is one concern: identifying that a piece of source content (a Rust function body, a line range, a markdown heading subtree) has changed since last grade. Three flavors, one module, identical error handling. Cross-link's L2 is structurally the same problem docs-alignment solved with `source_hash`/`test_body_hash`. Splitting them into separate modules duplicates error-handling boilerplate.

**Origin.** Subagent cross-gate design review 2026-05-08 finding S-2.

**Trade-off accepted.** Coupling to `pulldown-cmark`'s AST shape; if the parser changes, all three hashes might shift in lockstep. Acceptable; the parser is workspace-pinned and stable.
