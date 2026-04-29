# v0.12.0 — Naming and Architecture

> **Audience.** The agent planning P57 (Quality Gates skeleton) and any phase that adds catalogs/gates/verifiers. Sibling docs in `.planning/research/v0.12.0/*.md`.

## Naming critique — why we're renaming

The owner explicitly flagged: *"end-state.py is a weird way to name the check that validates releases."* They're right. `scripts/end-state.py` does five jobs that have nothing in common except "they happen at session end":

| Current name | What it actually does | Should be |
|---|---|---|
| `scripts/end-state.py` (freshness rows) | static structural invariants | `quality/gates/structure/` |
| `scripts/end-state.py` (crates-io rows) | release-artifact existence | `quality/gates/release/` |
| `scripts/end-state.py` (mermaid rows) | docs-build verification | `quality/gates/docs-build/` |
| `SESSION-END-STATE.json` | conflated catalog | one catalog per dimension under `quality/catalogs/` |
| `scripts/repro-quickstart.sh` | tutorial replay | `quality/gates/docs-repro/tutorial-replay.sh` |
| `scripts/dark-factory-test.sh` | agent-UX regression | `quality/gates/agent-ux/dark-factory.sh` |
| `scripts/check-docs-site.sh` | mkdocs strict | `quality/gates/docs-build/mkdocs-strict.sh` |
| `scripts/banned-words-lint.sh` | code+docs lint | `quality/gates/structure/banned-words.sh` |
| `scripts/green-gauntlet.sh` | composite "run everything" | supplanted by `quality/runners/run.py --cadence pre-pr` |

The word **gate** is load-bearing: it implies blocking, named, owned. The word **dimension** is load-bearing: it gives an owner-caught miss a structural home (CLAUDE.md meta-rule "fix it twice" extends to "and tag the dimension"). The word **cadence** is load-bearing: it routes the gate to the right runner.

`end-state.py` keeps existing — but as a **30-line shim** with header comments warning future agents: "this file does not grow; new gates go under `quality/gates/<dim>/`." Owner-flagged concern (this planning session): without that warning, agents will bloat this file because it's the path of least resistance.

## Directory layout

```
quality/
├── PROTOCOL.md              ← autonomous-mode runtime contract (read at start of every phase)
├── SURPRISES.md             ← append-only journal of unexpected obstacles + resolutions
├── gates/
│   ├── code/                (mostly references ci.yml; cargo + clippy + test live there)
│   ├── docs-build/          mkdocs-strict.sh, mermaid-renders.sh, link-resolution.py
│   ├── docs-repro/          snippet-extract.py, container-rehearse.sh, tutorial-replay.sh
│   ├── release/             gh-assets-present.py, brew-formula-current.py,
│   │                        crates-io-max-version.py, installer-asset-bytes.py
│   ├── structure/           freshness-invariants.py (the migrated end-state.py freshness rows),
│   │                        banned-words.sh
│   ├── agent-ux/            dark-factory.sh
│   ├── perf/                (intentionally sparse in v0.12.0; v0.12.1 fills out)
│   └── security/            (intentionally sparse in v0.12.0; v0.12.1 fills out)
├── catalogs/                ← DATA layer; pure JSON
│   ├── README.md            (schema spec)
│   ├── docs-reproducible.json
│   ├── freshness-invariants.json
│   ├── release-assets.json
│   ├── perf-targets.json    (stub in v0.12.0)
│   ├── subjective-rubrics.json
│   └── orphan-scripts.json  (waivers for scripts that resist absorption)
├── reports/
│   ├── verifications/<gate>/<claim-id>.json   (per-claim artifacts; existing layout, namespaced)
│   └── verdicts/<cadence>/<timestamp>.md      (human-readable rollups)
└── runners/
    ├── run.py               (single entry point; reads gate metadata, composes by tag)
    └── verdict.py           (collates artifacts → verdict.md → exit nonzero if any RED)
```

A new gate is now: **add one row to a catalog + write the verifier in the right dimension dir.** No new top-level script, no new pre-push wiring, no new CI job. The runners discover it.

## Catalog schema

Every catalog row across every dimension uses the same shape. (Catalog files are JSON arrays of these objects, one file per dimension.)

```jsonc
{
  "id":            "release/installer-curl-sh",          // <dimension>/<slug> — stable, never reused
  "dimension":     "release",
  "cadence":       "weekly",                              // pre-push|pre-pr|weekly|pre-release|post-release|on-demand
  "kind":          "asset-exists",                        // mechanical|container|asset-exists|subagent-graded|manual
  "sources":       ["docs/index.md:42-46", "README.md:42-44"],   // for traceback when source drifts
  "command":       "curl --proto '=https' ...",           // what the user runs (for docs-repro / asset-exists rows)
  "expected":      { "asserts": ["..."] },                // concrete shell predicates the verifier checks
  "verifier":      {
    "script":      "quality/gates/release/installer-asset-bytes.py",
    "args":        ["--url", "https://github.com/.../reposix-installer.sh", "--min-bytes", "1024"],
    "timeout_s":   30,
    "container":   null                                   // or e.g. "ubuntu:24.04" for container-kind
  },
  "artifact":      "quality/reports/verifications/release/installer-curl-sh.json",
  "status":        "FAIL",                                // PASS|FAIL|PARTIAL|NOT-VERIFIED|WAIVED
  "last_verified": "2026-04-27T15:59:00Z",                // RFC3339 UTC
  "freshness_ttl": "30d",                                 // for subjective/manual rows; null for mechanical
  "blast_radius":  "P0",                                  // P0|P1|P2 for triage routing
  "owner_hint":    ".github/workflows/release.yml tag glob",  // where the FIX likely lives
  "waiver":        null                                   // {until: "2026-05-15", reason: "...", dimension_owner: "..."} or null
}
```

### Field semantics

| Field | Required? | Notes |
|---|---|---|
| `id` | yes | stable slug, never reused; `<dimension>/<slug>` form |
| `dimension` | yes | one of the 8 dimensions |
| `cadence` | yes | one of the 6 cadences |
| `kind` | yes | one of the 5 kinds |
| `sources` | recommended | file:line refs to the doc/code surface this gate is about; lets the verifier flag drift |
| `command` | for docs-repro / install rows | the literal command a user runs |
| `expected.asserts` | yes | concrete predicates; "exit 0", "stdout matches /.../", "file X exists with mode 0755" |
| `verifier.script` | yes | path under `quality/gates/<dim>/` |
| `verifier.args` | optional | command-line args |
| `verifier.timeout_s` | yes | budget; runner kills + records FAIL on overage |
| `verifier.container` | for container-kind | image name, e.g. `ubuntu:24.04` |
| `artifact` | yes | path the verifier writes; runner uses for `last_verified` + verdict |
| `status` | yes | runner updates after each verify; not hand-edited |
| `freshness_ttl` | for subjective/manual | duration string ("14d", "30d"); expired rows flip to NOT-VERIFIED |
| `blast_radius` | yes | P0/P1/P2 — drives verdict severity |
| `owner_hint` | recommended | one-liner describing where the fix likely lives |
| `waiver` | nullable | the principled escape hatch (see PROTOCOL §waivers) |

### Per-dimension catalog files

| File | Owner dimension | Source-of-truth seed |
|---|---|---|
| `docs-reproducible.json` | docs-repro | `.planning/docs_reproducible_catalog.json` (DRAFT from this session) |
| `freshness-invariants.json` | structure | migrated from `scripts/end-state.py` freshness rows |
| `release-assets.json` | release | populated in P58 from RELEASE-04 |
| `perf-targets.json` | perf | stub in v0.12.0; full in v0.12.1 |
| `subjective-rubrics.json` | (cross-dimension) | seeded with hero-clarity, install-positioning, headline-numbers |
| `orphan-scripts.json` | (meta) | waivers for scripts that genuinely resist absorption |

## Runner contract

`quality/runners/run.py --cadence <C>` does exactly:

1. Discover all catalog files under `quality/catalogs/`.
2. For each row, filter: `row.cadence == C AND row.waiver is null OR row.waiver.until > now()`.
3. Sort by `blast_radius` (P0 first) so the worst things fail loudest in CI logs.
4. For each row:
   a. Invoke `row.verifier.script` with `row.verifier.args` under `row.verifier.timeout_s`.
   b. Capture stdout/stderr; write JSON to `row.artifact` with shape `{ts, exit_code, stdout, stderr, asserts_passed: [...], asserts_failed: [...]}`.
   c. Update `row.status` based on exit code (0=PASS, 1=FAIL, 2=PARTIAL, timeout=FAIL).
   d. Update `row.last_verified`.
5. Write the catalog file back.
6. Exit 0 if all P0+P1 rows are PASS or WAIVED; exit 1 otherwise. P2 failures log loud but don't block.

`quality/runners/verdict.py [--cadence <C>]` does exactly:

1. Read every `row.artifact` for rows in scope.
2. Compute counts by status.
3. Write `quality/reports/verdicts/<cadence>/<ts>.md` with:
   - Top-line GREEN/RED verdict
   - Status counts
   - List of FAIL rows with their `owner_hint`
   - List of WAIVED rows with their `waiver.reason` and `waiver.until`
   - List of NOT-VERIFIED rows with their `freshness_ttl` and last_verified age
4. Exit 0 iff verdict is GREEN.

`session-end` mode (called by the thin `scripts/end-state.py` shim): runs `verdict.py` across every cadence and emits a single `SESSION-VERDICT.md` for the current session.

## What stays in `scripts/`

After v0.12.0 closes:

```
scripts/
├── hooks/
│   ├── pre-push           ← body is one line: quality/runners/run.py --cadence pre-push
│   └── test-pre-push.sh   ← updated to test the new entry point
├── install-hooks.sh       ← unchanged; developer install of git hooks
└── end-state.py           ← 30-line shim → quality/runners/verdict.py session-end
```

Everything else either:
- Moved into `quality/gates/<dim>/` (most cases)
- Reduced to a one-line shim that delegates (if external code imports it)
- Documented in `quality/catalogs/orphan-scripts.json` with an explicit waiver explaining why it can't be absorbed (rare)

The SIMPLIFY-01..12 requirements name each existing surface and its target home. P63 final audit: `find scripts/ -maxdepth 1 -type f | grep -v hooks | grep -v install-hooks.sh` returns empty (or only files with waiver rows).

## What stays in `examples/`

`examples/0[1-5]-*/run.sh` stay where they are (they're discoverable as documentation), but each becomes a docs-repro catalog row (container-rehearsal-kind, post-release cadence). The `examples/README.md` gets a callout: "each example below is a tracked Quality Gate — see `quality/catalogs/docs-reproducible.json` for the verifier."

This way:
- The `examples/` directory keeps serving its discovery role.
- The CI knows when an example silently breaks.
- A reader of `examples/01-shell-loop/run.sh` finds the same command that the catalog tracks — single source of truth.

## What stays in `benchmarks/`

`benchmarks/fixtures/*` stay (they're test inputs). The script `scripts/bench_token_economy.py` moves to `quality/gates/perf/token-economy-bench.py` per SIMPLIFY-11. The README in `benchmarks/` gets a pointer to the new gate location.

## CI workflow shape after v0.12.0

```
.github/workflows/
├── ci.yml                  ← cargo (code dim); existing; mostly unchanged
├── docs.yml                ← mkdocs deploy; existing; unchanged
├── audit.yml               ← cargo-audit; existing; unchanged
├── release.yml             ← release pipeline; FIXED in P56 (RELEASE-01 tag glob)
├── release-plz.yml         ← crates.io publish; existing; unchanged
├── bench-latency-cron.yml  ← existing; later folded into quality-weekly.yml in v0.12.1
├── quality-weekly.yml      ← NEW (P58): cron, runs run.py --cadence weekly
└── quality-post-release.yml ← NEW (P58): triggered by release.yml success, runs run.py --cadence post-release
```

## Boundary with `scripts/catalog.py`

The existing `scripts/catalog.py` is a per-file catalog renderer that reads `.planning/v0.11.1-catalog.json` and emits a markdown report. It's a different domain (per-FILE planning catalog vs per-CHECK quality catalog). SIMPLIFY-03 audits whether they should be merged. Initial assessment: keep separate; document the boundary in `quality/catalogs/README.md`. The planning catalog tracks "what files are touched in this session" — orthogonal to "what quality gates are GREEN."

## Migration safety: parallel-then-cut

Per OP-5 (Reversibility enables boldness) and the owner's explicit decision: ship the new system **alongside** the old. The pre-push hook runs both for the migration window. Compare verdicts in `quality/reports/verdicts/parity/` for two pre-push cycles. Hard-cut (delete old) only after parity is demonstrated. If anything stalls progress, pivot to hard-cut earlier rather than ship a half-migrated system.

## Cross-references

- `v0.12.0-vision-and-mental-model.md` for WHY each piece exists
- `v0.12.0-roadmap-and-rationale.md` for which phase ships which piece
- `v0.12.0-autonomous-execution-protocol.md` for `quality/PROTOCOL.md` content
- `.planning/REQUIREMENTS.md` `## v0.12.0` for the full RELEASE-* / QG-* / etc. requirement list
