# v0.12.0 — Roadmap Rationale

> **Audience.** The agent planning each phase of v0.12.0. Pairs with `.planning/ROADMAP.md` (the structured artifact) — this doc explains the WHY behind the phase shape, dependencies, and pivot rules.

## Phase ordering — why P56 before P57

P56 (RELEASE-01: install URL fix) ships BEFORE the framework, even though the framework would prevent the recurrence.

**Why:** users are blocked right now. Every documented install path is broken. The framework is high-leverage long-term but cannot ship in <1hr; the release.yml fix can. **OP-1 close-the-loop** says: don't let the user-facing breakage marinate while we build the system that would have caught it.

Trade-off: P56 ships a fix without its catalog row, so the regression class can recur until P58 builds the release dimension. Mitigation: after P56 ships, the same agent's next action is to scaffold the release-asset catalog row in v0.12.0/quality/catalogs/release-assets.json (even before P57 finalizes the schema) so the fix is at least covered in spirit.

## Phase-by-phase rationale

### P56 — Restore release artifacts (THE BLOCKER)

**Goal:** Fix `release.yml` so it fires on release-plz's per-crate tags. Verify all 5 install paths (curl, powershell, brew, cargo binstall, build-from-source) work end-to-end against a fresh release.

**Why first:** users are blocked. Every documented install path either fails outright (curl/powershell — `Not Found`), falls back to slow source build (cargo binstall), or installs the wrong version (brew formula stale). Diagnosed in `v0.12.0-install-regression-diagnosis.md`.

**Dependencies:** none. Phase 56 is the entry-point phase.

**Pivot rule:** if extending the tag glob breaks the `${{ github.ref }}`-derived version derivation, fall back to a release-plz post-publish step that mirrors a workspace `vX.Y.Z` tag whenever `reposix-cli` ships. Either approach lifts all 4 broken install rows.

**Done means:** a fresh `reposix-cli-v0.11.3` release exists with all 7 expected assets (5 platform archives + 2 installers + SHA256SUMS). All 4 install rows in `.planning/docs_reproducible_catalog.json` flip from FAIL/PARTIAL to PASS.

### P57 — Quality Gates skeleton + structure dimension

**Goal:** Build the framework: `quality/{gates,catalogs,reports,runners}/`, `quality/PROTOCOL.md`, `quality/SURPRISES.md`. Migrate the 6 freshness invariant rows from `scripts/end-state.py` into `quality/gates/structure/` as the framework's first proof-of-life. Reduce `end-state.py` to a thin shim.

**Why second:** every later phase needs the framework. Until P57 ships, we have no `quality/runners/run.py` to invoke and no catalog schema to fill. Migrating the structure dimension first (as opposed to release or docs-repro) is deliberate: structure rows are mechanical, deterministic, well-understood — perfect for shaking out the framework's plumbing without dimension-specific complexity.

**Dependencies:** P56's verifier-verdict GREEN (release artifacts exist; otherwise we're building on broken ground for any release-related row).

**Pivot rule:** if pre-push perf regresses (the runner discovery + composition adds >10s vs the old script chain), keep `scripts/end-state.py` as the pre-push entry, run new system in parallel for parity-checking, hard-cut in P63. **Do not silently downgrade** — file a waiver row + SURPRISES journal entry.

**Done means:** 
- `quality/runners/run.py --cadence pre-push` produces the same verdict as the old `scripts/end-state.py verify` for the 6 freshness rows.
- `scripts/end-state.py` is ≤30 lines + anti-bloat header comment.
- `quality/PROTOCOL.md` is written and read at start of P58.
- `quality/SURPRISES.md` exists (may be empty).
- CLAUDE.md updated with the dimension/cadence/kind taxonomy section.

### P58 — Release dimension + code absorption

**Goal:** Build the dimension that would have caught RELEASE-01 within a week. `quality/gates/release/` with verifiers for `gh-assets-present`, `brew-formula-current`, `crates-io-max-version`, `installer-asset-bytes`. New CI workflows `quality-weekly.yml` (cron) + `quality-post-release.yml` (triggered).

**Why third:** RELEASE-01 was the proximate cause of v0.12.0; the release dimension prevents recurrence. Also folds `scripts/check_clippy_lint_loaded.sh` and `scripts/check_fixtures.py` into the code dimension (which mostly references existing CI).

**Dependencies:** P57 catalog schema GREEN; P56's release pipeline working.

**Pivot rule:** if `quality-post-release.yml` chains brittle (chaining workflows in GH Actions has a history of subtle failures), fall back to a single workflow with `cron + workflow_dispatch + workflow_run` triggers. Document the choice in SURPRISES.md.

**Done means:** the next time release-plz silently changes its tag scheme (or any equivalent regression), `quality-weekly.yml` flags it within 7 days. CLAUDE.md updated with the release-dimension entry. SIMPLIFY-04 and SIMPLIFY-05 absorbed.

### P59 — Docs-repro dimension

**Goal:** Snippet extraction from user-facing docs + container rehearsal harness + tutorial replay. Promote `scripts/repro-quickstart.sh` into a tracked gate. Make every `examples/0[1-5]-*/run.sh` a docs-repro catalog row.

**Why fourth:** docs-repro is the dimension where the install-curl regression actually MANIFESTED (a documented command stopped working for a fresh user). With release dim + docs-repro dim both shipped, the regression class is double-covered: the asset-exists drift detector catches the URL going dark; the container rehearsal catches the URL "exists but doesn't actually install a working binary."

**Dependencies:** P58 release dimension GREEN (so post-release container rehearsal can run against verified-present assets).

**Pivot rule:** if container time blows the budget (>15min per release), drop multi-persona; ship ubuntu-only first; mac/windows matrix moves to v0.12.1.

**Done means:** every fenced code block in README + docs/index.md + docs/tutorials/* has a catalog row. Container rehearsal runs in CI on post-release. Examples are gated. SIMPLIFY-06, SIMPLIFY-07, SIMPLIFY-11 (file-relocate only) absorbed.

### P60 — Docs-build migration

**Goal:** Move `scripts/check-docs-site.sh`, `scripts/check-mermaid-renders.sh`, `scripts/check-doc-links.py` into `quality/gates/docs-build/`. Pre-push hook body becomes one line. Supplant `scripts/green-gauntlet.sh`.

**Why fifth:** by P60, four dimensions are done; the docs-build migration is mostly mechanical (rename + colocate) and serves to demonstrate that the framework can absorb a fully-functioning existing surface without behavior change. Last-stage validation that the framework composes correctly.

**Dependencies:** P59 docs-repro GREEN.

**Pivot rule:** if path moves break hooks, leave shims at old paths; document in SURPRISES.md.

**Done means:** `scripts/` is down to `hooks/` + `install-hooks.sh` + thin shims. SIMPLIFY-08, SIMPLIFY-09, SIMPLIFY-10 absorbed.

### P61 — Subjective gates skill

**Goal:** `quality/catalogs/subjective-rubrics.json` with 3 seed rubrics (hero clarity, install positioning, headline numbers). `reposix-quality-review` skill that dispatches subagents per stale row. Pre-release cadence wires it.

**Why sixth:** subjective gates are the new capability — making rubric checks tractable via TTL freshness and subagent dispatch. Lower urgency than the dimensions but high leverage long-term (cold-reader-pass kind of checks become routine instead of one-off heroics).

**Dependencies:** P57 catalog schema; P58 cadence routing.

**Pivot rule:** if subagent grading varies wildly run-to-run (same artifact graded 8/10 then 5/10), switch rubric to numeric scores with a stability check (re-grade twice; flag if delta > N). Keep the catalog row but mark it PARTIAL until stability stabilizes.

**Done means:** at least one subjective rubric runs end-to-end (subagent dispatch → JSON artifact → catalog updates → verdict.md reflects the score). The skill is invokable via owner-on-demand and via pre-release cron.

### P62 — Repo-org-gaps cleanup

**Goal:** Audit `.planning/research/v0.11.1/repo-organization-gaps.md` — every remaining gap is either fixed + given a structure-dimension catalog row that prevents recurrence, OR explicitly waived with reason.

**Why seventh:** owner explicitly requested this be folded into v0.12.0 (not lost as a separate todo). It's mostly already-done work plus a handful of remaining gaps; doing it inside the QG framework means each closure becomes a permanent catalog row.

**Dependencies:** P57 catalog schema; P61 subjective rubric pattern (some gaps are subjective).

**Pivot rule:** if a gap genuinely can't be auto-detected (e.g., "naming feels off"), file it as a subjective rubric (P61 pattern) instead of forcing it to mechanical.

**Done means:** every row in v0.11.1-repo-organization-gaps.md has either a corresponding catalog row OR a waiver. The doc itself can be archived (or a stub left pointing to the catalog rows).

### P63 — Retire old + cohesion + v0.12.1 forward

**Goal:** Final SIMPLIFY-12 audit (`scripts/` is clean). MIGRATE-01 deletes fully-migrated old scripts. MIGRATE-02 cohesion pass on CLAUDE.md (each prior phase already added incrementally; this is the cross-link audit). MIGRATE-03 stubs v0.12.1 carry-forward.

**Why eighth/last:** can't retire old until new is proven (per parallel migration rule). Can't write the cohesion CLAUDE.md update until all prior phases have shipped their incremental updates.

**Dependencies:** every prior phase verdict GREEN + 2 consecutive GREEN pre-push runs across the full runner.

**Pivot rule:** if any SIMPLIFY-* item genuinely can't be absorbed, file an `orphan-scripts.json` waiver row with reason (e.g. "tied to external CI system; can't move without coordinated change").

**Done means:** `find scripts/ -maxdepth 1 -type f | grep -v hooks | grep -v install-hooks.sh` returns empty (modulo waivers). v0.12.1 carry-forward filed. Milestone closes via standard `/gsd-complete-milestone` flow.

## Phase preconditions are gate states, not just predecessor merged

The standard GSD pattern is "Phase N depends on Phase N-1." v0.12.0 strengthens this:

> **Phase N may only start when Phase N-1's `quality/reports/verdicts/p<N-1>/` shows GREEN.**

A merged-but-RED predecessor doesn't unblock the next phase. This is the same rule that prevents the framework from shipping half-broken: if P57's runner doesn't actually produce GREEN verdicts for the migrated structure rows, P58 can't start (because it would be building on unproven plumbing).

When the autonomous agent picks up the next phase, its FIRST action is `cat quality/reports/verdicts/p<previous>/<latest>.md` — if it's not GREEN, the agent escalates rather than proceeding. (Per the §0.8 verifier dispatch pattern, a fresh subagent grades; the executing agent can't talk it out of RED.)

## Blast-radius analysis

| Phase | If it ships RED, what breaks? | Recovery cost |
|---|---|---|
| P56 | Users still blocked on install. | High — back to square one |
| P57 | Framework is half-built; no later phase can ship its dimension correctly. | Very high — every later phase has to wait |
| P58 | Release dimension absent; RELEASE-01 class can recur silently. | Medium — quality-weekly.yml not running |
| P59 | Docs-repro absent; install-curl-class regressions still slip past. | Medium — manual cold-reader needed |
| P60 | Docs-build still in `scripts/`; framework looks half-migrated. | Low — cosmetic but signals incomplete |
| P61 | Subjective rubrics not auto-graded; one-off heroics continue. | Low — the dimensions still work mechanically |
| P62 | Repo-org gaps remain as a forgotten todo. | Low — doesn't affect quality gates |
| P63 | Old scripts hang around; framework looks half-finished forever. | Medium — confusing for next-agent maintenance |

P56 and P57 are existential (can't ship subsequent phases without them). P58 is high-value (the proximate cause of v0.12.0). P59–P63 are completion / quality-of-life. The autonomous agent should escalate hard if P56 or P57 fails to verify GREEN; can defer / file v0.12.1 carry-forward for P59+ if time runs out.

## Compression options if time-boxed

If autonomous mode runs out of time and must descope:

| Tier | Phases that ship | Floor that holds |
|---|---|---|
| **Floor** | P56 only | install paths work; framework deferred to v0.12.1 |
| **Minimal viable framework** | P56 + P57 | structure dim migrated; release/docs-repro deferred |
| **Plus the proximate cause** | P56 + P57 + P58 | release dim catches the regression class |
| **Recommended** | P56 + P57 + P58 + P59 | all four high-value dimensions shipped |
| **Full** | P56–P63 | everything; clean repo |

Anything beyond the **Recommended** tier is high-leverage but not user-facing-urgent. Owner has explicitly accepted descoping to v0.12.1 if needed (see `v0.12.0-open-questions-and-deferrals.md`). The catalog rows for SIMPLIFY-* items are themselves the v0.12.1 todo if time runs out.

## Cross-references

- `.planning/ROADMAP.md` `## v0.12.0 Quality Gates (PLANNING)` — the structured roadmap artifact
- `.planning/REQUIREMENTS.md` `## v0.12.0 Requirements` — full RELEASE-* / QG-* / etc. list with phase mapping
- `v0.12.0-vision-and-mental-model.md` — WHY this milestone exists at all
- `v0.12.0-naming-and-architecture.md` — the file layout each phase ships into
- `v0.12.0-autonomous-execution-protocol.md` — runtime contract for the agent executing each phase
