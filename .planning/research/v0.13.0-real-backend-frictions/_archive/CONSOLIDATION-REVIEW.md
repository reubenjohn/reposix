# Consolidation Review — post-Decision-absorption + renumber

**Reviewer:** adversarial review subagent (zero session context)
**Date:** 2026-05-08
**Scope:** README.md (new front door); 03-synthesis/{REMEDIATION-PLAN, PATTERNS, COMPLETENESS-CHECK, STRATEGIC-REFRAME}.md; the 5 archived files in `_archive/`. Read-only.

---

## Summary verdict

**Iterate before declaring cold-agent-ready.** The consolidation got the load-bearing structural moves right (decision absorption, renumber, drift fixes, README front-door, archive trail). But four cross-doc inconsistencies will trip a cold agent who reads README + REMEDIATION-PLAN side-by-side, the most concrete being a numerical mismatch on the milestone-close commitment ("~41 days" vs. "~42–45 days") that the agent will have to resolve before invoking `/gsd-phase`. STRATEGIC-REFRAME was correctly preserved as historical record. ~30-minute polish pass closes the gaps.

I find: **4 STRONG** + **5 MEDIUM** + **6 WEAK** + 2 things the consolidation got right.

---

## STRONG findings (load-bearing)

### S1 — Effort total mismatch: README "~41 days" vs REMEDIATION "~42–45 days"

`README.md:11` claims `Total estimated effort: **~41 days** (was ~36d before Decision 2 added new P93)`. `REMEDIATION-PLAN.md:436` (§5 Effort summary table row) says `**Total** | | **~42–45 days**`. The README appears to be doing arithmetic-shortcut (36 + 5 = 41) rather than summing the per-phase effort ranges (5–6 + 5–6 + 5 + 5 + 5 + 5 + 5 + 4 + 3 = 42–44).

**Fix:** `README.md:11` → `~42–45 days` (or `~42–44`); update parenthetical accordingly.

### S2 — Disposition banner cites `DECISIONS-NEEDED.md` without `_archive/` prefix

`COMPLETENESS-CHECK.md:11` banner: `S2/S3 ratified per ` `DECISIONS-NEEDED.md` ` Decisions 1+2`. That file lives at `_archive/DECISIONS-NEEDED.md` post-consolidation; the bare reference no longer resolves from the live tree. Cold agent who follows the citation gets a 404.

**Fix:** `COMPLETENESS-CHECK.md:11` → `_archive/DECISIONS-NEEDED.md`.

### S3 — Decision 1 SC formatting inconsistency in P93 (new) vs P91/P92/P94

P91/P92/P94 carry the standard SC text `**Mid-stream litmus checkpoint (Decision 1):** ...` (REMEDIATION lines 222, 252, 310). P93's SC #5 (`REMEDIATION-PLAN.md:280`) reads `Mid-stream litmus checkpoint REOPEN gate (T1 + T4): no HIGH frictions before P94 starts.` — short form, drops the `(Decision 1)` parenthesis. RBF-LR-05 (line 271) does name Decision 1, but the SC#5 line is the one a verifier subagent will grep for.

**Fix:** P93 SC#5 → match P91/P92/P94 paragraph format with `**Mid-stream litmus checkpoint (Decision 1):**` prefix, so `grep -n "Mid-stream litmus checkpoint (Decision 1)"` returns 4 hits as the README implies.

### S4 — COMPLETENESS-CHECK body still says "P96 is the milestone-close ritual" post-renumber

`COMPLETENESS-CHECK.md:49`: `P89–P95 each ship internal plumbing. P96 is the milestone-close ritual.` Post-renumber P96 is *surprises absorption* and P97 is milestone-close. The disposition banner "ratifies S2 per Decision 1" but doesn't flag that the body's phase numbers are pre-renumber. A cold agent skimming COMPLETENESS-CHECK § S2 in the prescribed reading order (README:39 sends them there) reads phase numbers that no longer match REMEDIATION-PLAN.

**Fix:** Either (a) extend the disposition banner with "phase numbers throughout this doc are pre-Decision-2 (P96=close, P97 added by renumber)" or (b) leave a sed s/P96 is the milestone-close/P97 is the milestone-close/ pass over the body. (a) preserves historical-record intent; (b) reduces cold-reader confusion.

---

## MEDIUM findings

### M1 — Pervasive "v0.13.1" nomenclature drift in PATTERNS + COMPLETENESS-CHECK

`PATTERNS.md:197` ("v0.13.1's framework-fix work") and `COMPLETENESS-CHECK.md:1, 19, 41, 45-75 passim` still say "v0.13.1" plan/vision/litmus. README:9 explicitly settles "extend v0.13.0", not v0.13.1 (despite the directory name). Already flagged in `_archive/COMPLETENESS-CHECK-2.md:62-72` as a known issue. Not fixed during consolidation.

### M2 — README "P89, P90, P96, P97 are top-level" elides P93/P94/P95 mixed modes

`README.md:45` lists 4 top-level phases. REMEDIATION-PLAN P93 (line 286), P94 (line 316), P95 (line 358) all have **mixed** execution modes (gsd-execute-phase for code; top-level for ADR or RAISE-LIST drain). A cold agent reading only the README will assume P93/P94/P95 are pure `gsd-execute-phase` and miss the ADR/orchestration carve-out. Add: "P93/P94/P95 are mixed (ADR/RAISE-LIST work top-level; code work `gsd-execute-phase`)."

### M3 — Decision 4 housekeeping step 3 contradicts itself silently

`README.md:28` says: guard fails if VERDICT.md is dated `before 2026-05-08` (i.e., the original 2026-05-01 verdict). But Decision 4 (line 18) says P97's verdict OVERWRITES that file. So the guard works only between the housekeeping edit and P97 OVERWRITE. After P97, the guard has nothing to check. Should clarify the guard is provisional / superseded by the new verdict. The `Or rename the script to .disabled until P97 GREEN` half-sentence resolves it but is offered as alternative, not as the canonical path.

### M4 — README:64 trail order vs `_archive/` mtimes

The trail order in `README.md:64` ("HANDOFF → SYNTHESIS-VERIFICATION → DECISIONS-NEEDED → READY-TO-EXECUTE → COMPLETENESS-CHECK-2") is plausible chronologically but unverifiable from the bundle alone (mtimes of `_archive/*.md` would resolve it; one-line note on which is the latest revision would help cold readers route). Minor but the README presents this as authoritative session-history, which it isn't.

### M5 — REMEDIATION §5 effort table P95 = "5d (split candidate)"; rest of doc says ~30+h = ~4 days at 8h/day

`REMEDIATION-PLAN.md:350` says P95 effort `5 days (15 REQ-IDs at average S = ~30+h)`. ~30+h at 8h/day is closer to 4 days, not 5. The "split candidate" caveat partially absorbs this. Inflates the total by 1 day.

---

## WEAK findings (one-line each)

- **W1** — REMEDIATION-PLAN.md:14 inventory says "~43 explicit + ~8 cluster-folded" but section title "(~43 explicit + ~8 cluster-folded, classified)" — minor word duplication.
- **W2** — REMEDIATION-PLAN.md:190 still says "this list seeds P92 + P94 + P95 work" — math is correct post-renumber, but a cold reader unfamiliar with the renumber may second-guess; one-line "post-renumber: P94 = bus-push, P95 = catalog migration" anchor would help.
- **W3** — README.md:46 says "Two-channel rule" but doesn't link to where in CLAUDE.md / archived docs the rule's full text lives (SESSION-2026-05-08-HANDOFF item 5).
- **W4** — Archived `READY-TO-EXECUTE.md:10` and `_archive/DECISIONS-NEEDED.md:10` reference "P89–P96" — pre-renumber. Acceptable as historical trail per the prompt, but worth flagging for archaeology.
- **W5** — REMEDIATION-PLAN.md:14 inventory paragraph and §1 row-tables don't agree on "8 cluster-folded" arithmetic (44 H-rows + cluster covers ≈ 48 mapped findings, not 51). Already audited in `_archive/SYNTHESIS-VERIFICATION.md` "Stale-claim section" #3 — preserved unchanged.
- **W6** — `README.md:33` reading-order step 4 sends cold agent to STRATEGIC-REFRAME § Q5 only; no reference to the historical-only nature of the file (it predates Decisions 1–4). Cold agent may assume Q5 is current; in fact Q5 was authored before the four Decisions ratified.

---

## Things the consolidation got right

The decision absorption is surgically correct: **all 4 Decision SCs trace into the right post-renumber phases** (D1 in P91/P92/P93/P94 SCs; D2 in new P93 with RBF-LR-01..05 + §6 deferral table updated at line 442; D3 in RBF-FW-11 + RBF-FW-12 with effort bumped to 5–6d; D4 in P96 RBF-S-05 + P97 RBF-G-04 with verdict-file overwrite). STRATEGIC-REFRAME.md was correctly left untouched (mtime 15:00, predates 16:30 consolidation activity); README's archive trail is honest about the 5 superseded files; PATTERNS' meta-pattern paragraph is preserved verbatim.

---

**End of consolidation review.**
