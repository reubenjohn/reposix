# 91-PLAN-CHECK.md — fresh-eyes plan check for Phase 91

**Checker:** independent (no part in authoring the plans). **Base:** main @ 32ba856.
**Verdict:** **AMEND — 2 MUST-FIX.** Coverage of ROADMAP requirements is complete; the
two MUST-FIX items are both catalog-honesty defects that either SystemExit the catalog
load or contradict a ratified decision. Reality spot-checks of code citations all held
(builder.rs:90, path.rs:57-72, fast_import.rs:63/156-157, diff.rs:74-77/106/169-191,
main.rs:433, precheck.rs:151, sync.rs:44/94, reconciliation.rs:60/182, backend_dispatch.rs
pub items) — the line inventory is accurate.

---

## MUST-FIX

### MF-1 — `coverage_kind: "sim"` is not a valid value → catalog load SystemExit on the FIRST row P91 mints
**Where:** `91-01-PLAN.md:100` (`- coverage_kind: "sim"`); echoed at `91-OVERVIEW.md:55`
(`coverage_kind: sim`) and `91-DECISIONS.md:141` (D91-11: "`coverage_kind: sim`").
**Evidence:** `quality/runners/_audit_field.py:54` — `_COVERAGE_KINDS = {"real-backend",
"sim-only", "mechanical", "manual"}`. `_audit_field.py:278-284` raises `SystemExit` for any
row whose `coverage_kind` is non-null and not in that set. `"sim"` is not a member; the valid
spelling is `"sim-only"`. No existing catalog row uses `coverage_kind` at all
(`grep '"coverage_kind"' quality/catalogs/` → empty), so there is no sibling to copy the
correct value from — the plan invented `"sim"`. This defeats the plan's own stated goal
("get `coverage_kind` right FIRST TRY or catalog load SystemExits", 91-01:53) and the
`run.py` load in Task 1's `<verify>` would crash on the mint commit.
**Amendment:** Set row #1's `coverage_kind` to **`"mechanical"`** (most accurate — it is a
grep + cargo-unit proof, not even a sim-HTTP proof) or `"sim-only"`. Since row #1 sets
`transport_claim: false`, `coverage_kind` is *optional* (the `is_transport_claim` gate at
`_audit_field.py:266` is suppressed) — omitting it entirely is also valid and simplest. Fix
the value in 91-01, 91-OVERVIEW, and D91-11 so the executor does not re-introduce `"sim"`.

### MF-2 — `real-git-push-e2e` `minted_at` instruction contradicts ratified D91-11 and postdates an existing row
**Where:** `91-02-PLAN.md:192` ("add minted_at \"2026-07-05T00:00:00Z\"") and
`91-OVERVIEW.md:62` ("adds `minted_at`") vs. **`91-DECISIONS.md:143` (D91-11): the legacy
`real-git-push-e2e` row "keeps legacy status (**no minted_at added**)".**
**Evidence:** The row was minted 2026-07-04 with `last_verified: "2026-07-04T08:37:48Z"`
(`quality/catalogs/agent-ux.json:1364`), i.e. *before* `P90_MINT_CUTOFF = 2026-07-05`
(`_audit_field.py:42`). `minted_at` is a **write-once honesty anchor** (`_audit_field.py:233-243`);
stamping it `2026-07-05` on a row actually minted the day before is the exact backdate/postdate
dodge the field guards against, and directly contradicts the ratified decision. The executor
cannot obey both 91-02 and D91-11. (Load would not crash — the row already carries a long
`claim_vs_assertion_audit` at line 1373 — but the honesty rule is violated.)
**Amendment:** Follow D91-11: on `real-git-push-e2e` **do NOT add `minted_at`**; only remove
the `waiver` block, re-add `pre-pr` to `cadences`, and set `transport_claim: false`. As a legacy
row (no `minted_at`, `last_verified` < cutoff) it stays exempt from the coverage_kind gate and
loads clean. Reword 91-02:192 and 91-OVERVIEW:62 to strike "add minted_at".

---

## SHOULD-FIX

### SF-1 — `run.py --dry-run` flag does not exist; every 91-01 `<verify>` command is wrong as written
**Where:** `91-01-PLAN.md:103, 137, 164` all invoke `python3 quality/runners/run.py --cadence
on-demand --dry-run`.
**Evidence:** `quality/runners/run.py:380` defines only `--cadence` (required, `choices`).
argparse rejects unknown `--dry-run` with a nonzero exit and no load. The plan *does* hedge
this in `what_to_notice` (91-01:185-186), so an executor can recover, but the literal verify
commands fail. **Amendment:** Change the three commands to `python3 quality/runners/run.py
--cadence on-demand 2>&1 | head` and instruct the executor to confirm no `SystemExit:` /
`FAIL: ... row ...` line appears (the catalog LOAD precedes gate execution, so load-honesty is
provable even though on-demand also executes the exit-75 ql-001 skeleton).

### SF-2 — "RED-if-bug-returns" for the re-keyed fixtures is enforced only by executor prose, not mechanically
**Where:** `91-02-PLAN.md:178, 181` — "git-stash the fix, run the re-keyed tests, confirm they
FAIL, unstash. Record the RED observation in the SUMMARY."
**Evidence:** The ql-001 verifier (`ql-001-canonical-path.sh`) asserts grep canonicality + that
the regression test *names* exist — it never asserts those tests are RED against the unfixed
planner. A re-key that stays green with the bug present would pass the gate and the SUMMARY
claim is unfalsifiable by the framework. Overview risk #3 acknowledges the hazard but nothing
gates it. **Amendment:** Acceptable under the ownership charter + verifier-subagent read, but
add to 91-02's `<done>`: paste the actual `cargo test` FAIL output (test name + assertion line)
into the SUMMARY, not a prose "confirmed RED" — gives the verifier subagent a checkable artifact.

### SF-3 — RBF-A-05 ownership is duplicated across 91-04 and 91-05
**Where:** `91-04-PLAN.md:14` (`requirements: [RBF-A-05]`) and `91-05-PLAN.md:11`
(`requirements: [RBF-A-05]`).
**Evidence:** ROADMAP maps RBF-A-05 to the dvcs-third-arm populate (SC-4) = 91-04. 91-05 is the
litmus rewrite (SC-5/SC-6, D91-06/07); its RBF-A-05 tag appears to be the "populated-mirror
analog" mention (D91-07) copied into the frontmatter. Two plans claiming the same requirement
muddies the coverage ledger and the verifier's per-requirement grading.
**Amendment:** Drop `RBF-A-05` from 91-05's frontmatter (leave the mirror-population as litmus
prep under D91-07); or, if kept, annotate it as "RBF-A-05 (real-mirror analog only; primary owner
91-04)". No behavioral change.

---

## NIT

- **N-1 (`91-01-PLAN.md:99-101`):** Row #1 sets `transport_claim: false` *and* `coverage_kind`.
  With `transport_claim: false` the coverage_kind requirement is suppressed; the cleanest mint
  omits `coverage_kind` entirely rather than carrying a value that must be kept valid. (Folds
  into MF-1's fix.)
- **N-2 (`91-01-PLAN.md:120`):** Row #2 cites `verifier.script:
  quality/gates/agent-ux/attach-sync-real-backend.sh`, which is not created until 91-03. Harmless
  — the row is `pre-release-real-backend` cadence (not run at load, pre-push, or on-demand), and
  pushes only land at 91-06 after 91-03 ships the script. Worth a one-line note in the SUMMARY so
  a mid-phase `run.py` inspector isn't surprised by the dangling path.
- **N-3 (`91-05-PLAN.md:12-17`):** must_haves `truths` assert the script "drives vanilla-clone +
  attach + edit + push + dual-audit" but the executor's `<done>` (91-05:104) only checks the
  env-unset exit-75 path. The end-to-end drive is verifiable only by the coordinator (creds). This
  is by design (checkpoint task), but the truths read as executor-verifiable when they are not —
  consider tagging them "(coordinator-verified)".

---

## Coverage ledger (all mapped; no orphans)

| Req | Owner plan | Req | Owner plan |
|-----|-----------|-----|-----------|
| RBF-A-01/02 (attach/sync dispatch) | 91-03 | QL-001 crit 1-6 + waiver retire | 91-02 |
| RBF-A-03 (token scrub) | 91-03 | SC-1 (attach 5-case reconcile) | 91-03 + 91-04 |
| RBF-A-04 (real smokes) | 91-03 | SC-2 (real smokes) | 91-03 |
| RBF-A-05 (dvcs-third-arm) | 91-04 (dup 91-05 → SF-3) | SC-3 (no P-token stderr) | 91-03 |
| RBF-A-06 (REQUIREMENTS flip) | 91-06 | SC-4 (dvcs-third-arm counts) | 91-04 |
| RBF-A-07 (CLAUDE.md example) | 91-06 | SC-5/6 (T2 + sanctioned-body) | 91-05 + coordinator |
| D91-01..12 | all mapped | SC-7/8/9 | 91-06 / 91-06+coord / 91-02 |

Charter obligations present: CLAUDE.md same-PR (91-06 T1), banned-token grep verify (91-03 T2
`<verify>`), OP-3 `with_audit` on attach/sync + ForkAsNew create (D91-03/04, 91-03), mkdocs gates
on docs edits (91-04 T2, 91-06 T2). Catalog-first integrity holds: 91-01 is implementation-free
(only catalog rows + an exit-75 verifier skeleton). Wave serialization is one-cargo-at-a-time
realistic (the single `cargo build --workspace --bins` in 91-04 is called out and isolated).

---

## Amendments applied (planner, 2026-07-04)

All findings addressed; checker findings above retained verbatim as the record.

- **MF-1** — `coverage_kind: "sim"` eliminated everywhere. 91-01 Task 1 now instructs OMITTING
  `coverage_kind` on row #1 (transport_claim: false suppresses the gate; valid enum
  {real-backend, sim-only, mechanical, manual} cited inline, "mechanical" named as the fit if
  carried — folds in N-1). 91-01 interfaces block corrected (`sim-only`/`mechanical`/`manual`
  valid-but-insufficient; `"sim"` not a member). 91-OVERVIEW row-1 bullet rewritten to match.
  91-DECISIONS D91-11 corrected inline with an "(amended per PLAN-CHECK MF-1)" annotation —
  not silently rewritten.
- **MF-2** — 91-02 Task 4 now says "do NOT add minted_at" for `real-git-push-e2e` (legacy row
  per D91-11; backdate/postdate rationale quoted inline). 91-OVERVIEW edited-rows bullet now
  reads "keeps legacy status (NO minted_at, per D91-11 / PLAN-CHECK MF-2)".
- **SF-1** — all three `--dry-run` invocations in 91-01 replaced with
  `python3 quality/runners/run.py --cadence on-demand 2>&1 | head -20` + instruction to confirm
  no SystemExit/validate_row error (load precedes gate execution; exit-75 skeleton run expected).
  The obsolete what_to_notice hedge about --dry-run removed.
- **SF-2** — 91-02 Task 3 action + done now require pasting the actual `cargo test` FAIL output
  (test names + assertion lines) verbatim into the SUMMARY as a verifier-checkable artifact.
- **SF-3** — 91-05 frontmatter annotates RBF-A-05 as "(real-mirror analog; primary owner 91-04)"
  with a YAML comment directing the verifier to grade RBF-A-05 against 91-04. (Kept non-empty
  per the plan-frontmatter rule; annotation option chosen over deletion.)
- **N-1** — folded into MF-1 (coverage_kind omitted on row #1).
- **N-2** — 91-01 what_to_notice now carries the dangling-verifier-path note (row #2 cites
  attach-sync-real-backend.sh which 91-03 creates; harmless, SUMMARY note required).
- **N-3** — 91-05 truths tag the end-to-end drive as "(coordinator-verified — requires creds;
  executor verifies the env-unset exit-75 path only)".

Not amended (out of checker scope, noted for the record): 91-04's "add minted_at" on
`dvcs-third-arm` stands — that row's re-verification lands post-cutoff, where minted_at is
load-required (framework A(a)); the checker flagged only the real-git-push-e2e instance,
whose exemption is explicitly ratified by D91-11.
