## v0.14.0 Wave-2 hardening (PLANNING)

> **Status:** scoped 2026-07-11 by owner-settled decision (this document encodes the
> owner's ordering verbatim — it is NOT a discovery/research pass). Phase numbering
> continues from v0.13.1's close: v0.13.0-ext closed at **P97**, v0.13.1 (onboarding
> hotfix) claimed **P98–P101** and SHIPPED 2026-07-07 (tag v0.13.1, `04640d5`). This
> milestone claims **P102–P112**, plus an out-of-band **Phase 113** (lost-update
shared-cursor guard — an emergent silent-data-loss fix that shipped in v0.14.0 and is
reconciled into this ROADMAP at milestone-close; see § Phase 113 below). The
queued-but-not-yet-replanned v0.13.2
> "Cross-link fidelity" milestone (still scoped at its original P98–P107 placeholder
> range) shifts further when it is eventually replanned — same renumber-on-insertion
> convention used for v0.13.1 and, before it, v0.12.1's P66 insertion. This scaffold
> does NOT edit v0.13.2's files.

> **D2 anchor.** `S-260707-pr-08` (`.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`
> — "agent 'worktrees' are NOT isolated; a sim-seed leaf corrupted the shared repo
> (`t <t@t>` flipped `core.bare=true`)", HIGH) is the founding incident. It recurred
> during the v0.13.1 session (`S-260707-pr-08` amendment note + the separate
> `S-260707-rbf-lr03-external-write-crash` triage) — always local, origin never
> affected, but the corruption hit **4× in one session** per STATE.md `last_activity`.
> D2 exists to make this class of incident structurally impossible before any other
> autonomous fleet run is trusted again.

**Thesis.** v0.13.0/v0.13.1 shipped a working DVCS front door but left two classes of
debt: (1) the fleet's own safety substrate (worktree isolation, identity hygiene, shared
`.git/config` protection) is documented-but-not-enforced, and a real corruption already
hit 4× in one session; (2) five carried HIGH-severity items from the v0.13.0/v0.13.1
intake (GitHub real-backend 404, RBF-LR-03 reconciliation, waived tutorials, open RUSTSEC
advisories, a pagination-truncation data-loss hazard) never got a dedicated hardening
pass. v0.14.0 closes both, in that order — the fleet must be self-safe before it is
trusted to run the rest of the backlog unattended.

**Ordering is a hard constraint, not a suggestion.** Phase 102 (D2) is a **serializing
gate**: no other phase in this milestone — and no other autonomous fleet run anywhere in
the project — starts until P102 lands GREEN, pushed, and verifier-PASS. Phases 103–109
(the carried HIGHs + early cheap wins) MAY then run in parallel with each other, subject
to the unconditional CLAUDE.md constraint that **all cargo lanes still serialize
machine-wide** (one `cargo check`/`build`/`test`/`clippy` invocation at a time, project-wide,
regardless of how many phases are logically "in flight").

**Recurring success criteria for EVERY phase (P102–P112)** — non-negotiable per CLAUDE.md
Operating Principles + the autonomous-execution protocol; NOT separate REQ-IDs:

1. **Catalog-first** — phase's FIRST commit writes/updates catalog rows under
   `quality/catalogs/<file>.json` BEFORE any implementation commit (per `quality/CLAUDE.md`).
2. **CLAUDE.md updated in the same PR** (fix-it-twice meta-rule) — any phase introducing a
   new file/convention/gate/hook revises the relevant CLAUDE.md section in the same PR.
3. **Per-phase push** — `git push origin main` BEFORE verifier-subagent dispatch; the
   verifier grades RED if the phase shipped without the push landing.
4. **Phase close = unbiased verifier subagent (OP-7)** — verdict at
   `quality/reports/verdicts/p<N>/VERDICT.md`; the phase does not close on RED.
5. **Eager-resolution preference (OP-8)** — items <1h / no new dependency fixed in-phase;
   else appended to `SURPRISES-INTAKE.md` or `GOOD-TO-HAVES.md`, never silently skipped.
6. **Simulator-first (OP-1)** — sim is the default/only backend exercised except where a
   phase's explicit charter is real-backend verification (P104, P106); real-backend
   mutation there is **owner-gated**, not self-authorized.
7. **Tainted-by-default (OP-2) + audit log non-optional (OP-3)** apply unchanged to any
   phase touching cache/backend/export code paths.
8. **Verify against reality** — hooks/guards claimed as "enforced" must be proven
   triggered (a rejected commit, a blocked write) — not just code-read-asserted.
9. **Cargo serialization** — at most ONE cargo invocation machine-wide, project-wide,
   across every phase in flight; prefer `-p <crate>` over `--workspace`.

---

> **Archived detail:** completed-phase bodies (Goal / Success criteria / Context
> anchor for P102-P111, P113) live in [ARCHIVE.md](./ARCHIVE.md); this ROADMAP keeps
> the phase index (headings + terminal status + pointers). Phase 112's DO-NOT-START
> stub is kept in full below.

### Phase 102: D2 self-safe dark-factory hardening (HARD SERIALIZING GATE)

> Detail (Goal / Requirements / Success criteria / Context anchor) archived → [ARCHIVE.md § Phase 102](./ARCHIVE.md).

### Phase 103: Early cheap wins — doc-alignment grade/persist split + structure waiver OP-8 split

**STATUS: CLOSED (GREEN 2026-07-12)** — unbiased phase-close verdict GREEN (3/3 items),
`quality/reports/verdicts/p103/VERDICT.md`, graded HEAD `d0f23d5`; implementation commits
`12abdfb`/`1136bb3`/`dad227e`.

> Detail (Goal / Requirements / Success criteria / Context anchor) archived → [ARCHIVE.md § Phase 103](./ARCHIVE.md).

### Phase 104: GitHub-v09 helper-path 404 fix (S-260707-gh404)

**STATUS: CLOSED (GREEN 2026-07-12)** — unbiased phase-close verdict GREEN (1/1 gradeable
row PASS + 1 honestly-fail-closed NOT-VERIFIED real-backend row),
`quality/reports/verdicts/p104/VERDICT.md`, verdict commit `22f94d0`, graded HEAD `03942f8`.

> Detail (Goal / Requirements / Success criteria / Context anchor) archived → [ARCHIVE.md § Phase 104](./ARCHIVE.md).

### Phase 105: RBF-LR-03 rebase-recovery reconciliation redesign

**STATUS: CLOSED (GREEN 2026-07-12)** — unbiased phase-close verdict GREEN (row
`agent-ux/rebase-recovery-reconciles`, 13/13 asserts), `quality/reports/verdicts/p105/VERDICT.md`,
verify commit `0d3afe9`, graded HEAD `8afb52d`.

> Detail (Goal / Requirements / Success criteria / Context anchor) archived → [ARCHIVE.md § Phase 105](./ARCHIVE.md).

### Phase 106: Waived tutorials reproduce (docs-repro/tutorial-replay + examples 01/02/04/05)

**STATUS: CLOSED (GREEN 2026-07-12)** — 5 docs-repro rows (tutorial-replay +
example-01/02/04/05) minted PASS by an unbiased verifier at `804eedc`; STATE.md attests the
GREEN close at HEAD `7827d36` (CI run 29201112349 success, `code/ci-green-on-main` PASS).

> Detail (Goal / Requirements / Success criteria / Context anchor) archived → [ARCHIVE.md § Phase 106](./ARCHIVE.md).

### Phase 107: RUSTSEC memmap2 + quinn-proto advisories in `Cargo.lock`

**STATUS: CLOSED (GREEN 2026-07-12)** — unbiased phase-close verdict GREEN (4/4 deliverables
PASS), `.planning/milestones/v0.14.0-phases/evidence/P107-VERIFICATION.md` (committed
`24bb079`); ground-truth cargo-audit evidence at `7cfd165` (0 live vulns, both advisory IDs
cleared by version floor). (STATE.md's earlier "P107 next / not yet started" narrative was
stale — superseded by this reconciliation.)

> Detail (Goal / Requirements / Success criteria / Context anchor) archived → [ARCHIVE.md § Phase 107](./ARCHIVE.md).

### Phase 108: `prune_oid_map` pagination-truncation data-loss hazard

**STATUS: CLOSED (paperwork) — implementation SHIPPED pre-P108; P108 closed the paperwork.**
Fork A (completeness-gated prune) landed BEFORE this phase executed: `Listing { records,
is_complete }` + `list_records_complete()` in `crates/reposix-core/src/backend.rs`, BOTH
prune sites gated in `crates/reposix-cache/src/builder.rs` (delta `if is_complete` ~L166,
full-rebuild `if all_is_complete` ~L502), GitHub/JIRA/Confluence emit `is_complete=false`
on truncation, and the capped-mock regression `crates/reposix-cache/tests/
pagination_prune_safety.rs` (3 tests) is GREEN (re-verified 2026-07-11 via `cargo test -p
reposix-cache --test pagination_prune_safety`). Catalog row
`agent-ux/p94-pagination-prune-completeness-gate` is PASS with a truthful `owner_hint`
(the stale "NOT IMPLEMENTED" hint was corrected here). The SEPARATE, design-level
**slug→id / interrupted-create duplicate** waiver (ADR-010 §3) is NOT part of this fix and
was FILED as a remainder in `GOOD-TO-HAVES-09` — it remains OPEN / unstarted (the v0.14.0
reconciliation-redesign headline pivot), explicitly not cleared by P108.
**DEFERRED-TO-v0.15.0 (owner scope call, 2026-07-12):** GTH-09 / ADR-010 slug→id
durable-create is explicitly deferred past v0.14.0 milestone-close — NOT a silent slip.
Tracked at `.planning/milestones/v0.14.0-phases/GOOD-TO-HAVES.md` GOOD-TO-HAVES-09 (STATUS:
DEFERRED-TO-v0.15.0) and mirrored at root `.planning/GOOD-TO-HAVES.md` GOOD-TO-HAVES-09.

> Detail (Goal / Requirements / Success criteria / Context anchor) archived → [ARCHIVE.md § Phase 108](./ARCHIVE.md).

### Phase 109: Carried RBF-FW-11 + quality-convergence write-contention

**STATUS: CLOSED (GREEN 2026-07-12)** — RBF-FW-11 grandfather-commit rule shipped (`10bd508`)
atop the catalog-first GREEN contract (`1cb9dd1`, `pytest quality/runners/test_audit_field.py`
GREEN); STATE.md attests P109 shipped GREEN. Terminal evidence is the shipped fix + catalog-first
test + STATE attestation (no separate `p109/VERDICT.md` dir was minted).

> Detail (Goal / Requirements / Success criteria / Context anchor) archived → [ARCHIVE.md § Phase 109](./ARCHIVE.md).

### Phase 110: +2 reservation Slot 1 — surprises absorption (OP-8)

> Detail (Goal / Requirements / Success criteria / Context anchor) archived → [ARCHIVE.md § Phase 110](./ARCHIVE.md).

### Phase 111: +2 reservation Slot 2 — good-to-haves polish + milestone close (OP-9)

> Detail (Goal / Requirements / Success criteria / Context anchor) archived → [ARCHIVE.md § Phase 111](./ARCHIVE.md).

### Phase 112: OD-4 launch-readiness — SCOPE-BUT-DO-NOT-START stub

**STATUS: COMPLETE — scope stub landed 2026-07-12** at
`.planning/milestones/v0.14.0-phases/112-od-4-launch-readiness-scope-stub/PLAN.md`. The stub
names the four OD-4 pillars (asciinema hero demo, CI-verified honest headline numbers,
install-path excellence, Show-HN positioning kit) with a one-line description each, carries a
bold DO-NOT-START banner, and defers execution to a post-tag `/gsd-new-milestone` session. Zero
implementation, zero phase decomposition, no verifier-subagent dispatch (lightweight owner ack
suffices per success criteria below).

**Goal:** Produce a scoped placeholder ONLY for the OD-4 launch-readiness milestone
(asciinema hero demo, honest headline numbers, install-path excellence, Show-HN
positioning kit). This phase authors a stub roadmap/scope pointer — **NOT execution
detail** — and marks it clearly DO-NOT-START until wave-2 hardening (P102–P111) closes
GREEN. No implementation, no asciinema recording, no copy drafting happens in this phase.

**Requirements:** D2-OD4-STUB-01 · **Depends on:** none for scoping (this phase can be
authored any time), but **execution against it is blocked until P111 GREEN** · **Plan:**
N/A — this phase produces a scope stub, not an execution plan.

**Success criteria:**
1. A stub file exists (suggested: `.planning/milestones/v0.15.0-launch-readiness-phases/
   ROADMAP.md` or equivalent, named per the OD-4 mandate — exact milestone version number
   TBD at scoping time, do not guess it here) containing ONLY: the four OD-4 deliverable
   names (asciinema demo, honest headline numbers, install excellence, Show-HN kit), a
   one-line description each, and a bold "DO NOT START until wave-2 hardening (v0.14.0,
   P102–P111) closes GREEN" banner.
2. No phase decomposition, no REQ-IDs, no success criteria, no execution detail — those
   are authored later by a dedicated OD-4 scoping session, not by this stub.
3. `.planning/STATE.md` records the OD-4 stub's existence and its DO-NOT-START gate in the
   cursor narrative.
4. Phase close: `git push origin main`; this phase does NOT require a verifier-subagent
   catalog-row dispatch (it produces planning prose only, no code/gates) — a lightweight
   human/owner acknowledgment that the stub correctly says "do not start" suffices.

**Context anchor:** `.planning/STATE.md` OD-4 decision reference (`89-OWNER-DECISIONS.md`
§ "DECISION OD-4"); the user's owner-settled scope for this milestone (asciinema demo,
honest headline numbers, install excellence, Show-HN kit — verbatim from the OD-4
mandate).

### Phase 113: Lost-update shared-cursor guard (OUT-OF-BAND — shipped in v0.14.0)

**STATUS: CLOSED — code shipped in v0.14.0; verifier-close folded into P111 milestone-close
(regression test GREEN).**

> Detail (renumber rationale / what shipped / regression test / catalog row) archived → [ARCHIVE.md § Phase 113](./ARCHIVE.md).

