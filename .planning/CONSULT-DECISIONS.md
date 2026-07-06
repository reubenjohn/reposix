# CONSULT-DECISIONS — decision ledger (bounded to LIVE decisions)

Escalation-valve + owner decision ledger. **Holds only OPEN / live / still-relevant
decisions.** A decision that is CLOSED, implemented, or superseded is **DELETED** — `git
log` / `git show` is the archive (reversible). No unbounded growth.
`[SELF]` = decided under the escalation-valve bar (below E1–E4), recorded not escalated.
`[FABLE]`/`[CONSULT]` = fable-consult invoked. `[OWNER]` = owner decision.

Format: `## <date> [SELF|FABLE|OWNER] <one-line>` then rationale + evidence.

---

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

## 2026-07-06 [SELF] Tag-timing: T1 — ship v0.13.0 now, RBF-LR-03 pivot becomes v0.14.0

- **Context:** With RBF-LR-03 escalated to a milestone-sized pivot (explore→prototype→
  stress-test→converge→clean-impl, not a point fix), "solve before tag" would delay the
  v0.13.0 tag by a full milestone. The owner delegated the call explicitly ("will
  ultimately leave this to you… least complex/confusing way") and named the constraint:
  do NOT suppress gates.
- **Decision:** **T1.** Tag v0.13.0 now. RBF-LR-03 ships as an **honestly-WAIVED,
  documented known-limitation** (narrow: real backend + mid-batch-create network drop →
  one hand-deletable duplicate). The reconciliation pivot becomes the **v0.14.0 headline
  milestone**. T1 requires **NO gate suppression** — the waiver is honest and owner-signed;
  a completed, honest milestone ships rather than being held hostage to a large redesign.
- **Rationale:** Owner design taste — ship honest milestones + document known limitations
  out loud, don't hold a green milestone for a big redesign. The footgun is narrow and
  recoverable; the pivot is too valuable to rush under a tag deadline.
- **Reversibility:** Fully reversible — sequencing only; the tag can be cut at any HEAD.
- **Commit:** (this entry; handover encodes the sequencing).

## 2026-07-06 [SELF] Real-backend 9th probe — VERIFIED (owner's "it's all set" is correct)

- **Context:** Owner asked why the 9th probe reads NOT-VERIFIED when an earlier agent
  concluded it "is all set" — env/perms/worktree suspected. All three ruled out (.env
  present + readable, correct worktree).
- **Crux:** The real-Confluence probe **genuinely ran GREEN.** Committed catalog row
  `agent-ux/milestone-close-vision-litmus-real-backend` (quality/catalogs/agent-ux.json)
  carries `last_real_grade: "PASS"` / `last_real_verified: 2026-07-05T02:23:17Z`, and a
  FRESH ephemeral PASS exists at `quality/reports/…/…-2026-07-06T06-28-00Z.json` (exit 0,
  real Confluence page 2818063 round-trip, dual-table audit, mirror refs advanced). The
  mechanical `status: NOT-VERIFIED` is **honest-by-design**: this P0 row has NO waiver and
  fails-closed to NOT-VERIFIED whenever re-graded in a shell without creds (env-gate, exit
  75), while **preserving** `last_real_grade`. NOT-VERIFIED ≠ never-passed; it means "the
  last mechanical grading context had no creds." Nothing is misconfigured.
- **Decision:** Treat the real-backend probe as SATISFIED for the tag via committed
  `last_real_grade: PASS` + the 07-06 green transcript. No NEW real-backend call is
  required to tag; the owner need not re-run it. (`quality/reports/**` is gitignored /
  ephemeral by design — the durable record is the catalog's `last_real_grade`.)
- **Reversibility:** N/A (finding, not a change).
- **Commit:** (this entry).
