# 09 — Brownfield support and onboarding

> **Read first.** [`02-architecture.md`](./02-architecture.md) § "Edge states" + § "Ratcheting coverage" + § "Phased enforcement modes."
> **Read next.** [`07-extraction-plan.md`](./07-extraction-plan.md) — brownfield is the killer feature for the standalone tool.

## The brownfield problem in one sentence

A repo with 233 existing edges and zero historical L3 grades cannot be onboarded by demanding 100% coverage on day 1; the gate must be useful while coverage is climbing.

## The four mechanics that make brownfield work

The architecture chapter introduces these primitives. This chapter explains how they compose into an onboarding experience.

| Mechanic | What it does | Without it |
|---|---|---|
| **UNGRADED edge state** | Distinguishes "never been graded" from "drifted since grade" | A new repo's first push BLOCKs every edge |
| **Ratcheting coverage floor** | Allows pushes that don't decrease coverage; floor advances on improvements | Either day-1 100% requirement OR no enforcement at all |
| **Phased enforcement modes** | Adopters move warn → block-broken → … → block-newedge over time | Either too strict or too loose |
| **CI/local cost split** | L3 bootstrap on CI; pre-push is mechanical-only | Contributor laptops hang for minutes per push |

## The onboarding journey (step-by-step)

### Day 1: Install + measure

```bash
# Install the gate
cargo install reposix-quality  # OR pip install cross-link-fidelity (post-extraction)

# Discover the graph (read-only, no commits)
cross-link suggest-scopes
```

`suggest-scopes` walks markdown, classifies edges by structural heuristics, and emits a starting `.cross-link-fidelity` config:

```
Found 233 edges in 49 markdown files.
  - 14 edges from anchor READMEs (file path matches **/README.md)
  - 47 edges in "See also" lists
  - 172 inline references in body prose

Suggested scopes:
  default          max_level=L1  enforcement_mode=warn
  anchor-readme    max_level=L3  enforcement_mode=block-stale  count=14
  see-also         max_level=L1  enforcement_mode=block-broken count=47

Wrote .cross-link-fidelity (review before commit).
```

Owner reviews the suggested config, commits it. Tracker is empty; no edges graded.

### Day 1 (still): Status check + warn-only

```bash
cross-link status
```

```
Edge inventory: 233 total
  GRADED:    0  (0%)
  STALE:     0
  BROKEN:    2   ← link-resolves failures
  UNGRADED:  231

Coverage @ L3: 0% (floor: 0%)
Enforcement: warn (no pushes will block on this gate)

Run `cross-link bind --fix-broken` to address 2 BROKEN edges.
```

Pre-push gate is on but only WARNs. Owner addresses the 2 BROKEN edges (typo in a path, removed file), pushes. No friction.

### Week 1: Background bootstrap

Owner schedules a CI cron job:

```yaml
# .github/workflows/cross-link-bootstrap.yml
on:
  schedule:
    - cron: "0 4 * * *"  # daily 04:00 UTC
jobs:
  bootstrap-batch:
    steps:
      - run: cross-link plan-refresh --batch 20 --target UNGRADED
      - run: git commit -am "cross-link: bootstrap batch (20 edges, $(date +%F))"
```

Each daily run grades 20 UNGRADED edges. After ~12 days, all 231 are GRADED. Coverage @ L3 climbs from 0% → 8% → 16% → ... → 100% over ~2 weeks. Cost: ~$0.60/day, $8 total bootstrap spend.

Owner could also run `cross-link bootstrap --all` once for $12 instant coverage. Both work.

### Cost at scale

The $8/$12 numbers are for this project's 233-edge graph. Per-edge unit cost is ~$0.05 (Sonnet, p50). Sized examples:

| Edge count | `bootstrap --all` cost | Daily batch (20/day) duration | Daily batch cost |
|---|---|---|---|
| 200 | ~$10 | 10 days | ~$1/day |
| 400 (this project) | ~$20 | 20 days | ~$1/day |
| 1500 | ~$75 | 75 days (≈2.5 mo) | ~$1/day |
| 10k | ~$500 | 500 days | ~$1/day; consider `--batch 50` (~$2.50/day, 200 days) |

For graphs >1000 edges, `--all` is rarely the right move. Use incremental `--batch N` over weeks; coverage climbs visibly via the badge while cost stays ≤ $5/day.

**Required secrets / infrastructure:**

- `ANTHROPIC_API_KEY` repository secret — granted to the bootstrap workflow only.
- A bot identity for the auto-commit step — typically a deploy key or fine-grained GitHub App. The workflow needs `contents: write` on the branch it commits to (usually `main`).
- Branch protection rules need an exemption for the bot OR the bot pushes to a `cross-link/bootstrap-batch` branch and opens a PR.
- For repos with CODEOWNERS, the auto-commit may trigger required-review failures; either commit-and-PR or add the bot to the relevant CODEOWNERS lines.

For repos without owner-side budget approval flow, this can be a multi-week security review. Plan for it.

### Week 2: Promote to block-broken

Once coverage @ L3 ≥ 30%, owner edits `.cross-link-fidelity`:

```toml
[scopes.default]
enforcement_mode = "block-broken"  # was "warn"
```

Pre-push now BLOCKs on broken links (cheapest signal). Floor for `default` is set to current coverage (30%, monotonic).

### Week 4: Promote to block-stale

Coverage @ L3 ≥ 80%. Floor is at 80%. Owner promotes:

```toml
[scopes.default]
enforcement_mode = "block-stale"
```

Now: editing a target file with stale parent forecasts BLOCKs the push. Drift is caught.

### Week 6: Block-floor + new-edge contract

Coverage stable around 95%. Owner promotes to `block-floor` for `default`, `block-newedge` for the `anchor-readme` scope (where stakes are highest). Steady-state reached.

## The migration assistant: `cross-link suggest-scopes`

The single command that makes onboarding tractable. Its job is to look at an existing graph and propose a *reasonable starting config* — not a perfect one, just enough to commit and start running.

### Heuristics for auto-classification

The assistant uses these signals to suggest scope membership:

| Signal | Suggested scope | Suggested level |
|---|---|---|
| Source path matches `**/README.md` AND target is a sibling/descendant | `anchor-readme` | L3 |
| Source path is in `**/concepts/` or `**/architecture/` | `concept-doc` | L3 |
| Edge is in a list under heading `/^see also$/i` etc | `nav-only` | L1 |
| Source path matches `.planning/archive/**`, `.planning/research/**` | `historical` | L0, `count_in_coverage: false` |
| Edge target is a non-md file | `external-target` | L1 (no anchor checks possible) |
| Default fallback | `default` | L1 (until owner promotes) |

### What `suggest-scopes` won't catch

The heuristics above are starting points, not perfect. Specifically:

- **Monorepo with 5 sub-projects.** A blanket `**/concepts/` glob matches every project's `concepts/` dir into a single `concept-doc` scope. For monorepos, post-suggestion edits should split into `<project>/concepts/` per-project scopes.
- **Auto-generated docs in non-`.planning/` paths.** `target/doc/` (Rust), `_build/` (Sphinx), `site/` (mkdocs), `.docusaurus/`, `_site/` (Jekyll) — none match the `historical` scope's `.planning/archive/**` glob. Post-suggestion the owner should add an `ignore: true` scope for build outputs.
- **Vendored markdown.** Third-party docs imported via git-subtree or vendor-copy. Often shouldn't be graded subjectively (they describe an external thing). Post-suggestion add a `vendor` scope with `count_in_coverage: false`.
- **Multi-language docs** (`docs/zh/`, `docs/ja/`, `docs/de/`). Each language is a separate forecast — the L3 judge prompt assumes English. Either restrict scope to `docs/` (English only) or define per-language scopes.
- **Symlinks and submodules.** Walker may discover edges across symlinks; submodule contents are usually out-of-scope.

The suggestion is a starting point, not a destination. Plan for ~30 minutes of post-suggestion config refinement before committing.

### Output shape

```toml
# .cross-link-fidelity — suggested by `cross-link suggest-scopes` on 2026-05-08
# Review and edit before committing. Total edges classified: 233.

schema_version = "1.0.0"

[scopes.default]
max_level = "L1"
enforcement_mode = "warn"
tags = ["pre-push"]

[scopes.anchor-readme]
source_glob = "**/README.md"
target_glob = "**/*"
max_level = "L3"
enforcement_mode = "warn"  # promote to block-stale once coverage > 30%
count = 14  # for owner sanity-check

[scopes.nav-only]
source_glob = "**/*.md"
target_glob = "**/*"
source_section_regex = "(?i)^see also$"
max_level = "L1"
count = 47

[scopes.historical]
source_glob = ".planning/archive/**"
target_glob = "**/*"
max_level = "L0"
count_in_coverage = false  # don't penalize coverage metric for archived content
count = 31
```

The `count = N` field is informational (machine-stripped on next CLI run); it lets the owner sanity-check that the heuristics matched what they expected.

## Coverage as social pressure (the badge mechanism)

Codecov works because the badge in the README publicly tracks the number. Cross-link-fidelity adopts the same trick.

Tracker emits a shields.io endpoint badge per scope:

```
quality/reports/badges/cross-link-fidelity-default.json
{
  "schemaVersion": 1,
  "label": "fidelity @ L3",
  "message": "87%",
  "color": "green"
}
```

Color thresholds: red <50%, yellow 50–80%, green 80–95%, brightgreen ≥95%.

README badge: `![fidelity](https://.../endpoint?url=raw.githubusercontent.com/.../cross-link-fidelity-default.json)`.

**Why the badge matters:** for adopters in `warn` enforcement mode, the badge IS the enforcement. The number doesn't go down (ratcheting); contributors see it climb week-over-week; nobody wants to be the one whose PR makes it red.

## The ignore-vs-archive distinction

Two flavors of "exclude this":

1. **`ignore: true`** — don't even discover edges in this scope. They don't exist as far as the gate is concerned. Use for generated docs, `target/`, `node_modules/`.

2. **`count_in_coverage: false`** — discover edges, run L0/L1 (link still has to resolve), but exclude from coverage @ L3 metric. Use for archived planning, historical research dirs. Edges are still BROKEN-checkable; they just don't pull the coverage metric down.

The distinction matters because archive dirs often have many cross-references that you legitimately don't want to grade subjectively, but you DO want to know if a link to `docs/concepts/dvcs-topology.md` from an archived plan is still resolving.

## Two failure modes the brownfield design avoids

1. **The "100% on day 1" failure.** Adopting the gate requires either: (a) a megacommit grading every edge, blocking all work for hours, OR (b) shipping with `enforcement_mode: warn` and never promoting because nobody owns the coverage push. The ratchet mechanism + `block-broken` initial enforcement gives a useful middle ground that grows over time.

2. **The "stuck at warn forever" failure.** If the gate is *only* warn-mode, contributors ignore it. The `block-broken` baseline is cheap (BROKEN is always wrong) and gives the gate teeth from day 1. The ratchet ensures coverage can't silently regress while warn-mode is the only enforcement at the L3 level.

## Floor reset semantics

`cross-link reset-floor <scope> --reason "<text>"` is the only verb that lowers a floor. Specification:

- **What it does:** sets the scope's `coverage_floor` to current measured coverage @ L3, regardless of prior value.
- **Required flag:** `--reason "<text>"` (free-form; minimum 20 characters; written to tracker as audit entry).
- **Audit trail:** writes a row to the tracker's `audit_events` array with timestamp, scope, prior_floor, new_floor, reason, git_user. PR review surfaces the diff.
- **Permissions:** v1 has no permission system; the verb runs for whoever can git-commit. v1.1 may add `--approved-by <github-user>` requirement (sketched in [`08-open-questions.md`](./08-open-questions.md) Q8).
- **When to use:** scope rename redistributes edges, large file deletion shifts denominator, intentional curriculum change ("we're rewriting the docs site").
- **When NOT to use:** "I don't want to fix the failing edges." Reset-floor is for measurement-validity events, not for shedding work.

## Stuck-at-block-broken recovery

The "stuck at warn forever" failure mode (chapter ends with this) is the obvious one. The less-obvious failure is **stuck at `block-broken`** — coverage @ L3 plateaus because the team can't keep up with refresh, badge stays yellow, no momentum to promote.

**Diagnostic questions** (run through these if the floor isn't moving for >2 weeks):

1. **Is the cron firing?** `cross-link status --cron-health` checks last-run-at vs schedule. If stale, fix CI.
2. **Are batches succeeding?** If `plan-refresh --batch 20` keeps producing 20 BLOCK verdicts that nobody addresses, the team needs a triage rotation.
3. **Are the BLOCKs actually correct?** If 80% of BLOCK verdicts are "judge wants more detail than the README needs," the `grading_context` for that scope is too strict. Loosen it (target frontmatter `grading_context` is the lever).
4. **Is the scope too broad?** A `default` scope at L3 catches everything; split high-stakes (anchor-readme) from low-stakes (inline references) so the team can prioritize.

**Escape hatches:**

- `cross-link mark-broken --bulk --reason "deferred to v0.14.x doc rewrite"` tombstones a chunk of edges with audit entries.
- Per-scope `enforcement_mode = "warn"` temporarily, while the team builds capacity. Coverage badge still ratchets visibly; teeth gone but visibility preserved.
- `freshness_ttl: 90d` (instead of 30d) reduces re-grade pressure for stable scopes.

The chapter's earlier promotion ladder (warn → block-broken → block-stale → block-floor → block-newedge) is a guide, not a contract. Some scopes will live at `block-broken` forever, and that's fine.

## Open questions specific to brownfield

(Full open-question list in [`08-open-questions.md`](./08-open-questions.md).)

- **Cross-scope coverage aggregation.** Is there a project-aggregate floor across scopes, or only per-scope? Probably per-scope only — different scopes have different cost/value profiles.
- **What if the badge URL embeds drift?** If the badge is `87% → 86%`, do we cache-bust automatically, or does the floor mechanism prevent this?
- **Multi-stakeholder onboarding.** A repo with 30 maintainers has no single "owner" to commit the suggested config. v1 punts; v1.1 may add a `cross-link propose-config` PR-creation flow.
