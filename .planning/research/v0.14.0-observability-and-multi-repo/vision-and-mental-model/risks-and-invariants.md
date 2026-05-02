[← index](./index.md)

## Risks and how we'll know early

- **OTel adds non-trivial overhead on the helper hot path.** Sampling-based mitigation is standard but `tracing-opentelemetry` is not free. **Early signal:** measure helper RPC turn-around with OTel enabled at 100% sample vs disabled, in the dark-factory regression. If overhead is > 10% at full sampling, document the recommended sample rate and ship with a sane default.
- **`origin_backend` enforcement breaks existing checkouts that predate the field.** Backfill is the answer (per Q-OOT.1) but the rollout matters. **Early signal:** prototype the migration in a sandbox space with deliberately-unstamped records; if backfill needs > 1 manual step from the user, redesign before shipping.
- **Cache desync incidence is too low to measure in a one-week window.** If Phase 5x's data is too sparse to drive the L2-vs-L3 decision, we're stuck. **Mitigation:** plan a fall-back of "ship L2 by default" if telemetry is inconclusive; L2 is cheap enough to ship without the data and L3 can layer on later if hot.
- **Multi-project helper (Phase 56) leaks state across projects.** Cross-project isolation is the load-bearing claim. **Mitigation:** the CI test in the success gate is non-negotiable; treat any cross-project leak as a security regression, not a feature bug.

## Tie-back to project-level invariants

- **OP-1 (simulator-first):** all v0.14.0 phases run end-to-end against the simulator. Two simulators in one process serve as "two ISSUES backends" for the origin-of-truth tests. Real-backend tests gate the milestone close, not individual phase closes.
- **OP-2 (tainted by default):** OTel span attributes and audit-event records carry tainted bytes from the backend. The dashboard page (Phase 57) MUST treat audit-event payload as tainted — escape on render, never inject into HTML directly. The `reposix tail --json` stream emits tainted bytes; the consumer is responsible for downstream sanitization.
- **OP-3 (audit log):** every cache repair (L2) or transactional rollback (L3) writes to `audit_events_cache`. Every `reposix migrate-origin` writes to both tables (cache audit for the helper turn, backend audit for the rewrite REST call).
- **OP-7 (verifier subagent grades GREEN):** every v0.14.0 phase close dispatches the verifier per `quality/PROTOCOL.md`. The L2-vs-L3 decision (Phase 5x telemetry → Phase 5x+1 implementation) is itself a verifier-graded artifact: the verdict file cites the desync-incidence data and explains the choice.
- **OP-8 (+2 phase practice):** v0.14.0 reserves its last two phases for surprises absorption + good-to-haves polish. Multi-scope milestones surface MORE surprises than single-thesis milestones (three intersecting designs); do not omit the +2 reservation.
