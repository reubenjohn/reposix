# Phase decomposition, exclusions, and tie-back

← [back to index](./index.md)

## Phase decomposition (sketch — final shape decided by `/gsd-plan-phase`)

A reasonable v0.13.0 phase sequence:

| Phase | Scope | Acceptance |
|---|---|---|
| **N**     | `reposix attach` core (cache build from REST against existing checkout; reconciliation rules; tests against deliberately-mangled checkouts) | `reposix attach` works on a vanilla GH-cloned checkout; reconciliation table populated; conflict cases produce clear errors |
| **N+1**   | Mirror-lag refs (`refs/mirrors/confluence-head`, `confluence-synced-at`); read/write helpers; integration with existing single-backend push to start writing them | Refs visible via plain `git fetch`; `git log` works; existing single-backend push updates them |
| **N+2**   | Bus remote URL parser + dispatch in helper; cheap precheck A (mirror) + B (SoT); bail-with-hint paths | Bus URL parses; both prechecks trip correctly in tests; rejection messages are informative |
| **N+3**   | Bus remote write fan-out (SoT first, mirror second); audit rows; mirror-lag tracking on partial failure | Round-trip test green; fault-injection tests (kill mirror push, kill SoT mid-write, etc.) all produce correct audit + recoverable state |
| **N+4**   | Webhook-driven mirror sync (GH Action workflow + setup guide); end-to-end test with a real confluence webhook against a sandbox space | Workflow runs on dispatch; updates mirror within latency target; `--force-with-lease` race protection verified |
| **N+5**   | Docs: `docs/concepts/dvcs-topology.md`, `docs/guides/dvcs-mirror-setup.md`, troubleshooting matrix entries; cold-reader pass via `doc-clarity-review` | Docs ship; cold-reader pass returns no critical friction |
| **N+6**   | Dark-factory regression extension (third arm: vanilla-clone + attach + bus-push) | New transcript in `scripts/dark-factory-test.sh` passes against sim and TokenWorld |
| **N+7**   | Surprises absorption (+2 reservation slot 1 per OP-8) | `SURPRISES-INTAKE.md` drained; each entry RESOLVED \| DEFERRED \| WONTFIX |
| **N+8**   | Good-to-haves polish (+2 reservation slot 2 per OP-8) | `GOOD-TO-HAVES.md` drained; XS items closed; M items deferred to v0.14.0 |

That's 9 phases. Adjust during planning — phase N+3 (bus write fan-out) is the riskiest and may want to split.

## What we're NOT building (and why)

- **A `reposix sync` command that does the mirror push for you.** Out of scope. The bus remote does it inline; the GH Action does it on webhook. A separate `reposix sync` is a backstop daemon, deferred per the vision doc.
- **A way to add a third bus endpoint.** The algorithm generalizes; the URL scheme generalizes. But the v0.13.0 implementation hardcodes 1+1 because nothing in scope needs more. Generalize when a real use case appears.
- **Bidirectional bus** (mirror writes propagate back to SoT). The mirror is read-only from confluence's perspective. Vanilla `git push origin` from Dev B's checkout to the GH mirror would create commits the SoT never sees — those would be lost on the next webhook sync, which would force-with-lease over them. We document this constraint loudly in `dvcs-topology.md`. To write back to SoT, you must go through a reposix-equipped bus push.
- **Conflict resolution UI / interactive merge against confluence-side edits.** The standard `git pull --rebase` flow handles it. The helper's reject message points at it. No new tooling.

## Tie-back to the existing helper code

The bus remote is structurally an extension of `crates/reposix-remote/src/main.rs::handle_export` (lines 300-407). The dispatch happens earlier — at URL parsing, before `handle_export` is reached, the helper decides whether to instantiate a single-backend handler or a bus handler. The single-backend `handle_export` is preserved verbatim; the bus handler wraps it with the precheck phase and the mirror-write phase.

`crates/reposix-remote/src/stateless_connect.rs` (read path) is untouched — bus is push-only per Q3.4.

`crates/reposix-cache/` grows the reconciliation table (used by `attach`) and the mirror-lag ref helpers.

`crates/reposix-cli/` grows the `attach` subcommand and the URL parser for `bus://`.

No new crates; everything fits within the existing workspace shape.
