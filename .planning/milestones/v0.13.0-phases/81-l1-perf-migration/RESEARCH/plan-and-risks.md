# Plan Splitting, Risks, and Documentation Deferrals

← [back to index](./index.md)

## Plan Splitting

**Recommended: SINGLE PLAN with 4 tasks.**

| Task | Goal | Cargo-heavy? |
|------|------|--------------|
| **Task 1** | Catalog rows + new `Cache::read_last_fetched_at` / `write_last_fetched_at` public methods + cli `Sync { reconcile }` subcommand stub | Yes — `cargo check -p reposix-cache -p reposix-cli` |
| **Task 2** | Helper precheck rewrite (replace lines 334–382) + `plan()` called against cache-derived prior + cursor write after success + L2/L3 inline comment + first-push fallback | Yes — `cargo check -p reposix-remote`; `cargo test -p reposix-remote` for existing conflict-detection tests |
| **Task 3** | `reposix sync --reconcile` handler in `crates/reposix-cli/src/sync.rs` + smoke test | Yes — `cargo test -p reposix-cli --test sync` |
| **Task 4** | `crates/reposix-remote/tests/perf_l1.rs` (N=200 wiremock-counted regression) + CLAUDE.md update + README.md (if Commands section names `reposix sync --reconcile`) + verdict push | Yes — `cargo test -p reposix-remote --test perf_l1` |

4 cargo-heavy tasks fits inside the CLAUDE.md "≤4 cargo-heavy tasks per plan" guideline. Sequential per CLAUDE.md "Build memory budget"; per-crate `cargo check`/`test` invocations only.

If any task balloons (e.g., Task 2 reveals a hidden coupling between `plan()` and the `&[Record]` shape that requires a wider refactor), surface as a SURPRISES-INTAKE candidate per OP-8 rather than expanding the phase.

## Pitfalls and Risks (consolidated)

(See [Common Pitfalls](./pitfalls.md) for detailed analysis.)

| Risk | Severity | Mitigation |
|------|----------|------------|
| Backend-side deletes silently miss precheck | MEDIUM (user-visible at REST time as 404) | Document the L1 trade-off inline + in CLAUDE.md; user recovery via `reposix sync --reconcile`. |
| Clock skew false-positives | LOW | Self-healing on next push; document as known quirk. |
| First-push has no cursor → falls back to `list_records` | LOW (intentional) | Already in algo; one-time cost. |
| Cache write fails after REST write succeeds | LOW (P80 already establishes "best-effort cache writes don't poison ack" pattern) | Same pattern; warn-log; user recovers via `reposix sync --reconcile`. |
| `wiremock::Match::expect(0)` doesn't actually fail RED if list_records is called | MEDIUM (test could silently pass) | Verify wiremock semantics during Task 4; add a positive control (a SECOND test that DOES expect a list_records call to ensure the matcher works). |
| Tainted prior-blob bytes leak into log lines | LOW-MEDIUM (OP-2 violation) | Existing `log_helper_push_rejected_conflict` only takes `id + versions`; preserve that shape; never log blob body. |

## Documentation Deferrals

- **`docs/concepts/dvcs-topology.md`** — P85, NOT P81. P85 will write the full L2/L3 deferral note + user-facing "what `--reconcile` does" prose.
- **P81 documents inline:**
  - One comment block in `crates/reposix-remote/src/main.rs` near the new precheck citing `architecture-sketch.md` § "Performance subtlety" + the v0.14.0 hardening doc (per success criterion 5).
  - Two CLAUDE.md additions: (1) § Commands gets `reposix sync --reconcile` documented under the "Local dev loop" block; (2) § Architecture (or a new sub-section) names the L1 trade-off in 1–2 sentences and points at `architecture-sketch.md`.

**Confirmed:** No new docs/site pages in P81. Doc-alignment row binds the existing architecture-sketch prose to the regression test; no fresh prose is authored.
