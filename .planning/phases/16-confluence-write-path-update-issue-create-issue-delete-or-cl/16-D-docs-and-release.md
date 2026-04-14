# Plan: Wave D — Docs, version bump to v0.6.0, Phase 16 SUMMARY.md, STATE.md cursor update

## Goal

Ship the human-readable artifacts that make Phase 16 a real v0.6.0 release: CHANGELOG entry, workspace version bump `0.5.0 → 0.6.0`, Phase 16 SUMMARY with traceability from requirements to commits, STATE.md cursor, and a cloned `scripts/tag-v0.6.0.sh` tag-push script. No code changes in this wave — everything is docs + metadata.

## Wave

D (depends on: Waves A + B + C merged and green; unblocks: user-driven `scripts/tag-v0.6.0.sh` push).

## Addresses

- Milestone v0.6.0 release gate — "Write Path + Full Sitemap" kickoff.
- STATE.md cursor advancement so session-7+ agents know Phase 16 is SHIPPED.
- Traceability: every requirement (WRITE-01..04) and locked decision (LD-16-01..03) has a paragraph in SUMMARY.md pointing to the wave + commits that landed it.

## Tasks

### D1. Update CHANGELOG.md with v0.6.0 entry

- **File:** `CHANGELOG.md`
- **Action:** Add a new top-level section **above** the existing `[v0.5.0]` entry:
  ```markdown
  ## [v0.6.0] — 2026-04-XX

  ### Added
  - Confluence write path: `ConfluenceBackend::create_issue`,
    `update_issue`, and `delete_or_close` now emit real HTTP calls against
    the Confluence Cloud REST v2 API (POST/PUT/DELETE `/wiki/api/v2/pages`).
    Covers REQ WRITE-01, WRITE-02, WRITE-03. (Phase 16 Waves B+C.)
  - ADF ↔ Markdown converter (`crates/reposix-confluence/src/adf.rs`) —
    hand-rolled, no external ADF crate. Supports H1–H6, paragraphs,
    fenced code blocks with language attribute, inline code, bullet
    and ordered lists. Unknown node types emit a `[unsupported ADF node
    type=X]` fallback marker so agents can detect lossy reads with `grep`.
    Covers REQ WRITE-04. (Phase 16 Wave A.)
  - Client-side audit log on `ConfluenceBackend` via
    `with_audit(conn)` builder — every write call inserts one row into
    the `audit_events` table (reuses the SG-06 append-only schema from
    Phase 1). Best-effort: audit failure never masks a successful write.
    Locked decision LD-16-03. (Phase 16 Wave C.)
  - `ConfluenceBackend::supports()` now returns `true` for
    `BackendFeature::Delete` and `BackendFeature::StrongVersioning`
    (previously only `Hierarchy`).

  ### Changed
  - **BREAKING (pre-1.0):** `reposix_confluence::ConfluenceReadOnlyBackend`
    renamed to `ConfluenceBackend`. No compatibility alias — callers in
    `reposix-cli`, `reposix-fuse`, and integration tests were updated in
    the same commit. User-Agent header updated to `reposix-confluence/0.6`.
  - Confluence read path now requests `?body-format=atlas_doc_format`
    (was `?body-format=storage`) and runs the ADF→Markdown converter on
    the result; falls back to `?body-format=storage` when the ADF body
    is empty (covers Confluence pages that predate ADF).

  ### Security
  - SG-06 now covers the Confluence write path: every write is auditable,
    audit rows are append-only (triggers from
    `reposix-core/fixtures/audit.sql`), audit failures log-and-swallow.
  - LD-16-02: all write methods take `Untainted<Issue>` — the trait
    signature enforces that `sanitize()` was called upstream.
  ```
- **Expected line impact:** ~35 lines added to CHANGELOG.md.
- **Verification:** `rg '\\[v0\\.6\\.0\\]' CHANGELOG.md` returns ≥1 hit.

### D2. Bump workspace version `0.5.0 → 0.6.0`

- **Files:**
  - `Cargo.toml` (workspace) — line 15, `version = "0.5.0"` → `"0.6.0"`.
  - `Cargo.lock` — regenerate via `cargo check --workspace`.
- **Action:** Edit the single line in root `Cargo.toml`; run `cargo check --workspace` to regen the lockfile. Do NOT hand-edit `Cargo.lock`.
- **Expected line impact:** 1 line in Cargo.toml; Cargo.lock diff large but automated.
- **Verification:** `cargo metadata --format-version 1 --no-deps | jq -r '.packages[] | select(.name == "reposix-core") | .version'` → `0.6.0`. All 8 workspace crates report the new version.

### D3. Clone `scripts/tag-v0.5.0.sh` → `scripts/tag-v0.6.0.sh`

- **Files (new):** `scripts/tag-v0.6.0.sh`
- **Action:** Copy `scripts/tag-v0.5.0.sh` to `scripts/tag-v0.6.0.sh`. Update inside the new file:
  - Every `v0.5.0` string → `v0.6.0`.
  - Every `0.5.0` literal → `0.6.0`.
  - The tag message / CHANGELOG excerpt should reference v0.6.0's "Write Path + Full Sitemap" milestone and summarize Phase 16 (Confluence write path landed; swarm + OP-2/OP-1/OP-3 are still queued under this milestone — note that v0.6.0 is a *milestone-start* tag, not a milestone-complete tag, if that's how the existing v0.5.0 script framed it; otherwise match the existing convention).
  - `chmod +x scripts/tag-v0.6.0.sh`.
- **Expected line impact:** ~40–80 lines (depends on size of tag-v0.5.0.sh; diff near-zero vs source).
- **Verification:** `bash -n scripts/tag-v0.6.0.sh` — syntax check; `[ -x scripts/tag-v0.6.0.sh ]` — executable bit. Do NOT actually run it in this wave (user-gated per Phase 15 convention).

### D4. Write Phase 16 SUMMARY.md

- **File (new):** `.planning/phases/16-confluence-write-path-update-issue-create-issue-delete-or-cl/16-SUMMARY.md`
- **Action:** Follow the structure of `.planning/phases/15-.../15-SUMMARY.md`. Required sections:
  1. **Header** — phase identity, milestone (v0.6.0), status (SHIPPED), session and date.
  2. **What shipped** — bulleted list of user-visible behaviors. Minimum bullets:
     - Agents can create/update/delete Confluence pages via the `IssueBackend` trait (FUSE + git-remote paths automatically inherit this via Phase 14's trait routing).
     - Markdown bodies round-trip through ADF↔Markdown with fallback markers on unknown node types.
     - All writes are auditable via optional `ConfluenceBackend::with_audit(conn)`.
  3. **Requirements closed:**
     | REQ | Description | Closed by | Test evidence |
     | --- | --- | --- | --- |
     | WRITE-01 | `create_issue` | Wave B | `create_issue_posts_to_pages`, `create_issue_with_parent_id` |
     | WRITE-02 | `update_issue` | Wave B | `update_issue_sends_put_with_version`, `update_issue_409_maps_to_version_mismatch`, `update_issue_none_version_fetches_then_puts` |
     | WRITE-03 | `delete_or_close` | Wave B | `delete_or_close_sends_delete`, `delete_or_close_404_maps_to_not_found` |
     | WRITE-04 | ADF↔Markdown round-trip | Wave A + Wave C | `adf::tests::*` + `roundtrip.rs` integration test |
  4. **Locked decisions honored:**
     - LD-16-01: IssueBackend trait-only routing — no new public API on `ConfluenceBackend` except `with_audit`.
     - LD-16-02: `Untainted<Issue>` parameter type enforces upstream `sanitize()` call.
     - LD-16-03: `audit_write` called from every write method on success AND failure paths; `{create,update,delete_or_close}_writes_audit_row` + `audit_records_failed_writes` tests lock it down.
  5. **Wave commits** — git sha for each wave's merge commit (fill in at write-time).
  6. **Test counts** — baseline 278 → Wave A ≈ 293 → Wave B ≈ 307 → Wave C ≈ 315+. Final number at ship time (run `cargo test --workspace 2>&1 | grep '^test result' | awk '{sum+=$4} END {print sum}'` and paste).
  7. **Clippy + fmt** — both clean across `--workspace --all-targets -- -D warnings`.
  8. **Deferred to later phases** — explicitly list: Confluence comments (Phase 23), attachments (Phase 24), swarm confluence-direct (Phase 17). Agents reading SUMMARY next session must not be confused about what did NOT ship.
  9. **Known limitations / documented tradeoffs:**
     - Storage XHTML is plain HTML (not Atlassian `<ac:structured-macro>`); agents writing code-block-heavy pages may see server-side re-rendering. Tracked for v0.7.0 hardening.
     - ADF unknown node types emit `[unsupported ADF node type=X]` marker; agents can `grep -r 'unsupported ADF' mount/pages/` to find lossy reads.
     - `version.number = current + 1` convention is verified via wiremock (Wave B) but not yet against a real Atlassian tenant — contract test under `--ignored live` flag is pending user-driven execution (see VALIDATION.md §Manual-Only Verifications).
  10. **Next post-phase gate** — user runs `scripts/tag-v0.6.0.sh` after green-gauntlet full passes.
- **Expected line impact:** ~150 lines.
- **Verification:** `[ -f .planning/phases/16-*/16-SUMMARY.md ]` and word-count > 500.

### D5. Update STATE.md cursor

- **File:** `.planning/STATE.md`
- **Action:**
  - Update `milestone` / `milestone_name` frontmatter to `v0.6.0` / `Write Path + Full Sitemap`.
  - Update `status` frontmatter to `executing` (Phase 16 is shipping; next phase 17 is next up).
  - Update `last_activity` timestamp.
  - Add a new bullet under `## Accumulated Context → Roadmap Evolution`:
    ```markdown
    - **Phase 16 SHIPPED (2026-04-XX, session N):** 4 waves landed on `main`
      — Wave A (`<sha>` ADF converter module) · Wave B (`<sha>` write
      methods + struct rename) · Wave C (`<sha>` audit log + ADF read path)
      · Wave D (`<sha>` CHANGELOG + version bump 0.5.0→0.6.0 + SUMMARY.md).
      Closes REQ WRITE-01..04 for the Confluence backend. Workspace test
      count N (baseline 278 + M new). Clippy `-D warnings` clean. v0.6.0
      milestone tag pending user `scripts/tag-v0.6.0.sh` execution.
      Details: `.planning/phases/16-.../16-SUMMARY.md`.
    ```
  - Update `## Current Position`:
    - Phase: **Phase 16 — SHIPPED** (was: Not started)
    - Cursor: v0.6.0 in-flight; Phase 16 closed; Phase 17 (swarm confluence-direct) is next recommended execution target
    - Last activity: today's date + "Phase 16 Confluence write path shipped"
  - Update `progress.completed_phases` from 1 → 2, `completed_plans` accordingly (add 4 for the four wave PLANs).
- **Expected line impact:** ~15 lines changed in frontmatter + bullets.
- **Verification:** `rg 'Phase 16 SHIPPED' .planning/STATE.md` returns 1 hit; `rg 'milestone: v0\.6' .planning/STATE.md` returns 1 hit.

### D6. Update ROADMAP.md — mark Phase 16 entry as shipped

- **File:** `.planning/ROADMAP.md`
- **Action:** Find the Phase 16 entry under the v0.6.0 milestone section. Change `[ ]` checkbox to `[x]`. Add a short suffix: ` — SHIPPED 2026-04-XX`. If the Phase 16 block has a `**Plans:**` subsection with placeholders, replace them with the four actual wave plans (`16-A-adf-converter.md`, `16-B-write-methods.md`, `16-C-audit-and-integration.md`, `16-D-docs-and-release.md`), each marked `[x]`.
- **Expected line impact:** ~6 lines.
- **Verification:** `rg '\\[x\\].*Phase 16' .planning/ROADMAP.md` returns 1 hit.

### D7. Git commit D7-final

- **Action (manual, executor-run):** After D1–D6 land, stage all Wave D files with:
  ```bash
  git add CHANGELOG.md Cargo.toml Cargo.lock scripts/tag-v0.6.0.sh \
    .planning/phases/16-*/16-SUMMARY.md .planning/STATE.md .planning/ROADMAP.md
  git status   # sanity check: no unintended files staged
  git commit   # the commit message is the one thing NOT generated by Claude — use the body from the CHANGELOG entry
  ```
- **Do NOT run `scripts/tag-v0.6.0.sh`** — that's user-gated (Phase 15 convention: executors never push tags).

## Verification

Before merging Wave D:

```bash
cargo check --workspace                                              # Cargo.lock regenerates cleanly
cargo test --workspace                                               # no regressions
cargo clippy --workspace --all-targets -- -D warnings                # clean
cargo fmt --all --check                                              # formatted
rg 'v0\.6\.0'  CHANGELOG.md Cargo.toml scripts/tag-v0.6.0.sh         # version references present
rg 'Phase 16 SHIPPED' .planning/STATE.md                             # state updated
[ -f .planning/phases/16-*/16-SUMMARY.md ]                           # summary exists
```

Test count MUST be ≥ 315 (same as end of Wave C; no new tests in D).

## Threat model

| Threat ID | STRIDE | Component | Disposition | Mitigation |
|---|---|---|---|---|
| T-16-D-01 | Repudiation | Tag pushed without human review | Accept | `scripts/tag-v0.6.0.sh` is NOT invoked by Wave D — convention from Phase 15 Wave B (executors never tag). User runs `green-gauntlet --full` then invokes the script manually. |
| T-16-D-02 | Tampering | Incorrect version number in Cargo.lock allows confused-deputy upgrade | Mitigate | Cargo.lock regenerated via `cargo check --workspace` — never hand-edited. Workspace-inheritance (`version.workspace = true`) ensures all 8 crates bump together atomically. |
| T-16-D-03 | Information Disclosure | SUMMARY.md leaks a tenant subdomain, token, or real page ID | Accept / Document | SUMMARY.md uses only synthetic fixture IDs (`99`, `42`, `777`) — same convention as Phase 15. Review checklist before commit: grep for `atlassian.net` (should be 0 hits outside of code-sample blocks) and for `@` email characters (should only appear in fixture strings like `"test@example.com"`). |

## Success criteria

1. `CHANGELOG.md` has a `[v0.6.0]` entry summarizing Phase 16 work.
2. Workspace version is `0.6.0` across all crates; `cargo check --workspace` clean.
3. `scripts/tag-v0.6.0.sh` exists, executable, shell-syntax clean, references v0.6.0 throughout.
4. `.planning/phases/16-*/16-SUMMARY.md` exists with all 10 sections.
5. `.planning/STATE.md` cursor reflects Phase 16 SHIPPED; milestone v0.6.0 in-flight.
6. `.planning/ROADMAP.md` Phase 16 entry checked off.
7. `cargo test --workspace` still green; test count ≥ 315; clippy `-D warnings` clean; fmt clean.
8. No secrets or tenant strings introduced in any Wave D file (grep check in T-16-D-03 passes).
