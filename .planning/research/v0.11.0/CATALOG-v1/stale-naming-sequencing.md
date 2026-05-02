← [back to index](./index.md)

# Stale references audit, naming generalization, recommended sequencing

## Stale references audit

A grep pass for terms that should be 0 or near-0 in tracked code post-v0.9.0 (counts exclude `.planning/milestones/`, `.planning/research/v0.1-fuse-era/`, `docs/archive/`, and `CHANGELOG.md` historical sections, which are correctly historical):

| Term | Count (approx) | Where it lives |
|---|---:|---|
| `fuser` (cargo dependency) | 0 | crate purged |
| `fusermount` | ~16 | `scripts/demos/*.sh`, `scripts/dev/test-*.sh`, `docs/reference/{cli,crates}.md`, `docs/development/contributing.md`, `docs/decisions/{001,002,003}.md`, `docs/demo.md` (stub — false positive), `README.md` |
| `FUSE` (case-insensitive) | ~70 | `README.md` (27), `docs/reference/{cli,crates,git-remote,confluence,jira,http-api,testing-targets}.md` (~25), `docs/decisions/{001,002,003,005}.md` (~10), `docs/development/{contributing,roadmap}.md`, `crates/reposix-{core,confluence,remote,swarm}/**` doc comments (~10), `crates/reposix-cli/{src,tests}` (~8 — most are valid migration tests) |
| `reposix mount` | ~25 | `scripts/demos/*.sh` (most), `docs/reference/{cli,confluence,crates}.md`, `README.md`, `HANDOFF.md`, `docs/decisions/002`, `docs/demos/index.md` |
| `reposix-fuse` (binary or crate name) | ~12 | `.github/workflows/release.yml` (broken), `scripts/demos/*.sh`, `docs/reference/crates.md`, `docs/decisions/{002,003,004}.md`, `docs/development/{contributing,roadmap}.md`, `HANDOFF.md` |
| `IssueId` | 304 | active code (load-bearing rename target — see below) |
| `IssueBackend` | ~25 docs / 0 active code | `docs/reference/crates.md`, `docs/connectors/guide.md` (stub), `docs/decisions/{001,002}.md`, `docs/why.md` (stub — false), `docs/demos/index.md`, `docs/reference/confluence.md`, `docs/development/roadmap.md`, `docs/security.md` (stub — false), `scripts/demos/{08, parity, parity-confluence, 05}.sh`, `scripts/tag-v0.8.0.sh`, `docs/archive/{...}` (correctly historical). `crates/` source has 0 — Phase 27 rename is clean. |
| `IssueBackend` in active Rust code | 0 | The trait is `BackendConnector`. References in `crates/` `.rs` files are all in *comments* like "FUSE daemon and CLI orchestrator" — those are doc-comment cleanups, not code. |

---

## Naming generalization candidates

> **OWNER FLAGGED:** `IssueId` is too narrow ("could be a doc, issue, note, etc"). The architecture is now backend-agnostic (sim, GitHub Issues, Confluence pages, JIRA tickets) but the type name still privileges one shape. The frontmatter field name (`id`) is already generic; only the Rust type names lag.

| Current name | Proposed | Scope | Risk | Notes |
|---|---|---|---|---|
| `IssueId` | `RecordId` | workspace-wide; ~304 call sites across 13 src files + tests | Medium — typed-error variants reference it; `Issue.parent_id: Option<IssueId>` cross-references; Confluence/JIRA tests use `IssueId(N)` as constructor. Mechanical rename via cargo + `rust-analyzer` — no semantic change. | YAML serialization is `#[serde(transparent)]` — on-disk format `id: 42` is unaffected. Same precedent as ADR-004 `IssueBackend → BackendConnector`. |
| `Issue` | `Record` | workspace-wide; load-bearing struct; ~600+ call sites | High — frontmatter YAML `Frontmatter` DTO references it; test fixtures named `sample()` return `Issue`; `Issue.title`/`.body`/`.status` field names; `IssueStatus` enum. **Decision recommendation: rename `Issue` → `Record` AND `IssueStatus` → `RecordStatus`** (the GitHub-specific status terms `Open`/`InProgress`/`Done` are already overly specific; consider also widening the enum if doing this rename). | YAML round-trip safe (`Frontmatter` DTO is internal). |
| `IssueStatus` | `RecordStatus` (or `WorkflowState`) | workspace-wide; ~50 call sites | Medium | Tied to the `Issue → Record` decision. The JIRA-flavored enum (`Open/InProgress/InReview/Done/WontFix`) is already a "Jira-flavored superset" per the doc comment — fine to keep semantically; only the type name moves. |
| `Error::InvalidIssue` | `Error::InvalidRecord` | workspace-wide; ~10 call sites | Low | Tied to `Issue` rename. |
| `error.rs::Issue` body / file format violation message | (text update) | `crates/reposix-core/src/error.rs:16` | Low | Trivial. |
| `path::validate_issue_filename` | `path::validate_record_filename` | ~6 call sites | Low | Tied to `Issue` rename. |
| `slug_or_fallback(title, id: IssueId)` | (just take `id: RecordId`) | already semantically generic | Low | Function purpose is generic; only the type rename lands. |
| `ProjectSlug` | (keep) | — | — | Already generic. |
| `BackendConnector` | (keep) | — | — | Already renamed Phase 27 (ADR-004). Good. |
| `bucket` term in code/test naming | (keep) | — | — | Already generic — refers to `issues/` vs `pages/` collection at the working-tree root. Survived the FUSE deletion intact. |

**Other narrow terms to consider sweeping in the same rename pass:**

- `update_issue` / `create_issue` / `delete_or_close` (`BackendConnector` methods) → `update_record` / `create_record` / `close_record`. ~50 call sites; mechanical. Tied to the `Issue` rename to keep the API surface coherent.
- `list_issues` / `list_changed_since` (returns `Vec<IssueId>`) → `list_records` / `list_changed_since` (returns `Vec<RecordId>`). Same pass.
- Module name `crates/reposix-core/src/issue.rs` → `record.rs` (and re-export `pub use record::*`).
- `crates/reposix-core/src/backend/sim.rs::SimBackend` is fine (a backend not a record).

**Suggested sequencing for the rename:**
1. Plan as a single coordinated phase (call it `RENAME-02` per the existing ADR-004 → ADR-005 numbering convention).
2. Hard rename, no backward-compat aliases (precedent: Phase 27 / ADR-004).
3. Tooling: `cargo check --workspace` + `rust-analyzer rename` + a tracking script under `scripts/check_naming_generic.sh` that fails CI on regressions.
4. Run `bash scripts/dark-factory-test.sh sim` after the rename — agent UX is "pure git", so the rename should be invisible to the dark-factory regression by construction. That's the cleanest possible end-to-end check.

---

## Recommended sequencing

**Tonight (overnight polish session).** Land the must-do items 1–6: fix `release.yml` (one line), delete the dead FUSE swarm code + the FUSE demo scripts (file deletions only), rewrite `docs/reference/cli.md` + `docs/reference/crates.md` (highest user-visible misleading docs), and delete `HANDOFF.md` (after verifying nothing in `.planning/STATE.md` or `MILESTONES.md` is missing). These are all low-risk, high-leverage, and keep the v0.10.0 milestone scaffolding plan intact. Item 7 (README rewrite) is owner-tagged for Phase 45 — leave it to that phase rather than pre-empting; just delete the explicit "v0.7.x — pre-FUSE-deletion" Quickstart section as a strict subset of the Phase 45 work.

**v0.10.0 milestone (already-planned Phases 40–45).** The v0.10.0 plan-of-record handles most of the docs work organically (Phase 41 = how-it-works trio, Phase 42 = tutorials/guides, Phase 43 = nav + theme, Phase 44 = clarity-review gate, Phase 45 = README + tag). The "should-do this milestone" items above slot into Phase 41 (`docs/reference/git-remote.md`, `confluence.md`, `jira.md` rewrites are reference-page work that aligns with the trio), Phase 42 (rewrite the demo scripts), Phase 43 (banned-words linter would catch any future regressions), and Phase 44 (the gate — running doc-clarity-review against everything). Add a Phase 43.1 or 44.1 explicitly for "scope-superseded ADR annotations" since those are decision-records edits and not nav work. Item 10 (helper-hardcodes-SimBackend) is the v0.9.0 milestone-audit carry-forward; STATE.md correctly schedules it before any v0.11.0 benchmarks.

**v0.11.0 ("Performance & Sales Assets" per the audit) and beyond.** The `IssueId/Issue/IssueStatus` rename is a v0.11.0 candidate — it's a code-level change that should not happen during a docs milestone (would invalidate every example in the freshly-written docs), and not before v0.10.0 ships (locks the new docs to a not-yet-existent type vocabulary). Plan it as the first phase of v0.11.0 with the dark-factory regression as its acceptance gate. The "re-record demo gifs" and "consolidate redundant CI jobs" items are nice-to-have backlog and don't block any milestone; both are appropriate to land opportunistically as their work surfaces (Phase 45 will already need fresh recordings, for example).
