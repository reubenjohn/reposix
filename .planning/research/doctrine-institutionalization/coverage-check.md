# Coverage Check — directive→theme→encoding-status table (chapter 3 of 3)

Sharded verbatim from `DIRECTIVES.md` (kept as its own chapter, not folded into
`themes-08-14.md`, because themes-08-14 + this table together exceed the
20,000-byte cap — the 3-way split the landing checklist authorizes). Back to `index.md`.

---

## Coverage Check

Every known-directive (a)–(l), every numbered directive #1–20, and both late
2026-07-04 messages map to at least one theme above. Nothing was dropped.

| Source item | Theme(s) | Encoding status |
|---|---|---|
| (a) three-tier delegation (fable→opus/sonnet/haiku, never fable at leaf) | 1 | COMMITTED (89-OWNER-DECISIONS.md OD-4); NOT in CLAUDE.md |
| (b) coordinators are routers not workers (≤100 tool calls, ≤400-word reports, never idle-wait) | 2 | HIDDEN STATE only |
| (c) proactive relief at wave boundaries past ~50% context; executor writes+commits handover | 3, 10 | HIDDEN STATE (rule) + COMMITTED (exemplar artifacts) |
| (d) understand underlying intention over faithful execution; OD-4 resequencing | 4 | COMMITTED (one resulting decision); general principle NOT ENCODED |
| (e) tangent scoping (<1h inline / bigger→intake / balloon→SCOPE DECISION NEEDED) | 5 | PARTIALLY (OP-8 <1h rule COMMITTED; "SCOPE DECISION NEEDED" phrase not found verbatim anywhere — flagged as a synthesis, not a quote) |
| (f) quality upkeep as standing parallel operation (audit fleets, debt-drain windows, no dispatch over undrained BLOCKERs, fable-tier unification) | 6 | PARTIALLY (D-CONV decisions + OD-4 item 2 COMMITTED; "Operating Cadence A/B" framing HIDDEN STATE only) |
| (g) one cargo machine-wide, jobs=2, RAM budget | 8 | COMMITTED, fully, CLAUDE.md § Build memory budget |
| (h) durable state over chat: /tmp ledger died with OS crash — repo from row one | 9 | NOT ENCODED as general principle (adjacent but narrower OP-4 exists) |
| (i) orchestrator relay: sub-agent replies mis-route; check tree before re-running | 11 | NOT ENCODED — episodic only, no owner quote behind it |
| (j) external mutations need owner-named-target approval; classifier denials are design feedback | 12 | NOT ENCODED as general rule; one worked episode (directive #9) COMMITTED nowhere as doctrine |
| (k) session pause/resume briefs at logical boundaries | 10 | PARTIALLY (exemplar artifacts COMMITTED; no general template written down) |
| (l) fix-it-twice meta-rule; noticing as deliverable; polish-for-adoption north star | 7, 13 | COMMITTED — both halves already in CLAUDE.md |
| #1 opening ask: tooling/guardrails/hooks as deliverable; dynamic workflows from message one | 5, 14 | PARTIALLY (OP-8 eager-fix COMMITTED; broader framing NOT ENCODED) |
| #2 context 18%→50% target; delegate to fable-that-delegates | 1, 3 | HIDDEN STATE (target); COMMITTED (OD-4 tiering) |
| #3 full ownership; ground-level noticing is required signal | 7 | COMMITTED (Ownership charter) |
| #4 pause is first-class; produce real handoff | 10 | COMMITTED (exemplar artifacts) |
| #5 explicit three-tier model doctrine | 1 | COMMITTED (OD-4); NOT in CLAUDE.md |
| #6 "CI badge is red" — verify-against-reality trigger | 7 | COMMITTED (ownership charter pt. 4 covers the principle; this specific episode is illustrative only) |
| #7 leverage subagents more; don't race features ahead of debt | 2, 6 | PARTIALLY |
| #8 the capstone quality-upkeep + executive-pivot + strict-tiering message | 1, 4, 6 | PARTIALLY (see each theme) |
| #9 Google API key false positive — owner-only security unblock | 12 | NOT ENCODED as general rule |
| #10 OS crash x3; resume orphaned agents via SendMessage; ledger loss | 9, 10 | NOT ENCODED (durable-state principle); COMMITTED (exemplar recovery + reconstructed ledger) |
| #11 pause at earliest logical point (2nd instance) | 10 | COMMITTED (exemplar artifacts) |
| #12 quality infra itself was an unplanned tangent; account for tangent-within-tangent | 5 | NOT ENCODED as general rule |
| #13 diagnose context-rot from outside; prescribe coordinator rotation | 3 | HIDDEN STATE (rule); COMMITTED (91-HANDOVER.md exemplar) |
| #14 relieved coordinator must be told in advance to persist handover | 3, 10 | HIDDEN STATE (rule); COMMITTED (exemplar) |
| #15 "coordinators not workers" — sharpest finding | 2 | HIDDEN STATE only |
| #16 capstone institutionalization directive | 14 | NOT ENCODED — this document is the first response to it |
| #17 context awareness from turn one | 3 | HIDDEN STATE only |
| #18 real OOM crash confirmed | 8 | COMMITTED (CLAUDE.md Build memory budget, as originating incident) |
| #19 "be careful about parallel agents issuing large rust commands" → codified rule | 8 | COMMITTED, fully |
| #20 owner watching UI for stalled dialogs — reality-check from owner's side | 7 | COMMITTED (principle, via ownership charter pt. 4); episode itself not separately documented |
| Late msg: "CLAUDE.md... alone is insufficient for highly autonomous usecases" | 14 | NOT ENCODED — today's directive, addressed by this extraction task itself |
| Late msg: "worth persisting a report in the codebase... if 600 word limit too lossy" | 14 | NOT ENCODED — today's directive; this file + its companion report are the direct response |
| Mid-task correction: "There are hooks in ~/.claude/" | 14 | Confirmed TRUE by ground-truth read of `~/.claude/settings.json` (11 GSD hooks wired); not reposix-specific yet |
| Mid-task correction: "Only focus on the largest one not the others" | 5, 14 | Episode of the discipline in Theme 5 being exercised live by the owner |

**Items with NO transcript quote behind them (accumulator-only,
owner-directive dates cited therein, not independently verifiable against
raw transcript text by this miner):** the 5-rule "COORDINATOR CONTEXT
DISCIPLINE" block's exact wording (Theme 2/3) is the accumulator's own
paraphrase/operationalization of directives #5, #8, #13, #14, #15 — dated
"owner directive 2026-07-04" inline but not a single verbatim owner
sentence; the "OPERATING CADENCE A/B" framing (Theme 6) is similarly the
accumulator's own synthesis, dated "owner directive 2026-07-04 'leave no
stone unturned'" (which correctly cites directive #7's verbatim phrase as
its anchor, but the A/B structure itself is orchestrator-authored, not
owner-authored). Both are faithful operationalizations of real owner intent
but should not be mistaken for things the owner typed verbatim.
