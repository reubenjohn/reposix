# P116 — ADR-010 Decision Packet (options + tradeoffs; NO implementation)

**For:** owner/manager ruling (decide-and-disclose → `.planning/CONSULT-DECISIONS.md`).
**Scope:** two ADR-010-touching decisions, co-located because both amend
`docs/decisions/010-l2-l3-cache-coherence.md`. This packet enumerates options only;
per the P116 charter (ROADMAP.md:110-120, REQUIREMENTS.md ADR-01/FIX-03) **no chosen
option is implemented pre-ruling**, and FIX-03's v0.15 build depth is set *by* the
ruling (SC4).
**Provenance:** drafted 2026-07-16 by an opus researcher under L0 workhorse #43;
committed + routed to the manager (w1:p7) for ruling by L0 workhorse #44, 2026-07-16.
**Status: AWAITING RULING** — no option below is implemented until the ruling lands in
`.planning/CONSULT-DECISIONS.md`.

Neither decision changes the SoT: the SoT is always correct on both paths. What is at
stake is (1) how the *external mirror* tracks the SoT, and (2) how a *create* survives a
mid-batch interruption on an id-assigning backend. Both stay inside the existing egress
allowlist — the mirror push and REST write are already gated on `REPOSIX_ALLOWED_ORIGINS`
(`docs/concepts/dvcs-topology.md:167`), and the slug→id map is local cache metadata, so
neither option widens the taint/egress surface.

---

## Decision 1 — ADR-01: mirror fan-out coherence (RBF-LR-04)

### What's actually open (grounded)

The bus push writes SoT-first, then fans out to the mirror best-effort
(`dvcs-topology.md:158-165`). Two distinct things are both called "mirror" and are
currently conflated in the docs:

- **(a) cache observability ref** `refs/mirrors/<sot>-head` — advanced by
  `refresh_for_mirror_head`, gated on `files_touched > 0`
  (`crates/reposix-remote/src/write_loop.rs:314-322`; a no-op push issues zero
  `list_records`, asserted as a feature in `reposix-remote/tests/perf_l1.rs:386-390`).
- **(b) external GitHub mirror repo** whose content a fresh `git clone` actually reads.

Two concrete defects were filed against the fan-out and **RULED-DEFER→v0.15.0** pending
this ruling:

1. **Fan-out pushes the PRE-write client tree, not the post-write materialized
   snapshot** (`v0.14.0 surprises-intake/part-03.md:286-311`). After any SoT-changing
   push the mirror trails the backend by the freshly-minted `version:` frontmatter int
   **and** a markdown↔storage re-normalization of the body — so the mirror is *never*
   left `== SoT`. This is why the milestone-close vision litmus is non-idempotent against
   its own mirror (green once, red on immediate re-run). Interim op-recovery:
   `scripts/refresh-tokenworld-mirror.sh` before each litmus run.
2. **The doc-truth prose is empirically wrong** (`part-02.md:301-329`, STATUS DEFERRED
   *"pending the B1 mirror-refresh manager decision"*). Root `CLAUDE.md` /
   `dvcs-topology.md:169-198` point operators at `reposix sync --reconcile` as the
   external-mirror catch-up, but `--reconcile` heals only the **local** cache
   (`oid_map`/cursor + the cache-internal `refs/mirrors/*` ref) — it does **not** touch
   external mirror repo content. The correct rewrite is blocked because it depends on the
   manager blessing *which* refresh path is authoritative for the external mirror.

ADR-010 §2 (Decision item 2) already ratified the RBF-LR-04 "keep + qualify" branch for
the observability ref but explicitly **left the fix-wave a lever**: `files_touched > 0`
gate vs. unconditional refresh (ADR-010 lines 207-219). The external-mirror content
question above was never settled.

### Options

| # | Option | Blast radius | Coherence w/ ADR-010 promise | Cost |
|---|---|---|---|---|
| A | **Doc-truth only.** Keep pre-write fan-out; rewrite the two conflated "mirrors" distinctly; state honestly "clone reads SoT-current, mirror lags until next SoT-changing push"; correct the `--reconcile` claim. | None (docs) | Fully coherent — restates "mirror best-effort by design" | ~hours |
| B | **A + name the authoritative external catch-up.** Bless webhook + 30-min cron (`dvcs-mirror-setup.md:9,126-143`) as the convergence mechanism and `scripts/refresh-tokenworld-mirror.sh` as the manual op-recovery; document mirror-lag as accepted. | None (docs + op) | Coherent; makes the "qualified promise" honest | ~hours |
| C | **Post-write snapshot fan-out (product fix).** After the SoT write, fan-out pushes the *materialized* backend-current tree so the mirror is left `== SoT`, making the litmus self-idempotent (no pre-step). | Mirror-push branch only; SoT path untouched | Coherent; upgrades best-effort→eager on the push path | Multi-step: fan-out must re-materialize post-write tree before push; adds work/latency to the push. **Still cannot cover out-of-band backend writes** (web-UI/PATCH) — those only converge via the webhook/cron. |
| D | **Unconditional observability-ref refresh** (drop the `files_touched > 0` gate). | Cache observability ref only | The RBF-LR-04 lever; does NOT touch external mirror content | Cheap, but regresses the asserted no-op perf skip (`perf_l1.rs:386-390`) for zero external-mirror benefit |

### Recommended default: **B** (A folded in), reject D, offer C as an elective upgrade

Rationale: the doc-truth correction (A) is mandatory regardless of ruling — it is
currently a *lying doc* pointing operators at a command that won't fix their symptom.
B then resolves the one thing the manager actually must bless: the external-mirror
refresh path. It leans on the webhook/cron convergence that already exists and is
coherent with the ratified "mirror is a read surface, best-effort, SoT never wrong"
contract (`dvcs-topology.md:200-206`). **C is the honest product fix for litmus
self-idempotency**, but it is ROI-negative as the *primary* remedy: it re-materializes
the tree on every push yet still can't converge out-of-band edits, so the webhook/cron
(and thus B's doc) is needed either way. D is rejected — negligible benefit, breaks a
green perf assertion. Elect C only if the manager wants the litmus green without the
documented pre-step.

**A ruling unblocks:** the DEFERRED doc-truth rewrite (part-02 row + `dvcs-topology.md`
twin), retires the litmus-non-idempotency intake (part-03 row), and closes the RBF-LR-04
lever left open in ADR-010 §2.

---

## Decision 2 — FIX-03 / GTH-09: slug→id durable-create

### What's actually open (grounded)

ADR-010 §3 carries a **WAIVED known limitation** (lines 238-279): a `create` against an
id-assigning real backend (GitHub Issues / JIRA / Confluence) cut off **mid-batch** can,
on retry, leave **one duplicate record**. The convergence contract ("already-landed
writes are diffed away against the recomputed base") holds for **updates** (stable ids)
but is false for **creates** — the agent-picks a placeholder id
(`write_loop.rs:251,274` — `PlannedAction::Create(issue)` carries `issue.id.0`), the
backend mints its *own* id, and that placeholder→backend-id mapping has **no durable
home**. So on retry, PRECHECK B (`write_loop.rs:159`) re-diffs against the new SoT but
cannot recognize the already-landed record as "the same create," and re-creates it.
Blast radius: real backend **+** create **+** mid-batch network drop; recoverable by
hand-deleting one duplicate; sim + client-id backends unaffected. Cache non-append-only
tables today are `oid_map` + `meta` (`crates/reposix-cache/src/meta.rs:1`) — a slug→id
map would join them.

The manager must scope **two** things (REQUIREMENTS.md FIX-03:56-69, SC4): (a) which
design, and (b) v0.15 **implementation depth** — design-only vs design+build.

### Options

| # | Option | Guarantee | Cost / blast radius |
|---|---|---|---|
| A | **Design-only this milestone.** Write the ADR-010 §3 amendment fixing a target design; defer the build. Matches P116's decision-only charter. | Waiver stays live (one hand-deletable duplicate) | Lowest; no code |
| B | **Durable slug→id map (the GTH-09 fix sketch).** Client mints a stable local slug pre-push; commit-sequence models "slug X → (pending) → backend id N", persisted alongside `oid_map`; an interrupted create leaves a pending-slug-no-confirmed-id state the retry reconciles against instead of re-creating. | Eliminates the duplicate | Multi-crate (core `diff::plan`, remote `write_loop`, new cache table + migration); this is the v0.14.0 reconciliation-redesign headline-scoped pivot |
| C | **Dedup-on-retry (lite).** Before a create retry, query the backend for an existing record matching the slug/title and adopt its id. | Weaker — relies on backend-searchable uniqueness; fuzzy title match; can't distinguish a legit duplicate title from a resumed create; per-backend search semantics differ | Cheap (a pre-create existence check, no schema) |
| D | **Durable pending-create intent-log.** Write a pending-create record (slug + payload hash) to the cache BEFORE the REST call; on retry, if an unconfirmed intent exists, resolve by listing recent backend records and matching the hash. | Strong for the interrupted-create case; middle ground | New cache state but no full commit-sequence redesign; still needs a backend-list reconcile step |

### Recommended default: **A this milestone, selecting B as the target design**

Rationale: P116 is explicitly decision-only, so the deliverable is a *chosen direction*,
not a build. B is the honest fix and the direction ADR-010 §3, the RETROSPECTIVE, and the
GTH-09 fix sketch all already point at. C is fragile against real-backend id/search
semantics (exactly the surface the waiver is about) and gives a weaker guarantee; D is a
credible cheaper middle ground worth naming if the manager wants the hazard closed *within*
v0.15 without the full commit-sequence redesign. Recommend: **record B as the sanctioned
design, keep implementation deferred, and have the ruling set the v0.15 depth (SC4)** — if
the manager elects to close the hazard this milestone, budget a dedicated design+build
phase for B (or D as the reduced-scope variant).

**A ruling unblocks:** revision of ADR-010 §3 (qualify/remove the waiver), retirement of
the GTH-09 `DEFERRED` status (root + v0.14.0 `GOOD-TO-HAVES.md:44`/`:232`), and the SC4
scoping of FIX-03's v0.15 implementation extent.

---

## Sources
ADR-010: `docs/decisions/010-l2-l3-cache-coherence.md` (§2 lines 207-219, §3 lines
238-279). Fan-out defects: `.planning/milestones/v0.14.0-phases/surprises-intake/part-03.md:279-322`,
`.../part-02.md:299-329`. Code: `crates/reposix-remote/src/write_loop.rs:159,251,274,314-322`,
`tests/perf_l1.rs:386-390`, `crates/reposix-cache/src/meta.rs:1`. Docs:
`docs/concepts/dvcs-topology.md:158-206`, `docs/guides/dvcs-mirror-setup.md:9,126-143`.
Requirements: `.planning/REQUIREMENTS.md` FIX-03:56-69 / ADR-01:151-156;
`.planning/milestones/v0.15.0-phases/ROADMAP.md` (and root `ROADMAP.md:110-120`). GTH-09:
`.planning/GOOD-TO-HAVES.md:44-78`, `.planning/milestones/v0.14.0-phases/GOOD-TO-HAVES.md:232-246`.
