---
ratifier: fresh unbiased subagent (zero prior context)
date: 2026-07-13
head_at_ratification: b8e309f
graded_from: quality/catalogs/agent-ux.json @ 4645060; VERDICT.md @ b8e309f
result: GREEN-WITH-RECORDED-CAVEATS
---

# v0.14.0 Milestone-Close — Independent Ratification

**Verdict under review:** `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md`
(`head_at_grade: 4645060`), claiming **GREEN-WITH-RECORDED-CAVEATS — READY TO TAG (with
recorded caveats)** under **Manager Ruling #4 / Option B**.

This ratification was performed with zero prior session context, per the reposix
milestone-close doctrine (`quality/CLAUDE.md` § "Verifier-subagent dispatch"). No cargo
was invoked, no cadence was re-run, no real-backend calls were made. This is a
documentary re-grade from committed artifacts only, matching the established pattern —
the transient JSONs under `quality/reports/verifications/agent-ux/*.json` were
deliberately NOT used as evidence (they are gitignored scratch, clobbered after grading
by an unrelated background creds-absent run); the committed catalog
(`quality/catalogs/agent-ux.json`) is authoritative.

## RESULT: **GREEN-WITH-RECORDED-CAVEATS** — ratified, tag-ready

All 7 criteria PASS. No defect found. The verdict is honest, the evidence is real, and
the process integrity (no waiver, no catalog surgery, no gate-policy change) holds.

## Per-criterion grading

### 1. The verdict never claims the cadence exited 0 — PASS

`VERDICT.md` frontmatter line 8: `aggregate_real_backend: "5 PASS / 0 FAIL / 1
NOT-VERIFIED — cadence exit 1 (NOT exit 0; see t4 caveat)"`. Body opens (line 20-22):
"**The `pre-release-real-backend` cadence exited 1 — NOT 0.**" and repeats this framing
in "How this went from RED to GREEN..." (line 61: "Cadence exit code: 1"), in the t4
caveat section (line 138-142: "this document states the cadence exit code (1) in its
first paragraph and its frontmatter, never claiming exit-0"), and in the closing verdict
line (line 200-206: "the `pre-release-real-backend` cadence honestly exits **1**"). Grep
of the full file for "exit-0" / "exit 0" surfaces only negated/conditional uses (e.g.
"never claiming exit-0", "would flip to PASS only when..."). No affirmative exit-0 claim
anywhere. Tag-readiness is explicitly attributed to Ruling #4, never to the cadence
passing.

### 2. The sole NOT-VERIFIED (t4) is an honest environment floor, not a silent waiver — PASS

Read `quality/catalogs/agent-ux.json` row `agent-ux/t4-conflict-rebase-ancestry-real-backend`
directly (dumped in full during this ratification): `"status": "NOT-VERIFIED"`,
`"waiver"` key present with value `null`, `"blast_radius": "P0"`,
`"claim_vs_assertion_audit"` intact (not stripped), `owner_hint` names the git-floor gap
factually. Diff-audited commit `4645060` (`git show 4645060 -- quality/catalogs/agent-ux.json`):
the row's status field flips `FAIL` (19:06:07Z, pre-Ruling#3-fix) → `NOT-VERIFIED`
(23:54:53Z, post-`cb8ad11` fix) — a 2-line mechanical diff (`status` + `last_verified`
only), consistent with a runner persist path, not a hand-authored rewrite. No `waiver`
block was added at any point in the commit chain (`cb8ad11` → `4645060` → `b8e309f`);
`cb8ad11`'s only catalog touch was two `owner_hint` prose corrections (verified via `git
show cb8ad11 -- quality/catalogs/agent-ux.json` — the diff hunks show only the
`owner_hint` line changing, `status`/`waiver` lines are unchanged context). No gate-policy
script (`quality/runners/`, `quality/PROTOCOL.md` mechanics) was touched by any commit in
the Ruling #3/#4 arc — only the two per-row verifier shell scripts
(`quality/gates/agent-ux/{t4-conflict-rebase-ancestry-real-backend,github-front-door-real-backend}.sh`).

### 3. Load-bearing GREEN evidence is real (litmus + p93, both P0, PASS against live TokenWorld) — PASS

Catalog: both rows `status: "PASS"`, `blast_radius: "P0"`, `waiver: null`,
`coverage_kind: "real-backend"`. Read the cited transcripts directly:

- `quality/reports/transcripts/milestone-close-vision-litmus-real-backend.txt` (ts
  `2026-07-13T23:54:45Z`, `exit_code: 0`) shows a real vanilla clone, `reposix attach
  confluence::REPOSIX`, a reconciliation (`matched=3 backend_deleted=0`), an edit to
  `pages/2818063.md` (protected-denylist honoured — the sacrificial page, not a protected
  one), a real `git push reposix main` with a secret-scan pass, REST-confirmed server-side
  change, dual-table audit rows, and mirror-ref advancement. This is a genuine
  subprocess/network transcript, not a synthetic assertion.
- `quality/reports/transcripts/p93-partial-failure-recovery-real-confluence.txt` (ts
  `2026-07-13T23:54:53Z`, `exit_code: 0`) shows `cargo test -p reposix-cli --test
  agent_flow_real partial_failure_recovery_real_confluence -- --ignored --exact` →
  `test result: ok. 1 passed` in 9.04s — a real `#[ignore]` real-backend smoke, not a
  sim/mock run (the test name and file are explicitly the real-Confluence variant per
  `crates/reposix-cli/tests/agent_flow_real.rs`).

Both transcripts' timestamps (23:54:4x/23:54:5x UTC on 2026-07-13) post-date the item-5
ADF fix (`49666eb`, 2026-07-13T20:48:15Z) that was the root cause of the prior RED verdict
— confirmed via `git show -s --format='%ci'` cross-reference. This closes the obvious
question of whether these PASS grades could be stale pre-fix residue: they are not.

### 4. OP-9 retro distillation exists with the three verbatim caveats — PASS

`.planning/RETROSPECTIVE.md` § "Tag-prep arc (fix-first items 4–8) — item-5 ADF
regression, litmus non-idempotency, and the three tag rulings" (lines 90-160) contains
all three caveats verbatim, block-quoted exactly as required:
- Ruling #2 (litmus non-idempotency, RULED-DEFER→v0.15.0) — line 131-132.
- Item-7 CREATE-recovery WAIVED — line 134-135.
- Ruling #4 (t4 git-floor caveat, RULED OPTION-B, commit `dcf49d4`) — line 153-156,
  reproducing all three binding conditions verbatim.

These match `VERDICT.md`'s own quotations of the same rulings word-for-word (cross-checked
line by line).

### 5. Ruling #4 is properly recorded in `.planning/CONSULT-DECISIONS.md` — PASS

Two entries confirmed present: the OPEN git-floor escalation
("## 2026-07-13 [OWNER] v0.14.0 tag: pre-release-real-backend blocked by a THIRD gap...",
lines 13-64) whose `Status:` field (line 63-64) reads "RULED OPTION-B — Manager Ruling #4
(commit `dcf49d4` ...)" — the flip from "Decision-pending" to ruled is present, not left
dangling — and the Ruling #4 block itself
("## 2026-07-13 [MANAGER] Ruling #4 (E3 valve) — OPTION B...", lines 66-94) carrying the
three binding conditions verbatim and a `Status:` of "RULED OPTION-B — successor #13
executing STEP 3.3 mechanicals to READY-TO-TAG."

### 6. No P0 waiver / catalog surgery / gate-policy code change used to manufacture GREEN — PASS

Confirmed via direct row inspection (criterion 2) that `waiver: null` on all six graded
rows including both P0 rows. Confirmed via `git show cb8ad11` that the harness-gap fix
touched exactly two verifier shell scripts
(`quality/gates/agent-ux/github-front-door-real-backend.sh`,
`.../t4-conflict-rebase-ancestry-real-backend.sh`) plus prose-only `owner_hint` catalog
edits — no `status`/`waiver` field was hand-set, no runner/gate-policy engine code
(`quality/runners/`, `quality/PROTOCOL.md`) was touched. Ruling #3's own binding
guardrails (quoted in `RETROSPECTIVE.md` line 137-146) explicitly required "row status
left runner-minted, never hand-set," and the commit trail honors that.

### 7. TokenWorld end-state is the sanctioned 2 protected + sacrificial, no residue — PASS

Cross-referenced three independent sources, all consistent: (a) the litmus transcript
itself — "edit target: .../pages/2818063.md (protected denylist honoured)"; (b)
`.planning/CONSULT-DECISIONS.md` line 29-30 — "TokenWorld end-state INTACT (2 protected
`7766017`/`7798785` + sacrificial `2818063`; t4 bailed pre-mutation, no residue)"; (c)
`VERDICT.md` § "TokenWorld end-state" — "2 protected fixtures (`7766017`, `7798785`)
intact, plus sacrificial page `2818063` present and current." No live TokenWorld query was
made by this ratifier (out of scope per the documentary-ratification charter); this is a
documentary cross-check of consistent, independently-authored records, not a live
re-verification.

## Noticing

- **Cosmetic catalog-persist gap, correctly self-disclosed.** The
  `github-front-door-real-backend` row carries `status: "PASS"` (the operative grade) but
  also stale `skip_reason: "env-missing"` / `last_real_grade: null` fields left over from
  an earlier env-gated skip — confirmed present in the row as of `4645060`. `VERDICT.md`
  already flags this itself under "Awareness note — non-blocking cosmetic artifact gap"
  and does not hand-edit it away. This ratifier concurs it is cosmetic (the top-level
  `status` field the runner and `run.py --cadence` exit-code logic actually read is
  correct) but agrees it is a real runner-persist-path bug worth a v0.15.0 follow-up: the
  persist path should clear stale `skip_reason`/`last_real_grade` fields when a row
  transitions from skipped to executed.
- **`attach-sync-real-backend`'s PASS predates the item-5 ADF fix, but legitimately.** Its
  `last_verified` (`2026-07-13T19:06:17Z`) is earlier than the item-5 fix landing
  (`49666eb`, `20:48:15Z`) — at first glance this looks like potentially-stale evidence
  for a P1 real-Confluence row. Traced it down: `git show 82498cc` shows litmus/p93 (which
  DID depend on the ADF bug) were FAIL at the same ~19:06Z timestamp and only flipped PASS
  after the fix, while attach-sync stayed PASS the whole time — because `reposix attach` +
  `sync --reconcile`'s assertions (`crates/reposix-cli/tests/agent_flow_real.rs`
  `assert_sync_reconcile_ok`) check config wiring + a reconcile-summary string, not page
  body/ADF content, so the row never touched the buggy code path. This is legitimate, not
  stale evidence — but it is also independently corroborated by an already-filed OPEN item
  in `.planning/RETROSPECTIVE.md` line 100-102: a B3 re-run separately found
  `attach-sync-real-backend`'s PASS "coverage-hollow (never exercises the
  `refs/reposix/origin/main` round-trip — OPEN→v0.15.0)." That gap is already tracked and
  routed; not a new finding, but worth surfacing here since it touches one of the six
  graded rows.
- The verdict's own honesty is unusually strong for a "GREEN" document — it repeats the
  exit-1 fact five separate times rather than once, and dedicates an entire section to the
  one caveat rather than minimizing it. This is the correct posture for a
  GREEN-WITH-RECORDED-CAVEATS state and made this ratification faster to perform, not
  slower.

## Final verdict

**GREEN-WITH-RECORDED-CAVEATS — READY TO TAG.** All 7 required criteria PASS with cited,
independently-checked evidence. The milestone-close posture is genuine: the cadence
honestly exits 1, the sole non-PASS row is a documented environment floor under a properly
recorded manager ruling with no waiver/surgery/policy-change, and both P0 real-backend
rows that could execute did PASS against live TokenWorld with real transcripts to show for
it.
