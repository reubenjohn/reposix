# COMPLETENESS-CHECK-2 — orchestrator-level files for next session

**Auditor:** unbiased completeness subagent (zero session context)
**Date:** 2026-05-08
**Scope:** the three orchestrator-level files written in the current session — `SYNTHESIS-VERIFICATION.md`, `DECISIONS-NEEDED.md`, `READY-TO-EXECUTE.md`. Not the four `03-synthesis/` docs (already audited at `03-synthesis/COMPLETENESS-CHECK.md`).

---

## Summary verdict

**Borderline ready.** A cold agent following the prescribed reading order will reach "I can start work" in ~30 minutes, but they will trip on at least three load-bearing issues: an internal contradiction about which decisions DECISIONS-NEEDED actually contains, a missed-decision risk around retroactive treatment of the existing `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` (graded GREEN but now de-facto stale), and a +2-reservation semantics ambiguity that the new files inherited from REMEDIATION-PLAN without surfacing. Three STRONG findings + four MEDIUM + four WEAK below. Recommendation: one ≤30-minute pass over READY-TO-EXECUTE + DECISIONS-NEEDED to align them and surface the missed decisions; SYNTHESIS-VERIFICATION can stand.

---

## STRONG findings (load-bearing — cold agent trips here)

### S-1 — Misalignment: READY-TO-EXECUTE Quality bar item 2 cites three decisions in DECISIONS-NEEDED that don't all exist

`READY-TO-EXECUTE.md:38` says: "Owner has signed off on `DECISIONS-NEEDED.md` answers (S1 arbiter mechanism, S2 mid-stream checkpoints, S3 vision-coverage-delta on deferrals)."

But:
- `DECISIONS-NEEDED.md:5-11` Decision 1 = S2 (mid-stream checkpoints). ✓
- `DECISIONS-NEEDED.md:15-21` Decision 2 = S3 (vision-coverage-delta). ✓
- `DECISIONS-NEEDED.md:25-31` Decision 3 = "patch-class vs redesign-class scope tension" — invokes S1 + Q3, but is **not** a clean "S1 arbiter mechanism" question. It's a tension between F-K1..F-K8 sufficiency and structural redesign.
- `SESSION-2026-05-08-HANDOFF.md:8` already states **"S1 (external arbiter) is approved"** — so S1 itself is closed, not in DECISIONS-NEEDED.

**Impact:** the cold agent reads READY-TO-EXECUTE, looks for "S1 arbiter mechanism" in DECISIONS-NEEDED, doesn't find it, and either (a) re-opens an already-closed decision or (b) treats Decision 3 as the S1 question and conflates structural-axis with arbiter-mechanism.

**Recommended fix:** re-word READY-TO-EXECUTE Quality bar item 2 to "(S2 mid-stream checkpoints, S3 vision-coverage-delta on deferrals, plus the patch-vs-redesign tension surfaced by SYNTHESIS-VERIFICATION)" and add an explicit note that S1 itself was decided in the prior session per `SESSION-2026-05-08-HANDOFF.md`.

### S-2 — Missed decision: what happens to the existing `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` (graded GREEN 2026-05-01)

The verdict file at `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` exists and was graded GREEN. So do all 11 phase verdicts at `quality/reports/verdicts/p7{8,9},p8{0..8}/VERDICT.md`. If we extend v0.13.0 with P89–P96, what happens to those existing GREENs? Three options exist (leave as historical / re-grade with amendment / retract) — `03-synthesis/COMPLETENESS-CHECK.md:81-83` (M2) flagged this explicitly as "MEDIUM-confidence gap … the plan implicitly assumes 'no retroactive re-grade needed'."

DECISIONS-NEEDED does **not** carry M2. READY-TO-EXECUTE §2 line 20 says "**SKIP M/W gaps** unless surfaced by `DECISIONS-NEEDED.md`." So M2 disappears from the cold agent's view — and the 12 GREEN verdict files **are the supply-chain trust artifact future planners read**. Leaving them GREEN-without-asterisk is the same fossilization (PATTERNS C9) that v0.13.0-extension is trying to fix.

This is also entangled with the `REMEDIATION-PLAN.md:364` claim that the milestone-close verdict will land at `quality/reports/verdicts/milestone-v0.13.1/VERDICT.md` — but if we extend v0.13.0, should it overwrite the existing `milestone-v0.13.0/VERDICT.md`, or sit beside it? The new files don't address this, so the cold agent will improvise.

**Recommended fix:** add a fourth decision to `DECISIONS-NEEDED.md` — "Retroactive verdict-file treatment: leave / amend / retract (and which verdict-file location does P96's milestone-close write to)." This is a load-bearing missed decision because it determines whether v0.13.0's existing verdict files get a "GREEN-with-amendment" overlay before P96 closes.

### S-3 — Internal contradiction: "Tag NOT pushed" vs `STATE.md` saying "ready-to-tag; owner pushes tag"

`READY-TO-EXECUTE.md:8` says: "v0.13.0 graded GREEN on 2026-05-01; tag NOT pushed. The milestone is in a 'ready-to-tag' state that the owner has frozen pending corrective phases."

Live `.planning/STATE.md:6` says `status: ready-to-tag`; `.planning/STATE.md:18` says "11/11 phases complete (P78..P88) … Orchestrator does NOT push tag (ROADMAP P88 SC6 -- STOP at tag boundary)." Live `CHANGELOG.md:9` says "Release status: PENDING owner tag-cut." None of these reflect "owner has frozen pending corrective phases" — every live artifact still tells the cold agent "the next step is to push the tag."

So the cold agent reads READY-TO-EXECUTE saying "frozen," then opens STATE.md/CHANGELOG.md and sees "owner pushes tag." If the cold agent runs `bash .planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` (per CHANGELOG.md:9), all 8 guards pass right now and the tag ships — silently undoing the strategic decision.

**Recommended fix:** READY-TO-EXECUTE Quality bar should add an item 0 (or P89-01-prep): "STATE.md `status:` must be flipped from `ready-to-tag` to `extending-via-corrective-phases`; CHANGELOG.md:9 'PENDING owner tag-cut' line must be updated to 'PENDING P89–P96 GREEN'; tag-v0.13.0.sh guards 7+8 (P88 verdict, milestone-v0.13.0 verdict) must be amended to also require P96 + milestone-v0.13.0-extended verdicts before the script will succeed." Without this, the strategic decision is undefended at the CHANGELOG/script/STATE-file layer.

---

## MEDIUM findings (worth surfacing; not load-bearing)

### M-1 — SYNTHESIS-VERIFICATION not in the cold-agent reading order

`READY-TO-EXECUTE.md:14-22` lists 6 reading-order items; SYNTHESIS-VERIFICATION is NOT one. `README.md:15-46` doesn't reference SYNTHESIS-VERIFICATION either (the file post-dates the README). Yet DECISIONS-NEEDED Decision 3 (`DECISIONS-NEEDED.md:25-31`) drills into "SYNTHESIS-VERIFICATION's cross-doc tension #1" — meaning the cold agent IS expected to read the file when answering Decision 3, but isn't told to budget time for it in the 30-minute reading order.

The file is 83 lines, ~5 minutes. **Recommended fix:** add as reading-order item 1.5 (between README and SESSION-HANDOFF) or 5.5 (between COMPLETENESS-CHECK skim and PATTERNS meta), or note "skim only when reviewing DECISIONS-NEEDED Decision 3."

### M-2 — Nomenclature drift the new files diagnosed AND inherited

`SYNTHESIS-VERIFICATION.md:53` correctly flags REMEDIATION-PLAN's "v0.13.1 milestone identity" usage as cosmetic but worth a search-and-replace. The new orchestrator files do this themselves:
- `DECISIONS-NEEDED.md:1` title: "v0.13.1 — Decisions needed before P89-01"
- `DECISIONS-NEEDED.md:17` (S3 question): "WAIVED + until_date = v0.13.1 release"
- `READY-TO-EXECUTE.md:1` title: "READY-TO-EXECUTE — v0.13.1 real-backend frictions"
- `READY-TO-EXECUTE.md:54`: "The whole point of v0.13.1 is operationalizing this"
- `READY-TO-EXECUTE.md:40` references "v0.13.1-era entries"

Meanwhile READY-TO-EXECUTE §1 line 9 settles "DO NOT ship as a separate v0.13.1 milestone." This is the same contradiction flagged in REMEDIATION-PLAN.

**Recommended fix:** the orchestrator's last cleanup pass should s/v0.13.1/v0.13.0 (corrective phases)/ across the new files OR explicitly state at the top of both files: "Throughout this doc, `v0.13.1` is shorthand for the P89–P96 corrective work. The actual tag is `v0.13.0`."

### M-3 — +2 reservation semantics ambiguity inherited from REMEDIATION-PLAN

`READY-TO-EXECUTE.md:10` says "Phase shape settled (P89–P96): 6 work + 2 reservation." `REMEDIATION-PLAN.md:325`/`350` mark P95/P96 as `+2 reservation slot 1` / `slot 2`. But v0.13.0 ALREADY USED its +2 reservation as P87/P88 (per existing ROADMAP.md `### Phase 87: Surprises absorption (+2 reservation slot 1)` / `### Phase 88: Good-to-haves polish (+2 reservation slot 2)`). CLAUDE.md OP-8 says "Every milestone reserves its **last two phases**" — singular ending.

Extending v0.13.0 with another +2 means the milestone has TWO sets of reservation slots. None of the orchestrator files address this. The cold agent will either (a) drop P95/P96's intake-drain semantic because P87/P88 already drained, (b) re-drain SURPRISES-INTAKE.md and find the file empty (P87 already drained it), or (c) treat P95 as a fresh +2 ignoring the prior drain — which is what REMEDIATION-PLAN actually intends but doesn't state.

**Recommended fix:** READY-TO-EXECUTE Quality bar should add an explicit note: "P95/P96 are a SECOND +2 reservation pair on top of P87/P88. The intake files (`SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md`) carry P89–P94-era entries only; pre-P87 entries already drained at v0.13.0's first close." Alternatively add as Decision 4 in DECISIONS-NEEDED: "approve double-+2 inside one extended milestone, or treat P95/P96 as P89–P94 absorption instead of milestone-close absorption."

### M-4 — REQ-ID prefix collision unaddressed

`READY-TO-EXECUTE.md:37` Quality bar item 1 says ROADMAP entries should cite "REMEDIATION-PLAN REQ-IDs." REMEDIATION-PLAN uses prefix `RBF-` (`RBF-FW-01..05`, `RBF-A-01..05`, … `RBF-G-05`). But the existing `.planning/milestones/v0.13.0-phases/ROADMAP.md` uses prefixes `DVCS-*` (e.g. `DVCS-MIRROR-LAG-01`, `DVCS-PERF-01`, `DVCS-SURPRISES-01`, `DVCS-GOOD-TO-HAVES-01`). `03-synthesis/COMPLETENESS-CHECK.md:118-120` (W3) flagged this as a weak nit; the new orchestrator files don't carry it forward.

If the cold agent appends P89–P96 entries with `RBF-*` REQ-IDs, the ROADMAP becomes a mixed-prefix document. That's confusing for cross-phase grep + automation that filters by `^DVCS-`.

**Recommended fix:** decide before P89-01: keep `RBF-*` (consistent with REMEDIATION-PLAN, less rework) OR rename to `DVCS-RBF-*` / `DVCS-FW-*` etc. (consistent with v0.13.0 milestone). Either is fine; ambiguity is not.

---

## WEAK findings (nitpicks)

- **W-1.** `READY-TO-EXECUTE.md:39` references `.planning/phases/89-*/89-PLAN-OVERVIEW.md`. Live convention is mixed: `.planning/phases/87-surprises-absorption/87-01-PLAN.md` (no -OVERVIEW); `.planning/phases/82-bus-remote-url-parser/82-PLAN-OVERVIEW/` is a directory. The cold agent will figure this out from context, but the citation is over-specific without being canonical.
- **W-2.** `READY-TO-EXECUTE.md:13` says "a `DECISIONS-NEEDED.md` is being produced in parallel" — but the file already exists at write time of READY-TO-EXECUTE. Stale phrasing for a cold reader who arrives after both are landed.
- **W-3.** `READY-TO-EXECUTE.md:54` says "The whole point of v0.13.1 is operationalizing [OP-1]" — but v0.13.0 already ratified OP-1 in its milestone-kickoff (`PROJECT.md:86`). The "real-backend tests gate the milestone close, not individual phase closes" was already a v0.13.0 commitment that v0.13.0's milestone-close verifier silently exempted. The point of the corrective phases is honoring that commitment, not creating it.
- **W-4.** Neither `DECISIONS-NEEDED.md` nor `READY-TO-EXECUTE.md` mentions which CLI to use for `gsd-review` (`SESSION-2026-05-08-HANDOFF.md:8` says `claude` + `codex` available; gemini not installed). The cold agent has to grep three files to learn this.

---

## What the three new files got right

`SYNTHESIS-VERIFICATION.md` is rigorous and load-bearing — every sampled claim verified against live tree, three citation drifts surfaced honestly, cross-doc tensions named explicitly. `READY-TO-EXECUTE.md`'s recommendation that this is roadmap-extension (`/gsd-phase`, not `/gsd-new-milestone`) is the right tool-routing decision and saves the cold agent from silently forking the milestone identity.

---

**End of completeness check 2.**
