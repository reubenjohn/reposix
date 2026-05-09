# v0.13.0 Extension Remediation Plan — HIGH-finding decomposition + phase shape

**Author:** remediation-decomposition synthesis subagent (zero prior context)
**Date:** 2026-05-08
**Inputs read:**
- `01-dark-factory-may02/SUMMARY.md` (37 frictions / 16 HIGH on real Confluence + sim + GH mirror)
- `02-phase-audits-may08/{phase-audit-p78..p88,vision-audit}.md` (~51 HIGH on a 12-subagent codebase audit)
- `CLAUDE.md` (project Operating Principles, Quality Gates taxonomy, +2 reservation)
- `quality/PROTOCOL.md` (catalog-first contract, verifier subagent dispatch)
- `.planning/milestones/v0.13.0-phases/ROADMAP.md` (canonical phase shape)

## 1 — HIGH finding inventory (~43 explicit + ~8 cluster-folded, classified)

**~51 HIGH findings across the 12 May 8 audit files. ~43 are explicitly inventoried below as `H-` rows; the remaining ~8 fold into cluster covers (e.g. `H-E1` covers Cluster E's compound init UX issues; `H-F1` covers Cluster F).** A future executor should expect cluster-level coverage, not bijective `H-N → finding-N` mapping.

Each row: `<audit-file> F<n> — paraphrase` | **class** | **effort** | **deps**.

Effort key: **XS** = <1h, **S** = 1–4h, **M** = 4–16h, **L** = >16h (split candidate).
Class key: **C** = code-fix, **F** = framework-fix (catalog/verifier/cadence/brief), **D** = doc-fix.

### A — `reposix attach` real-backend wiring (CLUSTER A, P79 silent scope-cut)

| ID | Finding | Class | Effort | Depends on |
|---|---|---|---|---|
| H-A1 | `vision-audit F1` — `reposix attach` is sim-only; vision litmus test cannot run on any real backend | C | M | F-fix H-K1 (cadence) |
| H-A2 | `vision-audit F2` / `p79 F1` — production error literal leaks `P79-02 scaffold` / `P79-03` to user stderr | C | XS | — |
| H-A3 | `p79 F2` — 79-03 PLAN silently dropped real-backend wiring 79-02 promised | F+D | XS | retroactive intake fix |
| H-A4 | `p79 F3` — verifier mistook "tests pass" for "feature ships"; DVCS-ATTACH-01..04 graded sim-only | F | S | H-K1 |
| H-A5 | `p79 F4` — `agent_flow_real.rs` covers `init` but NOT `attach`; name promises more than asserts | C+F | S | H-A1 |
| H-A6 | `p81 F1` — `reposix sync --reconcile` rejects every real backend with same scaffold-leak pattern | C | S | H-A1 (same dispatch shape) |
| H-A7 | `p86 F13` — third-arm harness creates EMPTY work tree + empty bare mirror; reconciliation walk asserts shape not content | C | S | H-A1 |

### B — Push/fetch architectural cornerstone broken (CLUSTERS B, C)

| ID | Finding | Class | Effort | Depends on |
|---|---|---|---|---|
| H-B1 | `vision-audit F4` / dark-factory CLUSTER B — `git pull --rebase` recovery fails on sim; helper fetch mints fresh root commit, breaks ancestry | C | M | — |
| H-B2 | `vision-audit F3` / dark-factory CLUSTER C — every `git push` writes ZERO `helper_push_*` audit rows; cache.db never created on production helper path; OP-3 violated | C | M | — |
| H-B3 | `p83 F3` — `audit-completeness` row claims dual-table; production helper instantiates `ConfluenceBackend::new_with_base_url(...)` WITHOUT `.with_audit(...)` so `audit_events` is unwritten on every real-backend push | C | S | H-B2 |
| H-B4 | `p86 F5` — wire-path delegation anchor `bus_write_happy.rs` uses wiremock + SimBackend, never a real backend; "dual-table" assertion checks only `audit_events_cache` | F+C | S | H-B2, H-B3 |

### C — Bus push collides with documented mirror tree (CLUSTER D)

| ID | Finding | Class | Effort | Depends on |
|---|---|---|---|---|
| H-C1 | dark-factory CLUSTER D / `vision-audit F5` / `p83 F2` / `p84 F8` / `p85 F1` — helper export validator (`diff.rs:99-123`) rejects `.github/workflows/*.yml`, `README.md`, `.reposix/*` — exactly files the docs tell users to commit | C | M | architectural decision (skip-list vs path-prefix scope) |
| H-C2 | `p82 F4` — STEP 0 (`resolve_mirror_remote_name`) requires three-step undocumented prereq chain (`git remote add mirror`, `git fetch mirror`); only error path tested, success path uses synthetic fixture | C+D | S | — |
| H-C3 | `p84 F1` — workflow YAML self-clobbers itself on first successful run; force-push from `/tmp/sot` deletes `.github/workflows/...` from `main`, severing workflow from itself | C+D | M | architectural (separate branch / re-add / sibling repo) |

### D — `reposix sync --reconcile` / cache-desync recovery dark on real backends

(Subset of A — H-A6 covers this; recovery flow doc fix lives in K-cluster.)

### E — Init UX broken on first contact (CLUSTER E)

| ID | Finding | Class | Effort | Depends on |
|---|---|---|---|---|
| (no HIGH; tracked as MED in dark-factory SUMMARY but **3 compound issues** named load-bearing because they break the documented quickstart on EVERY first run) | F1+F2+F5 from CLUSTER E | C+D | S | — |

(Promoted from MED by orchestrator: dark-factory SUMMARY says CLUSTER E is "init UX broken on first contact"; we treat it as `H-E*` because three independent issues compound on the documented copy-paste flow. Item ID = **H-E1**, effort S, code+doc fix.)

### F — Tutorial / README / testing-targets stale (CLUSTERS F, H)

| ID | Finding | Class | Effort | Depends on |
|---|---|---|---|---|
| (no HIGH; CLUSTER F = MED-MED-MED but compound effect kills first-contact flow) | tracked as **H-F1** to align with dark-factory recommended P92 | D | S | — |
| dark-factory CLUSTER H — `testing-targets.md` cites "TokenWorld"; tenant only has "REPOSIX" → real-backend test config silently mis-targets | D | XS | — |

### G — Quality framework structurally exempts real-backend flows (CLUSTER G — META)

| ID | Finding | Class | Effort | Depends on |
|---|---|---|---|---|
| H-G1 | dark-factory CLUSTER G / `vision-audit F7` — `dark_factory_real_confluence` test stops at "URL has the right shape"; never `git fetch` / `git push` against real backend | F+C | S | H-K1 |
| H-G2 | `vision-audit F9` — milestone-close verdict's 8 probes do NOT include any real-backend flow probe; OP-1 named gate but never operationalized | F | S | H-K1 |
| H-G3 | `vision-audit F12` — P78–P88 = 11 vertical slices each pass own verifier; no horizontal probe ever validates vision composition end-to-end | F | M | H-K1, H-K3 |
| H-G4 | `p86 F1` / `p86 F2` / `p86 F10` / `p86 F12` — P86 ROADMAP SC #3 (end-to-end push), SC #5 (TokenWorld coverage) silently dropped via "Rule 3 eager-resolution pivot"; "pure git" public claim never qualified | F+D | S | H-G1, H-K2 |
| H-G5 | `p86 F3` / `p86 F4` — ROADMAP SC #1 prompt ("install reposix, attach, fix the bug, push your fix back") not exercised; `dark_factory_real_*` tests confirmed to never `git fetch`/`git push` | F+C | S | H-G1, H-A5 |

### H — Quality framework verifier-honesty gaps

| ID | Finding | Class | Effort | Depends on |
|---|---|---|---|---|
| H-H1 | `vision-audit F6` — P85 cold-reader pass (DVCS-DOCS-04) NOT_VERIFIED at milestone close; gate meant to catch CLUSTER E + F never blocked the GREEN verdict | F | XS | H-K3 |
| H-H2 | `vision-audit F8` — SURPRISES-INTAKE captured 5 polish items but ZERO of 16 dark-factory HIGH frictions; "found-it-but-skipped-it" no-op for items no phase tried | F | S | H-G1 (no E2E test ⇒ nothing to file) |
| H-H3 | `p87 F1` — Honesty spot-check sample structurally excluded the two phases (P79, P86) where biggest scope cuts occurred | F | XS | meta-rule for absorption phase |
| H-H4 | `p87 F2` — Honesty spot-check authored by milestone orchestrator, not independent verifier; OP-7 violated for meta-grading | F | XS | meta-rule |
| H-H5 | `p87 F3` — Honesty spot-check rubric grades "did the phase use the framework?" instead of "did the phase deliver the architecturally-promised outcome?" | F | S | rubric extension |
| H-H6 | `p87 F8` — P85 graded GREEN by P87 with "no skipped findings"; cold-reader walkthrough never done; testing-targets.md TokenWorld-vs-REPOSIX gap missed | F | XS | H-H5 |

### I — Catalog/verifier shape misalignment (test name promises more than asserts)

| ID | Finding | Class | Effort | Depends on |
|---|---|---|---|---|
| H-I1 | `p78 F1` — `no-pre-pivot-doc-stubs.sh` does substring grep; catalog row promises structural assertion (parse `redirect_maps`, resolve target) | F | S | — |
| H-I2 | `p78 F2` — `repo-org-audit-artifact-present.sh` is single `grep -qE` for vocabulary; catalog row promises gap-mapping audit (every numbered gap → closure path) | F | S | — |
| H-I3 | `p80 F1` — workflow template falsely asserts `reposix init` populates `refs/mirrors/*`; init code path has no `write_mirror_*` call → first cron tick of fresh mirror silently breaks observability | C+D | S | — |
| H-I4 | `p80 F2` — `mirror-refs-readable-by-vanilla-fetch` test uses `clone --mirror` of local path; never traverses helper, never speaks `stateless-connect` (the thing the catalog claim says is being proved) | F+C | S | — |
| H-I5 | `p80 F3` — Zero real-backend coverage for any DVCS-MIRROR-REFS row; transport-layer claim verified only against wiremock | F | S | H-K1 |
| H-I6 | `p81 F2` — `perf/handle-export-list-call-count` tests NO-OP push (`no_op_tree_export`); skips `refresh_for_mirror_head` which still calls `list_records`. Headline perf claim verified only on path that doesn't exercise the claim | C+F | S | — |
| H-I7 | `p82 F1` — DVCS-BUS-FETCH-01 verifies absence of `stateless-connect` capability; catalog claim is "fetch falls through to single-backend path" — actual fall-through is to deprecated v0.8 `import` slated for Phase 36 removal | C+D | S | — |
| H-I8 | `p82 F3` — DVCS-BUS-PRECHECK-01 / -02 / -FETCH-01 have ZERO real-backend coverage; everything wiremock or `tempfile + git init --bare` | F | S | H-K1 |
| H-I9 | `p82 F10` — CLAUDE.md "Bus URL form" para says fetch falls through to "single-backend path"; actually routes through deprecated `import` | D | XS | clarify after H-I7 fix |
| H-I10 | `p83 F1` — Fault-injection coverage 100% wiremock + `file://` mirror; zero real-backend exercise (network timeout, GH rate limit, OAuth revocation, push-protection rules) | F | S | H-K1 |
| H-I11 | `p84 F2` — `webhook-latency-floor` row passes vacuously (`n: 1, p95: 5s`, synthetic-dispatch); `freshness_ttl: null` so placeholder permanent | F | XS | H-K2 |
| H-I12 | `p84 F3` — `cargo binstall reposix-cli` cannot resolve in any release; `pkg-url` mismatch with release-pipeline archive name; row asserts only spelling not resolvability | C+F | S | release-pipeline fix |
| H-I13 | `p85 F3` — Cold-reader rubric criterion 4 ("walk-through is runnable as-written") graded 8/10 yet walk-through is not runnable end-to-end | F | XS | H-H5 |
| H-I14 | `p85 F4` — Bus URL form `reposix::<sot>?mirror=<url>` undocumented in any P85 surface; mermaid shows `git push (bus)` arrows but prose never shows the URL form | D | S | — |
| H-I15 | `p85 F12` — Mental walk-through of Pattern C (vanilla-clone + attach + push) fails on real backends at multiple steps | F+D | S | H-A1, H-C1 |
| H-I16 | `p88 F1` — All 4 P88 catalog rows are presence/structure-only; verify file existence + heading shape only (10 lines of `aaaaa` would PASS the CHANGELOG check) | F | S | — |
| H-I17 | `p88 F2` — Deferral rationale for GOOD-TO-HAVES-01 ("DEFERRED to v0.14.0") empirically false: work shipped ~17 minutes after milestone close in commit `fd2e247` | F+D | XS | retroactive intake |
| H-I18 | `p88 F3` — RETROSPECTIVE distillation captures ZERO of 16 HIGH dark-factory frictions; OP-9 ritual produced clean-looking distillation uninformed by reality | D | S | retroactive (after dark-factory + audits land) |
| H-I19 | `p88 F4` — Milestone-close verdict silently dropped SC5's "TokenWorld arm GREEN" clause; verdict probe #4 ran only sim arm | F | XS | H-K3 |

## 2 — Cross-cutting framework fixes (the load-bearing FIRST work)

These are the framework changes that **must land first** because the code/doc fixes that follow depend on them being trustworthy. Without these, every code fix below ships back into a framework that exempts real-backend flows.

| Framework fix | Description | Effort |
|---|---|---|
| **F-K1** — `cadence: pre-release-real-backend` | New cadence in `quality/PROTOCOL.md` + `quality/runners/run.py` that gates on `REPOSIX_ALLOWED_ORIGINS` + `ATLASSIAN_API_KEY` / `GITHUB_TOKEN` / `JIRA_API_TOKEN`. Rows tagged with this cadence MUST execute against the sanctioned target (TokenWorld Confluence / `reubenjohn/reposix` issues / JIRA `TEST`). Default-skip in CI; required in milestone-close. | M |
| **F-K2** — `kind: shell-subprocess` verifier | New verifier kind that drives `reposix init/attach/sync/push` as actual subprocesses against a real backend with explicit env-control assertions (no `assert_cmd` env injection). Produces a transcript artifact. Replaces "cargo-test-as-verifier" for transport claims. | M |
| **F-K3** — Milestone-close litmus-test probe | Add a ninth probe to milestone-close verdict template that runs the vision document's litmus test verbatim against a real backend before the tag. RED ⇒ milestone does NOT close. Operationalizes OP-1 + ROADMAP SC "real-backend tests gate milestone close". | S |
| **F-K4** — Catalog-row honesty rules | (a) Rows whose `description` includes transport/perf claim MUST carry a `coverage_kind: real-backend` flag → verifier required to exercise real backend OR row carries explicit `WAIVED + until_date` (no PASS-with-comment). (b) `expected.asserts` cross-checked against `asserts_passed` artifact at runner time (see p86 F6). (c) `kind: subagent-graded` rows MUST wire to `dispatch.sh` (no decorative kinds; see p86 F7). | M |
| **F-K5** — Honesty-spot-check meta-rule (absorption phase) | (a) Sample MUST include every phase that closed without filing intake. (b) Spot-check author ≠ milestone orchestrator. (c) Rubric question = "walk one critical example end-to-end mentally — does it work?" not "did the phase use the framework correctly?". (d) Verifier hash-binds spot-check content (not just file existence). | S |
| **F-K6** — Deferral-pointer linter | Pre-push check `grep -rn 'not yet wired in P\d+\|land(s\|ing) (alongside\|in) P\d+\|substrate-gap-deferred' crates/` and cross-reference against the named phase's PLAN files. If named downstream phase doesn't deliver, BLOCK. | S |
| **F-K7** — Banned production-error tokens | Add `P\d+-\d+` regex to `quality/gates/structure/banned-words.sh` (production strings only). Phase IDs leaking into binary output is a process-failure smell. | XS |
| **F-K8** — Dishonest-test triage helper | Cross-cutting `quality/gates/agent-ux/test-name-vs-asserts.sh` rule: if test name contains `real|dark_factory|round_trip|end_to_end|push|fetch` AND test body does not invoke `Command::new("git").arg("push|fetch")` AND does not call against a non-`127.0.0.1` host AND not gated `#[ignore]` real-backend, RAISE. | S |

## 3 — Proposed v0.13.0 extension phase shape

**Constraints applied:**
- Reserve last 2 phases for surprises absorption + good-to-haves polish (CLAUDE.md OP-8 / OP-9).
- Framework fixes land FIRST (P89 + P90) so subsequent code/doc fixes ship into a trustworthy framework.
- Each phase ≤ 5 days. Items above XS+S+M effort are decomposed.
- REQ-IDs use `RBF-` prefix (Real-Backend Frictions, v0.13.0 extension series).

The dark-factory SUMMARY proposed 4 work + 2 reservation (P89–P94). After audit-driven decomposition I recommend **7 work + 2 reservation = 9 phases (P89–P97)** because the framework-fix work is bigger than the original proposal accounted for, and Decision 2 promotes 1 additional code phase from v0.14.0 deferral.

---

### **P89 — Framework fixes: real-backend cadence, shell-subprocess kind, milestone-close litmus probe**

**Goal:** Build the framework infrastructure that makes every other v0.13.0 extension phase's catalog rows trustworthy. Fix-class: **F**.

**REQ-IDs:**
- `RBF-FW-01` — New `cadence: pre-release-real-backend` (per F-K1).
- `RBF-FW-02` — New `kind: shell-subprocess` verifier (per F-K2).
- `RBF-FW-03` — Milestone-close ninth probe runs vision litmus test against real backend (per F-K3).
- `RBF-FW-04` — Banned-production-error-tokens regex (per F-K7).
- `RBF-FW-05` — Deferral-pointer linter (per F-K6).
- `RBF-FW-11` — Structural claim-vs-assertion congruence check: every catalog row's `description` carries a `claim_vs_assertion_audit` paragraph explaining how the verifier's assertion would falsify the description claim if false; runner cross-checks per Decision 3 (the F-K1..F-K8 patches are necessary but not sufficient — STRATEGIC Q3).

**HIGH findings closed by this phase:** H-G2 (operationalize OP-1 milestone-close gate); H-G3 (horizontal probe); H-K1/K2/K3/K6/K7 inputs to subsequent phases.

**Success criteria:**
1. `quality/PROTOCOL.md` documents the new cadence + kind with worked example.
2. `quality/runners/run.py` recognizes `pre-release-real-backend`; default-skips when env not set; requires explicit env to run.
3. Milestone-close verdict template has a 9th probe entry; absent ⇒ verdict graded RED.
4. Pre-push gate runs the deferral-pointer linter; banned-production-error-tokens regex extended.
5. Catalog-first commit mints 5 rows in `quality/catalogs/{agent-ux,framework}.json` with `status: NOT-VERIFIED` BEFORE implementation commits land.
6. `claim_vs_assertion_audit` field present on every new catalog row P89/P90 mints; runner cross-check passes.

**Effort:** 5–6 days (M+S+S+XS+S+S = ~18–22h).

**Dependencies:** none (entry-point phase).

**Execution mode:** **top-level** (per CLAUDE.md rule for orchestration-shaped phases — fan-out across PROTOCOL.md, runners, catalog, verifiers).

---

### **P90 — Framework fixes: catalog-row honesty rules, dishonest-test triage, honesty-check meta-rule**

**Goal:** Close the verifier-shape exemptions that let P78–P88 graded GREEN with sim-only coverage. Fix-class: **F**.

**REQ-IDs:**
- `RBF-FW-06` — Catalog-row honesty: `coverage_kind: real-backend` required for transport/perf rows; `WAIVED + until_date` blocks PASS-with-comment (per F-K4a).
- `RBF-FW-07` — Runner cross-checks `expected.asserts` ↔ `asserts_passed` artifact at grade time (per F-K4b).
- `RBF-FW-08` — `kind: subagent-graded` rows MUST wire to `dispatch.sh` (per F-K4c).
- `RBF-FW-09` — `quality/gates/agent-ux/test-name-vs-asserts.sh` triage gate (per F-K8).
- `RBF-FW-10` — Absorption-phase honesty-check meta-rule: sample EVERY no-intake phase + spot-check ≠ orchestrator + rubric "walk it mentally" + content-hash binding (per F-K5).
- `RBF-FW-12` — Milestone-close adversarial pass: a fresh subagent reads catalog row descriptions only (no implementation context) and grades whether the assertion would falsify the description; runner blocks GREEN if ≥1 row's audit fails per Decision 3.

**HIGH findings closed by this phase:** H-A4, H-G1 (verifier shape rules), H-H1, H-H2, H-H3, H-H4, H-H5, H-H6 (honesty-check mechanics), H-I3..H-I19 partially (verifier-shape framework that the I-cluster code/doc fixes will then consume).

**Success criteria:**
1. Pre-push gate runs `test-name-vs-asserts.sh`; flagged rows produce a structured RAISE list.
2. Runner refuses to flip a row PASS if `expected.asserts` text does not align with `asserts_passed` strings (vocabulary mismatch ⇒ verifier graded RED, see p86 F6).
3. Catalog migration script flips every `kind: subagent-graded` row that lacks `dispatch.sh` wiring to `kind: mechanical` (or wires `dispatch.sh` if intent was real grading) — fixes p86 F7.
4. Absorption-phase template (consumed by P96) carries the meta-rule verbatim.
5. Walk the new gates over the live catalog ⇒ produce a RAISE LIST of every existing dishonest test/row (this list seeds P92 + P94 + P95 work).
6. Milestone-close adversarial pass dispatch documented in `quality/PROTOCOL.md`; rubric file at `quality/dispatch/milestone-adversarial.md`.

**Effort:** 5–6 days (M+S+XS+S+S+S = ~18–20h, plus the catalog walk's RAISE LIST is a deliverable).

**Dependencies:** **P89 GREEN** (uses `cadence: pre-release-real-backend` + `kind: shell-subprocess` from P89).

**Execution mode:** **top-level**.

---

### **P91 — `reposix attach` + `sync --reconcile` real-backend wiring (Cluster A)**

**Goal:** Land confluence/github/jira backends for `attach` and `sync --reconcile`. The work P79-03 was supposed to ship and silently dropped. Fix-class: **C+F+D**.

**REQ-IDs:**
- `RBF-A-01` — `attach.rs:147-166` real-backend dispatch wired (mirror `refresh.rs:174-240` / `backend_dispatch.rs:234-271` pattern); credential plumbing threaded.
- `RBF-A-02` — `sync.rs:79-92` real-backend dispatch wired (same shape as RBF-A-01).
- `RBF-A-03` — Production error strings drop `P79-02 scaffold` / `P79-03` / `P82+` phase IDs (caught by F-K7 banned-tokens regex).
- `RBF-A-04` — `agent_flow_real.rs` ships `attach_real_{confluence,github,jira}` and `sync_real_{confluence,github,jira}` `#[ignore]` smoke tests (vanilla `git init` + `reposix attach $BACKEND::$PROJECT` + assert post-conditions; same shape for sync).
- `RBF-A-05` — `dark-factory.sh dvcs-third-arm` harness creates a NON-EMPTY work tree + populated bare mirror so reconciliation walk exercises matched/no_id/backend_deleted/mirror_lag/orphan cases (closes p86 F13).
- `RBF-A-06` — REQUIREMENTS.md DVCS-ATTACH-01..04 status flipped to reflect real-backend coverage; `architecture-sketch/index.md` lookup table loses "shipped" overstatement.
- `RBF-A-07` — CLAUDE.md attach example replaced (current example uses `git clone git@github.com:...` then `reposix attach sim::demo` — incoherent; see p79 F8).

**HIGH findings closed:** H-A1, H-A2, H-A3, H-A5, H-A6, H-A7.

**Success criteria:**
1. `reposix attach confluence::REPOSIX --remote-name reposix` against a vanilla mirror clone configures git correctly + reconciles by frontmatter id (5 cases per architecture sketch).
2. `agent_flow_real.rs` `attach_real_*` family GREEN with credentials set, default-skipped without (gated by `cadence: pre-release-real-backend`).
3. `cargo run -p reposix-cli -- attach confluence::REPOSIX` no longer emits any `P\d+-\d+` token to stderr (banned-tokens regex enforced).
4. `dark-factory.sh dvcs-third-arm` populates work tree before attach; reconciliation report names non-zero counts in at least three reconciliation cases.
5. T2 in dark-factory exercise re-runs and passes 5/5 boxes.
6. **Mid-stream litmus checkpoint (Decision 1):** After this phase declares GREEN, re-run dark-factory T2 against TokenWorld (the relevant real backend). If ≥1 HIGH-severity friction surfaces, this phase REOPENS — the next phase MUST NOT start until T2 returns ≤0 HIGH frictions. The checkpoint is a phase gate, not a soft success criterion.

**Effort:** 5 days (M+S+XS+S+S+XS+XS = ~14–16h, but the populated-fixture work in RBF-A-05 is non-trivial).

**Dependencies:** **P89 + P90 GREEN** (uses cadence + kind + honesty rules).

**Execution mode:** `gsd-execute-phase` (code-heavy, fits executor envelope).

---

### **P92 — Push-flow correctness fixes: rebase recovery + OP-3 audit log silence (Clusters B + C)**

**Goal:** Fix the v0.9.0 architectural cornerstone (`git pull --rebase`) and the OP-3 audit-log silence. Both broken on every push from a partial-clone working tree. Fix-class: **C**.

**REQ-IDs:**
- `RBF-B-01` — Helper-side fetch preserves ancestry across post-push refetches; no fresh root commits per fetch (closes dark-factory CLUSTER B / vision-audit F4 / `T4 HIGH-1`).
- `RBF-B-02` — Helper opens cache.db with correct gitdir/cwd resolution; `helper_push_*` rows MUST land for OP-3 compliance on every push (closes dark-factory CLUSTER C / vision-audit F3).
- `RBF-B-03` — `instantiate_{confluence,github,jira}` chains `.with_audit(audit_conn)` so `audit_events` table is written on every real-backend write (closes p83 F3).
- `RBF-B-04` — `bus_write_audit_completeness.rs` extended to actually query `audit_events` (not just `audit_events_cache`); closes p83 F3 / p86 F5.
- `RBF-B-05` — `agent_flow_real.rs` smoke that performs `git push` against TokenWorld and asserts BOTH cache + backend audit tables have rows.
- `RBF-B-06` — Dark-factory regression's third arm extended to drive a real `git push reposix main` against in-process sim + file-bare mirror, asserting OP-3 dual-table audit row presence (closes p86 F1 / SC #3 restoration).
- `RBF-B-07` — Behavioral no-retry verifier replaces the source-grep at `bus-write-no-helper-retry` (closes p83 F6).

**HIGH findings closed:** H-B1, H-B2, H-B3, H-B4, H-G4 partially (SC #3 restored), H-G5.

**Success criteria:**
1. Two-writer conflict scenario in T4 completes step 6 + step 7 against sim AND TokenWorld.
2. After every `git push` from a partial-clone working tree (sim + real Confluence + real GH issues + real JIRA), `audit_events_cache` AND `audit_events` BOTH show rows for the action; `cache.db` is created on first push if missing.
3. `bus_write_audit_completeness.rs` queries both tables; OP-3 dual-table assertion is real not metaphorical.
4. Verifier subagent's "honesty spot-check" treats audit-row absence as RED, not "out of scope for this layer".
5. **Mid-stream litmus checkpoint (Decision 1):** After this phase declares GREEN, re-run dark-factory T1 + T4 (sim end-to-end + rebase recovery against sim and TokenWorld). If ≥1 HIGH-severity friction surfaces, this phase REOPENS — the next phase MUST NOT start until T1 + T4 return ≤0 HIGH frictions. The checkpoint is a phase gate, not a soft success criterion.

**Effort:** 5 days (M+M+S+S+S+S+S = ~22h). **NOTE:** B-01 is genuinely a debugger-required investigation; if it bottoms out at >16h, split as P92a (rebase) + P92b (audit) — flag for re-scoping after the first day's research subagent.

**Dependencies:** **P89 + P90 + P91 GREEN** (P91 lands real-backend dispatch that audit must then write through).

**Execution mode:** `gsd-execute-phase`.

---

### **P93 — L2/L3 cache-coherence + SotPartialFail recovery (Decision 2 promotion)**

**Goal:** Pull two items out of v0.14.0 deferral that the v0.13.0 round-trip vision actually depends on: (a) L2/L3 cache-coherence redesign so `refresh_for_mirror_head` doesn't silently re-run `list_records` on the no-op-push path, and (b) `SotPartialFail` + recovery-via-fetch-replan-push test so partial-failure recovery is exercised, not just architected. Fix-class: **C+F**.

**REQ-IDs:**
- `RBF-LR-01` — L2/L3 cache-coherence: design decision (L2 = re-fetch on cache miss; L3 = transactional cache writes; trade-off doc + ADR + chosen path implemented).
- `RBF-LR-02` — `refresh_for_mirror_head` no longer no-ops on the post-write path; honest L1 promise without asterisk.
- `RBF-LR-03` — `SotPartialFail` recovery test: simulate SoT-success + mirror-fail; assert next push reads new SoT via PRECHECK B and replans correctly.
- `RBF-LR-04` — `agent_flow_real.rs` ships `partial_failure_recovery_real_*` `#[ignore]` smoke for at least Confluence (TokenWorld arm).
- `RBF-LR-05` — Mid-stream litmus checkpoint REOPEN gate per Decision 1: re-run T1 + T4 against sim and TokenWorld after this phase ships GREEN; ≥1 HIGH friction REOPENs the phase before P94 starts.

**HIGH findings closed:** Decision 2 promotion of items previously in §6 deferral table ("L2/L3 cache-coherence redesign" + "`SotPartialFail` + recovery-via-fetch-replan-push test").

**Success criteria:**
1. L2/L3 ADR landed in `docs/decisions/`.
2. `cargo test -p reposix-cache --test cache_coherence` passes against the chosen architecture.
3. `partial_failure_recovery_real_confluence` smoke GREEN with credentials, default-skipped without (gated by `cadence: pre-release-real-backend`).
4. CLAUDE.md / docs L1 promise updated to remove the asterisk if RBF-LR-02 lands honest, OR keep asterisk + qualify in `dvcs-topology.md` if architectural reality requires it.
5. **Mid-stream litmus checkpoint (Decision 1):** After this phase declares GREEN, re-run dark-factory T1 + T4 against sim and TokenWorld (the relevant real backends for cache-coherence + recovery). If ≥1 HIGH-severity friction surfaces, this phase REOPENS — the next phase MUST NOT start until T1 + T4 return ≤0 HIGH frictions. The checkpoint is a phase gate, not a soft success criterion.

**Effort:** 5 days (M+M+S+S+S = ~18–22h). **NOTE:** RBF-LR-01 ADR conclusion (L2 vs L3 vs hybrid) drives effort. Flag for re-scoping after ADR lands.

**Dependencies:** **P89 + P90 + P91 + P92 GREEN** (uses real-backend dispatch from P91 + audit log from P92 to ground the cache-coherence assertions).

**Execution mode:** `gsd-execute-phase` for code; ADR (RBF-LR-01) is **top-level**.

---

### **P94 — Bus-push compatibility with documented mirror setup (Cluster D)**

**Goal:** Fix the architectural collision where helper export validator rejects exactly the files the documented mirror-setup tells users to commit. Fix-class: **C+D**.

**REQ-IDs:**
- `RBF-C-01` — Architectural decision (ADR landed in `docs/decisions/`): skip-list, allowlist, or path-prefix scope (e.g. `?records_root=pages/`)? Trade-off doc + decision.
- `RBF-C-02` — Helper export accepts (or skips) non-frontmatter files in the mirror tree (`.github/workflows/*`, `README.md`, `.reposix/*`) per RBF-C-01 ADR (closes H-C1).
- `RBF-C-03` — Bus push against a mirror with `.github/workflows/reposix-mirror-sync.yml` succeeds (T3 step 5 passes).
- `RBF-C-04` — `dvcs-mirror-setup.md` Step 4 fix for self-clobber: workflow YAML lives in dedicated branch (`refs/heads/sot-mirror`) OR workflow re-commits itself to `/tmp/sot` before push OR sibling-repo pattern (architectural decision per RBF-C-01) (closes H-C3).
- `RBF-C-05` — STEP 0 / PRECHECK A three-step prereq chain documented: `git remote set-url origin reposix::<sot>?mirror=<plain>` + `git remote add mirror <plain>` + `git fetch mirror` named explicitly in `dvcs-mirror-setup.md` (closes H-C2).
- `RBF-C-06` — Bus URL form `reposix::<sot>?mirror=<url>` documented in `dvcs-topology.md`, `dvcs-mirror-setup.md`, `troubleshooting.md` (closes H-I14).
- `RBF-C-07` — `bus_url::parse` validates / errors on unencoded `?` in mirror value (or doc moves "MUST encode" to "WILL encode silently" with clear behavior) (closes p82 F2 + F6).

**HIGH findings closed:** H-C1, H-C2, H-C3, H-I14, H-I15.

**Success criteria:**
1. `reposix attach confluence::REPOSIX` then `git push` succeeds against `reubenjohn/reposix-tokenworld-mirror` with a `.github/workflows/reposix-mirror-sync.yml` present.
2. First successful workflow run does NOT delete the workflow file from `main` (or RBF-C-04 ADR documented alternate topology).
3. ADR landed and reviewed.
4. T3 in dark-factory exercise re-runs and passes 8/8 boxes.
5. **Mid-stream litmus checkpoint (Decision 1):** After this phase declares GREEN, re-run dark-factory T3 against TokenWorld + GH mirror (the relevant real backend). If ≥1 HIGH-severity friction surfaces, this phase REOPENS — the next phase MUST NOT start until T3 returns ≤0 HIGH frictions. The checkpoint is a phase gate, not a soft success criterion.

**Effort:** 5 days (M+M+S+M+S+S+S = ~22h). **NOTE:** RBF-C-01 ADR + RBF-C-02 implementation may dominate; if ADR conclusion is "path-prefix scope" the implementation grows to L. Flag for re-scoping after ADR lands.

**Dependencies:** **P89 + P90 + P91 + P92 + P93 GREEN** (need real-backend dispatch + audit log + cache-coherence foundation to validate end-to-end; bus-push correctness depends on P93's L2/L3 cache invariants because helper export traverses cached blobs).

**Execution mode:** `gsd-execute-phase` for code; ADR (RBF-C-01) is **top-level** (requires architectural decision-making).

---

### **P95 — Quality framework upgrade + doc fixes (Clusters E + F + H + I)**

**Goal:** Apply the F-K2 / F-K4 / F-K8 framework rules from P89/P90 to existing rows; fix init UX nits; refresh stale tutorial; fix testing-targets; document git ≥2.34 requirement. Fix-class: **F+C+D**.

**REQ-IDs:**
- `RBF-D-01` — Init's "Next:" hint contradicts itself (`git checkout origin/main` fails); WARN-noise on success path; bare `git push` requires `--set-upstream` (closes H-E1, dark-factory CLUSTER E F1+F2+F5).
- `RBF-D-02` — README + `first-run.md` tutorial expected output refreshed to match actual sim seed (closes dark-factory CLUSTER F).
- `RBF-D-03` — `testing-targets.md` says REPOSIX (not TokenWorld); add a verifier that probes the configured tenant for the named space (closes dark-factory CLUSTER H1).
- `RBF-D-04` — README adds `reposix attach` mention; install docs surface git ≥2.34 requirement.
- `RBF-D-05` — Pattern C tutorial added (`docs/tutorials/round-tripper.md`); cross-linked from `dvcs-topology.md` Pattern C (closes p85 F4 / F11 / F16).
- `RBF-D-06` — Migrate every existing P78-P88 catalog row to F-K2/F-K4 honesty rules (P90's RAISE LIST drives this). Each `kind: mechanical` row whose description implies functional verification gets a `coverage_kind` field; transport/perf rows get `cadence: pre-release-real-backend`; vacuous rows get `WAIVED + until_date`.
- `RBF-D-07` — `webhook-latency-floor` row WAIVED with explicit `until_date` until `cargo binstall reposix-cli` substrate ships (closes p84 F2 / F13 / H-I11).
- `RBF-D-08` — `cargo binstall reposix-cli` substrate fix: `pkg-url` template aligned with release-pipeline archive name (closes p84 F3 / H-I12).
- `RBF-D-09` — `scripts/webhook-latency-measure.sh --synthetic` flag implemented (CHANGELOG + RETROSPECTIVE both promised it; doesn't exist) (closes p84 F7).
- `RBF-D-10` — `init.rs` writes `refs/mirrors/<sot>-{head,synced-at}` so first cron tick of fresh mirror has refs (closes p80 F1 / H-I3) OR workflow YAML drops the `refs/mirrors/*` push line until subsequent sync happens (per RBF-C-01 ADR).
- `RBF-D-11` — `vanilla_fetch_brings_mirror_refs` test exercises actual `git upload-pack --advertise-refs --stateless-rpc` advertisement assertion (closes p80 F2 / F11 / H-I4).
- `RBF-D-12` — `perf_l1.rs` test exercises a non-no-op push (or CLAUDE.md / docs honestly state the L1 promise carries an asterisk re. `refresh_for_mirror_head`) (closes p81 F2 / H-I6).
- `RBF-D-13` — DVCS-BUS-FETCH-01 either explicitly asserts behavioral fetch over a bus URL succeeds (and via stateless-connect not deprecated import) OR catalog row description is corrected to "absence of stateless-connect on bus URL" (closes p82 F1 / F10 / H-I7 / H-I9).
- `RBF-D-14` — Walk the F-K8 dishonest-test triage RAISE LIST end-to-end; every flagged row either fixed or `WAIVED + until_date`.
- `RBF-D-15` — CLAUDE.md "agent UX is pure git" / "Zero reposix CLI awareness required beyond init/attach" claims qualified with backend-coverage state OR P92's real-backend gate is GREEN such that the unqualified claim is now accurate (closes H-G4 / vision-audit F12).

**HIGH findings closed:** H-E1, H-F1, H-G4 (qualifier), H-H6, H-I3, H-I4, H-I6, H-I7, H-I9, H-I11, H-I12, H-I14 (substantively in P94), H-I15 (substantively in P94), H-I16, H-I19, plus the trailing MED items the dark-factory SUMMARY listed under CLUSTER E/F/H.

**Success criteria:**
1. Dark-factory T1 re-runs and passes 8/8 boxes (cluster E + F resolved).
2. P78-P88 RAISE LIST from P90 fully drained: every flagged row now passes F-K4/F-K8 rules.
3. Mermaid + mkdocs + banned-words + cold-reader passes; new tutorial cross-linked.
4. CLAUDE.md "pure git" claim either qualified or P92-real-backend-gate-green.
5. README + `first-run.md` cold-reader pass GREEN (run `/reposix-quality-review --rubric cold-reader-hero-clarity` after refresh).

**Effort:** 5 days (15 REQ-IDs at average S = ~30+h), **CANDIDATE FOR SPLIT.** If P90's RAISE LIST is large, split as:
- **P95a** — RBF-D-01..09 (init UX + tutorial + testing-targets + docs honesty).
- **P95b** — RBF-D-10..15 (catalog migration + framework-rule application + claim qualifier).

The orchestrator should split during P90's RAISE LIST review.

**Dependencies:** **P89 + P90 + P91 + P92 + P93 + P94 GREEN** (P90's RAISE LIST drives RBF-D-06/D-14; P91's real-backend dispatch enables RBF-D-15's claim test; P92's audit fix needed for D-06 transport rows; P93's L2/L3 architecture must land before D-12's L1 promise honesty work).

**Execution mode:** `gsd-execute-phase` (D-01..05, D-08, D-09 = code/doc); **top-level** for D-06/D-14 (RAISE LIST drain is orchestration-shaped fan-out).

---

### **P96 — Surprises absorption (+2 reservation slot 1, OP-8)**

**Goal:** Drain anything P89–P95 surfaced but couldn't fix without doubling scope; retroactively file the 16 dark-factory HIGH frictions + audit findings as v0.13.0-discovered intake (closes the chronological-defensibility issue from p87 F8 / vision-audit F8). Fix-class: **F+D**.

**REQ-IDs:**
- `RBF-S-01` — Drain `SURPRISES-INTAKE.md` for v0.13.0 extension (every entry → RESOLVED | DEFERRED | WONTFIX with commit SHA or rationale).
- `RBF-S-02` — Retroactive v0.13.0 SURPRISES-INTAKE entries: file the 16 dark-factory HIGH frictions + ~51 audit HIGH findings against their originating phases (closes vision-audit F8 — chronologically defensible because dark-factory was post-tag, but documenting the gap is OP-9 honesty).
- `RBF-S-03` — Apply F-K5 honesty-spot-check meta-rule to v0.13.0 extension absorption: spot-check author ≠ orchestrator (dispatched as fresh subagent); sample includes EVERY no-intake phase; rubric = "walk one critical example end-to-end mentally — does it work?".
- `RBF-S-04` — RETROSPECTIVE.md v0.13.0 amendment: the 16 HIGH dark-factory frictions added as a "What Was Inefficient" section AND root-cause analysis ("framework structurally exempted real-backend flows") added under "Patterns Established (Anti-pattern)" (closes p88 F3 + F9 / H-I18).
- `RBF-S-05` — Apply "EXTENDED-PENDING-P89-P97" overlay note to the 12 existing GREEN verdict files at `quality/reports/verdicts/p7{8,9},p8{0..8}/VERDICT.md` + `milestone-v0.13.0/VERDICT.md`. Overlay is a 2026-05-08-dated banner at the top of each file pointing forward to P97's milestone-close verdict for the post-extension state. Per Decision 4 option (b).

**HIGH findings closed:** H-A3 (retroactive intake), H-H2 (retroactive close), H-I17 (GOOD-TO-HAVES-01 deferral honesty), H-I18 (RETROSPECTIVE amendment).

**Success criteria:**
1. Every v0.13.0 intake entry has terminal STATUS with commit SHA or rationale.
2. Independent (non-orchestrator) honesty-spot-check subagent dispatched; verdict GREEN under F-K5 rubric.
3. RETROSPECTIVE.md v0.13.0 amendment merged; the 16 HIGH frictions are visible to the next milestone's planner.

**Effort:** 4 days (S+M+S+S = ~10–14h).

**Dependencies:** **P89–P95 GREEN**.

**Execution mode:** **top-level** (orchestration-shaped: fan out to dispatch independent honesty subagent + drain intake + amend RETROSPECTIVE).

---

### **P97 — Good-to-haves polish + milestone close (+2 reservation slot 2, OP-9 ritual)**

**Goal:** XS / S items the running phases observed but didn't fold in; milestone-close ritual; tag `v0.13.0`. Fix-class: **F+D+C**.

**REQ-IDs:**
- `RBF-G-01` — Drain `GOOD-TO-HAVES.md` for v0.13.0 extension; XS items always close, M items default-defer to v0.14.0.
- `RBF-G-02` — Cold-reader pass on revised docs (`/reposix-quality-review --all-stale`).
- `RBF-G-03` — RETROSPECTIVE.md distillation for v0.13.0 extension (per OP-9 ritual).
- `RBF-G-04` — Milestone-close verdict using the new probe template (P89's RBF-FW-03); **OVERWRITES `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`** (replacing the prior 2026-05-01 GREEN verdict with the post-extension verdict per Decision 4). 9th probe = vision litmus test against TokenWorld (P91's RBF-A-05 + P92's RBF-B-06 + P93's RBF-LR-04 + P94's RBF-C-03 + P95's RBF-D-15 must all be green for this probe to fire green).
- `RBF-G-05` — Tag `v0.13.0`.

**HIGH findings closed:** the milestone-close-litmus-test gate (RBF-FW-03) closes H-G2 + H-I19 by construction here.

**Success criteria:**
1. Milestone-close verdict at `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` is GREEN under the new 9-probe template (i.e. real-backend litmus test ran and passed).
2. RETROSPECTIVE.md v0.13.0 extension section exists with all five OP-9 subheadings and substantive content.
3. `v0.13.0` tag landed; release pipeline ships GH assets.
4. Re-running the dark-factory exercise produces ≤ 5 frictions (down from 37) and ZERO HIGH (down from 16).

**Effort:** 3 days (S+S+M+S+S = ~10h).

**Dependencies:** **P89–P96 GREEN**.

**Execution mode:** **top-level**.

---

## 4 — Dependency graph (sequential within waves)

```
P89 (framework-1) ─→ P90 (framework-2) ─→ P91 (attach) ─→ P92 (push/audit) ─→ P93 (L2/L3 + recovery) ─→ P94 (bus tree) ─→ P95 (catalog migration + doc fixes) ─→ P96 (absorption) ─→ P97 (good-to-haves + close)
```

**Parallelism note:** P91 ↔ P92 ↔ P93 ↔ P94 are partially parallelizable for code work (different files), BUT all four need the same `cadence: pre-release-real-backend` substrate from P89 + the same honesty rules from P90 + share credential plumbing for the same `BackendConnector` adapters; AND CLAUDE.md memory budget restricts cargo work to one phase-executor subagent at a time; AND each phase's mid-stream litmus checkpoint REOPEN gate (Decision 1) blocks the next from starting until clean. Recommend **serial execution** (P91 → P92 → P93 → P94) unless the orchestrator has clear evidence of disjoint files at task-decomposition time.

## 5 — Effort summary

| Phase | Goal headline | Effort | Class |
|---|---|---|---|
| P89 | Framework: real-backend cadence + shell-subprocess kind + milestone-close probe + claim-vs-assertion congruence | 5–6d | F |
| P90 | Framework: catalog honesty rules + dishonest-test triage + spot-check meta-rule + milestone-close adversarial pass | 5–6d | F |
| P91 | Code: `attach` + `sync --reconcile` real-backend wiring | 5d | C+F+D |
| P92 | Code: rebase recovery + OP-3 audit log silence | 5d (split candidate) | C |
| P93 | Code+ADR: L2/L3 cache-coherence + SotPartialFail recovery (Decision 2 promotion) | 5d (split candidate) | C+F |
| P94 | Code+ADR: bus-push collision with mirror tree | 5d (split candidate) | C+D |
| P95 | Catalog migration + init UX + tutorial + 15 mixed fixes | 5d (split candidate) | F+C+D |
| P96 | Surprises absorption (independent honesty spot-check) + verdict-file overlay (Decision 4) | 4d | F+D |
| P97 | Good-to-haves + 9-probe milestone close + tag (overwrites milestone-v0.13.0/VERDICT.md) | 3d | F+D+C |
| **Total** | | **~42–45 days** | |

## 6 — What this plan does NOT fix (deferred to v0.14.0+)

These items are visible in the audits + dark-factory + vision audit but are explicitly out of v0.13.0 extension scope. Each carries a rationale.

Note: 2 items previously in this table were promoted into v0.13.0 extension scope per Decision 2 — see new P93.

| Item | Source | Why deferred |
|---|---|---|
| **Reflog growth on long-lived caches** | `p80 F6` | Future operational concern (DoS-disk on long-lived caches); currently only an in-source comment. v0.14.0 observability/multi-repo milestone is the natural home. |
| **Phase 36 deprecated `import` removal** | `p82 F1`, `p82 F10` | Bus URL fetch currently routes through deprecated v0.8 `import`. Phase 36 already scheduled to remove `import`. v0.13.0 extension RBF-D-13 makes the catalog row honest; v0.14.0 / Phase 36 ships the actual stateless-connect-only fetch path. |
| **`pushurl` vs `url` in mirror remote resolution** | `p82 F13` | Edge case for users with split push/fetch URLs. SUSPECT-rated; could be plan-scoped intentionally. v0.14.0 enhancement. |
| **`force` semantics for inbound user-driven `git push --force` to bus URL** | `p83 F9` | SUSPECT — needs planning-level decision (force-with-lease? `?force=ok` query param? user runs `reposix sync --reconcile`?). v0.13.0 extension not the venue. |
| **`mkdocs` site-build performance / Mermaid render perf** | not v0.13.0-extension-relevant | Out of scope. |
| **Crash-atomicity between `write_mirror_head` and `push_mirror`** | `p83 F10` | Edge case under helper crash. Documented in plan; behavioral defect is on crash only. v0.14.0 atomicity work. |
| **Bus push on JIRA / GitHub Issues SoT (vs Confluence-only)** | not yet a HIGH; orthogonal | v0.13.0 architecture-sketch posits Confluence-as-canonical-SoT. JIRA/GH-issues-as-SoT is a v0.14.0 generalization. v0.13.0 extension only re-instates the milestone's own claims. |
| **GOOD-TO-HAVES-01 Path B (extend `bind` to remaining 7 dimensions)** | `p87 F6`, `p88 F8` | Path A shipped post-milestone; Path B (other 7 dimensions) is v0.14.0 polish. v0.13.0 extension's framework rules don't depend on Path B. |
| **Multi-source legacy backfill (14 rows still in path-(a) tradeoff)** | `p78 F7`, `p78 F8` | Partial closure of MULTI-SOURCE-WATCH-01; the residual 14 legacy multi-source rows skip drift compare. Not load-bearing for v0.13.0 extension real-backend work; v0.14.0 docs-alignment polish. |
| **Cold-reader rubric subagent quality** | `p85 F2`, `p85 F5` | Rubric subagent missed F1/F4/F11/F12 in P85 grading; rubric reliability is a meta-question the framework hasn't fully closed. v0.13.0 extension fixes the missing assertions (RBF-D-15) but does NOT redesign rubric grading. |

**Items NOT deferred (every HIGH gets remediated in v0.13.0 extension):** every item in the ~51-finding inventory above is mapped to a phase. The deferrals above are MED/LOW/SUSPECT items that the audits enumerated but the orchestrator-defined HIGH bar didn't promote.

## 7 — Honesty caveats

Four places where this plan trades scope for shippability:

1. **P92 effort is borderline.** RBF-B-01 (rebase ancestry) is genuinely a "needs a debugger" investigation. If it bottoms out >16h, split as P92a/P92b. Flag for re-scoping at day-1 research.

2. **P94 ADR conclusion drives effort.** RBF-C-01's path-prefix-scope outcome could grow RBF-C-02 to L. Re-scope after ADR.

3. **P95 RAISE LIST is unknown ahead of time.** P90 produces it; if the list is larger than estimated, split as P95a/P95b. The orchestrator should look at the RAISE LIST before scoping P95.

4. **P93 is fresh-promotion territory.** Both REQ-IDs in P93 (L2/L3 + SotPartialFail recovery) were §6 deferrals before Decision 2 promoted them. The original deferral rationale ("v0.14.0 cache-coherence work is the natural home") was not invalidated by new evidence; it was overridden by the realization that v0.13.0's round-trip vision depends on these working. If P93 ADR (RBF-LR-01) concludes "this genuinely requires wider Cache crate refactoring," re-scope P93 as P93a/P93b — but do NOT defer back to v0.14.0 without explicit owner sign-off, since that re-instates the C8 anti-pattern Decision 2 closes.

These risks are visible in CLAUDE.md OP-8's "scope-creep-to-fit-the-finding" failure mode — naming them upfront keeps the +2 reservation honest.

---

**End of remediation plan.**
