# p93 real-Confluence recovery failure — DP-2 diagnosis (read-only lane)

Row: `agent-ux/p93-partial-failure-recovery-real-confluence`. Verifier exits 1
deterministically (both credentialed runs) at `agent_flow_real.rs:690` —
push 2 (recovery) returns `error refs/heads/main some-actions-failed`.

## Root cause (proven, evidence-quoted)

CREATE-recovery cannot converge against an id-assigning backend:

1. `reposix-confluence/src/lib.rs:296-344` `create_record` POSTs `/wiki/api/v2/pages`
   with **no `id` field** — Confluence assigns its own page id; the returned
   `ConfPage.id` becomes the SoT record id. The harness's client-chosen `id_a`
   (`agent_flow_real.rs:629`, a timestamp) is a **local tree id only**, never sent.
2. `diff::plan` (`reposix-remote/src/diff.rs:242`) decides create-vs-update by
   `prior_by_id.get(&id)` — matched by **record id**. `None → Create`.
3. Push 1 lands page A on Confluence under Confluence's assigned id. The partial-fail
   branch (`write_loop.rs:285-299`) **deliberately skips oid_map/cursor writes**
   ("convergence is the next push's PRECHECK-B re-read of the current SoT").
4. Push 2 re-sends `pages/{id_a}.md`. PRECHECK-B materializes `prior` from the SoT,
   where page A is keyed by Confluence's id, **not** `id_a`. `prior_by_id.get(&id_a)
   → None → plan re-CREATEs page A → Confluence unique-title-within-space reject →
   `some-actions-failed`.

## Why the sim twin stays green

`reposix-remote/tests/partial_failure_recovery.rs:182+` models **UPDATEs to
pre-existing** records (issues 1,2 already in SoT at v1). Recovery diffs away
already-landed issue 1 (`PATCH /issues/1` `.expect(1)`) because its id (1) is
**stable across pushes and matches the SoT**. Id-stability is what makes convergence
work. CREATE-recovery against an id-assigning backend has no such stability.

## Classification: MIXED (dominant bounded fix = HARNESS)

- **Harness (bounded, <1h):** the real smoke fabricates a CREATE-recovery scenario
  the product's id-stable UPDATE-replan convergence never claimed to support. A real
  agent would `git pull` after a partial fail (bringing down A's reconciled backend
  id) before retrying — it would never re-push an identical CREATE. Fix: make the
  recovery UPDATE-based (mirror the sim twin), OR `reposix sync` between push 1 and
  the retry so the tree carries A's backend id and plan diffs A away. **Add teardown**
  (delete created pages) — currently leaves orphans. Files: `agent_flow_real.rs:605-698`.
- **Product (real but out-of-bound gap):** partial-fail recovery of a mix that
  includes a landed CREATE is genuinely non-convergent against id-assigning backends.
  Closing it (write-back reconciliation of client-create-id → backend-id on the
  partial-fail path, or title/content create-dedup) touches oid_map + create-identity
  semantics — **exceeds <1h/no-new-dep; FILE it**, do not eager-fix.

## Was p93 ever green on real Confluence?

**No.** Catalog row `status: NOT-VERIFIED`, `last_verified: null` (catalog-first
scaffold, agent-ux.json:1986-2013). Masked first by autonomous no-creds
(NOT-VERIFIED), then by the space-guard self-reject; the first-ever credentialed run
(post-B2-pin) deterministically FAILED. Pre-existing never-passing row.

## Orphan cleanup (recommend, do NOT execute here)

Two `p93 smoke A (kind=test <ts>)` pages in TokenWorld. Labels aren't wired
(`lib.rs` deferred), so sweep by TITLE: resolve ids via
`GET /wiki/api/v2/pages?title=...` then `DELETE /wiki/api/v2/pages/{id}`. Not trivial
(credentialed mutation + id resolution) → left for the fix lane's teardown.
