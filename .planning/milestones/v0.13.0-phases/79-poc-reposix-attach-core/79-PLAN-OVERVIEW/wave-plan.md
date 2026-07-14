← [back to index](./index.md)

# Wave plan

Strictly sequential — three waves, one plan each. POC findings inform
attach design; scaffold lands before tests; tests must observe the
implementation from 79-02.

| Wave | Plans | Cargo? | File overlap         | Notes                                                                                              |
|------|-------|--------|----------------------|----------------------------------------------------------------------------------------------------|
| 1    | 79-01 | YES*   | none with 79-02/03   | POC may compile small scratch Rust modules (gix-walking proof, frontmatter-id matching proof)      |
| 2    | 79-02 | YES    | none with 79-01      | scaffold in `crates/reposix-cli/` + reconciliation module in `crates/reposix-cache/`               |
| 3    | 79-03 | YES    | overlap with 79-02   | adds tests + 1 new line in 79-02-touched files (`attach.rs` + `reconciliation.rs` may be edited if a tested behavior reveals a small fix); CLAUDE.md edit |

\* POC may use shell + small-scratch-Rust mix; if Rust scratch is limited to
a single new throwaway crate at `research/v0.13.0-dvcs/poc/scratch/Cargo.toml`
(NOT a workspace member), it does not contend with workspace cargo locks
that 79-02 will later acquire.

Per CLAUDE.md "Build memory budget" the executor for each wave holds the
cargo lock for that wave's compilation. Sequential by-wave honors this
without ambiguity.

`files_modified` overlap audit (per gsd-planner same-wave-zero-overlap rule):

| Plan  | Files                                                                                                                                                                                                                                                                                                                                                              |
|-------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 79-01 | `research/v0.13.0-dvcs/poc/run.sh`, `research/v0.13.0-dvcs/poc/POC-FINDINGS.md`, `research/v0.13.0-dvcs/poc/{scratch/**,fixtures/**}` (all under throwaway path); CLAUDE.md edit OPTIONAL (only if a permanent learning surfaces; default is NO CLAUDE.md edit since POC is throwaway)                                                                              |
| 79-02 | `crates/reposix-cli/src/main.rs`, `crates/reposix-cli/src/lib.rs`, `crates/reposix-cli/src/attach.rs` (new), `crates/reposix-cache/src/lib.rs`, `crates/reposix-cache/src/reconciliation.rs` (new), `crates/reposix-cache/src/db.rs`, `crates/reposix-cache/src/cache.rs` (3 new public APIs + audit hook), `crates/reposix-cache/src/builder.rs` (audit-call site if needed), `quality/catalogs/agent-ux.json`, `quality/gates/agent-ux/reposix-attach.sh` (new) |
| 79-03 | `crates/reposix-cli/tests/attach.rs` (new), `crates/reposix-cache/src/cache.rs` (one-line `materialized_blob_is_tainted` integration test wiring may belong in `tests/`; see plan), `crates/reposix-cli/src/attach.rs` OR `crates/reposix-cache/src/reconciliation.rs` (small fix-forward if a test surfaces a defect; defaults to NO edit), `quality/catalogs/agent-ux.json` (status FAIL → PASS rewritten by runner), `CLAUDE.md`                                                                  |

Wave 1 ↔ Wave 2 file overlap: NONE (POC under `research/`, scaffold under `crates/`).

Wave 2 ↔ Wave 3 file overlap: ALLOWED (sequential dependency). Wave 3
edits `quality/catalogs/agent-ux.json` (the runner rewrites the row's
`status` field) AND may patch `crates/reposix-cli/src/attach.rs` or
`crates/reposix-cache/src/reconciliation.rs` if integration tests surface
a defect — but only as fix-forward; the SCAFFOLD lands in 79-02 and is
considered complete then.

Optional: if 79-01 surfaces a permanent learning that warrants a CLAUDE.md
edit (e.g., a new convention paragraph), the orchestrator folds that edit
into 79-03's CLAUDE.md update — keeps CLAUDE.md edits to one wave.
