# Cache coherence redesign — optimistic concurrency on writes (v0.14.0 proposal)

**Status.** Proposal. Not yet ratified by the v0.14.0 planner. Counter-proposal to the L2/L3 framing in `vision-and-mental-model.md § L2/L3 cache-desync hardening` (which remains in place as one option for the planner to choose between).

**Authored.** 2026-05-01, after v0.13.0 milestone-close. Owner conversation `2026-05-01` raised the question: *"Is it truly a trade-off? Can we not have performance and safety?"* This doc is the design that argues we mostly can.

**Reading order for the v0.14.0 planner.**
1. `vision-and-mental-model.md` — the milestone scope (OTel + origin-of-truth + cache hardening + +2 reservation). Treat the L2/L3 section there as Option A.
2. This doc — Option B. Read both, ratify one (or hybridize) in `decisions.md` at planning time.
3. `.planning/research/v0.13.0-dvcs/architecture-sketch.md § "Performance subtlety"` — origin of the L1 ↔ L2/L3 ladder.

---

## TL;DR

The L2/L3 ladder treats conflict detection as something we have to do *before* writing, in a precheck. But every well-designed REST API already provides conflict detection on the *write itself*, via etags / `If-Match` / version preconditions. Lean on that, and the precheck-vs-cache-staleness trade-off mostly evaporates.

- **Performance:** drop the `list_changed_since` precheck for backends with strong versioning. One round-trip saved per push.
- **Safety:** every PATCH carries `If-Match: <cached_version>` (or backend equivalent). Backend rejects 412 / version-mismatch on stale cache. Helper translates → `error refs/heads/main fetch first`. Identical recovery UX to today's L1.
- **Self-healing:** if cache write fails after a successful backend write, next push's `If-Match` uses the stale version, gets 412, fetches fresh, retries. No background reconcile job needed.
- **Read cache:** stale cache reads are tolerated. Most agentic workflows don't notice, and `reposix sync --reconcile` stays as the user-explicit "I want a fresh read cache" escape hatch.

The proposal is not novel — it's standard optimistic concurrency control. The novelty is recognizing that v0.13.0's L1/L2/L3 framing absorbed a precheck-shaped assumption from the original `list_records` walk and never relaxed it.

---

## Chapters

- **[Design — plumbing, layers, cost, and residuals](./chapter-1-design.md)** — Trait surface status; four-layer redesign; cost table vs L1/L2/L3; three residuals.
- **[Phase shape and v0.14.0 sequencing](./chapter-2-phases.md)** — Phases A-H; milestone fit; planner cut options.
- **[Open questions, risks, invariants, and start instructions](./chapter-3-planner.md)** — Q-CC.1–Q-CC.8; risk catalog; OP tie-backs; where to start.
