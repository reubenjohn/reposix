# reposix v0.11.0 — DRAFT preview

> **v0.11.0 not yet cut.** This is a preview; the cut may aggregate further work. Surface tracks `main` as of 2026-04-25.

Informal name: *Capabilities & Self-Diagnosis*. Six entries already on `main`.

## What's in flight

### IssueId → RecordId rename ([ADR-006](docs/decisions/006-issueid-to-recordid-rename.md))

Workspace-wide hard rename. `IssueId` → `RecordId`, `Issue` → `Record`, `BackendConnector::*_issue` → `*_record`. **YAML wire format is unchanged.** No compatibility aliases (precedent: ADR-004).

### `reposix doctor` — 14 setup checks

CLI subcommand auditing a `reposix init`'d tree with copy-pastable fix commands. Spans git layout, `extensions.partialClone`, helper on `PATH`, cache DB integrity + append-only triggers, freshness, env-var sanity, sparse-checkout, `rustc` version. Tiers OK / INFO / WARN / ERROR; exit 1 on ERROR. `--fix` applies the deterministic non-destructive subset.

### `reposix history` + `reposix at <ts>` ([ADR-007](docs/decisions/007-time-travel-via-git-tags.md))

Every `Cache::sync` writes `refs/reposix/sync/<ISO8601-no-colons>` at the synthesis commit. `transfer.hideRefs` keeps these private — the helper does NOT advertise them to the agent. `history` lists tags; `at <ts>` returns the closest tag at-or-before the instant. New audit op `sync_tag_written`. Flagged as the brainstorm's highest-novelty entry — no prior art for "tag every external sync as a first-class git ref."

### Helper URL-scheme backend dispatch ([ADR-008](docs/decisions/008-helper-backend-dispatch.md))

`git-remote-reposix` reads `argv[2]` and instantiates the matching `BackendConnector` instead of hardcoding `SimBackend`. Closes the v0.9.0 Phase 32 carry-forward debt. Atlassian URLs gained `/confluence/` or `/jira/` markers. Real-backend `git fetch` against TokenWorld, `reubenjohn/reposix`, and JIRA `TEST` works when creds are present. New audit op `helper_backend_instantiated`.

### `reposix gc` — cache eviction

Strategies: `lru` (default, until `--max-size-mb` cap of 500), `ttl` (older than `--max-age-days`, default 30), `all`. `--dry-run` plans without touching disk. **Tree, commit, and ref objects are never touched** — only loose blobs under `objects/<2>/<38>`. Evicted blobs re-fetch transparently. New audit op `cache_gc`.

### `reposix tokens` — token-economy ledger

Reads `op='token_cost'` rows (one per helper RPC turn) and prints totals plus an honest MCP-equivalent baseline (100k schema discovery + 5k per tool call). Token estimate is `chars / 4` over wire bytes; output flags that the heuristic over-estimates for binary packfile content and that savings vary by workload.

## Not in v0.11.0 (yet)

The brainstorm at `.planning/research/v0.11.0-vision-and-innovations.md` lists twelve innovations; the cut may pick two or three of the remaining six. Candidates: OpenTelemetry tracing, multi-project helper process, plugin registry mock, `reposix conflict <id>` field-level YAML merge UI.

Final scope lands when the owner cuts the release.
