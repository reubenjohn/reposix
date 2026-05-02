← [back to index](./index.md)

# Milestone v0.7.0 — Hardening + Confluence Expansion

**Goal:** Harden the platform under real-world load conditions and expand Confluence support beyond pages.
**Phases:** 21–25 | **Requirements:** REQUIREMENTS.md §v0.7.0

### Phase 21: OP-7 hardening bundle — contention swarm, 500-page truncation probe, chaos audit-log restart, macFUSE parity CI matrix (v0.7.0)

**Goal:** Audit the two OP-7 items already shipped in session-4 drive-bys (credential pre-push hook, SSRF regression tests) and close the five remaining hardening items: contention swarm mode proving If-Match 409 determinism (HARD-01); Confluence 500-page truncation probe with WARN + `--no-truncate` flag (HARD-02, SG-05 compliance); kill-9 chaos test against the sim's WAL-mode audit log (HARD-03); macOS + macFUSE CI parity matrix (HARD-04); and tenant-URL redaction in list_issues error messages (HARD-05).
**Requirements**: HARD-00, HARD-01, HARD-02, HARD-03, HARD-04, HARD-05
**Depends on:** Phase 20
**Plans:** 5/5 plans complete

Plans:
- [x] 21-A-audit.md — HARD-00 audit of pre-push hook + SSRF tests (Wave 1)
- [x] 21-B-contention.md — HARD-01 ContentionWorkload + Mode::Contention (Wave 2, depends on A)
- [x] 21-C-truncation.md — HARD-02 list_issues_strict + --no-truncate + HARD-05 tenant-URL redaction (Wave 3, depends on A)
- [x] 21-D-chaos.md — HARD-03 kill-9 chaos audit-log integrity test (Wave 4, depends on B)
- [x] 21-E-macos.md — HARD-04 macos-14 CI matrix + hooks CI step (Wave 5, depends on A, autonomous=false)

### Phase 22: OP-8 honest-tokenizer benchmarks — replace len-div-4 with count_tokens API, per-backend comparison tables (v0.7.0)


**Goal:** Replace the `len(text) // 4` token approximation in `scripts/bench_token_economy.py` with real Anthropic `client.messages.count_tokens()` calls, cache results in `benchmarks/fixtures/*.tokens.json` with SHA-256 content-hash keys for offline CI reproducibility, add per-backend (MCP, GitHub, Confluence, Jira-placeholder) comparison tables, and re-state the headline token-reduction number in `docs/why.md` with the real measurement — honest regardless of whether it's higher or lower than the prior 91.6% estimate. Python + Markdown only; no Rust changes.
**Requirements**: BENCH-01, BENCH-02, BENCH-03, BENCH-04
**Depends on:** Phase 21
**Plans:** 3/3 plans complete

Plans:
- [x] 22-A-bench-upgrade-PLAN.md — scripts/bench_token_economy.py + requirements-bench.txt: count_tokens API, SHA-256 cache, --offline flag, 6 pytest cases (Wave 1, autonomous, BENCH-01)
- [x] 22-B-fixtures-and-table-PLAN.md — benchmarks/fixtures/{github_issues.json, confluence_pages.json, README.md}: synthetic REST payloads + fixture provenance doc (Wave 1, autonomous, BENCH-02 prerequisites)
- [x] 22-C-wire-docs-ship-PLAN.md — per-backend table wiring + one-shot cache population (checkpoint) + docs/why.md headline update + CHANGELOG + SUMMARY (Wave 2, autonomous=false, BENCH-02/03/04)


### Phase 23: OP-9a — Confluence comments exposed as pages/id.comments/comment-id.md (v0.7.0)

**Goal:** Expose Confluence page inline and footer comments as synthesized `.comments/` subdirectories under each page in the FUSE mount: `pages/<padded-id>.comments/<comment-id>.md`. Each comment file has YAML frontmatter (id, author, created_at, resolved, parent_comment_id, kind) and a Markdown body. Also adds a `reposix spaces --backend confluence` subcommand for listing Confluence spaces (CONTEXT.md locked decision).
**Requirements**: CONF-01, CONF-02, CONF-03
**Depends on:** Phase 22
**Plans:** 3/3 plans complete

Plans:
- [x] 23-01-PLAN.md — `ConfluenceBackend::list_comments` + `list_spaces` + `ConfComment`/`ConfSpaceSummary` public types (Wave 1, autonomous)
- [x] 23-02-PLAN.md — `reposix spaces` CLI subcommand + table renderer (Wave 2, autonomous, depends on 23-01)
- [x] 23-03-PLAN.md — FUSE `.comments/` synthesis: inode constants, CommentsSnapshot, fs.rs dispatch, MountConfig wire-through (Wave 2, autonomous, depends on 23-01)

### Phase 24: OP-9b — Confluence whiteboards attachments and folders (v0.7.0)

**Goal:** Surface Confluence whiteboards (`whiteboards/<id>.json`), page attachments (`pages/<id>.attachments/<filename>`), and folder-parented page hierarchy (`tree/` overlay) in the FUSE mount. Expands the multi-content-type overlay from Phase 23.
**Requirements**: CONF-04, CONF-05, CONF-06
**Depends on:** Phase 23
**Plans:** 3/3 plans complete

Plans:
- [x] 24-01-PLAN.md — `list_attachments` + `list_whiteboards` + `download_attachment` methods; `ConfAttachment`/`ConfWhiteboard` structs; `translate()` folder fix (Wave 1)
- [x] 24-02-PLAN.md — FUSE overlay: `AttachmentsSnapshot`, 4 new `InodeKind` variants + dispatch (Wave 2)
- [x] 24-03-PLAN.md — Green gauntlet + CHANGELOG + STATE.md + SUMMARY (Wave 3)

### Phase 25: OP-11 — docs reorg: InitialReport.md and AgenticEngineeringReference.md to docs/research/ plus root cleanup (v0.7.0)

**Goal:** Move `InitialReport.md` and `AgenticEngineeringReference.md` to `docs/research/`, update all cross-references in `CLAUDE.md`, `README.md`, and `.planning/research/`, add a Research section to mkdocs.yml nav, and bump workspace to v0.7.0.
**Requirements**: none (docs-only phase)
**Depends on:** Phase 24
**Plans:** 2 plans complete

Plans:
- [x] 25-01-PLAN.md — cross-ref audit, root clutter cleanup, mkdocs.yml nav
- [x] 25-02-PLAN.md — v0.7.0 version bump, CHANGELOG promotion, STATE.md, SUMMARY

---
