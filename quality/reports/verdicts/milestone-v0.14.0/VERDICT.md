---
milestone: v0.14.0
date: 2026-07-13
verdict: GREEN-WITH-RECORDED-CAVEATS
status: READY TO TAG (with recorded caveats)
head_at_grade: 4645060
ruling: "Manager Ruling #4 / Option B (dcf49d4) — tag proceeds under a recorded caveat"
aggregate_real_backend: "5 PASS / 0 FAIL / 1 NOT-VERIFIED — cadence exit 1 (NOT exit 0; see t4 caveat)"
verifier: "unbiased milestone-close verifier subagent (no session context — grades from the recorded probe artifacts of the creds-loaded run committed at 4645060, NOT a fresh re-run by this agent), authored under Manager Ruling #4"
constraint_notes:
  - "No cargo invoked; no real-backend calls triggered by this grading pass. Grades ONLY the committed catalog state (quality/catalogs/agent-ux.json @ 4645060) + its cited transcripts/artifacts."
  - "No tag cut, no tag push. Tag push remains the manager's, per the 2026-07-12 owner delegation."
  - "Explicit-path add of this VERDICT.md only."
---

# Milestone v0.14.0 — Aggregate Milestone-Close Verdict

## Status: GREEN-WITH-RECORDED-CAVEATS — READY TO TAG (with recorded caveats)

**The `pre-release-real-backend` cadence exited 1 — NOT 0.** Read that first, because it
is the fact this verdict must never blur: the honest, currently-committed cadence result
is **5 PASS / 0 FAIL / 0 PARTIAL / 0 WAIVED / 1 NOT-VERIFIED**, cadence exit code **1**.
This milestone is tag-ready **by Manager Ruling #4 (Option B)** — a recorded caveat call
under the manager's standing release authority (`.planning/CONSULT-DECISIONS.md`, manager
commit `dcf49d4`) — **not because the cadence passed.** GREEN-WITH-RECORDED-CAVEATS is a
distinct, honest verdict state from a clean GREEN: it says the product's real-backend
behavior is proven where it could be exercised, and the one row that could not be
exercised is held at an honest NOT-VERIFIED rather than waived, skipped, or hidden.

## How this went from RED to GREEN-WITH-RECORDED-CAVEATS

The prior verdict (dated 2026-07-12, `head_at_grade: 9890a67`) graded this milestone
**RED — NOT READY TO TAG** on an aggregate of **1 PASS / 3 FAIL / 2 NOT-VERIFIED**. That
RED is now stale — superseded by two rounds of fixes, both landed and independently
re-verified against live TokenWorld:

1. **The item-5 string-encoded-ADF regression (fixed `49666eb`).** `reposix-confluence`
   could not parse any real Confluence page — the Confluence Cloud v2 API string-encodes
   `body.atlas_doc_format.value` as a JSON *string*, while `ConfBodyAdf.value`
   deserialized it as an object, so `adf_root_type` always read `""` and tripped the
   fail-closed unreadable-ADF sentinel on every real page. This was the root cause behind
   the prior verdict's B1 (vision-litmus FAIL) and B2 (p93 FAIL). A follow-on test-fix
   lane (5a–5d, `.planning/RETROSPECTIVE.md` "Tag-prep arc") closed a vacuous regression
   test and two sibling wiremock-fixture gaps the fix review surfaced.
2. **Two harness gaps, fixed under Manager Ruling #3 (Option A) at `cb8ad11`.** Not
   product regressions: `github-front-door-real-backend`'s harness never built/PATHed the
   `git-remote-reposix` helper binary, so `reposix init github::...` died before any
   GitHub REST call; `t4-conflict-rebase-ancestry-real-backend`'s pre-mutation
   sanctioned-target guard accepted only the display-name `"TokenWorld"` and rejected the
   Atlassian space KEY `REPOSIX` that the real cadence env actually drives the space by
   (both spellings resolve to the same space id `360450`) — so the guard could
   structurally never pass. Ruling #3 fixed both harnesses (PATH export; case now accepts
   `TokenWorld|REPOSIX`) while keeping the fail-closed pre-mutation protected-id + tenant
   guard intact.

With both fix rounds landed, a **creds-loaded 9th-probe re-run** re-established the honest
result and its runner-minted grades are **committed at `4645060`**
(`quality/catalogs/agent-ux.json`, commit `chore(quality): persist honest 5/0/1
real-backend grades...`). That commit is what this verdict grades from.

## Aggregate: 5 PASS / 0 FAIL / 1 NOT-VERIFIED — cadence exit 1

| Row | blast_radius | Grade | Evidence |
|---|---|---|---|
| `agent-ux/milestone-close-vision-litmus-real-backend` | **P0** | ✅ **PASS** | `quality/reports/transcripts/milestone-close-vision-litmus-real-backend.txt`; catalog `last_verified: 2026-07-13T22:37:09Z` |
| `agent-ux/p93-partial-failure-recovery-real-confluence` | **P0** | ✅ **PASS** | `quality/reports/transcripts/p93-partial-failure-recovery-real-confluence.txt`; catalog `last_verified: 2026-07-13T22:37:21Z` |
| `agent-ux/github-front-door-real-backend` | P1 | ✅ **PASS** | `quality/reports/transcripts/github-front-door-real-backend.txt`; catalog `last_verified: 2026-07-13T23:55:05Z` |
| `agent-ux/attach-sync-real-backend` | P1 | ✅ **PASS** | `quality/reports/transcripts/attach-sync-real-backend.txt`; catalog `last_verified: 2026-07-13T19:06:17Z` |
| `agent-ux/cadence-pre-release-real-backend` | P1 | ✅ **PASS** | cadence wiring self-test; catalog `last_verified: 2026-07-13T01:51:55Z` |
| `agent-ux/t4-conflict-rebase-ancestry-real-backend` | **P0** | 🟠 **NOT-VERIFIED** | `quality/reports/verifications/agent-ux/t4-conflict-rebase-ancestry-real-backend.json` (git-floor evidence — see below); catalog `last_verified: 2026-07-13T23:54:53Z` |

**Cadence exit code: 1.** Six rows, five PASS, one NOT-VERIFIED — a non-zero exit is the
correct, honest mechanical result for that mix; it is not a discrepancy to explain away.

## Load-bearing GREEN evidence — both P0 real-backend rows PASS against live TokenWorld

The non-skippable 9th probe, **`milestone-close-vision-litmus-real-backend` (P0)**, PASSED
for real: sanctioned Confluence space `REPOSIX` confirmed, vanilla clone obtained,
`reposix attach` wired partial-clone git config, reconciliation was mass-delete-safe
(`matched=3 backend_deleted=0`), an edit+commit+`git push` round-tripped through the real
helper (not a synthetic stream), the server-side change was confirmed via REST, both audit
tables (`audit_events_cache` + `audit_events`) carried rows for the action, and the mirror
refs advanced — full assertion list in
`quality/reports/verifications/agent-ux/milestone-close-vision-litmus-real-backend.json`,
transcript at `quality/reports/transcripts/milestone-close-vision-litmus-real-backend.txt`.

**`p93-partial-failure-recovery-real-confluence` (P0)** PASSED for real: a page's
rename-into-title-collision was atomically rejected by Confluence, the next push
re-read SoT via PRECHECK, diffed away the already-landed sibling page, and replanned +
landed the rejected page — the real UPDATE-recovery round-trip against live TokenWorld.
Transcript at `quality/reports/transcripts/p93-partial-failure-recovery-real-confluence.txt`.

Both are P0 rows and both PASSED against the live sanctioned backend. The milestone's
real-backend behavior is proven, not merely asserted.

## The sole cadence caveat: t4 NOT-VERIFIED — an environment limitation, not a product regression, not a silent waiver

`agent-ux/t4-conflict-rebase-ancestry-real-backend` is a **P0** row and it did **not**
PASS this cadence run — it is honestly held at **NOT-VERIFIED**. This is the one thing
standing between this milestone and a clean exit-0. It is recorded here in full, not
minimized:

Creds were live for this row (the env-gate cleared, and Ruling #3's fixed sanctioned
space-KEY `REPOSIX` guard was in effect), so the row got as far as attempting the
two-writer conflict+refetch scenario against real TokenWorld — then bailed **before any
mutation**, at a git-version floor:

```
asserts_failed: ["git 2.25.1 < 2.34"]
skip_reason: precondition-not-met
```

This is exit 75 (precondition-not-met), not a behavioral FAIL. It is **not** the
env-missing skip state (creds were present and validated) and it is **not** a product
defect — it is this VM's installed git (2.25.1) falling short of the `2.34` floor the
`stateless-connect` two-writer conflict-rebase-ancestry scenario needs to run reliably.
Nothing about reposix's real conflict-handling behavior was disproven; the scenario simply
could not be exercised on this host. Per OD-2, an inability to execute a real-backend P0
probe at milestone-close would normally be hard RED — Manager Ruling #4 is the ruling that
converts that hard-RED default into a **recorded caveat**, not a waiver of the row itself.

The row stays **runner-minted** NOT-VERIFIED — no hand-edit, no catalog surgery, no
gate-policy change. **Manager Ruling #4's three binding conditions** (quoted verbatim,
`.planning/CONSULT-DECISIONS.md` "2026-07-13 [MANAGER] Ruling #4"):

> 1. t4 row stays runner-minted NOT-VERIFIED — NO waiver, NO catalog surgery, NO
>    gate-policy code change. The honest cadence result on record is 5 PASS / 0 FAIL / 1
>    NOT-VERIFIED (t4, git 2.25.1 < 2.34 env floor, exit 75 precondition-not-met, bailed
>    pre-mutation).
> 2. The verdict re-mint is GREEN-WITH-RECORDED-CAVEATS and must NEVER claim cadence
>    exit-0. It records the 5/0/1 result + the caveat: t4's two-writer conflict+refetch
>    scenario is un-runnable on THIS VM (env floor, not product; sim twin green in CI;
>    litmus P0 + p93 P0 PASSED against live TokenWorld; `reposix doctor` itself treats
>    sub-2.34 as WARN).
> 3. Option A (VM git upgrade, interactive sudo) is RAISEd to the owner — do NOT attempt
>    it.

This verdict complies with all three: the row above is graded exactly as the runner minted
it (NOT-VERIFIED, no waiver field set); this document states the cadence exit code (1) in
its first paragraph and its frontmatter, never claiming exit-0; and no git-upgrade attempt
was made by this or any prior session — the VM git-version floor is raised to the owner
as an open item, not acted on autonomously.

**Corroborating context for why this caveat is acceptable, not a shrug:** the sim-arm
twin of this exact scenario (`agent-ux/t4-conflict-rebase-ancestry`, no `-real-backend`
suffix) is green in CI on every push — the conflict-reject + no-fresh-root-on-refetch
mechanism itself is proven, just not against this specific real-backend target on this
specific host. `reposix doctor` itself treats sub-2.34 git as WARN, not ERROR, consistent
with the project's own documented stance that 2.34+ is "recommended," not "required," for
reliable partial-clone reads.

## Two secondary recorded caveats (DEFER → v0.15.0, non-blocking)

Beyond the t4 environment-floor caveat, this tag carries two prior-ruled, already-defer
red caveats — named here for completeness, neither reopened by this verdict:

- **(a) Litmus non-idempotency vs. its own mirror fan-out.** The vision-litmus probe is
  non-idempotent against its own GitHub mirror fan-out — a pre-existing ADR-010 RBF-LR-04
  characteristic (the inline mirror fan-out pushes the pre-write client tree, not the
  post-write materialized snapshot), shipped identically in v0.13.0, not a v0.14.0
  regression. The interim op (`scripts/refresh-tokenworld-mirror.sh`, run immediately
  before the litmus probe) is the documented recovery; one clean run legitimately grades
  item-5 GREEN for this tag. Product fix (mirror-sync pushing the post-write snapshot)
  routed to v0.15.0 first-class. **Ruling #2** (E2/ADR valve, RULED-DEFER→v0.15.0).
- **(b) p93 CREATE-recovery WAIVED.** `p93-partial-failure-recovery-real-confluence`
  PASSED as an UPDATE-recovery proof against live TokenWorld. It does not cover
  CREATE-recovery: a partial-fail whose landed action was a create against an
  id-assigning backend genuinely does not converge on refetch-and-replan the same way an
  update does. This is the owner-signed **WAIVED known limitation of ADR-010 §3 /
  RBF-LR-03** — hand-recoverable, routed to v0.15.0, not a silent gap.

Full distillation of both, plus the item-5 fix arc and Ruling #3's harness-gap fixes:
`.planning/RETROSPECTIVE.md` § "Tag-prep arc (fix-first items 4–8) — item-5 ADF
regression, litmus non-idempotency, and the three tag rulings."

## Awareness note — non-blocking cosmetic artifact gap (filed, not acted on here)

The `github-front-door-real-backend` catalog row carries the operative grade **PASS** (the
row this verdict grades from), but the runner's persist path left a stale cosmetic
`skip_reason: env-missing` / `last_real_grade: null` on the same row alongside that PASS —
a filed runner-persist-path gap, not a real failure and not evidence the row actually
skipped. This verdict does not hand-edit the catalog to clean that up (no catalog surgery
per this agent's charter); it is noted here as a caveat footnote for the next session that
touches the persist path.

## TokenWorld end-state

Verified: **2 protected fixtures** (`7766017`, `7798785`) intact, plus sacrificial page
`2818063` present and current. `t4-conflict-rebase-ancestry-real-backend` bailed
**pre-mutation** at the git-version floor — no test residue, nothing to clean up.

## No tag pushed

**This verdict does not cut or push a tag.** Per the 2026-07-12 owner delegation
(`.planning/CONSULT-DECISIONS.md`), the v0.14.0 tag cut+push is the manager's action; this
verdict brings the milestone to READY-TO-TAG-with-recorded-caveats and stops there.

---

**VERDICT: GREEN-WITH-RECORDED-CAVEATS — READY TO TAG (with recorded caveats).** The
`pre-release-real-backend` cadence honestly exits **1** (5 PASS / 0 FAIL / 1
NOT-VERIFIED) — this verdict never claims otherwise. Both P0 real-backend rows that could
execute (vision-litmus, p93) PASSED against live TokenWorld; the sole NOT-VERIFIED row
(t4, also P0) is an environment limitation — this VM's git 2.25.1 falls below the 2.34
floor the scenario needs — not a product regression, ruled acceptable as a recorded
caveat under **Manager Ruling #4 (Option B)**, with its three binding conditions honored
verbatim above. Two secondary caveats (litmus non-idempotency, p93 CREATE-recovery) are
prior-ruled DEFER→v0.15.0 and carried forward, not reopened. TokenWorld end-state is
clean (2 protected + `2818063`, no residue). No tag has been cut or pushed by this
verdict.

_Verifier: Claude (unbiased milestone-close, authored under Manager Ruling #4). Graded
from the recorded catalog state + transcripts at `head_at_grade: 4645060`; cadence NOT
re-run, no cargo, no real-backend calls, no tag, no push._
