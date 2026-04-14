# Requirements — reposix v0.6.0 Write Path + Full Sitemap

<!-- Scoped requirements for milestone v0.6.0. -->
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
- [ ] **INDEX-01**: `cat mount/tree/<subdir>/_INDEX.md` returns a recursive markdown sitemap of that subtree, computed via cycle-safe DFS from `TreeSnapshot`
- [ ] **INDEX-02**: `cat mount/_INDEX.md` returns a whole-mount overview listing all backends, buckets, and top-level entry counts

### Directory Views (OP-1 remainder)
- [ ] **NAV-01**: `ls mount/labels/<label>/` lists all issues/pages carrying that label as read-only symlinks pointing to the canonical file in the bucket
- [ ] **NAV-02**: `ls mount/spaces/<key>/` lists all pages in that Confluence space (multi-space mount support)

### Cache Refresh (OP-3)
- [ ] **CACHE-01**: `reposix refresh` subcommand re-fetches all pages from the backend and writes a git commit into the mount's working tree
- [ ] **CACHE-02**: `git diff HEAD~1` in the mount shows what changed at the backend since the last refresh (mount-as-time-machine)
- [ ] **CACHE-03**: `mount/.reposix/fetched_at.txt` records the timestamp of the last backend round-trip

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
