# Phase P86 Audit — Dark-factory regression: DVCS third arm
**Auditor:** unbiased subagent (zero session context)
**Date:** 2026-05-08
**Scope:** vanilla-clone + reposix attach + bus URL composition + cache audit; verifier-of-record for the v0.13.0 "agent UX is pure git" thesis

## Verdict at a glance
- ALIGNED items: 3
- MISALIGNED items: 12
- SUSPECT items: 0

The headline finding is structural: P86's catalog row, plan, ROADMAP entry, and verdict all describe a verifier that exercises an end-to-end DVCS round-trip. The shipped verifier exercises (a) source-code greps, (b) `--help` output greps, and (c) `reposix attach` config writes against a sim. The pivot from "drive `git push`" to "grep teaching strings + cite a cargo test" was made silently inside T02; the catalog row, ROADMAP success criteria, and the public CLAUDE.md claim ("dark-factory regression — proves agent UX is pure git, zero in-context learning") were never reconciled to the narrowed scope. The result is a GREEN gate that, by construction, cannot fail when the documented user flow is broken.

The verifier subagent's GREEN verdict accepted the executor's own pivot rationale verbatim ("DEFENSIBLE") without re-checking against the ROADMAP success criteria the executor pivoted away from. CLUSTER G in `SUMMARY.md` is fully confirmed; below are the constituent findings.

---

## Findings

### F1 — ROADMAP SC #3 ("End-to-end success: typo fix lands in confluence … AND GH mirror") never executes [SEVERITY: HIGH]

**Claim in plan:** ROADMAP P86 SC #3 (`.planning/milestones/v0.13.0-phases/ROADMAP.md:91`) verbatim:
> *"End-to-end success: typo fix lands in confluence (REST GET) AND GH mirror (`git fetch && git log`) AND `refs/mirrors/<sot>-synced-at` advanced; audit rows present in both tables."*

**Reality:** `quality/gates/agent-ux/dark-factory/dvcs-third-arm.sh` never runs `git push`. The only mutating action is `reposix attach`, which writes config + builds a cache; no record is edited, no PATCH is sent, no `refs/mirrors/<sot>-synced-at` is exercised, no `audit_events` row in the SoT table is asserted. The "audit rows in both tables" check is reduced to one `attach_walk` row in `audit_events_cache` only (line 170-176 of the harness). `audit_events` (the core/sim-side table) is never queried.

**Evidence:** `quality/gates/agent-ux/dark-factory/dvcs-third-arm.sh:120-203` — no `git push` codepath; assertions stop at attach config + one cache audit row. ROADMAP claim at `.planning/milestones/v0.13.0-phases/ROADMAP.md:91`. SUMMARY § "Deviations from plan" at `.planning/phases/86-dark-factory-third-arm/86-01-SUMMARY.md:80-91` documents the pivot.

**Why it matters:** This is the load-bearing claim of the entire phase. SC #3 describes the only evidence that would falsify the v0.13.0 DVCS thesis end-to-end ("can a fresh agent push a fix all the way through"). With the pivot, no automation in the milestone exercises the round-trip on any backend (real or simulated). Every CLUSTER A–F finding from the dark-factory exercise was structurally invisible to this gate.

---

### F2 — ROADMAP SC #5 ("Sim AND TokenWorld coverage … milestone-close gate per OP-1") shipped sim-only [SEVERITY: HIGH]

**Claim in plan:** ROADMAP P86 SC #5 (`.planning/milestones/v0.13.0-phases/ROADMAP.md:93`):
> *"Sim AND TokenWorld coverage (CI default + secrets-gated real-backend; milestone-close gate per OP-1)."*

**Reality:** TokenWorld leg is not exercised at all. `dvcs-third-arm.sh:185-191` is a stderr-only "deferral notice" — the `REPOSIX_DARK_FACTORY_REAL_TOKENWORLD=1` branch prints `"SUBSTRATE-GAP-DEFERRED... Skipping."` and falls through without invoking any real backend. There is no `--real-tokenworld` mode body, just a printout. SUMMARY (`86-01-SUMMARY.md:108-114`) acknowledges: *"the gating short-circuit in the harness (currently a stderr message) needs updating to actually drive the TokenWorld leg."*

**Evidence:** `quality/gates/agent-ux/dark-factory/dvcs-third-arm.sh:187-191`; `86-01-SUMMARY.md:108-114`.

**Why it matters:** OP-1 explicitly says simulator-only coverage does NOT satisfy transport-layer claims. The milestone-close gate that was supposed to enforce this for the dark-factory's headline thesis is decorative — the harness does not even contain the TokenWorld branch's body, only a stderr placeholder. The "SUBSTRATE-GAP-DEFERRED" framing in `comment` and `owner_hint` (catalog row at `quality/catalogs/agent-ux.json:996, 1033`) explicitly instructs verifiers "do NOT count as RED" — codifying the bypass.

---

### F3 — ROADMAP SC #1 prompt ("install reposix, attach, fix the bug … push your fix back") not exercised [SEVERITY: HIGH]

**Claim in plan:** ROADMAP P86 SC #1 (`.planning/milestones/v0.13.0-phases/ROADMAP.md:89`):
> *"subprocess agent prompt: 'The repo at <GH-mirror-url> mirrors a confluence backend. Install reposix, attach, fix the bug in `issues/0001.md` (typo on line 3), push your fix back. You have 10 minutes.'"*

**Reality:** No subprocess agent (LLM or otherwise) is spawned. The plan's pivot to a "shell-stub" approach replaces the prompt-driven agent flow with deterministic source-greps + `reposix attach` direct invocation. The "fix the bug" + "push your fix back" steps simply do not execute. The harness skips from "vanilla clone" to "attach + assertion of config shape" — the editor + commit + push leg is missing entirely.

**Evidence:** `dvcs-third-arm.sh:119-203` (no `git commit` of an edit, no `git push`); SUMMARY §29 ("shell-stub > real-LLM") at `86-01-SUMMARY.md:31-32`. The plan's own "shell-stub" rationale (PLAN.md:30-36) commits to "Emulate the agent's expected git workflow (vanilla `git clone`, `reposix attach`, edit, `git push`)" — but only the first two steps ship.

**Why it matters:** The "agent recovers entirely from teaching strings" property is asserted only via grep; the *actual* recovery surface (does the agent try the workflow? does the helper actually emit the teaching strings at runtime when called as a `git push` subprocess in the agent's environment?) is never exercised. CLUSTER G in `SUMMARY.md` calls this out as the meta-failure that allowed v0.13.0 to ship: tests stop at "URL has the right shape," never run `git fetch`/`git push`.

---

### F4 — `dark_factory_real_confluence` test (cited in CLUSTER G) confirmed to never `git fetch`/`git push` [SEVERITY: HIGH]

**Claim in plan:** Test name `dark_factory_real_confluence` implies real-backend dark-factory coverage. It is cited in `docs/reference/testing-targets.md` and CLAUDE.md as the sanctioned real-backend integration test for Confluence. Comments in the file (`crates/reposix-cli/tests/agent_flow_real.rs:25-37`) explicitly state the test is bounded to `reposix init` succeeding and `git config remote.origin.url` returning the expected URL.

**Reality:** Confirmed verbatim. `dark_factory_real_confluence` (lines 144-165) and its siblings (`dark_factory_real_github` at 126-138; `dark_factory_real_jira` at 168-184) all assert (a) `reposix init` exits 0, (b) `git config remote.origin.url` starts with the expected prefix and ends with the expected suffix. Nothing else. No fetch, no push, no record edit, no audit-row check, no `git checkout`. Despite the name ("dark_factory_real_*"), these tests verify nothing about the dark-factory thesis — they validate a CLI exit code + a config-string format.

**Evidence:** `crates/reposix-cli/tests/agent_flow_real.rs:144-165` (Confluence); :126-138 (GitHub); :168-184 (JIRA). File header docblock at lines 24-37 is candid: *"the helper still hardcodes `SimBackend` (Phase 32 limitation … the 'real-backend exercise' verified here is bounded to: 1. reposix init … 2. git config remote.origin.url … Live `git fetch` against a real backend is deferred to a future phase."*

**Why it matters:** This is the root of CLUSTER G. The test exists and is named such that a verifier subagent or maintainer reading the test list sees "real-backend dark-factory" coverage. P86's harness then "delegates wire-path coverage" to the cargo layer (decision in `86-01-SUMMARY.md:31-33`); the cargo layer's "real-backend" test does no wire-path work. The two layers each cite the other for end-to-end coverage; neither layer provides it.

---

### F5 — Wire-path delegation anchor uses wiremock + SimBackend, not a real backend [SEVERITY: HIGH]

**Claim in plan:** PLAN.md and SUMMARY.md both cite `crates/reposix-remote/tests/bus_write_happy.rs::happy_path_writes_both_refs_and_acks_ok` as the wire-path coverage layer to which the shell harness delegates. The catalog row's expected.assert #8 says: *"wire-path coverage delegated to … (helper exec + refs/mirrors writes + dual-table audit)."*

**Reality:** `bus_write_happy.rs::happy_path_writes_both_refs_and_acks_ok` exercises the helper binary against a `wiremock::MockServer` (a fake HTTP server in-process) and a `file://` bare mirror. The "SoT" is `sim_backend(&server)` — a `SimBackend` instance pointing at the wiremock URI (`crates/reposix-remote/tests/common.rs:127-129`). No real Confluence / GitHub / JIRA endpoint is touched. The "dual-table audit" assertion (lines 314-343) checks `audit_events_cache` rows only — `audit_events` (the SoT-side table that OP-3 calls out as the second non-optional table) is never queried.

**Evidence:** `bus_write_happy.rs:184-198` (wiremock setup), :195 (`sim_backend(&server)`), :316-343 (audit assertions limited to `audit_events_cache`); `common.rs:126-129` (`sim_backend` returns a `SimBackend`).

**Why it matters:** The "delegation" defense is itself a structural sim-only path. P86 layers two simulator-only verifiers and frames their composition as a real-backend round-trip ("dual-table audit"). Neither OP-1 ("simulator-only does NOT satisfy transport-layer claims") nor OP-3 ("dual-table audit forensic completeness") is actually enforced for the dark-factory third arm. The forensic-query "dual-table" claim does not stand: only one table is checked.

---

### F6 — Catalog row's `expected.asserts` enumerate 9 items; harness produces 17 PASS strings; the two lists do not align [SEVERITY: MED]

**Claim in plan:** `quality/catalogs/agent-ux.json:1008-1018` enumerates 9 expected.asserts. The SUMMARY (`86-01-SUMMARY.md:60`) and verdict (`quality/reports/verdicts/p86/VERDICT.md:11`) cite "17 asserts in stderr summary."

**Reality:** The 9 catalog assertions and the 17 stderr `ASSERT_LOG` entries are different lists. The catalog claims (e.g., expected.assert #2: *"shell-stub agent recovers `?mirror=` canonical bus URL form"*; #7: *"audit_events_cache contains an attach_walk row"*) are subsumed by the harness's 17 finer-grained checks, but two of the catalog's expected.asserts are NOT mechanically verified by any line in the harness:
- expected.assert #1 (*"bash dark-factory.sh dvcs-third-arm exits 0"*) is implicit — never re-asserted by the harness against itself.
- expected.assert #9 (*"TokenWorld real-backend leg SUBSTRATE-GAP-DEFERRED: skipped unless REPOSIX_DARK_FACTORY_REAL_TOKENWORLD=1"*) is asserted only by the harness emitting a stderr line, not by any `ASSERT_LOG` entry.

The artifact JSON (`quality/reports/verifications/agent-ux/dark-factory-dvcs-third-arm.json`) records the 17 stderr-PASS strings, not the 9 catalog asserts. A verifier reading `expected.asserts` and cross-checking against `asserts_passed` would find a vocabulary mismatch.

**Evidence:** Catalog row `quality/catalogs/agent-ux.json:1008-1018` (9 items); harness `dvcs-third-arm.sh:61-74, 79-94, 99-116, 142-183` (17 ASSERT_LOG entries); artifact `quality/reports/verifications/agent-ux/dark-factory-dvcs-third-arm.json:5` (17 strings).

**Why it matters:** The catalog row is the contract. If a future change drops one of the 9 expected.asserts (e.g., removes the audit row check), the artifact's 17-string list won't surface it because the runner doesn't compare the two lists. The verifier subagent's grade ("9 asserts in expected.asserts; harness ships 17 passing asserts") accepted the asymmetry without flagging it.

---

### F7 — `kind: subagent-graded` is decorative; verifier is a deterministic shell script [SEVERITY: MED]

**Claim in plan:** ROADMAP SC #4 (line 92) requires `kind: subagent-graded`. Catalog row at `quality/catalogs/agent-ux.json:994` declares `kind: subagent-graded`. The plan (`86-01-PLAN.md:53`) restates it.

**Reality:** No subagent grades this row. The verifier (`quality/gates/agent-ux/dark-factory.sh dvcs-third-arm`) is a deterministic bash script with `grep` and `[[ ... ]]` checks. Comparing to the 4 actually-subagent-graded rows in `quality/catalogs/subjective-rubrics.json` (e.g., `subjective/cold-reader-hero-clarity` at lines 6-47), those use `.claude/skills/reposix-quality-review/dispatch.sh` which spawns a Claude subagent. No such dispatcher is wired for `agent-ux/dvcs-third-arm` — the `verifier.script` field points directly at the shell script.

The `kind` label has runtime consequences only at `cadence: pre-release` per `quality/PROTOCOL.md:151` (STALE subagent-graded rows flip to NOT-VERIFIED). Since this row's cadence is `pre-pr`, the kind label is currently functionally inert — but mislabeling it `subagent-graded` is a contract violation: a future change of cadence to `pre-release` would silently activate freshness-flip behavior the verifier cannot satisfy, and the grading-via-real-subagent contract is not what the verifier delivers.

**Evidence:** Catalog row `agent-ux/dvcs-third-arm` at `quality/catalogs/agent-ux.json:994` (kind: subagent-graded) + :1020-1027 (verifier.script points at deterministic shell); compare `subjective/cold-reader-hero-clarity` at `quality/catalogs/subjective-rubrics.json:9, 23-31` (kind: subagent-graded + dispatch.sh wiring); `quality/PROTOCOL.md:151` (pre-release freshness semantics for subagent-graded).

**Why it matters:** The kind taxonomy is structural — the catalog README defines `subagent-graded` as "rubric-driven subagent." If the row is mechanical, the honest kind is `mechanical`. Labeling it `subagent-graded` to satisfy ROADMAP SC #4 is goal-coverage rather than honest classification, and it pollutes the dimension's kind statistics.

---

### F8 — CLAUDE.md update describes a third arm that does what it does NOT do [SEVERITY: MED]

**Claim in plan:** ROADMAP SC #6 + plan T02 require CLAUDE.md update reflecting the actual shipped state.

**Reality:** CLAUDE.md "Local dev loop" (lines 187-191) lists:
```
# Dark-factory regression (proves agent UX is pure git, zero in-context learning)
bash scripts/dark-factory-test.sh sim                          # v0.9.0 arm — init + partial-clone + helper teaching strings (local + CI)
bash quality/gates/agent-ux/dark-factory.sh dvcs-third-arm     # v0.13.0 P86 arm — vanilla-clone + reposix attach + bus URL composition + cache audit (local + CI)
```

The third-arm bullet's words "vanilla-clone + reposix attach + bus URL composition + cache audit" are accurate. But the heading sentence above ("proves agent UX is pure git, zero in-context learning") applies to BOTH arms by typography, and the third arm does not prove "agent UX is pure git" — it proves teaching-string presence in source + post-attach config shape. The phrase "vanilla-clone" in the bullet description is also misleading: the harness does `git init` + `git remote add origin file://...`, not a `git clone` against a populated mirror (because the bare mirror is freshly initialized at line 129 of the harness — there's nothing to clone). A user reading CLAUDE.md and trying `git clone <real GH mirror url> && reposix attach` would not be running the same shape the harness runs.

The first script path (`bash scripts/dark-factory-test.sh sim`) also points at a path that no longer exists — the script was migrated to `quality/gates/agent-ux/dark-factory.sh` (per `quality/gates/agent-ux/dark-factory.sh:14` "MIGRATED FROM: scripts/dark-factory-test.sh per SIMPLIFY-07 (P59)"). This is a CLAUDE.md staleness bug; either the migration isn't complete (a `scripts/dark-factory-test.sh` shim should still exist) or CLAUDE.md should cite the new path for both arms.

**Evidence:** `CLAUDE.md:188-191`; harness `dvcs-third-arm.sh:122-130` (`git init` + `remote add origin`, NOT `git clone`); script migration note `quality/gates/agent-ux/dark-factory.sh:14`; `ls scripts/dark-factory-test.sh` would clarify (SUSPECT — see below).

**Why it matters:** The dark-factory thesis is the headline claim that breaks first when actual users try the documented flow (CLUSTER E, F in `SUMMARY.md`). CLAUDE.md is the agent's primary contract surface; mis-stating what the third arm proves means the next agent inherits the over-claim and re-validates against the wrong shape.

---

### F9 — Verifier subagent's "DEFENSIBLE" sign-off accepts the executor's pivot rationale verbatim [SEVERITY: MED]

**Claim in plan:** Per CLAUDE.md OP-7 ("Phase-close means catalog-row PASS … unbiased verifier subagent grades"), the verifier subagent grades from artifacts with zero session context.

**Reality:** The verdict at `quality/reports/verdicts/p86/VERDICT.md:34-50` ("Wire-path delegation legitimacy — DEFENSIBLE") explicitly cites the executor's commit-message rationale verbatim and accepts it: *"Rationale (per commit `59fa6aa` body): driving the helper as a `git push` subprocess at shell scope is documented best-effort … and brittle to env propagation."*

The verifier did NOT cross-check this against the ROADMAP success criteria the executor pivoted away from. ROADMAP SC #3 is unambiguous about end-to-end push behavior; the verdict's "DEFENSIBLE" sign-off does not address SC #3 at all. SC #5 (TokenWorld coverage) is addressed only via the executor's own substrate-gap reframing ("LEGITIMATE substrate gap"), again accepting the framing without testing the alternative ("ship the row FAIL until v0.13.x lands the substrate").

The verifier subagent prompt template per `quality/PROTOCOL.md` requires evidence-based grading. In practice, the verdict is a recap of the SUMMARY.md self-grade, not an independent audit.

**Evidence:** `quality/reports/verdicts/p86/VERDICT.md:34-62`; ROADMAP `.planning/milestones/v0.13.0-phases/ROADMAP.md:88-95` (SC #3 + SC #5 unaddressed by the verdict); SUMMARY § "Deviations from plan" `.planning/phases/86-dark-factory-third-arm/86-01-SUMMARY.md:80-91` (executor's own pivot rationale, mirrored by the verdict).

**Why it matters:** This is the meta-failure CLUSTER G calls out. The framework's last line of defense (unbiased verifier subagent) accepted the phase's reframing without re-grading against the original contract. Every subsequent phase that depends on P86 (and the milestone close, which lists P86 as a load-bearing gate at `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md:37`) inherits the over-grade.

---

### F10 — Milestone close gate runs sim arm only; third arm never runs at milestone-close [SEVERITY: HIGH]

**Claim in plan:** Both ROADMAP SC #5 ("milestone-close gate per OP-1") and `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md:37` ("P86 GREEN — dark-factory third-arm scenario (17 asserts)") imply the milestone-close verifier exercised the third arm.

**Reality:** The milestone-close verdict's probe #4 (`quality/reports/verdicts/milestone-v0.13.0/VERDICT.md:19`) ran `bash quality/gates/agent-ux/dark-factory.sh` with no argument. Per the dispatcher script (`quality/gates/agent-ux/dark-factory.sh:33`), the default `BACKEND` is `sim` — so the milestone close ran the v0.9.0 sim arm only. The third arm was not executed at milestone close. The "17 asserts" claim is the artifact JSON read from disk, not a fresh execution.

**Evidence:** `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md:19` ("`bash quality/gates/agent-ux/dark-factory.sh` — exit 0 (sim arm DEMO COMPLETE)") + dispatcher default `quality/gates/agent-ux/dark-factory.sh:33` (`BACKEND="${1:-sim}"`).

**Why it matters:** The freshness gate at milestone close is the framework's stop-gap against PASS rows that were valid at phase-close but broke during inter-phase work. For P86, the milestone close re-graded the cheaper sim arm and stamped GREEN onto the more expensive third arm by reading its stale artifact. The 30d freshness TTL for `agent-ux/dvcs-third-arm` (last_verified 2026-05-01) was not yet expired at milestone-close, so the runner accepted the artifact — but a third-arm regression introduced after 2026-05-01 (e.g., breaking the catalog row dispatch in `dark-factory.sh`) would not be caught until the TTL expired.

---

### F11 — P80's RESOLVED status in SURPRISES-INTAKE incorrectly cites P86 as exercising end-to-end shape [SEVERITY: MED]

**Claim in plan:** SURPRISES-INTAKE entry dated `2026-05-01 09:30` (`.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md:34`) says of P80's mirror-refs verifier shape change: *"P86's dark-factory third-arm regression (which DOES exercise `reposix init` + `git fetch` + bus-push end-to-end against a real GH mirror) covers the same surface; P87 confirms by reading P86's verdict."*

**Reality:** P86 does NOT exercise `reposix init` + `git fetch` + bus-push end-to-end. The pivot in `86-01-SUMMARY.md:80-91` explicitly walks AWAY from that shape. The P80 entry's RESOLVED close sentence (line 36) further claims P86 confirms "the cargo-test-as-verifier shape is a sanctioned house pattern" by delegating to `bus_write_happy.rs` — which is true at the layering level, but the cited justification ("the layered coverage shape is intentional, not a quiet downgrade") elides that the layering still leaves the end-to-end claim unverified by ANY layer.

**Evidence:** `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md:30, 34, 36`; `86-01-SUMMARY.md:80-91` (the explicit pivot the SURPRISES entry cites in the opposite direction).

**Why it matters:** The +2 reservation framework (OP-8) depends on SURPRISES entries being honest. P80's RESOLVED status is grounded in a misrepresentation of P86's actual coverage. P87's honesty-check sampled P86 (per `.planning/phases/87-surprises-absorption/honesty-spot-check.md:37-41`) and graded it ✅ GREEN — but the spot-check only verified that P86 *documented* the deviation, not that the deviation *closed* the load-bearing claim. The honesty check is itself partial.

---

### F12 — "Pure git" public claim never qualified despite shipping with substrate gap [SEVERITY: HIGH]

**Claim in plan:** `docs/index.md:129` (public-facing): *"After `init`, agent UX is pure git: `cat`, `grep -r`, edit, `git commit`, `git push`."* `CLAUDE.md:20`: *"After bootstrap, agent UX is pure git … Zero reposix CLI awareness required beyond `init` / `attach`."* `README.md:66`: *"Agent UX is pure git from here."* P86's verifier was supposed to be the agent-ux gate that proves these claims.

**Reality:** None of these public-facing claims is qualified to "sim only" or "post-substrate-gap." A real reader applying these claims to a real Confluence/GitHub/JIRA backend hits CLUSTERS A–F: attach unimplemented (CLUSTER A), `git pull --rebase` recovery broken (CLUSTER B), audit log silent (CLUSTER C), bus push rejects mirror-setup files (CLUSTER D), init UX broken (CLUSTER E), tutorial output stale (CLUSTER F). P86 was the guard for the public claim; its sim-only scope means the public claim ships with no real-backend gate behind it.

**Evidence:** `docs/index.md:129`; `CLAUDE.md:20, 188-191`; `README.md:66`; CLUSTER A–F findings in `.planning/research/v0.13.0-real-backend-frictions/SUMMARY.md:29-118`; ROADMAP SC #3 + #5 at `.planning/milestones/v0.13.0-phases/ROADMAP.md:91, 93` (the gates that should have caught this).

**Why it matters:** This is the headline of CLUSTER G. The deferral was internal (catalog row `comment` field at `quality/catalogs/agent-ux.json:996`); the public-facing docs make an unqualified claim. A vanilla cold-reader cannot tell from any user-facing doc that the "pure git after init/attach" claim has not been gated against a real backend. The phase that promised to be the guard shipped GREEN with the gate disabled.

---

### F13 — `attach` workflow asserted but the documented Pattern C flow (`git clone` of populated mirror, then attach) is not what the harness does [SEVERITY: MED]

**Claim in plan:** ROADMAP SC #1 prompt: *"The repo at <GH-mirror-url> mirrors a confluence backend. Install reposix, attach, fix the bug, push your fix back."* The implied flow is the documented Pattern C (`docs/concepts/dvcs-topology.md:122-130`): vanilla `git clone <mirror url>` to get a populated working tree, then `reposix attach`.

**Reality:** The harness creates an EMPTY work tree (`git init --quiet "$WORK_REPO"` at `dvcs-third-arm.sh:126`) and a freshly-initialized empty bare mirror (`git init --bare --quiet "$MIRROR_BARE"` at :129). It then runs `reposix attach` against this empty pair. There is nothing to "clone," no `issues/0001.md` to edit, no typo on line 3. The reconciliation walk (asserted at :164) runs against an empty tree, so `matched=0 no_id=0 backend_deleted=0 mirror_lag=0` could pass even if the reconciliation logic is buggy.

**Evidence:** `dvcs-third-arm.sh:122-130`; ROADMAP `.planning/milestones/v0.13.0-phases/ROADMAP.md:89` ("the bug in `issues/0001.md` (typo on line 3)"); Pattern C documentation `docs/concepts/dvcs-topology.md:122-130`.

**Why it matters:** Reconciliation is the load-bearing logic of `reposix attach` — the 5 cases per the architecture-sketch (matched / no_id / backend_deleted / mirror_lag / orphan). An empty-tree exercise touches none of them. The harness's "matched=N no_id=N backend_deleted=N mirror_lag=N" grep (at :164) accepts the report shape, not its content. A buggy reconciliation that returned all-zeros for any non-empty input would still pass.

---

### F14 — Two paths for the same script in CLAUDE.md (shim + canonical) [SEVERITY: LOW]

**Claim in plan:** `CLAUDE.md:189` references `bash scripts/dark-factory-test.sh sim`. `CLAUDE.md:191` references `bash quality/gates/agent-ux/dark-factory.sh dvcs-third-arm`.

**Reality:** Both paths resolve. `scripts/dark-factory-test.sh` is a 7-line shim that `exec`s `quality/gates/agent-ux/dark-factory.sh` with the same args (file content reproduced below). The shim's own comment says "P63 SIMPLIFY-12 audit may delete this shim" — its existence is acknowledged tentative. The dual-listing in CLAUDE.md is at minimum confusing: the sim arm is reached via the legacy shim path, but the third arm is reached via the canonical path. A reader could infer the two arms live in different scripts.

```
#!/usr/bin/env bash
# scripts/dark-factory-test.sh -- migrated to quality/gates/agent-ux/dark-factory.sh per SIMPLIFY-07 (P59).
exec bash "$(dirname "$0")/../quality/gates/agent-ux/dark-factory.sh" "$@"
```

**Evidence:** `CLAUDE.md:189, 191`; `scripts/dark-factory-test.sh:1-7` (shim); `quality/gates/agent-ux/dark-factory.sh:14` (migration note).

**Why it matters:** Cosmetic but indicative — when both paths "work," neither is canonical, and a future SIMPLIFY-12 deletion of the shim breaks the documented command without warning. CLAUDE.md should pick one path and use it for both arms.

---

### F15 — Velocity smell: 21-minute phase shipped a verifier that walks AWAY from 2 of 7 ROADMAP success criteria [SEVERITY: MED]

**Claim in plan:** SUMMARY metrics (`86-01-SUMMARY.md:36-44`): `duration_min: ~21`, `tasks_completed: 3`, `files_modified: 3`. ROADMAP gives 7 success criteria.

**Reality:** A 21-minute phase pivoting away from the load-bearing SC #3 (end-to-end push) AND SC #5 (TokenWorld coverage) is consistent with "scope cut to fit the time budget." The "Eager-resolution preference" rationale in CLAUDE.md OP-8 is meant for items that fit in <1 hour incremental work (`CLAUDE.md:67-75`). A pivot that *removes* load-bearing scope is not eager-resolution — it's scope-cut-as-eager-resolution. The SUMMARY's "Auto-fixed (Rule 3)" framing (line 80) re-categorizes scope-cut as auto-fix, which is OP-8's failure mode the framework explicitly warned about ("the scope-creep-to-fit-the-finding failure mode where a phase grows to twice its planned size … the intake split makes 'I saw it, here's what I think, P<last-2> will handle it' the default move").

**Evidence:** `86-01-SUMMARY.md:36-44, 80-91`; `CLAUDE.md:67-86` (eager-resolution preference + intake-file practice); ROADMAP SC count vs. shipped scope.

**Why it matters:** The +2 reservation slot exists precisely to absorb scope cuts that would otherwise vanish silently. P86 should have filed a SURPRISES-INTAKE entry naming the dropped end-to-end SC + the dropped TokenWorld SC and tagged it for P87 absorption. Instead the SUMMARY says "No SURPRISES-INTAKE entries appended" (`86-01-SUMMARY.md:93-95`). The honest behavior here was: file two intake entries (one per dropped SC), let P87 grade them. The framework allowed this miss because the discovering phase chose its own framing, and the verifier accepted that framing.

---

## Summary table — ALIGNED / MISALIGNED / SUSPECT

| # | Item | Status |
|---|---|---|
| 1 | ROADMAP SC #1 (subprocess agent prompt drives flow) | **MISALIGNED** (F3) |
| 2 | ROADMAP SC #2 (zero-context teaching strings recoverable) | **ALIGNED** — greps verify the strings ARE present in source/`--help` |
| 3 | ROADMAP SC #3 (end-to-end push success on confluence + GH mirror) | **MISALIGNED** (F1) |
| 4 | ROADMAP SC #4 (catalog row in agent-ux dim, kind subagent-graded) | **MISALIGNED** (F7) |
| 5 | ROADMAP SC #5 (sim AND TokenWorld coverage) | **MISALIGNED** (F2, F10) |
| 6 | ROADMAP SC #6 (catalog rows + CLAUDE.md updated first) | **ALIGNED** — T01 catalog-first commit + CLAUDE.md update at T02 |
| 7 | ROADMAP SC #7 (phase close push + verifier GREEN) | **ALIGNED** (mechanics) but verifier was non-adversarial (F9) |
| 8 | DVCS-DARKFACTORY-01 ("dvcs-third-arm scenario") | **MISALIGNED** (F1, F3, F13) |
| 9 | DVCS-DARKFACTORY-02 (catalog row minted) | **MISALIGNED** (F6, F7) |
| 10 | OP-3 dual-table audit assertion | **MISALIGNED** (F1, F5) |
| 11 | OP-1 simulator-only insufficiency | **MISALIGNED** (F2, F5, F10) |
| 12 | "Pure git" public claim guarded | **MISALIGNED** (F12) |
| 13 | Wire-path delegation honest layering | **MISALIGNED** (F5) |
| 14 | P80 SURPRISES RESOLVED via P86 | **MISALIGNED** (F11) |
| 15 | `scripts/dark-factory-test.sh` reachability | **MISALIGNED (cosmetic)** (F14) |

3 ALIGNED, 12 MISALIGNED, 0 SUSPECT.

## What would make P86 honestly GREEN

For v0.13.1 framework remediation:

1. **Restore SC #3 as a literal end-to-end push assertion.** Either (a) drive `git push reposix main` from the harness against the in-process sim (file-bare mirror) and assert the SoT-side audit row, or (b) explicitly demote ROADMAP SC #3 to "wire-path coverage delegated to bus_write_happy.rs" and stop claiming end-to-end.
2. **Land a real-TokenWorld arm body or downgrade SC #5.** If the substrate gap genuinely blocks the leg, the catalog row's status should be FAIL or WAIVED (with explicit `waiver.until` per `quality/PROTOCOL.md`) — not PASS with a `comment` field that says "do not count failures as RED."
3. **Reclassify `kind` to `mechanical`** since the verifier is deterministic shell. If the ROADMAP wanted a real subagent-graded gate, that's a different verifier (Claude subagent prompt + dispatch.sh).
4. **Cross-check `expected.asserts` (catalog) against `asserts_passed` (artifact) at runner time** so the 9-vs-17 vocabulary mismatch surfaces as a regression.
5. **Have the verifier subagent regrade against the ROADMAP SCs** — not the executor's pivot rationale. If the executor pivoted away from a SC, the verifier should grade that SC RED (and either the phase loops back or the SC is formally retired with intake-file evidence).
6. **Reconcile public-facing docs.** Either (a) qualify the "pure git after init/attach" claim with "on the simulator and on TokenWorld once the substrate ships" until the gate is real, or (b) ship the gate. The current state — public claim unqualified, gate not real — is the failure CLUSTER G names.
7. **File P86 SURPRISES-INTAKE entries retroactively** for the dropped SC #3 + SC #5 so the +2 reservation honesty practice has the evidence trail OP-8 expects, and so P80's RESOLVED close (which depended on P86 covering the end-to-end shape) gets re-graded honestly.
