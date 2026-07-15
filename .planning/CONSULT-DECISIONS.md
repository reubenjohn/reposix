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

## 2026-07-15 [SELF] A1 — one "benchmark session" (≤50 ceiling) = one live agentic conversation, not one metered API call

- **Context:** P115 (BENCH-01 live MCP benchmark re-measurement) execution is gated on
  defining the unit of the owner's spend ceiling ("up to 50 benchmark sessions on the
  existing subscription, no new API dollars, escalate only past 50", 2026-07-14).
  Planning (115-PLAN.md) correctly did not assume it; #31 routed the ruling to the
  manager. Low-stakes: 5 of 8 benchmark rows are latency-track at zero session budget;
  only 3 token rows consume sessions.
- **Decision:** One benchmark session = **one live agentic conversation / task run**
  (fresh context through task completion or abort), regardless of how many internal API
  calls or turns it makes. Failed/aborted runs still count against the 50. Re-runs are
  new sessions. The benchmark ledger records per session: id, date, benchmark row, model,
  outcome, and approximate token totals. Guard: any single session ballooning past ~5x
  the median token cost of prior sessions is flagged in the ledger, not silently
  absorbed. Past 50 sessions → owner escalation (already owner-directed).
- **Rationale:** Matches the owner's phrasing ("sessions on the existing subscription" —
  subscription usage is consumed by conversations, not metered per-call dollars) and the
  benchmark's own measurement unit (cost/latency per task run). The per-API-call reading
  would make the ceiling incoherent: one agentic task is dozens of calls, so the cap
  would bind on turn count rather than benchmark work, and no meaningful "50" survives.
- **Reversibility:** Fully reversible until sessions are spent — redefine before
  execution consumes budget; disclosed via handover + this ledger (owner veto window).
- **Commit:** (this commit). DELETE this entry once P115's ledger/capture tasks encode
  the definition and the phase closes.

## 2026-07-15 [SELF] P115-T2 — canonical latency measurement environment = CI `bench-latency-v09`; sim cold-init is environment-dependent, not one fixed value

- **Context:** P115-T2 re-measured sim latency to resolve the documented 24-vs-27ms cold-init discrepancy to "one authoritative figure." Re-measurement showed sim `init` is highly environment-dependent: 27ms (legacy dev machine), 42-45ms (this dev VM, warm, N=3), 155ms (dev VM cold/loaded first-run outlier — the mistaken figure in commit 9384ca6), 278ms (CI `bench-latency-v09`, ubuntu-24.04 hosted runner, commit 3278abc). No single stable number exists; "resolve to ONE figure" required a canonical-environment decision the plan did not anticipate.
- **Decision:** The canonical, authoritative latency reference is the **CI `bench-latency-v09` job** — reproducible (documented ubuntu-24.04 runner image), runs on every push, and measures sim AND all three real backends from one controlled environment, verifiable from the run log. `docs/benchmarks/latency.md` reports the CI figures with run id + runner image as provenance. Dev-VM measurements are non-canonical (machine-load-dependent). The legacy 24/27ms figures are superseded.
- **Rationale:** OP-1 verify-against-reality — a hero number must be reproducible from a committed artifact (the CI log), not a one-off dev-VM sample.
- **Downstream (flagged to manager, non-blocking):** the honest CI sim cold-init (~278ms) is ~10x the legacy 27ms hero claim, so un-waiving the latency-track doc-alignment rows (T6) requires CHANGING the doc CLAIMS to the CI figures, not merely re-verifying the old numbers. Overlaps v0.21 "benchmark honesty" scope; surfaced to the manager (w1:p7) for owner awareness. Honesty is non-negotiable, so the correction proceeds regardless.
- **Reversibility:** Fully reversible — docs + this ledger entry only; no code/contract change. DELETE once T6 encodes the canonical-source methodology and the phase closes.
- **Commit:** (this commit).
