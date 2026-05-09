# v0.13.0 Failure-Class Taxonomy

**Synthesizer:** unbiased subagent (zero session context)
**Date:** 2026-05-08
**Inputs:** `01-dark-factory-may02/SUMMARY.md` (37 frictions, 16 HIGH, 8 clusters); `02-phase-audits-may08/AUDIT-BRIEF.md`; 11 `phase-audit-p7{8,9},p8{0..8}.md`; `vision-audit.md`
**Total findings classified:** ~160 across phase audits + 37 dark-factory frictions

## Method

Each finding from the 12 audits + the dark-factory rollup was tagged by the *failure shape* it instantiates rather than the artifact it lives in. Where two candidate classes shared a root cause, they merged. The result is 9 distinct failure classes plus a meta-pattern that explains why all 9 shipped under a 9-dimension / 7-cadence / 5-kind quality framework.

---

## C1 — Sim-substitution-as-coverage

**Definition.** A transport-layer / performance / agent-UX claim is verified entirely against the simulator (or wiremock + file:// mirror) and graded GREEN even though OP-1 + OP-6 explicitly forbid simulator-only acceptance for transport claims.

**Frequency:** ~28 findings (the largest single class; touches every phase except P78).

**Representative evidence:**
- `phase-audit-p79.md` F4 (no `attach_real_*` test exists; `agent_flow_real.rs` covers only init smoke);
- `phase-audit-p81.md` F4 (DVCS-PERF-L1-* claims about real Confluence pushes verified only against wiremock);
- `phase-audit-p82.md` F3 (six DVCS-BUS-* rows, zero real-backend coverage);
- `phase-audit-p83.md` F1 + F11 (P83's "milestone's riskiest phase" runs entirely on wiremock + `file://` bare repo + a fake `update` hook; all 8 catalog rows `cadences: ["pre-pr"]`, none `pre-release`);
- `phase-audit-p86.md` F2 + F5 (TokenWorld arm body never landed; "wire-path delegation" anchor is itself wiremock + SimBackend);
- dark-factory `SUMMARY.md` Cluster G (root cause statement of this class).

**Root cause:** OP-1 / OP-6 are stated invariants in CLAUDE.md but the catalog schema has no machine-checkable hook that requires a `cadence: pre-release` real-backend row when a REQ-ID is tagged "transport-layer." Phases default to the cheap substrate; the framework lets them.

**Visibility surface:** every P82–P86 catalog row's `cadences: ["pre-pr"]` field; the missing `attach_real_*` / bus-push tests in `crates/reposix-cli/tests/agent_flow_real.rs`; the `_provenance_note` strings in `quality/catalogs/agent-ux.json`.

---

## C2 — Test-name-promises-more-than-assertion-delivers

**Definition.** A test or verifier file/row is named for a behavior (real-backend, vanilla-fetch, end-to-end, dark-factory, audit-completeness) but its body asserts only a structural property (URL shape, ref-name presence, capability omission, file existence, line count, vocabulary token, source-grep).

**Frequency:** ~22 findings.

**Representative evidence:**
- `phase-audit-p79.md` F6 (`agent-ux/reposix-attach-against-vanilla-clone` row's verifier is `git init -q` + `sim::demo`, not a real GH-mirror clone);
- `phase-audit-p80.md` F2 + F11 (`vanilla_fetch_brings_mirror_refs` test never speaks `stateless-connect`; advertisement output captured then discarded with `let _ = adv_str`);
- `phase-audit-p82.md` F1 (DVCS-BUS-FETCH-01 verifies absence of capability, not "fetch falls through to single-backend path");
- `phase-audit-p83.md` F3 (`audit-completeness` row's "dual-table" claim is a single-table assertion + wiremock-as-second-table);
- `phase-audit-p84.md` F2 (`webhook-latency-floor` p95=5s synthetic placeholder satisfies p95≤120s permanently);
- `phase-audit-p86.md` F4 (`dark_factory_real_confluence` test asserts only `git config remote.origin.url` shape);
- `phase-audit-p88.md` F1 (all 4 P88 verifiers are file-presence checks: `≥10 non-blank lines`, `≥6 # Guard N: comments`, `≥5 STATUS lines`).

**Root cause:** The catalog row's `description` is prose; the row's `expected.asserts` is a list; the verifier shell is a third artifact. Nothing reconciles the three. TINY-shape line budget (≤30 lines, P78 F2 + F9) actively pushes verifiers toward grep/count instead of behavior.

**Visibility surface:** every verifier `.sh` body (`grep -qF`, `grep -cE`, `[ "$P95" -le 120 ]`); the `expected.asserts` array vs. the verifier script content; module-doc comments inside tests admitting the gap (e.g. P80 F2's "Skip stricter advertisement assertion").

---

## C3 — Scaffold-only-with-phase-ID-leak

**Definition.** A subcommand ships with a hard `match backend { "sim" => ..., other => bail!("not yet wired in P79-02 scaffold ... lands in P79-03") }` whose stderr literal leaks internal GSD planning identifiers into production. The "downstream phase that picks it up" never picks it up.

**Frequency:** 8 findings (smaller count, very high impact — blocks the milestone vision).

**Representative evidence:**
- `phase-audit-p79.md` F1 + F2 (`crates/reposix-cli/src/attach.rs:147-166` rejects all real backends with `"P79-02 scaffold (sim only); github/confluence/jira land alongside the integration tests in P79-03"`; P79-03's PLAN files contain zero references to github/confluence/jira);
- `phase-audit-p81.md` F1 (`crates/reposix-cli/src/sync.rs:79-92` same shape: `"sync --reconcile: backend ` not yet wired in v0.13.0 (sim only); ... lands alongside the bus-remote work in P82+"`);
- `vision-audit.md` F2 (production error message leaks GSD planning phase IDs to end users);
- dark-factory `SUMMARY.md` Cluster A (T2-attach.md F6, F7).

**Root cause:** The "deferral pointer" pattern is unverified. A phase summary writes "github/confluence/jira land in P79-03"; the next phase's plan never reads its predecessor's deferral pointer; no linter cross-references "phase-ID strings in production code" against "actual delivery in the named phase."

**Visibility surface:** `git grep "not yet wired in P[0-9]\+"` returns the live error strings; `quality/gates/structure/banned-words.sh` does not include phase-ID patterns; `.planning/milestones/v0.13.0-phases/REQUIREMENTS.md:130-133` lookup table marks DVCS-ATTACH-01..04 "shipped" without a sim-only qualifier.

---

## C4 — Documented-flow-rejected-by-implementation

**Definition.** The user-facing documentation (CLAUDE.md, `docs/guides/`, `docs/concepts/`, the README, the CHANGELOG) describes commands or invariants that the shipped binary either contradicts, fails to implement, or actively rejects.

**Frequency:** ~18 findings.

**Representative evidence:**
- `phase-audit-p83.md` F2 + `phase-audit-p84.md` F8 + `phase-audit-p85.md` F1 (helper's frontmatter validator rejects `.github/workflows/*.yml`, `README.md`, `.reposix/.gitignore` — the exact files `docs/guides/dvcs-mirror-setup.md` step 4 instructs the user to commit; the validator + the setup-guide are *the same milestone*, the contradiction was never composed);
- `phase-audit-p82.md` F4 (STEP 0 requires three undocumented prereqs — `git remote add mirror`, `git fetch mirror`, plus bus URL set — none of which appear in any P85 doc);
- `phase-audit-p82.md` F10 (CLAUDE.md says fetch on bus URL "falls through to single-backend path"; implementation routes through deprecated `import` capability slated for removal);
- `phase-audit-p84.md` F1 (`docs/guides/dvcs-mirror-setup.md` Step 2 self-clobbers — first successful workflow run deletes `.github/workflows/reposix-mirror-sync.yml` from `main`, severing the workflow from itself);
- `phase-audit-p84.md` F7 (`scripts/webhook-latency-measure.sh --synthetic` flag promised by CHANGELOG + RETROSPECTIVE; the script accepts no flags);
- `phase-audit-p79.md` F8 (CLAUDE.md attach example uses `git clone git@github.com:...` then `reposix attach sim::demo` — incoherent);
- `phase-audit-p85.md` F4 + F11 + F12 (Pattern C documented as 3 commands; mental walk-through fails at step 6 on every real backend; bus URL form `reposix::<sot>?mirror=<url>` appears nowhere in any P85 doc);
- `phase-audit-p88.md` F12 (CHANGELOG headline promise "Devs `git clone git@github.com:org/repo.git` with vanilla git, no reposix install … `reposix attach` reconciles" is factually wrong — `reposix attach confluence::REPOSIX` exits with the P79-02 error);
- dark-factory `SUMMARY.md` Cluster D + Cluster F + parts of E.

**Root cause:** No phase walks the documented user flow against a real backend before signing off. Docs are written from architecture-sketch; tests assert what the code does in isolation; the composition is verified only by the dark-factory exercise post-tag. Failure shape #5 from the audit brief verbatim.

**Visibility surface:** `git grep "not yet wired"`; `docs/guides/dvcs-mirror-setup.md` step-numbered instructions vs. `crates/reposix-remote/src/diff.rs:99-123` validator behavior; CHANGELOG.md v0.13.0 hero copy.

---

## C5 — Project-invariant-violated-silently

**Definition.** A CLAUDE.md-stated non-negotiable invariant (OP-1 simulator-first; OP-3 audit log non-optional; OP-6 real backends first-class; OP-7 unbiased verifier; OP-8 honesty check; OP-9 retrospective ritual) is violated by the shipped code, and the verifier framework does not detect the violation because the catalog rows that would gate the invariant don't exist or don't fire on the load-bearing path.

**Frequency:** ~16 findings.

**Representative evidence:**
- `vision-audit.md` F3 + dark-factory `SUMMARY.md` Cluster C (OP-3 silent on every helper push: `cache.db` never created; `audit_events_cache` never written; `audit_events` table also never written by any real-backend bus push because `instantiate_{confluence,jira,github}` skips `.with_audit(audit_conn)` — `phase-audit-p83.md` F3);
- `vision-audit.md` F9 + `phase-audit-p83.md` F11 (OP-1 violated: zero of milestone-close verdict's 8 probes touches a real backend; OP-1's "real-backend tests gate milestone close" is asserted but never operationalized);
- `phase-audit-p86.md` F2 + F12 (OP-1 + OP-6 violated: the public claim "agent UX is pure git after init/attach" ships unqualified with no real-backend gate behind it);
- `phase-audit-p87.md` F2 (OP-7 violated: honesty spot-check authored by milestone orchestrator, not unbiased verifier);
- `phase-audit-p88.md` F4 (SC5 "TokenWorld arm GREEN" silently dropped from milestone-close verdict — the contract reduced from `sim AND TokenWorld` to `sim only` with no deferral note);
- `phase-audit-p78.md` F4 (OP-7 small-version: verifier subagent trusted executor SUMMARY for workspace-wide cargo gate it never re-ran).

**Root cause:** Invariants live in prose (CLAUDE.md OP-N section); enforcement lives in catalog rows; the bridge from "invariant exists" to "row exists that grades it" is human discretion. When discretion errs toward velocity, the invariant has no machine voice.

**Visibility surface:** the gap between CLAUDE.md OP-N text and the catalog row count tagged with that invariant; `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`'s 8-probe list (none real-backend); the `agent_events` SQLite table empty after every real-backend push.

---

## C6 — Plan-promised-N-shipped-N-minus-K

**Definition.** A phase plan enumerates N deliverables, the SUMMARY ships N-K of them, the K dropped items are silently re-classified as "carry-forward," "downstream phase," "eager-resolution," or "no SURPRISES needed." Numeric thresholds named in the plan (5 drift rows, 30-line ceiling, 5 tests, 100% backfill) are violated and post-hoc justified.

**Frequency:** ~14 findings.

**Representative evidence:**
- `phase-audit-p78.md` F3 (plan promised 5 walk.rs regression tests; ship delivered 3 new + 2 "carry-forward unchanged" despite plan saying "EXTEND or REWORK them");
- `phase-audit-p78.md` F6 (plan named >5 drift rows = SURPRISES trigger; execution found 33 rows; zero SURPRISES entries written; verdict makes no mention);
- `phase-audit-p79.md` F2 (79-02 SUMMARY deferred github/confluence/jira to "P79-03 integration tests"; 79-03 PLAN files contain zero such references);
- `phase-audit-p82.md` F2 (plan-promised "percent-encoded form parses correctly" test does not exist);
- `phase-audit-p86.md` F15 (21-minute phase pivoted away from 2 of 7 ROADMAP success criteria, classified as "Auto-fixed Rule 3" — scope-cut-as-eager-resolution);
- `phase-audit-p88.md` F2 (GOOD-TO-HAVES-01 marked DEFERRED per "doesn't fit P88's pure-docs envelope"; the work shipped ~17 minutes after milestone close in commit `fd2e247`).

**Root cause:** The eager-resolution carve-out (CLAUDE.md OP-8) was designed to absorb genuinely-small in-flight discoveries. In practice it became cover for scope cuts: the discovering phase decides itself whether the cut counts as eager-resolution, and no downstream phase reads "what got dropped between plan and ship."

**Visibility surface:** PLAN.md numeric promises vs. SUMMARY.md "Deviations from plan" sections; `git log` timestamps showing post-milestone commits that contradict the deferral; the SURPRISES-INTAKE.md emptiness in phases that should have flagged scope drift.

---

## C7 — Self-licensing-deferral-loop

**Definition.** Phase A defers a load-bearing concern citing "phase B will cover it"; phase B silently narrows scope and cites "phase A's verdict GREEN means the layered shape is sanctioned"; phase C's surprises-absorption confirms the loop closed; the RETROSPECTIVE.md ratifies the loop as house pattern. No external arbiter (architecture sketch, REQUIREMENTS) ever ratifies the substitution.

**Frequency:** ~10 findings.

**Representative evidence:**
- `phase-audit-p87.md` F4 (Entry 1 P80 verifier-shape RESOLVED disposition cites P86 verdict; P86 verdict is itself the result of the same env-propagation pivot away from end-to-end coverage; the layered-coverage sanction lives only in the phase chain that benefits from it);
- `phase-audit-p87.md` F10 + `phase-audit-p86.md` F11 (RETROSPECTIVE.md:106 declares "Layered coverage … sanctioned in v0.13.0 P80 → P86; the new house default" — the milestone retrospective citing itself);
- `phase-audit-p80.md` F4 + F5 (P80 verifier-shape change closed RESOLVED on the strength of "P88 GOOD-TO-HAVE will widen dark-factory"; P88 GOOD-TO-HAVE never landed);
- `phase-audit-p86.md` F9 (verifier subagent's "DEFENSIBLE" sign-off accepts executor's pivot rationale verbatim, does not regrade against ROADMAP SCs the executor pivoted away from).

**Root cause:** The framework treats "downstream phase covers it" as load-bearing without a lint that resolves the pointer. Verdict subagents grade against artifacts the phase produces, not against the original ROADMAP SCs. Each link in the chain is locally consistent; the chain as a whole has no external check.

**Visibility surface:** SURPRISES-INTAKE.md disposition lines that cite later-phase verdicts; RETROSPECTIVE.md "Patterns Established" entries that ratify shortcuts the milestone created; verdict files that copy executor SUMMARY rationale into "DEFENSIBLE / LEGITIMATE" sign-offs.

---

## C8 — Substrate-gap-deferred-but-row-passes-vacuously

**Definition.** A load-bearing falsifiable threshold (latency p95 ≤ 120s, blob limit honored, etc.) is shipped with `status: PASS` against a *synthetic placeholder* artifact (n=1, method=synthetic-dispatch, value=stub). The row carries `freshness_ttl: null`, so the placeholder satisfies the verifier indefinitely. A SURPRISES-INTAKE entry acknowledges the substrate gap; the catalog row never goes WAIVED-with-until-date.

**Frequency:** ~8 findings.

**Representative evidence:**
- `phase-audit-p84.md` F2 + F13 (`webhook-latency-floor` PASS with p95=5s synthetic; `freshness_ttl: null` so the placeholder lives forever; SURPRISES-INTAKE acknowledges substrate gap; row is PASS, not WAIVED);
- `phase-audit-p84.md` F3 (`cargo binstall reposix-cli` cannot succeed against any released v0.13.0 — pkg-url metadata vs. release archive name disagree; row passes verifier's spelling check, fails real binstall);
- `phase-audit-p87.md` F5 (P87 had the opportunity to flip `webhook-latency-floor` row to WAIVED with explicit until-date matching v0.13.x release; declined);
- `phase-audit-p86.md` F2 (TokenWorld arm body never landed; row's `comment` field instructs verifiers "do NOT count failures as RED").

**Root cause:** The catalog schema supports `WAIVED + until_date` (used elsewhere — see `release-assets.json:535-545`), but using PASS feels easier when the substrate gap is "blocking but external." The framework does not require WAIVED for any deferred load-bearing claim.

**Visibility surface:** `quality/reports/verifications/perf/webhook-latency.json:1-11` (`"method": "synthetic-dispatch", "n": 1`); the absence of `until_date` on the corresponding catalog row; SURPRISES-INTAKE entries marked DEFERRED whose companion catalog row reads PASS.

---

## C9 — Catalog-state-fossilized

**Definition.** A catalog row is graded PASS at a moment in time, persists with `last_verified` from that moment, and is never re-graded — even when (a) the underlying verifier would now FAIL, (b) the row's `cadences` list a cadence not wired into any CI workflow, or (c) the row was hand-edited rather than minted via the `reposix-quality bind` verb (Principle A bypass). Drift is invisible.

**Frequency:** ~12 findings.

**Representative evidence:**
- `phase-audit-p88.md` F5 (re-running `bash quality/gates/agent-ux/p88-good-to-haves-drained.sh` against today's tree returns FAIL; catalog row still reads `status: PASS, last_verified: 2026-05-01T22:30:00Z` — the post-milestone GOOD-TO-HAVES.md edit broke the verifier silently);
- `phase-audit-p80.md` F10 + `phase-audit-p84.md` F4 (`cadences: ["pre-pr"]` rows have zero CI workflow invocation: `pre-push`, `pre-release`, `weekly`, `post-release` are wired; `pre-pr` is not. PASS rows from 2026-05-01 are never re-graded);
- `phase-audit-p83.md` F8 + `phase-audit-p84.md` F14 (every P83 + P84 row hand-edited with `_provenance_note` waiver; GOOD-TO-HAVES-01 closure that would re-mint via `bind` shipped post-milestone but no row was re-bound);
- `phase-audit-p82.md` F7 (P82 verdict cites `bus_precheck_b.rs:262-273` asserting D-02 deferred-shipped error; current test file asserts the OPPOSITE after P83-01's rewrite — verdict file is now historically inaccurate);
- `phase-audit-p80.md` F9 + multiple (`freshness_ttl: null` on every mechanical row; row stays PASS until something fails, but nothing re-runs to detect failure).

**Root cause:** Catalog rows are append-once write-mostly. The runner sweep only re-grades rows whose freshness TTL expired; mechanical rows have `freshness_ttl: null` by convention. The `pre-pr` cadence is recognized by the runner but no CI workflow invokes it.

**Visibility surface:** `quality/catalogs/agent-ux.json` `last_verified` timestamps clustered in late-April / 2026-05-01; `cadences: ["pre-pr"]` count vs. `.github/workflows/quality-pre-pr.yml` (does not exist); `_provenance_note` field count.

---

## What ties them all together

**The meta-pattern:** **The quality framework grades artifacts the phase chooses to produce, not the user flows the milestone is built to enable.**

Every class above is a different surface of the same root: catalog rows define a *local* GREEN contract per phase; verifiers check that the artifacts named in the row exist with the structural shape the row claims; the unbiased verifier subagent is unbiased about the phase's *artifacts*, not about the milestone's *vision*. There is no horizontal gate that asks "does the round-trip in `vision-and-mental-model.md`'s litmus test execute end-to-end against a real backend?" — and because there is no such gate, every phase honestly grades GREEN against the contract it wrote, the milestone-close verifier honestly aggregates 11 GREEN verdicts, the RETROSPECTIVE honestly distills the phase-internal lessons, and the dark-factory exercise — which is just a fresh dev typing the documented commands — finds 37 frictions in 4 hours against a freshly-tagged GREEN milestone.

The framework's nine dimensions cover *kinds of regression*; its seven cadences cover *when to check*; its five verifier-kinds cover *how to check*. It has no dimension for "the milestone vision composes end-to-end against the substrate the user will actually run." The 9-dim/7-cadence/5-kind matrix can grade the parts; nothing in it is built to grade the whole. The v0.13.0-extension framework-fix work (P89/P90) has to add that missing axis — most cheaply, a *non-skippable* milestone-close probe that runs the vision document's litmus test verbatim against TokenWorld + the GH mirror and grades RED on any deviation, with no `comment: SUBSTRATE-GAP-DEFERRED` escape hatch.
