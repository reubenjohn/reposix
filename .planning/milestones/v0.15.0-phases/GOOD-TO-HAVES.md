# v0.15.0 Good-to-haves / carried-forward hardening

> **Purpose:** concrete landing spots for the DEFERRED / DEFERRED-TO-v0.15.0 entries the
> v0.14.0 surprises-intake promised would land here. Owner ask (2026-07-12): *labels alone
> don't count* — each carried-forward entry needs a real row with **severity + a concrete
> fix-sketch**, verbatim-faithful to the intake. Source of truth for the originals (archived,
> not deleted): `.planning/milestones/v0.14.0-phases/surprises-intake/part-01.md` + `part-02.md`.
> Landed by gsd-quick `260712-oke`. OP-8 drains this file in v0.15.0's last two phases.
>
> **Roadmap-gap reconciliation:** the intake cited "v0.15.0 framework-/helper-hardening phases"
> that the v0.15.0 `ROADMAP.md` did not list (it had only the two UX `Phase TBD` stubs). The
> two **HIGH** entries below (GTH-V15-04 modern-git verification, GTH-V15-06 subprocess-bypass
> self-safety refusal) now have `### Phase (candidate)` stubs under `ROADMAP.md` §
> "Hardening candidates"; the five MEDIUM entries live here as drain rows.

## Split index (OP-8 file-size drain)

This ledger exceeded the *.md 20k budget and was split into 9 per-part child files under `good-to-haves/`. Every entry is preserved verbatim; append new entries to the last part (or a new part) and add the title here.

- [`good-to-haves/part-01.md`](good-to-haves/part-01.md) — 2 entries:
  - Carried-forward from the v0.14.0 surprises-intake (7 entries)
  - Hygiene (file-size early-warning)
- [`good-to-haves/part-02.md`](good-to-haves/part-02.md) — 8 entries:
  - From the b773c04 RED-main arc (2026-07-13, SESSION-HANDOVER successor #16 noticings)
  - From the L0 relief handover #19→#20 queue (2026-07-14, doc-alignment refresh session)
  - From L0 rotation #22 (t4 real-backend re-run, 2026-07-14)
  - From L0 rotation #26 (carry-forward intake filing, 2026-07-15)
  - From L0 rotation #27 manager queue (2026-07-15)
  - From L0 rotation #30 push-unblock docs-alignment refresh (2026-07-15)
  - From gsd-quick lane 260715-mk5 public roadmap diagram (2026-07-15)
  - Back-pointer note (bidirectional trail — INTENTIONALLY SKIPPED)
- [`good-to-haves/part-03.md`](good-to-haves/part-03.md) — 5 entries:
  - From owner↔ex-manager-#9 review session (2026-07-15)
  - From L0 relief handover #39→#40 (carry-forward noticing filed, 2026-07-16)
  - From P115 T6 Wave 2 item 5 (regen-clobber guard, 2026-07-16)
  - From P115 T6 Wave 2 item 6b (cold-init reconcile + loop/perf un-waive, 2026-07-16)
  - From the P115 owner-directive lane (retirement-narrative strip, 2026-07-16)
- [`good-to-haves/part-04.md`](good-to-haves/part-04.md) — 3 entries:
  - From the owner-directive lane (post-P115-close scheduling, 2026-07-16)
  - From P117-01 SC4 Option-B ratification (2026-07-16)
  - From P117-01 W1 intake triage (2026-07-16, L0 #55)
- [`good-to-haves/part-05.md`](good-to-haves/part-05.md) — 4 entries:
  - From P117-02 W2 intake triage (2026-07-16)
  - From P117-03 W2 intake triage (2026-07-16)
  - From P117-02 pre-push regression-fix lane (2026-07-16, GUARD escalation)
  - From P117 W2 Step C-i (2026-07-16, GTH-V15-49 implementation fix-twice)
- [`good-to-haves/part-06.md`](good-to-haves/part-06.md) — 3 entries:
  - From P117 W3 push-blocker fix lane (2026-07-17, `structure/banned-words` gate)
  - From P117 W4 push-blocker fix lane (2026-07-17, `docs-alignment` cascade)
  - From P117 W5 FINAL cleanup + fallback-UX fix lane (2026-07-17)
- [`good-to-haves/part-07.md`](good-to-haves/part-07.md) — 4 entries:
  - From P118 close intake-filing (2026-07-17, pre-existing drift surfaced)
  - From P119 docs/planning-simplification (2026-07-17, the "P112 RAISE" cleanup)
  - From P120 CLOSE Wave A (2026-07-17, credential-leak hardening)
  - From P120 CLOSE Wave B (2026-07-17, close-wave polish + intake triage)
- [`good-to-haves/part-08.md`](good-to-haves/part-08.md) — 3 entries:
  - From P120 CLOSE push-preview (2026-07-17, first full pre-push run on the close commits)
  - From P121 W4 (2026-07-18, docs + bookkeeping close-out — triaged W3.6 noticings)
  - From P121 review (post-review fix wave, 2026-07-18)
- [`good-to-haves/part-09.md`](good-to-haves/part-09.md) — 7 entries:
  - From P121 CLOSE (bookkeeping + push, 2026-07-17)
  - From P122 W4 (stateless-connect read-path verification, 2026-07-18)
  - From P122 close (deterministic close, 2026-07-18)
  - From Phase 123 close (P114 OQ1 residual carry-forward)
  - From Phase 123 close (REQUIREMENTS.md coverage-table staleness noticing)
  - From Phase 123 close (Lane 1 code-review noticing, file-size early-warning)
  - From Phase 124 close (container-rehearse harness hardening, 2026-07-18)
