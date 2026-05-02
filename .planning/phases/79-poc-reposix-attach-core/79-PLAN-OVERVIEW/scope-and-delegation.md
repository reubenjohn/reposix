← [back to index](./index.md)

# +2 reservation: out-of-scope candidates

Initialize `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` and
`GOOD-TO-HAVES.md` lazily — only when an entry surfaces during a plan's
execution. P78 closed without surfacing entries; **P79 surfaces ONE
GOOD-TO-HAVES entry pre-execution** (GOOD-TO-HAVES-01, see § above).

Anticipated candidates the plans flag (per OP-8):

- **MEDIUM** — POC time overrun (79-01 T05 fail mode). >2d → SURPRISES candidate.
- **MEDIUM** — Reconciliation rule count exceeds 5 (79-01 fail mode). Routes to 79-02 plan revision; if scope expansion is unmanageable, SPLIT (orchestrator decides).
- **LOW** — `--orphan-policy` option needs additional values not enumerated in sketch. Eager-resolve in 79-02 if < 30 min; else GOOD-TO-HAVES.
- **LOW** — `reposix detach` referenced in Q1.2 error message but not implemented. If error message is acceptable without a working `detach` command, file as GOOD-TO-HAVES; otherwise SURPRISES.
- **LOW** — POC reveals that `extensions.partialClone=<reposix-remote-name>` clobbers a pre-existing `extensions.partialClone=origin` set by a prior `reposix init`. Per Q1.3 decision: idempotent re-attach refreshes cache state — confirm clobber is the right semantic in POC; if not, file as REVISE.
- **PRE-EXECUTION** — GOOD-TO-HAVES-01 (already filed): `reposix-quality bind` agent-ux dim support.

Items NOT in scope for P79 (deferred per the v0.13.0 ROADMAP):

- Mirror-lag refs (P80). Do not pre-stage `refs/mirrors/...` write code.
- Bus remote URL parser / prechecks / writes (P82+). Bus-remote *shape* may be exercised in POC for findings-gathering only; production implementation defers.
- L1 perf migration (P81). Do not touch the existing `list_records` walk.
- Webhook-driven sync (P84). Out of scope.
- DVCS docs (P85). Out of scope; 79-03 only updates CLAUDE.md, not `docs/`.
- L2/L3 cache-desync hardening (deferred to v0.14.0 per architecture-sketch).
- Multi-SoT attach (Q1.2 — explicitly REJECTED in 79-02; v0.14.0 origin-of-truth scope).

# Subagent delegation

Per CLAUDE.md "Subagent delegation rules" + the gsd-planner spec
"aggressive subagent delegation":

| Plan / Task                              | Delegation                                                                                                                                                                                                                                          |
|------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 79-01 T01 (POC scaffold + fixtures)      | `gsd-executor` — small shell + Rust scratch; holds cargo lock for any Rust compile.                                                                                                                                                                |
| 79-01 T02-T04 (POC three-paths run)      | Same 79-01 executor (continues sequentially). Time-bounded — orchestrator pings at 4hr / 8hr / 16hr marks.                                                                                                                                       |
| 79-01 T05 (FINDINGS + push)              | Same 79-01 executor (terminal task).                                                                                                                                                                                                              |
| **POC-FINDINGS checkpoint**              | Orchestrator (top-level) reads FINDINGS, decides routing. If REVISE/SPLIT tag fires, dispatches `gsd-planner` with `<revision_context>` for an in-place 79-02 (or 79-03) plan revision (NEW commit; NO amend).                                    |
| 79-02 T01 (catalog row + verifier)       | `gsd-executor` — catalog-first commit; **hand-edits agent-ux JSON per documented gap (NOT Principle A)**; verifier shell ships in same atomic commit.                                                                                            |
| 79-02 T02 (subcommand scaffold)          | Same 79-02 executor. Cargo lock held for the cli crate.                                                                                                                                                                                          |
| 79-02 T03 (cache reconciliation module + new APIs + audit hook + Tainted type assertion + push) | Same 79-02 executor (terminal task). Adds `Cache::list_record_ids` / `find_oid_for_record` / `connection_mut` / `log_attach_walk` (confirmed not-yet-extant via grep). |
| 79-03 T01 (5 reconciliation integration tests) | `gsd-executor` (Wave 3) — holds cargo lock; runs `cargo nextest run -p reposix-cli --tests attach`.                                                                                                                                              |
| 79-03 T02 (re-attach idempotency + reject + Tainted-materialization integration test + audit-row test) | Same 79-03 executor.                                                                                                                                                                                                                              |
| 79-03 T03 (CLAUDE.md update + catalog flip + push)   | Same 79-03 executor (terminal task).                                                                                                                                                                                                              |
| Phase verifier (P79 close)               | Unbiased subagent dispatched by orchestrator AFTER 79-03 pushes per `quality/PROTOCOL.md` § "Verifier subagent prompt template" (verbatim). Zero session context; grades catalog rows from artifacts.                                            |

Phase verifier subagent's verdict criteria (extracted for P79):

- POC-01: `research/v0.13.0-dvcs/poc/run.sh` exits 0 against the simulator; `POC-FINDINGS.md` exists with `## Implications for 79-02` subsection; three integration paths (a/b/c) each have a transcript-style log under `research/v0.13.0-dvcs/poc/logs/` or inline in FINDINGS; time-budget annotation present.
- DVCS-ATTACH-01: `reposix attach` subcommand exists in `--help` output; integration test `attach_against_vanilla_clone_sets_partial_clone` passes; cache appears at `resolve_cache_path(<sot-backend>, <sot-project>)`; remote URL written matches `reposix::<sot-spec>` (or `?mirror=` form when default `--bus`).
- DVCS-ATTACH-02: 5 reconciliation test cases pass — `attach_match_records_by_id`, `attach_warns_on_backend_deleted`, `attach_skips_no_id_files`, `attach_errors_on_duplicate_id`, `attach_marks_mirror_lag_for_next_fetch`.
- DVCS-ATTACH-03: 2 tests pass — `re_attach_same_sot_is_idempotent`, `re_attach_different_sot_is_rejected`.
- DVCS-ATTACH-04 (REFRAMED): (1) static type-system assertion test compiles + passes — proves `Cache::read_blob` returns `Tainted<Vec<u8>>`; (2) integration test `attach_then_read_blob_returns_tainted` passes — exercises one materialization via `Cache::read_blob` after attach and asserts the bytes are `Tainted<Vec<u8>>`.
- New catalog row `agent-ux/reposix-attach-against-vanilla-clone` is bound (hand-edit per documented gap GOOD-TO-HAVES-01) with `kind: mechanical`, `cadence: pre-pr`, verifier exits 0; status PASS after 79-03 T03.
- OP-3: cache `audit_events_cache` table contains a row with `event_type = 'attach_walk'` (or whatever `Cache::log_attach_walk` writes) for the integration-test attach run. **Unconditional — no deferral pre-authorized.**
- Recurring (per phase): catalog-first ordering preserved (79-02 T01 commits catalog rows BEFORE T02-T03 implementation); per-plan push completed; verdict file at `quality/reports/verdicts/p79/VERDICT.md`; CLAUDE.md updated in 79-03.
