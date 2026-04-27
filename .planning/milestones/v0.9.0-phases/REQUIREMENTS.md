# Requirements — v0.9.0 Architecture Pivot (HISTORICAL)

> **Status:** SHIPPED 2026-04-24. Phases 31–36 closed. ARCH-01..19 all validated.
>
> Extracted from the top-level `.planning/REQUIREMENTS.md` on 2026-04-27 during v0.12.0 milestone scaffolding. The "v1 Requirements" section in the original top-level file was actually v0.9.0 ARCH-NN detail — preserved here for historical traceability.
>
> Convention reference: `CLAUDE.md` §0.5 / Workspace layout.

**Milestone goal:** Replace the FUSE virtual filesystem with git's built-in partial clone mechanism. The `git-remote-reposix` helper becomes a promisor remote tunnelling protocol-v2 traffic to a local bare-repo cache built from REST responses. Agents interact with the project using only standard git commands (`clone`, `fetch`, `cat`, `grep`, `commit`, `push`) — zero reposix-specific CLI awareness required. FUSE is deleted entirely; `crates/reposix-fuse/` is removed and the `fuser` dependency is purged.

**Source of truth:** `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` (canonical design doc, 440 lines, ratified 2026-04-24). Supporting POC artifacts in `.planning/research/v0.9-fuse-to-git-native/poc/`.

**Operating-principle hooks (non-negotiable, per project CLAUDE.md):**

- **Simulator-first.** All ARCH requirements ship and test against `reposix-sim` by default. Real backends (GitHub, Confluence, JIRA) are exercised only behind `REPOSIX_ALLOWED_ORIGINS` + explicit credentials.
- **Tainted-by-default.** Every byte materialized into the bare-repo cache or returned to git originated from a remote (real or simulator). Tainted content must be wrapped in `reposix_core::Tainted<T>` along the data path; sanitization is explicit.
- **Audit log non-optional.** Every blob materialization, every `command=fetch`, every `export` push (accept and reject) writes a row to the SQLite audit table.
- **Egress allowlist.** All HTTP construction goes through the existing `reposix_core::http::client()` factory; no new direct `reqwest::Client::new()` call sites.
- **Working tree = real git repo.** The mount point is no longer synthetic — it is a true git working tree backed by `.git/objects` (partial clone, blobs lazy).
- **Self-improving infrastructure (OP-4).** Project CLAUDE.md and Claude Code skills MUST ship in lockstep with the code that invalidates them. Phase 36 explicitly bundles agent-grounding updates with FUSE deletion.

### v1 Requirements (the v0.9.0 architecture-pivot detail)

#### Cache & data path

- [shipped] **ARCH-01**: New crate `crates/reposix-cache/` constructs a real on-disk bare git repo from REST responses via the existing `BackendConnector` trait. Lazy blob materialization. Source: architecture-pivot-summary §2.
- [shipped] **ARCH-02**: `reposix-cache` writes one audit row per blob materialization. Cache returns blob bytes wrapped in `reposix_core::Tainted<Vec<u8>>`. Audit schema: `(ts, backend, project, issue_id, blob_oid, byte_len, op="materialize")`. SQLite WAL append-only.
- [shipped] **ARCH-03**: `reposix-cache` enforces `REPOSIX_ALLOWED_ORIGINS` egress allowlist by reusing the single `reposix_core::http::client()` factory.

#### Transport (read path)

- [shipped] **ARCH-04**: `git-remote-reposix` advertises `stateless-connect` capability and tunnels protocol-v2 fetch traffic (handshake, ls-refs, fetch with filter) to the cache's bare repo. The existing `export` capability for push remains alongside `stateless-connect` (hybrid helper). Source: architecture-pivot-summary §3.
- [shipped] **ARCH-05**: The Rust `stateless-connect` implementation correctly handles all three protocol gotchas surfaced during POC iteration:
  1. **Initial advertisement does NOT terminate with `0002`** — only `0000` (flush).
  2. **Subsequent RPC responses DO need trailing `0002`** — after each response pack.
  3. **Stdin reads in binary mode throughout** — consistent `BufReader<Stdin>`.

  Refspec namespace MUST be `refs/heads/*:refs/reposix/*` (NOT `refs/heads/*:refs/heads/*`).

#### Sync (delta path)

- [shipped] **ARCH-06**: `BackendConnector::list_changed_since(timestamp) -> Vec<IssueId>` added as a trait method. All backends implement it using their native incremental query mechanism: GitHub `?since=<ISO8601>`, Jira `JQL: updated >= "<datetime>"`, Confluence `CQL: lastModified > "<datetime>"`, sim filtering with `since` REST param. Source: architecture-pivot-summary §4.
- [shipped] **ARCH-07**: Delta sync flow: on `git fetch`, helper reads `last_fetched_at`, calls `list_changed_since`, materializes changed items, atomically writes new `last_fetched_at`. Cache update + `last_fetched_at` write in one SQLite transaction. One audit row per delta-sync invocation.

#### Push path

- [shipped] **ARCH-08**: Push-time conflict detection lives inside the `export` handler. Flow: parse fast-import stream; for each changed file, fetch backend version via REST; if backend version differs, emit `error refs/heads/main fetch first`; reject path drains stream and never touches bare cache; on success apply REST writes + emit `ok refs/heads/main`. Audit row for every push attempt.
- [shipped] **ARCH-09**: Blob limit guardrail. Helper counts `want <oid>` lines per `command=fetch` request and refuses if count exceeds `REPOSIX_BLOB_LIMIT` (default 200). Refusal stderr message names `git sparse-checkout` (self-teaching dark-factory pattern).
- [shipped] **ARCH-10**: Frontmatter field allowlist on the push path. Server-controlled fields (`id`, `created_at`, `version`, `updated_at`) stripped before REST. `Tainted<T>` -> `Untainted<T>` is the explicit `sanitize` step.

#### CLI & agent UX

- [shipped] **ARCH-11**: `reposix init <backend>::<project> <path>` replaces `reposix mount`. Runs `git init`, configures `extensions.partialClone=origin`, sets `remote.origin.url=reposix::<backend>/<project>`, runs `git fetch --filter=blob:none origin`. CHANGELOG `[v0.9.0]` documents the breaking CLI change.
- [shipped] **ARCH-12**: End-to-end agent UX validation. A fresh subprocess agent (no reposix CLI awareness) given ONLY a `reposix init` command + a goal succeeds using pure git/POSIX. Validation specifically exercises the conflict-rebase cycle. The dark-factory acceptance test.

#### Demolition & grounding

- [shipped] **ARCH-13**: `crates/reposix-fuse/` deleted entirely. `fuser` dependency removed from workspace `Cargo.toml`. `fuse-mount-tests` feature gates removed. CI no longer runs `apt install fuse3` or mounts `/dev/fuse`.
- [shipped] **ARCH-14**: Project `CLAUDE.md` updated as part of the same phase that lands FUSE deletion. All FUSE references purged from elevator pitch, Operating Principles, Workspace layout, Tech stack, "Commands you'll actually use", and the Threat-model table. New steady-state **Architecture (git-native partial clone)** section. Tech stack adds `git >= 2.34` runtime requirement. Per OP-4 self-improving infrastructure: agent grounding must match shipped reality.
- [shipped] **ARCH-15**: A project Claude Code skill `reposix-agent-flow` created at `.claude/skills/reposix-agent-flow/SKILL.md`. Encodes the dark-factory autonomous-agent regression test. Invoked from CI (release gate) and from local dev (`/reposix-agent-flow`). Per OP-4: ships in the same phase as ARCH-13 and ARCH-14.

#### Real-backend validation, performance, canonical test targets

- [shipped] **ARCH-16**: Real-backend smoke tests against canonical test targets: Confluence "TokenWorld", `reubenjohn/reposix` GitHub issues, JIRA project `TEST`. Gated by `REPOSIX_ALLOWED_ORIGINS` + credentials.
- [shipped] **ARCH-17**: Latency & performance envelope captured for each backend. Committed benchmark script writes Markdown artifact under `docs/benchmarks/v0.9.0-latency.md` (renamed to `latency.md` in v0.11.2 §0.3).
- [shipped] **ARCH-18**: Canonical test-target documentation. `docs/reference/testing-targets.md` enumerates the three sanctioned targets with env-var setup, rate-limit notes, cleanup procedure.
- [shipped] **ARCH-19**: CI integration-contract job for each real backend. Three CI jobs (`integration-contract-confluence-v09`, `integration-contract-github-v09`, `integration-contract-jira-v09`) run the ARCH-16 smoke suite when the corresponding secret block is present.

### Future Requirements (deferred at v0.9.0; partially closed since)

*(Open questions from architecture-pivot-summary §7. Several closed in v0.10/v0.11.)*

- Cache eviction policy for `reposix-cache` (LRU / TTL / per-project disk quota / manual `reposix gc`). **Partially closed:** `reposix gc --orphans/--purge` shipped in v0.11.0 Phase 55.
- `import` capability deprecation (kept one release cycle past v0.9.0). **Status:** still alongside `stateless-connect`; remove in v0.12.0+.
- Stream-parsing performance for `export` — production state-machine parser with commit-or-rollback barrier at fast-import `done` terminator.
- Threat-model update for push-through-export flow — `research/threat-model-and-critique.md` revision once the helper-as-egress-surface model is stable. **Deferred to a security-focused milestone.**
- Non-issue file handling on push (e.g., changes to `.planning/` paths) — reject vs. silently commit-to-cache decision.

### Out of Scope

- **Bringing FUSE back.** The pivot is one-way; `crates/reposix-fuse/` is removed, not deprecated.
- **Custom CLI for read operations.** Agents use `git clone`, `git fetch`, `cat`, `grep` — no `reposix list`, no `reposix get`. The whole point is zero in-context learning.
- **Full real-backend exercise in autonomous mode.** Real backends require explicit credentials and a non-default `REPOSIX_ALLOWED_ORIGINS`. Autonomous execution validates against the simulator only.
- **Cache eviction implementation in v0.9.0.** Decision deferred per architecture-pivot-summary §7 open question 1; closed in v0.11.0.
- **`docs/` rewrite.** Carried forward to v0.10.0 (DOCS-01..09) — shipped.

### Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| ARCH-01..03 | 31 | shipped |
| ARCH-04..05 | 32 | shipped |
| ARCH-06..07 | 33 | shipped |
| ARCH-08..10 | 34 | shipped |
| ARCH-11..12 | 35 | shipped |
| ARCH-13..15 | 36 | shipped |
| ARCH-16 | 35 | shipped |
| ARCH-17 | 35 (capture) + 36 (artifact) | shipped |
| ARCH-18 | 36 | shipped |
| ARCH-19 | 36 | shipped (pending-secrets resolved in v0.11.0) |
