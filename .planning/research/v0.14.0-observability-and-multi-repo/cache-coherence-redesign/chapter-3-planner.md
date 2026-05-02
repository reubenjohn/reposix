← [back to index](./index.md)

# Open questions, risks, invariants, and start instructions

## Open questions for the planner (decisions.md style)

- `Q-CC.1` Does GitHub's Issues REST API actually accept `If-Match` on PATCH `/repos/{owner}/{repo}/issues/{issue_number}`? Docs imply yes; Phase A validates. **Decision-blocking** — if no, Phase C replans (likely fall back to JIRA-style read-then-write).
- `Q-CC.2` What's JIRA's actual support for `If-Unmodified-Since` on issue PATCH? Atlassian docs say yes for some endpoints, ambiguous for issues. Phase A validates. **Decision-blocking.**
- `Q-CC.3` Confluence `version.number = current + 1` — does the server accept the body unconditionally and reject only on 409, or does it require a separate `If-Match` header? Sim contract test asserts the body shape; real Confluence may have more.
- `Q-CC.4` Does the cache's audit row for `update_record` carry the `expected_version` value? If so, recovery audit is rich; if not, P83's `helper_push_partial_fail_*` family doesn't extend cleanly to the new 412 rejection shape. Likely needs a new audit op `helper_push_412_stale_version`.
- `Q-CC.5` Bus push (v0.13.0 P83) interaction: bus push fans SoT-first, mirror-best-effort. If SoT 412s on `If-Match`, what happens? Probably: helper acks `error refs/heads/main fetch first` exactly as today's bus precheck B does. Mirror push never fires (bus rule). But the `helper_push_partial_fail_mirror_lag` op shouldn't fire either — this is a pre-mirror rejection. New audit op needed: `helper_push_412_pre_mirror`.
- `Q-CC.6` Read-cache freshness for non-write workflows: how stale is acceptable? If an agent runs `grep -r TODO issues/` on a year-old cache, that's not a safety problem but it's a UX problem. Document a "freshness budget" in `docs/concepts/dvcs-topology.md`?
- `Q-CC.7` Does `reposix sync --reconcile` need to learn the new audit op vocabulary? If a reconcile finds a record where `cache.version != backend.version`, that's a "cache-update-failed-after-backend-200" signal — distinct from "user edited backend out-of-band." Telemetry should distinguish (mirrors Q-CD.2 from the L2/L3 doc).
- `Q-CC.8` Migration: existing checkouts have caches built before this change. Do they need a one-shot `--reconcile` to populate version metadata, or is the cache schema already version-complete? Likely already complete (`Record.version` is in the cache schema today) but Phase A confirms.

---

## Risks — known unknowns and unknown unknowns

### Known unknowns (Phase A drains)

- **Real-backend etag/version semantics may differ from docs.** Most likely culprit: GH Issues. Mitigation: Phase A is decision-blocking before Phase B-F scope.
- **JIRA timestamp resolution may be coarse enough to make `If-Unmodified-Since` useless.** If `updated` is second-resolution and concurrent edits within 1s are common, race window is real. Mitigation: Phase A measures incidence on TEST project; Phase D picks strategy from data.
- **Confluence's 409 semantics may have edge cases under attachment-bearing pages.** P82's audit work uncovered some non-trivial Confluence rate-limit behavior; concurrent-edit response may have similar surprises. Mitigation: Phase A includes attachment-bearing concurrent-edit test.
- **Cache-write-failure recovery via 412 assumes the next push touches the same record.** What if the user's next 1000 pushes all touch *other* records? The stale record sits silently desyncd. Mitigation: webhook + opportunistic verification on `list_changed_since`. If still concerning, add a "verify N records per push" piggyback (cheap; bounded). Likely a Phase F sub-task.

### Unknown unknowns (the +2 reservation absorbs)

- **Bus-remote interaction.** P83 ratified SoT-first + mirror-best-effort. Adding 412 rejection at SoT introduces a new failure mode mid-push. Detailed audit-trail design owned by Q-CC.5.
- **Multi-actor races.** v0.13.0 P82-83 didn't surface these because the bus is push-only and pushes serialize on the helper. Cache-write-side races (helper push + webhook listener both updating the cache simultaneously) may surface in Phase B/C/D under fault injection.
- **Connector-API churn.** If we discover GH Issues needs a slightly different `update_record` signature (e.g., the etag is opaque and shouldn't be `Option<u64>`), the trait surface changes. Backwards-compat for in-flight v0.14.0 phases needs management. Mitigate by Phase A producing a "trait surface confirmation" doc before Phase B touches code.
- **Rate-limit interaction.** The proposal *reduces* REST call count on the success path (no precheck) but `If-Match` headers don't change rate-limit cost. On the failure path (412 + refetch + retry), the worst case is 2x the success path. For backends with tight rate limits (Confluence Cloud: 5000 req/hr), persistent contention on hot records could make rate limits hurt sooner. Phase A measures.
- **Webhook-driven cache update (Layer 3) introduces a writer to the cache from outside the helper.** Today the helper is the sole writer; a CI workflow updating the cache concurrently is a new shape. Audit-log invariants (append-only) hold trivially, but cache-tree atomicity may not. Likely needs Phase F to add a cache-side write lock or branch the writes.

The **+2 reservation slots are non-optional**, mirroring v0.13.0's pattern. The "absorption-not-skipping" honesty check (OP-8 verifier) applies to every Phase A-F close.

---

## Tie-back to project invariants

- **OP-1 (simulator-first):** Phase B is sim-only by design. Phases C/D/E require real backends but are gated on `--ignored` test runs.
- **OP-2 (tainted by default):** `If-Match` headers carry version numbers (server-controlled, untainted). Cache-side version data is server-controlled at write time. Read-side version read is untainted.
- **OP-3 (audit log dual-table):** every 412 rejection writes a row to `audit_events_cache`. Every successful 412-driven retry writes both (`audit_events_cache` for the helper turn, `audit_events` for the backend write). New audit ops: `helper_push_412_stale_version`, `helper_push_412_retry_success`.
- **OP-6 (real backends are first-class):** Phases C/D/E land real-backend tests; "simulator-only coverage" doesn't satisfy.
- **OP-7 (verifier subagent on close):** every Phase A-F dispatches a verifier per `quality/PROTOCOL.md`. Phase A's verifier grades the validation findings against the per-backend predictions in this doc — if findings contradict the doc, that's a RED that triggers replanning.
- **OP-8 (+2 reservation):** Phases G/H reserved. Eager-resolution preference applies for trivial in-phase findings.
- **OP-9 (milestone-close distillation):** v0.14.0's RETROSPECTIVE.md section captures the cache-coherence redesign learnings cross-milestone.

---

## Where to start when you pick this up

1. Read this doc.
2. Read `vision-and-mental-model.md § L2/L3 cache-desync hardening` for the option-A framing this proposal counters.
3. Skim `crates/reposix-core/src/backend.rs:53-167` (capability + versioning enums) — they're the trait substrate this proposal completes.
4. Skim `crates/reposix-core/src/backend/sim.rs:333-345` for the canonical `If-Match` shape.
5. Run `/gsd-discuss-phase` with this doc + the vision doc as inputs. Ratify Option A vs Option B vs hybrid in `decisions.md`.
6. If Option B (this doc) is ratified: `/gsd-plan-phase` for Phase A first; do NOT scope Phases B-F before Phase A's findings land.
