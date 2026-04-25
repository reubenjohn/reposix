# Requirements — Active milestone: v0.10.0 Docs & Narrative Shine

**Active milestone:** v0.10.0 Docs & Narrative Shine (planning_started 2026-04-24).
**Previous validated milestone:** v0.9.0 Architecture Pivot — Git-Native Partial Clone (SHIPPED 2026-04-24, see "v0.9.0 Requirements (Validated)" section below).

---

## v0.10.0 Requirements — Docs & Narrative Shine

**Milestone goal:** Make the reposix value proposition land in 10 seconds for a cold reader, with progressive disclosure of architecture and a tested 5-minute first-run tutorial. Sales-ready docs with hard numbers, agent-SDK guidance, and a banned-word linter that enforces P1/P2 framing rules.

**Source of truth:** `.planning/research/v0.10.0-post-pivot/milestone-plan.md` (forward-plan draft, ratified 2026-04-24). Framing principles inherited from `.planning/notes/phase-30-narrative-vignettes.md` (P1 complement-not-replace; P2 progressive disclosure — banned-word list **revised for git-native**: `FUSE`, `inode`, `daemon`, `mount`, `fusermount` removed because they no longer apply; new banned-above-Layer-3 list is `partial-clone`, `promisor`, `stateless-connect`, `fast-import`, `protocol-v2`).

**Operating-principle hooks (non-negotiable, per project CLAUDE.md):**

- **Self-improving infrastructure (OP-4).** Every doc claim is grounded in a committed artifact (`docs/benchmarks/v0.9.0-latency.md`, `docs/reference/testing-targets.md`, source files). No marketing copy that the codebase can't back up.
- **Close the feedback loop (OP-1).** Every mermaid diagram is rendered via mcp-mermaid and screenshot-verified via playwright before merge. The 5-minute tutorial is run end-to-end by a test fixture — the doc IS the test.
- **Numbers, not adjectives.** Every adjective on the README hero and `docs/index.md` hero is replaced with a measured number sourced from `docs/benchmarks/v0.9.0-latency.md` or v0.9.0 audit/threat-model artifacts.
- **Ground truth obsession (OP-6).** Banned-word linter is committed in `scripts/banned-words-lint.sh`, runs in pre-commit + CI; the layer-banned-word list lives in a checked-in config (`docs/.banned-words.toml` or equivalent). Ad-hoc grep is not a linter.

### Active

- [ ] **DOCS-01**: Reader can understand reposix's value proposition within 10 seconds of landing on `docs/index.md` (hero with V1 before/after code block + three-up value props citing actual latency numbers from `docs/benchmarks/v0.9.0-latency.md` — `8 ms` `get-issue`, `24 ms` `reposix init`, `9 ms` `list issues`, `5 ms` helper capabilities probe). P1 "complement, not replace" framing — the word "replace" is banned from hero copy.
- [ ] **DOCS-02**: Three-page "How it works" section — `docs/how-it-works/{filesystem-layer,git-layer,trust-model}.md` — each with one mermaid diagram (rendered via mcp-mermaid + playwright-screenshot verified) carved from the existing architecture argument and the v0.9.0 architecture-pivot summary. **Filesystem-layer** is reframed for git-native (the cache + working tree, not FUSE).
- [ ] **DOCS-03**: Two home-adjacent concept pages: `docs/concepts/mental-model-in-60-seconds.md` (clone = snapshot · frontmatter = schema · `git push` = sync verb) and `docs/concepts/reposix-vs-mcp-and-sdks.md` (positioning page grounding P1, with a numbers-table contrasting tokens-per-task, latency, and dependency footprint).
- [ ] **DOCS-04**: Three guides: `docs/guides/write-your-own-connector.md` (BackendConnector walkthrough), `docs/guides/integrate-with-your-agent.md` (Claude Code / Cursor / SDK patterns), `docs/guides/troubleshooting.md` (push rejections, audit-log queries, blob-limit recovery).
- [ ] **DOCS-05**: Simulator relocated from "How it works" to Reference (`docs/reference/simulator.md`).
- [ ] **DOCS-06**: 5-minute first-run tutorial `docs/tutorials/first-run.md` against the simulator, ending with a real edit committed and pushed. Tutorial is end-to-end runnable; the runner script (`scripts/tutorial-runner.sh`) verifies each step.
- [ ] **DOCS-07**: MkDocs nav restructured per Diátaxis (Home / How it works / Tutorials / Guides / Reference / Decisions / Research). P2 banned terms (FUSE, fusermount, kernel, syscall — plus the revised git-native list `partial-clone`, `promisor`, `stateless-connect`, `fast-import`, `protocol-v2`) do not appear above Layer 3 (How it works) in any user-facing page.
- [ ] **DOCS-08**: mkdocs-material theme tuned (palette, hero features, social cards). README hero rewritten — every adjective replaced with a measured number sourced from v0.9.0 latency or v0.9.0 audit/threat-model.
- [ ] **DOCS-09**: Banned-word linter `scripts/banned-words-lint.sh` runs on every doc commit (pre-commit hook + CI) and rejects violations of the P2 progressive-disclosure layer rules. The layer-banned-word list lives in a checked-in config (`docs/.banned-words.toml`) so adding a layer banned word is a reviewable diff.
- [ ] **DOCS-10**: Per-page `doc-clarity-review` skill run as a release gate; zero critical friction points in any user-facing page. Findings logged per phase; the gate runs in Phase 44 over the full doc tree.
- [ ] **DOCS-11**: README updated to point to mkdocs site as the source of truth for narrative; root-level docs (`README.md`, `CLAUDE.md`) are stubs/grounding-only and stop duplicating narrative copy. CHANGELOG `[v0.10.0]` block + playwright screenshots committed for landing + how-it-works + tutorial pages.

### Out of Scope

- New backend connectors, new CLI commands, new transport features. v0.10.0 is docs-only.
- Benchmark harness improvements beyond cross-linking — `cargo run -p reposix-bench` lands in v0.11.0.
- Agent-SDK recipes (Claude Code / Cursor / Aider / Continue / Devin / SWE-agent CI fixtures) — those land in v0.12.0. v0.10.0 ships `docs/guides/integrate-with-your-agent.md` as a pointer page only.
- Resolving the v0.9.0 carry-forward "helper hardcodes `SimBackend`" tech debt — scheduled before v0.11.0 benchmark commits, not v0.10.0.

### Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| DOCS-01 | 40 | planning |
| DOCS-02 | 41 | planning |
| DOCS-03 | 40 | planning |
| DOCS-04 | 42 | planning |
| DOCS-05 | 42 | planning |
| DOCS-06 | 42 | planning |
| DOCS-07 | 43 | planning |
| DOCS-08 | 43 (linter wiring) + 45 (README hero rewrite) | planning |
| DOCS-09 | 43 | planning |
| DOCS-10 | 44 | planning |
| DOCS-11 | 45 | planning |

---

## v0.9.0 Requirements (Validated) — Architecture Pivot — Git-Native Partial Clone

**Milestone goal:** Replace the FUSE virtual filesystem with git's built-in partial clone mechanism. The `git-remote-reposix` helper becomes a promisor remote tunnelling protocol-v2 traffic to a local bare-repo cache built from REST responses. Agents interact with the project using only standard git commands (`clone`, `fetch`, `cat`, `grep`, `commit`, `push`) — zero reposix-specific CLI awareness required. FUSE is deleted entirely; `crates/reposix-fuse/` is removed and the `fuser` dependency is purged.

**Source of truth:** `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` (canonical design doc, 440 lines, ratified 2026-04-24). Supporting POC artifacts in `.planning/research/v0.9-fuse-to-git-native/poc/`.

**Operating-principle hooks (non-negotiable, per project CLAUDE.md):**

- **Simulator-first.** All ARCH requirements ship and test against `reposix-sim` by default. Real backends (GitHub, Confluence, JIRA) are exercised only behind `REPOSIX_ALLOWED_ORIGINS` + explicit credentials.
- **Tainted-by-default.** Every byte materialized into the bare-repo cache or returned to git originated from a remote (real or simulator). Tainted content must be wrapped in `reposix_core::Tainted<T>` along the data path; sanitization is explicit.
- **Audit log non-optional.** Every blob materialization, every `command=fetch`, every `export` push (accept and reject) writes a row to the SQLite audit table.
- **Egress allowlist.** All HTTP construction goes through the existing `reposix_core::http::client()` factory; no new direct `reqwest::Client::new()` call sites. `REPOSIX_ALLOWED_ORIGINS` is enforced before any outbound request.
- **Working tree = real git repo.** The mount point is no longer synthetic — it is a true git working tree backed by `.git/objects` (partial clone, blobs lazy).
- **Self-improving infrastructure (OP-4).** Project CLAUDE.md and Claude Code skills MUST ship in lockstep with the code that invalidates them. Phase 36 explicitly bundles agent-grounding updates with FUSE deletion.

---

## v1 Requirements

### Cache & data path

- [ ] **ARCH-01**: New crate `crates/reposix-cache/` constructs a real on-disk bare git repo from REST responses via the existing `BackendConnector` trait. The cache produces a fully populated tree (filenames, directory structure, blob OIDs) but stores blobs lazily — a blob is only materialized when the helper requests it on behalf of git. The cache is the substrate that `git-remote-reposix` proxies protocol-v2 traffic to. Source: architecture-pivot-summary §2 (How it works), §5 (Add).

- [ ] **ARCH-02**: `reposix-cache` writes one audit row per blob materialization (per OP "audit non-optional"). Cache returns blob bytes wrapped in `reposix_core::Tainted<Vec<u8>>` (per OP "tainted-by-default") so downstream code cannot accidentally route tainted bytes into side-effecting actions on other systems without an explicit `sanitize` step. Audit schema: `(ts, backend, project, issue_id, blob_oid, byte_len, op="materialize")`. The cache table mirrors today's SQLite WAL append-only policy — no UPDATE or DELETE on audit rows.

- [ ] **ARCH-03**: `reposix-cache` enforces `REPOSIX_ALLOWED_ORIGINS` egress allowlist by reusing the single `reposix_core::http::client()` factory. No new `reqwest::Client` construction site is added; clippy `disallowed_methods` already catches direct calls. Tests assert that an attempt to materialize a blob from a backend whose origin is not in the allowlist returns `EPERM`-equivalent and is audited.

### Transport (read path)

- [ ] **ARCH-04**: `git-remote-reposix` advertises the `stateless-connect` capability and tunnels protocol-v2 fetch traffic (handshake, ls-refs, fetch with filter) to the cache's bare repo. The existing `export` capability for push remains alongside `stateless-connect` (hybrid helper, confirmed working in POC `poc/git-remote-poc.py`). Both capabilities live in the same binary; git dispatches based on direction. Source: architecture-pivot-summary §3 (Transport routing).

- [ ] **ARCH-05**: The Rust `stateless-connect` implementation correctly handles all three protocol gotchas surfaced during POC iteration (architecture-pivot-summary §3 "Three protocol gotchas"):
  1. **Initial advertisement does NOT terminate with `0002`** — only `0000` (flush). Rust port must send the bytes from `upload-pack --advertise-refs --stateless-rpc` verbatim, no trailing response-end.
  2. **Subsequent RPC responses DO need trailing `0002`** — after each response pack, write the bytes from `upload-pack --stateless-rpc` followed by the response-end marker.
  3. **Stdin reads in binary mode throughout** — the helper reads the protocol stream via a `BufReader<Stdin>` consistently; mixing text and binary reads corrupts the framing.

  Refspec namespace MUST be `refs/heads/*:refs/reposix/*` (NOT `refs/heads/*:refs/heads/*` — the empty-delta bug from POC). The current `crates/reposix-remote` already uses the correct namespace and must not regress.

### Sync (delta path)

- [ ] **ARCH-06**: `BackendConnector::list_changed_since(timestamp) -> Vec<IssueId>` is added as a trait method. All backends (`SimBackend`, `GithubBackend`, `ConfluenceBackend`, `JiraBackend`) implement it using their native incremental query mechanism: GitHub `?since=<ISO8601>`, Jira `JQL: updated >= "<datetime>"`, Confluence `CQL: lastModified > "<datetime>"`. The simulator implements it by filtering its in-memory issue set against the `since` query parameter exposed in its REST surface. Source: architecture-pivot-summary §4 (Delta sync).

- [ ] **ARCH-07**: Delta sync flow: on `git fetch`, the helper reads `last_fetched_at` from the cache DB, calls `list_changed_since(last_fetched_at)` on the backend, materializes the changed items into the bare-repo cache, advertises the updated tree to git via protocol v2, then atomically writes the new `last_fetched_at` (per architecture-pivot-summary §4 "Fetch flow"). Tree sync is unconditional and not gated by the blob limit (tree metadata is small — full sync is cheap). Cache update and `last_fetched_at` write happen in one SQLite transaction so a crash mid-sync cannot leave divergent state. One audit row per delta-sync invocation: `(ts, backend, project, since_ts, items_returned, op="delta_sync")`.

### Push path

- [ ] **ARCH-08**: Push-time conflict detection lives inside the `export` handler. Flow (architecture-pivot-summary §3 "Conflict detection happens inside `handle_export`"):
  1. Parse the fast-import stream in memory (or via state machine).
  2. For each changed file, fetch the current backend version via REST.
  3. If the backend version differs from the agent's commit base: emit `error refs/heads/main fetch first` (canned status — git renders the standard "perhaps a `git pull` would help" hint) AND a detailed diagnostic via stderr through the existing `diag()` channel.
  4. Reject path drains the incoming stream and never touches the bare cache (no partial state).
  5. On success: apply REST writes, update bare-repo cache, emit `ok refs/heads/main`.

  Audit row for every push attempt (accept and reject): `(ts, backend, project, ref, files_touched, decision, reason)`.

- [ ] **ARCH-09**: Blob limit guardrail. The helper counts `want <oid>` lines per `command=fetch` request and refuses if the count exceeds `REPOSIX_BLOB_LIMIT` (default 200, env-configurable). Refusal writes a stderr error message that *names* the remediation: `"error: refusing to fetch <N> blobs (limit: <M>). Narrow your scope with `git sparse-checkout set <pathspec>` and retry."`. The error message is deliberately self-teaching (dark-factory pattern from architecture-pivot-summary §4 "Blob limit as teaching mechanism" — an agent unfamiliar with reposix observes the error, runs `git sparse-checkout`, and recovers without human prompt engineering).

- [ ] **ARCH-10**: Frontmatter field allowlist on the push path. Server-controlled fields (`id`, `created_at`, `version`, `updated_at`) are stripped from inbound writes BEFORE the REST call, mirroring the policy currently enforced on the FUSE write path. An attacker-authored issue body with `version: 999999` MUST NOT update the server version. The `Tainted<T>` -> `Untainted<T>` conversion is the explicit `sanitize` step where this stripping happens.

### CLI & agent UX

- [ ] **ARCH-11**: `reposix init <backend>::<project> <path>` replaces `reposix mount`. The new command:
  1. Runs `git init <path>`.
  2. Configures `extensions.partialClone=origin`.
  3. Sets `remote.origin.url=reposix::<backend>/<project>`.
  4. Runs `git fetch --filter=blob:none origin`.
  5. Optionally runs `git checkout origin/main` (or leaves the working tree empty for sparse-first agents).

  `reposix mount` is removed in the same release. CHANGELOG `[v0.9.0]` documents the breaking CLI change with a migration note. Source: architecture-pivot-summary §5 (Change).

- [ ] **ARCH-12**: End-to-end agent UX validation. A fresh subprocess agent (no reposix CLI awareness, no in-context system-prompt instructions about reposix) is given ONLY a `reposix init` command and a goal (e.g., "find issues mentioning 'database' and add a TODO comment to each"). The agent succeeds using pure git/POSIX tools: `cd /tmp/repo && cat issues/<id>.md && grep -r TODO . && <edit> && git add . && git commit && git push`. The validation specifically exercises the conflict-rebase cycle (a second writer modifies a backend issue between the agent's `clone` and `push`; agent observes `! [remote rejected] main -> main (fetch first)`, runs `git pull --rebase`, retries `git push`, succeeds). This is the dark-factory acceptance test — see architecture-pivot-summary §4 "Agent UX: pure git, zero in-context learning".

### Demolition & grounding

- [ ] **ARCH-13**: `crates/reposix-fuse/` is deleted entirely. The `fuser` dependency is removed from the workspace `Cargo.toml`. All FUSE-only test feature gates (`fuse-mount-tests`) are removed. CI no longer runs `apt install fuse3` or mounts `/dev/fuse`. After this requirement lands, `cargo metadata --format-version 1 | grep fuser` returns nothing, and `cargo check --workspace && cargo clippy --workspace --all-targets -- -D warnings` is green without any FUSE-related package on the host. Source: architecture-pivot-summary §5 (Delete).

- [ ] **ARCH-14**: Project `CLAUDE.md` is updated as part of the same phase that lands FUSE deletion (NOT a follow-up phase). Specifically:
  - All FUSE references are purged from the elevator pitch, Operating Principles, Workspace layout, Tech stack, "Commands you'll actually use", and the Threat-model table.
  - Any "Architecture transition" or "v0.9.0 in progress" banner is replaced with a steady-state **Architecture (git-native partial clone)** section describing the cache + helper + agent-UX flow.
  - The Tech stack row for `fuser` is removed. The replacement row mentions `git >= 2.34` as a runtime requirement (per architecture-pivot-summary §7 risks).
  - "Commands you'll actually use" no longer mentions `reposix mount` or `cargo test --features fuse-mount-tests`. New commands shown: `reposix init`, `git clone reposix::sim/proj-1 /tmp/repo`.
  - Threat-model table updated: the helper (not the FUSE daemon) is the egress surface. Allowlist now applies to the helper + cache, not the FUSE daemon.

  Per OP-4 self-improving infrastructure: agent grounding must match shipped reality — there can be no window where CLAUDE.md describes deleted code.

- [ ] **ARCH-15**: A project Claude Code skill `reposix-agent-flow` is created at `/home/reuben/workspace/reposix/.claude/skills/reposix-agent-flow/SKILL.md` (Claude Code skill convention — directory with `SKILL.md` and YAML frontmatter `name:` + `description:`). The skill encodes the dark-factory autonomous-agent regression test: spawn a fresh subprocess Claude (or scripted shell agent acting as one) inside an empty directory, hand it ONLY a `reposix init` command and a natural-language goal, and verify the agent completes the task using pure git/POSIX tools — including the conflict-rebase cycle and the blob-limit-induced sparse-checkout recovery from ARCH-09. The skill is invoked from CI (release gate) and from local dev (`/reposix-agent-flow`). The skill MUST mention that it is the StrongDM/dark-factory regression harness for the v0.9.0 architecture and reference architecture-pivot-summary §4 "Agent UX". Per OP-4: ships in the same phase as ARCH-13 and ARCH-14.

### Real-backend validation, performance, and canonical test targets

- [ ] **ARCH-16**: Real-backend smoke tests against the project's canonical test targets. The helper + cache must be validated end-to-end against real APIs, not just the simulator. Targets: Confluence "TokenWorld" space, reposix's own GitHub issues (`reubenjohn/reposix`), JIRA project `TEST` (overridable via `JIRA_TEST_PROJECT` or `REPOSIX_JIRA_PROJECT`). Gated by `REPOSIX_ALLOWED_ORIGINS` + credentials; skipped in CI when creds absent (`#[ignore]` or `skip_if_no_env!`). Must exercise: partial clone, lazy blob fetch, delta sync after upstream mutation, push round-trip, conflict rejection. Simulator-only coverage does NOT satisfy this requirement — at least one real backend must be exercised under the same suite for v0.9.0 to ship.

- [ ] **ARCH-17**: Latency & performance envelope captured for each backend. A committed benchmark script runs the reposix golden path (`clone` → first blob read → batched checkout of 10 blobs → edit → push) against sim + each real backend, records wall-clock latency per step, and writes a Markdown artifact under `docs/benchmarks/v0.9.0-latency.md`. Targets are encoded as soft thresholds (e.g. sim cold clone < 500ms, real backend < 3s). Regressions are flagged but not CI-blocking. This is a **sales asset** — the artifact is the reposix-vs-MCP-vs-REST comparison table that ships in narrative docs (v0.10.0).

- [ ] **ARCH-18**: Canonical test-target documentation. `docs/reference/testing-targets.md` enumerates TokenWorld (Confluence), `reubenjohn/reposix` issues (GitHub), and JIRA project `TEST` — with env-var setup, rate-limit notes, cleanup procedure ("do not leave junk issues"), and the explicit "go crazy, it's safe" permission statement from the owner. CLAUDE.md cross-references this file so any agent looking for sanctioned test targets is one hop away from the truth.

- [ ] **ARCH-19**: CI integration-contract job for each real backend. Three CI jobs (`integration-contract-confluence-v09`, `integration-contract-github-v09`, `integration-contract-jira-v09`) run the ARCH-16 smoke suite when the corresponding secret block is present. Each job writes a run artifact (latency rows from ARCH-17). Pattern mirrors the existing `integration-contract-confluence` job from Phase 11 — `pending-secrets` status when creds unavailable, green when present.

---

## Future Requirements

*(Deferred; emerge from open questions in architecture-pivot-summary §7.)*

- Cache eviction policy for `reposix-cache` (LRU / TTL / per-project disk quota / manual `reposix gc`).
- `import` capability deprecation (kept one release cycle past v0.9.0, removed in v0.10.0 or v0.11.0).
- Stream-parsing performance for `export` — production state-machine parser with commit-or-rollback barrier at fast-import `done` terminator.
- Threat-model update for push-through-export flow — `research/threat-model-and-critique.md` revision once the helper-as-egress-surface model is stable.
- Non-issue file handling on push (e.g., changes to `.planning/` paths) — reject vs. silently commit-to-cache decision.

---

## Out of Scope

- **Bringing FUSE back.** The pivot is one-way; `crates/reposix-fuse/` is removed, not deprecated.
- **Custom CLI for read operations.** Agents use `git clone`, `git fetch`, `cat`, `grep` — no `reposix list`, no `reposix get`. The whole point is zero in-context learning.
- **Full real-backend exercise in autonomous mode.** Real backends require explicit credentials and a non-default `REPOSIX_ALLOWED_ORIGINS`. Autonomous execution validates against the simulator only.
- **Cache eviction implementation in v0.9.0.** Decision deferred per architecture-pivot-summary §7 open question 1; the cache grows monotonically until then.
- **`docs/` rewrite.** Carried forward to v0.10.0 (DOCS-01..09).

---

## Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| ARCH-01 | 31 | planning |
| ARCH-02 | 31 | planning |
| ARCH-03 | 31 | planning |
| ARCH-04 | 32 | planning |
| ARCH-05 | 32 | planning |
| ARCH-06 | 33 | planning |
| ARCH-07 | 33 | planning |
| ARCH-08 | 34 | planning |
| ARCH-09 | 34 | planning |
| ARCH-10 | 34 | planning |
| ARCH-11 | 35 | planning |
| ARCH-12 | 35 | planning |
| ARCH-13 | 36 | planning |
| ARCH-14 | 36 | planning |
| ARCH-15 | 36 | planning |
| ARCH-16 | 35 | planning |
| ARCH-17 | 35 (capture) + 36 (artifact) | planning |
| ARCH-18 | 36 | planning |
| ARCH-19 | 36 | shipped |

*(v0.9.0 ARCH-01..19 all shipped 2026-04-24. DOCS-01..09 (originally deferred from v0.9.0) are owned by the active v0.10.0 section above. DOCS-10 and DOCS-11 are new in v0.10.0.)*
