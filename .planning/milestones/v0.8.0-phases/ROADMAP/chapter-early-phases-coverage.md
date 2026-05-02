← [back to index](./index.md)

# Early Phases: Coverage, Decision Gates, Progress, Phases 8–15

## Coverage

| Req | Phase | Notes |
|-----|-------|-------|
| FC-01 Simulator-first architecture | Phase 2 | sim binary + seed + rate-limit + 409 |
| FC-02 Issues as Markdown + YAML | Phase 1 | render/parse already in core; server-field stripper lands here |
| FC-03 FUSE mount with read+write | Phase 3 (read) + Phase S (write) | read-path is MVD, write is STRETCH |
| FC-04 `git-remote-reposix` helper | Phase S | STRETCH; dropped if T+3h gate goes read-only |
| FC-05 Working CLI orchestrator | Phase 3 | `sim`, `mount`, `demo` subcommands |
| FC-06 Audit log (SQLite, queryable) | Phase 1 (schema) + Phase 2 (writes) | DDL in core, writes in sim middleware |
| FC-07 Adversarial swarm harness | Phase S | STRETCH |
| FC-08 Working CI on GitHub Actions | baseline + Phase 4 (green-on-demo-commit) + Phase S (FUSE mount in CI) | MVD owns fmt/clippy/test/coverage (already green); FUSE-in-CI is STRETCH |
| FC-09 Demo-ready by morning | Phase 4 | README + recording + walkthrough |
| SG-01 Outbound HTTP allowlist | Phase 1 | single `http::client()` factory + clippy lint |
| SG-02 Bulk-delete cap on push | Phase S | lives on the push path; no MVD surface for it |
| SG-03 Server-controlled frontmatter immutable | Phase 1 | `sanitize()` strips `id`/`created_at`/`version`/`updated_at` |
| SG-04 Filename = `<id>.md`; path validation | Phase 1 (validator) + Phase 3 (FUSE enforcement) | |
| SG-05 Tainted-content typing | Phase 1 | `Tainted<T>`/`Untainted<T>` + trybuild test |
| SG-06 Audit log append-only | Phase 1 (schema + triggers) + Phase 2 (enforcement test) | |
| SG-07 FUSE never blocks kernel forever | Phase 3 | `with_timeout(5s)` wrapper + EIO path |
| SG-08 Demo shows guardrails firing | Phase 4 | on-camera allowlist refusal + field-strip |

**Coverage:** 17/17 requirements mapped ✓  (no orphans, no duplicates —
shared requirements are split by sub-deliverable: schema/enforcement, or
read/write.)

## Decision gates

- **T+3h (03:30 PDT) — STRETCH commit gate.** If Phase 1 is not green and
  Phases 2+3 are not both at or near success-criteria, CUT Phase S entirely
  and reallocate the remaining budget to Phase 4 polish. No litigation of this
  decision at T+5h.
- **T+5h (05:30 PDT) — recording cutoff.** Start the asciinema/script(1)
  recording no later than 05:30 regardless of STRETCH status. Re-records eat
  ~30 min each; budgeting for exactly one re-record.
- **T+7h (07:30 PDT) — push and walk away.** Final commit pushed, CI green,
  README rendered on github.com verified via playwright screenshot (per user's
  global OP #1).

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 ∥ 3 → [T+3h gate] → (S?) → 4.
Phases 2 and 3 run in parallel. Phase S is conditional.

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Core contracts + security guardrails | 0/3 | Not started | - |
| 2. Simulator + audit log | 0/2 | Not started | - |
| 3. Read-only FUSE mount + CLI | 0/2 | Not started | - |
| 4. Demo + recording + README | 0/1 | Not started | - |
| S. STRETCH: write + swarm + FUSE-in-CI | 2/3 | Complete (swarm+CI deferred) | 2026-04-13 |
| 8. Demo suite + real-backend seam | 0/4 | Post-ship value add | - |

### Phase 8: Demo suite + real-backend seam (post-ship) (v0.2)
**Goal**: Split the demo monolith into a maintainable suite AND carve the IssueBackend seam so a real GitHub adapter can land without reshaping the FUSE/remote layers.
**Added**: 2026-04-13 09:05 PDT (post-v0.1.0-ship; user-requested value add)
**Deadline**: 12:15 PDT (~3h 10min)
**Depends on**: Phase 4 demo, Phase 7 robustness
**Requirements**: introduces maintainability + real-backend parity (v0.2 prep)

**Plans**:
- [ ] 08-A: Demo restructure (`scripts/demos/_lib.sh` + 4 Tier-1 one-liners + assert.sh + smoke.sh + full.sh rename + old demo.sh shim + docs/demos/index.md + CI smoke job).
- [ ] 08-B: `IssueBackend` trait in `reposix-core` + `SimBackend` impl + `reposix list` CLI subcommand.
- [ ] 08-C: `reposix-github` crate with `GithubReadOnlyBackend` + state-mapping ADR + `scripts/demos/parity.sh`.
- [ ] 08-D: Contract test suite parameterized over both backends + Tier-1 demo recordings + docs updates.

### Phase 11: Confluence Cloud read-only adapter (v0.3)
**Goal**: Ship a `reposix-confluence` crate implementing `IssueBackend` against Atlassian Confluence Cloud REST v2 (`https://<tenant>.atlassian.net/wiki/api/v2/`). Adopt **Option A** from HANDOFF §3 — flatten page hierarchy; encode `parent_id` + `space_key` in frontmatter — so existing FUSE + CLI machinery works unchanged. Basic auth (`email:ATLASSIAN_API_KEY`). CLI dispatch for `list --backend confluence` and `mount --backend confluence --project <SPACE_KEY>`. Wiremock unit tests ≥5. Contract test parameterized like GitHub's. Tier 3B parity demo + Tier 5 live-mount demo. ADR-002 for page→issue mapping. Docs update. Rename `TEAMWORK_GRAPH_API` → `ATLASSIAN_API_KEY` in `.env.example`. CHANGELOG + `v0.3.0` tag.

(Note: gsd-tools auto-allocated "Phase 9" above, but Phases 9-swarm and 10-FUSE-GitHub already shipped from the previous session as committed git history without formal ROADMAP.md entries. Skipping numerically to Phase 11 keeps the numbering honest.)

**Added**: 2026-04-13 ~20:55 PDT (overnight session 3)
**Depends on**: Phase 10 (FUSE mount via `IssueBackend` trait already shipped)
**Deadline**: 08:00 PDT 2026-04-14

**Success Criteria** (each is a Bash assertion):
  1. `cargo test --workspace --locked` returns ≥180 pass / 0 fail.
  2. `cargo clippy --workspace --all-targets -- -D warnings` exits 0.
  3. `bash scripts/demos/smoke.sh` still 4/4 green — Tier 1 demos untouched.
  4. `reposix list --backend confluence --project <SPACE_KEY>` (with `ATLASSIAN_API_KEY` + `REPOSIX_ALLOWED_ORIGINS=...,https://<tenant>.atlassian.net` set) against real Atlassian prints ≥1 issue row.
  5. `reposix mount /tmp/reposix-conf-mnt --backend confluence --project <SPACE_KEY>` + `ls` + `cat *.md` returns real page frontmatter + body; `fusermount3 -u` succeeds; subsequent mount works again (re-entrant).
  6. `bash scripts/demos/06-mount-real-confluence.sh` exits 0 when `ATLASSIAN_API_KEY` is set; skips cleanly (exit 0) when unset.
  7. `docs/decisions/002-confluence-page-mapping.md` exists and documents the field mapping; `.env.example` reflects the renamed var.

**Plans**: 6 plans (6 waves collapsed into 3 execution waves)

Plans:
- [ ] 11-A-confluence-crate-core.md — `reposix-confluence` crate core (`ConfluenceReadOnlyBackend` + wiremock unit tests ≥10). Wave 1.
- [ ] 11-B-cli-dispatch.md — CLI `--backend confluence` dispatch (`reposix list` / `reposix mount` / `reposix-fuse`) + `integration-contract-confluence` CI job. Wave 1.
- [ ] 11-C-contract-test.md — Contract test parameterized over sim + wiremock-confluence + live-confluence (`#[ignore]`-gated). Wave 2.
- [ ] 11-D-demos.md — Tier 3B `parity-confluence.sh` + Tier 5 `06-mount-real-confluence.sh`; both skip cleanly with no env. Wave 1.
- [ ] 11-E-docs-and-env.md — ADR-002, `docs/reference/confluence.md`, README + architecture + demos-index + CHANGELOG `[Unreleased]` + `.env.example` rename. Wave 2.
- [ ] 11-F-release.md — `MORNING-BRIEF-v0.3.md`, CHANGELOG promotion to `[v0.3.0]`, `scripts/tag-v0.3.0.sh` (gated by human-verify checkpoint). Wave 3.

### Phase 12: Connector protocol — 3rd-party plugin ABI (v0.4 target)
**Goal**: Make new backends addable **without changes to the reposix repo**. Three models ranked by ROI:

1. **Short-term, already unlocked (no new code):** the `IssueBackend` trait in `reposix-core` is a published public trait. A 3rd party publishes `reposix-adapter-<name>` on crates.io; users add it to their own fork's `Cargo.toml` + a one-line `ListBackend::Custom` dispatch. Phase 11-E ships `docs/connectors/guide.md` explaining this path with GitHub + Confluence as worked examples.
2. **Medium-term (this phase):** subprocess / JSON-RPC plugin ABI. Connectors are standalone binaries on `PATH` named `reposix-connector-<name>` (convention mirrors git-remote-helpers, which reposix already uses for `git-remote-reposix`). reposix daemon spawns and speaks a documented JSON-RPC protocol over stdio. Gives: polyglot (Python, Go, Rust), no-recompile add/remove, stable ABI, easy sandboxing per plugin, and natural SG-01 re-enforcement (the daemon proxies outbound HTTP on the plugin's behalf). **This phase ships the protocol spec + reference impl + migration of `reposix-github` behind it as a canary.**
3. **Long-term deferred (v0.5+):** WASM plugin model — wasmtime-based, strong sandbox, but needs HTTP-in-WASM plumbing that does not exist yet.

Phase 12 scope: model (2) above. Explicit non-goals: (1) already shipped via docs; (3) deferred.

**Added**: 2026-04-13 ~21:35 PDT (session 3, user-requested forward-look while closing Phase 11 plans)
**Depends on**: Phase 11 (proves the second adapter is on-pattern with the first)
**Status**: Skeleton only. Not executed tonight. Full /gsd-plan-phase 12 + discussion required before execution.

**Success Criteria** (draft — to be finalized in /gsd-plan-phase 12):
  1. A `docs/decisions/003-connector-protocol.md` ADR documents the JSON-RPC protocol (methods, message framing, error shape, security model including SG-01 delegation).
  2. A reference connector binary `reposix-connector-github` (moved from the current in-tree `reposix-github` crate, or a new standalone binary that wraps it) authenticates + lists + reads via the protocol.
  3. `reposix list --backend $PROTOCOL/name` dispatches to `reposix-connector-<name>` on PATH, matches the `IssueBackend` shape the in-tree adapters produce (contract test parameterized over protocol-backed adapter too).
  4. A 30-minute "write your own connector" tutorial under `docs/connectors/` walks a Python user through building `reposix-connector-example` with just `stdin`/`stdout`.
  5. The existing `reposix list --backend github` (compiled-in path) still works — this is additive, not a replacement.

**Plans**: TBD (run `/gsd-plan-phase 12` next milestone)

### Phase 13: Nested mount layout: pages/ + tree/ symlinks for Confluence parentId hierarchy (BREAKING: flat <id>.md moves to pages/<id>.md) (v0.4)

**Goal:** Convert the FUSE mount from a flat `<padded-id>.md` root into a two-view layout — (a) a per-backend writable collection (`pages/` for Confluence, `issues/` for sim+GitHub) containing the canonical real files keyed by stable numeric id, and (b) a synthesized read-only `tree/` overlay of FUSE-emitted symlinks exposing Confluence's native parentId hierarchy at human-readable slug paths. The tree/ overlay dissolves concurrent-edit merge hell because the writable substrate stays on a single stable path regardless of title/reparent churn. Ships OP-1 from HANDOFF.md (the "hero.png" promise) scoped to Confluence-only; GitHub and sim remain flat under their per-backend collection bucket. BREAKING change: callers that read `mount/<id>.md` must now read `mount/pages/<id>.md` (Confluence) or `mount/issues/<id>.md` (sim + GitHub).
**Requirements**: OP-1 from HANDOFF.md — flat-layout-to-folder-structure promise. Addresses design Q#1 (symlinks not duplicate content), Q#2 (tree is read-only — writes go through the target), Q#4 (sibling-suffix collision resolution, no numeric id leakage into the human-visible slug path).
**Depends on:** Phase 11 (Confluence adapter). Independent of Phase 12 (connector ABI) — does not block or unblock it.
**Plans:** 9 plans across 5 waves

Plans:
- [x] 13-A-core-foundations
- [x] 13-B1-confluence-parent-id
- [x] 13-B2-fuse-tree-module
- [x] 13-B3-frontmatter-parent-id
- [x] 13-C-fuse-wiring
- [x] 13-D1-breaking-migration-sweep
- [x] 13-D2-docs-and-adr
- [x] 13-D3-release-scripts-and-demo
- [x] 13-E-green-gauntlet

### Phase 14: Decouple sim REST shape from FUSE write path and git-remote helper — route through IssueBackend trait (v0.4.1)

**Goal:** The FUSE daemon and git-remote helper route all read/write operations through `IssueBackend::{create_issue, update_issue, delete_or_close}` instead of the old hardcoded sim-specific REST shape (`fetch.rs` / `client.rs`). Deletes ~1,068 LoC. Closes HANDOFF.md open-gap items 7 and 8. Ships v0.4.1.
**Requirements**: HANDOFF.md known-open-gaps items 7 + 8
**Depends on:** Phase 13
**Plans:** 4 waves (A: sim 409 contract pins · B1: fs.rs write through IssueBackend · B2: git-remote through IssueBackend · C: verification · D: docs + CHANGELOG) — SHIPPED

Plans:
- [ ] TBD (run /gsd-plan-phase 14 to break down)

### Phase 15: Dynamic _INDEX.md synthesized in FUSE bucket directory (OP-2 partial) (v0.5.0)

**Goal:** Ships `mount/<bucket>/_INDEX.md` — a synthesized, read-only YAML-frontmatter + markdown-table sitemap of every tracked issue/page in the bucket directory, computed at read time from the `IssueBackend::list_issues` cache. Agents can `cat pages/_INDEX.md` for a one-shot directory overview. Partial OP-2 scope (bucket level only; tree-level and mount-root deferred to Phase 18). Ships v0.5.0.
**Requirements**: OP-2 (partial) from HANDOFF.md
**Depends on:** Phase 14
**Plans:** 2 waves (A: inode reservation + render_bucket_index + BucketIndex dispatch + 4 tests + live proof script · B: CHANGELOG [v0.5.0] + version bump + README + SUMMARY + tag script) — SHIPPED

Plans:
- [ ] TBD (run /gsd-plan-phase 15 to break down)


---

</details>
