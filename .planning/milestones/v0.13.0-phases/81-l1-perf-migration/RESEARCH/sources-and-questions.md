# Sources, Metadata, and Open Questions

← [back to index](./index.md)

## Sources

### Primary (HIGH confidence)
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Performance subtlety: today's `list_records` walk on every push" + § "3. Bus remote with cheap-precheck + SoT-first-write".
- `.planning/research/v0.13.0-dvcs/decisions.md` § Q3.1 (RATIFIED inline L1).
- `crates/reposix-core/src/backend.rs:215–264` (BackendConnector trait + `list_changed_since` default + signature `Vec<RecordId>`).
- `crates/reposix-confluence/src/lib.rs:141–`, `crates/reposix-github/src/lib.rs:472–`, `crates/reposix-jira/src/lib.rs:108–`, `crates/reposix-core/src/backend/sim.rs:281–303` — all four backends override `list_changed_since` with native incremental queries.
- `crates/reposix-cache/src/builder.rs::sync` (lines 220–end) — full L1 reference impl already in cache crate.
- `crates/reposix-cache/src/meta.rs` — `set_meta`/`get_meta` API + `last_fetched_at` is already a known cursor key (see `delta_sync.rs` tests).
- `crates/reposix-cache/src/cache.rs::list_record_ids` (line 345) + `find_oid_for_record` (line 381) — read-side primitives for cache-derived prior.
- `crates/reposix-remote/src/main.rs::handle_export` (lines 299–550 post-P80) — current call site shape.
- `crates/reposix-remote/src/diff.rs::plan` (line 99) — existing `prior: &[Record]` signature.
- `crates/reposix-cli/src/main.rs:37–326` — Cmd enum; confirms NO existing `Sync` subcommand.
- `quality/catalogs/perf-targets.json` + `quality/catalogs/agent-ux.json` — existing dimension homes for new rows.

### Secondary (verified via cross-reference)
- P80 mirror-ref ship pattern: `crates/reposix-cache/src/mirror_refs.rs` (`write_mirror_synced_at`, `read_mirror_synced_at`) — the public-API shape the new `read_last_fetched_at`/`write_last_fetched_at` should mirror.
- P79 attach precedent: `crates/reposix-cli/src/attach.rs` — clap subcommand structure for the new `Sync` subcommand.
- P73 wiremock idiom: `crates/reposix-confluence/tests/auth_header.rs::auth_header_basic_byte_exact` — pattern for byte-exact REST-call matchers in tests.

## Metadata

**Confidence breakdown:**
- Trait + cache substrate inventory: HIGH — code paths verified via grep + read.
- New algorithm shape: HIGH for the happy path; MEDIUM for the delete-detection seam (see §3 Pitfall 3 — planner must ratify "L1-strict" trade-off explicitly).
- Catalog row design: HIGH — existing dimension homes + verifier shapes are well-precedented.
- Test fixture strategy: MEDIUM — wiremock `expect(0)` semantics need confirmation in Task 4 (positive-control test recommended).
- L2/L3 deferral: HIGH — explicit in `decisions.md` Q3.1 and architecture-sketch.

**Research date:** 2026-05-01
**Valid until:** 2026-05-30 (30 days; substrate is stable Rust code with workspace-pinned deps).

## Open Questions for the Planner

1. **Delete-detection seam (Pitfall 3).** Confirm L1-strict (cache-trusted prior, REST 404 on backend-deleted record) is the chosen path. The architecture-sketch already says yes; this research confirms no surprise blockers in the code; the planner just needs to make the trade explicit in PLAN.md so the executing subagent and the verifier subagent both grade against the same contract.

2. **`Sync` subcommand vs `Refresh --reconcile`.** Architecture-sketch and ROADMAP use `reposix sync --reconcile`. The CLI today has `reposix refresh` (no `sync`). Two paths: (a) NEW `Sync { reconcile }` subcommand — recommended; (b) ADD `--reconcile` flag to existing `Refresh`. Recommend (a) — `refresh` writes `.md` files into a working tree (different from cache rebuild); conflating the two would muddy CLI semantics. (a) is also what the docs tell users to type.

3. **Should `plan()` signature change to `Vec<RecordId>` for the prior set?** Today `plan()` consumes `&[Record]` to compute deletes (records-in-prior-not-in-new). With L1, the prior shape is "IDs the cache knows about + lazily-fetched bodies." Two paths: (a) keep `plan()` signature; helper materializes a `Vec<Record>` from cache before calling — extra parses but minimal change to `diff.rs`; (b) change `plan()` to take `Vec<RecordId>` for the prior-id set + a closure to fetch the prior `Record` on demand — cleaner but wider blast radius. Recommend (a) — change is local to `handle_export`, `diff.rs` test surface untouched.

4. **Catalog row: `perf-targets.json` vs new `dvcs-perf.json`?** Recommend reuse `perf-targets.json` — three existing rows, well-precedented, freshly de-WAIVED in v0.12.1 P63. New file would orphan one row in its own catalog and complicate runner discovery.

5. **CLAUDE.md update scope.** Two paragraphs (one in § Commands, one in § Architecture or a new "L1 conflict detection" sub-section under § Architecture). Confirm the verifier-subagent grade rubric expects CLAUDE.md to land in the same PR per QG-07 (yes — checked).
