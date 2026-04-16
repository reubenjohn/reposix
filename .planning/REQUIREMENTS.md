# Requirements — reposix v0.6.0 + v0.7.0

<!-- v0.6.0: Write Path + Full Sitemap -->
<!-- v0.7.0: Hardening + Confluence Expansion -->
<!-- REQ-ID format: [CATEGORY]-[NUMBER] -->

## Active

### Write Path (Confluence)
- [ ] **WRITE-01**: Agent can create a new Confluence page by writing a new `.md` file in the FUSE mount (`ConfluenceBackend::create_issue`)
- [ ] **WRITE-02**: Agent can update a Confluence page by editing its `.md` file in the FUSE mount (`ConfluenceBackend::update_issue`)
- [ ] **WRITE-03**: Agent can delete/close a Confluence page by unlinking its `.md` file (`ConfluenceBackend::delete_or_close`)
- [ ] **WRITE-04**: Page bodies round-trip through `atlas_doc_format` ↔ Markdown conversion with no data loss for headings, paragraphs, and code blocks

### Swarm
- [ ] **SWARM-01**: `reposix-swarm --mode confluence-direct` exercises `ConfluenceBackend` directly (no FUSE overhead), mirroring `SimDirectWorkload` pattern
- [ ] **SWARM-02**: Swarm run against confluence-direct produces summary metrics + audit-log rows, matching the sim-direct output format

### Index Synthesis (OP-2 remainder)
- [x] **INDEX-01**: `cat mount/tree/<subdir>/_INDEX.md` returns a recursive markdown sitemap of that subtree, computed via cycle-safe DFS from `TreeSnapshot`
- [x] **INDEX-02**: `cat mount/_INDEX.md` returns a whole-mount overview listing all backends, buckets, and top-level entry counts

### Directory Views (OP-1 remainder)
- [ ] **NAV-01**: `ls mount/labels/<label>/` lists all issues/pages carrying that label as read-only symlinks pointing to the canonical file in the bucket
- [ ] **NAV-02**: `ls mount/spaces/<key>/` lists all pages in that Confluence space (multi-space mount support)

### Cache Refresh (OP-3)
- [x] **CACHE-01**: `reposix refresh` subcommand re-fetches all pages from the backend and writes a git commit into the mount's working tree
- [ ] **CACHE-02**: `git diff HEAD~1` in the mount shows what changed at the backend since the last refresh (mount-as-time-machine)
- [x] **CACHE-03**: `mount/.reposix/fetched_at.txt` records the timestamp of the last backend round-trip

## Future Requirements

- Multi-space mount: `reposix mount --backend confluence --project '*'` mounts all readable spaces under `spaces/<key>/...`
- `mount/recent/<yyyy-mm-dd>/` time-bucketed view of pages modified on a given day
- `mount/pulls/` namespace for GitHub pull requests
- Offline mode: `--offline` CLI flag guarantees zero egress; cache is authoritative

## Out of Scope (v0.6.0)

- **OP-10 — Eject 3rd-party adapters**: User-gated hard stop; explicitly excluded from this milestone.
- **Phase 12 — Subprocess/JSON-RPC connector ABI**: Not started; user-gated design question open.
- **Confluence attachments / whiteboards / live docs**: Deferred to v0.7.0 (OP-9b).
- **Confluence comments**: Deferred to v0.7.0 (OP-9a).
- **Hardening probes (OP-7)**: Deferred to v0.7.0.
- **Honest-tokenizer benchmarks (OP-8)**: Deferred to v0.7.0.
- **Docs reorg (OP-11)**: Deferred to v0.7.0.

## Traceability

| REQ-ID | Phase |
|--------|-------|
| WRITE-01..04 | Phase 16: Confluence write path |
| SWARM-01..02 | Phase 17: Swarm confluence-direct mode |
| INDEX-01..02 | Phase 18: OP-2 remainder |
| NAV-01..02   | Phase 19: OP-1 remainder |
| CACHE-01..03 | Phase 20: OP-3 |

---

# Milestone v0.7.0 — Hardening + Confluence Expansion

## Active

### Hardening (OP-7)
- [x] **HARD-01**: `reposix-swarm --contention` mode proves `If-Match` 409 is deterministic under N-client contention on same issue
- [x] **HARD-02**: 500-page space truncation emits `WARN` log and `--no-truncate` flag errors instead of silently capping (SG-05 compliance)
- [x] **HARD-03**: Chaos audit-log test: kill -9 sim mid-swarm shows no dangling/torn rows in WAL-mode DB
- [x] **HARD-04**: macFUSE parity: CI matrix entry for macOS with macFUSE, `fusermount3 → umount -f` conditional swap

### Benchmarks (OP-8)
- [ ] **BENCH-01**: `bench_token_economy.py` uses `client.messages.count_tokens()` instead of `len(text)/4`; results cached in `benchmarks/fixtures/*.tokens.json`
- [x] **BENCH-02**: Per-backend comparison table (sim, github, confluence) for token reduction vs raw JSON API
- [x] **BENCH-03**: Cold-mount time-to-first-ls matrix: 4 backends × [10, 100, 500] issues
- [x] **BENCH-04**: `docs/why.md` honest-framing section updated with real tokenization numbers

### Confluence Comments (OP-9a)
- [ ] **CONF-01**: `cat mount/pages/<id>.comments/<comment-id>.md` returns comment body in Markdown frontmatter format
- [ ] **CONF-02**: `ls mount/pages/<id>.comments/` lists all inline + footer comments for that page
- [ ] **CONF-03**: Comments are read-only (no write path in this phase)

### Confluence Whiteboards / Attachments / Folders (OP-9b)
- [ ] **CONF-04**: `ls mount/whiteboards/` lists Confluence whiteboards; each exposed as `<id>.json` (raw)
- [ ] **CONF-05**: `ls mount/pages/<id>.attachments/` lists page attachments; binary passthrough
- [ ] **CONF-06**: Folders (`/folders` endpoint) exposed as a separate tree alongside page hierarchy

### Docs Reorg (OP-11)
- [ ] **DOCS-01**: `InitialReport.md` moved to `docs/research/initial-report.md` with redirect note at old path
- [ ] **DOCS-02**: `AgenticEngineeringReference.md` moved to `docs/research/agentic-engineering-reference.md` with redirect note
- [ ] **DOCS-03**: All cross-refs in `CLAUDE.md`, `README.md`, and planning docs updated to new paths

## Future Requirements

- Confluence live docs (`/custom-content/` type discriminator) exposed as `livedocs/<id>.md`
- `reposix spaces --backend confluence` subcommand to list all readable spaces
- Multi-space mount: `--project '*'` mounts every readable space under `spaces/<key>/...`
- Windows VFS layer (very long-term)

## Out of Scope (v0.7.0)

- **OP-10 — Eject 3rd-party adapters**: User-gated hard stop.
- **Phase 12 — Subprocess/JSON-RPC connector ABI**: Design question still open.
- **Confluence write path for comments/attachments**: Read-only in this milestone.
- **Real Jira adapter**: Not started; no connector ABI yet.

## Traceability

| REQ-ID | Phase |
|--------|-------|
| HARD-01..04  | Phase 21: OP-7 hardening bundle |
| BENCH-01..04 | Phase 22: OP-8 honest-tokenizer benchmarks |
| CONF-01..03  | Phase 23: OP-9a Confluence comments |
| CONF-04..06  | Phase 24: OP-9b whiteboards/attachments/folders |
| DOCS-01..03  | Phase 25: OP-11 docs reorg |
