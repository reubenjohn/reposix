# CONSULT-DECISIONS — decision ledger (bounded to LIVE decisions)

Escalation-valve + owner decision ledger. **Holds only OPEN / live / still-relevant
decisions.** A decision that is CLOSED, implemented, or superseded is **DELETED** — `git
log` / `git show` is the archive (reversible). No unbounded growth.
`[SELF]` = decided under the escalation-valve bar (below E1–E4), recorded not escalated.
`[FABLE]`/`[CONSULT]` = fable-consult invoked. `[OWNER]` = owner decision.

Format: `## <date> [SELF|FABLE|OWNER] <one-line>` then rationale + evidence.

---

## 2026-07-12 [OWNER] Shared-tree contention RESOLVED — session serialization (no parallel tree-writers, no worktree infra)

- **Decision:** Multiple sessions/agents writing the shared working tree concurrently caused git index/commit races and mid-flight `TaskStop`s (v0.14.0 hygiene lane, 2026-07-12). Owner ruling: **serialize** — exactly one session/agent writes the shared tree at a time. No parallel sessions writing the shared tree; **no new worktree infrastructure** (rejected as over-engineering for the current single-machine autonomous cadence).
- **Operational shape:** a coordinator MAY fan out *read-only* inspection agents in parallel, but tree-mutating work (file edits + `git add`/`commit`) runs one agent at a time — dispatch the next tree-writer only after the prior agent's commit has landed.
- **Fix-twice:** doctrine landed in `.planning/ORCHESTRATION.md` (single-writer discipline section).

## 2026-07-12 [OWNER] v0.14.0 AND v0.13.0 tag cuts DELEGATED to the MANAGER (herdr w1:p7)

- **Decision:** Owner delegated both the v0.14.0 and (sequenced after it) the v0.13.0 tag cut+push end-to-end to the outer-loop MANAGER. The manager routes the sub-work (9th probe → mint+ratify aggregate milestone verdict → author tag script → push) THROUGH the workhorse and verifies each artifact; **the workhorse authors artifacts but never cuts or pushes a tag** — it stops at READY-TO-TAG. External mutation (tag push) is pre-approved under this delegation, for the MANAGER only.
- **Evidence:** `.planning/MANAGER-HANDOVER.md` § Live state (2026-07-12); `.planning/SESSION-HANDOVER.md` §3.

## 2026-07-12 [OWNER] Authorized external mutation — land lost-update (shared-cursor) HIGH security fix onto GitHub `origin/main`

- **Authorization:** Owner-authorized external mutation (2026-07-12 manager relay). Landed
  from a throwaway `/tmp` clone; the shared repo working tree / git state was NOT touched
  (concurrent milestone-C2 was actively editing it), and no `cargo` ran locally (C2 held the
  machine-wide cargo token — CI validates post-push).
- **What landed (4 cherry-picks onto `origin/main`, in order):**
  1. `5028542` — catalog-first contract: `113-lost-update-shared-cursor/PLAN.md` (renamed
     from 106 by the renumber below), 6 rows in `quality/catalogs/code.json`, gate script
     `quality/gates/code/lost-update-shared-cursor.sh`.
  2. `34cfbea` + `39f9d64` — **THE CODE FIX (HIGH security value):** the shared cursor no
     longer gates conflict detection, so a concurrent writer's push can no longer silently
     lost-update. Files: `crates/reposix-remote/src/{diff.rs,precheck.rs}` +
     `tests/partial_failure_recovery.rs`. Landed **byte-identical** to
     `backup-lost-update-424d367` (verified `git diff` empty against the backup tip).
  3. `424d367` — docs renumber 106→113 + mark the SURPRISES-INTAKE entry RESOLVED.
- **Renumber LANDED (not deferred).** Rationale: ROADMAP Phase 106 is already taken by
  "Waived tutorials reproduce"; the renumber targets **113**, which sits above the highest
  existing v0.14.0 phase (112) and collides with no active C2 phase. All 4 cherry-picks
  applied cleanly — git auto-merged `code.json` and `SURPRISES-INTAKE.md`; verified
  post-merge: JSON valid, 6 lost-update rows present, RESOLVED-by-P113 marker coherent.
  **Noticed / filed:** the 113 phase dir has a PLAN.md but ROADMAP.md has no `### Phase 113`
  heading (renumber intentionally did not touch ROADMAP) — an orphan-phase-number
  inconsistency for the owner/C2 to reconcile against final v0.14.0 numbering.

## 2026-07-06 [OWNER] RBF-LR-03 pivot — model create/multi-step interactions as a commit sequence with slug→ID translation

- **Context:** The v0.13.0 tag was gated on RBF-LR-03 (ADR-010 §3): a create-partial-fail
  against an id-reassigning real backend (GitHub/JIRA/Confluence) can duplicate a record
  on retry, because the placeholder-id → backend-id mapping has no home and id-matching
  re-plans the already-done create. Offered the owner document-and-defer vs. three point
  fixes (content-match / persist-map / idempotency-key). The owner rejected the framing as
  a point fix and directed a design **pivot** instead.
- **Status of this vision — DIRECTIONAL INSPIRATION, NOT A SPEC.** The slug/symlink/
  commit-sequence model below is the owner's *inspiration for the direction*, captured
  faithfully. The v0.14.0 coordinator-of-coordinators exploration **OWNS the outcome** and
  may converge on a *different* mechanism (idempotency-key, content-match, the
  commit-sequence model, or a synthesis) after prototyping on real backends. The
  exploration is NOT bound to implement this sketch literally — it is bound to solve the
  root problem (placeholder-id has no home → partial-fail duplicates) cleanly.
- **Decision (owner vision, captured faithfully):** Backends OWN their UIDs; the current
  agent-picks-a-placeholder-id mechanism is bad design. Replace it with a **user-authored
  slug** model:
  1. The user creates their own **slug** and pushes.
  2. On push the virtual remote synthesizes a **commit sequence**: (a) a commit that
     translates slug → backend-assigned ID, (b) the correctly ID-named record file, (c) the
     slug becomes a **symlink** under `slugs/` pointing at the ID-named file, (d) an
     invariant that no other slug in `slugs/` points to that ID, (e) a **merge commit** so
     the agent only ever has to **fast-forward**.
  3. **Generalization:** ANY multi-step client↔server interaction is modeled as a
     **series of commits**, so a partial failure leaves a well-defined intermediate state
     the cache + backend can **reconcile by replaying/continuing the sequence** — no
     torn-state ambiguity, no lost mapping.
  4. **Open question (unresolved):** on full success, optionally **squash** the sequence
     for efficiency — owner is unsure whether squashing reintroduces reconciliation
     complications. To be settled by the exploration, not assumed.
- **Directive:** This is "complex and crucial — exactly the kind of thing I meant by
  pivots." Run a **coordinator-of-coordinators** effort that EXPLORES candidate mechanisms,
  PROTOTYPES the top few **against a real backend**, STRESS-TESTS surviving approaches on
  **all available backends** via prototypes before convergence, then implements the
  strategic, clean, debt-free version — accepting potentially large refactors + docs +
  quality-infra/CI changes. Do NOT converge on paper; converge on prototypes that survived
  a real backend. **~Milestone-sized; gate the spend before the prototype phase.**
- **Rationale:** Point fixes each patch the symptom while leaving the placeholder-id
  design — the actual root cause — in place. The commit-sequence model makes partial-fail
  reconciliation a property of the data model rather than a special case.
- **Reversibility:** Fully reversible — this ledger entry + exploration artifacts only; NO
  code or ADR-010 change yet (ADR-010 §3 is revised only AFTER the exploration converges).
  Tag-timing settled separately below (T1).
- **Commit:** 131315c (+ amendment).

## 2026-07-07 [SELF] D3 — zero-shot human-simulation testing becomes a standing milestone-close gate

- **Context:** This session's zero-shot human-simulation testing (fresh, context-free
  agents following only the published docs) is what caught the sim-onboarding break
  (D1) — a gap no in-context agent or existing catalog gate had surfaced.
- **Decision:** Institutionalize as a STANDING milestone-close gate (new agent-ux catalog
  row), not a one-off session activity. Every milestone-close dispatches N fresh,
  context-free agents that install the shipped artifact the way the docs say and attempt
  the documented workflows (read path: init/attach → clone → grep/cat; write path:
  edit → commit → push; recovery: conflict-rebase, blob-limit sparse-checkout). Any
  doc-lie or broken path grades RED.
- **Rationale:** In-context agents share the session's accumulated assumptions and won't
  independently rediscover a docs/reality gap the way a fresh agent following only the
  docs will; this class of gap is exactly what a milestone-close should catch before
  shipping.
- **Reversibility:** Fully reversible — new catalog row + gate, additive.
- **Commit:** (this entry; catalog row to be filed as part of v0.13.1 or v0.14.0
  scoping).

## 2026-07-12 [SELF] D4 — fleet-safety verdict JSONs: UNTRACK over byte-stabilize

[SELF] (2026-07-12): fleet-safety verification JSONs re-dirty CI checkout → chose UNTRACK (git rm --cached) over byte-stabilize. (a) Investigation confirmed NOTHING reads the committed JSON content as a baseline — run.py write-back-merges the copy it just regenerated this run (never diffs committed bytes); catalog expected.artifact fields are write-targets not read-back baselines; no verifier/verdict.py/test_audit_field.py compares committed bytes. Pure per-run outputs. (b) .gitignore:72 verifications/*/*.json ALREADY ignores them; force-added despite the pattern; p93/perf baselines carry explicit ! re-includes, these do not. (c) Exact P102 precedent fbe02c8 git rm --cached on force-added per-run transcripts; shell-coverage.json/*.cobertura.xml already gitignored+untracked. (d) Byte-stable (309f0b6) is fundamentally fragile: the JSON's only non-static fields (asserts_passed/failed, exit_code) derive from live guard-scenario ASSERT outcomes that can flip PASS↔FAIL across git-version/env between CI and local — 309f0b6 removed the ts field but cannot make asserts environment-invariant. Untracking removes the failure class entirely.

## 2026-07-12 [SELF] D5 — should a RED release-plz block phase-close via ci-green-on-main P0 bar?

[SELF] (2026-07-12): should a RED release-plz block phase-close via the ci-green-on-main P0 bar? YES in principle — this BLOCKER proves an unwatched red release workflow rots silently (Global CLAUDE.md: health is a maintained asset; never let a metric you don't watch decay). But NOT implemented inline: (a) ci-green-on-main.sh hardcodes WORKFLOW=ci.yml; a clean fold = parameterize into a required-workflow list or a sibling code/release-green-on-main row (catalog-first ordering + a verifier grade) — non-trivial. (b) Open semantic question needing verification before P0-wiring: does release-plz run on EVERY push to main, and does a 'no release needed' outcome conclude success or skipped? A false-RED would block UNRELATED phases → warrants owner gate. FILED to SURPRISES-INTAKE with sketch.

## 2026-07-12 [OWNER] dependabot #64-66 closed-as-redundant

[OWNER] (2026-07-12): dependabot PRs #64 (tower-http 0.6.8→0.7.0), #65 (gix 0.83.0→0.85.0), #66 (rusqlite 0.39.0→0.40.1) closed as redundant — `cargo audit` reports 0 live advisories, none touch the flagged crates (memmap2/quinn-proto), bases superseded on current main.

## 2026-07-12 [SELF] D6 — UX error-message audit scheduled as a v0.15.0 first-class phase

[SELF] (2026-07-12): the Rust-compiler-grade UX mandate (every user-facing error teaches the fix / suggests the alternative / gives a copy-paste recovery command; exemplar `reposix-cli/src/init.rs::refuse_existing_repo_root`) is encoded as a standing north-star (root CLAUDE.md § Ownership charter item 5 + new crates/CLAUDE.md § Error-message convention, fix-twice) and scheduled as its OWN v0.15.0 first-class phase — NOT an absorption slot. Rationale: (a) v0.14.0 is being closed honestly and GTH-09 just deferred to avoid a milestone balloon, so new broad work does not belong inside it; (b) the mandate is active immediately so P106's remaining tutorial/onboarding work inherits the three-part bar now, ahead of the phase running; (c) the audit is broad — every CLI subcommand (init/attach/list/sync/doctor/…) AND the reposix-remote git helper (main.rs, stateless_connect.rs) — so it warrants its own phase, not an absorption slot. Reversible: owner (or the v0.14.0-closing C2) can pull it into v0.14.0. Stub committed at `.planning/milestones/v0.15.0-phases/ROADMAP.md`; phase number left unassigned (C2 owns live P102–P113 numbers — collision guard).

## 2026-07-13 [MANAGER] B1 — Confluence page 2818063 mirror-drift: manager-decided restore (Branch 2), executed 2026-07-13

- **Classification:** `[MANAGER]` delegated call under the owner's tag-delegation standing authority (see § 2026-07-12 "v0.14.0 AND v0.13.0 tag cuts DELEGATED to the MANAGER"); TokenWorld is a pre-approved mutation target (root CLAUDE.md OP-6).
- **Finding (read-only diagnosis, opus):** page 2818063 ("reposix demo space Home", the space Home page) was **LEGITIMATELY DELETED out-of-band** (trashed in Confluence UI 2026-07-06T06:28:06Z); Confluence SoT is authoritative, the GitHub mirror is stale. **NOT a reposix data-loss bug** — the mass-delete guard worked as designed. Evidence: live `GET /pages/2818063?status=current` → 404, `?status=trashed` → 200; core `audit_events` had **ZERO DELETE rows** (append-only triggers intact, only 2 PUT/200 updates ≤ 2026-07-04); mirror `reubenjohn/reposix-tokenworld-mirror` HEAD `3be8390` has `pages/2818063.md` never marked `D`. Restore precedent: `91-DECISIONS.md:170` (2818063 previously trashed-then-restored).
- **Manager decision:** Branch 2 — restore/untrash 2818063 in TokenWorld (rationale: restore precedent makes the 3-page state canonical; `docs/reference/confluence.md:123` stays truthful; the vision-litmus reconciles matched=3/backend_deleted=0).
- **Execution (opus, manager-authorized):** `PUT https://reuben-john.atlassian.net/wiki/api/v2/pages/2818063` body `{"id":"2818063","status":"current","title":"reposix demo space Home","body":{...unchanged...},"version":{"number":8,...}}` → HTTP 200, `status=current` (the v2 `updatePage` op documents restore-from-trash on the `status` field: "restore a 'trashed' or 'deleted' page to 'current'"). Only 2818063 touched. Before: space 360450 current = `{7766017,7798785}`. After: `{2818063,7766017,7798785}` — known-good 3-page state, API-verified. Restore is status-only (version stayed 7, content byte-identical).
- **Follow-ups:** (1) the reposix-side vision-litmus PASS is **not yet re-confirmed** — the successor runs it first in item 4 of the tag sequence. (2) `docs/reference/confluence.md:123` lists `2818063.md` — now truthful again (no change needed). (3) self-healing-fixture GTH filed (`GTH-V15-09`, `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md`).

## 2026-07-13 [SELF] B1 — mirror-reconcile to propagate the authorized 2818063 restore to the stale TokenWorld GitHub mirror

- **Context:** The manager-authorized restore of Confluence page 2818063 (see § 2026-07-13 [MANAGER] B1 above) bumped the backend out-of-band via the REST v2 API, so the mirror-head refresh promise never fired and the GitHub mirror `reposix-tokenworld-mirror.git` still carries a stale `pages/2818063.md`. The milestone-close vision-litmus reconcile is CLEAN (matched=3, backend_deleted=0), but its final `git push` is correctly rejected by the push-time lost-update guard because the mirror's stale record version disagrees with the backend-current version.
- **Decision:** Run the documented operational catch-up (`reposix sync --reconcile` for the TokenWorld backend, per root CLAUDE.md § "Mirror-head refresh promise") to refresh the stale mirror to backend-current, then re-run the litmus. This is the sanctioned manual escape hatch for exactly this out-of-band-drift case.
- **Rationale:** Strictly DOWNSTREAM of an already-authorized mutation (the restore) — it only propagates the restored, owner-sanctioned backend state onto the mirror. No new decision about backend content; it heals a coherence gap the out-of-band restore opened. Below the escalation-valve bar (no E1–E4 criterion: reversible, no architecture change, no scope tradeoff, first attempt).
- **Reversibility:** Fully reversible — the mirror ref can be reset to its prior tip; no irreversible backend mutation (the litmus's own edit is the documented sanctioned TokenWorld marker churn).
- **Commit:** 76a5659 (this ledger entry).
- **OUTCOME (2026-07-13, executed):** the documented fix is INSUFFICIENT — `reposix sync --reconcile` rebuilds the LOCAL cache only and does NOT push to the GitHub mirror repo (mirror HEAD `3be8390` byte-identical before→after; litmus re-run still exit 1, `2818063` local base v1 vs backend v7). Root-caused: the stale data lives in the GitHub mirror REPO content (refreshed only by a bus push, which is deadlocked by the stale record), not the cache. **B1 is BLOCKED, escalated to the MANAGER** for a mirror-refresh-path decision (external mutation to the sanctioned mirror via a non-standard path — above the self-decide bar). Evidence + recommended options: `.planning/milestones/v0.14.0-phases/evidence/B1-mirror-reconcile-FINDINGS-2026-07-13.md`; durable fix captured in GTH-V15-09 (amended for the mirror-drift dimension), commit 7fb350d.

## 2026-07-12 [FABLE] B1 litmus: harness artifact vs product gap — honest-green path

- Decision: **(B)** — recommend the OWNER relax B1's non-waivable/no-caveat constraint; ship v0.14.0 with B1 as a DOCUMENTED caveat and route the product fixes to v0.15.0 (precedent: v0.13.0 shipped with the owner-signed duplicate-create known limitation, `docs/concepts/dvcs-topology.md:206`). MANDATORY companion inside the caveat (required by the "docs must remain truthful" constraint, and doc repair is NOT barred product code): scope the over-claiming recovery promises NOW — `docs/guides/troubleshooting.md:259-272` and `docs/concepts/dvcs-topology.md:93` claim the P105/RBF-LR-03 fix makes `git pull --rebase && git push` reconcile "regardless of how the SoT moved," which is true only for `reposix init` (Pattern B) trees and is a doc-lie for the vanilla-clone+`attach` (Pattern C) round-tripper. Option (A) is REJECTED as dishonest; no smaller honest (C) exists.
- Harness-vs-product verdict: **GENUINE PRODUCT GAP, not a harness artifact.** (1) Lineage: `crates/reposix-remote/src/main.rs:400-418` `resolve_import_parent()` chains the synthesized bus history onto the static literal `refs/reposix/origin/main`; absent → `None` → "parentless root" (`main.rs:380`). Only `reposix init` creates that ref (`crates/reposix-cli/src/init.rs:298-304` configures the `remote.origin.fetch` refspec into `refs/reposix/origin/*`); `reposix attach` does a plain `git remote add` (`crates/reposix-cli/src/attach.rs:259`) and never seeds it. So for EVERY attach tree — not just the litmus — the bus fetch is a synthetic unrelated root (`[new branch] main -> reposix/main`, transcript:58) and the documented recovery becomes a cross-root rebase → `add/add` conflict on every shared record (transcript:23-37,66). The litmus topology IS the documented Pattern C thesis path (`docs/concepts/dvcs-topology.md:133-156`); a real user doing exactly what the docs say hits the identical wall. (2) ADF: `adf_to_markdown failed; using empty body` (transcript:42, `reposix_confluence::translate`) is a WARN-and-degrade that substitutes an empty body for unparseable real-backend content — a silent data-loss product path exposed by, not manufactured by, the fixture.
- Rationale: Option (A)'s bus-tracking-ref checkout converts the litmus from Pattern C to Pattern B — it greens by ceasing to test the broken documented path, and (since a bus-ref checkout starts backend-current) the drift/recovery leg would never fire: a doubly hollow green masking two real defects (Pattern-C recovery gap + ADF empty-body data loss). Candidate (C)s all fail: attach/helper parent-chaining or fail-closed ADF translate are product code (barred mid-sequence); pre-refreshing the mirror to eliminate drift is itself blocked on the separate escalated mirror-refresh decision and would green only by ensuring the known-broken recovery never fires. The honest v0.14.0 story is a green-with-documented-caveat plus truthful docs; v0.15.0 product fixes: (a) extend RBF-LR-03 parent-chaining to the attach topology (attach seeds `refs/reposix/origin/main` from the tree's tip, or `resolve_import_parent` falls back to the tree's HEAD/tracking ref), (b) `adf_to_markdown` fails closed instead of emptying the body, (c) then re-green the litmus on the unmodified Pattern-C harness.
- Risks + what would change this answer: if the owner refuses to relax non-waivable, the only honest alternative is authorizing the PRODUCT fix (a)+(b) mid-sequence — i.e. the constraint pair stays unsatisfiable, as the finding states. Verdict would flip to "harness artifact" only if attach were shown to seed `refs/reposix/origin/main` somewhere unexamined (it is not — attach.rs has zero `refs/reposix` writes) or if Pattern C were documented as read-only (it is documented as the write-capable round-tripper path).
- Spot-checks performed: `B1-litmus-selfheal-INSUFFICIENT-FINDINGS-2026-07-12.md` (full); transcript `milestone-close-vision-litmus-real-backend.txt` (full, 68 lines); `quality/gates/agent-ux/lib/litmus-flow.sh` (full, incl. d413432 self-heal block L100-111); `milestone-close-vision-litmus.sh` (full); `docs/concepts/dvcs-topology.md` (full); `docs/guides/troubleshooting.md` (recovery sections); `crates/reposix-remote/src/main.rs:360-440`; `crates/reposix-cli/src/attach.rs` (remote wiring); `crates/reposix-cli/src/init.rs` (refspec doc + config site).

## 2026-07-12 [OWNER] B1 vision-litmus: blessed self-heal proven insufficient — relax non-waivable, or authorize mid-sequence litmus-substrate redesign?

- Context: DECISION-1 blessed a bus-fetch-rebase self-heal as B1's genuine fix under a "non-waivable P0, NO caveat escape" constraint. Implemented (d413432) + proven to fire, but proven to be UNABLE to green the litmus — two real substrate gaps (unrelated bus/mirror ancestry; unparseable ADF fixture). Evidence: .planning/milestones/v0.14.0-phases/evidence/B1-litmus-selfheal-INSUFFICIENT-FINDINGS-2026-07-12.md
- The conflict: B1 is (a) non-waivable/no-caveat AND (b) no product/defect fix mid-tag-sequence — and the one sanctioned mid-sequence change is proven insufficient. Unsatisfiable together.
- Options: (A) authorize a litmus-substrate redesign NOW (bus-tracking-ref checkout + parseable durable fixture) — honors non-waivable, costs E2 architecture work mid-sequence + tag delay. (B) OWNER relaxes B1's non-waivable/no-caveat constraint → ship v0.14.0 with B1 as a documented caveat, substrate fix routed to v0.15.0 (precedent: v0.13.0 GREEN-with-caveats). (C) a smaller honest litmus fix if one exists (fable consult is hunting this).
- Executor recommendation: (B) documented caveat + v0.15.0 fix. This CONTRADICTS the non-waivable constraint → owner must relax it.
- Decision: OPEN — awaiting owner/manager.
- Reversibility: recording is free; no product change pending the decision.

## 2026-07-13 [OWNER] fix-first calibration for tag-blocking product bugs

- **Decision:** Product fixes are AUTHORIZED before a milestone tag. Tag-blocking product bugs default to FIX-FIRST — no owner consult needed — UNLESS the fix turns architectural (then STOP + escalate to manager/owner). Supersedes the prior §4 HOLD and the non-waivable-B1 bind. Relayed by the manager under standing tag authority (2026-07-13).
- **Supersedes:** the `.planning/SESSION-HANDOVER.md` §3/§4 "no product/defect fix mid-tag-sequence" HOLD (successor #5 handover), and resolves the OPEN `2026-07-12 [OWNER] B1 vision-litmus: blessed self-heal proven insufficient — relax non-waivable, or authorize mid-sequence litmus-substrate redesign?` entry above — B1 is now folded into the fix-first item-4a lane (attach-lineage fix), not a standalone waiver/redesign choice.
- **Reversibility:** Fully reversible — a calibration rule, not a specific mutation; each individual fix still passes its own sim-first/tests/code-review gates before landing, and a fix that turns out architectural still escalates per the exception clause.

## 2026-07-13 [MANAGER] APPROVED — drop stale mirror record `pages/2818063.md` to unblock the v0.14.0 vision litmus

- **Classification:** `[MANAGER]` call under the owner's tag-delegation standing authority (see § 2026-07-12 "v0.14.0 AND v0.13.0 tag cuts DELEGATED to the MANAGER"); TokenWorld + its sanctioned mirror are pre-approved mutation targets under that delegation.
- Context: item-4 attach-lineage fix SHIPPED GREEN (`22a7777`), but the milestone-close vision litmus (`quality/gates/agent-ux/dark-factory/dvcs-third-arm.sh`) hard-REDs at GUARD A — the litmus mirror carries a stale record for a TRASHED backend page (`pages/2818063.md`), independent of the fix. Blocks item 5 → item 8 (9th probe) → READY-TO-TAG.
- **Decision (2026-07-13):** APPROVED-by-manager-standing-authority. DROP the stale record `pages/2818063.md` from the mirror (NOT restore — restore adds a 3rd page, breaks the durable-pair contract).
- **Verified basis (manager relay, 2026-07-13):** (a) page 2818063 is TRASHED on the Confluence backend; (b) TokenWorld currently = EXACTLY the 2 protected pages (`7766017` + `7798785`); (c) mirror target confirmed owner-named-sanctioned = `reubenjohn/reposix-tokenworld-mirror`, the litmus's own configured mirror (`quality/gates/agent-ux/milestone-close-vision-litmus.sh:107`); (d) `REPOSIX_LITMUS_MIRROR` confirmed **unset** this session → `MIRROR_URL` defaults to the sanctioned target (verified 2026-07-13, successor #7).
- **Guardrails (binding on execution):** DROP only, never restore; TokenWorld must show EXACTLY 2 current pages with the protected pair intact BEFORE and AFTER (`python3 scripts/confluence_tokenworld.py list`); prefer the documented mirror catch-up path (`docs/concepts/dvcs-topology.md`) over ad-hoc mirror surgery; audit rows must land for the mutation (OP-3); confirm `REPOSIX_LITMUS_MIRROR` unset-or-identical before any push.
- Rationale: DROP realigns the mirror with the current 2-page TokenWorld backend; restore would violate the durable-pair contract. Evidence: `.planning/milestones/v0.14.0-phases/evidence/B1-mirror-reconcile-FINDINGS-2026-07-13.md`.
- Reversibility: mirror push is a git ref/tree change (revertible); touches the sanctioned fixture but is downstream of an already-authorized backend state (the 2-page durable pair).
- Commit: `73f374b` (ruling recorded).

### ⚠ EXECUTION HALTED 2026-07-13 (successor #7) — approved DROP proven INSUFFICIENT; re-escalated [MANAGER]-pending

Before executing, a read-only investigation (opus) + direct code verification found the approved **DROP-only** remediation **cannot re-green the litmus** and conflicts with the "exactly 2 pages" guardrail. **Nothing was mutated** — no mirror push, no backend write.

- **Confirmed finding (code-verified, not just read):** After a DROP the mirror holds only the two PROTECTED pages (`7766017`+`7798785`). The unmodified litmus target-selection loop (`quality/gates/agent-ux/lib/litmus-flow.sh:64-72`) skips `PROTECTED_IDS` and needs an **editable, non-protected** `pages/<id>.md` to round-trip; with none left it hard-fails at `:72` (`"no editable non-protected pages/<id>.md record in the clone"`). **`2818063` WAS the litmus's sacrificial editable page.** So DROP converts the GUARD-A red (`:47`) into a target-selection red (`:72`) — the litmus is still RED, just differently.
- **Root contradiction:** the unmodified Pattern-C litmus **requires 3 pages** (2 protected + 1 sacrificial editable that is ALSO backend-current so GUARD A's `backend_deleted=0` holds). The "TokenWorld = EXACTLY 2 pages" guardrail is **incompatible** with greening the unmodified harness. (The earlier `2026-07-13 [MANAGER] B1` RESTORE decision — 3 pages, `matched=3` — recognized this; the subsequent flip to DROP dropped that insight.)
- **Sanctioned mirror-refresh path is BROKEN:** the mirror-sync GitHub Action's only run (`25223195636`, 2026-05-01) FAILED at `cargo binstall reposix-cli` (P84 crates.io substrate gap), and `reposix sync --reconcile` rebuilds only the LOCAL cache (prior `[SELF] B1` finding). So the ONLY available DROP mechanism is **manual git surgery on the mirror** (clone→`git rm`→push), which bypasses reposix and writes **NO audit row** (OP-3 concern) — violating the "prefer the documented catch-up path" guardrail.
- **Recommendation to MANAGER (three honest options):**
  1. **RESTORE `2818063` (revert to the 3-page fixture)** — untrash on the backend (PUT `status=current`, exactly as `[MANAGER] B1` did → HTTP 200; the mirror already carries the record, so NO mirror surgery, NO audit bypass). GUARD A then passes `matched=3/backend_deleted=0` and `2818063` is the edit target. Residual risk: the final push may hit a version-mismatch (mirror `v1` vs backend `v7`) that the litmus's own `git pull --rebase reposix main` recovery (`:104-107`) is designed to self-heal — needs a real-backend test run to confirm. Reframes the "durable-pair contract" as "2 protected pages never deleted + 1 sacrificial editable" (the pair stays protected).
  2. **Authorize a litmus-harness change** to edit a protected page (drop the denylist for the edit target) — but the charter says re-green on the UNMODIFIED harness, so this needs explicit owner relaxation.
  3. **Accept the 9th probe as documented-caveat NOT-VERIFIED**, route the fixture/substrate repair (broken mirror-sync Action + fixture shape) to v0.15.0 — but the `pre-release-real-backend` row is P0/non-waivable, so this likely requires an owner waiver-relaxation.
- **Successor #7 recommends Option 1 (RESTORE)** — cleanest, no audit bypass, matches the earlier correct manager call; but it DIRECTLY CONTRADICTS the standing "DROP only, never restore / exactly 2 pages" relay, so it is the MANAGER's call, not self-decidable. Evidence: this session's investigation + `litmus-flow.sh:47,64-72,104-107`; broken Action run `25223195636`.

#### RESOLVED 2026-07-13 — APPROVED-RESTORE-by-manager-standing-authority (Option 1)

The MANAGER independently **code-verified** the finding (`litmus-flow.sh:47` GUARD-A red on `backend_deleted!=0`; `:64-72` protected-denylist target-selection ⇒ DROP can never green; `:101-108` bounded rebase recovery) and ruled: **Option 1 RESTORE APPROVED. The prior DROP-only / exactly-2-pages guardrail was built on successor #6's false premise and is WITHDRAWN.** Relay 2026-07-13.
- **Binding guardrails (manager relay):** (1) ONLY backend mutation = untrash `2818063` (`status=current`) via the committed NAMED tool `python3 scripts/confluence_tokenworld.py restore` — NEVER ad-hoc curl; protected pair `7766017`/`7798785` untouched (the script refuses to delete them). (2) Verify before+after with `scripts/confluence_tokenworld.py list` — protected pair intact, end state = EXACTLY 3 current pages (`7766017`,`7798785`,`2818063`). (3) NO mirror surgery — the mirror record stays (that IS the point of RESTORE). (4) Re-run the litmus on the UNMODIFIED Pattern-C harness; if the final push hits the v1-vs-v7 version mismatch and the ONE documented rebase recovery (`:105`) does NOT self-heal, surface it **RED honestly** — that is a real coherence bug and fix-first applies. (5) **Fix-it-twice doc-truth:** the EXACTLY-2-pages fixture doctrine is WRONG — correct shape = 2 protected pages never deleted + 1 sacrificial editable (`2818063`, current or trashed between runs); eager-fix `docs/reference/testing-targets.md` + the SESSION-HANDOVER constraint line (manager fixes MANAGER-HANDOVER.md). (6) File the broken mirror-sync Action (run `25223195636`, cargo-binstall P84 gap) as intake — it stays broken under RESTORE.
- **Status:** EXECUTED by successor #8 → litmus came back **RED (real coherence bug)** — see the EXECUTED subsection below. Commit: `73f374b`→`a1b06c3` (finding) + `c46ae55` (ruling).

#### EXECUTED 2026-07-13 (successor #8) — RESTORE done, litmus RED (real coherence bug); tag HALTED, [OWNER/MANAGER]-pending

The RESTORE lane ran EXACTLY per the 6 guardrails (opus, sole tree-writer). **The restore succeeded; the litmus it gates did NOT self-heal — guardrail 4's RED branch fired.** Not papered over.

- **Restore OK:** `restore 2818063` → `status current (version 7)`. TokenWorld AFTER = **EXACTLY 3 current pages** (`2818063` current-v7, `7766017` PROTECTED, `7798785` PROTECTED). Guardrail-2 assertion PASSED. Guardrail 1/2/3 satisfied (no mirror surgery, protected pair untouched).
- **9th probe `run.py --cadence pre-release-real-backend --persist` exit 1** (2 PASS / 4 FAIL):
  - **litmus (`agent-ux/milestone-close-vision-litmus-real-backend`) FAIL** — final push hit the anticipated v1-vs-v7 mismatch; the ONE documented recovery (`git pull --rebase reposix main`, `quality/gates/agent-ux/lib/litmus-flow.sh:100-111`) collapsed to `CONFLICT (content): Merge conflict in pages/2818063.md` → fail at `:108` (`"a REAL coherence bug, not mirror lag"`).
  - **p93 (`p93-partial-failure-recovery-real-confluence`) FAIL (exit 101)** — independent confirmation: panic at `crates/reposix-cli/tests/agent_flow_real.rs:925`, recovery push returned `some-actions-failed`. NOTE: p93 was reported GREEN at 15:28 today (item 7) and RED at 19:06 — the trigger (RESTORE artifact vs product regression) is NOT yet pinned down.
- **Prime root-cause suspects (lane NOTICED, code-reading — needs a repro to confirm):** (1) `adf_to_markdown` fails for ALL 3 pages (`root node type must be "doc", got ""`) → bodies fetch as the fail-closed unreadable-ADF sentinel → guarantees the rebase content conflict; hits the PROTECTED pair too (broader translate bug). (2) delta-sync cursor bug — `last_fetched_at` advanced past a concurrent write; `list_changed_since` silently dropped `2818063` (only the push-time lost-update guard caught it).
- **Guardrails 5/6 NOT executed** (agent stopped at the RED branch per instruction): doc-truth fix + mirror-sync intake still TODO for the successor.
- **DP/valve classification:** repro EXISTS (executed litmus + p93) so DP-2 prove-before-fix is met, BUT the fix is a product-code debug campaign of **unknown, possibly >1-phase-slot** size with an unpinned trigger → **escalation valve E3 (release-timeline/scope tradeoff = owner's/manager's call)**. fix-first authorizes the fix in principle (`2026-07-13 [OWNER] fix-first calibration`), but fix-vs-waive-vs-rescope + WHEN (delay the tag) is the manager's decision, not self-decidable.
- **Successor #8 recommendation:** authorize a **bounded read-only DIAGNOSTIC lane FIRST** (root-cause the empty-ADF translate + the 15:28-green→19:06-red delta — is the trigger `2818063`'s specific empty-ADF body, or a genuine `reposix-confluence` regression?) BEFORE committing to a fix campaign; then fix → re-run item 5 → item 8. Do NOT waive: this is a real UPDATE-recovery coherence failure, distinct from the already-WAIVED CREATE-recovery RBF-LR-03 gap. Evidence: dirty→committed `quality/catalogs/agent-ux.json` honest grades; lane report in this session's transcript.
- **Fixture cleanup owed:** orphan test page `9994241` left in TokenWorld (p93 teardown got Confluence 500 on DELETE) — `python3 scripts/confluence_tokenworld.py delete 9994241`.
- **Catalog honesty caveat:** the committed `agent-ux.json` also flipped `t4`(P0) + `github-front-door`(P1) NOT-VERIFIED→FAIL, but their `owner_hints` say "verifier not implemented" → they SHOULD read NOT-VERIFIED. Suspected runner mis-grade (not hand-corrected — that would be papering over a separate bug); filed for the fix lane.

##### RULED 2026-07-13 [MANAGER] (E3 valve, standing authority + owner fix-first) — tag HALTED; bounded DIAGNOSTIC lane FIRST

- **Decision:** (1) Tag stays HALTED; do NOT waive (concur: UPDATE-recovery failure ≠ WAIVED CREATE-recovery RBF-LR-03). (2) APPROVED a bounded DIAGNOSTIC lane (opus; NO product-code mutations; TokenWorld sacrificial-page repro writes pre-authorized, protected pair untouched) to pin: (a) empty-ADF `adf_to_markdown` (`root node type must be "doc", got ""`) = real translate regression vs `2818063`-restore artifact — verify against a FRESH cache since it allegedly hits the protected pair too; (b) the p93 15:28-green→19:06-red delta; (c) the delta-sync `last_fetched_at` cursor suspect. Deliverable = COMMITTED report: pinned root cause + fix-size estimate + BOUNDED-vs-ARCHITECTURAL classification. (3) After diagnosis: **BOUNDED → fix-first, proceed autonomously** (fix → re-run item-5 litmus → item-8 mechanicals → STOP at READY-TO-TAG); **ARCHITECTURAL → HALT + escalate to manager**. (4) Same wave: guardrails 5/6 + delete orphan `9994241` + keep the runner mis-grade FILED (never hand-correct). (5) Successor #8 continues as executor (manager waived rotation at "14%"; successor #8 rotated anyway at the ABSOLUTE ~142k token hard-stop — doctrine relief is token-absolute, not %-of-window — handing the diagnostic charter to #9 so tag-critical work runs on a fresh context; continuity preserved via this ledger + the committed diagnostic report).
- **Reversibility:** ruling is a process decision; the diagnostic lane makes no product-code mutation and only sanctioned-fixture repro writes (revertible). Guardrail 6 landed in the deck-clearing commit below; guardrail 5 (testing-targets.md doc-truth) was drafted then REVERTED — any `docs/**` edit trips the P0 `docs-alignment/walk` pre-push gate, needing a `/reposix-quality-refresh` orchestration — so it is RE-OWED to #9; orphan-delete `9994241` owed (creds).
