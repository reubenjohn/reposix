# Phase P79 Audit — POC + `reposix attach` core
**Auditor:** unbiased subagent (zero session context)
**Date:** 2026-05-08

## Verdict at a glance
- ALIGNED items: 5
- MISALIGNED items: 7
- SUSPECT items: 1

## Executive summary

P79 was graded GREEN despite shipping a `reposix attach` subcommand whose
"backend connector resolution" explicitly bails on `github`, `confluence`,
and `jira` with a one-line error that **leaks two GSD phase IDs**
(`P79-02 scaffold`, `P79-03`) directly to end users. The verifier did
not catch it because the milestone's REQ-ID acceptance (DVCS-ATTACH-01..04)
was satisfiable entirely against the simulator, and 79-02 SUMMARY
explicitly told the verifier the github/confluence/jira wiring would
"land alongside the integration tests in P79-03" — but 79-03's PLAN
file makes **zero** mention of real-backend wiring. The deferral pointer
was a lie nobody read. The fix-forward is small: `attach.rs` lines
147-166 need to call the same `instantiate_{github,confluence,jira}`
pattern already wired in `crates/reposix-remote/src/backend_dispatch.rs`
(lines 234-271) and `crates/reposix-cli/src/refresh.rs` (lines 174-240).

## Findings

### F1 — `reposix attach` rejects all real backends with a phase-ID-leaking scaffold error [SEVERITY: HIGH]
**Claim in plan:**
- DVCS-ATTACH-01 (`.planning/milestones/v0.13.0-phases/REQUIREMENTS.md:45`):
  *"`reposix attach <backend>::<project>` subcommand in `crates/reposix-cli/`. ...
  REST-lists backend; populates cache OIDs lazily; reconciles by walking
  current HEAD tree..."*
- Vision (`vision-and-mental-model.md` quoted in `architecture-sketch/index.md:5`):
  *"Install reposix only when they want to write back; `reposix attach`
  reconciles their existing checkout against the SoT, then `git push` via
  a bus remote fans out atomically to confluence (SoT-first) and the GH
  mirror."*
- `architecture-sketch/innovations.md:14`: *"reposix attach confluence::SPACE"*
- `docs/concepts/dvcs-topology.md:135` (Pattern C — round-tripper, the
  v0.13.0 thesis path): *"reposix attach confluence::SPACE"*

**Reality:** `crates/reposix-cli/src/attach.rs:147-166` constructs the
`BackendConnector` with a hard match: only `sim` is wired; every other
backend bails with literal text leaking phase IDs to the user:

```rust
other => bail!(
    "attach: backend `{other}` not yet wired in P79-02 scaffold (sim only); \
     github/confluence/jira land alongside the integration tests in P79-03"
),
```

A user who reads `dvcs-topology.md` Pattern C and runs
`reposix attach confluence::SPACE` is stopped cold. Confirmed by
T2-attach.md dark-factory exercise (Step 3, finding F6).

**Evidence:** `crates/reposix-cli/src/attach.rs:147-166` (specifically
the `bail!` at 162-165). Cross-referenced against
`crates/reposix-cli/src/refresh.rs:174-240` (proves all three real-backend
connectors are wired and credential-threaded for the `refresh` subcommand)
and `crates/reposix-remote/src/backend_dispatch.rs:234-271` (proves the
helper has full `instantiate_{github,confluence,jira}` plumbing).

**Why it matters:** This is the canonical doc-vs-reality HIGH bug — every
user-facing doc and the architecture-sketch lead with `confluence::SPACE`
as the prototypical attach invocation; the binary refuses it. Pattern C
is named "the v0.13.0 thesis path" in `dvcs-topology.md:141`. The
milestone's litmus test (`vision-and-mental-model.md` "dual-side
round-trip … vanilla-git clone → attach → edit → bus-push") cannot
execute on the only real backend the project has standing infrastructure
for (TokenWorld). Plus the error leaks internal GSD planning identifiers
that have no meaning to users.

---

### F2 — 79-03 PLAN silently dropped the real-backend wiring 79-02 SUMMARY promised [SEVERITY: HIGH]
**Claim in plan:**
- 79-02 SUMMARY (`.planning/phases/79-poc-reposix-attach-core/79-02-SUMMARY.md:59`):
  *"Backend connector: only `sim` is wired in this scaffold ...
  github/confluence/jira bail with a clear error pointing to 79-03
  (deferred per the option-a build sequence)."*
- 79-02 SUMMARY § "Deviations from plan" (`.planning/phases/79-poc-reposix-attach-core/79-02-SUMMARY.md:144-146`):
  *"This is a **deliberate scope reduction** (the integration tests land
  in 79-03 ... real-backend wiring needs the credential paths threaded
  through, which is out-of-scope for the scaffold)."*

**Reality:** 79-03 PLAN files (`79-03-PLAN/index.md`, `T01.md`, `T02.md`,
`T03.md`) contain **zero references** to github, confluence, jira,
real-backend wiring, or credential plumbing. T01 ships 6 reconciliation
case integration tests (sim-only, fixed `sim::demo` spec). T02 ships
4 idempotency/Tainted/audit tests (sim-only, fixed `sim::demo` /
`sim::project-{a,b}`). T03 ships CLAUDE.md docs + push (no code).
The "pointer to 79-03" baked into the production error message at
`attach.rs:163-164` was followed up on by no plan task and noticed by
no verifier.

Confirmed by:
- `grep -rn "real-backend\|github\|confluence\|jira"
  .planning/phases/79-poc-reposix-attach-core/79-03-PLAN/` — zero hits
  outside test-spec strings like `sim::demo`.
- `git log --oneline 086422e..dd3c801`: the three 79-03 commits
  (`a558d4a`, `791f7b9`, `dd3c801`) modify `attach.rs` only for the
  `REPOSIX_SIM_ORIGIN` env override; they do not touch the
  `match backend` arm.

**Evidence:** Production error string is still in the binary at
`crates/reposix-cli/src/attach.rs:162-165` as of HEAD; `79-03-PLAN/index.md`
shipped 79-03 without addressing it; `79-03-SUMMARY.md` was written
post-hoc after an OS crash (per VERDICT.md advisory item 1) and never
flagged the dropped scope.

**Why it matters:** This is the textbook "Plan promises N items, ship
delivers N-K, K silently dropped" failure shape from the audit brief.
The deferral pointer was load-bearing — the verifier had no other reason
not to flag the sim-only attach as a HIGH gap. The planning system's
own integrity broke: a plan claimed work would land in a downstream plan,
the downstream plan didn't carry the work, and nobody noticed.

---

### F3 — Verifier's GREEN verdict mistook "tests pass" for "feature ships" [SEVERITY: HIGH]
**Claim in plan:** `quality/reports/verdicts/p79/VERDICT.md` grades
all 5 REQ-IDs PASS and concludes:

> *5/5 REQ-IDs PASS. Catalog row PASS. Pre-push gate 26/26 GREEN. ...
> **Verdict: GREEN.** Phase 79 ships.*

**Reality:** The DVCS-ATTACH-01 acceptance text in REQUIREMENTS.md:45
names backends abstractly (`<backend>::<project>`) and the verifier
graded PASS based on:
- `crates/reposix-cli/src/attach.rs` exists, subcommand wired
- `agent-ux/reposix-attach-against-vanilla-clone` catalog row PASS
- Integration test `attach_against_vanilla_clone_sets_partial_clone`
  passes against sim

None of the verifier's 5 REQ-IDs probe whether the implementation
honors the *vision-and-mental-model* it serves: the
"confluence-as-SoT, GH-mirror-as-universal-read-surface" round-trip.
The verifier had **zero session context** by design — it never
saw `architecture-sketch/innovations.md`'s `reposix attach
confluence::SPACE` or `dvcs-topology.md`'s Pattern C. It graded
the row from artifacts and the catalog asserts described only the
sim post-conditions (`extensions.partialClone == "reposix"`,
`remote.reposix.url` starts with `reposix::`).

**Evidence:** `quality/reports/verdicts/p79/VERDICT.md:11-15`
("REQ-ID grading" table — every PASS row cites a sim-backed test
or a catalog row that runs only against sim);
`quality/catalogs/agent-ux.json:43-77` (catalog row asserts +
verifier script `quality/gates/agent-ux/reposix-attach.sh:30-52`
which spawns `reposix-sim --bind 127.0.0.1:7898` and runs
`reposix attach sim::demo` — sim-only by construction).

**Why it matters:** This is failure shape #2 from the audit brief
("substrate gap deferrals masquerading as GREEN"). The verifier
honestly graded the artifacts it was given — but the catalog rows
the phase introduced did NOT exercise the load-bearing claim
(round-trip against a real SoT, which is the entire DVCS thesis).
The framework let the most user-facing claim of v0.13.0 ship without
real-backend evidence. Until the catalog row's `expected.asserts`
list includes a real-backend leg (or the row split into a sim arm
+ a real-backend arm gated by env vars), this exact failure
recurs on every future phase that touches a transport-layer claim.

---

### F4 — `agent_flow_real.rs` covers `reposix init` real-backend smoke but not `reposix attach` [SEVERITY: HIGH]
**Claim in plan:** None explicit — but the `agent_flow_real.rs` file
exists precisely as the project's real-backend gate (per
CLAUDE.md "Real backends are first-class test targets" / OP-6) and
is invoked via `cargo test --test agent_flow_real -- --ignored
dark_factory_real_{github,confluence,jira}`.

**Reality:** `crates/reposix-cli/tests/agent_flow_real.rs:127-189`
contains `dark_factory_real_github`, `dark_factory_real_confluence`,
`dark_factory_real_jira`. **All three are `reposix init` smoke tests.**
None of them exercise `reposix attach`. The closest the file gets
to attach is in module docs (line 1: *"Phase 35 Plan 03 real-backend
integration tests"*) — there is no `attach_real_*` family of tests.

**Evidence:** `crates/reposix-cli/tests/agent_flow_real.rs:125-189`
(only `init` is invoked: see `run_init_and_assert` calls at
lines 130, 156, 175). No symbol named `attach` appears in the
file. Verified via `grep -n "attach"
crates/reposix-cli/tests/agent_flow_real.rs` (zero hits).

**Why it matters:** Even if F1+F2 were fixed (i.e., real-backend
wiring were threaded into `attach.rs`), no test would exercise it.
Real-backend coverage exists for `init`, where it provides a
backstop for spec → URL translation; it does not exist for `attach`,
where the entire reconciliation walk + cache `build_from()` round-trip
needs verification against real REST shapes (frontmatter `id` field
extraction, paginated `list_records`, etc.). This is failure shape
#1 from the audit brief: "Test name promises one thing, assertions
deliver less" — the file is named `agent_flow_real.rs` but only
covers init's URL composition, not the full agent flow.

---

### F5 — `reposix sync --reconcile` carries the same scaffold-only deferral [SEVERITY: MED]
**Claim in plan:** This is a P81 finding, not P79 — but the same
shape pattern propagated. `crates/reposix-cli/src/sync.rs:88-91`:
```rust
other => bail!(
    "sync --reconcile: backend `{other}` not yet wired in v0.13.0 (sim only); \
     github/confluence/jira land alongside the bus-remote work in P82+"
),
```
The CLAUDE.md `reposix sync --reconcile` block (line 178) describes
it as "the L1 escape hatch (v0.13.0+): rebuild the cache from REST
when a push reject suggests cache desync."

**Reality:** Same as F1 — works on sim only; phase ID leaks into
production error.

**Evidence:** `crates/reposix-cli/src/sync.rs:79-92` (the `match
backend_slug.as_str()` arm).

**Why it matters:** The cache-desync recovery move CLAUDE.md
documents (`reposix sync --reconcile`) is unavailable on real
backends — the same backends users would actually run it on
(TokenWorld, GitHub, JIRA). Same root cause as F1: the
"P82+ wiring" deferral pointer was never followed up on. P81's
audit will probably surface this independently; flagging here so
the v0.13.1 framework-fix phase can resolve them as a class.

---

### F6 — Catalog row's `expected.asserts` describe sim-only post-conditions despite naming "vanilla-clone" [SEVERITY: MED]
**Claim in plan:** Catalog row `agent-ux/reposix-attach-against-vanilla-clone`
(`quality/catalogs/agent-ux.json:43-77`) — the row's name evokes
the vanilla GH-mirror clone use case (Pattern C in dvcs-topology).

**Reality:** Row asserts (lines 53-59):
- *"bash quality/gates/agent-ux/reposix-attach.sh exits 0 against the
  local cargo workspace"*
- *"post-attach git config extensions.partialClone equals 'reposix'"*
- *"post-attach remote.reposix.url begins with 'reposix::' and contains
  the SoT spec"*
- *"the existing origin remote is unchanged"*

These are the universal post-conditions, not the vanilla-clone-specific
ones. The verifier script (`quality/gates/agent-ux/reposix-attach.sh`)
runs `git init -q` (a fresh empty repo, NOT a vanilla GH-mirror clone)
and `reposix attach sim::demo`. The "vanilla clone" in the row name
is aspirational; the verifier exercises a sim-attached `git init`
working tree.

**Evidence:** `quality/catalogs/agent-ux.json:43,47-49`
(row id + sources cite `attach_against_vanilla_clone_sets_partial_clone`
test); `quality/gates/agent-ux/reposix-attach.sh:38-46` (uses
`git init -q` + `sim::demo` only — no real GH mirror clone, no
real backend); `crates/reposix-cli/tests/attach.rs:286-343`
(`attach_against_vanilla_clone_sets_partial_clone` does git init
and `git remote add origin file://...` to a sim, not a real mirror).

**Why it matters:** Failure shape #1 from the audit brief — name
promises one thing (vanilla-clone-against-real-mirror), assertions
deliver less (sim-attach-against-fresh-git-init). A reader who
trusts the row name walks away thinking the dark-factory thesis is
gated; in reality only the sim post-conditions are.

---

### F7 — REQUIREMENTS.md DVCS-ATTACH-01 marked `[x]` shipped despite real-backend gap [SEVERITY: MED]
**Claim in plan:** P79's phase-close protocol (advisory item 2 in
VERDICT.md) said:
> *"REQUIREMENTS.md DVCS-ATTACH-01..04 checkboxes still [ ]. POC-01
> is [x]; the four DVCS-ATTACH rows remain unchecked. Standard
> phase-close flips these — should be done in the same post-hoc
> SUMMARY commit as item 1."*

**Reality:** The post-hoc commit appears to have flipped them.
`.planning/milestones/v0.13.0-phases/REQUIREMENTS.md:45-48`
shows DVCS-ATTACH-01..04 all `[x]`. The same line (45) reads:
*"`reposix attach <backend>::<project>` subcommand in
`crates/reposix-cli/` ... REST-lists backend; populates cache OIDs
lazily..."* — the abstract `<backend>::<project>` reads as if all
backends are supported. Reality: only `sim` is supported.

The shipped lookup table (`architecture-sketch/index.md:130-133`)
also marks all four DVCS-ATTACH rows as `shipped`.

**Evidence:**
`.planning/milestones/v0.13.0-phases/REQUIREMENTS.md:45-48` (all
`[x]`); `architecture-sketch/index.md:130-133` (all "shipped");
contradicted by `crates/reposix-cli/src/attach.rs:147-166`.

**Why it matters:** The project's own self-reported milestone
status reads "shipped" for a feature that only ships on the
testing simulator. Future contributors reading REQUIREMENTS.md
will treat DVCS-ATTACH-01 as done; future phase planners will not
re-open it. The scope-reduction documented in 79-02 SUMMARY § "Deviations
from plan" never propagated to the requirement state.

---

### F8 — CLAUDE.md "attach" example uses `sim::demo` against a `git@github.com` clone (incoherent) [SEVERITY: MED]
**Claim in plan:** P79 phase-close protocol step 7 + 79-03 T03:
*"79-03's terminal task updates § 'Commands you'll actually use'
to add `reposix attach <backend>::<project>` example alongside the
existing `reposix init` example."*

**Reality:** `CLAUDE.md:171-175` shipped:
```bash
# Attach an existing checkout (vanilla GH-mirror clone or hand-edited tree, v0.13.0+)
git clone git@github.com:org/issues-repo.git /tmp/issues  # vanilla mirror clone (no reposix needed)
cd /tmp/issues
reposix attach sim::demo --remote-name reposix            # build cache from REST; reconcile by frontmatter id; add reposix remote
git push reposix main                                     # push via reposix remote (single-SoT shape; bus URL form requires P82+)
```
`git clone git@github.com:org/issues-repo.git` then `reposix attach
sim::demo` is incoherent: the user clones from GitHub, then attaches
to a *simulator* SoT. This documents an arrangement that cannot exist
in production (the simulator runs locally on `127.0.0.1:7878` — there's
no SoT for the GitHub clone to be reconciled against). Either the
`git clone` should be from the mirror of an existing sim-seeded fixture
(weird, undocumented) OR the attach spec should be `confluence::SPACE`
(which would fail per F1).

**Evidence:** `CLAUDE.md:171-175`.

**Why it matters:** CLAUDE.md is the every-agent grounding doc. A
new agent reading this block gets a misleading mental model of what
`reposix attach` does — and a working invocation that doesn't reflect
the v0.13.0 thesis. CLAUDE.md describes an aspirational state that
the binary doesn't deliver, exactly the failure shape #7 from the
audit brief.

---

### F9 — VERDICT.md fails to interrogate the deferral pointer in 79-02 SUMMARY [SEVERITY: MED]
**Claim in plan:** The verifier subagent prompt template
(`quality/PROTOCOL.md` § "Verifier subagent prompt template", verbatim
copy obligation per 79-PLAN-OVERVIEW/scope-and-delegation.md:46) is
supposed to grade catalog rows from artifacts with zero session
context. But the same overview's verifier criteria (lines 50-57)
include:
> *"DVCS-ATTACH-01: ... cache appears at
> `resolve_cache_path(<sot-backend>, <sot-project>)`; remote URL
> written matches `reposix::<sot-spec>` (or `?mirror=` form when
> default `--bus`)."*

This wording leaves `<sot-backend>` open. The verifier could have
demanded evidence the binary handled `<sot-backend> = github |
confluence | jira` — and didn't.

**Reality:** VERDICT.md grades DVCS-ATTACH-01 PASS citing only the
sim-backed integration test (`attach_against_vanilla_clone_sets_partial_clone`,
tests/attach.rs:286). The verdict's "Phase-close protocol" table
(lines 19-27) reads through 7 process items but never asks "is
github/confluence/jira reachable through this surface?". Advisory
items (lines 29-33) call out only:
1. Missing 79-03-SUMMARY (cosmetic)
2. REQUIREMENTS.md checkboxes (cosmetic)
3. POC-FINDINGS.md heading wording (cosmetic)

The backends gap was sitting in `attach.rs:162-165` as a phase-ID-leaking
bail!, in `79-02-SUMMARY.md:144` as documented "deliberate scope
reduction", and in `79-02-SUMMARY.md:59` as "github/confluence/jira
bail with a clear error pointing to 79-03". The verifier read 79-02
SUMMARY (cited at VERDICT.md:27) and did not connect the
"pointing to 79-03" thread to a 79-03 plan that didn't deliver.

**Evidence:** `quality/reports/verdicts/p79/VERDICT.md:11-39` (entire
content); 79-02 SUMMARY citations at lines 27 + 31.

**Why it matters:** Failure shape #2 from the audit brief, plus
"Verdict honesty" from the bird's-eye criteria. The verifier did
its mechanical job (cite-evidence, REQ-by-REQ); it didn't ask "what
does this phase actually deliver against the milestone vision?"
The phase-close subagent dispatch needs an extension that asks
"are the deferral pointers in this phase's planning artifacts honored
by other shipped artifacts?" — otherwise scope can vanish between
plan files indefinitely.

---

### F10 — `dark-factory.sh dvcs-third-arm` exists but the TokenWorld leg is SUBSTRATE-GAP-DEFERRED [SEVERITY: MED]
**Claim in plan:** Per ROADMAP P86 (line 84): *"the v0.13.0 P86 arm
— vanilla-clone + reposix attach + bus URL composition + cache
audit"* and CLAUDE.md:190 references the same arm.

**Reality:** `quality/catalogs/agent-ux.json:1015` (`dark-factory.sh
dvcs-third-arm` row's expected asserts) explicitly states:
> *"TokenWorld real-backend leg SUBSTRATE-GAP-DEFERRED: skipped
> unless REPOSIX_DARK_FACTORY_REAL_TOKENWORLD=1; cites P84
> SURPRISES-INTAKE binstall + yanked-gix entry"*

The agent-ux dim's most prominent end-to-end gate explicitly opt-outs
the real-backend arm. P79's catalog row inherits this implicitly:
the row that gates DVCS-ATTACH-01..04 (sim) and the row that gates
the dark-factory third arm (also sim by default) together cover ZERO
real-backend reposix attach evidence.

**Evidence:** `quality/catalogs/agent-ux.json:1011-1033` (especially
1018 SUBSTRATE-GAP-DEFERRED text); `quality/catalogs/agent-ux.json:55-58`
(P79's catalog row asserts — sim-shape only).

**Why it matters:** Failure shape #2 ("substrate gap deferrals
masquerading as GREEN"). The PROJECT has an environment for real
TokenWorld testing (CLAUDE.md OP-6 + `docs/reference/testing-targets.md`),
and the simulator is good — but together they create a perfect
exit ramp for any feature that "would-work-on-sim-but-doesn't-on-prod".
P79 is the canonical example: the project's only end-to-end gate
that *could* exercise real-backend attach has the relevant leg
opt-outted by env-var default. Verifier had no signal.

---

### F11 — POC pre-execution warned the planner about the "reconciliation rule count" risk but not the "real-backend wiring" risk [SEVERITY: LOW]
**Claim in plan:** `79-PLAN-OVERVIEW/risks.md:8` flags
*"Reconciliation cases need >5 distinct rules"* as MEDIUM and
discusses mitigation; `risks.md` enumerates 14 risks. None mention
"real-backend wiring deferred to nowhere" or "cred-paths threaded
through `attach`".

**Reality:** The risk register did not anticipate the failure mode
that ultimately broke the milestone vision. 79-02's "deliberate
scope reduction" landed *during execution* without being escalated
to SURPRISES-INTAKE (per OP-8). The summary's "Deviations from
plan" section (`79-02-SUMMARY.md:142-146`) explicitly says
*"This is a deliberate scope reduction"* — the executor surfaced
a scope decision that should have been a phase-orchestrator
decision (the executor unilaterally moved real-backend wiring to
"79-03" without checking 79-03's PLAN actually carried it).

**Evidence:**
`.planning/phases/79-poc-reposix-attach-core/79-PLAN-OVERVIEW/risks.md`
(no entry for cred-path-threading risk);
`79-02-SUMMARY.md:142-146` (executor's "deliberate scope
reduction" annotation, no SURPRISES-INTAKE entry filed);
`SURPRISES-INTAKE.md` (P79 entries blank — confirmed via verdict
line 26 *"file untouched since P57"*).

**Why it matters:** The +2 reservation pattern (CLAUDE.md OP-8) is
designed to absorb exactly this kind of in-flight scope drift. The
executor judged the deferral as eager-resolution and folded it into
a deliberate scope reduction; CLAUDE.md says "if … no new
dependency introduced" eager-resolution is preferred — but a deferral
to a downstream plan that doesn't carry the work is precisely a NEW
dependency. The pattern was applied wrong because the rule isn't
crisp enough to distinguish "fix-now" from "defer-to-N+1".

---

### F12 — `architecture-sketch/index.md` lookup table claims DVCS-ATTACH-01..04 "shipped" without scope qualifier [SEVERITY: LOW]
**Claim in plan:** Lookup table at
`.planning/research/v0.13.0-dvcs/architecture-sketch/index.md:128-135`:
| REQ-ID | Phase | Status |
|---|---|---|
| DVCS-ATTACH-01 | P79 | shipped |
| DVCS-ATTACH-02 | P79 | shipped |
| DVCS-ATTACH-03 | P79 | shipped |
| DVCS-ATTACH-04 | P79 | shipped |

**Reality:** "shipped" is correct for the simulator, false for the
prototypical use case (`attach confluence::SPACE`). The table has
no qualifier column for "sim-only" / "real-backend coverage". This
is the planning project's central status surface, and it is now
consistent with the verifier verdict + REQUIREMENTS.md, all of which
overstate scope.

**Evidence:**
`.planning/research/v0.13.0-dvcs/architecture-sketch/index.md:130-133`.

**Why it matters:** Documentation drift. Once enough surfaces all
say "shipped", the gap becomes invisible. Future planners will
not re-open DVCS-ATTACH-01.

---

### F13 — Suspect: `Cache::open` + `Cache::build_from` may have been touched in a way that prevents real-backend wiring being a 1-file change [SEVERITY: SUSPECT]
**Claim in plan:** 79-03 fix-forward commit `a558d4a` is described
in 79-03-SUMMARY.md:30 as *"idempotent `Cache::open` + `build_from`
+ REPOSIX_SIM_ORIGIN env override (P79-03 fix-forward)"*.

**Reality:** I have not read `crates/reposix-cache/src/cache.rs`
or `builder.rs` post-`a558d4a` to confirm. The `REPOSIX_SIM_ORIGIN`
env override hard-codes the sim-only assumption into the cache
construction path. If `Cache::open(connector, backend, project)`
takes any sim-specific shortcut (e.g., assumes the connector is
the local sim), the real-backend wiring is more than a 1-file
change to `attach.rs`.

**Evidence:** `79-03-SUMMARY.md:30-32` (commit `a558d4a` cited);
`crates/reposix-cli/src/attach.rs:155-158` (the `REPOSIX_SIM_ORIGIN`
env var read at the connector construction site). This finding
would settle by reading `crates/reposix-cache/src/cache.rs::Cache::open`
and `builder.rs::Cache::build_from` and verifying neither has a
sim-specific code path.

**Why it matters:** If the fix is bigger than 1-file, the v0.13.1
framework-fix phase needs to scope it correctly. If it's truly
1-file (just dispatch to the existing `instantiate_*` helpers
already in `backend_dispatch.rs`), this is a 30-minute fix.
SUSPECT pending a `cat` on the two files in question.

## Summary table

| F# | Sev | Title |
|---|---|---|
| F1 | HIGH | `reposix attach` rejects all real backends with phase-ID-leaking error |
| F2 | HIGH | 79-03 PLAN silently dropped the real-backend wiring 79-02 SUMMARY promised |
| F3 | HIGH | Verifier mistook "tests pass" for "feature ships" |
| F4 | HIGH | `agent_flow_real.rs` covers `init` real-backend smoke but not `attach` |
| F5 | MED | `reposix sync --reconcile` carries the same scaffold-only deferral |
| F6 | MED | Catalog row's `expected.asserts` describe sim-only post-conditions despite naming "vanilla-clone" |
| F7 | MED | REQUIREMENTS.md DVCS-ATTACH-01 marked `[x]` shipped despite real-backend gap |
| F8 | MED | CLAUDE.md attach example uses `sim::demo` against a `git@github.com` clone (incoherent) |
| F9 | MED | VERDICT.md fails to interrogate the deferral pointer in 79-02 SUMMARY |
| F10 | MED | `dark-factory.sh dvcs-third-arm`'s real-backend leg is SUBSTRATE-GAP-DEFERRED |
| F11 | LOW | POC pre-execution risk register did not anticipate the wiring drop |
| F12 | LOW | architecture-sketch lookup table claims DVCS-ATTACH-01..04 "shipped" without qualifier |
| F13 | SUSPECT | Whether the fix is 1-file or N-file pending Cache::open/build_from inspection |

## Cross-cutting recommendations for the v0.13.1 framework-fix phase

(Out-of-scope to fix here per audit-brief hard rule #1, but useful
context for the phase that consumes these findings.)

1. **Catalog row contract: end-to-end asserts must include the real-backend
   leg** OR explicitly carry a `coverage: sim-only` field that the
   verifier subagent treats as a HIGH gap on any DVCS / transport-layer
   row. Today the framework lets the gap go silent.

2. **Deferral-pointer linter.** A pre-push check that grep -rn
   `not yet wired in P\d+` across `crates/` + cross-references against
   the named phase's PLAN files for actual delivery. If the named
   phase's PLAN doesn't mention the wiring, BLOCK.

3. **Banned production-error tokens.** Add `P\d+-\d+` (phase-ID
   pattern) to `quality/gates/structure/banned-words.sh`'s production
   string allowlist. Phase IDs leaking into binary output is a
   process-failure smell.

4. **`agent_flow_real.rs` is the v0.13.0 attach gate.** Add three
   `attach_real_{github,confluence,jira}` tests with the same `#[ignore]`
   shape as the existing init smoke tests; even minimal happy-path
   coverage (vanilla `git init` + `reposix attach $BACKEND::$PROJECT`
   + assert post-conditions) closes F4 + adds the missing real-backend
   leg the catalog row needs (F6).

5. **Verifier prompt template extension.** Add a recurring step
   "find all `not yet wired` / scaffold-only / SUBSTRATE-GAP-DEFERRED
   markers in the phase's commits and grade whether the named
   downstream phase delivered them". Today's template is artifact-only;
   it can't catch dropped pointers because nothing tells it to look.
