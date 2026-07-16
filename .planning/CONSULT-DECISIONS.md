# CONSULT-DECISIONS — decision ledger (bounded to LIVE decisions)

Escalation-valve + owner decision ledger. **Holds only OPEN / live / still-relevant
decisions.** A decision that is CLOSED, implemented, or superseded is **DELETED** — `git
log` / `git show` is the archive (reversible). No unbounded growth.
`[SELF]` = decided under the escalation-valve bar (below E1–E4), recorded not escalated.
`[FABLE]`/`[CONSULT]` = fable-consult invoked. `[OWNER]` = owner decision.

Format: `## <date> [SELF|FABLE|OWNER] <one-line>` then rationale + evidence.

> **2026-07-14 prune:** the v0.14.0 tag saga (Rulings #2–5, the B1/litmus/t4 escalations,
> D3–D6, session-serialization, fix-first calibration, tag-delegation, external-mutation,
> dependabot) all CLOSED with v0.14.0 shipping — deleted here; the canonical record is
> `quality/reports/verdicts/milestone-v0.14.0/` + git history. Two of those rulings encoded
> STANDING doctrine, migrated before deletion: **single-writer / session-serialization** →
> `ORCHESTRATION.md` §2; **fix-first for tag-blocking product bugs** → `ORCHESTRATION.md` §11.

---

## 2026-07-06 [OWNER] RBF-LR-03 pivot — model create/multi-step interactions as a commit sequence with slug→ID translation

- **Status (2026-07-14):** OPEN / partially delivered. The *rebase-recovery
  deep-reconciliation* half of RBF-LR-03 shipped in v0.14.0 (Phase 105; see
  `docs/decisions/010-l2-l3-cache-coherence.md`). The **create-partial-fail** half — the
  commit-sequence / slug→ID redesign captured below — is **still unbuilt**; ADR-010 §3's
  convergence contract is revised only after that exploration converges. Kept live here as
  the canonical owner directive for that redesign.
- **Context:** A create-partial-fail against an id-reassigning real backend
  (GitHub/JIRA/Confluence) can duplicate a record on retry, because the placeholder-id →
  backend-id mapping has no home and id-matching re-plans the already-done create. Offered
  the owner document-and-defer vs. three point fixes (content-match / persist-map /
  idempotency-key). The owner rejected the point-fix framing and directed a design **pivot**.
- **Status of this vision — DIRECTIONAL INSPIRATION, NOT A SPEC.** The slug/symlink/
  commit-sequence model below is the owner's *inspiration for the direction*, captured
  faithfully. The exploration **OWNS the outcome** and may converge on a *different*
  mechanism (idempotency-key, content-match, the commit-sequence model, or a synthesis)
  after prototyping on real backends. It is NOT bound to implement this sketch literally — it
  is bound to solve the root problem (placeholder-id has no home → partial-fail duplicates)
  cleanly.
- **Decision (owner vision, captured faithfully):** Backends OWN their UIDs; the current
  agent-picks-a-placeholder-id mechanism is bad design. Replace it with a **user-authored
  slug** model:
  1. The user creates their own **slug** and pushes.
  2. On push the virtual remote synthesizes a **commit sequence**: (a) a commit that
     translates slug → backend-assigned ID, (b) the correctly ID-named record file, (c) the
     slug becomes a **symlink** under `slugs/` pointing at the ID-named file, (d) an
     invariant that no other slug in `slugs/` points to that ID, (e) a **merge commit** so
     the agent only ever has to **fast-forward**.
  3. **Generalization:** ANY multi-step client↔server interaction is modeled as a **series
     of commits**, so a partial failure leaves a well-defined intermediate state the cache +
     backend can **reconcile by replaying/continuing the sequence** — no torn-state
     ambiguity, no lost mapping.
  4. **Open question (unresolved):** on full success, optionally **squash** the sequence for
     efficiency — owner is unsure whether squashing reintroduces reconciliation
     complications. To be settled by the exploration, not assumed.
- **Directive:** "complex and crucial — exactly the kind of thing I meant by pivots." Run a
  **coordinator-of-coordinators** effort that EXPLORES candidate mechanisms, PROTOTYPES the
  top few **against a real backend**, STRESS-TESTS survivors on **all available backends**
  before convergence, then implements the strategic, clean, debt-free version — accepting
  potentially large refactors + docs + quality-infra/CI changes. Do NOT converge on paper;
  converge on prototypes that survived a real backend. **~Milestone-sized; gate the spend
  before the prototype phase.**
- **Rationale:** Point fixes each patch the symptom while leaving the placeholder-id design
  — the actual root cause — in place. The commit-sequence model makes partial-fail
  reconciliation a property of the data model rather than a special case.
- **Reversibility:** Fully reversible — this ledger entry + exploration artifacts only; NO
  code or ADR-010 change yet (ADR-010 §3 is revised only AFTER the exploration converges).
  Tag-timing settled separately (T1, git history).
- **Commit:** 131315c (+ amendment).

---

## 2026-07-16 [OWNER] strip retirement-history narrative from user-facing docs

- **Rationale:** owner directive 2026-07-16 — user-facing docs carry current truth only;
  correction history lives in git history + planning artifacts; removing complexity.
- **What was removed:** old-figure retirement stories — the 89.1%/85.5% section in
  `docs/benchmarks/token-economy.md`, the 4,883/531 origin sentence in
  `docs/concepts/reposix-vs-mcp-and-sdks.md`, the retired-figure clause in `docs/index.md`,
  and the "Superseded figures" paragraph in `docs/benchmarks/latency.md`.
- **What was kept:** current live numbers and all current-measurement caveats (read-only
  write-back scope, MCP-lossy caveat, live-capture provenance).
- **Evidence:** commit 5a5dd29, mkdocs-strict + mermaid + banned-words + docs-alignment
  walk all green, zero rows orphaned (2 latency rows re-bound for line shift).

---

## 2026-07-16 [OWNER] docs site must read as a furnished product — P117/P119 quality bar

- **Context:** received after commit `a1f2494`, alongside the SPECIFIC eager-fix directive
  to nest "Build from source (advanced)" under "30-second install" in `docs/index.md`
  (filed as `GOOD-TO-HAVES.md` GTH-V15-35, scheduled immediately post-P115-close as a
  tracked `/gsd-quick`).
- **Decision (owner quote, verbatim):** *"Its good, but we can do so much better!"* — the
  mkdocs docs site should read as a **FURNISHED PRODUCT** with streamlined documentation,
  not merely factually correct or destaled. This is the explicit quality bar for **P117**
  (doc-truth purge) and **P119** (docs simplification), covering information architecture,
  progressive disclosure (30-second path first, advanced/edge material pushed down),
  visual polish of the mkdocs site, and a cold-reader rubric pass over every landing
  surface.
- **Filed:** `GOOD-TO-HAVES.md` GTH-V15-36 (full text + fix-sketch); annotated inline on
  both `.planning/ROADMAP.md` Phase 117 and Phase 119 entries as "REQUIRED planning
  input" (the milestone-scoped `v0.15.0-phases/ROADMAP.md` is a superseded stub per
  GTH-V15-27 — the live P117/P119 entries are in the top-level `.planning/ROADMAP.md`).
- **Also owner-approved same session:** a P117 scope addition embedding the owner's 80s
  launch animation on the mkdocs home page, productionization checklist filed as
  `GOOD-TO-HAVES.md` GTH-V15-37, referenced from the P117 ROADMAP annotation.
- **Status:** OPEN — shaping input for the P117/P119 planners; not itself an
  implementation task.

