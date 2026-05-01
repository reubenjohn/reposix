---
phase: 85
plan: 01
subsystem: docs
tags: [dvcs, docs-alignment, subjective, mkdocs, playwright]
dependency_graph:
  requires: [P79, P80, P81, P82, P83, P84]
  provides:
    - docs/concepts/dvcs-topology.md (DVCS mental model — three roles + Q2.2 phrasing)
    - docs/guides/dvcs-mirror-setup.md (owner walk-through; cron-only fallback; cleanup)
    - docs/guides/troubleshooting.md § "DVCS push/pull issues" (4 entry matrix)
    - quality/catalogs/doc-alignment.json + 3 verifier scripts (DVCS-DOCS-01..03)
    - quality/catalogs/subjective-rubrics.json + dvcs-cold-reader rubric (DVCS-DOCS-04, NOT_VERIFIED)
    - .planning/verifications/playwright/concepts/dvcs-topology.json (mermaid SVG artifact)
  affects:
    - mkdocs.yml (Diátaxis nav extended; Concepts + Guides slots)
    - CLAUDE.md (Quick links cite the new doc surfaces)
tech_stack:
  added: []
  patterns:
    - "Layer-2 banned-words clean (no FUSE / kernel / partial-clone / promisor / stateless-connect / fast-import / protocol-v2 / daemon residue)"
    - "Diátaxis split: concept doc separate from guide doc; troubleshooting matrix is task-oriented"
    - "Presence-check verifiers (10-30 lines, FLAT placement) per P74 TINY shape"
    - "Subjective-rubric row pattern: NOT_VERIFIED until owner runs /reposix-quality-review post-phase"
key_files:
  created:
    - docs/concepts/dvcs-topology.md
    - docs/guides/dvcs-mirror-setup.md
    - quality/gates/docs-alignment/dvcs-topology-three-roles.sh
    - quality/gates/docs-alignment/dvcs-mirror-setup-walkthrough.sh
    - quality/gates/docs-alignment/dvcs-troubleshooting-matrix.sh
    - .planning/verifications/playwright/concepts/dvcs-topology.json
    - .planning/phases/85-dvcs-docs/85-01-PLAN.md
    - .planning/phases/85-dvcs-docs/85-01-SUMMARY.md
  modified:
    - docs/guides/troubleshooting.md
    - mkdocs.yml
    - CLAUDE.md
    - quality/catalogs/doc-alignment.json
    - quality/catalogs/subjective-rubrics.json
decisions:
  - "Q2.2 phrasing rendered with <sot-host> placeholder rather than the literal 'confluence' from the decision record — generalizes to GitHub-Issues / JIRA backends without rewriting prose; verifier checks for the load-bearing phrase fragments ('mirror last caught up', 'NOT a', 'current SoT state') rather than verbatim string."
  - "Mermaid diagram in concept doc rather than ASCII alternative — playwright artifact captured (mermaid_count=1, svg_counts=[1], console_errors=[])."
  - "Subjective rubric row created with status NOT_VERIFIED + full criteria inline rather than awaiting Path A dispatch within this phase — cold-reader pass is owner-driven (CLAUDE.md OP-1 + the existing /reposix-quality-review pattern); the executing agent does not grade subjective clarity itself."
  - "Three docs-alignment verifier scripts placed FLAT under quality/gates/docs-alignment/ (sibling of jira-adapter-shipped.sh) per P74 precedent — no sub-area subdirectory."
metrics:
  duration_min: ~25
  completed_date: 2026-05-01
  tasks_completed: 8
  files_created: 8
  files_modified: 5
  catalog_deltas:
    claims_total: "390 -> 393 (+3)"
    claims_bound: "273 -> 276 (+3)"
    alignment_ratio: "0.8198 -> 0.8214"
    subjective_rows: "3 -> 4 (+1 NOT_VERIFIED)"
---

# Phase 85 Plan 01: DVCS docs Summary

**One-liner:** Three new doc surfaces (topology concept, mirror-setup guide, troubleshooting DVCS section) + 3 docs-alignment rows + 1 subjective rubric row make v0.13.0 legible to a cold reader.

## What shipped

### Doc surfaces (3 new files / 1 append)

- **`docs/concepts/dvcs-topology.md`** (164 lines / 1.5k words). Three roles (SoT-holder, mirror-only consumer, round-tripper) introduced in a tabular header, then re-explained in three "when-to-choose-which-pattern" subsections (Pattern A / B / C). Mermaid diagram shows the role topology with bus push (yellow), webhook sync, vanilla clone arrows. Mirror-lag refs explained with the verbatim Q2.2 phrasing: *"`refs/mirrors/<sot-host>-synced-at` is the timestamp the mirror last caught up to <sot-host> — it is NOT a 'current SoT state' marker."* Out-of-scope subsection covers bidirectional bus, multi-SoT, long-running sync process, atomic 2PC.

- **`docs/guides/dvcs-mirror-setup.md`** (198 lines / 1.5k words). Owner walk-through in 5 numbered steps: create mirror repo (`gh repo create`) → drop workflow into mirror repo (`curl ... dvcs-mirror-setup-template.yml`) → configure secrets/variables (`gh secret set`, `gh variable set`) → smoke-test manual run (`gh workflow run`, `gh run watch`) → configure Confluence webhook (Atlassian admin → POST `/repos/<org>/<repo>/dispatches`). Backends-without-webhooks fallback (Q4.2): cron-only mode by removing `repository_dispatch:` from `on:`. Cleanup procedure (5 numbered tear-down steps). Quick-fix table at the end + cross-link to troubleshooting.

- **`docs/guides/troubleshooting.md`** (+~120 lines, "DVCS push/pull issues" section). Four entries: (1) Bus-remote `fetch first` rejection — cites mirror-lag refs, walks through `reposix sync --reconcile && git pull --rebase` recovery; (2) Attach reconciliation warnings — table covering all 5 P79 cases (match / no-id / backend-deleted / duplicate-id / mirror-lag) with resolution per row; (3) Webhook race conditions — `--force-with-lease` semantics + bus-vs-webhook race, outcome is benign exit; (4) Cache-desync recovery — when to suspect, recovery commands, audit-log signal query.

### Catalog rows + verifier scripts (3 docs-alignment + 1 subjective)

Three docs-alignment rows BOUND via `target/release/reposix-quality doc-alignment bind`:

| Row id | Verifier | Closes |
|---|---|---|
| `docs-alignment/dvcs-topology-three-roles-bound` | `quality/gates/docs-alignment/dvcs-topology-three-roles.sh` | DVCS-DOCS-01 |
| `docs-alignment/dvcs-mirror-setup-walkthrough-bound` | `quality/gates/docs-alignment/dvcs-mirror-setup-walkthrough.sh` | DVCS-DOCS-02 |
| `docs-alignment/dvcs-troubleshooting-matrix-bound` | `quality/gates/docs-alignment/dvcs-troubleshooting-matrix.sh` | DVCS-DOCS-03 |

Each verifier is 20-40 lines, FLAT placement, P74 TINY shape. Body-hash drift on either source or verifier fires `STALE_DOCS_DRIFT` via the walker.

One subjective rubric row appended:

- `subjective/dvcs-cold-reader` — status `NOT_VERIFIED`, `freshness_ttl: 30d`, full rubric criteria inline (7 criteria including "three roles introduced before use", "mirror-lag refs not confused with current SoT state", "walk-through is runnable as-written", "zero plumbing-jargon leaks above Layer 3"). Owner runs `/reposix-quality-review --rubric dvcs-cold-reader` post-phase to flip from `NOT_VERIFIED` to PASS. This closes DVCS-DOCS-04 by establishing the verification contract; the actual cold-reader pass is owner-driven per CLAUDE.md OP-1.

### Mermaid render artifact

`.planning/verifications/playwright/concepts/dvcs-topology.json` — captured via headless chromium against `mkdocs serve` on `127.0.0.1:8766`; values `{mermaid_count: 1, svg_counts: [1], console_errors: []}`. `bash scripts/check-mermaid-renders.sh` GREEN.

### Nav + CLAUDE.md updates

- `mkdocs.yml`: `dvcs-topology.md` slotted under Concepts (after `reposix-vs-mcp-and-sdks.md`); `dvcs-mirror-setup.md` slotted under Guides (after `integrate-with-your-agent.md`, before `troubleshooting.md`).
- `CLAUDE.md` § "Quick links": three new bullets citing the new doc surfaces with phase tags `(P85)`.

## Catalog deltas

| Metric | Before | After | Δ |
|---|---|---|---|
| `claims_total` | 390 | 393 | +3 |
| `claims_bound` | 273 | 276 | +3 |
| `claims_missing_test` | 0 | 0 | 0 |
| `alignment_ratio` | 0.8198 | 0.8214 | +0.0016 |
| `subjective.rows` | 3 | 4 | +1 (NOT_VERIFIED) |

Walker exits 0 (zero `STALE_DOCS_DRIFT`, zero `MISSING_TEST` for new rows).

## Pre-push gate results

| Gate | Result |
|---|---|
| `bash quality/gates/structure/banned-words.sh` (--all) | `✓ banned-words-lint passed (all mode).` |
| `bash scripts/check-docs-site.sh` (mkdocs --strict) | `OK: docs site clean` |
| `bash scripts/check-mermaid-renders.sh` | `✓ 6 source-mermaid pages all have valid artifacts.` |
| `bash quality/gates/docs-alignment/walk.sh` | exit 0; only coverage-info notes (out-of-eligible files; pre-existing) |

One in-flight banned-words finding caught and resolved during execution: an "out of scope" bullet originally read "Daemon-mode sync." Renamed to "Long-running sync process." (Layer-2 P1 banned word `daemon`).

One in-flight mkdocs-strict finding caught and resolved: anchor mismatch on three internal links — em-dash `—` in headings collapses to single hyphen during slugify, but my links used double-hyphen. Three links fixed; mkdocs-strict re-ran clean.

One in-flight rebind: `dvcs-mirror-setup-walkthrough-bound` STALE_DOCS_DRIFT fired after the anchor-link fix; row re-bound with rationale documenting the round-trip.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Banned-word leak ("Daemon-mode")**
- **Found during:** Task 1 (concept doc) post-write banned-words run.
- **Issue:** Out-of-scope bullet used "Daemon-mode sync" — `daemon` is Layer-2 banned per `scripts/banned-words-lint.sh`.
- **Fix:** Renamed to "Long-running sync process."
- **Files modified:** `docs/concepts/dvcs-topology.md`
- **Commit:** `672be2d` (rolled into the docs commit; pre-existed in pre-commit working tree).

**2. [Rule 1 - Bug] mkdocs-strict anchor mismatch on internal links**
- **Found during:** Task 4 (mkdocs nav) post-update strict build.
- **Issue:** I linked to `#pattern-a--vanilla-mirror-only-mirror-only-consumer` (double-hyphen) but mkdocs slugify produces `#pattern-a-vanilla-mirror-only-mirror-only-consumer` (single hyphen) — em-dash `—` becomes one separator, not two.
- **Fix:** Updated 3 internal links across 2 files.
- **Files modified:** `docs/guides/dvcs-mirror-setup.md`, `docs/guides/troubleshooting.md`
- **Commit:** `672be2d` (rolled into the docs commit).

**3. [Rule 3 - Blocking] STALE_DOCS_DRIFT on dvcs-mirror-setup-walkthrough-bound**
- **Found during:** Task 6 walker run after binding (the bind hashed source state at one moment; the anchor-link fix mutated the source after).
- **Issue:** Walker fired STALE on the just-bound row.
- **Fix:** Re-ran `reposix-quality doc-alignment bind` with the same arguments + a "re-bound after anchor-link fix" rationale tail; walker re-ran clean.
- **Files modified:** `quality/catalogs/doc-alignment.json` (source_hash refresh)
- **Commit:** `06b8014` (rolled into the catalog commit).

### Out-of-scope discoveries

None. Phase ran exactly as scoped — pure docs + catalog work, no surprises that warranted SURPRISES-INTAKE entries. Per CLAUDE.md OP-8 honesty check: this phase did honestly look for out-of-scope items (the three auto-fixes above are in-scope discoveries that resolved within the phase).

### CLAUDE.md driven adjustments

- Verifier shapes follow the P74 TINY pattern (10-30 lines, FLAT placement) — matches the recently-tightened CLAUDE.md guidance for the docs-alignment dimension.
- Build-memory-budget rule honored: zero cargo invocations in this phase (pure shell + python3 + node + sed/grep). The reposix-quality binary used was the pre-existing `target/release/reposix-quality`.
- Per-phase push cadence (CLAUDE.md § "Push cadence"): push happens before verifier dispatch.

## Known Stubs

None. The subjective rubric row's `NOT_VERIFIED` state is intentional and documented (cold-reader pass is owner-driven per CLAUDE.md "Cold-reader pass on user-facing surfaces" — every other subjective rubric in `quality/catalogs/subjective-rubrics.json` follows the same pattern of "row exists with criteria inline; owner flips to PASS post-phase").

## Threat Flags

None. The new docs do not introduce new network endpoints, auth paths, file access patterns, or schema changes. The mirror-setup guide documents secrets handling (`gh secret set`) but the actual secrets flow through pre-existing P84 + P83 surfaces.

## Self-Check: PASSED

- File `docs/concepts/dvcs-topology.md`: FOUND
- File `docs/guides/dvcs-mirror-setup.md`: FOUND
- File `docs/guides/troubleshooting.md` § "DVCS push/pull issues": FOUND (line 227 anchor)
- File `quality/gates/docs-alignment/dvcs-topology-three-roles.sh`: FOUND
- File `quality/gates/docs-alignment/dvcs-mirror-setup-walkthrough.sh`: FOUND
- File `quality/gates/docs-alignment/dvcs-troubleshooting-matrix.sh`: FOUND
- File `.planning/verifications/playwright/concepts/dvcs-topology.json`: FOUND
- File `mkdocs.yml` (modified): nav contains both new pages
- File `CLAUDE.md` (modified): Quick links contains the three new bullets
- File `quality/catalogs/doc-alignment.json` (modified): 3 dvcs rows BOUND
- File `quality/catalogs/subjective-rubrics.json` (modified): subjective/dvcs-cold-reader row present, status NOT_VERIFIED
- Commit `672be2d` (docs): FOUND
- Commit `06b8014` (test/catalog): FOUND
- Commit (close — created with this SUMMARY): TO-FOLLOW
