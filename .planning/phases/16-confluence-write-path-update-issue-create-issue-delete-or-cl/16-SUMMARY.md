---
phase: 16
milestone: v0.6.0
milestone_name: Write Path + Full Sitemap
status: SHIPPED
completed: 2026-04-14
session: 7
waves: [A, B, C, D]
subsystem: reposix-confluence
requirements_closed: [WRITE-01, WRITE-02, WRITE-03, WRITE-04]
tags: [confluence, write-path, adf, audit-log, version-bump, wave-d]
---

# Phase 16 Summary: Confluence Write Path (v0.6.0 milestone start)

## One-liner

Confluence write path shipped: `create_issue`/`update_issue`/`delete_or_close` call real Confluence Cloud REST v2 endpoints; ADF-to-Markdown converter handles round-trips; client-side audit log wires SG-06 to every write; read path upgraded to `atlas_doc_format` with `storage` fallback.

## Phase Identity

| Field | Value |
|-------|-------|
| Phase | 16 |
| Milestone | v0.6.0 "Write Path + Full Sitemap" (milestone-start) |
| Status | SHIPPED |
| Session | 7 (2026-04-14) |
| Waves | A (ADF converter), B (write methods + rename), C (audit log + integration), D (docs + version bump) |
| Subsystem | `reposix-confluence` |

## What Shipped

User-visible behaviors delivered by this phase:

- **Agents can create, update, and delete Confluence pages** via the `IssueBackend`
  trait. Because Phase 14 routes all FUSE writes and `git-remote-reposix` pushes
  through `IssueBackend` dispatch, both paths automatically inherit all three write
  methods with zero FUSE or remote-helper changes.
- **`create_issue`** — `POST /wiki/api/v2/pages` with space-id resolution and optional
  `parentId` support. Returns the new page ID as the created issue's ID.
- **`update_issue`** — `PUT /wiki/api/v2/pages/{id}` with optimistic locking.
  Accepts an explicit `expected_version`; if `None`, fetches the current version
  via a GET first (`fetch_current_version` helper). Returns `Error::VersionMismatch`
  on HTTP 409.
- **`delete_or_close`** — `DELETE /wiki/api/v2/pages/{id}`. Maps HTTP 204 to `Ok(())`,
  HTTP 404 to `Error::NotFound`.
- **Markdown bodies round-trip through ADF** — `markdown_to_storage` (via
  `pulldown-cmark` HTML renderer) converts Markdown to Confluence Storage XHTML on
  writes; `adf_to_markdown` (recursive `serde_json::Value` visitor) converts ADF
  JSON to Markdown on reads. Supports H1–H6, paragraphs, fenced code blocks with
  language attribute, inline code, bullet and ordered lists. Unknown ADF node types
  emit `[unsupported ADF node type=X]` — agents can `grep -r 'unsupported ADF' mount/`
  to detect lossy reads.
- **All writes are auditable** — `ConfluenceBackend::with_audit(conn)` builder accepts
  a `rusqlite::Connection`. Every write call (create, update, delete) inserts one row
  into the `audit_events` table reusing the SG-06 append-only schema from Phase 1.
  Audit is best-effort: a failed insert logs a warning but never masks a successful
  write. Audit rows capture: `method` (POST/PUT/DELETE), `path` (e.g.
  `/wiki/api/v2/pages/99`), `status_code`, `title` (max 256 chars, never body).
- **`ConfluenceBackend` struct rename** — `ConfluenceReadOnlyBackend` renamed to
  `ConfluenceBackend` (pre-1.0 breaking change; all callers updated in the same commit).
  User-Agent header updated to `reposix-confluence/0.6`.
- **`supports()` capability matrix updated** — `Delete` and `StrongVersioning` now
  return `true` (previously only `Hierarchy`).
- **Read path upgraded to ADF format** — `get_issue` requests
  `?body-format=atlas_doc_format`; falls back to `?body-format=storage` when the ADF
  body is absent or null (pre-ADF pages, Confluence data center).

## Requirements Closed

| REQ | Description | Closed by | Test evidence |
|-----|-------------|-----------|---------------|
| WRITE-01 | `create_issue` on `ConfluenceBackend` — POST to REST v2 | Wave B (commit `b905cb0`) | `create_issue_posts_to_pages`, `create_issue_with_parent_id` |
| WRITE-02 | `update_issue` — PUT with optimistic locking + 409 handling | Wave B (commit `b905cb0`) | `update_issue_sends_put_with_version`, `update_issue_409_maps_to_version_mismatch`, `update_issue_none_version_fetches_then_puts` |
| WRITE-03 | `delete_or_close` — DELETE with 204/404 mapping | Wave B (commit `b905cb0`) | `delete_or_close_sends_delete`, `delete_or_close_404_maps_to_not_found` |
| WRITE-04 | ADF ↔ Markdown round-trip converter | Wave A (commit `5c3c273`) + Wave C integration test (commit `3918452`) | `adf::tests::*` (18 unit tests) + `roundtrip.rs::create_then_get_roundtrip_with_audit` |

## Locked Decisions Honored

- **LD-16-01: IssueBackend trait-only routing** — No new public API was added to
  `ConfluenceBackend` except `with_audit`. The three write methods implement the
  existing `IssueBackend` trait signatures exactly; FUSE and the git remote helper
  required zero changes to pick them up. Commit: `b905cb0`.

- **LD-16-02: `Untainted<Issue>` parameter type** — All three write methods accept
  `Untainted<Issue>` in their `IssueBackend` trait signatures. The compiler enforces
  that `sanitize()` was called upstream before any content reaches the Confluence API.
  This is a compile-time enforcement of T-16-B-03 (content injection via frontmatter
  fields). Commit: `b905cb0`.

- **LD-16-03: Best-effort audit on every write path** — `audit_write` is called from
  `create_issue`, `update_issue`, and `delete_or_close` on both success and failure
  paths. Audit failure (SQLite error) is swallowed with a `warn!` log; it never masks
  a successful write. The `audit_records_failed_writes` test locks this down. Commits:
  `34a704c`, `c4614a0`.

## Wave Commits

| Wave | Commit(s) | Description |
|------|-----------|-------------|
| A — ADF converter | `48aec91` | Add `pulldown-cmark` workspace dep |
| A — ADF converter | `5c3c273` | Implement `adf.rs` `markdown_to_storage` + `adf_to_markdown` + 18 unit tests |
| B — Write methods | `59217ba` | Rename `ConfluenceReadOnlyBackend` → `ConfluenceBackend` across workspace |
| B — Write methods | `b905cb0` | Add `supports(Delete\|StrongVersioning)` + `write_headers` helper + implement create/update/delete |
| B — Write methods | `51caac6` | Add 13 wiremock tests for create/update/delete + supports test |
| C — Audit + integration | `b4f538a` | Add `rusqlite` + `sha2` deps to `reposix-confluence` |
| C — Audit + integration | `34a704c` | Add `audit` field + `with_audit` builder + `audit_write` helper; wire into create/update/delete |
| C — Audit + integration | `6504713` | Switch `get_issue` to ADF body format with storage fallback |
| C — Audit + integration | `c4614a0` | Add 6 audit unit tests |
| C — Audit + integration | `3918452` | Add `roundtrip.rs` integration test (WRITE-04 end-to-end) + fmt |
| D — Docs + release | (this commit) | CHANGELOG `[v0.6.0]`, version bump `0.5.0→0.6.0`, `scripts/tag-v0.6.0.sh`, STATE.md cursor, ROADMAP.md |

## Test Counts

| Milestone | Count | Delta |
|-----------|-------|-------|
| Baseline (pre-Phase 16) | 278 | — |
| After Wave A | 296 | +18 |
| After Wave B | 308 | +12 |
| After Wave C | 317 | +9 |
| After Wave D (ship) | **317** | +0 (docs only) |

Net new tests in Phase 16: **+39** (18 ADF unit + 13 write method wiremock + 6 audit unit + 2 integration round-trip).

## Clippy + Fmt Status

| Gate | Wave A | Wave B | Wave C | Wave D |
|------|--------|--------|--------|--------|
| `cargo clippy --workspace --all-targets -- -D warnings` | clean | clean | clean | clean |
| `cargo fmt --all --check` | clean | clean | clean | clean |

All four waves shipped with both gates fully green. No lint suppressions added.

## Deferred to Later Phases

The following items were explicitly out-of-scope for Phase 16 and must NOT be
confused with shipped work:

- **Confluence comments** — Phase 23 (`reposix-confluence` comments API, v0.7.0).
- **Confluence attachments** — Phase 24 (`reposix-confluence` attachments API, v0.7.0).
- **Swarm confluence-direct mode** — Phase 17 (add `--mode confluence-direct` to
  `reposix-swarm` using `SimDirectWorkload` as template, v0.6.0).
- **Tree-level `_INDEX.md` and mount-root `_INDEX.md`** — Phase 18 (OP-2 remainder).
- **`reposix refresh` cache-refresh subcommand** — Phase 20 (OP-3).
- **Live contract test against a real Atlassian tenant** — pending user-driven
  execution via `cargo test --ignored -- live` (requires `ATLASSIAN_API_KEY`,
  `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT` env vars).

## Known Limitations / Documented Tradeoffs

1. **Storage XHTML is plain HTML** — `markdown_to_storage` outputs plain HTML via
   `pulldown-cmark`'s `html::push_html`, not Atlassian-specific
   `<ac:structured-macro>` tags. Agents writing code-block-heavy pages may see
   server-side re-rendering into the macro format. The ADF round-trip (Wave C
   integration test) proves that pages survive a create→get cycle without data loss
   for the tested construct set. Tracked for v0.7.0 hardening (LD-16-A-04 accepted
   risk).

2. **ADF unknown node types emit fallback markers** — `adf_to_markdown` emits
   `[unsupported ADF node type=X]` for node types not in the supported set. Agents
   can detect lossy reads with `grep -r 'unsupported ADF' mount/pages/`. The fallback
   is intentional and searchable — it is not silent data loss.

3. **`version.number = current + 1` convention** — Verified via wiremock mocks in
   Wave B but not yet against a real Atlassian tenant. The live contract test is
   under `--ignored live` and requires a real tenant. See `VALIDATION.md`
   §Manual-Only-Verifications for the exact command.

4. **`fetch_current_version` round-trip** — When `expected_version = None` is passed
   to `update_issue`, a GET round-trip fetches the current version before the PUT.
   This is an accepted extra latency (LD-16-B-04) — the alternative (requiring callers
   to always pass a version) would break the `IssueBackend` trait contract.

5. **Audit title truncation** — Audit rows store `title` truncated to 256 chars
   (LD-16-C-04). Body content is never stored in the audit log, per the T-16-C-04
   threat (audit log body exfil). Titles containing `|` are not escaped in the audit
   column — this is a cosmetic issue, not a security issue.

## Next Post-Phase Gate

User runs `scripts/tag-v0.6.0.sh` after the full green-gauntlet:

```bash
bash scripts/green-gauntlet.sh --full   # or equivalent: cargo test + clippy + smoke.sh
bash scripts/tag-v0.6.0.sh              # creates and pushes the v0.6.0 annotated tag
```

The script enforces all 7 guards (branch=main, clean tree, tag not yet local, tag not
yet remote, CHANGELOG has `[v0.6.0]`, `Cargo.toml` version is `0.6.0`, tests + smoke
green) before pushing. Wave D executor does NOT invoke it.

After tagging: Phase 17 (swarm confluence-direct) is the recommended next execution
target.

## Self-Check: PASSED

- Wave A commits `48aec91`, `5c3c273` present in git log
- Wave B commits `59217ba`, `b905cb0`, `51caac6` present in git log
- Wave C commits `b4f538a`, `34a704c`, `6504713`, `c4614a0`, `3918452` present in git log
- `crates/reposix-confluence/src/adf.rs` exists
- `crates/reposix-confluence/tests/roundtrip.rs` exists
- `scripts/tag-v0.6.0.sh` exists and is executable
- `CHANGELOG.md` contains `[v0.6.0]` section
- `Cargo.toml` workspace version = `0.6.0`
- 317 workspace tests (verified by `cargo test --workspace`)
- Clippy `-D warnings` clean, `cargo fmt --all --check` clean
- No `atlassian.net` domains outside code-sample blocks (T-16-D-03 check)
- No real email/token strings (only `test@example.com` fixture values)
