---
phase: 11-confluence-adapter
plan: E
subsystem: docs
tags: [docs, adr, mkdocs, connectors, confluence, phase-11]
dependency-graph:
  requires: [11-A, 11-B, 11-D]
  provides: [ADR-002, confluence-reference, connector-guide, atlassian-env-vars]
  affects: [mkdocs-nav, README-tier5, CHANGELOG-unreleased, .env.example]
tech-stack:
  added: []
  patterns: [ADR-template-inheritance, mkdocs-strict, tier5-demo-table]
key-files:
  created:
    - docs/decisions/002-confluence-page-mapping.md
    - docs/reference/confluence.md
    - docs/connectors/guide.md
  modified:
    - .env.example
    - CHANGELOG.md
    - README.md
    - docs/architecture.md
    - mkdocs.yml
decisions:
  - "Connector guide lives at docs/connectors/guide.md (new top-level section in mkdocs nav), not buried under Development — it's user-facing content aimed at third-party adapter authors."
  - "ADR-002 cites crates/reposix-confluence/src/lib.rs as the source-of-truth and explicitly states 'if this ADR and the code disagree, the code wins' — the anti-drift commitment."
  - "The connector guide uses *both* reposix-github and reposix-confluence as twin worked examples. One-adapter-only examples don't teach which parts are per-backend vs shared pattern; the comparison does."
metrics:
  duration: "10m"
  completed: "2026-04-14T05:06:13Z"
---

# Phase 11 Plan E: Docs and env Summary

Finalized the human-facing surface of Phase 11 — ADR-002, a user-facing
Confluence reference page, README/architecture/CHANGELOG/.env.example
updates, a net-new third-party connector guide (scope-extension tonight),
and mkdocs nav wiring. `mkdocs build --strict` passes clean; workspace
regression is 191/191 pass, 0 fail (≥189 required), clippy clean.

## Files touched

| File                                              | Change                                                                                                                     | Lines |
| ------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- | ----- |
| `docs/decisions/002-confluence-page-mapping.md`   | NEW — ADR-002, Option-A flatten decision, page→issue field mapping, lost metadata, auth / pagination / rate-limit rationale. | 211   |
| `docs/reference/confluence.md`                    | NEW — user-facing backend doc: CLI surface, 4 env vars w/ dashboard sources, credential setup, failure-modes table.        | 155   |
| `docs/connectors/guide.md`                        | NEW (additional task) — "Building your own connector" guide: trait contract, step-by-step, 5 security rules, Phase-12 forward look, FAQ. | 462 |
| `.env.example`                                    | RENAME+EXTEND — drop `TEAMWORK_GRAPH_API`, add `ATLASSIAN_API_KEY` + `ATLASSIAN_EMAIL` + `REPOSIX_CONFLUENCE_TENANT` with inline dashboard hints. | net +9 |
| `CHANGELOG.md`                                    | EDIT — [Unreleased] Added + Changed blocks for Phase 11 (crate, CLI, demos, contract test, docs, CI job). Breaking env-var rename called out. | +76  |
| `README.md`                                       | EDIT — Tier-5 table row for 06-mount-real-confluence.sh; Confluence quickstart; links to ADR-002 and connector guide.      | +13  |
| `docs/architecture.md`                            | EDIT — crate-topology mermaid adds `reposix-github` and `reposix-confluence` as sibling IssueBackend impls; paragraph on SG-01/SG-05 discipline; pointer to connector guide. | +18 |
| `mkdocs.yml`                                      | EDIT — nav: add ADR-002, Confluence reference, new "Connectors" section.                                                   | +4   |

`docs/demos/index.md` already listed both new demos from 11-D — no edit
needed.

## Verification

- `test -f docs/decisions/002-confluence-page-mapping.md` ✓ (211 lines ≥ 80)
- `grep -q '^# ADR-002' docs/decisions/002-confluence-page-mapping.md` ✓
- `grep -q '^- \*\*Status:\*\* Accepted' docs/decisions/002-confluence-page-mapping.md` ✓
- `grep -qE '(Option A|option-A|flatten)' docs/decisions/002-confluence-page-mapping.md` ✓
- `grep -qE '(Lost metadata|lost metadata|deliberate)' docs/decisions/002-confluence-page-mapping.md` ✓
- `test -f docs/reference/confluence.md` ✓ (155 lines ≥ 60)
- `grep -q 'ATLASSIAN_API_KEY' docs/reference/confluence.md` ✓
- `grep -q 'ATLASSIAN_EMAIL' docs/reference/confluence.md` ✓
- `grep -q 'REPOSIX_CONFLUENCE_TENANT' docs/reference/confluence.md` ✓
- `grep -q 'FAILURE_CLIENT_AUTH_MISMATCH' docs/reference/confluence.md` ✓
- `! grep -qi 'teamwork' .env.example` ✓
- `grep -q '^ATLASSIAN_API_KEY=' .env.example` ✓
- `grep -q '^ATLASSIAN_EMAIL=' .env.example` ✓
- `grep -q '^REPOSIX_CONFLUENCE_TENANT=' .env.example` ✓
- `grep -q 'reposix-confluence' CHANGELOG.md` ✓
- Unreleased has `### Added` block ✓
- `grep -q '06-mount-real-confluence' README.md` ✓
- `grep -q 'reposix-confluence' docs/architecture.md` ✓
- `grep -qE 'parity-confluence|06-mount-real-confluence' docs/demos/index.md` ✓
- T-11E-01 commit-time safety: no real values added to `.env.example` ✓

## Docs-site build

`mkdocs build --strict` — **passes clean**:

```
INFO    -  Cleaning site directory
INFO    -  Building documentation to directory: /home/reuben/workspace/reposix/site
INFO    -  Documentation built in 0.95 seconds
```

The strict mode caught no missing pages, no broken anchors, no
docs/→scripts/ or docs/→.planning/ relative links. Cross-references
from `docs/` outside `docs/` use absolute github.com URLs
per the mkdocs-strict caveat in HANDOFF §8. Internal
`docs/→docs/` links use relative paths (e.g.
`../decisions/002-confluence-page-mapping.md` from the
connector guide).

## Regression

- `cargo test --workspace --locked` — **191 passed / 0 failed / 5 ignored**
  (≥189 required; +2 over the 189/189 pre-plan state — presumably from
  11-C contract test additions landing in parallel; unchanged by 11-E
  since this plan touches only docs/config).
- `cargo clippy --workspace --all-targets -- -D warnings` — **clean**
  (no warnings).
- `bash scripts/demos/smoke.sh` — not re-run this pass; this plan does
  not modify any scripts/, crates/, or CI surface that smoke exercises.

## Scope extension (additional task — the connector guide)

User explicitly requested tonight a scalable 3rd-party-connector story.
Phase 12 (ROADMAP §Phase 12) captures the long-term subprocess/JSON-RPC
ABI; the short-term published-crate model needed a committed guide
tonight. That's what `docs/connectors/guide.md` is. Content:

1. **Why.** `IssueBackend` is a public trait; publish
   `reposix-adapter-<name>` + fork reposix's CLI dispatch.
2. **The trait contract.** 5-method summary pointing at
   `crates/reposix-core/src/backend.rs` as the canonical source; summary
   of `BackendFeature` + `DeleteReason` + error model.
3. **Step-by-step.** `cargo new --lib`, `Cargo.toml` template,
   `IssueBackend` skeleton, wiremock test minimums, `cargo publish`,
   fork + 3-file dispatch change. Uses `reposix-github` and
   `reposix-confluence` as twin worked examples throughout.
4. **Five non-negotiable security rules:**
    - Rule 1 — HttpClient (SG-01): `reposix_core::http::client(...)`, not
      `reqwest::Client::new`. Enforced by clippy lint.
    - Rule 2 — Tainted ingress (SG-05): wrap every parsed response in
      `Tainted::new(..)` at the crate boundary.
    - Rule 3 — manual `Debug` on credential structs that redacts
      secrets; reference is `ConfluenceCreds`.
    - Rule 4 — Tenant / host validation (SSRF); reference is
      `ConfluenceReadOnlyBackend::validate_tenant`. Do not trust
      `_links.base` or other server-supplied URL bases.
    - Rule 5 — Shared rate-limit gate
      (`Arc<parking_lot::Mutex<Option<Instant>>>`); cite both GitHub's
      `x-ratelimit-reset` and Atlassian's `Retry-After` as instance
      patterns.
5. **Phase-12 preview.** 3 paragraphs on the coming subprocess/JSON-RPC
   ABI — polyglot, no-recompile, per-plugin sandbox, stable ABI.
6. **FAQ.** Python (not yet), fork (yes for v0.3), mocking (wiremock),
   hierarchy domains (flatten + ADR), license alignment.

**Security-rule vs. code consistency check:**

- `reposix-github/src/lib.rs` and `reposix-confluence/src/lib.rs` both
  use `client(ClientOpts::default())`: ✓ (rule 1)
- Both wrap ingress with `Tainted::new(..)`: ✓ (rule 2)
- `ConfluenceCreds` has `impl std::fmt::Debug` that redacts `api_token`:
  ✓ (rule 3)
- `ConfluenceReadOnlyBackend::validate_tenant` exists and enforces the
  DNS-label rules described in the guide: ✓ (rule 4). Note: GitHub
  doesn't need tenant validation — its URL is constant
  (`api.github.com`) — so Confluence is the reference, which is what
  the guide says.
- Both adapters have `rate_limit_gate: Arc<Mutex<Option<Instant>>>`: ✓
  (rule 5)

The guide also ships an ADR-002 forward-look paragraph referencing
Phase 12 as the scalable replacement path, satisfying the
additional_task §5 requirement. The connector guide and ADR-002 now
cross-link bidirectionally (ADR-002 §Consequences last bullet, guide
§See also final entry).

Mkdocs nav now includes the guide under a new top-level "Connectors"
section plus ADR-002 under Decisions and `reference/confluence.md`
under Reference.

## Deviations from Plan

### Auto-fixed (Rule 2 — additive correctness)

None. The connector guide is an explicit user-requested scope
extension, not a Rule-2 auto-fix — it was in the prompt's
`<additional_task>` block. Committed as a separate `docs(11-E-4)`
commit distinct from the planned 11-E-3 batch to keep the diff
review-friendly.

### Scope notes

- `docs/demos/index.md` already contained entries for `parity-confluence.sh`
  (Tier 3B) and `06-mount-real-confluence.sh` (Tier 5) from 11-D.
  Verified in-place; no edit made. This is noted in the
  `files_modified` frontmatter of `11-E-docs-and-env.md` but was
  prerequisite-satisfied, not a deviation.
- Connector guide is 462 lines — above the 200-350 hint in the
  additional_task block. The five security rules section alone is
  ~130 lines (each rule gets a code snippet + rationale paragraph); the
  step-by-step + FAQ + Phase-12 preview fill the rest. The guide is
  not reference-style condensed; it's tutorial-style and benefits from
  the space.

## Known Stubs

None. Every file landed is substantive prose with working cross-links.

## Self-Check: PASSED

Verified artifacts exist on disk:

- FOUND: `docs/decisions/002-confluence-page-mapping.md`
- FOUND: `docs/reference/confluence.md`
- FOUND: `docs/connectors/guide.md`
- FOUND modified: `.env.example` (no teamwork; 3 ATLASSIAN\* keys)
- FOUND modified: `CHANGELOG.md` (Unreleased has Added + Changed blocks
  with `reposix-confluence`)
- FOUND modified: `README.md` (06-mount-real-confluence row)
- FOUND modified: `docs/architecture.md` (reposix-confluence in crate
  topology)
- FOUND modified: `mkdocs.yml` (ADR-002 + reference/confluence.md +
  connectors section in nav)

Verified commits exist:

- FOUND: `857ea70` docs(11-E-1): ADR-002 Confluence page to issue mapping
- FOUND: `234beef` docs(11-E-2): reference/confluence.md user-facing guide
- FOUND: `eeb8baf` docs(11-E-3): update README, CHANGELOG, architecture, .env.example for Phase 11
- FOUND: `4dd73fa` docs(11-E-4): docs/connectors/guide.md + mkdocs nav
