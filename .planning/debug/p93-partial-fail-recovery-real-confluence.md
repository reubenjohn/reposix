---
status: diagnosed
trigger: "p93-partial-failure-recovery-real-confluence FAILS on LIVE Confluence (TokenWorld); push 2 recovery returns some-actions-failed instead of ok refs/heads/main. Passes in sim."
created: 2026-07-05T00:00:00Z
updated: 2026-07-05T01:00:00Z
---

## Current Focus

CONFIRMED root cause (real-backend artifact obtained). Verdict: E4 owner escalation —
ADR-010's PRECHECK-B/diff::plan convergence strategy is create-blind on every real
(id-reassigning) backend. NOT applying a fix (would require a create-identity
reconciliation design ADR-010 did not specify + the codebase deliberately avoided).

reasoning_checkpoint:
  hypothesis: "On push 2, diff::plan re-plans Create(A) because the landed page A returns from Confluence with a BACKEND-ASSIGNED id != local placeholder id_a; id-only matching cannot diff it away; the re-Create hits Confluence's unique-title-per-space constraint -> HTTP 400 -> some-actions-failed. The same id-mismatch also makes plan emit Delete(real-A) (stray prior)."
  confirming_evidence:
    - "Live TokenWorld: POST /wiki/api/v2/pages returned backend-assigned id 8388609 (reposix sends no id)."
    - "Live TokenWorld: 2nd POST with same title -> HTTP 400 'A page already exists with the same TITLE in this space'."
    - "Residual orphan 'p93 smoke B' with NO matching 'smoke A' from a prior run == push-2 deleted landed-A while creating B."
    - "TokenWorld total pages == 0 (no durable fixtures) -> push-1 prior empty (clean 2 creates), push-2 prior == [landed A under real id]."
    - "execute_action Create arm discards create_record's returned backend id (`let _new`); no id reconciliation anywhere."
  falsification_test: "If Confluence honored a client-chosen page id on create (round-trip), or if diff::plan matched on title/content, push 2 would converge. Neither holds."
  fix_rationale: "N/A — no honest in-family fix; see E4 finding in Resolution."
  blind_spots: "Did not run the full reposix probe (would leak pages, adds nothing beyond the isolated REST artifact). GitHub/JIRA not directly probed but all server-assign ids identically."

## Symptoms

expected: push 2 (recovery) converges -> `ok refs/heads/main`. PRECHECK-B re-reads SoT, diff::plan diffs already-landed page A away, replans only B, B now succeeds.
actual: push 2 returns `error refs/heads/main some-actions-failed` (exit 101, panic at agent_flow_real.rs:690).
errors: "push 2 (recovery) must succeed and converge; stdout=error refs/heads/main some-actions-failed"
reproduction: cadence probe p93-partial-failure-recovery-real-confluence; `cargo test -p reposix-cli --test agent_flow_real partial_failure_recovery_real_confluence -- --ignored` with ATLASSIAN creds in env.
started: first credentialed run of this probe (never passed on real Confluence; sim-only green hid it).

## Eliminated

## Evidence

- checked: sim recovery test crates/reposix-remote/tests/partial_failure_recovery.rs
  found: it models UPDATEs to issues 1 & 2 which ALREADY EXIST on the modeled SoT with STABLE ids. It never exercises a CREATE. So the sim-green scenario is NOT the same shape as the real test (which does CREATEs).
  implication: sim-green did not cover create-partial-fail recovery at all; the divergence is CREATE vs UPDATE, not sim vs real transport.

- checked: sim create route crates/reposix-sim/src/routes/issues.rs:253 (create_record)
  found: sim ALSO reassigns ids on create (`new_id = MAX(id)+1`; CreateIssueBody has no `id` field). No real or sim backend honors a client-chosen id on create.
  implication: NO backend round-trips a client-chosen record id on create. The placeholder id_a can never match the landed record id.

- checked: crates/reposix-remote/src/main.rs:484 execute_action Create arm
  found: `let _new = rt.block_on(backend.create_record(...))?;` — the returned Record (which for Confluence carries the REAL backend-assigned id) is DISCARDED. No cache/oid_map write-back of the backend id.
  implication: the landed create's real id is never recorded anywhere reposix can later match against the local placeholder id.

- checked: crates/reposix-remote/src/diff.rs plan()
  found: matching is purely by RecordId (`prior_by_id: HashMap<RecordId,&Record>`); a pushed record whose id is absent from prior -> Create. No title/content-based create-dedup.
  implication: on push 2 prior (from list_records) contains page A under its real conf id, NOT id_a -> plan emits Create(A) again.

- checked: crates/reposix-remote/src/write_loop.rs apply_writes + precheck.rs
  found: partial-fail path does NOT advance cursor/oid_map (SotOk branch skipped). Push 2 precheck materializes prior via list_records fallback -> prior includes landed page A under real conf id. Confirms plan re-Creates A.
  implication: confirms mechanism end-to-end from static reading.

- checked: docs/decisions/010-l2-l3-cache-coherence.md §3 (RBF-LR-03)
  found: ADR-010 convergence contract states diff::plan "recomputes against that new base ... the already-landed writes are diffed away." This holds for UPDATEs (stable ids) but is FALSE for CREATEs on any id-reassigning backend (all real backends).
  implication: the E4 question — ADR-010's convergence claim is create-blind on real backends.

- checked: LIVE TokenWorld via curl (cargo-free diagnostic + cleanup, both torn down)
  found: create#1 -> HTTP 200, backend-assigned id=8388609; create#2 (same title) -> HTTP 400 "A page already exists with the same TITLE in this space"; residual orphan 'p93 smoke B' (no matching A) from a prior run; space total pages=0 (no fixtures). All p93 smoke pages deleted -> space clean.
  implication: DECISIVE real-backend confirmation of the mechanism. Both the duplicate-title 400 (blocks re-Create A) and the backend-assigned id (breaks id-matching) are proven live.

## Resolution

root_cause: |
  diff::plan (crates/reposix-remote/src/diff.rs) matches records ONLY by frontmatter
  RecordId. On a CREATE, no real backend honors a client-chosen id — Confluence assigns
  its own page id (proven live: 8388609). So the landed page A comes back under a
  backend id that never equals the local placeholder id_a. On push 2, PRECHECK-B's prior
  (from list_records) contains A under its real id, so diff::plan (a) cannot match/diff
  A away -> re-plans Create(A) -> Confluence rejects the duplicate title with HTTP 400
  -> execute_action Err -> failed_ids=[id_a] -> SotPartialFail -> `some-actions-failed`;
  and (b) sees the real-id-A prior record as absent from the tree -> plans Delete(real-A),
  churning the just-landed page. The sim test only ever exercised UPDATEs to pre-existing
  STABLE ids, so it never hit id-reassignment — that is why sim-green hid this.

  ADR-010 §3 (RBF-LR-03) convergence contract ("diff::plan recomputes ... the
  already-landed writes are diffed away") is TRUE for updates but STRUCTURALLY FALSE for
  creates on every id-reassigning backend (Confluence/GitHub/JIRA/sim all server-assign
  ids). Even a happy-path create does not round-trip: after create+push+pull the agent
  holds both the stale pages/id_a.md placeholder and the real pages/{backend_id}.md, and
  the next push re-Creates id_a -> the same 400. The create-identity round-trip is a
  pre-existing hole; this probe is the first test to exercise a real create-then-converge.

fix: |
  NONE APPLIED — E4 owner escalation. No honest, in-family, <1h fix exists that does NOT
  change ADR-010's strategy:
  - execute_action create-idempotency (treat duplicate-400 as success, mirroring delete
    Fork B) => band-aid: silently swallows genuine unrelated title collisions AND does not
    reconcile identity (working tree vs SoT never agree on the id) => masks ADR-010's claim
    rather than satisfying it; still leaves the Delete(real-A) churn.
  - title/content natural-key create-dedup in diff::plan => reintroduces content/path-ish
    matching the codebase DELIBERATELY removed after a live mass-delete incident; a real
    design reversal with its own blast radius; also does not close the identity gap.
  - persistent placeholder-id -> backend-id reconciliation map, or a get-or-create /
    idempotency-key wire semantic => a new subsystem, not specified by ADR-010.
  Closing this requires an owner-level create-identity design decision.
verification: N/A (no fix applied). TokenWorld left clean (0 p93 pages). No audit rows to
  assert because no reposix mutation was performed (curl diagnostic is not a reposix path).
files_changed: []
