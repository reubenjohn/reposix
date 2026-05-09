# Phase P80 Audit — Mirror-lag refs (`refs/mirrors/<sot>-{head,synced-at}`)

**Auditor:** unbiased subagent (zero session context)
**Date:** 2026-05-08

## Verdict at a glance
- ALIGNED items: 4
- MISALIGNED items: 7
- SUSPECT items: 1

## Aligned (for context, not findings)

- Cache-side helpers `Cache::write_mirror_head` / `Cache::write_mirror_synced_at` / `Cache::read_mirror_synced_at` / `Cache::log_mirror_sync_written` exist and are unit-tested at `crates/reposix-cache/src/mirror_refs.rs:97-300` (with a `#[cfg(test)]` block at lines 302–371).
- `handle_export` success branch at `crates/reposix-remote/src/main.rs:398-419` invokes `write_mirror_synced_at` + `log_mirror_sync_written`; the post-P81 `write_loop::apply_writes` at `crates/reposix-remote/src/write_loop.rs:282-300` invokes `refresh_for_mirror_head` + `write_mirror_head` (the wiring is split across two source files but functionally complete on the single-backend push path).
- The annotated tag is real (object-kind-Tag, parsed by `git cat-file -p`) and the message body is `mirror synced at <RFC3339>` per writer at `mirror_refs.rs:155-216` and integration test assertion at `tests/mirror_refs.rs:191-217`.
- OP-3 audit row (`op = 'mirror_sync_written'`) is unconditional on the SoT-OK branch (`main.rs:408`); audit-completeness is asserted by the `write_on_success_updates_both_refs` test.

## Findings

### F1 — Workflow template falsely asserts `reposix init` populates `refs/mirrors/*`; it does not [SEVERITY: HIGH]
**Claim in plan:** The reference GH-Action workflow that owners are told to install on the mirror repo says, verbatim: *"Post-init the cache has refs/mirrors/confluence-{head,synced-at} populated (P80 ships these on init). Working tree at /tmp/sot is a partial-clone checkout from the cache."*
**Reality:** P80 wires mirror-ref writes ONLY into the helper push path (`handle_export` success → `write_loop::apply_writes`). `reposix init` shells out to `git init`, configures `extensions.partialClone`, and runs `git fetch --filter=blob:none origin` — none of those code paths call `write_mirror_head` / `write_mirror_synced_at`. `Cache::build_from()` (the cache's tree-builder, called during the helper's first connect) also does not write the refs. A post-init working tree therefore has empty `refs/mirrors/*` until at least one push lands. The workflow's `git push mirror "refs/mirrors/confluence-head" "refs/mirrors/confluence-synced-at"` then fails with "src refspec ... does not match any" on the very first cron tick of a fresh mirror — and the trailing `|| echo "warn: ..."` swallows it silently.
**Evidence:**
- Wrong claim: `docs/guides/dvcs-mirror-setup-template.yml:69-71` (also duplicated by P84 plan at `.planning/phases/84-webhook-mirror-sync/84-01-PLAN/T02-workflow-yaml.md:117-118`).
- Push command that depends on the false assumption: `docs/guides/dvcs-mirror-setup-template.yml:102-104`.
- Init implementation lacking the writes: `crates/reposix-cli/src/init.rs:162-237` (no `write_mirror_*` call).
- Cache build path lacking the writes: `crates/reposix-cache/src/builder.rs:56` (`build_from`) does not invoke `write_mirror_head`; only call sites are `write_loop.rs:288` and `bus_handler.rs:283` (both push-path).
**Why it matters:** This is the documented owner-facing flow for the entire DVCS observability story. A first-cron-tick of a fresh mirror produces refs/heads/main but silently no refs/mirrors/* — exactly the scenario the milestone was supposed to make observable, broken on day one.

### F2 — Catalog row `mirror-refs-readable-by-vanilla-fetch` does not exercise vanilla `git fetch` [SEVERITY: HIGH]
**Claim in plan:** Catalog row description states *"vanilla-git readers can observe mirror lag without any reposix awareness (proves stateless-connect advertises mirror refs)"* (`quality/catalogs/agent-ux.json:130-133`). The verifier-shell module-doc says *"the dark-factory contract is 'agents who want mirror-lag refs can pull them with vanilla git'"* (`quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh:13-20`).
**Reality:** The integration test `vanilla_fetch_brings_mirror_refs` runs `git clone --mirror <local-cache-bare-repo-path>` (`tests/mirror_refs.rs:275-283`), which hits the cache's bare repo via a local file path. It never traverses the helper, never speaks `stateless-connect`, never exercises an `upload-pack --advertise-refs` over stdio. The test even contains an explicit comment-out at lines 322-331 acknowledging the gap: *"Skip stricter advertisement assertion to keep the test boundary at 'agent can observe via plain git' rather than re-asserting protocol-v2 internals."* The thing the catalog claim says is being proven — that `stateless-connect` advertises mirror refs — is the precise thing skipped.
**Evidence:**
- Catalog: `quality/catalogs/agent-ux.json:116-150` (sources include `stateless_connect.rs` (ref advertisement); description claims advertisement is proved).
- Test that doesn't exercise the helper: `crates/reposix-remote/tests/mirror_refs.rs:247-332` (uses `clone --mirror` of a local path).
- Self-acknowledged gap inside the test: `tests/mirror_refs.rs:322-331`.
**Why it matters:** This is failure pattern #1 from the AUDIT-BRIEF (CLUSTER A — test name promises one thing, assertions deliver less). A future regression that breaks `stateless-connect`'s ref advertisement would not flip this row red. The dark-factory claim "agents do `git fetch` and see refs/mirrors/*" is structurally untested.

### F3 — No real-backend coverage for any of the three DVCS-MIRROR-REFS rows [SEVERITY: HIGH]
**Claim in plan:** Catalog rows are tagged `kind: mechanical` only; PLAN-CHECK chapter "Test fixture strategy" promises *"option (b) Real GH mirror reubenjohn/reposix-tokenworld-mirror — `#[ignore]`-tagged smoke at milestone-close"* (`80-RESEARCH/chapter-testing-and-constraints.md` table at line 9-13).
**Reality:** Zero tests reference real Confluence / GitHub / JIRA backends for mirror-refs. `grep -rn "mirror_refs\|refs/mirrors\|mirror_sync_written" crates/reposix-cli/tests/agent_flow_real.rs` returns nothing. The "ignored smoke" never landed; nothing is tagged `pre-release` cadence either. All four integration tests use `wiremock::MockServer` against a local axum-style mock at the SoT URL.
**Evidence:**
- Real-backend test file: `crates/reposix-cli/tests/agent_flow_real.rs` — no mirror-refs hits.
- All P80 catalog rows have `cadences: ["pre-pr"]` and `kind: mechanical` (`quality/catalogs/agent-ux.json:110-114, 148-150, 185-188`); none `pre-release`, none `subagent-graded`.
- Promised but absent: the "milestone-close real GH mirror smoke" from `80-RESEARCH/chapter-testing-and-constraints.md` line 12.
**Why it matters:** The milestone's whole vision is "Confluence is SoT, GitHub mirror is the universal-read surface" — yet the observability primitive (mirror-lag refs) has been verified only against a wiremock that always responds 200. This matches CLUSTER E from the dark-factory exercise (transport-layer claim depending on real-backend behaviour with simulator-only coverage).

### F4 — Verifier-shell shape changed away from the dark-factory contract; SURPRISES journal closed prematurely [SEVERITY: MED]
**Claim in plan:** PLAN-OVERVIEW chapter-2 + 80-PLAN chapter T01 explicitly require `reposix init` → `git fetch` / `git push` end-to-end shells (`80-01-PLAN/03-T01-catalog-first.md`, `03b-T01-verifier-shells.md`). The whole point of the dark-factory shape is that the verifier exercises the agent-UX surface — `reposix init` → vanilla git → assertions in pure shell.
**Reality:** All three shipped verifier shells are 32-line `cargo test ... --test mirror_refs <name>` thin wrappers (`quality/gates/agent-ux/mirror-refs-write-on-success.sh:28`, `mirror-refs-readable-by-vanilla-fetch.sh:28`, `mirror-refs-cited-in-reject-hint.sh:30`). They invoke no `reposix` CLI and no `git` plumbing — they delegate entirely to the in-tree integration tests. The agent-UX surface (`reposix init` → `git fetch` → vanilla `git for-each-ref refs/mirrors/`) is not exercised anywhere.
**Evidence:**
- Shipped shells (all three are cargo-test wrappers): `quality/gates/agent-ux/mirror-refs-{write-on-success,readable-by-vanilla-fetch,cited-in-reject-hint}.sh`.
- The verdict caught the deviation (`quality/reports/verdicts/p80/VERDICT.md:152` "verifier-shell shape change... bypasses the dark-factory contract layer") and flagged it as advisory.
- Filed as `quality/SURPRISES.md:28-36` (2026-05-01 P80 entry) and closed RESOLVED on the strength of P86 having "the same pattern" — but P86's third arm is itself a separate dark-factory regression that does not retroactively re-cover P80's claim about `refs/mirrors/*` propagating through `reposix init` + `git fetch`.
**Why it matters:** This is failure-pattern #6 (velocity-as-skip-signal): the planned shape was deliberately abandoned because it kept hitting `fatal: could not read ref refs/reposix/main`, but the closure rationale ("P86 is the same pattern") does not actually substitute for the missing P80 surface. P88 GOOD-TO-HAVE was suggested but appears not to have landed (verified below).

### F5 — Promised P88 GOOD-TO-HAVE for dark-factory widening to refs/mirrors/* did not materialize [SEVERITY: MED]
**Claim in plan:** Verdict advisory item 3 (`quality/reports/verdicts/p80/VERDICT.md:243-258`) commits to *"P88 good-to-have widening of the dark-factory regression to cover refs/mirrors/* propagation through reposix init"*. The SURPRISES journal entry's RESOLVED rationale also says *"P88 may add explicit naming in CLAUDE.md ... as a GOOD-TO-HAVE"*.
**Reality:** No GOOD-TO-HAVES.md row exists for "widen dark-factory to cover refs/mirrors/*". P86 verdict ships GREEN with the third arm scoped to bus-URL composition + cache audit, NOT to reading `refs/mirrors/*` after `reposix init` + `git fetch`. The CLAUDE.md update committed in d50533d does describe the namespace but does not codify the cargo-test verifier kind.
**Evidence:**
- Closure note: `quality/SURPRISES.md:36` ("RESOLVED ... P88 may add explicit naming").
- Verdict advisory item: `quality/reports/verdicts/p80/VERDICT.md:255-258`.
- `dark-factory.sh dvcs-third-arm` (P86 arm) at `quality/gates/agent-ux/dark-factory.sh` does not assert refs/mirrors/* visibility from the working tree; no follow-up phase added it.
**Why it matters:** Failure pattern #3 (plan promises N, ship delivers N-K). The P80 closure was contingent on a future widening that never landed. The hole flagged in F4 is therefore still open.

### F6 — Reflog growth deferral is not on any v0.14.0 backlog row [SEVERITY: LOW]
**Claim in plan:** `crates/reposix-cache/src/mirror_refs.rs:42-47` cites the v0.14.0 vision doc as the deferral target. PLAN-CHECK and the verdict both list this as accepted.
**Reality:** The cited target file (`.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md`) does not appear to surface "reflog growth on long-lived caches" as a tracked item — the deferral pointer leads to a vision file that does not enumerate it. SUSPECT — would need a grep across the v0.14.0 research dir to confirm; if the item is implicit, future agents will not find it.
**Evidence:** `crates/reposix-cache/src/mirror_refs.rs:43-47`.
**Why it matters:** Future operational concern (DoS-disk on long-lived caches) currently has only an in-source comment; if the v0.14.0 vision doc does not enumerate it, the deferral is lossy.

### F7 — Honesty of the H3 fix is real, but `reject_hint_first_push_omits_synced_at_line` substring "synced from" is the wrong negation token [SEVERITY: LOW]
**Claim in plan:** Verdict (lines 86-99) lauds the H3 fix as substantive: assertion 1 enters the conflict branch; assertions 2/3 then assert absence of "synced from" / "minutes ago".
**Reality:** The hint string at `crates/reposix-remote/src/write_loop.rs:194-195` is *"hint: your origin (GH mirror) was last synced from {sot} at ..."*. The negation `!stderr.contains("synced from")` correctly matches that token. So `synced from` is present in the populated branch and absent in the None branch. PASS — the test is correct as written. (Initial concern dismissed; pinning here so future readers don't re-flag.)
**Evidence:** `crates/reposix-remote/src/write_loop.rs:194-195`; `crates/reposix-remote/tests/mirror_refs.rs:484-491`.
**Why it matters:** Locks in the H3 fix as legitimately substantive; not a finding, kept here for trail-completeness.

### F8 — Bus push does not read mirror-head ref or update both refs atomically; Q2.3 claim drifted [SEVERITY: MED]
**Claim in plan:** Architecture sketch Q2.3 says *"bus updates both refs (consistency over optimization)"*; ROADMAP P80 entry says *"Bus push will update both refs (P83)"*.
**Reality:** Bus-write fan-out at `crates/reposix-remote/src/bus_handler.rs:280-289` calls `write_mirror_synced_at` + `log_mirror_sync_written` — but does NOT call `write_mirror_head` or `refresh_for_mirror_head` from the bus path. Only the single-backend `write_loop::apply_writes` at `write_loop.rs:282-300` writes `refs/mirrors/<sot>-head`. On a bus push, only the synced-at tag is updated; the head ref pointer never advances on bus pushes. Q2.3 "bus updates both refs" is partially false.
**Evidence:**
- Bus path mirror-ref calls: `crates/reposix-remote/src/bus_handler.rs:283-289` (synced-at + audit only; no head-ref write).
- Single-backend path: `crates/reposix-remote/src/write_loop.rs:285-300` (writes head ref).
- Architecture-sketch wording: ROADMAP entry "Phase 80" line 37 *"Q2.3: bus updates both refs (consistency over optimization)"*.
**Why it matters:** Mirror-lag observability for bus-using owners (the documented v0.13.0 happy path) is half-broken: synced-at advances but head ref never does. A vanilla-git mirror-only consumer reading `git log refs/mirrors/<sot>-head` sees a frozen SHA after the first push. Strictly a P83 wiring miss but the contract was set in P80.

### F9 — Catalog rows have no `last_verified` freshness TTL; no re-grade signal if behavior drifts [SEVERITY: LOW]
**Claim in plan:** Standard for `mechanical` rows is no TTL.
**Reality:** All three rows have `freshness_ttl: null`. Row stays PASS until something fails. That's by design for mechanical rows but couples PASS to "we ran cargo test once on 2026-05-01"; if the cargo test target itself silently degrades or gets `#[ignore]`-tagged, the row never goes stale.
**Evidence:** `quality/catalogs/agent-ux.json:107, 144, 181`.
**Why it matters:** Low — every PR triggers cargo test in CI, so silent degradation is unlikely. Noted only because the audit brief asks about freshness signal as part of failure pattern #2 ("substrate-gap deferrals masquerading as GREEN").

### F10 — `cadences: ["pre-pr"]` rows are NOT enforced by any CI workflow [SEVERITY: MED]
**Claim in plan:** PLAN-OVERVIEW assumes `--cadence pre-pr` runs in CI and grades the rows on every PR.
**Reality:** Searching `.github/workflows/` for `pre-pr` returns zero matches; `pre-push`, `pre-release`, `weekly`, `post-release` are all wired but `pre-pr` is not. The `ci.yml` workflow runs `cargo test --workspace --locked`, which DOES execute the integration tests transitively, so the assertions run — but the runner verdict (which determines whether the catalog row is PASS) is never re-graded. A row marked PASS on 2026-05-01 stays PASS even if the underlying test starts failing in CI, because nothing flips the JSON.
**Evidence:**
- Workflows: `.github/workflows/ci.yml` (no `cadence` flag), `quality-pre-release.yml`, `quality-weekly.yml`, `quality-post-release.yml`. No `quality-pre-pr.yml`.
- Runner runs by cadence: `quality/runners/run.py:46` (`pre-pr` is a recognized cadence) but no callsite invokes it in CI.
**Why it matters:** Failure pattern #4 (project-invariant violated silently). The "pre-pr" cadence catalog rows are effectively only graded by the local pre-push hook IF a developer runs `python3 quality/runners/run.py --cadence pre-pr` manually. The catalog system suggests these rows are CI-graded; they are not.

### F11 — `vanilla_fetch_brings_mirror_refs` ignores the protocol-v2 advertisement check it set out to make [SEVERITY: MED]
**Claim in plan:** The test at `tests/mirror_refs.rs:307-321` actually runs `git upload-pack --advertise-refs --stateless-rpc` against the cache's bare repo and stores the output in `adv_str`.
**Reality:** Line 331: `let _ = adv_str;` — the captured advertisement output is discarded with no assertion. The test goes through the motion of the protocol-v2 check and then explicitly throws it away. No grep, no contains-check, no assertion that the advertisement names `refs/mirrors/*`.
**Evidence:** `crates/reposix-remote/tests/mirror_refs.rs:312-331`.
**Why it matters:** This is the second instance of CLUSTER A in this phase: a chunk of test plumbing exists exactly to assert the contract that the catalog row claims (advertisement contains the refs), but the assertion is `let _ = ...`. A reviewer skimming the test sees the upload-pack call and assumes coverage; there is none.

### F12 — REQUIREMENTS / STATE / SUMMARY post-hoc commit was filed but the verdict's three advisories are not all closed [SEVERITY: LOW]
**Claim in plan:** Verdict advisories 1 (80-01-SUMMARY), 2 (REQUIREMENTS + STATE flips), 3 (SURPRISES journal).
**Reality:** Commit `b6888f7` (`phase-close(P80)`) is on origin/main and contains 80-01-SUMMARY.md (read at `.planning/phases/80-mirror-lag-refs/80-01-SUMMARY.md`). SURPRISES journal entry exists at `quality/SURPRISES.md:28-36`. Advisories 1 and 3 closed. Advisory 2 (REQUIREMENTS.md flips + STATE.md cursor) — could not verify in this audit; the SUMMARY claims it landed but I did not read REQUIREMENTS.md / STATE.md directly. SUSPECT.
**Evidence:** `.planning/phases/80-mirror-lag-refs/80-01-SUMMARY.md` (exists), `quality/SURPRISES.md:28-36` (exists).
**Why it matters:** Bookkeeping; minor. Worth confirming during the v0.13.1 framework-fix phase.

## Summary

The phase has solid in-tree implementation (mirror_refs.rs is well-engineered, the H3 vacuous-test risk is genuinely closed, OP-3 audit is unconditional). The misalignments cluster around **what is verified** rather than **what was built**:

1. **F1 (HIGH)** — Owner-facing workflow template makes a verifiably false claim about init-time ref population. First-cron-tick of a fresh mirror silently breaks observability.
2. **F2 + F11 (HIGH/MED)** — Two distinct test surfaces named for vanilla-fetch / advertisement coverage skip the actual stateless-connect / advertisement assertions.
3. **F3 (HIGH)** — Zero real-backend coverage; transport-layer claim verified only against wiremock.
4. **F4 + F5 (MED)** — Verifier-shell shape was changed away from the dark-factory contract; the closure path (P88 GOOD-TO-HAVE) did not land.
5. **F8 (MED)** — Bus push does not advance `refs/mirrors/<sot>-head`; the cross-phase contract drifted between P80 and P83.
6. **F10 (MED)** — `pre-pr` cadence rows are not enforced by any CI workflow; PASS state is sticky.

If P80 is graded against the milestone vision ("vanilla git readers observe mirror lag without reposix"), F1 and F2 are load-bearing failures; the documented user flow does not actually work, and the test that's supposed to prove the underlying mechanism does not exercise it. The verdict's GREEN grade is consistent with the catalog-row-only contract but inconsistent with the milestone-level vision.
