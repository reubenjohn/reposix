<!-- Shard of quality/reports/raise-list-p90.md (P90 RAISE LIST). Index has the other sections. -->
> [Back to RAISE LIST — P90 (index)](../raise-list-p90.md)

## 5. Waiver dispositions                                 <!-- 90-05 fills from R2 § D -->

25 waiver blocks existed repo-wide at R2 time: 19 in unified catalogs + 6
per-row in `doc-alignment.json`. Every renewed waiver below carries a fresh
`last_verified: 2026-07-04T12:00:00Z`, a future `until` (all ≤ 90 days from
that anchor, per PROTOCOL.md's waiver rule), and a `tracked_in` naming a LIVE
ROADMAP phase (P92/P95/P97) instead of the dead "v0.12.1
MIGRATE-03/SEC-0x/CROSS-0x" label — closing the C7 deferral-loop shape that
caused this exact cliff (12 rows all pointing at a milestone that never
shipped the promised gates).

| id | disposition | action taken in P90 90-05 |
|---|---|---|
| `release/cargo-binstall-resolves` | **LANDED, waiver CLEARED** (D90-02a) | The ~10-LOC `pkg-url`/`bin-dir` metadata alignment this waiver named was **already shipped pre-P90** by commit `33dd41f` "fix(binstall): correct pkg-url to real release asset name (QL-003)" (2026-07-03, an ancestor of P90's HEAD — D90-11-style already-fixed disposition). Re-verified against reality this dispatch: `crates/{reposix-cli,reposix-remote}/Cargo.toml`'s `pkg-url = "…/reposix-v{ version }-{ target }.tar.gz"` + `bin-dir = "reposix-v{ version }-{ target }/{ bin }{ binary-ext }"` (+ windows `.zip` override) byte-match `.github/workflows/release.yml`'s actual archive-staging logic (`release.yml:167-193`: `STAGE="reposix-${version_tag}-${target}"`, `ARCHIVE="${STAGE}.tar.gz"` or `.zip`). Status flipped `WAIVED` → `NOT-VERIFIED` (not a fabricated PASS — no real `cargo binstall --dry-run` container run was executed in this dispatch per the no-cargo constraint); waiver block removed; `expected.asserts` text updated to retire the stale "PARTIAL acceptable" carve-out. |
| `docs-repro/{example-01,02,04,05}` + `tutorial-replay` (5, EXPIRED 2026-05-12) | **RENEWED — still-broken** | Root cause re-confirmed live: `container-rehearse.sh` never brings up the external simulator the example scripts assume (`run.sh`/`run.py` abort "sim not reachable"); `tutorial-replay` additionally blocked by cold `cargo build` exceeding the 5-min container budget AND (push step) QL-001. New `until: 2026-09-15`, `tracked_in: P95` (RBF-D-14 RAISE LIST drain) for the container-infra cause; `tutorial-replay`'s push-step cause additionally needs P91's QL-001 fix first. |
| `cross-platform/{windows,macos}-rehearsal` (2026-07-26) | **RENEWED**, `tracked_in` repointed | Verifier confirmed still absent (`quality/gates/cross-platform/` empty save README/.gitkeep); windows/macos GH runner cost reasoning still holds. `until: 2026-09-15`, `tracked_in: P97` (launch-readiness slot) — dead `v0.12.1 CROSS-01/02` label repointed. |
| `perf/{latency-bench,token-economy-bench}` (2026-07-26) | **RENEWED**, repointed | Both confirmed still sim/fixture-derived only (no real-backend headline cross-check). `until: 2026-09-15`, `tracked_in: P97` — dead `v0.12.1 MIGRATE-03` label repointed. |
| `perf/headline-numbers-cross-check` (2026-07-26) | **RENEWED + dangling-verifier flag** | `quality/gates/perf/headline-numbers-cross-check.py` reconfirmed absent (see § 1). `until: 2026-09-15`, `tracked_in: P97`, `owner_hint` now explicitly names the missing verifier (was previously only implicit in the waiver reason). |
| `security/allowlist-enforcement` + `security/audit-immutability` (2026-07-26, P0) | **RENEWED + FIXED the 2 dangling verifier scripts** (D90-02d) | Both verifier scripts landed this commit: `quality/gates/security/allowlist-enforcement.sh` wraps the REAL, pre-existing `cargo test -p reposix-core --test http_allowlist` (13 tests already covering egress-reject/redirect-recheck/env-override/loopback-allow) + a static grep of `clippy.toml` + the allowlist-check-before-send ordering in `http.rs:294-321`; `quality/gates/security/audit-immutability.sh` wraps the REAL, pre-existing `cargo test -p reposix-core --test audit_schema` (8 tests) + `cargo test -p reposix-cache --test audit_is_append_only` (1 test) + a static grep confirming WAL mode on the cache-side connection. **Neither test suite was previously known to be missing — R2's "no test named http_allowlist/audit_immutability found" was a substring-match miss; the real integration-test files exist and (per manual code review) look correct.** The wrapper scripts themselves have NOT been executed via a real `cargo test` invocation in this dispatch (hard no-cargo constraint) — `until: 2026-08-15` (sooner runway than the other renewals, P0), `tracked_in: P92`, with an **explicit line item**: confirm both wrapper scripts' exit codes via a real pre-pr/CI run and flip WAIVED→PASS once confirmed. |
| `subjective/{cold-reader-hero-clarity,install-positioning,headline-numbers-sanity}` (2026-07-26) | **RENEWED — NOT mooted by 90-03's dispatch wiring** (D90-02b) | Confirmed: these 3 rows were ALREADY dispatch-wired before 90-03; the waiver covers a distinct gap (the runner subprocess lacks Task tool access, so a bare re-sweep would overwrite the ratified Path B artifacts — scores 8/9/9 CLEAR — with Path-B stubs). 90-03's dvcs-cold-reader wiring fix does not touch this gap. `until: 2026-09-15`, `tracked_in: P95` (runner dispatch-and-preserve invariant) — dead `v0.12.1 MIGRATE-03` label repointed. |
| `code/cargo-test-pass` (2026-07-26) | **RENEWED** | Reason re-confirmed still true: local `cargo nextest run --workspace` still violates the CLAUDE.md memory-budget rule + pre-pr cadence cap; `ci.yml`'s `test` job remains canonical. `until: 2026-09-15`, `tracked_in: P97` — dead `v0.12.1 MIGRATE-03` label repointed. |
| `structure/file-size-limits` (2026-08-08) | **Left alone (not at cliff)** | Not touched — its `until` is still >30 days out and its `tracked_in` ("v0.13.0 extension") is not the dead-label pattern. Noting per plan: 10 enumerated violations remain undrained (6 research-bundle files + 3 + the AGENTS.md symlink); the symlink-exclusion verifier fix is a separate XS item, not landed here. |
| `agent-ux/real-git-push-e2e` (2026-07-31) | **NOT renewed** (D90-01, confirmed, unchanged) | Routes to P91's QL-001 fix. Waiver is the intentional backstop if P91 slips; P90/90-05 does not touch it. |
| `docs/index/git-checkout-branch-command` (2026-07-31) | **STAYS WAIVED** (D90-01/-07, confirmed, unchanged) | QL-001-blocked by definition; routes to P91. Not touched here (doc-alignment dimension, 90-06 territory besides). |
| 5× doc-alignment MISSING_TEST rows (`cli.md` ×4 + `exit-codes-locked`) (2026-07-31) | **Handled in 90-06** | Pointer only — real tests land in 90-06's cargo wave per D90-07(1); not touched by 90-05. |

**Mass-renewal audit (D90-02 closing check):** every renewed waiver above now
carries a `tracked_in` naming P92, P95, or P97 — all three exist in
`.planning/milestones/v0.13.0-phases/ROADMAP.md` today. The
`quality/reports/raise-list-p90.md`-referenced sweep (`grep -r 'v0.12.1'
quality/catalogs/*.json | grep tracked_in`, run as part of this dispatch's
verification) returns zero hits — the 2026-07-26 cliff will not silently
repeat in 3 weeks.
