# v0.12.0 — Vision and Mental Model

> **Audience.** The next agent picking up v0.12.0. Read this BEFORE planning P56. Sibling docs:
> - `v0.12.0-naming-and-architecture.md` — concrete file layout + catalog schema
> - `v0.12.0-roadmap-and-rationale.md` — phase-by-phase reasoning
> - `v0.12.0-autonomous-execution-protocol.md` — runtime contract for autonomous mode
> - `v0.12.0-install-regression-diagnosis.md` — P56's specific work (THE blocker)
> - `v0.12.0-decisions-log.md` — owner Q&A from the planning session
> - `v0.12.0-open-questions-and-deferrals.md` — what's explicitly out

## The thing we are fixing

The v0.11.x cycle shipped a §0.8 SESSION-END-STATE framework (`scripts/end-state.py`) that grades 20 claims at pre-push. It catches:

- ✓ Freshness invariants (no version-pinned filenames; install path leads with package manager; benchmarks in mkdocs nav; no loose ROADMAP/REQUIREMENTS).
- ✓ Mermaid renders (playwright artifact per how-it-works page).
- ✓ crates.io max_version per crate.
- ✓ Mkdocs strict build.

It catches everything those rows enumerate. The session ended at 20/20 GREEN.

But the **curl install URL went dark** for two releases without anyone noticing — because:

1. release-plz cut over to per-crate tags (`reposix-cli-v0.11.1`, `reposix-cli-v0.11.2`, …).
2. `.github/workflows/release.yml` only fires on tag glob `v*`. Per-crate tags don't match (they start with `r`, not `v`).
3. `release.yml` hasn't run since the manual `v0.11.0` workflow_dispatch on 2026-04-26.
4. Every release after v0.11.0 has `assets:[]` — no installer, no archives.
5. `releases/latest/download/reposix-installer.sh` 302s to a release with no asset → `Not Found`.

The §0.8 framework didn't catch this because it doesn't have a row for "the documented install URL returns >0 bytes of shell." The class of regression — *external integration drift between releases* — has no home in the framework. So when the owner asked "what else might not be reproducible?", the honest answer was: **nobody knows, because nobody's checking.**

That's the gap v0.12.0 closes. Not by adding one more bespoke check — by building a coherent system where every regression class HAS a home, every check is a catalog row, every catalog row has a verifier, and every verifier produces an artifact that an unbiased subagent can grade.

## Three orthogonal axes

The mental model is a 3D grid. Every quality check (gate) sits at some `(dimension, cadence, kind)` coordinate. This is the structural break from "every script is its own special snowflake."

### Axis 1 — Dimension (what is being checked)

The eight regression classes the project actually has. Each owner-caught miss in v0.11.x maps cleanly to one of these:

| Dimension | What it checks | Examples of regressions it would catch |
|---|---|---|
| **code** | does the code compile, lint clean, pass tests | clippy regression, test failure, unsafe code introduced |
| **docs-build** | does the site build; do mermaids render; do links resolve | mermaid race condition (§0.1), broken pymdownx config, link rot |
| **docs-repro** | do documented commands actually work for a fresh user | the curl-installer regression; tutorial bit-rot; install-path drift |
| **release** | are install URLs / brew formula / crates.io versions current and consistent | RELEASE-01 (this session); brew formula stale; binstall fallback to source |
| **structure** | freshness invariants (file org, naming, no orphans) | §0.3 version-pinned filenames; §0.4 missing nav entry; §0.5 loose ROADMAP/REQUIREMENTS |
| **perf** | benchmarks vs headline copy | latency regression; token-economy bench drift from "89.1%" hero number |
| **security** | allowlist enforcement, audit immutability | new direct `reqwest::Client::new()` site bypassing the factory; UPDATE/DELETE on audit table |
| **agent-ux** | dark-factory regression (pure-git, zero in-context learning) | helper namespacing change breaks fresh-agent flow; sparse-checkout teaching string regresses |

In v0.12.0 we ship the framework + four dimensions in depth (release, structure, docs-repro, docs-build) + thin homes for two more (code, agent-ux). Perf and security ship as stubs with v0.12.1 carry-forward.

### Axis 2 — Cadence (when the gate runs)

The cadence determines budget and blocking-vs-alerting:

| Cadence | When | Cost budget | Blocking? | Example gates |
|---|---|---|---|---|
| **pre-push** | local, every push | <60s | yes | freshness invariants, banned-words, mkdocs-strict |
| **pre-pr** | per-PR CI | <10min | yes | cargo, dark-factory, mermaid renders |
| **weekly** | cron | <5min | alerting only | asset-exists URL HEADs, waiver-TTL audit, catalog↔doc snippet-drift |
| **pre-release** | on tag push, BEFORE release.yml promotes | <15min | yes | container-rehearse the JUST-built binaries; subjective rubrics with TTL ≥14d expired |
| **post-release** | after release.yml ships assets | <15min | alerting + auto-rollback | container-rehearse `releases/latest/...` URLs from a fresh image |
| **on-demand** | manual / subagent dispatch | unbounded | no | subjective rubrics, cold-reader pass |

**Cadence is `weekly`, not `nightly`** — explicit owner cost decision. Drift detectors don't need to fire 7× more often than weekly to catch a regression class that takes hours-to-days to manifest.

The install-curl regression: the asset-exists check at `cadence: weekly` (cheap HEAD) would have caught it within 7 days. Container rehearsal at `cadence: post-release` (~5min once per ship) proves a fresh user can install. We don't need both checking nightly; the cheap one weekly + the expensive one per-release is the right cost shape.

### Axis 3 — Kind (how the gate is verified)

| Kind | Mechanism | Cost | Example |
|---|---|---|---|
| **mechanical** | deterministic shell + asserts | low | freshness regex, mkdocs strict |
| **container** | spin fresh ubuntu/alpine/macos, run snippet, assert post-conditions | medium | install-from-curl rehearsal |
| **asset-exists** | HEAD/GET URL, assert HTTP 200 + min-bytes + magic-prefix | very low | the cheap weekly drift detector |
| **subagent-graded** | dispatch unbiased subagent with rubric, persist scored artifact | medium | cold-reader hero clarity |
| **manual** | human-only; catalog enforces "rerun within Nd or RED" | unbounded | rubrics that resist automation |

The genius of `subagent-graded` is that it makes inherently subjective checks **trackable**: the catalog enforces "this rubric was checked recently," even if the check itself is a judgment call. No more "we audited that once six months ago and never again."

## Why this composes (the part that matters for the next agent)

A gate is just `(dimension, cadence, kind, verifier, expected_outcome)`. The runner doesn't care which dimension or kind — it discovers gates tagged with the requested cadence and runs them.

```
quality/runners/run.py --cadence pre-push
  → reads quality/catalogs/*.json
  → filters: row.cadence == "pre-push" AND row.status != "waived"
  → for each row: invoke row.verifier with timeout
  → write artifact to row.artifact path
  → exit non-zero if any row's verifier exited non-zero

quality/runners/verdict.py
  → reads all row.artifact files
  → emits quality/reports/verdicts/<cadence>/<ts>.md
  → exits non-zero if any row is RED
```

Adding a new check is:
1. Append a row to the right catalog file.
2. Write the verifier in `quality/gates/<dim>/`.
3. Done. No new pre-push wiring, no new CI workflow, no new bespoke script.

When an owner catches a miss next time, the meta-rule is: *"fix the issue, update CLAUDE.md, AND tag the dimension."* The dimension tag tells the agent which `quality/catalogs/<file>.json` to add the row to and which `quality/gates/<dim>/` directory the verifier belongs in. The fix becomes structural rather than ad-hoc.

## What this is NOT

To set expectations for the autonomous agent:

- **Not a CI rewrite.** GitHub Actions workflows mostly stay. New workflow `quality-weekly.yml` (cron) and `quality-post-release.yml` (triggered) get added; existing workflows largely call into `quality/runners/run.py`.
- **Not a new test framework.** Cargo + nextest stay. The code dimension references existing CI; it doesn't try to re-implement what already works.
- **Not a feature-flag system.** Waivers are the principled way to say "this row is known-failing for reason X until date Y." They're explicit, expire, and require justification.
- **Not infinitely extensible.** v0.12.0 ships 4 dimensions in depth + 2 thin + 2 stubs. v0.12.1 fills out perf + security. v0.12.2+ is for new dimensions IF they emerge from owner-caught misses, not speculative.

## The one principle that everything else follows from

**The executing agent's word is not the verdict.** Every claim that the system is OK gets graded by an unbiased subagent that has zero session context — only the catalog row + the artifact. This is the §0.8 verifier pattern from v0.11.2 generalized to every gate, every phase, every milestone close.

If the agent says "I shipped X" but the verifier subagent reads the catalog and sees no artifact dated this session, the row is RED. The agent doesn't get to talk the verifier out of it. The structural break from "agent self-reports done" to "verifier grades from artifacts" is the single most important design choice in v0.12.0. Everything else — dimensions, cadences, catalogs, runners — is plumbing for that one principle.

## Why this is a milestone, not a phase

Building this right requires:

- A unified catalog schema (`v0.12.0-naming-and-architecture.md` §catalog-schema)
- Runners that compose by cadence
- Migrating ~14 existing scripts into the dimension structure
- A new skill (`reposix-quality-review`) for subjective rubrics
- A runtime contract (`quality/PROTOCOL.md`) for autonomous-mode agents
- CLAUDE.md updates per phase (not just at the end) — owner-flagged QG-07
- Aggressive simplification: absorbing every existing quality script into the framework instead of leaving them as parallel cruft (owner directive — see SIMPLIFY-01..12)
- Phase preconditions that are gate states, not just "previous phase merged"

That's 8 phases of real work (P56–P63). One phase couldn't ship even half of it coherently. The framework only earns its keep when every dimension has at least one gate; partial framework + bespoke scripts is the worst of both worlds.

## What "done" looks like for v0.12.0

When the milestone closes:

- `quality/` directory exists with `gates/{code,docs-build,docs-repro,release,structure,agent-ux}/`, `catalogs/`, `reports/`, `runners/`, `PROTOCOL.md`, `SURPRISES.md`.
- `scripts/` holds only `hooks/` and `install-hooks.sh`. Every other script either folded into `quality/gates/<dim>/`, reduced to a one-line shim, or has a waiver row in `quality/catalogs/orphan-scripts.json`.
- `examples/0[1-5]-*/run.sh` each appear as catalog rows (container-rehearsal-kind, post-release cadence).
- The curl installer URL works end-to-end for a fresh ubuntu container (RELEASE-01 fix verified by the new container-rehearsal gate).
- Pre-push hook body is a single `quality/runners/run.py --cadence pre-push` invocation.
- `scripts/end-state.py` is a ≤30-line shim with anti-bloat header comments.
- A weekly cron workflow runs the asset-exists drift detectors and would catch a future RELEASE-01-class regression within 7 days.
- A post-release workflow runs container rehearsal and would catch a "asset shipped but doesn't actually work on a fresh box" regression immediately after every release.
- CLAUDE.md has a new section documenting the dimension/cadence/kind taxonomy + the meta-rule extension ("fix the issue, update CLAUDE.md, AND tag the dimension").
- v0.12.1 carry-forward filed: perf-dimension, security-dimension, cross-platform container rehearsals, the `Error::Other` 156→144 partial migration completion.

When all of that ships, every future quality miss the owner catches becomes one catalog row + one verifier — never another bespoke script. That's the slope-of-future-sessions change that makes v0.12.0 the highest-leverage milestone since the v0.9.0 architecture pivot.
