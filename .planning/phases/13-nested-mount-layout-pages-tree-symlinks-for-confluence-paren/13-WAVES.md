# Phase 13 — Wave structure & dependency graph

**Phase:** 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
**Target release:** v0.4.0
**Plans:** 9 executable plans across 5 waves

## Wave graph

```
Wave A (serial, foundational)
  13-A-core-foundations  — reposix-core: parent_id + slugify + BackendFeature::Hierarchy + root_collection_name
     │
     ├─► Wave B (3 plans parallel — all depend on A only, no shared files)
     │     ├─ 13-B1-confluence-parent-id  — reposix-confluence: deserialize parentId, supports(Hierarchy), root_collection_name="pages"
     │     ├─ 13-B2-fuse-tree-module       — reposix-fuse::tree.rs NEW module (pure in-memory; zero coupling to fs.rs)
     │     └─ 13-B3-frontmatter-parent-id  — reposix-core::issue::frontmatter extend Frontmatter struct + roundtrip tests
     │
     └─► Wave C (serial — the only wave that edits fs.rs)
           13-C-fuse-wiring               — reposix-fuse::fs.rs + inode.rs wiring: root bucket, .gitignore, tree/ dispatch, readlink, integration tests
              │
              ├─► Wave D (3 plans parallel — D1 touches repo-wide docs/demos/tests; D2 touches docs/; D3 touches scripts/)
              │     ├─ 13-D1-breaking-migration-sweep  — every demo/doc/test referencing old mount/<id>.md flat path
              │     ├─ 13-D2-docs-and-adr              — ADR-003, CHANGELOG [v0.4.0], CLAUDE.md fuser-0.17 fix, README tree/ section
              │     └─ 13-D3-release-scripts-and-demo  — scripts/tag-v0.4.0.sh + scripts/demos/07-mount-real-confluence-tree.sh
              │
              └─► Wave E (serial, phase gate)
                    13-E-green-gauntlet   — workspace fmt/clippy/test + --ignored live Confluence + smoke.sh + mkdocs --strict + live REPOSIX-space verify + capture proofs into 13-SUMMARY.md
```

## Plan → files_modified matrix (parallel-safety check)

| Plan | Crate files | Tests | Docs/Scripts |
|---|---|---|---|
| A | `reposix-core/src/{issue,backend,path,lib}.rs` | new path::tests, new backend::tests | — |
| B1 | `reposix-confluence/src/lib.rs` | in-crate wiremock | — |
| B2 | `reposix-fuse/src/{tree.rs NEW,lib.rs}` | in-crate unit + optional proptest | — |
| B3 | `reposix-core/src/issue.rs` (frontmatter sub-module only) | in-crate roundtrip tests | — |
| C | `reposix-fuse/src/{fs.rs,inode.rs}` | `tests/readdir.rs`, new `tests/nested_layout.rs` | — |
| D1 | — | `crates/reposix-fuse/tests/sim_death_no_hang.rs` path update | demos/*.sh, docs/demo.md, README.md, all docstrings |
| D2 | — | — | `docs/decisions/003-*.md`, `CHANGELOG.md`, `CLAUDE.md`, `README.md`, `mkdocs.yml` if needed |
| D3 | — | — | `scripts/tag-v0.4.0.sh`, `scripts/demos/07-mount-real-confluence-tree.sh`, `scripts/demos/_lib.sh` if extended |
| E | — | full workspace + live | `.planning/phases/13-.../13-SUMMARY.md` |

**Potential conflicts:**
- A vs B3: both touch `reposix-core/src/issue.rs`. **Resolution:** A modifies the `Issue` struct top-level; B3 modifies only the nested `frontmatter` submodule. Conflict risk = low but non-zero → **B3 runs strictly after A (depends_on: [A])**, which is already the wave rule. B3 runs in parallel with B1/B2 because B1 and B2 do not touch `issue.rs`.
- D1 vs D2: D1 patches existing demos; D2 writes a new ADR + appends CHANGELOG + README. README gets a new top-level section from D2 while D1 patches inline path examples inside. **Resolution:** D1 uses line-anchored sed/awk edits on existing code blocks; D2 appends new sections. Conflict risk = low. If conflict emerges, D2 reruns after D1.
- D1 vs C: C updates `tests/readdir.rs`. D1 must NOT touch that file (already covered by C). **Resolution:** D1's file allowlist explicitly excludes any file C is listed as modifying.

## Execution order (yolo mode)

```
Wave 1: [A]                                    — serial
Wave 2: [B1, B2, B3]                           — parallel
Wave 3: [C]                                    — serial
Wave 4: [D1, D2, D3]                           — parallel
Wave 5: [E]                                    — serial phase gate
```

## Success-gate summary (per wave)

- **A green:** `cargo test -p reposix-core --locked` shows new tests for `slugify_title`, `slug_or_fallback`, `dedupe_siblings`, `parent_id` serde + `backend_feature_hierarchy_default_false`. `cargo clippy -p reposix-core --all-targets --locked -- -D warnings` clean.
- **B1 green:** `cargo test -p reposix-confluence --locked` — new tests for `populates_parent_id_for_page_parent`, `non_page_parent_type_debug_log`, `supports_hierarchy_returns_true`, `root_collection_name_returns_pages`.
- **B2 green:** `cargo test -p reposix-fuse tree::` — unit tests for `build_tree` + `dedupe_siblings` + `resolve_symlink` + cycle detection. No FUSE mount involved.
- **B3 green:** `cargo test -p reposix-core frontmatter::tests` — `parent_id_roundtrips`, `parent_id_absent_omitted_from_yaml`.
- **C green:** `cargo test --release -p reposix-fuse -- --ignored --test-threads=1 nested_layout` green; existing `readdir.rs` still green after path update.
- **D1 green:** `git grep -nE "mount/0[0-9]+\.md|reposix-mnt/[0-9]+\.md"` returns no matches outside `.planning/` history snapshots; `bash scripts/demos/smoke.sh` 4/4 green.
- **D2 green:** `test -f docs/decisions/003-nested-mount-layout.md`; `grep -q '## \[v0.4.0\]' CHANGELOG.md`; `grep -q 'fuser.*0\.17' CLAUDE.md`; `mkdocs build --strict` exits 0.
- **D3 green:** `bash -n scripts/tag-v0.4.0.sh` + `bash -n scripts/demos/07-mount-real-confluence-tree.sh`; both scripts' `--help` / skip-without-creds paths exit 0.
- **E green:** full workspace `cargo fmt/clippy/test`; `--ignored` live Confluence contract green; manual live verify recorded in 13-SUMMARY.md with `ls`/`cat`/`readlink` output; mkdocs strict green.

## Decision coverage matrix (CONTEXT.md locked decisions → plan mapping)

| Locked decision (from 13-CONTEXT.md §decisions) | Plan | Coverage |
|---|---|---|
| Per-backend `root_collection_name` trait method | A, B1 | Full — A adds default `"issues"`; B1 overrides to `"pages"` |
| `Issue::parent_id: Option<IssueId>` (serde default, skip-if-none) | A, B3 | Full — A adds field; B3 adds frontmatter roundtrip |
| `slugify_title` + `slug_or_fallback` | A | Full |
| Sibling collision dedupe (`-N` suffix, stable-sort by IssueId) | A | Full — tested in A |
| `BackendFeature::Hierarchy` variant + confluence override | A, B1 | Full |
| Confluence deserializes `parentId`/`parentType`, filters to `"page"` | B1 | Full |
| FUSE `tree/` module — build, resolve, cycle-break, depth-aware relative paths | B2 | Full |
| Inode-range dispatch (root bucket / tree-dirs / tree-symlinks) | B2 (ranges) + C (dispatch) | Full |
| FUSE root emits `<bucket>/`, `tree/` (conditional), `.gitignore` | C | Full |
| `readlink` handler returning symlink-size-correct `FileAttr` | C | Full |
| Existing flat `<id>.md` at mount root REMOVED | C | Full — readdir(root) no longer lists it |
| Writes through `tree/foo.md` follow symlink to `pages/<id>.md` | C (integration test) | Full — POSIX symlinks, no special code |
| `.gitignore` content is exactly `/tree/\n`, read-only, always emitted | C | Full |
| BREAKING CHANGELOG block (every demo/doc/test path updated) | D1, D2 | Full — D1 updates demos/docs/tests; D2 writes the CHANGELOG block |
| ADR-003 supersedes ADR-002 | D2 | Full |
| `scripts/tag-v0.4.0.sh` + new Confluence-tree demo | D3 | Full |
| CLAUDE.md fuser-version correction (0.15 → 0.17) | D2 | Full |
| Live REPOSIX-space verify (homepage 360556 → 3 children) | E | Full (manual, recorded) |
| `mkdocs build --strict` still green | D2, E | Full |

No decision is Partial. No decision is silently scope-reduced. ✓

## Threat model (Phase-level; each plan's `<threat_model>` block addresses its share)

| Threat ID | Category | Component | Disposition | Carried by plan(s) |
|---|---|---|---|---|
| T-13-01 (HIGH) | Tampering | `slugify_title` must not emit shell/path-control chars | mitigate | A |
| T-13-02 (HIGH) | Tampering | slug must never resolve to `.`/`..`/empty | mitigate | A |
| T-13-03 (MED) | DoS | parent_id cycle → infinite recursion in tree builder | mitigate | B2 |
| T-13-04 (MED) | Tampering | Sibling slug collision post-truncation | mitigate | A (dedupe) + B2 (applied at build time) |
| T-13-05 (LOW) | Tampering | Symlink target escaping mount (`../../../etc/passwd`) | mitigate | B2 + C |
| T-13-06 (MED, accepted) | Tampering | Unicode NFC normalization gap (visually identical titles → different slugs) | accept + document | A + D2 (ADR-003 known limitation) |

All HIGH threats have tests in the plan that introduces the mitigation (A unit tests for T-13-01 + T-13-02; B2 unit tests for T-13-03 + T-13-04; C integration test for T-13-05).
