# Phase P88 Audit — good-to-haves polish + milestone close
**Auditor:** unbiased subagent (zero session context)
**Date:** 2026-05-08

## Verdict at a glance
- ALIGNED items: 3
- MISALIGNED items: 7
- SUSPECT items: 2

P88 was the v0.13.0 milestone-close phase. It minted 4 catalog rows, drained 1
GOOD-TO-HAVES entry, wrote a CHANGELOG section + tag-script + RETROSPECTIVE
distillation, and dispatched a milestone-close verdict. Verdict GREEN. But the
post-milestone evidence is unambiguous: P88's catalog rows are file-presence
checks, not honesty checks; its deferral rationale was demonstrably false (the
"deferred" item was implemented 23 minutes after milestone close); its
RETROSPECTIVE distillation does not mention a single one of the 16 HIGH
real-backend frictions the dark-factory exercise found 24 hours later; and the
milestone-close verdict text silently dropped the SC5 "TokenWorld arm GREEN"
clause without a deferral note.

## Findings

### F1 — All 4 P88 catalog rows are presence-only / structure-only checks; none grades content honesty [SEVERITY: HIGH]
**Claim in plan:** "TINY shell verifiers under `quality/gates/agent-ux/`" that establish "the eventual GREEN contract" for milestone-close (88-01-PLAN.md:57-62, 67-71). Plan claims verifiers cover "GOOD-TO-HAVES drained, CHANGELOG entry present, tag-script present, RETROSPECTIVE distilled."
**Reality:** Each verifier asserts file-presence + structural shape only:
- `p88-good-to-haves-drained.sh:30,40-44` — counts `## GOOD-TO-HAVES-NN` headings, counts `STATUS: (RESOLVED|DEFERRED|WONTFIX)` lines, asserts terminal_count >= entry_count. No check that DEFERRED items have a target milestone, that rationale is non-empty, or that RESOLVED items name a commit SHA. A `STATUS: WONTFIX` with empty rationale would PASS.
- `v0.13.0-changelog-entry-present.sh:24-39` — checks `## [v0.13.0]` heading exists + section has ≥10 non-blank lines. No check that the entry references shipped REQ-IDs, names carry-forwards, or matches the SC2 contract ("every shipped REQ-ID category named, v0.14.0 carry-forwards named"). 10 lines of `aaaaa` would PASS.
- `v0.13.0-tag-script-present.sh:29-37` — checks file exists, executable, has ≥6 `# Guard N:` comments, contains `git tag -s`. **The verifier never executes the script** — does not verify guards actually run `git diff --quiet`, that the cargo-test guard runs cargo test, etc. Six guards that all `echo "guard N: PASS"` would PASS.
- `v0.13.0-retrospective-distilled.sh:42-46` — checks 5 specific OP-9 subheading strings exist as `^#{2,4} <heading>$` markdown headings. No check that subheadings have substantive content (a section with `### What Was Built` and one blank line under it would PASS).

**Evidence:**
- `quality/gates/agent-ux/p88-good-to-haves-drained.sh:1-66`
- `quality/gates/agent-ux/v0.13.0-changelog-entry-present.sh:1-43`
- `quality/gates/agent-ux/v0.13.0-tag-script-present.sh:1-42`
- `quality/gates/agent-ux/v0.13.0-retrospective-distilled.sh:1-58`
- 88-01-SUMMARY.md:84-87 lists assertions actually run; all four are structural.
**Why it matters:** This is the user's hypothesis confirmed: P88 catalog rows are rubber stamps. Failure-mode #1 from the audit brief (test name promises one thing, assertions deliver less) — the catalog row description "GOOD-TO-HAVES drained" implies semantic completion; the verifier only checks that *some* terminal STATUS string is grep-able. A future agent reading the GREEN verdict would believe content-correctness was graded; nothing was graded.

### F2 — Deferral rationale for GOOD-TO-HAVES-01 was empirically false: "deferred" work shipped 23 minutes after milestone close [SEVERITY: HIGH]
**Claim in plan:** GOOD-TO-HAVES-01 (extend `reposix-quality bind` to all dimensions) "DEFERRED to v0.14.0" because "the extension requires Rust + tests + cross-dimension schema design that's outside P88's docs+catalog+shell scope" (88-01-PLAN.md:15). Decision PCT-04 (88-01-SUMMARY.md:48-49) elaborates: "the work is well-scoped at S size but doesn't fit P88's pure-docs envelope; doing it here would double the phase's scope per the OP-8 'scope-creep-to-fit-the-finding' anti-pattern."
**Reality:** Commit `fd2e247` ("feat(reposix-quality): bind --dimension agent-ux (closes 80% of GOOD-TO-HAVES-01)") landed on `2026-05-01 15:49:43 -0700` — 23 minutes after the P88 close commit `ad576ae` at `15:26:14`. The commit shipped 824 insertions across 5 files (`commands/agent_ux.rs:230 lines`, `tests/bind_agent_ux.rs:464 lines`, `main.rs:+129 lines`, etc.) including 9 integration tests. The work was "well-scoped at S size" — and was actually delivered in roughly the same wall-clock time it took to claim it didn't fit.
**Evidence:**
- `git log --pretty="%h %ai %s" e32c20a^..fd2e247` shows the timestamps.
- Commit `fd2e247` (`feat(reposix-quality): bind --dimension agent-ux (closes 80% of GOOD-TO-HAVES-01)`).
- Commit `ad576ae` (P88 telemetry tick — terminal P88 commit, 15:26:14).
- 88-01-PLAN.md:15 — explicit DEFERRED rationale.
- 88-01-SUMMARY.md:48-49 — Decision PCT-04 says S items "default-defer to v0.14.0."
- Updated GOOD-TO-HAVES.md:24 now reads `**STATUS:** PARTIAL — Path A shipped post-milestone; Path B (remaining 7 dimensions...) DEFERRED to v0.14.0` — STATUS retroactively edited from `DEFERRED` to `PARTIAL`.
**Why it matters:** This is failure-mode #6 from the audit brief (velocity-as-skip-signal). The "pure-docs envelope" framing was a procedural shield, not a real constraint. The closer reading: P88's plan-author decided the work was out-of-scope for the discovering phase; the next-session author disagreed and shipped it 23 minutes later. The decision-of-record should have been "fix it now per OP-8 eager-resolution preference," not "defer." Worse, the milestone-close verdict graded GREEN against a checked DEFERRED status, and the deferral was reverted out-of-band post-tag — a quality-framework integrity issue (the verdict file claims tag-readiness based on a state that was overwritten 23 minutes later).

### F3 — RETROSPECTIVE distillation captures zero of the 16 HIGH real-backend frictions [SEVERITY: HIGH]
**Claim in plan:** "RETROSPECTIVE.md v0.13.0 distillation per OP-9 ritual: What Was Built / What Worked / What Was Inefficient / Patterns Established / Key Lessons. Source material: SURPRISES-INTAKE + GOOD-TO-HAVES + per-phase verdicts P78-P87 + autonomous-run findings" (88-01-PLAN.md:107-110, ROADMAP.md:125). OP-9 (CLAUDE.md): "Without this step, learnings get lost in milestone archives — the +2 phase practice produces signal that's worth keeping cross-milestone (failure modes, patterns, process gaps)."
**Reality:** The RETROSPECTIVE.md v0.13.0 section (lines 15-51) lists 5 "milestone-specific lessons" + 4 "v0.13.0-specific patterns" + 4 "inefficiencies" + 3 carry-forwards. Not one of these references:
- Cluster A — `reposix attach` rejecting all non-sim backends with an internal-phase-ID-leaking error (audit `git grep "P79-02 scaffold"` returns the live error string in production code).
- Cluster B — `git pull --rebase` recovery broken (helper-side fetch mints fresh root commits with no ancestry; load-bearing v0.9.0 architectural cornerstone).
- Cluster C — OP-3 audit-log violated on every `git push` (zero `helper_push_*` rows; cache.db never created on the helper-push path).
- Cluster D — Bus-push helper rejects exactly the files the documented mirror-setup tells the user to commit.
- Cluster E — `init` "Next:" hint failing on first contact.
- Cluster F — Tutorial expected output is fictional vs. the actual sim seed.
- Cluster G — Quality framework structurally exempting real-backend flows.

The dark-factory exercise that surfaced these (2026-05-02, 24h post-tag) found "37 frictions, 16 HIGH" against the v0.13.0 milestone — meaning 16 HIGH issues existed at the point P88 wrote the RETROSPECTIVE and were grep-able from the shipped code/docs. The distillation discusses telemetry-bearing catalog churn, the OP-9 ritual, eager-resolution carve-outs, and CARRY-FORWARD entries about gix yanked pins — none of which would have flagged the v0.13.0 failures.
**Evidence:**
- `.planning/RETROSPECTIVE.md:15-51` — full v0.13.0 distillation section.
- `.planning/research/v0.13.0-real-backend-frictions/SUMMARY.md:18-110` — 37 frictions / 8 clusters / dated 2026-05-02 (1 day post-P88).
- `.planning/research/v0.13.0-real-backend-frictions/T1-sim-baseline.md` and T2/T3/T4 — per-cluster evidence.
- Even the `RETROSPECTIVE-FULL.md:21` mentions "TokenWorld arm SUBSTRATE-GAP-DEFERRED" — the only acknowledgment of the real-backend gap in the entire milestone-close artifact set.
**Why it matters:** Failure-mode #4 from the audit brief (project's own non-negotiable invariants violated silently). OP-9 codifies the RETROSPECTIVE ritual specifically because "raw intake format is too granular for future readers to skim — the +2 phase practice produces signal that's worth keeping cross-milestone (failure modes, patterns, process gaps)." P88 produced a clean-looking distillation that was *uninformed by the actual reality of the shipped milestone* — because no part of P88's process was wired to interrogate real-backend behavior. The OP-9 ritual passed the verifier and missed its actual purpose.

### F4 — Milestone-close verdict silently dropped SC5's "TokenWorld arm GREEN" clause [SEVERITY: HIGH]
**Claim in plan:** ROADMAP P88 SC5 (lines 126): "Milestone-close verifier dispatched and GREEN at `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`: P78-P88 catalog rows all GREEN-or-WAIVED; **dark-factory three-arm transcript GREEN against sim AND TokenWorld**; no expired waivers without follow-up; RETROSPECTIVE v0.13.0 section exists; +2 reservation operational." 88-01-PLAN.md:50, 142-143 promotes this verbatim as the success criterion.
**Reality:** `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md:19` reports probe 4 as: `bash quality/gates/agent-ux/dark-factory.sh — exit 0 (sim arm DEMO COMPLETE)`. The TokenWorld arm is never mentioned. The verdict text contains zero references to `TokenWorld`, `three-arm`, `real backend`, or `REPOSIX space` (`grep -i` confirms). The SC5 contract was watered down silently from "AND TokenWorld" to "sim arm only."
**Evidence:**
- `.planning/milestones/v0.13.0-phases/ROADMAP.md:126` — "GREEN against sim AND TokenWorld" verbatim.
- `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md:19` — only sim-arm execution recorded.
- `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` full file — no TokenWorld / three-arm / real-backend mention anywhere.
**Why it matters:** Failure-mode #2 from the audit brief (substrate-gap deferrals masquerading as GREEN). SC5 made TokenWorld a hard milestone-close requirement; the verdict report responded by checking only sim and grading GREEN. A reader of the verdict would not know SC5 was not satisfied without comparing line-by-line against ROADMAP. This is the framework's biggest single failure mode — and it's the failure that allowed Cluster A (attach unimplemented for real backends) to ship with a checked DVCS-ATTACH-01 box.

### F5 — Catalog rows fossilize "PASS" state; underlying verifier currently FAILS [SEVERITY: MED]
**Claim in plan:** Catalog rows mint `status: PASS, last_verified: 2026-05-01T22:30:00Z`; the catalog-first contract is "rows defining the GREEN contract exist before the work; verifier subagent grades artifacts" (CLAUDE.md "Catalog-first rule").
**Reality:** Re-running `bash quality/gates/agent-ux/p88-good-to-haves-drained.sh` against today's working tree returns:
```
FAIL: .planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md has 1 entry/entries but only 0 terminal STATUS lines
```
The catalog row in `quality/catalogs/agent-ux.json` still reads `status: PASS, last_verified: 2026-05-01T22:30:00Z`. The reason: the post-milestone GOOD-TO-HAVES.md edit (commit `fd2e247`) flipped STATUS from `DEFERRED` (verifier-matched) to `PARTIAL — Path A shipped post-milestone; Path B ... DEFERRED to v0.14.0`. The verifier's regex matches the literal first word after `STATUS:` and PARTIAL is not in the (RESOLVED|DEFERRED|WONTFIX) allow-list. So the verifier silently broke when the deferral was undone.
**Evidence:**
- `bash quality/gates/agent-ux/p88-good-to-haves-drained.sh; echo $?` returns FAIL / exit=1 against current HEAD.
- `quality/catalogs/agent-ux.json` row `agent-ux/p88-good-to-haves-drained` still has `status: PASS, last_verified: 2026-05-01T22:30:00Z`.
- `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md:24` — STATUS line: `**STATUS:** PARTIAL — ...`
- `quality/gates/agent-ux/p88-good-to-haves-drained.sh:42` — regex `STATUS[[:space:]]*\**:?\**[[:space:]]+(RESOLVED|DEFERRED|WONTFIX)`.
**Why it matters:** Tells you the catalog rows are write-once memorials, not running checks. A milestone-close row catches a one-shot snapshot, then never re-runs. If the artifact drifts (as it did within an hour of milestone close), the row stays GREEN. This is a cadence design gap — `cadence: on-demand` plus no auto-retrigger means the verifier is functionally inert post-archive.

### F6 — Tag-script verifier checks ≥6 guards but never validates that they fire correctly [SEVERITY: MED]
**Claim in plan:** "Tag-script authored at `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` with ≥6 safety guards (clean tree, on `main`, version match, CHANGELOG entry exists, tests green, signed tag); tag-gate guards re-run cleanly post-P88" (ROADMAP.md:124, 88-01-PLAN.md:85).
**Reality:** `v0.13.0-tag-script-present.sh:29-37` greps for `^# Guard [0-9]+:` comments. It never executes the script in a sandbox or validates that any guard's logic does what its comment claims. A tag-script with 8 guards each implemented as `# Guard 1: clean tree` followed by `true` would PASS. The script in question (tag-v0.13.0.sh) does have substantive guard logic (lines 14, 17, 22, 27, 31, 35, 39, 44), but the verifier-of-the-verifier is structural only — meaning a future tag-script regression that hollows out a guard's body but keeps the comment would not be caught.
**Evidence:**
- `quality/gates/agent-ux/v0.13.0-tag-script-present.sh:29-37` — `grep -cE '^# Guard [0-9]+:'`.
- `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh:13-46` — actual guard logic (well-formed, but unverified).
- ROADMAP P88 SC3 (line 124): "tag-gate guards re-run cleanly post-P88" — never machine-checked; the SC implicitly relies on owner running the tag-script (which the orchestrator must NOT do per SC6).
**Why it matters:** Failure-mode #1 (test promises X, assertions deliver less). The catalog row's `description` ("Asserts the v0.13.0 tag-script exists, is executable, has >=6 `# Guard N:` comments, and contains a `git tag -s` invocation" — `agent-ux.json`) is honest about what's checked, but the row's status PASS is taken as evidence that the *tag-cut workflow is sound*, which it is not. The verifier does not catch hollowed-out guards; the tag-script's own guards run only when the owner runs it.

### F7 — DVCS-GOOD-TO-HAVES-01 trace-table marked "shipped" but the underlying entry was DEFERRED at milestone close [SEVERITY: MED]
**Claim in plan:** `.planning/milestones/v0.13.0-phases/REQUIREMENTS.md:161` — `| DVCS-GOOD-TO-HAVES-01 | P88 | shipped |`. Milestone-close verdict (`milestone-v0.13.0/VERDICT.md:48`): "0 deferred (DVCS-GOOD-TO-HAVES-01 deferral is intra-milestone; trace-table row is `shipped` because the +2 phase ran and produced terminal STATUS)."
**Reality:** The verdict's framing is a sleight of hand. DVCS-GOOD-TO-HAVES-01 is a meta-REQ ("the +2 phase drained the file") and that meta-REQ did fire. But the entry that file contained — extending `reposix-quality bind` to all dimensions — was DEFERRED, not shipped. Marking the meta-REQ "shipped" hides the fact that the underlying work was kicked to v0.14.0. (And, per F2, was actually shipped 23 minutes later; so the trace-table is now self-inconsistent in a different way — the DEFERRED rationale is stale and the underlying work landed in-milestone-window-but-out-of-phase.)
**Evidence:**
- `.planning/milestones/v0.13.0-phases/REQUIREMENTS.md:161` — trace status `shipped`.
- `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md:48` — verdict's framing.
- `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md:24` — STATUS now PARTIAL.
- Commit `fd2e247` (post-tag-window).
**Why it matters:** Quality-framework integrity issue. The trace-table is the contract a future planner skims to learn what shipped; "shipped" against a deferred entry is misleading. Compounded by F2 (the deferral was reverted), the row's status is now "shipped via a meta-rule and also shipped via post-milestone work but neither path was the planned path."

### F8 — All 4 P88 catalog rows carry `_provenance_note: "Hand-edit per documented gap"` — the `bind` extension that *would have* removed this gap was the GOOD-TO-HAVES-01 deferred item [SEVERITY: MED]
**Claim in plan:** Decision PCT-04 (88-01-SUMMARY.md:48-49) explicitly says "provenance flag on hand-edited rows continues to mark Principle A bypass until the verb extension lands."
**Reality:** All 4 P88 catalog rows are hand-edited with `_provenance_note` ("Hand-edit per documented gap (NOT Principle A): see GOOD-TO-HAVES-01 (deferred to v0.14.0)"). The provenance-note self-references the deferred GOOD-TO-HAVES-01 — which (per F2) was actually completed 23 minutes after milestone close. The rows are still hand-edited today, post the v0.13.0 tag, despite the `bind --dimension agent-ux` verb shipping. No re-bind happened.
**Evidence:**
- `quality/catalogs/agent-ux.json` — each P88 row's `_provenance_note` field.
- Commit `fd2e247` shipped Path A (`agent-ux` dimension) + 9 integration tests but did not re-bind any of the 4 P88 rows through the new code path.
- GOOD-TO-HAVES-01 acceptance criterion (GOOD-TO-HAVES.md:18): "The P79-02 hand-edit of `agent-ux/reposix-attach-against-vanilla-clone` is rebound via the new code path (provenance flag retained or auto-cleared)" — also not actioned.
**Why it matters:** A documented contract (the provenance note's own promise: "until verb extension lands") was orphaned the moment the verb extension landed. This is auditable drift; the rows still claim Principle A bypass while the bypass is no longer needed.

### F9 — RETROSPECTIVE inefficiency 3 mentions the bind-extension gap but frames it as "operationally tolerable" [SEVERITY: LOW]
**Claim in plan:** OP-9 ritual is meant to surface "failure modes, patterns, process gaps" cross-milestone (CLAUDE.md OP-9).
**Reality:** RETROSPECTIVE.md:45 reads: "`reposix-quality bind` only supports `docs-alignment` dimension... Operationally tolerable; v0.14.0 closes the cleaner provenance story." The sentence undersells the issue (every non-docs-alignment row mint silently bypasses Principle A) and was written hours before the gap was actually closed in `fd2e247`. Future planners reading this will not learn that the deferral rationale was empirically invalid.
**Evidence:**
- `.planning/RETROSPECTIVE.md:45` — full sentence.
- Commit `fd2e247` (post-RETROSPECTIVE) — closes 80% of the gap in <1 hour.
**Why it matters:** Cosmetic doc-drift, but it's the kind of "narrative wraps over reality" smell that makes future readers trust the RETROSPECTIVE less. Distinct from F2/F3 (which are HIGH severity); flagging this separately because it's a recoverable doc-edit, not a process failure.

### F10 — Phase ran in ~12 minutes wall-clock; SUMMARY claims ~55 minutes [SEVERITY: LOW]
**Claim in plan:** 88-01-SUMMARY.md:57: `duration: ~55 minutes (catalog-first + 4 task-commits + close)`.
**Reality:** Commit timestamps span `e32c20a` (15:14:56) to `ad576ae` (15:26:14) — exactly 11 minutes 18 seconds. The 55-minute figure may reflect plan-author wall-clock or cumulative subagent time; in any case the SUMMARY metric does not match the git log.
**Evidence:**
- `git log --pretty="%ai" e32c20a..ad576ae` — 6 commits across 11:18 minutes.
- 88-01-SUMMARY.md:57.
**Why it matters:** Velocity smell (failure-mode #6). 12-minute milestone-close phase is plausible if the work is truly file-presence checks (it is, per F1). But the gap between "55 minutes" claim and "12 minutes" reality reinforces the picture that P88 was a procedural sweep, not a substantive close. Combined with F2 (deferred-then-shipped-23-min-later), the velocity tells the same story: the milestone-close was the procedural-shape that lets the verifier call GREEN, not the work that would have surfaced the 16 HIGH frictions.

### F11 — RETROSPECTIVE distillation source-list says "per-phase verdicts" but no per-phase verdict lists a real-backend friction [SEVERITY: SUSPECT]
**Claim in plan:** 88-01-PLAN.md:48: "Source material: SURPRISES-INTAKE + GOOD-TO-HAVES + per-phase verdicts P78-P87 + autonomous-run findings."
**Reality:** Spot-checking per-phase verdicts (e.g. `quality/reports/verdicts/p86/VERDICT.md` — the dark-factory third-arm phase) would tell whether the verdict reports surfaced any of the 16 HIGH frictions. If P86's verdict noted "TokenWorld arm SUBSTRATE-GAP-DEFERRED" but graded GREEN anyway, then the RETROSPECTIVE distillation had upstream signal it could have escalated. If P86's verdict was as silent on real-backend flows as the milestone-close verdict, then the distillation had no upstream signal.
**Evidence I would gather to settle:**
- Read `quality/reports/verdicts/p86/VERDICT.md` line-by-line for any acknowledgment of TokenWorld arm gap.
- Read `quality/reports/verdicts/p79/VERDICT.md` for any acknowledgment that DVCS-ATTACH-01..04 ship sim-only.
- Cross-check whether SURPRISES-INTAKE.md ever flagged any cluster A/B/C/D issue.
**Why it matters:** Determines whether the OP-9 ritual breakdown happened at the distillation step (P88 author chose not to escalate visible signals) or at the upstream verdict step (per-phase verdicts didn't surface real-backend gaps in the first place). Both are framework-integrity issues, but the fix is different.

### F12 — CHANGELOG entry is substantive (~30 lines) but never reaches the headline-promise honesty bar [SEVERITY: SUSPECT]
**Claim in plan:** CHANGELOG entry per ROADMAP P88 SC2: "summarizes P78-P88 + lists every shipped REQ-ID by category + names v0.14.0 carry-forward."
**Reality:** CHANGELOG.md:7-53 does enumerate REQs by category and name carry-forwards. The substantive content bar is met. But the headline promise — "Devs `git clone git@github.com:org/repo.git` with **vanilla git, no reposix install**, get all markdown, edit, commit. Install reposix only when they want to write back; `reposix attach` reconciles their existing checkout against the SoT, then `git push` via a bus remote fans out atomically" (CHANGELOG.md:11) — is a load-bearing public claim that, per Cluster A from the dark-factory exercise, does not hold for any non-sim backend (`reposix attach confluence::REPOSIX` exits with the P79-02 scaffold error). The CHANGELOG entry was technically substantive but factually wrong on its own headline.
**Evidence I would gather to settle:**
- Run `cargo run -p reposix-cli -- attach confluence::REPOSIX --remote-name reposix` to confirm Cluster A's error string lands as documented.
- Compare against the `reposix attach` description in CHANGELOG.md:17.
**Why it matters:** The user's brief flagged this as the milestone-close phase "produced the tag script, CHANGELOG entry, and RETROSPECTIVE distillation" — and asked whether the catalog rows graded content honesty. F12 is the strongest case that they did not: the CHANGELOG verifier checked line-count, not statement-truth, so the headline promise made it past the gate. Marked SUSPECT because the cargo invocation is what would let me confirm the claim's exact failure shape; failure-mode #5 from the audit brief is the right framing if confirmed.

## Summary for v0.13.1 framework-fix phase

P88's failure shape is the cleanest demonstration of the framework's structural blindspot: a milestone-close phase whose 4 catalog rows are 100% file-presence/structural (F1), whose deferral rationale was reversed within 23 minutes (F2), whose RETROSPECTIVE distillation captured zero of the 16 HIGH real-backend frictions latent in the shipped milestone (F3), and whose milestone-close verdict silently dropped the SC5 "TokenWorld arm GREEN" clause (F4). The verifier subagent graded GREEN against rows that *cannot fail* on artifacts that *exist* — they cannot grade content honesty by design.

The implication for v0.13.1: the milestone-close ritual needs at least one row that *executes the docs against a real backend*. F4 shows the SC was already on paper — the failure was that no probe was wired to assert it. A "milestone-close real-backend transcript GREEN" verifier (probe-shape: `cargo run reposix attach <real-backend>::<project>` exits 0; bus push round-trip succeeds; OP-3 audit row written) would have caught Cluster A, Cluster C, and Cluster D before the tag.
