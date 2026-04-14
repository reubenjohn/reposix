# Changelog

All notable changes to reposix are documented here.
The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versions follow [SemVer](https://semver.org/spec/v2.0.0.html) once the project leaves alpha.

## [Unreleased]

— Nothing yet.

## [v0.4.0] — 2026-04-14

The "nested mount layout" cut. OP-1 from the v0.3 HANDOFF.md (the
"folder structure inside the mount" ask) ships, and Confluence's native
`parentId` hierarchy becomes a navigable directory tree backed by FUSE
symlinks. The v0.3 flat `<padded-id>.md` layout is retained under a
per-backend bucket (`pages/` or `issues/`), and a synthesized
read-only `tree/` overlay appears alongside when the backend exposes
hierarchy.

### BREAKING

- **FUSE mount layout reshuffled.** Previously, issues/pages rendered as
  `<padded-id>.md` at the mount root. They now render under a per-backend
  collection bucket: `issues/<padded-id>.md` for sim + GitHub,
  `pages/<padded-id>.md` for Confluence. Callers that `cat mount/0001.md`
  must switch to `cat mount/issues/0001.md` (or `mount/pages/...`). Run
  `ls mount/` to discover the bucket. Every doc, demo, test, and example
  in this release has been migrated; see [ADR-003](docs/decisions/003-nested-mount-layout.md)
  for the full design.
- **On-disk filename padding widened from 4 digits to 11.** Files in the
  bucket are now `{:011}.md` (e.g., `00000131192.md`) to accommodate
  astronomical Confluence page IDs and match the symlink-target
  construction in `TreeSnapshot` byte-for-byte. Anything pinning the old
  `{:04}.md` shape (custom scripts, `git-remote-reposix` fast-import
  blobs) must migrate.

### Added

- **`tree/` overlay exposes Confluence's native parentId hierarchy.** When
  mounting a Confluence space, a synthesized, read-only `tree/`
  subdirectory appears alongside `pages/`, containing symlinks at
  human-readable slug paths. Navigate the wiki with `cd`. (OP-1 from v0.3
  HANDOFF.md — the "hero.png" promise.) GitHub and sim mounts don't emit
  `tree/` — those backends don't expose parent metadata.
- **`Issue::parent_id: Option<IssueId>`** field on the core type;
  Confluence backend populates from REST v2 `parentId`. `#[serde(default)]`
  so legacy frontmatter files on disk parse unchanged.
- **`IssueBackend::root_collection_name`** trait method — default
  `"issues"`, Confluence returns `"pages"`.
- **`BackendFeature::Hierarchy`** capability variant — Confluence overrides
  to `true`, everyone else defaults to `false`.
- **Mount-level `.gitignore`** auto-emitted containing `/tree/` so the
  derived overlay never enters `git add` / `git push`. Compile-time const
  bytes, 0o444 perm, write-gated by inode-kind classifier.
- **`reposix_core::path::slugify_title`** + `slug_or_fallback` helpers —
  Unicode-lowercase → non-alphanumeric collapse → 60-byte truncate →
  fallback to `page-<11-digit-padded-id>` on empty/all-dash results.
  Sibling slug collisions resolved deterministically (ascending `IssueId`
  keeps bare slug, rest get `-N` suffix).
- **`_self.md` convention** — a page with ≥1 child becomes a directory
  named by the parent's slug; the parent's own body is exposed as a
  symlink `_self.md` inside that directory. POSIX-correct; `_self` is
  reserved by construction since `slugify_title` strips leading `_`.
- **ADR-003** documents the `pages/` + `tree/` symlink design and
  supersedes ADR-002's flat-layout decision.
- **Release script `scripts/tag-v0.4.0.sh`** + Confluence-tree demo
  `scripts/demos/07-mount-real-confluence-tree.sh`.
- **Prebuilt Linux binaries attached to GitHub releases.** The
  `.github/workflows/release.yml` workflow (landed alongside v0.3 but
  unannounced in the CHANGELOG until now — correcting the OP-12 gap):
  x86_64 and aarch64 tarballs plus `SHA256SUMS` are attached to every
  tag push matching `v*.*.*`. Users no longer need a Rust toolchain for
  the read-only workflow.

### Changed

- **CLAUDE.md tech-stack section** — corrected `fuser` version from 0.15
  to 0.17 (the actual workspace-resolved version; 0.15 was stale from an
  earlier phase). Reason-comment (no `libfuse-dev` needed on the dev host)
  unchanged.
- **CLAUDE.md Operating Principle #1** — refreshed wording. The old "v0.2
  gate" language is stale (v0.2, v0.3, v0.4 have all shipped real
  backends). New language clarifies that the simulator is the default/test
  backend, and that real-backend calls are gated by `REPOSIX_ALLOWED_ORIGINS`
  + explicit credential env vars.
- **README Quickstart** — leads with the prebuilt-binary path (curl + tar)
  with from-source kept as a contributor secondary option. Adds a new
  "Folder-structure mount (v0.4+)" section linking to ADR-003.

### Hardening

The Phase 13 code review surfaced six polish items, all addressed pre-tag:
- **Symlink `mtime` stability** — `symlink_attr` now uses a cached `mount_time`
  instead of `SystemTime::now()` on every `getattr`. Fixes drifting `st_mtim`
  that confuses rsync/make/backup tools. (`IN-03`)
- **Tree overlay refresh on first touch** — `readdir(tree/)` now triggers the
  same backend refresh the root/bucket readdirs do, eliminating an empty-tree
  trap when a user's first command is `ls mount/tree/`. (`WR-01`)
- **Symlink target `PATH_MAX` guard** — debug-assert + release-mode warn if a
  depth-correct symlink target ever exceeds 4095 bytes. Unreachable under the
  current 500-page Confluence cap, defensive against future bumps. (`WR-02`)
- **Confluence tracing cap** — attacker-controlled `parentId` / `parentType`
  strings are now truncated to 64 bytes before being emitted in a tracing
  span. Defense against log-injection and storage amplification. (`IN-01`)
- **Slugify input pre-cap** — `slugify_title` now bounds the intermediate
  `to_lowercase()` allocation at ~240 chars so pathological 10 MB titles no
  longer briefly balloon memory. Output is unchanged (still 60-byte cap).
  (`IN-02`)

### Security

- **Credential-hygiene pre-push hook** — `scripts/hooks/pre-push` rejects any
  push containing a literal `ATATT3…` (Atlassian API token prefix),
  `Bearer ATATT3…`, `ghp_…` (GitHub classic PAT), or `github_pat_…`
  (fine-grained PAT) in the outgoing ref range. Install via
  `bash scripts/install-hooks.sh`. Guides operators to rotate the token
  rather than bypass the check. (OP-7 from v0.3 HANDOFF.md.)
- **SSRF regression tests** — three new tests in `reposix-confluence`
  prove the adapter ignores attacker-controlled `_links.base`,
  `webui_link`, `_links.webui`, `_links.tinyui`, `_links.self`, and
  `_links.edit` fields even when the adversarial URL resolves. Each test
  uses a decoy `MockServer` with `.expect(0)` so a future "follow the
  link for screenshots" feature regression panics at drop with a
  specific URL list. (OP-7.)
- **`docs/security.md` refreshed for v0.4.** The v0.1-era "no real
  backend, no real victims" framing is replaced with the current model
  (three real backends behind one allowlist; `tree/` overlay security
  properties tested; swarm harness shipped).

### Migration

```bash
# v0.3 and earlier:
cat /tmp/mnt/0001.md

# v0.4+:
ls /tmp/mnt                       # discover the bucket
cat /tmp/mnt/issues/00000000001.md    # sim / GitHub
cat /tmp/mnt/pages/00000131192.md     # Confluence

# Recommended: install the credential-hygiene pre-push hook
bash scripts/install-hooks.sh
```

## [v0.3.0] — 2026-04-14

The "real Confluence" cut. v0.2.0-alpha landed the real-GitHub read path; this
release extends the same `IssueBackend` seam to Atlassian Confluence Cloud
REST v2. Same kernel path, same CLI surface, same SG-01 allowlist — just
a different backend plugged into the trait.

### BREAKING

- `.env` variable `TEAMWORK_GRAPH_API` renamed to `ATLASSIAN_API_KEY`. Users
  with an existing `.env` must rename the variable (or re-source after
  updating from `.env.example`). No reposix binary reads `TEAMWORK_GRAPH_API`
  anymore. The rename reflects that the adapter talks to the generic
  Confluence Cloud REST API, not the more specific Teamwork Graph product.

### Added

- **`crates/reposix-confluence`** — read-only Atlassian Confluence Cloud REST v2
  adapter implementing `IssueBackend`. Basic auth via
  `Authorization: Basic base64(email:api_token)` (no Bearer path — Atlassian
  user tokens cannot be used as OAuth Bearer). Cursor-in-body pagination via
  `_links.next` (capped at 500 pages per `list_issues`; server-supplied
  `_links.base` is deliberately ignored to prevent SSRF). Shared rate-limit
  gate armed by 429 `Retry-After` or `x-ratelimit-remaining: 0`. Tenant
  subdomain validated against DNS-label rules (`[a-z0-9-]`, 1..=63 chars)
  before any request is made. `ConfluenceCreds` has a manual `Debug` impl
  that redacts `api_token` — `#[derive(Debug)]` would leak the token into
  every tracing span.
- **`reposix list --backend confluence`** — `reposix list --backend confluence
  --project <SPACE_KEY>` reads a real Confluence space end-to-end. `--project`
  takes the human-readable space key (e.g. `REPOSIX`); the adapter resolves it
  to Confluence's numeric `spaceId` internally via
  `GET /wiki/api/v2/spaces?keys=...`.
- **`reposix mount --backend confluence`** — the FUSE daemon now mounts a
  Confluence space as a POSIX directory of `<padded-id>.md` files. Same
  `Mount::open(Arc<dyn IssueBackend>)` path as the `github` and `sim`
  backends; SG-07 read-path ceiling (5s get / 15s list) applies unchanged.
- **Env vars: `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`,
  `REPOSIX_CONFLUENCE_TENANT`** — three new required env vars for the
  Confluence backend. The CLI fails fast with a single error listing every
  missing variable.
- **`scripts/demos/parity-confluence.sh`** (Tier 3B demo) — `reposix list`
  against the simulator and `reposix list --backend confluence` against a
  real tenant, normalized via `jq`, key-set asserted equal. Structure
  identical, content differs. Skips cleanly if any Atlassian env var is
  unset.
- **`scripts/demos/06-mount-real-confluence.sh`** (Tier 5 demo) —
  FUSE-mounts a real Confluence space, `cat`s the first listed page, and
  unmounts. Token and email are never echoed — only tenant host + space key
  + allowlist appear on stdout. Skips cleanly with `SKIP:` if any
  Atlassian env var is unset.
- **`crates/reposix-confluence/tests/contract.rs`** — same contract
  assertions run against `SimBackend` (always), a wiremock-backed
  `ConfluenceReadOnlyBackend` (always), and a live
  `ConfluenceReadOnlyBackend` (`#[ignore]`-gated, opt-in via
  `cargo test -p reposix-confluence -- --ignored`).
- **`docs/decisions/002-confluence-page-mapping.md`** — ADR for the Option-A
  flatten decision. Documents the per-field page→issue mapping, the lost
  metadata (parentId, spaceKey, `_links.webui`, `atlas_doc_format`, labels),
  the Basic-auth-only rationale, and the cursor + rate-limit decisions.
- **`docs/reference/confluence.md`** — user-facing guide for the backend:
  CLI surface, env var table with dashboard sources, step-by-step credential
  setup, failure-modes table (`FAILURE_CLIENT_AUTH_MISMATCH` is the
  common one).
- **`docs/connectors/guide.md`** — "Building your own connector" guide. The
  short-term (v0.3) published-crate model: publish
  `reposix-adapter-<name>` on crates.io, fork reposix, add a few lines of
  dispatch. Worked examples: `reposix-github` and `reposix-confluence`.
  Five non-negotiable security rules for adapter authors (HttpClient,
  `Tainted<T>`, redacted `Debug`, tenant validation, rate-limit gate).
  Previews the Phase 12 subprocess/JSON-RPC connector ABI as the scalable
  replacement.
- **CI job `integration-contract-confluence`** — runs the contract test
  against a real Atlassian tenant on every push when the four Atlassian
  secrets (`ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`,
  `REPOSIX_CONFLUENCE_TENANT`, `REPOSIX_CONFLUENCE_SPACE`) are configured.
  Skips cleanly when they're not.

### Changed

- **`.env.example`** — renamed `TEAMWORK_GRAPH_API` → `ATLASSIAN_API_KEY`.
  Added `ATLASSIAN_EMAIL` and `REPOSIX_CONFLUENCE_TENANT`. Inline comments
  now describe the account-scoped-token failure mode so contributors hit
  it and fix it without needing to read ADR-002 cold. **Breaking change for
  anyone with `TEAMWORK_GRAPH_API` in their shell** — it is no longer read
  by any reposix binary.
- **Workspace manifest** — adds the `reposix-confluence` crate and
  `base64 = "0.22"` as a workspace dep.

### Fixed

- **FUSE read-path empty-body on list-only backends.** `readdir`
  pre-populated the in-memory cache with issues from `backend.list_issues()`
  so that subsequent `cat` served from cache. For GitHub that was fine —
  its REST list endpoint returns issue bodies — but Confluence REST v2's
  `/spaces/{id}/pages` returns `body: {}` on every item by design, which
  meant `cat <mount>/<page-id>.md` showed frontmatter only. Added
  `CachedFile.body_fetched: bool`; list-populated entries set it false and
  `resolve_name`/`resolve_ino` fall through to a fresh `get_issue` on first
  read. GitHub's fast-cache behaviour is preserved. Surfaced during live
  Tier 5 verification against a real Confluence tenant; fix verified by
  re-running `bash scripts/demos/06-mount-real-confluence.sh` end-to-end
  against `reuben-john.atlassian.net` space `REPOSIX` and seeing full
  HTML bodies render in `cat` output. File size proof: 198 → 447 bytes
  for the Welcome page.
- **Phase 11 code-review WR-01 — space-key query-string injection.** The
  `--project` value was interpolated into `?keys=…` via `format!` without
  percent-encoding; a space key containing `&`, `=`, `#`, space or non-ASCII
  could have smuggled extra query params. Now built via
  `url::Url::query_pairs_mut().append_pair()`.
- **Phase 11 code-review WR-02 — server-controlled `space_id` in URL
  path.** The numeric-looking `space_id` returned by `?keys=` lookup was
  spliced verbatim into a URL path. A malicious (or compromised) Confluence
  server could have returned `"12345/../../admin"` to pivot onto unintended
  endpoints. Now validated as `^[0-9]{1,20}$` before URL construction; an
  adversarial `id` returns `Error::Other("malformed space id from server")`.

## [v0.2.0-alpha] — 2026-04-13 (post-noon)

The "real GitHub" cut. v0.1.0 was simulator-only on purpose; this release lets reposix talk to a real backend.

### Added

- **`crates/reposix-github`** — read-only GitHub Issues adapter implementing `IssueBackend`. Honors the `x-ratelimit-remaining` / `x-ratelimit-reset` headers via a `Mutex<Option<Instant>>` rate-limit gate (next call sleeps until reset, capped at 60s). Allowlist-aware: callers must set `REPOSIX_ALLOWED_ORIGINS=...,https://api.github.com`. Auth via optional `Bearer` token; `None` falls back to anonymous (60/hr).
- **`reposix list --backend github`** — `reposix list --backend github --project owner/repo` reads real GitHub issues end-to-end via the CLI. The IssueBackend trait, abstract since v0.1, is now reachable from the shipped binary.
- **`reposix mount --backend github`** (Phase 10) — the FUSE daemon now speaks `IssueBackend` directly: `reposix mount /tmp/mnt --backend github --project owner/repo` exposes a real GitHub repo's issues as `<padded-id>.md` files via the same kernel path the simulator uses. Read-path SG-07 ceiling split into `READ_GET_TIMEOUT = 5s` (per-issue) and `READ_LIST_TIMEOUT = 15s` (paginated list) to absorb GitHub's cold-cache pagination without giving up the kernel-non-blocked invariant. `Mount::open` now takes an `Arc<dyn IssueBackend>`; `MountConfig` keeps the simulator REST origin for the write path (cleanup deferred to v0.3).
- **`scripts/demos/05-mount-real-github.sh`** (Tier 5 demo) — mounts `octocat/Hello-World` via `reposix mount --backend github`, lists the files, `cat`s issue #1's frontmatter, and unmounts. Skips cleanly if `gh auth token` is empty. Documented in `docs/demos/index.md` and the README's Tier-5 subsection.
- **`scripts/demos/parity.sh`** (Tier 3 demo) — `reposix list` against the simulator and `gh api` against `octocat/Hello-World`, normalized via `jq`, diffed line-by-line. The diff IS the proof: structure identical, only content differs.
- **`crates/reposix-github/tests/contract.rs`** — same five invariants run against `SimBackend` (always) and `GithubReadOnlyBackend` (`#[ignore]`-gated, opt-in via `cargo test -- --ignored`). The simulator lying becomes a CI failure.
- **`docs/decisions/001-github-state-mapping.md`** — ADR for how reposix's 5-valued `IssueStatus` round-trips through GitHub's `state + state_reason + label` model. Documents label-precedence tiebreak (review wins over progress).
- **`crates/reposix-swarm`** — adversarial swarm harness. `reposix-swarm --clients 50 --duration 30s --mode {sim-direct,fuse}` spawns N concurrent agents, each running `list + 3×read + 1×patch`. Reports HDR-histogram p50/p95/p99 per op, error counts, and asserts the SG-06 append-only audit invariant under load. Validated locally at 132,895 ops / 0% errors.
- **`scripts/demos/swarm.sh`** (Tier 4 demo) — wraps the swarm with sane defaults; tunable via `SWARM_CLIENTS` and `SWARM_DURATION` env vars. Not in CI smoke (too expensive per push).
- **CI: `integration-contract` job** — runs the contract test against real GitHub on every push, authenticated via `${{ secrets.GITHUB_TOKEN }}` (1000/hr ceiling, ample for per-push). Load-bearing (`continue-on-error: false`).
- **CI: `demos-smoke` job** — runs the four Tier 1 demos via `scripts/demos/smoke.sh` on every push. Load-bearing.
- **Codecov badge in README** — `CODECOV_TOKEN` was wired in v0.1.x; the badge just hadn't been added.

### Changed

- **`scripts/demo.sh` is now a shim** that execs `scripts/demos/full.sh`. Old documentation links continue to work; new structure is `scripts/demos/{01..04, full, parity, swarm}.sh`.
- **`continue-on-error: true` flipped off** on `integration (mounted FS)` (passing reliably since Phase 3) and `integration-contract` (now authenticated).
- **`SimBackend::with_agent_suffix`** — Phase 9 needed per-client agent IDs to dodge per-token rate limits during swarm runs.
- **`MORNING-BRIEF.md`, `PROJECT-STATUS.md`, `docs/why.md`** updated to mention real-GitHub via CLI, new test counts, deferred items.

### Fixed

- **Phase S H-01 / H-02** — `git-remote-reposix` `ProtoReader` now reads raw bytes instead of UTF-8 strings; CRLF round-trips, non-UTF-8 doesn't tear the protocol pipe.
- **Phase S H-03** — backend errors during `import`/`export` emit a proper `error refs/heads/main <kind>` line on the protocol stream instead of `?`-propagating into a torn pipe.
- **Phase S M-03** — `git-remote-reposix` `diff::plan` now does normalized-compare on blob bytes (parses both sides as `Issue`, compares values) so trailing-newline drift no longer emits phantom PATCHes for unchanged trees.
- **Phase 8 H-01 / L-02** — `reposix-core::http::HttpClient` is a sealed newtype around `reqwest::Client`; the inner client is unreachable, blocking allowlist-bypass via `client.get(url).send()`. `OriginGlob::parse` migrated to `url::Url::parse` so bracketed IPv6 (`http://[::1]:7777`) works.
- **Phase 8 H-02** — `reposix-core::audit::open_audit_db` enables `SQLITE_DBCONFIG_DEFENSIVE`, blocking the `writable_schema=ON` + `DELETE FROM sqlite_master` schema-attack bypass on the append-only audit invariant.
- **Phase 8 M-04** — `audit.sql` schema bootstrap wrapped in `BEGIN; ... COMMIT;` so the `DROP TRIGGER IF EXISTS; CREATE TRIGGER` race window can't be exploited by a concurrent UPDATE.
- **Phase 8 H-04 (CI)** — `reposix sim --no-seed` and `--rate-limit` flags actually plumb through to the spawned subprocess (were silently dropped).
- **Phase 8 H-01 (CI)** — `reposix demo` step 5 (audit-tail) used to short-circuit because the spawned sim was ephemeral; now uses an on-disk DB so the tail prints real rows. Also fixed the SQL column name (`agent` → `agent_id`).
- **Phase 8 LR-02** — `GithubReadOnlyBackend` now actually backs off on rate-limit exhaustion, not just logs.
- **Phase 8 MR-03** — `SimBackend::update_issue` "wildcard" test now uses a custom `wiremock::Match` impl that fails if `If-Match` is sent, not just a permissive matcher.
- **Phase 8 MR-05 (CI)** — `demos-smoke` and `integration-contract` jobs have explicit `timeout-minutes` so a stuck job can't block the next push indefinitely.

### Deferred to v0.3 / future

- Write path on `GithubReadOnlyBackend` — currently `create_issue` / `update_issue` / `delete_or_close` return `NotSupported`. v0.3.
- FUSE write-path callbacks (`release` PATCH, `create` POST) still speak the simulator's REST shape directly via `crates/reposix-fuse/src/fetch.rs`. The Phase-10 read-path rewire onto `IssueBackend` is shipped; lifting the write path is a v0.3 cleanup once `IssueBackend::create_issue`/`update_issue` exist on every adapter we want to support writing to.
- `git-remote-reposix` still hardcodes the simulator. The `IssueBackend` trait makes the rewire mechanical; tracked for v0.3.
- Adversarial swarm in CI (small-scope) — currently `swarm.sh` is excluded from `demos-smoke` because 30s per push is expensive.
- Jira and Linear adapters.

## [v0.1.0] — 2026-04-13 (~05:38)

Initial autonomous overnight build. Tagged at `v0.1.0`.

### Shipped

- Five-crate Rust workspace (`reposix-{core,sim,fuse,remote,cli}`).
- 8 security guardrails (SG-01 through SG-08), each enforced and tested.
- FUSE daemon (`fuser` 0.17, no `libfuse-dev` linkage required) supporting read + write.
- `git-remote-reposix` git remote helper supporting `import` / `export` capability with bulk-delete cap (SG-02).
- In-process axum REST simulator with rate-limit middleware, ETag-based 409 path, append-only SQLite audit log.
- `scripts/demo.sh` end-to-end walkthrough + `script(1)` recording showing three guardrails firing on camera (SG-01 allowlist refusal, SG-02 bulk-delete cap, SG-03 sanitize-on-egress).
- MkDocs documentation site at <https://reubenjohn.github.io/reposix/> with 11 mermaid architecture diagrams.
- `benchmarks/RESULTS.md` — measured **92.3% reduction** in input-context tokens (reposix vs MCP for the same task).
- 139 workspace tests passing, `cargo clippy --workspace --all-targets -- -D warnings` clean, `#![forbid(unsafe_code)]` at every crate root.
- CI green: rustfmt + clippy + test + coverage + integration mount.

### Notable design decisions

- Simulator-first per the StrongDM "dark factory" pattern (see `AgenticEngineeringReference.md` §1).
- Adversarial red-team subagent at T+0 — security guardrails became first-class requirements before any code was written.
- Rust over Python for the production substrate; FUSE perf matters under swarm load.
- `fuser` with `default-features = false` to avoid the `libfuse-dev` apt dep on hosts without sudo.

See [PROJECT-STATUS.md](PROJECT-STATUS.md) for the timeline + outstanding items, and [MORNING-BRIEF.md](MORNING-BRIEF.md) for the executive summary.

---

[Unreleased]: https://github.com/reubenjohn/reposix/compare/v0.4.0...HEAD
[v0.4.0]: https://github.com/reubenjohn/reposix/compare/v0.3.0...v0.4.0
[v0.3.0]: https://github.com/reubenjohn/reposix/compare/v0.2.0-alpha...v0.3.0
[v0.2.0-alpha]: https://github.com/reubenjohn/reposix/compare/v0.1.0...v0.2.0-alpha
[v0.1.0]: https://github.com/reubenjohn/reposix/releases/tag/v0.1.0
