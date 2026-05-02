← [back to index](./index.md)

## Action classification debate (W4 above)

The owner asked: *"does adding `next_action` add too much burden on the agent classifying or does it encourage more reasoning and accuracy?"*

**My recommendation: include it.** Reasoning:

- **Default-WRITE_TEST means inattentive extractors produce correct output by accident.** A careful extractor produces structured signal. Net positive.
- **Consumer-side win is large.** Punch list + cluster-phase scoping become filterable. v0.12.1 can scope phases as "close all FIX_IMPL_THEN_BIND" rather than reading 166 rationale fields by hand.
- **Existing failure mode the field would fix:** The `IMPL_GAP:` rationale-prefix convention this session is informal and grep-only. Structured field formalizes it.
- **Cost is one prompt instruction + one catalog field.** Trivial schema change.

The field is on the *Row*, not the verdict. Verdict (`last_verdict`) stays orthogonal — same row can be MISSING_TEST + WRITE_TEST (test missing, just write it) OR MISSING_TEST + FIX_IMPL_THEN_BIND (test missing because impl regressed; fix both). Both axes are needed.

---

## Time budget

| Block | Estimate | Cumulative |
|---|---|---|
| W1 (glossary bulk-confirm) | 5 min | 5 min |
| W2 (apply audit) | 15 min | 20 min |
| W3 P67 (audit + extractor prompt) | 1h | 1h20min |
| W4 P68 (next_action field) | 1.5h | 2h50min |
| W5 P69 (--i-am-human flag) | 30 min | 3h20min |
| W6 P70 (hook self-test) | 30 min | 3h50min |
| W7 P71 (schema cross-cut) | 1h | 4h50min |
| W8 P72-P80 (cluster phases) | 6-8h | 10h50min - 12h50min |
| Milestone-close verifier | 30 min | + |

5pm deadline = ~9h from session start. Realistic: W1-W7 + 2-3 cluster phases ship by 5pm. Remaining clusters become v0.12.1.x or are rolled into a follow-on session.

**Recommended phase order for next session:** W1, W2 (consume the audit), W3, W6 (small + unblocks future hook regressions). Then W4 (structured field) BEFORE the cluster phases, because each cluster phase benefits from the structured `next_action`.

---

## v0.12.0 G1 reminder

**The orchestrator does NOT push the v0.12.0 tag.** Status remains
`ready-to-tag-pending-owner-decision` until owner picks Path A or B
(see `quality/reports/verdicts/milestone-v0.12.0/VERDICT.md` § Gap-block
G1). v0.12.1 work proceeds in parallel; the tag-cut waits.

---

## Cleanup criterion

This file deletes itself when:
- W1 and W2 closed (catalog at expected ratios for v0.12.1).
- P67-P71 (W3-W7) shipped GREEN at per-phase verifier.
- A v0.12.1-specific HANDOVER (or none — STATE.md alone) replaces it.

The phase that ships W7 (or whichever closes the last item above) includes `git rm .planning/HANDOVER-v0.12.1.md` in its closing commit.
