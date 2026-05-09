# 03 — Schemas

> **Read first.** [`02-architecture.md`](./02-architecture.md) for the conceptual model.
> **Read next.** [`04-cli-and-workflow.md`](./04-cli-and-workflow.md) for verbs that read/write these schemas.
> **Concrete examples.** See [`examples/`](./examples/) for working files.

## Four schemas, four owners

| Schema | Lives in | Owner | Format |
|---|---|---|---|
| **Config** | `.cross-link-fidelity` (file or folder) | Human | TOML |
| **Catalog** | `quality/catalogs/cross-link-fidelity.json` | Machine (CLI) | JSON, runner-readable |
| **Tracker** | `quality/state/cross-link-fidelity-tracker.json` | Machine (CLI) | JSON, gate-internal |
| **Frontmatter** | YAML frontmatter at top of target docs | Human (doc author) | YAML |

> **Why catalog and tracker are separate (ADR-25).** The reposix runner at `quality/runners/run.py` discovers every JSON in `quality/catalogs/` and expects the unified row schema (`id`, `dimension`, `kind`, `cadences`, `expected.asserts`, `verifier`, `artifact`, `status`, `blast_radius`). A 400-edge tracker file collides with that schema and would crash discovery. Solution: the **catalog** at `quality/catalogs/cross-link-fidelity.json` carries ~4 runner-readable rows (one per scrutiny level + one for floor-not-decreased), each pointing at `quality/gates/cross-link-fidelity/walk.sh --level Lx`. The runner reads the catalog. The **tracker** at `quality/state/cross-link-fidelity-tracker.json` (or whatever the gate-internal location is — final path settled in P98 PLAN.md) holds the per-edge state. The verifier reads the tracker.

All four carry an explicit `schema_version` field (semver per ADR-1). Versioning is the load-bearing answer to "this becomes a standalone tool one day" (see [`07-extraction-plan.md`](./07-extraction-plan.md)).

## Config schema (TOML)

The `.cross-link-fidelity` file (or `.cross-link-fidelity/config.toml` for the folder form). One file per repo at the root.

> **Design note (post-research scrutiny, 2026-05-08).** Two structural decisions land below as ADR-23 (tag model over cadence-baked) and ADR-24 (per-scope `max_l3_per_push`; `walk_paths` derived from scope globs). See [`06-decisions-log.md`](./06-decisions-log.md) for rationale.

```toml
# Schema version. Bumped only on breaking changes; semver per ADR-1's strict-from-day-1 stance.
schema_version = "1.0.0"

# OPTIONAL: project-wide *path* settings only. Discovery is derived from scope globs (ADR-24);
# there is intentionally no `walk_paths` allowlist here — see ADR-24 rationale.
[project]
catalog_path = "quality/catalogs/cross-link-fidelity.json"     # runner-readable; ~4 rows
tracker_path = "quality/state/cross-link-fidelity-tracker.json"  # gate-internal; per-edge state
badge_path = "quality/reports/badges/"

# REQUIRED: the default scope. Provides field-level fall-through for named scopes.
# Tags here are the project default the user OPTS INTO at orchestration time
# (e.g. `cross-link walk --tags pre-push`); see ADR-23.
[scopes.default]
source_glob = "**/*.md"
target_glob = "**/*"
max_level = "L3"
tags = ["pre-push", "fidelity-critical"]
enforcement_mode = "block-newedge"
ignore = false
count_in_coverage = true
freshness_ttl = "30d"
floor_increment_per_pr = 0.0  # 0 = no forcing pressure
max_l3_per_push = 10  # per-scope cap; ADR-24

# OPTIONAL: named scopes. Last-match-wins ordering — later scopes override earlier ones.
[scopes.anchor-readme]
source_glob = "**/README.md"
target_glob = "**/*"
# inherits max_level=L3, tags, max_l3_per_push, etc from default
grading_context_override = "anchor-readme"  # references [grading_contexts.anchor-readme] below

[scopes.nav-only]
# section-scoped: only edges UNDER a "See also" heading
source_glob = "**/*.md"
source_section_regex = "(?i)^see also$"  # heading-text regex (renamed from source_section_pattern; ADR-23 hygiene)
max_level = "L1"
tags = ["pre-push"]
max_l3_per_push = 0  # never spend L3 budget on nav links

[scopes.historical]
source_glob = ".planning/archive/**"
target_glob = "**/*"
max_level = "L0"
count_in_coverage = false  # archived content doesn't pull coverage down
tags = ["weekly"]  # only swept by the weekly cron, not pre-push

[scopes.generated]
source_glob = "**/CHANGELOG.md"
target_glob = "**/*"
ignore = true  # don't even discover

# OPTIONAL: source-side grading context. Referenced by scope.grading_context_override.
[grading_contexts.anchor-readme]
audience = "agent navigating the doc tree for the first time"
context = """
This is a README that is expected to forecast its children.
The reader trusts the README to summarize what the children teach.
A failure mode is the README mentioning a concept the children no longer cover,
or the children covering a concept the README never hints at.
"""

# OPTIONAL: per-edge overrides. Key MUST match the tracker edge-ID format
# `<source_path>@<source_section|root>-><target_path>@<target_anchor|root>` (ADR-23 hygiene D-3).
# Use sparingly — most overrides should live at scope level.
[edge_overrides."docs/README.md@root->docs/concepts/dvcs-topology.md@root"]
max_level = "L1"
reason = "navigation pointer; concept doc is referenced from architecture chapter instead"
```

### Field semantics

| Field | Type | Default | Notes |
|---|---|---|---|
| `schema_version` | string (semver) | required | "1.0.0" today; bumped on breaking changes. |
| `project.catalog_path` | string | `"quality/catalogs/cross-link-fidelity.json"` | Runner-readable catalog (~4 rows) per ADR-25. |
| `project.tracker_path` | string | `"quality/state/cross-link-fidelity-tracker.json"` | Gate-internal per-edge tracker per ADR-25. Path-only; no `walk_paths` field — discovery derives from scope globs (ADR-24). |
| `project.badge_path` | string | `"quality/reports/badges/"` | Output dir for `cross-link badge`. |
| `scopes.<name>.source_glob` | glob | required for non-default | Source path glob. |
| `scopes.<name>.target_glob` | glob | `**/*` | Target path glob. |
| `scopes.<name>.source_section_regex` | regex | none | If set, edge must be UNDER a heading whose text matches. Section-scoped. |
| `scopes.<name>.max_level` | enum | inherit from default | `L0` / `L1` / `L2` / `L3`. |
| `scopes.<name>.tags` | list[string] | inherit | Free-form labels; used by `--tags` filter. Conventional values: `pre-commit`, `pre-push`, `weekly`, `on-demand`, `fidelity-critical` (see ADR-23). |
| `scopes.<name>.enforcement_mode` | enum | inherit | `warn` / `block-broken` / `block-stale` / `block-floor` / `block-newedge`. |
| `scopes.<name>.ignore` | bool | false | Skip discovery entirely. |
| `scopes.<name>.count_in_coverage` | bool | true | Include in coverage @ L3 metric. |
| `scopes.<name>.freshness_ttl` | duration | "30d" | After this, GRADED → STALE even without hash drift. |
| `scopes.<name>.floor_increment_per_pr` | float [0,1] | 0.0 | Per-PR coverage delta requirement. `0.0` is the documented opt-out sentinel. |
| `scopes.<name>.max_l3_per_push` | int | 10 | Per-scope cap on L3 calls per push; overage BLOCKs with split-PR directive (ADR-21 + ADR-24). |
| `scopes.<name>.grading_context_override` | string | none | References a `[grading_contexts.<name>]` block. |

### Discovery is derived, not declared

There is no `[project].walk_paths` field. Discovery walks the union of all non-`ignore` scopes' `source_glob`s. This deletes a former precedence footgun (top-level allowlist vs scope `source_glob` could disagree silently). If you want to exclude a path entirely, either give it a scope with `ignore = true` or omit it from every `source_glob`. See ADR-24.

### Tag-filter, not cadence-baked (config side)

Scopes carry `tags: list[string]`, not a single `cadence` enum. The CLI accepts `--tags <a,b>` and `--exclude-tags <c>`; the user wires the filter into whatever orchestration they prefer (`pre-commit`, `prek`, Claude Code hooks, GitHub Actions, Bazel, plain cron, an MCP tool, an in-house bash script). The framework does not assume an orchestrator. See ADR-23.

### `enforcement_mode` is config-side sugar

The five modes (`warn` / `block-broken` / `block-stale` / `block-floor` / `block-newedge`) are **framework concepts** living on scope blocks. They are user-friendly progression knobs.

For the **standalone tool** (no reposix runner), modes drive the CLI's exit code directly: a scope at `block-stale` makes `cross-link walk` exit non-zero when any STALE edge is detected.

For **reposix specifically**, modes compile down at walk-time to per-catalog-row `blast_radius`:

| Scope `enforcement_mode` | Effect on rows the scope owns |
|---|---|
| `warn` | All rows: `blast_radius: P2` (logs in CI but doesn't block). |
| `block-broken` | L0+L1 rows: `P0`; L3 + floor: `P2`. |
| `block-stale` | L0+L1+L3-stale rows: `P0`; floor: `P2`. |
| `block-floor` | All except newedge: `P0`. |
| `block-newedge` | All rows including newedge gate: `P0`. |

The framework does not duplicate blast_radius — it generates it. The catalog rows shipped in `quality/catalogs/cross-link-fidelity.json` are written WITH the post-mapping `blast_radius` field already filled in, by the gate's `cross-link bootstrap` verb. Hand-edited modes update the config; the next walk regenerates the catalog rows. This is the same shape as `enforcement_mode` translating to `blocking: true` in any other quality gate that uses progressive enforcement.

### Two layers — framework vs reposix integration

The framework that ships with the gate (the Rust binary + the config schema) is **orchestration-agnostic**:

- **Rust CLI** (`reposix-quality cross-link {walk, plan-refresh, ...}`). Takes `--tags <a,b>` and `--exclude-tags <c>`. Reads `.cross-link-fidelity`. Emits artifacts. Knows nothing about pre-commit, runners, or catalogs.
- **Config (`.cross-link-fidelity`)**. Defines scopes and their `tags`. User-authored. Travels with the standalone tool.

That's the entire framework surface. An adopter using `prek` writes:

```yaml
# .pre-commit-config.yaml or equivalent
- id: cross-link-l0-l1
  entry: reposix-quality cross-link walk --level L1 --tags pre-commit
  stages: [pre-commit]
```

An adopter using Claude Code hooks writes:

```json
// .claude/settings.json
{ "hooks": { "preToolUse": [{ "matcher": "Edit", "command": "reposix-quality cross-link walk --tags pre-edit" }] } }
```

An adopter using GitHub Actions writes a workflow step. Same CLI; orchestration of the user's choice.

**Reposix-specific integration** (NOT shipped with the standalone tool):

- **Catalog at `quality/catalogs/cross-link-fidelity.json`** is reposix's adapter to its own runner. Each row encodes "when reposix's runner fires for cadence X, invoke the framework CLI with these flags." `cadences: list[str]` on those rows is a typed enum validated at `quality/runners/run.py:45-47`; that's a reposix runner concern, not a framework concern.
- **Mapping from scope `tags` → catalog `cadences`** lives in those catalog rows' `verifier` fields (e.g., `walk.sh --tags pre-push`). The catalog acts as the bridge between the user-friendly tag convention and reposix's specific cadence taxonomy.

Mental model: **the framework owns `--tags`; reposix's runner owns `cadences`; the catalog is the bridge.** Adopters of the standalone tool ignore `cadences` entirely and wire the CLI directly into whatever orchestration they have.

### Glob conventions

- Use `**/*` for recursive match.
- `*.md` matches at one level only.
- Patterns are evaluated relative to repo root.
- Last-match-wins on scope ordering — gitignore mental model.

### File vs folder priority

If both `.cross-link-fidelity` (file) AND `.cross-link-fidelity/` (folder) exist at the repo root, the **folder wins** and the file is treated as an error (`cross-link audit` BLOCKs). Adopters migrating from file to folder should `git rm .cross-link-fidelity` in the same commit that introduces the folder.

### When to use the folder form

Move from `.cross-link-fidelity` (single file) to `.cross-link-fidelity/` (folder) when:

- Config exceeds ~200 lines.
- Multiple grading-context blocks need long-form prose.
- Per-scope configs benefit from being split for review.

Folder layout:
```
.cross-link-fidelity/
├── config.toml                          # main config + project-level settings
├── scopes/
│   ├── anchor-readme.toml               # one scope per file
│   ├── nav-only.toml
│   └── historical.toml
└── grading-contexts/
    ├── anchor-readme.md                 # long-form context (md, not toml)
    └── concept-doc.md
```

`scopes/*.toml` files contain a single `[scopes.<name>]` block. `grading-contexts/*.md` files are referenced by `grading_context_override = "<filename-without-ext>"`.

## Catalog schema (JSON, runner-readable)

The `quality/catalogs/cross-link-fidelity.json` file. ~4 rows; identical row schema to every other catalog (`docs-alignment.json`, `code.json`, etc.). The runner at `quality/runners/run.py` discovers it; the runner does NOT touch the per-edge tracker. Pattern adapted from `quality/catalogs/doc-alignment.json` summary-wrapper precedent.

```jsonc
{
  "$schema": "https://schemas.reposix.dev/quality-catalog/v1.json",
  "schema_version": "1.0.0",
  "summary": {
    "alignment_ratio": 0.87,         // graded / non-ignored
    "alignment_floor": 0.85,
    "coverage_ratio": 0.94,          // (graded + l2-fresh) / non-ignored
    "coverage_floor": 0.90,
    "edge_count_total": 233,
    "last_walked": "2026-05-08T14:32:11Z",
    "trend_30d": "+0.04"
  },
  "rows": [
    {
      "id": "cross-link-fidelity/l0-link-resolves",
      "dimension": "cross-link-fidelity",
      "kind": "mechanical",
      "cadences": ["pre-commit", "pre-push"],
      "blast_radius": "P0",
      "verifier": "quality/gates/cross-link-fidelity/walk.sh --level L0",
      "artifact": "quality/reports/verifications/cross-link-fidelity/l0-link-resolves.json",
      "expected": { "asserts_passed_min": 1, "asserts_failed_max": 0 },
      "freshness_ttl": null
    },
    {
      "id": "cross-link-fidelity/l1-anchor-resolves",
      "dimension": "cross-link-fidelity",
      "kind": "mechanical",
      "cadences": ["pre-commit", "pre-push"],
      "blast_radius": "P0",
      "verifier": "quality/gates/cross-link-fidelity/walk.sh --level L1",
      "artifact": "quality/reports/verifications/cross-link-fidelity/l1-anchor-resolves.json",
      "expected": { "asserts_passed_min": 1, "asserts_failed_max": 0 },
      "freshness_ttl": null
    },
    {
      "id": "cross-link-fidelity/l2-l3-fidelity-graded",
      "dimension": "cross-link-fidelity",
      "kind": "subagent-graded",
      "cadences": ["pre-push", "weekly"],
      "blast_radius": "P1",
      "verifier": "quality/gates/cross-link-fidelity/walk.sh --level L3",
      "artifact": "quality/reports/verifications/cross-link-fidelity/l3-fidelity.json",
      "expected": { "score_min": 0.85 },
      "freshness_ttl": "30d"
    },
    {
      "id": "cross-link-fidelity/coverage-floor-not-decreased",
      "dimension": "cross-link-fidelity",
      "kind": "mechanical",
      "cadences": ["pre-push", "pre-pr"],
      "blast_radius": "P0",
      "verifier": "quality/gates/cross-link-fidelity/floor-check.sh",
      "artifact": "quality/reports/verifications/cross-link-fidelity/floor-check.json",
      "expected": { "asserts_failed_max": 0 },
      "freshness_ttl": null
    }
  ]
}
```

### Why exactly these rows

- One row per **scrutiny level rung** (L0, L1, L3). L2 hash-drift detection is bundled INTO the L3 row's verifier (drift triggers the L3 dispatch — they're not independent things to fire from the runner).
- One row for the **floor invariant** (coverage_floor monotonic per ADR-22) — separate because it's a property of the tracker, not a per-edge check.
- That's it. The catalog stays small and stable; the per-edge bookkeeping lives in the tracker.

### Cadence/blast-radius mapping comes from the catalog, not the config

The runner fires `cross-link-fidelity/l0-link-resolves` on every `pre-commit` and `pre-push` because the row says `cadences: ["pre-commit", "pre-push"]`. The user's `.cross-link-fidelity` `tags` field on a scope is a *filter applied inside* the verifier, not a runner-discovery field. See "Scope `tags` ≠ runner `cadences`" above.

## Tracker schema (JSON, gate-internal)

The `quality/state/cross-link-fidelity-tracker.json` file. Machine-managed. Git-tracked (so PR reviewers can see drift). Never hand-edited. Not discovered by the runner.

```jsonc
{
  "$schema": "https://schemas.reposix.dev/cross-link-fidelity/tracker-v1.json",
  "schema_version": "1.0.0",
  "generator": "reposix-quality 0.13.2",
  "generated_at": "2026-05-08T14:32:11Z",
  "config_hash": "sha256:abcd1234...",  // hash of .cross-link-fidelity at time of walk
  "scopes": {
    "default": {
      "coverage_at_l3": 0.87,
      "coverage_floor": 0.85,
      "edge_count": 233,
      "graded": 203,
      "stale": 5,
      "broken": 0,
      "ungraded": 25,
      "last_walked_at": "2026-05-08T14:32:11Z"
    },
    "anchor-readme": { /* ... */ }
  },
  "edges": [
    {
      "id": "docs/README.md@root->docs/concepts/dvcs-topology.md@mirror-lag-refs",
      "source_path": "docs/README.md",
      "source_anchor": null,
      "source_section_path": null,
      "target_path": "docs/concepts/dvcs-topology.md",
      "target_anchor": "mirror-lag-refs",
      "scope": "anchor-readme",
      "state": "GRADED",
      "last_graded_hash": "sha256:5678ef...",
      "last_graded_at": "2026-04-22T09:14:00Z",
      "last_verdict": "PASS",
      "last_judge_rationale": "README's framing covers the three roles + lag-ref invariant adequately. No surprises in subsection content.",
      "max_level_reached": "L3",
      "discovered_at": "2026-04-15T12:00:00Z",
      "introduced_in_pr": null,
      "edge_override": null
    },
    {
      "id": "docs/quality/README.md@root->quality/PROTOCOL.md@root",
      "source_path": "docs/quality/README.md",
      "target_path": "quality/PROTOCOL.md",
      "scope": "default",
      "state": "STALE",
      "last_graded_hash": "sha256:1111aaa...",
      "last_graded_at": "2026-04-01T10:00:00Z",
      "last_verdict": "PASS",
      "current_target_hash": "sha256:2222bbb...",
      "drift_detected_at": "2026-05-08T14:32:11Z",
      "max_level_reached": "L3",
      "edge_override": null
    }
  ]
}
```

### Edge ID format

`<source_path>@<source_section|root>-><target_path>@<target_anchor|root>`

Stable across walks IFF source path, source section, target path, and target anchor are unchanged. Renames break the ID — the walker treats this as edge-deletion + edge-addition.

### Why git-tracked

PR reviewers can see "this PR drifted 14 edges" or "this PR ungrade'd 3 edges" as part of the diff. Same rationale as `Cargo.lock`. Diff noise is acceptable; the alternative is invisible drift.

### Walker idempotency

`cross-link walk` is deterministic: same input → same tracker output. Re-running the walker without code/doc changes produces zero diff. This is required for pre-push reliability.

## Frontmatter schema (YAML)

Optional. Lives at the top of any target doc that wants to provide grading context.

```yaml
---
# Existing frontmatter fields can coexist (issue id, version, etc).
id: dvcs-topology
title: DVCS topology

# Cross-link-fidelity grounding for L3 judges that grade inbound edges.
cross_link_fidelity:
  schema_version: "1.0.0"
  grading_audience: "agent planning a v0.14+ phase that touches the cache or helper"
  grading_context: |
    This doc teaches the three DVCS roles (SoT-holder / mirror-only / round-tripper)
    and the mirror-lag refs invariant. Inbound edges from anchor READMEs should
    forecast that a reader can navigate to the right role section without reading
    the whole doc. Inbound edges from how-to guides should at minimum mention that
    mirror-lag refs measure SoT-edit→mirror-sync gap, NOT current SoT state.
  must_forecast:                    # OPTIONAL: explicit list of concepts inbound docs SHOULD mention
    - "three DVCS roles"
    - "mirror-lag refs invariant"
  scope_override:                    # OPTIONAL: pin grading to a specific scope's defaults
    max_level: "L3"
---

# DVCS topology

[doc body]
```

### Field semantics

| Field | Required? | Notes |
|---|---|---|
| `cross_link_fidelity.schema_version` | yes if frontmatter present | "1.0.0" today |
| `cross_link_fidelity.grading_audience` | recommended | One sentence on the intended reader |
| `cross_link_fidelity.grading_context` | recommended | What inbound asserters should be forecasting |
| `cross_link_fidelity.must_forecast` | optional | Hard requirements; judge BLOCKs if any concept not mentioned in source |
| `cross_link_fidelity.scope_override` | rare | Override the scope-resolution; use when a doc is unusually high-stakes |

### Why namespaced (`cross_link_fidelity:`)

Owner-flagged correctly: `grading_context:` at the top level would collide with other tools. Namespacing reserves the prefix for this gate.

### Merge order at grading time

`target_frontmatter ⊕ edge_override ⊕ source_grading_context`

The L3 judge prompt receives all three as separate sections, with the target's frontmatter as the canonical source of truth. Edge overrides supplement; source defaults fill gaps.

## Schema versioning discipline

| Schema | Version field | Bump triggers |
|---|---|---|
| Config | `version` | Renaming/removing a scope field; changing a default that breaks existing configs |
| Tracker | `schema_version` (semver) | Major: edge-id format change. Minor: new fields. Patch: bug fixes. |
| Frontmatter | `cross_link_fidelity.schema_version` (semver) | Same as tracker |

Migration verbs: `cross-link migrate-config v1-to-v2`, `cross-link migrate-tracker 1.0.0-to-2.0.0`. Old versions remain readable for one major release; the binary refuses two majors back.

This discipline matters because the tracker is git-tracked: a schema bump touches every project that's ever adopted the gate. Migrations need to be lossless and human-reviewable.
