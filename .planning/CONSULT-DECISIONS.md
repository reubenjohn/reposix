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
