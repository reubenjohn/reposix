# 02 — Architecture

> **Read first.** [`01-vision-and-problem.md`](./01-vision-and-problem.md) for the why.
> **Read next.** [`03-schemas.md`](./03-schemas.md) for concrete config + tracker shapes.

## Mental model: the graph is the unit

The project's docs form a directed graph. Each node is a file (mostly markdown, but targets can be any file type). Each edge is one markdown link `[text](path#anchor?)` from a source doc to a target.

> **Key invariant.** The grading unit is the **edge**, not the node. A single file participates in many edges as both source and target; a single edit can stale many edges with one keystroke.

## The four stored primitives + one derived

| Primitive | What it is | Stored where |
|---|---|---|
| **Edge** | A `(source_path, source_anchor?, target_path, target_anchor?)` tuple discovered by markdown-AST walk | Tracker (machine) |
| **Scope** | A glob pattern set + level + tags + grading-defaults that governs a class of edges | Config (human) |
| **Tracker** | Per-edge `last_graded_hash` + `last_graded_at` + `last_verdict` + `last_judge_rationale` | `quality/state/cross-link-fidelity-tracker.json` (machine-managed, git-tracked) |
| **Config** | Scope definitions + defaults + grading-context overrides + enforcement mode | `.cross-link-fidelity` (human-authored, git-tracked) |
| **Edge state** *(derived)* | One of `UNGRADED \| GRADED \| STALE \| BROKEN` — computed every walk from `last_graded_hash` vs `current_target_hash` | Not stored; recomputed each walk |

Tracker = state. Config = policy. They never blend.

## Edge identity

> **Edge ID is path-derived.** This is load-bearing for the ratchet and for state preservation across edits.

```
edge_id = sha256(source_path || source_section_path || target_path || target_anchor)
```

Where:
- `source_path` and `target_path` are repo-root-relative POSIX paths.
- `source_section_path` is the slash-joined heading-text-path of the section containing the link (`null` for top-of-file links).
- `target_anchor` is the rendered mkdocs slug (`null` for whole-file targets).

**Implications:**
- **File rename** changes `target_path` → new edge_id → old edge becomes BROKEN, new edge becomes UNGRADED. Recovery: `cross-link rebind --auto` preserves `last_graded_hash` if target content_hash is unchanged. (See [`05-edge-cases.md`](./05-edge-cases.md) § 3.)
- **Heading rename in source** (where the link sits) changes `source_section_path` → new edge_id. Same recovery path.
- **Section rename in target** changes `target_anchor` → new edge_id. Same recovery path.
- **Whitespace fixes in target body** do NOT change edge_id; they change `current_target_hash` → STALE classification.

This means: **the ratchet preserves grade across content-stable refactors via `rebind --auto`, but treats genuine path/section reorganization as new edges that need re-grading.** That's the right tradeoff.

## Target hash — what gets hashed

The `current_target_hash` driving STALE detection depends on edge shape:

| Edge shape | What gets hashed |
|---|---|
| `[link](file.md)` (whole-file) | Full target file content, frontmatter stripped |
| `[link](file.md#anchor)` (section) | AST subtree rooted at the heading whose mkdocs slug = anchor; canonical (whitespace-normalized) |
| `[link](#anchor)` (intra-doc) | AST subtree rooted at the heading in the SOURCE file (source IS target for intra-doc) |
| `[link](script.sh)` (non-md target) | Full target file content (no AST parse possible) |

Sub-sub-headings nest: hashing `## Foo` includes everything under it until the next heading at depth ≤ 2. Hashing `### Bar` (under `## Foo`) only includes the `### Bar` subtree.

## Edge states — the brownfield-friendly primitive

Every edge sits in exactly one state at any time. The state taxonomy is the load-bearing answer to "how does an existing repo with 233 ungraded edges adopt this gate without a megacommit?"

| State | Means | Pre-push behavior |
|---|---|---|
| **GRADED** | L3-graded; target hash unchanged since grade | Pass |
| **STALE** | Was L3-graded; target hash drifted since grade | BLOCK (refresh or rebind required) |
| **BROKEN** | L0/L1 fail (link target or anchor unresolved) | BLOCK always — cheapest to fix |
| **UNGRADED** | Never been L3-graded | Pass (brownfield baseline) — but see "new-edge contract" below |

**The new-edge contract (asymmetry).** Edges *introduced in this PR* are held to the scope's `max_level` immediately — UNGRADED-on-arrival is rejected for L3 scopes, because new content has no excuse for being ungraded. Edges that already existed pre-adoption stay UNGRADED until their natural turn (`plan-refresh` cron or owner-driven sweep).

**Interaction with `max_l3_per_push` cap.** `max_l3_per_push` is **per-scope**, not project-wide (ADR-24): each scope owns its own budget (default 10), and budget exhaustion stays local to the offending scope. A PR adding 15 new edges to a single L3-scope hits *that scope's* cap of 10. Three options were considered:
- (a) BLOCK with directive to split the PR.
- (b) Batch-grade the first 10, mark the rest `STALE_PENDING`, BLOCK push.
- (c) Raise the cap for PR-introduced edges only.

Chosen: **(a) BLOCK with split-PR directive (ADR-21).** Smaller PRs are easier to review and align with the dark-factory ethos (review the framing, not the volume). A 15-new-edge PR likely does multiple things; splitting clarifies the atomic units. The recovery message names which scope hit its cap so the contributor knows which slice of the PR to peel off — not "the project hit its cap."

**The "NEW-IN-PR" tag.** Edges newly discovered in a walk are tagged with `introduced_in_pr: <branch-name>`. The tag clears when (a) the edge is graded for the first time and reaches GRADED, or (b) the branch lands on main and the next walk on main re-classifies the edge as no-longer-new. Until cleared, the edge is held to the new-edge contract regardless of subsequent edits.

The walker classifies every edge per push:

```
for each discovered edge e:
  if e.target file missing or e.anchor missing:
      e.state = BROKEN
  elif e.id not in tracker:
      e.state = UNGRADED  (and tag NEW-IN-PR if added this push)
  elif tracker[e.id].last_graded_hash != current_hash(e.target_subtree):
      e.state = STALE
  else:
      e.state = GRADED
```

State is derived, not stored. The tracker stores `last_graded_hash` + `last_graded_at` + `last_verdict`; the state falls out of the comparison.

## The ladder of scrutiny

Every edge sits at one of four levels. Each level is **strictly more rigorous and more expensive** than the one below it.

| Level | Check | Kind | Typical cadence | Per-edge cost |
|---|---|---|---|---|
| **L0** | Link target file exists | mechanical | pre-commit | ~0ms |
| **L1** | Anchor `#section` exists in target (mkdocs slug) | mechanical | pre-commit | ~0ms |
| **L2** | Target's content hash unchanged since last grade | mechanical | pre-push | ~0ms |
| **L3** | Sonnet judge: "does the asserter still adequately forecast the target for the stated reader?" | subagent-graded | weekly + on-L2-drift | ~$0.05 + ~30s |

**Each edge declares its `max_level`.** A scope with `max_level: L3` means edges in that scope are graded up to L3 on the configured cadence. A scope with `max_level: L1` only checks resolves + anchors.

**This project's defaults** (per owner direction):
- `default` scope: `max_level: L3`. Dark-factory ethos rejects "default to mechanical-only."
- Per-edge downgrade is allowed via scope overrides (e.g., nav-only "see also" lists → L1).

## Scope-only configuration model

Owner direction: there is no "global" config. **Everything is scope-specific.** A `default` scope exists for boilerplate reduction.

### Scope semantics

A scope matches edges by source-glob + target-glob (both required, both can be `**/*` for "all"). An edge resolves to a scope by **last-match-wins** in the config file (gitignore mental model — users already know it).

The `default` scope matches `source: "**/*", target: "**/*"` and provides defaults that named scopes inherit field-by-field.

### Field-level inheritance

A named scope only declares fields it overrides; everything else falls through to `default`. This is the "boilerplate reduction" the owner asked for.

For the canonical config schema (field names, types, defaults, semantics) and a working example see [`03-schemas.md`](./03-schemas.md) § "Config schema (TOML)" + [`examples/default-config.toml`](./examples/default-config.toml). Conceptually: scopes carry `tags` (orchestration-agnostic — see ADR-23), `max_level`, `enforcement_mode`, glob fields, and grading-context references. The `default` scope is the field-fallthrough source.

### Why scope-only > directory-walk

The original sketch had `.cross-link-fidelity` files at any depth, walker-merged on lookup. Scope-only kills the walk: scope membership is glob-driven, deterministic, and visible in one config file. Easier to reason about, easier to audit, easier to extract as a portable tool.

## Three files, three jobs (config / catalog / tracker)

Per ADR-25, three separate files. Full schemas + paths in [`03-schemas.md`](./03-schemas.md); summary here:

| File | Path | Owner | Purpose |
|---|---|---|---|
| **Config** | `.cross-link-fidelity` | Human | Scopes, levels, tags, grading contexts |
| **Catalog** | `quality/catalogs/cross-link-fidelity.json` | Machine (CLI) | ~4 runner-readable rows; conforms to unified `quality/catalogs/*` row schema |
| **Tracker** | `quality/state/cross-link-fidelity-tracker.json` | Machine (CLI) | Per-edge state (~400 entries); gate-internal; runner does NOT touch it |

All three carry `schema_version` semver. The tracker is git-tracked because PR review wants to see "this PR drifted 14 edges" as part of the diff — like `Cargo.lock`, you read it but you don't edit it.

## Three flavors of grading context

The L3 judge needs grounding to grade well. The grading context lives in three places, merged at grading time as `target ⊕ edge ⊕ source`.

| Owner | Where it lives | When to use |
|---|---|---|
| **Target** | YAML frontmatter at top of target doc, key `cross_link_fidelity.grading_context` | Default — "what reader/scope this doc serves." Travels with the doc. |
| **Edge** | Tracker file, optional per-edge `grading_context_override` | Edge-specific lens (e.g., "from this anchor README, give enough to decide whether to read in full") |
| **Source** | Config file, scope-level `grading_context` | Index pages with consistent linking style |

Concrete frontmatter shape:

```yaml
---
cross_link_fidelity:
  grading_audience: "agent planning a v0.13.2 phase"
  grading_context: |
    This doc teaches the architecture of the cross-link-fidelity gate.
    Inbound edges should forecast the four primitives, the scrutiny ladder,
    and the config-vs-tracker split.
---
```

**Why namespaced (`cross_link_fidelity:`) not bare (`grading_context:`):** owner-flagged correctly — `grading_context` is generic enough to collide with other tools' frontmatter conventions. The namespace prefix is load-bearing.

**Why frontmatter for target context, not code comments:** comments rot, can't be grepped reliably, get auto-stripped by formatters. Frontmatter travels with the doc, fits this project's existing `frontmatter+body` mental model (issues already use it).

## What is an edge?

> **Lock this down narrow at v1; broaden later.**

An edge is an explicit `[text](path#anchor?)` markdown link in a markdown file. That's it.

Excluded at v1:
- References in code comments
- mkdocs nav (lives in `mkdocs.yml`, separate concern; owned by `docs-build` dimension)
- Frontmatter `see_also` fields
- Implicit references like "see the architecture doc"
- HTML `<a>` tags in markdown (rare in this project; revisit if it shows up)

Included at v1:
- Relative paths: `[link](./other.md)`, `[link](../README.md)`
- Anchor refs: `[link](./other.md#section)`
- Bare anchor refs: `[link](#section)` (intra-doc)
- Non-md targets: `[link](../scripts/foo.sh)`, `[link](../config.toml)` — graded same way, just no AST hash on target

## Edge type auto-classification (v2 idea)

The owner asked about auto-classifying nav-only vs fidelity-claim edges. Heuristic v2 idea (do not implement at v1):

- Edge is in a list under a heading matching `/^see also$/i`, `/^references$/i`, `/^further reading$/i` → auto-downgrade to L1, `kind: nav-only`.
- Edge is in inline prose (not a list) → keep at scope-default (usually L3).
- Edge is the only content of an italic line ("*See also: [foo](./foo.md).*") → L1.

These would land as a separate scope built into the binary, not user-config — but the user can override via explicit scope rules.

## Cost model (sanity-check)

Project baseline (measured 2026-05-08):
- 49 docs/ markdown files
- 233 md→md edges in docs/
- 25 tracked READMEs project-wide
- 134 relative-parent edges (`../*`)
- 7 intra-doc anchor refs

Estimated total edge count after walker: **~400 edges**.

**Per-edge unit costs (Sonnet, observed range; varies with grading_context size and target subtree size):**
- p50: ~$0.03 (5k tokens in, 250 tokens out)
- p95: ~$0.08 (12k tokens in, 400 tokens out)
- worst-case: ~$0.15 (full file, 20k tokens in)

**Monthly steady-state cost components:**
- Drift-triggered re-grades (steady-state): ~30 edges/month × $0.05 avg = **~$1.50**
- Weekly TTL refresh (≥30 days since last grade): ~100 edges/month × $0.05 = **~$5**
- Bootstrap tail (UNGRADED→GRADED, after initial bootstrap): ~50 edges/month × $0.05 = **~$2.50**
- Retry overhead (Anthropic timeouts, 5–10% retry rate): **~$1**

**Total: ~$10/month** at default cadence, post-bootstrap (~$50/month was the original estimate; observed steady-state is closer to $10 because most edges don't drift weekly).

**One-time bootstrap cost** (separate from monthly):
- 200-edge graph: ~$10
- 400-edge graph: ~$20 (this project)
- 1500-edge graph: ~$75
- 10k-edge graph: ~$500 — at this scale, mandate `--batch` over weeks rather than `--all`.

**Per-push (drift) cost:** typical commit drifts 0–5 edges → ~$0.15. Worst case (refactor): ≤ `max_l3_per_push × p95` = 10 × $0.08 = $0.80.

This sits well under the project's existing Sonnet spend on subjective-rubrics. **The cost is not the bottleneck. The judge's signal-to-noise is.**

## Ratcheting coverage — the brownfield enforcement model

> **Coverage is not a hard target. It is a floor that only goes up.**

Per scope, the tracker records `coverage_floor: float` (= max-historical-coverage @ L3). The pre-push gate's contract is:

- A push is **allowed** if it doesn't decrease any scope's coverage @ L3 below its current floor.
- The floor advances automatically whenever coverage @ L3 is observed higher post-push.
- A push that *increases* coverage advances the floor immediately; future pushes can't drop below it.

This means: brownfield repos start at floor=0% and climb. Greenfield repos start at floor=100% on day 1 (because every PR-introduced edge gets graded — see the new-edge contract).

Optional per-scope tightening: `floor_increment_per_pr: 0.01` requires every PR to raise coverage by ≥1%. Off by default; opt-in for projects that want forcing pressure.

### Ratchet edge cases

The "floor only goes up" claim is naive without policy on edge-population-change events. Policy:

| Event | Floor behavior |
|---|---|
| **Pure deletion** of GRADED edges | Floor recomputed against new denominator; if ratio drops, floor *does NOT* move down. Floor is preserved at its prior value. The push passes IFF current coverage ≥ prior floor. |
| **Pure deletion** of UNGRADED edges | Coverage ratio *rises*; floor advances with it. |
| **File move / scope-membership shift** | Edges' scope membership changes per glob match. Old scope's floor is preserved (locked); new scope's floor starts from current measured. Use `cross-link rebind --auto` to preserve grade across path moves. |
| **Scope rename** (config edit) | Old scope's floor is retired (audit-logged); new scope inherits as fresh start UNLESS user sets `floor_initial: <prior_value>` explicitly during the rename. |
| **Bulk file move** (e.g., `docs/` → `docs/v1/`) | Without `rebind --auto`, every edge becomes BROKEN+UNGRADED. With it, edges preserve grade IFF target content_hash unchanged. The `--auto` is the dark-factory-aligned move; manual review of the rebind diff is the safety net. |
| **`reset-floor <scope> --reason "<text>"`** | Explicit owner action. Floor resets to current measured. Audit-log entry written to tracker; PR diff makes it visible. |

The contract: **floor is never silently lowered.** Every floor-decrease requires an explicit verb.

## Coverage badge — the social-pressure mechanism

Tracker emits per-scope shields.io endpoint badge JSON files at `quality/reports/badges/cross-link-fidelity-<scope>.json`. Adopters embed:

```markdown
![fidelity](https://img.shields.io/endpoint?url=raw.githubusercontent.com/<org>/<repo>/main/quality/reports/badges/cross-link-fidelity-default.json)
```

Color thresholds: red <50%, yellow 50–80%, green 80–95%, brightgreen ≥95%.

**Why the badge is load-bearing.** For adopters in `warn` enforcement mode, the badge IS the enforcement. The number doesn't go down (ratcheting); contributors see it climb week-over-week. This is how Codecov drove its adoption — the README badge is more visible than CI failures.

**Gaming-vector mitigation.** The naive failure mode: contributors avoid touching ungraded edges to keep the floor stable. To detect this:

- A **secondary "freshness sub-metric"** (sketched, deferred to v1.1): `% of edges modified in the last 30 days that were re-graded`. A repo with 95% coverage but 0% freshness is a smell.
- The PR comment from `cross-link report-pr` surfaces "edges in this PR" with their state transitions — making contributors visibly accountable for the edges they touched.

## Phased enforcement modes

A scope's `enforcement_mode` controls how strictly the gate enforces. Adopters progress phase-by-phase as confidence and coverage build:

| Mode | What blocks | When to use |
|---|---|---|
| `warn` | Nothing — emits warnings only | Day 1 of brownfield adoption; tracker still populates |
| `block-broken` | Only BROKEN edges block | Cheap baseline; BROKEN is always wrong regardless of L3 maturity |
| `block-stale` | BROKEN + STALE block | Coverage is meaningful; drift detection is on |
| `block-floor` | BROKEN + STALE + coverage-regression block | Steady-state for projects that own their floors |
| `block-newedge` | All of `block-floor` + reject UNGRADED-NEW-IN-PR for L3 scopes | Maximum rigor; this project's target |

This project's plan: bootstrap to ≥80% L3 coverage in CI, flip the `default` scope to `block-newedge` once stable. Other adopters can stay at `warn` or `block-broken` indefinitely.

Modes are config-side sugar: inside reposix they compile at walk-time to per-row `blast_radius` (P0/P1/P2) on the catalog rows the scope owns; the standalone tool drives the CLI exit code directly from modes. See [`03-schemas.md`](./03-schemas.md) § "`enforcement_mode` is config-side sugar" for the mapping table.

## Bootstrap and steady-state — CI-only vs local-only operations

Two distinct phases, each with strict cost-asymmetry rules:

### Bootstrap (CI-only or owner-only)

`cross-link bootstrap` walks the entire graph, populates the tracker, dispatches L3 judges for every in-scope edge. Lands as one or more commits over time.

**Why CI-only:** L3 bootstrap on a 400-edge graph is ~$12 in Sonnet calls and ~30 minutes wall-clock. Running this on a contributor's laptop pre-push is unacceptable. Contributors run only L0+L1+L2+drift-triggered-L3 (typically 0–5 edges per commit, ~$0.15).

**Two flavors:**

1. **`cross-link bootstrap --all`** — owner-run, one-shot, grades every UNGRADED edge in scope. Used at gate-adoption time.
2. **`cross-link plan-refresh --batch N --age-min Xd`** — incremental, cron-driven. Grades the N oldest-graded (or N UNGRADED) edges per run. Coverage climbs gradually, cost amortized to ~$0.50/day. **Cron lives in CI** (GitHub Actions `schedule:` trigger or equivalent). Running `plan-refresh` locally requires explicit `--local` flag with a cost-confirmation prompt; otherwise the verb refuses outside `CI=true` to prevent accidental burn.

Bootstrap MUST complete before flipping enforcement_mode past `block-broken`. Otherwise first push after gate-merge BLOCKs every UNGRADED edge.

### Steady-state (every push)

`cross-link walk` runs as part of pre-push:

1. Discover all edges via markdown-AST walk.
2. Classify each: GRADED / STALE / BROKEN / UNGRADED.
3. Apply scope's `enforcement_mode` to decide BLOCK / PASS.
4. For STALE edges (and NEW-IN-PR UNGRADED at L3 scopes): dispatch L3 judges, capped at `max_l3_per_push` (default 10). Overage BLOCKs with explicit `cross-link plan-refresh --since HEAD~1` recovery directive.

The cap is the load-bearing protection. Without it, a content-heavy edit cascades 50 judgements at push time and the contributor's pre-push hook hangs for 25 minutes.

## What lives where

Per ADR-25, the catalog (runner-readable) and the tracker (gate-internal per-edge state) live in different files:

```
.cross-link-fidelity                         # human-authored TOML
.cross-link-fidelity/                        # OR folder form for complex configs
├── config.toml                              #   main config
├── scopes/
│   └── anchor-readmes.toml                  #   per-scope split-out
└── grading-context/
    └── doc-name.md                          #   source-side grading context (rare)

docs/concepts/dvcs-topology.md               # target-side context in frontmatter
└─ frontmatter: cross_link_fidelity.grading_context

quality/catalogs/cross-link-fidelity.json    # catalog (~4 runner-readable rows; ADR-25)
quality/state/cross-link-fidelity-tracker.json  # tracker (gate-internal, not runner-discovered)

quality/gates/cross-link-fidelity/           # gate dimension home
├── README.md
├── walk.sh                                  # pre-push entry point
├── bootstrap.sh                             # one-shot bootstrap
└── verifiers/                               # per-level verifier scripts
    ├── l0-link-resolves.sh
    ├── l1-anchor-resolves.sh
    ├── l2-hash-drift.sh
    └── l3-fidelity-judge.sh

crates/reposix-quality/                      # CLI verbs (or scripts/cross-link.py for v1?)
└── src/cross_link/                          # see 07-extraction-plan.md for Rust-vs-Python decision
```

## Architectural decisions made

(Full log in [`06-decisions-log.md`](./06-decisions-log.md).)

- ADR-1: Edges, not nodes, are the grading unit.
- ADR-2: Four-level scrutiny ladder; level is per-scope.
- ADR-3: Scope-only configuration; no global config; `default` scope exists.
- ADR-4: Field-level inheritance from `default` (gitignore last-match-wins for scope ordering).
- ADR-5: Tracker (machine state) and config (human policy) are separate files.
- ADR-6: Three-flavor grading context (target ⊕ edge ⊕ source); namespaced under `cross_link_fidelity:` in frontmatter.
- ADR-7: An edge is an explicit markdown link in a markdown file. Code comments and mkdocs nav are out of scope at v1.
- ADR-8: Project default `max_level` is L3 (dark-factory ethos).
- ADR-9: Per-push L3 cap with explicit-out-of-band-refresh BLOCK on overage.
- ADR-10: Bootstrap is a separate CLI verb; pre-push enforcement requires bootstrap progress past floor.
- ADR-11: Edge state taxonomy (UNGRADED/GRADED/STALE/BROKEN) is the brownfield-friendly primitive; UNGRADED is a legitimate baseline state, not a defect.
- ADR-12: Ratcheting coverage floor (monotonic, per-scope) is the enforcement mechanism, not a hard target.
- ADR-13: Phased enforcement modes (`warn` → `block-broken` → `block-stale` → `block-floor` → `block-newedge`); adopters progress gradually.
- ADR-14: New-edge contract — edges introduced in a PR are held to scope's `max_level` immediately, regardless of repo brownfield state.
- ADR-15: Bootstrap is CI-only / owner-only; pre-push runs only L0+L1+L2+drift-triggered-L3 (cost asymmetry).
- ADR-16: Coverage badge (shields.io endpoint) is a first-class output — the social-pressure mechanism for ratcheting.
