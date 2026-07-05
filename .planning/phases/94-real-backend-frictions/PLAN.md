---
phase: 94-real-backend-frictions
plan: overview
type: execute
wave: 1
depends_on: [89, 90, 91, 92, 93]
requirements: []  # no LIVE v0.13.0 REQ-IDs; the ROADMAP RBF-C-* IDs are STALE/orphaned — see § scope-correction
autonomous: true
---

<objective>
Close the two HIGH real-backend carry-forwards from P92/P93 plus the two recurring P2
debt items that the milestone-close sweep must clear before the v0.13.0 tag. P94's TRUE
scope is **real-backend frictions** (STATE.md frontmatter, the LIVE cursor), NOT the
"Bus-push compatibility / Cluster D / RBF-C" prose still sitting in
`.planning/milestones/v0.13.0-phases/ROADMAP.md` § "Phase 94" — that prose is STALE and
orphaned (bus-push compat shipped P82–P86). See § scope-correction; a noticing item is
filed to route the ROADMAP reconciliation to P96/P97.

This plan is a **catalog-first SPEC lane**: it MINTS the four GREEN-contract catalog rows
(D1–D4) that predate any implementation, and specs each fix for a downstream code
executor. It writes NO Rust — the pagination fix, the fallback-sentinel fix, the badge
determination, and the sweep are executed by later P94 code/verify lanes citing these
row ids.
</objective>

<scope-correction>
The ROADMAP.md § "Phase 94: Bus-push compatibility with documented mirror setup (Cluster
D)" prose (RBF-C-01..07) is STALE relative to the LIVE STATE.md cursor. STATE.md
frontmatter (`next_phase: P94 … real-backend frictions (pagination-truncation E2-fork +
git-2.43 fallback-sentinel)`) is the source of truth for what P94 actually delivers. The
bus-push/mirror-setup work the ROADMAP prose describes already landed in P82–P86. Do NOT
plan against the ROADMAP prose. The stale-vs-live divergence is filed as a noticing item
(routed to the P96/P97 milestone-close docs reconciliation) — it is NOT re-planned here.
</scope-correction>

<deliverables>

## D1 — Pagination-prune fix (Fork A + Fork B, per the ratified consult)

**Ratified design (do NOT re-litigate):** `.planning/CONSULT-DECISIONS.md` §
"2026-07-05 [FABLE] pagination-truncation prune-safety fork" (commit 3478dcc). Fork A
(primary) + Fork B (defense-in-depth), NO revert of 272882c.

**Bug being closed** (SURPRISES-INTAKE.md ~L891-939, HIGH): `meta::prune_oid_map`
(272882c/e246e84) DELETEs `oid_map` rows whose `issue_id` is absent from a `keep_ids` set
built from `self.backend.list_records(&self.project)`. GitHub/JIRA/Confluence connectors
can silently return a **truncated** `Ok(partial_list)` at a pagination/size cap
(`github/lib.rs` `MAX_ISSUES_PER_LIST=500` / `MAX_RAW_ITEMS_PER_LIST`; `jira/lib.rs`
non-strict `list_issues_impl`; Confluence equivalent). Feeding a truncated `keep_ids` set
into the DELETE wipes `oid_map` rows for **live records beyond the cap** — a real record
looks ghost/deleted, recurring on EVERY sync. The sim never truncates, so every sim-run
gate (incl. all of P93's GREEN runs) is structurally blind to it.

**Fork A (primary) — completeness signal + gated prune.** Add a completeness signal to
`BackendConnector::list_records`'s return (e.g. `Listing { records, is_complete }`, or a
sibling `list_records_complete()` if the owner vetoes the E2 return-type change). Gate
BOTH `prune_oid_map` call sites on `is_complete == true`:
- `crates/reposix-cache/src/builder.rs:138-139` (full-rebuild path)
- `crates/reposix-cache/src/builder.rs:442` (delta path)
On `is_complete == false` the prune is SKIPPED — falling back to the pre-272882c accepted
HARD-02 posture (under-populated tree, rows retained; NO data loss). Completeness is
already known-but-dropped at every truncation exit (`github/lib.rs:498-499` early-return
at cap, `:502-510` raw valve; JIRA/Confluence likewise) — the flag is additive plumbing,
not new pagination logic. The sim MUST return `is_complete == true` so existing sim gates
keep exercising the prune path.

**Fork B (defense-in-depth) — idempotent delete.** Reclassify delete-time `NotFound` as
idempotent success in `crates/reposix-remote/src/write_loop.rs` (per the already-filed
GOOD-TO-HAVE). Bounds the residual blast (prune-skipped-on-truncation ghost rows firing
phantom SoT DELETEs) to audit noise instead of hard errors.

**GREEN definition (D1):** A capped-mock `BackendConnector` (returns `is_complete=false`
with records truncated at a cap) drives `Cache::sync` in a NEW regression test; after
sync, `oid_map` rows for records BEYOND the cap are NOT deleted (prune-skip branch
pinned). The sim's `is_complete=true` still fires the prune (regression: complete
listings still prune). Fork B: a delete against an already-absent record returns success,
not error. 272882c is NOT reverted. Catalog row: `agent-ux/p94-pagination-prune-completeness-gate`.

## D2 — git-2.43 fallback-sentinel (HIGH carry-forward)

**From** SURPRISES-INTAKE.md L602-610 (HIGH). Stock Ubuntu 24.04 git (2.43.0) fails EVERY
real single-backend `git push` outright (exit 128, no helper output). Root cause: git's
transport-helper tries `stateless-connect git-receive-pack` FIRST for the push direction;
`crates/reposix-remote/src/stateless_connect.rs`'s `handle_stateless_connect`
(`service != "git-upload-pack"` branch) replies with a custom `"unsupported service:
{service}"` line instead of the `git-remote-helpers(7)`-mandated `fallback` sentinel. Per
that spec the three valid replies are: empty line (proceed), `fallback` (retry via
another capability), or exit-with-error (don't fall back) — reposix picks the THIRD, so
git never tries the `export` capability the push needs. Version-windowed: git 2.54.0 (CI
runner) does NOT hit it; the `>= 2.34` floor does NOT protect (2.43 > 2.34 yet breaks).

**Sub-spec (fix, per intake):** change `handle_stateless_connect`'s non-upload-pack branch
to reply with the literal `fallback` line when `service != "git-upload-pack"`. Update
`crates/reposix-remote/tests/stateless_connect_e2e.rs::stateless_connect_rejects_non_upload_pack_service`
— it currently asserts the bug-preserving `"unsupported service: ..."` reply; the
assertion FLIPS to `fallback` alongside the production fix. Add a regression pinned to the
git-2.43 window (container repro per `92-T4-REPRO-NOTES.md`) proving a real single-backend
push succeeds on that version.

**GREEN definition (D2):** (a) `handle_stateless_connect` replies `fallback` for
`service != "git-upload-pack"`; (b) the e2e test's asserted reply shape flipped to
`fallback`; (c) a git-2.43 container repro drives a real single-backend `git push` to exit
0 (push succeeds via the `export` fallback). Catalog row:
`agent-ux/p94-git243-fallback-sentinel`.

## D3 — badges real-vs-transient determination (recurring P2)

**From** GOOD-TO-HAVES.md L487-508 (MEDIUM/P2). `badges-resolve` (`docs-build` dimension)
FAILs on pre-push; unconfirmed whether a shields.io/Codecov transient flake or a genuinely
broken URL. Acceptance (verbatim): re-run `badges-resolve` in isolation on ≥2 spaced
occasions to distinguish real vs transient. If transient: add retry/backoff before failing
OR a documented waiver note. If real: fix the badge URL/config.

**Sub-spec:** re-run `python3 quality/gates/docs-build/badges-resolve.py` ≥2× spaced;
record the pass/fail pattern. Determination → EITHER (real) fix the offending URL in
`README.md`/`docs/index.md` so `docs-build/badges-resolve` returns PASS, OR (transient)
add retry/backoff to `badges-resolve.py` (or a documented waiver note) so it reaches
brightgreen reliably. Close the GOOD-TO-HAVES entry with the finding.

**GREEN definition (D3):** the determination is made + recorded; `python3
quality/gates/docs-build/badges-resolve.py` exits 0; the GOOD-TO-HAVES `badges-resolve`
entry is resolved with the real-vs-transient verdict. Catalog row:
`docs-build/p94-badges-real-vs-transient`.

## D4 — catalog-freshness sweep before milestone-close

**Milestone-close readiness.** Re-grade every stale (TTL-expired or session-stale)
NON-P93 catalog row across all dimensions; confirm no NEW code regression hides behind a
stale NOT-VERIFIED/FAIL.

**Pre-established fact to bake in (do NOT re-triage as a bug):** the
`agent-ux/p92-mid-stream-litmus-t1-t4` FAIL is the EXPECTED git<2.34 environment gate
(local exit 75 on this box's git 2.25.1; CI run on git 2.54.0 PASSED). It is an
accounted-for env-gate, NOT a code regression. Any similar exit-75 env-gate FAIL is
likewise accounted-for.

**GREEN definition (D4):** every stale non-P93 row is re-graded; every resulting FAIL is
an accounted-for env-gate (named + justified, e.g. the p92-mid-stream git<2.34 exit 75),
and NONE is a genuine code regression. Catalog row: `structure/p94-catalog-freshness-sweep`.

</deliverables>

<wave-breakdown>

**Wave 0 — catalog-first mint (THIS lane, no cargo).** Author this PLAN + mint the four
GREEN-contract rows (D1–D4) + file the two noticing items. Commit BEFORE any
implementation so the phase-close verifier reads rows that predate the fix (catalog-first
rule, `quality/CLAUDE.md`).

**Wave 1 — D1 pagination fix (code, cargo).** Implement Fork A (completeness signal +
gated prune at both `builder.rs` call sites) + Fork B (idempotent delete in
`write_loop.rs`) + the capped-mock regression test. One-cargo-invocation budget. Flip
`agent-ux/p94-pagination-prune-completeness-gate` GREEN.

**Wave 1 — D2 fallback-sentinel (code, cargo + container).** Flip the
`handle_stateless_connect` reply to `fallback`; flip the e2e assertion; add the git-2.43
container repro. Flip `agent-ux/p94-git243-fallback-sentinel` GREEN (container arm) /
NOT-VERIFIED (docker absent). Can run in the same wave as D1 (disjoint files) but MUST
serialize the cargo invocation.

**Wave 2 — D3 badge determination (docs-build, no cargo).** Spaced re-runs +
fix-or-waive. Can overlap Wave 1 (no cargo). Flip `docs-build/p94-badges-real-vs-transient`
GREEN.

**Wave 3 — D4 freshness sweep (meta, no cargo but reads runner output).** Runs LAST,
after D1–D3 land, as the milestone-close readiness gate. Flip
`structure/p94-catalog-freshness-sweep` GREEN.

**Phase close.** `git push origin main` BEFORE the verifier subagent (push cadence,
CLAUDE.md § GSD workflow). Unbiased verifier grades D1–D4 rows from committed artifacts;
verdict at `quality/reports/verdicts/p94/VERDICT.md`. Mid-stream litmus (if the ROADMAP
Decision-1 checkpoint applies) re-runs dark-factory against sim; TokenWorld arm stays
NOT-VERIFIED when creds absent (env-gated, never skip-as-pass).
</wave-breakdown>

<success_criteria>
1. D1: `agent-ux/p94-pagination-prune-completeness-gate` PASS — capped-mock regression
   proves live rows beyond the cap survive a truncated-listing sync (prune-skip branch
   pinned); sim listings still prune; Fork B idempotent-delete holds; 272882c not reverted.
2. D2: `agent-ux/p94-git243-fallback-sentinel` PASS (or NOT-VERIFIED when docker absent,
   fail-closed) — `fallback` reply on non-upload-pack service; e2e assertion flipped;
   git-2.43 container push exits 0.
3. D3: `docs-build/p94-badges-real-vs-transient` PASS — real-vs-transient determined +
   recorded; `badges-resolve.py` exits 0; GOOD-TO-HAVES entry resolved.
4. D4: `structure/p94-catalog-freshness-sweep` PASS — every stale non-P93 row re-graded;
   every FAIL is an accounted-for env-gate (p92-mid-stream git<2.34 exit 75 named), none a
   code regression.
5. Catalog-first: D1–D4 rows minted (NOT-VERIFIED) BEFORE any implementation commit; each
   carries a verifier expectation an unbiased grader can execute.
6. Phase close: `git push origin main`; verifier subagent grades GREEN; verdict at
   `quality/reports/verdicts/p94/VERDICT.md`.
</success_criteria>

<verdict_location>
quality/reports/verdicts/p94/VERDICT.md
</verdict_location>

<context>
@.planning/STATE.md (LIVE cursor — P94 = real-backend frictions)
@.planning/CONSULT-DECISIONS.md (2026-07-05 [FABLE] pagination-truncation prune-safety fork — RATIFIED design for D1)
@.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md (L891-939 pagination-truncation; L602-610 git-2.43 fallback-sentinel)
@.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md (L487-508 badges-resolve real-vs-transient)
@.planning/phases/93-cache-coherence/93-RELIEF-HANDOFF.md (§4-6 pagination data-loss analysis)
</context>

<minted-rows>
Catalog GREEN-contract rows minted by this lane (catalog-first, all NOT-VERIFIED at mint):
- D1 → `agent-ux/p94-pagination-prune-completeness-gate` (quality/catalogs/agent-ux.json)
- D2 → `agent-ux/p94-git243-fallback-sentinel` (quality/catalogs/agent-ux.json)
- D3 → `docs-build/p94-badges-real-vs-transient` (quality/catalogs/docs-build.json)
- D4 → `structure/p94-catalog-freshness-sweep` (quality/catalogs/freshness-invariants.json)
</minted-rows>
