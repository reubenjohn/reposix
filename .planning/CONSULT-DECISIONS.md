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

## 2026-07-07 [SELF] D1 — v0.13.1 onboarding hotfix, sequenced BEFORE the v0.14.0 pivot

- **Context:** Zero-shot human-simulation testing (3 independent fresh-agent
  reproductions) found binary-install onboarding is 100% broken: `reposix-sim` (the
  documented DEFAULT backend, OP-1) ships in no prebuilt distribution, and `reposix init`
  silently exits 0 when the backend is unreachable, masking the failure.
- **Decision:** Ship a scoped v0.13.1 hotfix BEFORE the v0.14.0 reconciliation pivot.
  Acceptance (end-state; mechanism converges in discuss/plan): (i) the documented
  getting-started flow completes end-to-end on the shipped binary, not a source build;
  (ii) `reposix init` exits non-zero on an unreachable backend; (iii) the release-path
  sim→cargo fallback that hides the gap is removed; (iv) verified by a fresh zero-shot
  human-simulation agent (D3). Bias toward shipping `reposix-sim` in the release matrix
  (OP-1 makes it canonical), but honest de-advertisement is an acceptable convergence if
  shipping sim proves disproportionate.
- **Rationale:** An adoption-blocker on `releases/latest` cannot wait behind a
  milestone-sized pivot; ship honest milestones, don't let an urgent regression sit
  behind a large redesign (owner design taste).
- **Reversibility:** Fully reversible — sequencing + scope only.
- **Commit:** (this entry; SESSION-HANDOVER.md encodes the runbook).

## 2026-07-07 [SELF] D2 — v0.14.0 hardening: reject-t@t-identity hook + worktree isolation are P0

- **Context:** A dispatched sim/seed leaf corrupted the local shared repo TWICE this
  session (flipped `core.bare=true`; set `user.email`/`user.name` to `t<t@t>`). Root
  cause: agent worktrees are not isolated (shared `.git/config` + object store) plus cwd
  resets between Bash calls. The doc-only HARD-STOP rule (ORCHESTRATION.md § "Leaf
  isolation") did not prevent the recurrence.
- **Decision:** Treat a commit-time guard + real worktree isolation as P0 for v0.14.0
  hardening scoping. Sketch: a pre-commit/pre-push hook that hard-rejects any commit
  authored by `t<t@t>` (or any non-allowlisted identity), plus per-leaf isolated `/tmp`
  clones and unique `REPOSIX_CACHE_DIR` enforcement per leaf.
- **Rationale:** A doc rule alone did not stop a second recurrence in the same session;
  the guard needs to be code-enforced, not convention-enforced.
- **Reversibility:** Fully reversible — new hook + isolation convention, additive.
- **Commit:** (this entry; anchor intake `S-260707-pr-08`, HIGH).

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
