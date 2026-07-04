# P90 R2 — Catalog + Test Honesty Inventory

**Lane:** P90 RESEARCH R2 · **Date:** 2026-07-04 · **Tree:** main (clean) ·
**Charter:** ROADMAP.md § Phase 90 (`.planning/milestones/v0.13.0-phases/ROADMAP.md:153-169`);
F-K4/F-K8 specs at `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md:123,127`;
row schema at `quality/catalogs/README.md`.
**Consumers:** RAISE LIST (`quality/reports/raise-list-p90.md`), the
`quality/gates/agent-ux/test-name-vs-asserts.sh` gate (F-K8), and P92/P94/P95 planning.
Method: read-only grep/read; per-section enumeration delegated to parallel research
subagents, verifier scripts read directly; zero cargo invocations (VM memory budget).

---

## A. Transport/perf-claim row inventory (F-K4a)

Walked every row in all 12 non-doc-alignment catalogs (`doc-alignment.json` exempt —
distinct schema per `quality/catalogs/README.md` § "docs-alignment dimension").
Coverage legend: **real-backend** (non-loopback REST) / **sim** (in-process simulator,
127.0.0.1:78xx) / **localhost** (wiremock + `file://` mirror + `git init --bare`) /
**static-grep** (source/string asserts only, nothing executed) / **vacuous**
(assertion cannot fail meaningfully).

### A.1 Flagged rows — agent-ux.json

| id | kind | cadences | status | what the verifier ACTUALLY exercises | F-K4a disposition |
|---|---|---|---|---|---|
| agent-ux/dark-factory-sim | mechanical | on-demand | PASS | **sim + static-grep.** `dark-factory/sim.sh:31-48` spawns sim + `reposix init` + git-config asserts; the "teaching strings **emit** on conflict/blob-limit paths" claim is verified by grepping *source* (`sim.sh:53` greps `git sparse-checkout` in `stateless_connect.rs`; `:58` greps `git pull --rebase` in `main.rs`/`write_loop.rs`). **Never pushes** (QL-001 FINDING-B confirmed). | **description-soften** — "emit" → "present in helper source"; tag `coverage_kind: sim`. |
| agent-ux/reposix-attach-against-vanilla-clone | mechanical | pre-pr | PASS | **sim.** Real `reposix attach` vs sim REST; post-attach config asserts. | coverage_kind: sim + honest-description. |
| agent-ux/mirror-refs-write-on-success | mechanical | pre-pr | PASS | **localhost.** `cargo test --test mirror_refs write_on_success…` — helper via stdin vs wiremock. | coverage_kind: sim + honest (description names wiremock). |
| agent-ux/mirror-refs-readable-by-vanilla-fetch | mechanical | pre-pr | PASS | **localhost.** `mirror_refs::vanilla_fetch_brings_mirror_refs` — real `git clone --mirror` of local fixture. | coverage_kind: sim + honest. |
| agent-ux/mirror-refs-cited-in-reject-hint | mechanical | pre-pr | PASS | **localhost.** wiremock reject-hint test. | coverage_kind: sim + honest. |
| agent-ux/sync-reconcile-subcommand | mechanical | pre-pr | PASS | **sim/localhost.** `cargo test -p reposix-cli --test sync sync_reconcile_advances_cursor`. | coverage_kind: sim + honest. |
| agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first | mechanical | pre-pr | PASS | **localhost.** Real `git ls-remote` vs `file://` mirror + wiremock SoT (`bus_precheck_a.rs:57-72`). | coverage_kind: sim + honest. |
| agent-ux/bus-precheck-b-sot-drift-emits-fetch-first | mechanical | pre-pr | PASS | **localhost.** wiremock `?since=` drift + `Mock::expect(0)` on PATCH. | coverage_kind: sim + honest. |
| agent-ux/bus-fetch-not-advertised | mechanical | pre-pr | PASS | **localhost.** Capability-list assert. | coverage_kind: sim + honest. |
| agent-ux/bus-write-sot-first-success | mechanical | pre-pr | PASS | **localhost.** Helper `export` via stdin vs wiremock + `file://` mirror (`bus-write-sot-first-success.sh:21`). | coverage_kind: sim + honest. |
| agent-ux/bus-write-mirror-fail-returns-ok | mechanical | pre-pr | PASS | **localhost.** wiremock fault injection. | coverage_kind: sim + honest. |
| agent-ux/bus-write-no-helper-retry | mechanical | pre-pr | PASS | **static-grep.** `bus-write-no-helper-retry.sh:31-54` greps `bus_handler.rs` for retry constructs (`for _ in 0..`, `loop {`, `sleep`, `--force`). Runtime "single push_mirror invocation" is NOT executed here (covered by the wiremock sibling row). | **description-soften** — reframe as source-hygiene check; note the runtime claim lives in bus-write-mirror-fail-returns-ok. |
| agent-ux/bus-write-no-mirror-remote-still-fails | mechanical | pre-pr | PASS | **localhost.** `bus_write_no_mirror_remote` test. | coverage_kind: sim + honest. |
| agent-ux/bus-write-fault-injection-{mirror-fail, sot-mid-stream, post-precheck-409} | mechanical | pre-pr | PASS | **localhost.** wiremock 5xx/409 fault-injection suites. | coverage_kind: sim + honest (×3 rows). |
| agent-ux/bus-write-audit-completeness | mechanical | pre-pr | PASS | **localhost.** wiremock request-log + audit-row asserts. | coverage_kind: sim + honest. |
| agent-ux/webhook-trigger-dispatch | mechanical | on-demand | PASS | **static-grep + one real `gh api` GET.** `webhook-trigger-dispatch.sh:35` fetches live YAML, `:42` `diff -w` byte-equality, `:47-66` structural greps. **Never triggers a dispatch, never measures a sync.** | **description-soften** — honest as "reference workflow present + byte-equal", not "webhook-driven sync verified". |
| agent-ux/webhook-force-with-lease-race | mechanical | pre-pr | PASS | **localhost.** Local bare-repo SHA-race fixture; `--force-with-lease` rejection. | coverage_kind: sim + honest. |
| agent-ux/webhook-first-run-empty-mirror | mechanical | pre-pr | PASS | **localhost.** Local bare-repo lease-vs-plain-push branches. | coverage_kind: sim + honest. |
| agent-ux/webhook-latency-floor | asset-exists | pre-release | PASS | **VACUOUS.** `webhook-latency-floor.sh:14-20` reads the *committed* artifact `quality/reports/verifications/perf/webhook-latency.json` (`n:1`, `method:"synthetic-dispatch"`, `p95_seconds:5`) and asserts `<=120`. The artifact's own `note` says it measures dispatch→runner-pickup only; full-sync measurement deferred. Matches REMEDIATION-PLAN H-I11. | **WAIVED + until_date** (or soften to "dispatch-pickup latency, n=1, synthetic"). Real n=10 measurement gated on v0.13.x release with working binstall (`scripts/webhook-latency-measure.sh`). |
| agent-ux/dvcs-third-arm | subagent-graded | pre-pr | PASS | **sim + static-grep, no push, no subagent.** See § B.1. Header self-discloses `git push reposix main` NOT exercised (`dvcs-third-arm.sh:20-23`); wire-path "coverage" is a test-fn-existence grep (`:180-182`). | **coverage_kind: sim + honest-description** for transport; kind flip per § B.1. |
| agent-ux/milestone-close-vision-litmus-real-backend | shell-subprocess | pre-release-real-backend | NOT-VERIFIED | **Vacuous placeholder by design.** `milestone-close-vision-litmus.sh:36-50` always writes hardcoded NOT-VERIFIED + exit 75 (P91–P95 substrate absent). Honest per PROTOCOL.md OD-2 — but genuinely zero coverage today. | **already-honest** (P0, no waiver allowed by design). P91 fills the body (D90-06). |
| agent-ux/real-git-push-e2e | mechanical | pre-release, on-demand | WAIVED | **sim — the only genuine end-to-end `git push` in the framework.** `real-git-push-e2e.sh:109-155`: `reposix init` → checkout → edit → real `git push origin main` → asserts sim `audit_events` exactly 1 PATCH / 0 POST / 0 DELETE + no-op-push-writes-nothing. WAIVED because QL-001 (`diff.rs:106` path-shape bug) makes it honestly FAIL; exit 75 on git<2.34. | **coverage_kind: sim + honest-description.** Waiver dies with the P91 QL-001 fix (D90-01). |

Not flagged (checked, genuinely non-transport): `bus-url-*` parse rows, `bus-no-remote-configured-error`, `cadence-pre-release-real-backend` (skip-logic infra), `kind-shell-subprocess-worked-example` (kind-demo row; `bash --version` fallback is part of its documented contract per README.md), p87/p88 planning-presence rows.

### A.2 Flagged rows — docs-reproducible.json, perf-targets.json, release-assets.json, security-gates.json

| id | catalog | kind | cadences | status | what the verifier ACTUALLY exercises | F-K4a disposition |
|---|---|---|---|---|---|---|
| docs-repro/example-01-shell-loop | docs-reproducible | container | post-release | WAIVED | **sim (containerized)** — `container-rehearse.sh:109-115` runs `examples/01/run.sh` in ubuntu:24.04; **container never brings up the sim**, so the run aborts at `run.sh:19-21` "sim not reachable". | coverage_kind: sim + still-broken; see § D. |
| docs-repro/example-02-python-agent | docs-reproducible | container | post-release | WAIVED | same sim-not-up failure (`run.py:67-70`). | same. |
| docs-repro/example-04-conflict-resolve | docs-reproducible | container | post-release | WAIVED | same. | same. |
| docs-repro/example-05-blob-limit-recovery | docs-reproducible | container | post-release | WAIVED | same. | same. |
| docs-repro/tutorial-replay | docs-reproducible | container | post-release | WAIVED | **sim** — self-spawns sim (`tutorial-replay.sh:47-50`) but cold `cargo build` blows 5-min container budget; push step also QL-001-blocked. | coverage_kind: sim + still-broken; see § D. |
| benchmark-claim/8ms-cached-read | docs-reproducible | manual | weekly | NOT-VERIFIED | **VACUOUS.** `verifier.script: None`; asserts frontmatter freshness + headline greppable. No measurement in-row (measurement lives in sim-only, WAIVED perf/latency-bench). | **description-soften / WAIVE** — latency headline with no live measurement. |
| benchmark-claim/89.1-percent-token-reduction | docs-reproducible | manual | weekly | NOT-VERIFIED | **VACUOUS.** `verifier.script: None`; headline greppable + fixture exists. | **description-soften / WAIVE**. |
| perf/latency-bench | perf-targets | mechanical | weekly | WAIVED | **sim.** `latency-bench.sh:85-91` builds + spawns sim (127.0.0.1:7780), median-of-3 golden path. Real-backend columns skip without creds. Headline cross-check deferred (script self-declares stub). | coverage_kind: sim + honest-description ("numbers are sim-derived"). |
| perf/token-economy-bench | perf-targets | mechanical | weekly | WAIVED | **fixture/modeled.** `bench_token_economy.py` computes token counts from committed `benchmarks/fixtures/*.tokens.json` via `count_tokens`. Modeled comparison, not a live agent run. | coverage_kind: sim + honest-description ("modeled from fixtures"). |
| perf/headline-numbers-cross-check | perf-targets | mechanical | weekly | WAIVED | **static-grep by intent** — but verifier `.py` is **missing entirely** (confirmed absent from `quality/gates/perf/`). | **WAIVED + until_date**; dangling-verifier flag (see § H). |
| perf/handle-export-list-call-count | perf-targets | mechanical | on-demand | PASS | **localhost.** `perf_l1` test, N=200 wiremock, counts `list_changed_since` vs `list_records` calls. Genuinely drives the helper. | coverage_kind: sim + honest. |
| release/cargo-binstall-resolves | release-assets | container | post-release | WAIVED | **real-network install resolution.** `cargo-binstall-resolves.py:55` real `cargo binstall --dry-run`; PARTIAL accepted for source-fallback. | already-honest (install path, not product transport); waiver landable per § D. |
| security/allowlist-enforcement | security-gates | mechanical | pre-pr | WAIVED | **NO VERIFIER EXISTS** — `quality/gates/security/allowlist-enforcement.sh` absent (only `connector-audit-wired.sh` present in that dir). The outbound-allowlist rejection IS a transport claim; unverified at the gate layer. | **WAIVED + until_date** with dangling-verifier called out; P0 threat-model cut — prioritize (see § D). |

Not flagged: `code.json`, `cross-platform.json`, `docs-build.json`, `freshness-invariants.json`, `orphan-scripts.json` carry no transport/perf-behavior rows. The `release/gh-assets-present` / `install/*` / `crates-io-max-version` / `docs-build/badges-resolve` rows do real HTTP but are honest external-asset existence checks doing exactly what they claim. `security/audit-immutability`'s verifier is also missing (see § D/§ H) but its claim is SQLite append-only, not transport.

### A.3 Headline conclusion for the RAISE LIST

1. Most bus/mirror/precheck agent-ux rows are **honest localhost** — they genuinely drive `git-remote-reposix` via stdin against wiremock + `file://`, and say so. No overclaim; F-K4a tagging is `coverage_kind: sim` + no text change.
2. **Genuine overclaim/soft targets (RAISE):** `agent-ux/dark-factory-sim` ("emit" vs source-grep), `agent-ux/webhook-latency-floor` (vacuous n=1 synthetic), `agent-ux/webhook-trigger-dispatch` (YAML equality ≠ webhook runtime), `agent-ux/bus-write-no-helper-retry` (source grep for runtime claim), `benchmark-claim/{8ms,89.1%}` (verifier-less headlines), `security/allowlist-enforcement` + `perf/headline-numbers-cross-check` (dangling verifier paths).
3. **There is zero green real-backend transport coverage today.** The only true e2e push (`real-git-push-e2e`) is sim-only + WAIVED (QL-001); the real-backend litmus row is a designed exit-75 placeholder. Honest, but the gap is total — this is the quantified justification for P91/P92.

---

## B. Subagent-graded wiring audit (F-K4c)

Confirmed exactly **5** `kind: subagent-graded` rows (grep: 4 in `subjective-rubrics.json`, 1 in `agent-ux.json`).

| id | catalog | verifier.script | dispatch-wired? | recommendation |
|---|---|---|---|---|
| subjective/cold-reader-hero-clarity | subjective-rubrics.json | dispatch.sh | **YES** — `dispatch.sh:59-61` → `lib/dispatch_cold_reader.sh` (invokes `claude /doc-clarity-review` on README + docs/index, `dispatch_cold_reader.sh:37`). | keep. |
| subjective/install-positioning | subjective-rubrics.json | dispatch.sh | **YES** — `dispatch.sh:62-68` → `lib/dispatch_inline_subagent.sh:20`. | keep. |
| subjective/headline-numbers-sanity | subjective-rubrics.json | dispatch.sh | **YES** — `dispatch_inline_subagent.sh:24`. | keep. |
| subjective/dvcs-cold-reader | subjective-rubrics.json:135 | dispatch.sh | **NO — wiring gap.** Not in `dispatch.sh:58-69`'s `--rubric` cases; `dispatch_cold_reader.sh:18-19,28` is hardcoded to hero-clarity/README+index; `dispatch_inline_subagent.sh:19-33` handles only the other two ids. `--rubric subjective/dvcs-cold-reader` falls to the Path-B **stub** (`dispatch.sh:70-76`). Targets docs/concepts/dvcs-topology.md + 2 guides; no real grading path exists. Status honestly NOT-VERIFIED. | **WIRE-DISPATCH** — add a dispatch.sh case + a dvcs cold-reader lib targeting its 3 docs. This is the real F-K4c decorative-kind instance, not dvcs-third-arm's script. |
| agent-ux/dvcs-third-arm | agent-ux.json:992 (kind at :994) | quality/gates/agent-ux/dark-factory.sh | **N/A — zero subagent grading anywhere.** | **FLIP-TO-MECHANICAL** (§ B.1). |

### B.1 dvcs-third-arm: flip-to-mechanical (evidence)

`dark-factory.sh:36-59` is a pure arm dispatcher → `dark-factory/dvcs-third-arm.sh`, which is
entirely deterministic shell: source teaching-string greps (`:79-94`), `--help` greps
(`:99-116`), real `reposix attach` subprocess vs sim (`:134`), git-config asserts
(`:145-161`), sqlite3 `attach_walk` audit count (`:170-176`), and a test-fn-existence grep
for the wire path (`:180-182`). The header states "No real LLM in CI" (`:11`). No
`claude`/Task/rubric call exists in the script or its parents.

**Recommendation: FLIP-TO-MECHANICAL** — the row's GREEN contract is fully falsifiable by
shell asserts; `subagent-graded` is decorative (p86 F7's exact shape). This satisfies
D90-10's "if deterministic assert script with no grading step → `kind: mechanical`".

### B.2 Underscore-status typo check — MOOTED (already fixed)

`grep -rn 'NOT_VERIFIED' quality/catalogs/` → **zero hits**. Full stored-status census
across all 11 non-doc-alignment catalogs: 92× `PASS`, 19× `WAIVED`, 4× `NOT-VERIFIED`
(all canonical hyphen; `subjective-rubrics.json:166` for dvcs-cold-reader is hyphenated).
Git history: the typo was introduced at `06b8014` (P85 catalog-row commit) and removed at
`c0d5459` (D-CONV-3 scripts-collapse). **The SURPRISES-INTAKE 2026-07-03 21:35 LOW entry
is RESOLVED-by-c0d5459** — the intake entry should be closed with that SHA rather than
re-fixed. The only remaining `NOT_VERIFIED` strings in the repo are legitimate Python
constant names (`quality/runners/_realbackend.py:54,96`, `run.py:323`).

---

## C. Test-name-vs-asserts triage (F-K8 seed)

Enumerated every `#[test]`/`#[tokio::test]` fn in `crates/` matching
`(real|dark_factory|round_?trip|end_to_end|e2e|push|fetch)` (case-insensitive). ~60 matches.
Definition of "real push/fetch" used: (i) `Command::new("git")` push/fetch/clone subprocess,
OR (ii) `Command::cargo_bin("git-remote-reposix")` fed an `export`/protocol stream (that IS
the push path git drives), OR (iii) `#[ignore]` + credential-gated real backend.

### C.1 HONEST — genuine helper/git-driving integration tests (do NOT RAISE)

| file:line | test fn | body |
|---|---|---|
| crates/reposix-remote/tests/push_conflict.rs:110 | stale_base_push_emits_fetch_first_and_writes_no_rest | helper `export` vs wiremock; asserts `fetch first` + zero writes (`.expect(0)`) |
| crates/reposix-remote/tests/push_conflict.rs:205 | clean_push_emits_ok_and_mutates_backend | helper `export`; `ok refs/heads/main` + PATCH `.expect(1)` |
| crates/reposix-remote/tests/protocol.rs:106 | crlf_blob_body_round_trips_byte_for_byte | helper push; POST body byte-inspection (`:170`) |
| crates/reposix-remote/tests/mirror_refs.rs:247 | vanilla_fetch_brings_mirror_refs | helper push then real `git clone --mirror` (`:277`) + `for-each-ref` (`:293`) |
| crates/reposix-remote/tests/mirror_refs.rs:432 | reject_hint_first_push_omits_synced_at_line | drive_helper_export; stderr shape asserts |
| crates/reposix-remote/tests/bus_precheck_a.rs:95 | bus_precheck_a_emits_fetch_first_on_drift | real `git init/commit/push -f` fixture (`:57-72`); PRECHECK A real `git ls-remote` |
| crates/reposix-remote/tests/bus_precheck_b.rs:99,203 | bus_precheck_b_{emits_fetch_first_on_sot_drift, passes_when_sot_stable} | cache sync + wiremock drift + `file://` mirror |
| crates/reposix-remote/tests/bus_write_sot_fail.rs:163 | bus_write_sot_mid_stream_fail_no_mirror_push_no_lag_audit | helper bus export; mirror `rev-parse` baseline assert |
| crates/reposix-remote/tests/bus_write_post_precheck_409.rs:159 | bus_write_post_precheck_conflict_409_no_mirror_push | helper bus export; 409 path |
| crates/reposix-remote/tests/bus_write_mirror_egress.rs:37 | bus_push_to_non_allowlisted_mirror_is_denied_before_egress | real git remote fixture; denial-before-egress assert — negative security test, name exact |
| crates/reposix-swarm/tests/mini_e2e.rs:100 | swarm_mini_e2e_sim_5_clients_1_5s | real sim + 5 clients + real audit rows |

Also HONEST: the Confluence wiremock REST suites (`crates/reposix-confluence/tests/roundtrip.rs:72` create-then-get round-trip; `client.rs:1005,1340,1688`), `crates/reposix-github/src/lib.rs:1314` (`…real_issues…` = issues-vs-PRs domain term, mock-tested), and the entire pure-unit group where the keyword names a codec/serde/plan structure, not a git op: `record.rs:244,266,297,372,393,424`; `pktline.rs:126-162`; `diff.rs:272` (unit of the planner — see § E for its fixture hazard, a separate axis); `stateless_connect.rs:534,543`; `bus_url.rs:167`; `adf.rs:654`; `hash.rs:225`; `bind_validation.rs:129`; `sync_tag.rs:224`; `gc.rs:372`; `mirror_refs.rs:307`; `token_cost.rs:29`; `last_fetched_at.rs:29,49`; `audit.rs:652-825` (`log_helper_push_*`/`log_helper_fetch_*` SQLite-row asserts); `seed.rs:200`; `backend/sim.rs:644,678`; `cache_db.rs:160`; `tokens.rs:283,291`; `refresh_integration.rs:141`; `bus_url.rs` / `tests/bus_url.rs:14` (helper `capabilities` invocation, name says URL-parse round-trip).

### C.2 Expected-RAISE baseline for the F-K8 gate

Names promise real/push/fetch/dark-factory; bodies neither invoke a git push/fetch subprocess nor hit a non-127.0.0.1 host nor carry `#[ignore]` real-backend gating:

| # | file:line | test fn | body actually does | verdict |
|---|---|---|---|---|
| 1 | crates/reposix-cli/tests/agent_flow.rs:103 | dark_factory_sim_happy_path | `#[ignore = "spawns reposix-sim child"]` (sim-child, NOT real-backend); `reposix init` then asserts **only git config** (partialClone/promisor/filter/URL prefix); comment `:117-119` admits the trailing fetch fails and that's tolerated. No push, no fetch. | **DISHONEST** — "happy_path" implies the flow; body is config-only. |
| 2 | crates/reposix-cli/tests/doctor.rs:194 | doctor_blob_limit_zero_warns_on_real_remote | `reposix doctor` on a config whose remote URL is a github **string**; "real remote" = URL-classification, no network, not `#[ignore]`. | **DISHONEST** (name's "real_remote" over-promises). |
| 3 | crates/reposix-cli/tests/agent_flow.rs:174 | dark_factory_blob_limit_teaching_string_present | `fs::read_to_string` of `stateless_connect.rs`; source-string asserts. | **BORDERLINE** — `_teaching_string_present` suffix is honest; RAISEs only because `dark_factory` token matches. |
| 4 | crates/reposix-cli/tests/agent_flow.rs:203 | dark_factory_conflict_teaching_string_present | reads `main.rs`/`write_loop.rs`/`bus_handler.rs` source for strings. | **BORDERLINE** — same rationale. |

Adjacent, exculpated by `#[ignore]`+cred-gate but flagged THIN (RAISE-exempt per F-K8 spec; candidates for P91 deepening):

| file:line | test fn | why thin |
|---|---|---|
| crates/reposix-cli/tests/agent_flow_real.rs:128,146,170 | dark_factory_real_{github,confluence,jira} | body = `run_init_and_assert`: `reposix init <backend>::…` then asserts `git config remote.origin.url` prefix/suffix strings only; module doc `:29-36` says live fetch deferred, helper hardcodes SimBackend. Real creds gate a config-string assertion. |
| crates/reposix-cli/tests/attach.rs:548 | attach_marks_mirror_lag_for_next_fetch | real attach subprocess vs sim, real cache asserts; "for next fetch" is state-setup — no fetch runs. `#[ignore]` is sim-child. BORDERLINE. |

Gate-design note: the baseline shows the RAISE regex needs an allowlist channel from day one — the ~40 HONEST pure-unit `*_round_trip*`/`*push*` serde/codec tests (§ C.1) would false-positive on a naive name grep. Two viable shapes: (a) only scan `crates/**/tests/*.rs` integration tests + `#[ignore]`-check, treating in-module `#[cfg(test)]` units as exempt-by-location; (b) per-line `// test-name-honesty: ok — <reason>` marker mirroring `banned-words: ok` precedent. Option (a) misses in-module dishonest names; option (b) matches the framework's existing convention. The 4-row baseline above should be the gate's committed expected-RAISE fixture either way.

### C.3 Shell gates whose names imply push/fetch/e2e

| gate script:line | name implies | actually does | dishonest? |
|---|---|---|---|
| quality/gates/agent-ux/dark-factory/sim.sh:38-61 | dark-factory round-trip (clone→edit→push) | `reposix init` (`:38`), git-config asserts (`:43-48`), **source greps** for teaching strings (`:53-61`). Never pushes/fetches/checks out. | **YES** — QL-001 FINDING-B confirmed verbatim. |
| quality/gates/agent-ux/dark-factory/dvcs-third-arm.sh:118-183 | header `:6-7` "…bus-push **end-to-end**" | real attach + config + sqlite audit asserts; `:20-23` self-discloses push NOT exercised; wire path = test-file-existence grep (`:179-183`). | **PARTIAL** — honest by disclosure, header over-reaches. Rename/soften header. |
| quality/gates/agent-ux/real-git-push-e2e.sh:109-155 | real git push e2e | **genuinely does it**: init → checkout → commit → real `git push` (`:127`) → sim audit_events count asserts (`:131-140`) + no-op-push assert (`:144-155`). | NO — the honest counterpart (currently WAIVED on QL-001; exit 75 on git<2.34). |

---

## D. Waiver cliff triage

25 waiver blocks found repo-wide: 19 in unified catalogs + 6 per-row in doc-alignment.json.
Buckets: 5 **EXPIRED** (2026-05-12), 12 expiring **2026-07-26**, 7 at 2026-07-31, 1 at 2026-08-08.

| id | catalog | until | classification | rationale |
|---|---|---|---|---|
| docs-repro/example-01-shell-loop | docs-reproducible | **2026-05-12 EXPIRED** | renewable — still-broken | `container-rehearse.sh:109-115` never spawns sim; `examples/01-shell-loop/run.sh:19-21` aborts "sim not reachable". tracked_in "P59 Wave F" never landed. |
| docs-repro/example-02-python-agent | docs-reproducible | EXPIRED | renewable — still-broken | same root cause (`run.py:67-70`). |
| docs-repro/example-04-conflict-resolve | docs-reproducible | EXPIRED | renewable — still-broken | same. |
| docs-repro/example-05-blob-limit-recovery | docs-reproducible | EXPIRED | renewable — still-broken | same. |
| docs-repro/tutorial-replay | docs-reproducible | EXPIRED | renewable — still-broken (×2 causes) | self-spawns sim (`tutorial-replay.sh:47-50`) but cold cargo build >5min container budget; push step additionally QL-001-blocked. |
| code/cargo-test-pass | code.json | 2026-07-26 | renewable | reason still true: local nextest violates memory budget; CI `ci.yml` test job is canonical. Alternative: retire the local row and name CI as home. |
| cross-platform/windows-2022-rehearsal | cross-platform | 2026-07-26 | renewable | verifier "intentionally does not exist yet"; `quality/gates/cross-platform/` confirmed empty. |
| cross-platform/macos-14-rehearsal | cross-platform | 2026-07-26 | renewable | same. |
| perf/latency-bench | perf-targets | 2026-07-26 | renewable | bench runs (sim); headline cross-check stub deferred to "v0.12.1" which never shipped it. |
| perf/token-economy-bench | perf-targets | 2026-07-26 | renewable | same deferral. |
| perf/headline-numbers-cross-check | perf-targets | 2026-07-26 | renewable — verifier missing outright | `quality/gates/perf/headline-numbers-cross-check.py` absent. |
| release/cargo-binstall-resolves | release-assets | 2026-07-26 | **landable in P90 <1h** | waiver self-describes a ~10-LOC `pkg-url` metadata alignment in `crates/reposix-cli/Cargo.toml` vs `release.yml` archive shape; flips PARTIAL→PASS. Also unblocks `scripts/webhook-latency-measure.sh` (webhook-latency real measurement). |
| security/allowlist-enforcement | security-gates | 2026-07-26 | renewable, **P0 — prioritize**; possible <1h wrapper | verifier script absent. Allowlist code exists in `reposix-core/src/http.rs` but no test named `http_allowlist` found — a thin cargo-test wrapper may be landable if a real test exists/is written; otherwise route to P92 (security e2e). |
| security/audit-immutability | security-gates | 2026-07-26 | renewable, P0 | verifier + named test both absent. |
| subjective/cold-reader-hero-clarity | subjective-rubrics | 2026-07-26 | renewable — **NOT mooted by dispatch.sh** | see below. |
| subjective/install-positioning | subjective-rubrics | 2026-07-26 | renewable — NOT mooted | see below. |
| subjective/headline-numbers-sanity | subjective-rubrics | 2026-07-26 | renewable — NOT mooted | see below. |
| agent-ux/real-git-push-e2e | agent-ux | 2026-07-31 | route to P91 (QL-001 fix); do NOT renew | verifier is genuine (§ C.3); bug confirmed live at `diff.rs:106`. Waiver is the backstop if P91 slips (per D90-01: deliberately not renewed). |
| structure/file-size-limits | freshness-invariants:634-675 | 2026-08-08 | renewable | `--warn-only` args live (`:652`); 10 enumerated violations undrained (6+3 research bundles + AGENTS.md symlink); symlink-exclusion verifier fix is a separate XS. |
| 5× doc-alignment MISSING_TEST rows | doc-alignment | 2026-07-31 | **landable in P90** (per D90-07 they're in P90's cargo wave) | § F. |
| docs/index/git-checkout-branch-command | doc-alignment | 2026-07-31 | route to P91 — STAYS WAIVED | § F; QL-001-blocked by definition. |

**The 3 subjective waivers are NOT mooted by dispatch.sh wiring.** The rows were always
dispatch-wired; the waiver covers a different gap — *runner-sweep preservation of ratified
verdicts*. The waiver reason (identical ×3): the runner subprocess lacks Task access, so
re-execution would overwrite the ratified Path-B artifact (8/9/9 CLEAR scores in
`quality/reports/verifications/subjective/*.json`) with a stub. Confirmed real:
`dispatch_inline_subagent.sh:42-52` persists `--score 0 --verdict CONFUSING` + exit 1 when
`claude` CLI is absent, clobbering the artifact; even with `claude` present a sweep re-grades
and may diverge. The fix is a dispatch-and-preserve invariant in the runner (tracked
MIGRATE-03), not wiring. Classification: renewable with honest tracked_in.

**Mass-renewal warning:** everything tracked to "v0.12.1 MIGRATE-03 / SEC-0x / CROSS-0x"
points at a milestone that never shipped those gates; the 2026-07-26 cliff re-expires 12 rows
at once. P90's renewal commits must swap tracked_in to live phase targets (P92/P95/P97) or
the cliff repeats in 3 weeks.

Environment note: this box has git 2.25.1 < 2.34, so `real-git-push-e2e` and tutorial-replay
NOT-VERIFY here on environment grounds regardless of the code bug.

---

## E. Magic-fixture hazard sweep

Precedent confirmed and **worse than filed**: the path-shape schism is *triple*, not double.

| site | shape produced | file:line |
|---|---|---|
| cache tree builder (real read path) | `issues/1.md` (unpadded, prefixed) | crates/reposix-cache/src/builder.rs:90 (`format!("{}.md", …)`) + `:135` (`issues` subtree) |
| `reposix refresh` write path | `issues/00000000001.md` (11-padded) | crates/reposix-cli/src/refresh.rs:120 |
| push diff planner + fast-import emit | `0001.md` (4-padded, NO prefix) | crates/reposix-remote/src/diff.rs:106 + fast_import.rs:63 |

`issue_id_from_path` (`diff.rs:74-77`) parses only bare-integer stems, so on a real
`issues/1.md` tree every prior fails to match (spurious Creates) AND the conflict precheck
**silently skips** every real path (`precheck.rs:151-152` `continue`) — push-time conflict
detection is bypassed on real trees, not just the diff plan.

### HIGH — test green depends on the bug

| file:line | fixture | why | rating |
|---|---|---|---|
| crates/reposix-remote/src/diff.rs:283 | `unchanged_push_emits_no_patches` inserts `format!("{:04}.md", …)` | hand-builds the planner's own bug shape; asserts 0 actions — can never observe BUG-1 | HIGH |
| crates/reposix-remote/src/diff.rs:310 | `extra_trailing_newline_is_a_noop` inserts `"0001.md"` | same masking | HIGH |
| crates/reposix-remote/src/diff.rs:233-238 | `five_deletes_passes_cap` / `six_deletes_*` | SG-02 bulk-delete cap validated exclusively in the buggy key space | HIGH |
| crates/reposix-remote/tests/push_conflict.rs:154 | `one_file_export("0002.md", …)` | the ARCH-08 stale-base regression test only fires the precheck because the bare shape parses; with real `issues/2.md` the stale write would be **accepted** — the regression test cannot catch the real-world bypass | HIGH |
| crates/reposix-remote/src/precheck.rs:151 | `issue_id_from_path` gate + `:152` `continue` | whole class of "stale push slipped through" bugs untestable by construction | HIGH |
| crates/reposix-remote/tests/bus_write_happy.rs:251, bus_write_sot_fail.rs:244, bus_write_mirror_fail.rs:213, bus_write_post_precheck_409.rs:227, bus_write_audit_completeness.rs:214 | `("0001.md", blob1)…` fixtures | the ENTIRE bus-write fan-out suite (incl. rows graded honest in § A.1) rides the bug shape; fan-out/audit/lag asserts true only in that shape | HIGH |
| crates/reposix-remote/tests/protocol.rs:85 | `M 100644 :1 0001.md` | core `export` protocol fixture, same masking | HIGH |
| crates/reposix-remote/src/fast_import.rs:63 (+ main.rs:338 wiring) | emit side writes `{:04}.md` bare | the deprecated `import` transport genuinely emits bare paths — the source of why bare fixtures "look real" to test authors; the documented primary path (stateless-connect/cache) does not | HIGH |

### MED

| file:line | fixture | why | rating |
|---|---|---|---|
| crates/reposix-confluence/src/client.rs:1633-1656 | `update_issue_sends_put_with_version`: mock matches method+path only; asserts echoed `version==43` | name + comment (`:1649`) promise a request-body version assert; nothing asserts the outbound PUT body — wrong-version sends would pass | MED |
| crates/reposix-remote/tests/perf_l1.rs:239,303,311 | bare `0001.md` in parsed.tree | the L1 call-count economy test measures the wrong (all-Create) plan if shape breaks matching, while count asserts still pass | MED |
| cross-cutting | builder.rs:90 vs refresh.rs:120 vs diff.rs:106/fast_import.rs:63 vs attach tests' `issues/0001.md` vs path.rs:138-140 doc ("11-digit padded") | **no canonical `record_path(id)` helper exists**; four incompatible conventions each re-derived inline; `path.rs:140` documents a convention only refresh honors. Structural root cause; hazards will recur without it | MED |

### LOW

`seed.rs:143,147,195` + `routes/issues.rs:539,822` (`count==6` vs seed.json — fails loudly);
`backend/sim.rs:1058` (`version==1` echo-parse); pagination counts
(`reposix-github/tests/contract.rs:398`, `reposix-jira/src/client.rs:691`,
`reposix-confluence/src/client.rs:2385`); `reposix-cli/tests/attach.rs:353,393,495,611,756`
(`issues/0001.md` third shape — cosmetic because attach reconciles by frontmatter `id`, not path).

**Fix-first recommendation for P91:** introduce canonical `record_path(id) -> "issues/<id>.md"`
in reposix-core, route the 4 sites through it, re-key the diff/bus_write/push_conflict/protocol
fixtures to it — those tests then go correctly RED until BUG-1 is fixed. Today there is **no
push-side test using the real `issues/<id>.md` shape**; the only real-shape coverage is
cache-read-side (`delta_sync.rs:261`, `tree_contains_all_issues.rs:51`, `gix_api_smoke.rs:46`)
and never reaches `diff::plan`/`precheck`.

---

## F. MISSING_TEST doc-alignment rows

6 rows at `last_verdict: MISSING_TEST`, all waived until 2026-07-31. Ground truth: the CLI
has **15 subcommands** (clap `Cmd` enum, `crates/reposix-cli/src/main.rs:39-343`): sim, init,
attach, list, refresh, spaces, sync, doctor, history, log, at, gc, tokens, cost, version.

**Stale-claim finding:** row 1's `claim` text enumerates only 13 names — it **omits `attach`
and `sync`**, both shipped (main.rs:104,181) and documented (cli.md:14,19). Rebind must
update the claim text or the new test re-encodes under-coverage.

| row id | claim | doc cite | REAL test must assert | proposed test fn + location | size |
|---|---|---|---|---|---|
| docs/reference/cli.md/subcommands_exist | all subcommands documented in help | cli.md:5-29 | `reposix --help` stdout contains **all 15** names (loop over full list) | extend `help_lists_all_subcommands`, crates/reposix-cli/tests/cli.rs:4-23 — replace the 4-name array at cli.rs:13 | XS |
| docs/reference/cli.md/env_vars | 7 env vars documented | cli.md:334-343 | each documented var is **consumed** by code (env::var / EnvFilter site exists per name; RUST_LOG via `EnvFilter::try_from_default_env`, main.rs:352) | new `env_vars_are_consumed_by_binary`, crates/reposix-cli/tests/cli.rs | S |
| docs/reference/cli.md/exit_codes | exit codes 0/1/2 documented | cli.md:345-351 | drive CLI to all three observed codes: `version`→0, expected-failure→1, malformed init spec→2; assert `status.code()` | new `exit_codes_match_documented_contract`, crates/reposix-cli/tests/cli.rs (near cli.rs:43) | S |
| docs/reference/cli.md/spaces_confluence_only | spaces is Confluence-only | cli.md:315-323 | non-Confluence backend **rejected**: `spaces --backend github` exits non-zero, stderr names confluence (assert pre-egress error, no network) | new `spaces_rejects_non_confluence_backend`, crates/reposix-cli/tests/cli.rs | S |
| docs/decisions/009-stability-commitment/exit-codes-locked | exit codes locked under semver | exit-codes.md:20-157 | pin exact code sets: `reposix` ∈ {0,1(,2)}, `git-remote-reposix` ∈ {0,1,2}; table of (argv → expected code) asserted exactly | new `exit_codes_locked_reposix_and_helper` — cli.rs for the CLI arm + crates/reposix-remote/tests/ for the helper arm | S |

Previously-cited false-BOUND tests confirmed: `help_lists_all_subcommands` (cli.rs:4, asserts
4/15), `mount_subcommand_is_removed` (cli.rs:43, nothing about exit-code semantics),
`gc_help_renders` (crates/reposix-cli/tests/gc.rs:33, only `--strategy` help rendering).

**Row 6 — `docs/index/git-checkout-branch-command` (docs/index.md:129): STAYS WAIVED,
until 2026-07-31.** Unlike the 5 above (testable today, binding was throwaway), its behavior
is genuinely broken pending QL-001 (BUG-1/BUG-3); a real test = tutorial-replay step 4, which
cannot pass until P91 lands the round-trip fix. Consistent with D90-01/D90-07.

---

## G. Gate-authoring precedent for `test-name-vs-asserts.sh`

**Script shape** (from `quality/gates/structure/banned-production-tokens.sh:37-64` +
`deferral-pointer-linter.sh:40-115`): `set -euo pipefail`; `SCRIPT_DIR`→`REPO_ROOT` cd
preamble; single grep/find pass; per-violation structured stderr (`✖ path:line: content`)
+ `owner_hint:` + `see: quality/catalogs/<file> row <id>` teaching block; exit 1 on violation;
one-line `PASS: …` summary on success. Header comments carry the regex-scope trade-off
narrative in-file (banned-production-tokens.sh:7-30 is the model — F-K8's name-token
scope/allowlist rationale should live in the new script's header the same way).

**Allowlist precedent:** per-line marker `// banned-words: ok` (banned-production-tokens.sh:42,50).
The new gate should mint a sibling marker (e.g. `// test-name-honesty: ok — <reason>`) rather
than a separate allowlist file — and per the deferral-linter's no-PNN lesson
(deferral-pointer-linter.sh:8-15,82-91), a bare marker with no reason should itself RAISE.

**Catalog home:** ROADMAP names the script path under `quality/gates/agent-ux/`, so the row
belongs in **`quality/catalogs/agent-ux.json`** (wrapper `"dimension": "agent-ux"`), NOT a new
catalog — mirroring how the two structure gates live in `freshness-invariants.json` under
wrapper `"dimension": "structure"` with no `structure.json` (rows at
freshness-invariants.json:677-745). Note the runner keys scope off row `cadences` + wrapper
discovery (`run.py:66-80` `discover_catalogs` globs `quality/catalogs/*.json`;
`is_in_scope` at run.py:133-141 requires the requested cadence ∈ row `cadences`), so
catalog-file choice is organizational, not functional.

**Row fields to mirror** (from the banned-production-tokens row, freshness-invariants.json:677-710):
`kind: mechanical`; `cadences: ["pre-push"]` (per P90 SC-1; pre-commit optional if fast enough);
`expected.asserts` naming the exact regex + exemption classes; `claim_vs_assertion_audit`
≥50 chars (mandatory — post-cutoff row; plus write-once `minted_at` per D90-03);
`blast_radius: P1`; `timeout_s: 10-15`; `artifact: quality/reports/verifications/agent-ux/test-name-vs-asserts.json`;
status minted `NOT-VERIFIED` catalog-first (PROTOCOL.md:94).

**Exit codes** (PROTOCOL.md:189-200, `_realbackend.map_exit_code_to_status`): 0→PASS,
2→PARTIAL, 75→NOT-VERIFIED, else→FAIL. For a triage gate, a useful convention: exit 0 when
RAISE set == committed baseline (`quality/reports/raise-list-p90.md` acts as the expected
snapshot), exit 1 on NEW un-baselined RAISEs. Exit 75 is not needed (no env gating).

**Latency budget:** pre-push total is <60s (CLAUDE.md cadence table). A single
`rg -n --type rust` pass over `crates/` (+ optional second pass for `#[ignore]` context lines)
with pure-bash post-filtering is well under 1s on this repo (~60 name-matches total, § C);
no cargo, no network. Comfortably inside budget even stacked on the existing pre-push suite.

**Baseline contract:** seed the gate's expected-RAISE list from § C.2 (4 entries). The ~40
HONEST unit-test matches in § C.1 define the false-positive set the gate's scoping/allowlist
must clear before it can run at pre-push without noise.

---

## H. Noticing (charter rule 2)

1. **SURPRISES-INTAKE LOW entry already resolved, not closed.** The dvcs-cold-reader
   `NOT_VERIFIED` typo (intake 2026-07-03 21:35) was fixed by commit `c0d5459` but the intake
   entry still reads OPEN. Close it RESOLVED-c0d5459 — XS, one line. (Also: the intake text
   at SURPRISES-INTAKE.md:240 still describes the typo as live; P96's drain should cite the SHA.)
2. **`subjective/dvcs-cold-reader` is the real decorative-kind row** — `kind: subagent-graded`
   with no dispatcher case (`dispatch.sh:58-76` falls to the Path-B stub for it). The F-K4c
   migration brief fixates on dvcs-third-arm; this row is the one needing WIRE-DISPATCH.
3. **Two dangling verifier paths in security-gates.json** — `allowlist-enforcement.sh` and
   `audit-immutability.sh` don't exist under `quality/gates/security/` (only
   `connector-audit-wired.sh` does). Waivers currently hide the dangle; the RBF-FW-07
   missing-verifier→NOT-VERIFIED demotion (D90-04) will surface both the moment the waivers
   lapse on 2026-07-26. These are P0 threat-model rows — the collision of waiver-expiry and
   missing-script deserves an explicit P92 line item, not an accidental discovery.
4. **`update_issue_sends_put_with_version` lies in name and comment** —
   `reposix-confluence/src/client.rs:1633-1656` asserts the echoed response version, never the
   outbound PUT body; comment `:1649` claims otherwise. Not in F-K8's token set (no
   push/fetch/real in the name) — worth a wiremock `body_json`-matcher fix in P92's Confluence wave.
5. **`fast_import.rs:156-157` comment lies** ("consume one [LF] if present" — it eats a full
   line unconditionally; QL-001 BUG-3's root). The comment should die with the P91 fix.
6. **`dvcs-third-arm.sh` header over-claims** ("bus-push end-to-end", `:6-7`) while `:20-23`
   admits push is not exercised — internally inconsistent; fix the header when flipping the
   row's kind (same commit, XS).
7. **CLAUDE.md quality-dimension table drift risk:** the agent-ux row description in the
   dimension table says "dark-factory regression (sim arm + DVCS third arm)" — after P90 the
   dimension also owns test-name-vs-asserts; update the cell in the same PR per the
   CLAUDE.md-stays-current rule.
8. **`path.rs:138-140` documents an 11-digit filename convention only `refresh.rs` honors**
   (§ E MED) — stale doc claim in code; should be rewritten when `record_path()` lands.
9. **Waiver tracked_in decay pattern:** 12 of 19 unified-catalog waivers track to
   "v0.12.1 MIGRATE-03/SEC/CROSS" — a milestone label, not a live phase. Renewals that keep
   dead tracked_in pointers are the deferral-loop C7 shape the framework is supposed to
   prevent; P90's renewal commits should be required to name a phase that exists in ROADMAP.
10. **agent_flow_real.rs bodies are cred-gated config asserts** (module doc admits it,
    `:29-36`) — P91's `attach_real_*`/`sync_real_*` family should replace, not extend, this
    pattern; F-K8 exempts them today only via the `#[ignore]` escape hatch.
