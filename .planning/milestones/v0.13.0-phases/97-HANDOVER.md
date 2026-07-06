# 97-HANDOVER.md — v0.13.0 real-backend convergence (owner pre-tag action #1), relief handover, 2026-07-06

Written by the outgoing tree-writer/relief-handover-writer, dispatched to freeze this
session's tree at a clean/committed/pushed boundary for a fresh, higher-delegation
restart. **This is relief, not a same-coordinator pause** — the successor is a new
coordinator identity. Scope: clearing **OWNER PRE-TAG ACTION #1** from
`.planning/STATE.md` § "Workstream A" (run/ratify the `pre-release-real-backend` 9th
probe) for the v0.13.0 tag. That attempt is **fail-closed HALTED, correctly** — the
9th probe found a real P0 correctness bug (E4, below), not an infra flake. **No
v0.13.0 tag was pushed. No irreversible action was taken.**

**PUSH STATUS: BLOCKED, not landed.** Everything is committed locally (tree clean),
but `git push origin main` failed the pre-push gate on 2 unrelated FAILs — **neither
introduced by this handover session's own work**; both are pre-existing conditions in
commits already on HEAD before I started. Full detail + exact messages + named fixes:
§1 "Push status" and §6 step 0. Do not `--no-verify` past them.

**Required reading order for the successor:**
1. This file in full (all 6 sections) — **start with §1's push-status block**, the
   tree is not yet at origin.
2. `.planning/debug/p93-partial-fail-recovery-real-confluence.md` — the committed
   `/gsd-debug` root-cause artifact (E4 finding), referenced but not fully repeated
   below.
3. `docs/decisions/010-l2-l3-cache-coherence.md` (ADR-010, Status: ACCEPTED) § 3
   (RBF-LR-03 convergence contract) — the claim this bug falsifies for CREATEs.
4. `.planning/STATE.md` § "Workstream A" (OWNER PRE-TAG ACTIONS list) — **stale**,
   does not yet reflect this session's 9th-probe attempt or the E4 finding; the
   successor should update it once the finding is formally filed (§6 below).
5. (Background only, not required to act on) `.planning/phases/93-cache-coherence/93-RELIEF-HANDOFF.md`
   and `.planning/milestones/v0.13.0-phases/COORD-HANDOFF-P95-P97.md` — prior
   relief/succession handoffs for the P93/P95-P97 lineage this work continues. Both
   are fully resolved/superseded; read only for archaeology, not action items.

**Do-not-touch guardrails:**
- Do **not** push a `v0.13.0` git tag or run `tag-v0.13.0.sh.disabled` — L0/owner's
  action only, gated on ALL owner pre-tag actions clearing, which they have not.
- Do **not** self-resolve E4 (§5 item B.1) — it requires an owner-level ADR-010
  design decision (where does create-identity reconciliation live, given
  cursor/oid_map are deliberately not advanced on `SotPartialFail`). Package and
  escalate; do not pick a fix and ship it unilaterally.
- Do **not** trust `quality/catalogs/agent-ux.json` row `status` fields at face
  value without cross-checking the transcript/verification-artifact timestamp —
  two rows below are PASS-by-transcript-evidence but still show a stale
  `NOT-VERIFIED` in the catalog (not yet re-bound). See §4.
- Do **not** re-run any TokenWorld-mutating gate without checking §5(C)/§6 step 5
  cleanup status first (residual pages may exist from prior credentialed runs).
- Do **not** assume `git push` will succeed on retry without addressing §1's 2
  blocking gates first — retrying as-is reproduces the same 2 FAILs.

---

## 1. Ground truth (git)

Verified directly this session (not taken on faith from the dispatch briefing, which
had 2 factual errors — see §5's "corrections" list):

- **HEAD:** `c71af44` (tree clean, `git status --porcelain` empty).
- **origin/main:** `e8362f5` ("docs(state): advance STATE.md to v0.13.0 milestone
  CLOSED GREEN (20/20, P78–P97)") — **unchanged this session; the push did not land.**
- **Ahead/behind:** local is **5 ahead, 0 behind** origin/main.
- **Push status: BLOCKED.** `git push origin main` was attempted and rejected by the
  local pre-push hook (`.githooks/pre-push` → `quality/runners/run.py --cadence
  pre-push`), exit 1, **before any network push occurred** (nothing partially
  landed on origin). Two catalog rows FAILed out of 61 in-scope rows checked;
  everything else PASSED or WAIVED:
  - **`docs-alignment/walk` (P0, structure dimension).** Exact stderr:
    `docs-alignment: STALE_DOCS_DRIFT row=docs/index/ci-badge sources_drifted=[0]
    on docs/index.md -- run /reposix-quality-refresh docs/index.md` and the same
    for `row=docs/index/quality-weekly-badge`. **Root cause:** commits `8925311`
    and `f2bbf7f` (both already on HEAD before this session started, not
    authored by me) edited `docs/index.md`'s two CI/Quality-weekly badge URLs
    without re-binding the 2 doc-alignment claims that cite them. I confirmed
    this independently by running the compiled walker directly
    (`bash quality/gates/docs-alignment/walk.sh`) — it recomputes hash drift
    fresh against current `docs/index.md`, not just reading the catalog, and
    reports the identical 2 STALE_DOCS_DRIFT rows. **Fix named by the gate
    itself:** `/reposix-quality-refresh docs/index.md` (an orchestration-shaped,
    top-level-only flow per `.planning/CLAUDE.md` — NOT something I should run
    as a leaf tree-writer; it needs content re-verification + a `reposix-quality`
    rebuild + re-bind, out of a "no cargo needed" tree-hygiene scope).
  - **`docs-build/p94-badges-real-vs-transient` (P2).** Exact stderr:
    `FAIL (docs-build/p94-badges-real-vs-transient): GOOD-TO-HAVES badges-resolve
    entry is not RESOLVED (still OPEN or missing)`. **Root cause (verified, but
    NOT introduced by this session):** the verifier's Python check
    (`quality/gates/docs-build/p94-badges-real-vs-transient.sh`) greps the LIVE
    `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` for a full `## ...
    badges-resolve\` FAILs on pre-push...` heading + body, but that full entry
    was relocated to `GOOD-TO-HAVES-ARCHIVE.md` during the OP-8 Slot-2 drain
    (commits `d10534e`/`302e8ec`/`eba171a`) — only a one-line RESOLVED pointer
    remains in the live file, which the regex does not match. **I confirmed via
    `git show e8362f5:.../GOOD-TO-HAVES.md`** that this archived-away state
    ALREADY existed at the current `origin/main` tip — i.e. this is a
    **pre-existing latent bug**, not something my session's commits caused, and
    I do not know how the push that landed `e8362f5` got past it (possibly this
    exact row wasn't in `pre-push` cadence scope at that time, or a prior push
    used a different mechanism — I did not chase this further; flagging it as a
    genuine open question rather than guessing). **No fix is named by the gate
    itself** beyond its own assert message; the two candidate fixes are (a)
    teach the verifier to also scan `GOOD-TO-HAVES-ARCHIVE.md` for the RESOLVED
    entry, or (b) leave a matchable stub of the resolved entry in the live file.
    I did not apply either — modifying a shared quality-gate script that
    everyone's pre-push depends on is outside a tree-hygiene relief's mandate,
    and this task's own dispatch instructions said "report... do not force past
    it," not "fix quality gates you find broken."
  - **I did not use `--no-verify`.** Everything is committed locally; only the
    `push` step itself is outstanding. Re-attempting `git push origin main`
    as-is will reproduce the identical 2 FAILs until one of the two fixes above
    lands (see §6 step 0 for the exact unblock recipe).

**Per-commit one-liners, HEAD back to origin/main (newest first):**

| SHA | Message |
|---|---|
| `c71af44` | docs(handover): 97-HANDOVER.md — real-backend 9th-probe relief handoff |
| `2154151` | chore(quality): commit doc-alignment walk re-verdict after badge URL fixes |
| `232fea9` | docs(debug): E4 diagnosis — ADR-010 create-partial-fail cannot converge on id-reassigning backends |
| `f2bbf7f` | fix(docs): pin quality-weekly badge to main on homepage — match README |
| `8925311` | fix(docs): use modern CI badge URL on homepage — legacy /workflows/CI/badge.svg rendered stale 'failing' |
| `e8362f5` | (origin/main tip) docs(state): advance STATE.md to v0.13.0 milestone CLOSED GREEN (20/20, P78–P97) |

**Deviations the successor must know (ground-truth corrections to this task's
dispatch briefing):**
1. The briefing said HEAD was `8925311`, "one commit ahead of origin/main," and that
   the debug artifact was **untracked**. Ground truth at session start: HEAD was
   already `232fea9` (**3** ahead), and the debug artifact was **already committed**
   (as `232fea9`, by a prior debugger in this same session lineage, reflog shows
   `8925311` landed via `cherry-pick`). Nothing was lost; I did not need to
   re-commit the debug artifact.
2. The badge PNGs (`badges.png`, `reposix-home.png`) named in the briefing as
   "stray verification screenshots" **do not exist** at the repo root or anywhere
   else in the tree (checked `find . -maxdepth 2 -iname "*.png"` — only
   `.playwright-mcp/*.png`, which is gitignored and unrelated). `8925311`'s only
   change is `docs/index.md` (1 line, remote shields.io URL, no local image
   reference) — consistent with "expected" per the dispatch, so there was nothing
   to `rm` and nothing to `git add`.
3. One genuine uncommitted change I found and had to adjudicate myself (not
   mentioned in the dispatch briefing at all): `quality/catalogs/doc-alignment.json`
   had an unstaged diff — 2 badge-URL-verifier rows flipped `BOUND` →
   `STALE_DOCS_DRIFT` (legitimate walker re-grade after `8925311`/`f2bbf7f` edited
   `docs/index.md`; internally consistent with `structure/doc-alignment-summary-block-valid`'s
   ratio invariant). I committed it as-is (`2154151`) rather than discard it — see
   §5's "mid-execution decisions" for the reasoning. **This same drift is what
   later blocked the push** (§1) — discarding it would NOT have avoided the block
   (the walker recomputes fresh from `docs/index.md` regardless of the catalog
   file's committed state), it would only have hidden the finding.

## 2. Wave/cycle state

This isn't a numbered GSD phase (no `/gsd-execute-phase` was run) — it's the
milestone's **OWNER PRE-TAG ACTION #1** (`.planning/STATE.md` § Workstream A),
attempted this session:

| Step | Scope | State | Evidence |
|---|---|---|---|
| Run `pre-release-real-backend` cadence with real ATLASSIAN creds | Clear owner pre-tag action #1 | **HALTED — fail-closed, correctly** | `.planning/debug/p93-partial-fail-recovery-real-confluence.md` |
| `agent-ux/milestone-close-vision-litmus-real-backend` | Vision litmus against live TokenWorld/REPOSIX space | **PASS by transcript** (`quality/reports/transcripts/milestone-close-vision-litmus-real-backend-2026-07-06T06-28-00Z.txt`, exit_code 0) but catalog row still reads `NOT-VERIFIED` (`last_verified: 2026-07-06T05:03:59Z`, an EARLIER, pre-creds run) — **not yet re-bound** | transcript + catalog row cross-checked directly |
| `agent-ux/attach-sync-real-backend` | `attach_real_confluence` + `sync_real_confluence` against live backend | **PASS by transcript** (`quality/reports/transcripts/attach-sync-real-backend-2026-07-06T06-28-09Z.txt`, "2 passed; 0 failed") but catalog row **also still `NOT-VERIFIED`**, same stale `last_verified` — **not yet re-bound** | transcript + catalog row cross-checked directly |
| `dark_factory_real_{github,confluence,jira}` (3 `#[ignore]` tests, `agent_flow_real.rs`) | Referenced by the dispatch briefing as "3/3 PASS" | **NOT independently confirmed by me** — I found no fresh transcript/verification artifact for these 3 in `quality/reports/` this session; take the briefing's claim as **unverified**, re-run before relying on it | none found on disk |
| `agent-ux/p93-partial-failure-recovery-real-confluence` | Partial-fail-recovery smoke against live Confluence | **FAIL** (two stacked bugs — see below) | `quality/reports/verifications/agent-ux/p93-partial-failure-recovery-real-confluence.json` (`exit_code: 1`), debug artifact |
| `agent-ux/t4-conflict-rebase-ancestry-real-backend` | Sibling of the sim-arm T4 litmus, real-backend | **NOT IMPLEMENTED** — verifier script does not exist (`quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh` absent; sim sibling `t4-conflict-rebase-ancestry.sh` exists) | `ls` confirms absence; catalog row `NOT-VERIFIED`, `waiver: null` |
| Committed the `/gsd-debug` E4 root-cause note + `doc-alignment.json` walk re-verdict + this handover | Tree hygiene for relief | **DONE, but push BLOCKED (§1)** | `232fea9` (pre-existing), `2154151`, `c71af44` |

**No named-incident post-mortem** beyond the E4 finding itself (§5/debug artifact) —
the fail-closed halt is process working as designed, not an incident.

**Important nuance on the P93 real-Confluence FAIL** (read before re-running that
gate): the on-disk verification artifact
(`quality/reports/verifications/agent-ux/p93-partial-failure-recovery-real-confluence.json`,
`ts: 2026-07-06T05:15:47Z`) shows the failure came from the verifier script's
**sanctioned-space guard** (`quality/gates/agent-ux/p93-partial-failure-recovery-real-confluence.sh`
lines 47–54: `case "$SPACE" in TokenWorld) ;; *) FAIL ... esac`), which rejected
`REPOSIX_CONFLUENCE_SPACE=REPOSIX` (an alias/key for the SAME sanctioned TokenWorld
space per `docs/reference/testing-targets.md`) before the smoke test ever ran — NOT
a direct observation of the E4 bug through this gate. **The E4 bug itself was proven
separately**, via the debugger's direct REST calls to live TokenWorld bypassing this
gate entirely (documented in the debug artifact's Evidence section). Both bugs are
real and both block this gate; fixing the space-alias guard (§6 step 3) is a
prerequisite to actually exercising (and, once E4 is fixed, passing) the smoke
through the gate — fixing the guard alone will NOT make this row PASS, because E4
is still unfixed underneath it.

## 3. Binding constraints (unchanged)

- **One tree-writer at a time** — I was the only one this session; hold for the
  successor too.
- **One cargo invocation machine-wide** — I ran zero cargo invocations myself
  (git-only work, plus one direct `bash quality/gates/docs-alignment/walk.sh`
  invocation to diagnose the push block, which only runs a pre-built binary, no
  compile step). The pre-commit/pre-push hooks may invoke `cargo fmt --check`
  (only on staged `.rs` files — none here this session) — verify no other cargo
  process is running before the successor's first `cargo` command.
- **No `--no-verify`** — not used anywhere, including on the blocked push. Both
  commits went through `.githooks/pre-commit` cleanly (exit 0); the push attempt
  went through `.githooks/pre-push` and was correctly rejected (exit 1) — I did
  not override it.
- **Push only at green — push is currently NOT green** (§1). Do not push again
  until at least one of the two named fixes lands; do not `--no-verify` around it.
- **Commit trailer format** — `Co-Authored-By: Claude Opus 4.8 (1M context)
  <noreply@anthropic.com>` used on all 3 commits this session.
- **Model tiering** — unchanged; this was a leaf tree-hygiene task, not a
  coordinator dispatch.

## 4. Litmus / gate / REOPEN state

- **Pre-push gate (this session, blocking the handoff commits from reaching
  origin) — 2 FAILs, both pre-existing, neither introduced by this session.**
  Full detail in §1. Fix path: `/reposix-quality-refresh docs/index.md` for
  `docs-alignment/walk`; either scan-the-archive or restore-a-stub for
  `docs-build/p94-badges-real-vs-transient`. Neither fix belongs to a
  tree-hygiene leaf — both need a coordinator-level dispatch.
- **9th probe (`pre-release-real-backend`) — separately, not clean either.** Two
  of five real-backend arms independently confirmed PASS-by-transcript this
  session (litmus, attach-sync — see §2) but their catalog rows are **stale**
  (still `NOT-VERIFIED` from an earlier pre-creds run at `2026-07-06T05:03:59Z`)
  — re-bind, don't just re-run, once the successor is ready to close this out.
  One arm (`dark_factory_real_*`) is claimed-but-unconfirmed (§2). One arm
  (`p93-partial-failure-recovery-real-confluence`) is a confirmed **FAIL** for
  two stacked reasons (space-alias guard bug + underlying E4 correctness bug —
  see §2's nuance paragraph and §6 step 2/3). One arm
  (`t4-conflict-rebase-ancestry-real-backend`) is **NOT IMPLEMENTED** (no
  verifier script exists at all).
- **No open waiver-expiry clocks specific to this work.** The three P93
  cache-coherence rows waived at `543bfb4` (`p93-l2-l3-coherence-adr`,
  `p93-cache-coherence-refresh-honest`, `p93-delta-sync-coherence-invariant`) have
  **already flipped `WAIVED` → `PASS`** (confirmed via `jq` against
  `quality/catalogs/agent-ux.json` this session) — they are NOT part of this
  blocker. **Correction to the dispatch briefing:** it claimed "the p93 shell
  verifier row is also waived in the catalog per prior commit 543bfb4 — the next
  session must un-waive it." This is **inaccurate** — `543bfb4` never touched
  `agent-ux/p93-partial-failure-recovery-real-confluence` at all; that row's
  `waiver` field has been `null` since it was minted (`2026-07-05T10:30:00Z`) and
  still is. There is nothing to un-waive on that row — it needs a **fix**, not a
  waiver removal.
- **REOPEN state:** none active from prior phases (P93's HIGH #5 REOPEN was
  resolved and closed before milestone-close per `93-RELIEF-HANDOFF.md` lineage —
  confirmed via `p93-l2-l3-coherence-adr`/etc. now PASS). This session's E4 finding
  is a **NEW** halt, not a reopen of a prior one.
- Structure-dimension waiver `structure/file-size-limits` remains active until
  **2026-08-08** (unrelated to this work; noted for awareness — it fired
  informationally during this session's pre-commit hook runs, exit 0 both times,
  no action needed yet). Note also: this session's handover commit (`c71af44`)
  tripped a size WARNING (21439 chars vs the 20000 soft budget) under this same
  waiver's "warn-now, block-later" contract — non-blocking today, but if the
  successor extends this file further, consider splitting per CLAUDE.md's
  progressive-disclosure guidance rather than letting it keep growing.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

**Decisions I made and record here (none required an owner escalation; all
reversible/low-risk):**

- **Committed the `doc-alignment.json` walker diff as-is** rather than discarding
  it or leaving it uncommitted. Reasoning: `quality/gates/docs-alignment/README.md`
  says subagents never *hand-write* this file, but this diff was tool-generated
  (walker re-grade), already present when I started, and internally consistent with
  the `structure/doc-alignment-summary-block-valid` ratio invariant
  (`259/(393-57) = 0.7708333...`, matches). Discarding it would have silently
  reverted a true finding back to a falsely-optimistic `BOUND` **and would not have
  avoided the push block anyway** (§1) — the walker recomputes fresh from
  `docs/index.md`, independent of what the catalog file says. Filed as its own
  small commit (`2154151`) rather than folded into the handover commit, for a
  clean diff.
- **Chose `.planning/milestones/v0.13.0-phases/97-HANDOVER.md` as this file's
  path**, not a new `.planning/phases/9N-*/` directory. This work has no GSD phase
  number of its own (P95–97 already share this pattern — see
  `COORD-HANDOFF-P95-P97.md` in the same directory, and note P95/96/97 never got
  individual `.planning/phases/9N-*/` dirs at all). Creating a new numbered phase
  directory would misrepresent unplanned owner-pre-tag-action work as a formal GSD
  phase; this directory is the established, already-precedented home for
  cross-cutting P95-97-lineage coordination artifacts.
- **Did not attempt to fix either pre-push blocker myself** (§1). Both require
  either an orchestration-shaped top-level flow (`/reposix-quality-refresh`,
  needs a `reposix-quality` rebuild + content re-verification) or a change to a
  shared quality-gate script whose correctness every future push depends on —
  both outside a tree-hygiene relief's mandate, and this task's explicit
  contingency instruction was "report... do not force past it."

**Noticed, not yet filed (owned by the next session per the ownership charter — "an
empty noticing section is a red flag"):**

1. **[HIGH, needs SURPRISES-INTAKE filing]** The E4 finding itself is not yet
   recorded in `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` as a
   formal entry (BLOCKER-severity, per that file's entry format). I deliberately
   did **not** hand-edit that 745-line planning doc myself — `.planning/CLAUDE.md`
   requires planning artifacts be edited only via a GSD command, and I am a
   tree-hygiene/handover leaf, not a phase coordinator. The successor's first
   GSD-tracked action should file this entry (severity: BLOCKER, discovered-by:
   post-P97 owner-pre-tag-action, sketched resolution: pointer to ADR-010 revision
   need) before doing anything else that touches that file.
2. **[HIGH, blocks this session's own push]** The `docs-build/p94-badges-real-vs-transient`
   verifier reads a GOOD-TO-HAVES entry that the OP-8 Slot-2 archive-relocation
   (`d10534e`/`302e8ec`/`eba171a`) already moved out of the live file **before**
   `e8362f5` (the current origin/main tip) — i.e. this gate has been silently
   broken since before this session started, and I could not determine how the
   push that landed `e8362f5` got past it. Someone with more context on the
   runner's pre-push row-selection history should investigate (possible causes:
   the row's `cadences` list changed after `e8362f5`, or an intermediate push used
   a different mechanism) rather than assume my diagnosis of "always broken" is
   complete.
3. **[MEDIUM]** `.planning/STATE.md` § "Workstream A" § "OWNER PRE-TAG ACTIONS"
   does not mention this session's 9th-probe attempt or its outcome at all — it
   still reads as if action #1 is simply "not yet run." A fresh reader of
   STATE.md alone (without this handover) would not know a P0 bug was found. Not
   fixed by me (same reasoning as item 1 — STATE.md is GSD-tracked); flagged for
   the successor's first STATE.md touch.
4. **[LOW]** The `p93-partial-failure-recovery-real-confluence.json` verification
   artifact mixes a `skip_reason: "env-missing"` / `skipped_real_backend: true`
   marker with an `exit_code: 1` and a hard-FAIL stderr message in the same JSON —
   internally inconsistent framing (the script's own contract, per its header
   comment, is env-gate-unset → exit 75/never-1, non-sanctioned-target →
   exit 1/never-75; the recorded artifact shows exit 1, so the `skip_reason` field
   looks like a stale carry-over from a still-earlier run, not this one). Small
   inconsistency, not blocking, but will confuse a future reader of that artifact
   in isolation — worth a one-line note or fix in whatever wave rewrites this
   verifier per §6 step 3.
5. **[LOW]** Two real-backend catalog rows (`milestone-close-vision-litmus-real-backend`,
   `attach-sync-real-backend`) are PASS-by-transcript but stale-`NOT-VERIFIED`
   in the catalog (§2/§4) — needs a re-bind pass, not a re-run, once the successor
   is doing the final close (folds naturally into §6 step 6's re-run anyway, since
   that re-run will regenerate both).

## 6. Precise next steps (successor runbook)

0. **Unblock and land this session's push FIRST** (nothing else in this runbook
   matters until the tree is actually on origin). Two independent fixes, either
   is sufficient to get past its own gate (both are needed for a fully-clean run):
   - **`docs-alignment/walk`:** run the `/reposix-quality-refresh docs/index.md`
     flow (top-level orchestration-shaped, per `.planning/CLAUDE.md`) to
     re-verify the 2 badge URLs actually resolve/render and re-bind the
     `docs/index/ci-badge` + `docs/index/quality-weekly-badge` claims.
   - **`docs-build/p94-badges-real-vs-transient`:** dispatch a small fix (either
     teach `quality/gates/docs-build/p94-badges-real-vs-transient.sh` to also
     scan `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES-ARCHIVE.md` for the
     RESOLVED `badges-resolve` entry, or restore a matchable stub in the live
     `GOOD-TO-HAVES.md`) — then re-run `python3 quality/runners/run.py --cadence
     pre-push` locally to confirm exit 0 before retrying `git push origin main`.
   - After both land: `git push origin main`, then `git fetch origin main &&
     git rev-parse HEAD && git rev-parse origin/main` — confirm equal.
1. **File the E4 SURPRISES-INTAKE entry** (§5 noticed-item 1) via the proper GSD
   channel (`/gsd-quick` or the active coordinator's own intake-drain step) —
   BLOCKER severity, pointing at `.planning/debug/p93-partial-fail-recovery-real-confluence.md`
   and this file.
2. **Package E4 for an owner design decision** (do NOT self-resolve): the question
   is where a create-identity reconciliation map (placeholder id → real backend id)
   lives, given `cursor`/`oid_map` are deliberately not advanced on the
   `SotPartialFail` branch (`crates/reposix-remote/src/write_loop.rs` +
   `precheck.rs`). This is an ADR-010 revision (or superseding ADR), not an
   in-family code fix — see the debug artifact's "Resolution" section for the 3
   fixes already considered and rejected as band-aids.
3. **Mechanical, in parallel with #2:** fix the sanctioned-space alias guard in
   `quality/gates/agent-ux/p93-partial-failure-recovery-real-confluence.sh` lines
   47–54 to accept the alias set `{TokenWorld, REPOSIX, 360450}` (space name, key,
   and numeric id all resolve to the same sanctioned space per
   `docs/reference/testing-targets.md`), preserving the hard-FAIL behavior for any
   OTHER space. This unblocks the gate from even reaching the smoke — it will
   still hit E4 underneath until #2 lands; that's expected and correct, not a
   regression. (While in this file, also tidy the `skip_reason`/`exit_code`
   inconsistency from §5 noticed-item 4.)
4. **Implement `t4-conflict-rebase-ancestry-real-backend`** (currently
   `NOT-IMPLEMENTED`, no script exists). Recommended shape (per the dispatch
   briefing's sketch, ~1.5h, no adapter/schema change): a `#[ignore]` Rust smoke in
   `crates/reposix-cli/tests/agent_flow_real.rs` mirroring the sim sibling
   `quality/gates/agent-ux/t4-conflict-rebase-ancestry.sh` lines 182–193's core
   assertion (root commit of `refs/reposix/origin/main` byte-identical before/after
   a conflict→refetch cycle, the HIGH-1 no-disconnected-root guard, plus the
   commit-count>1 vacuous-guard) against live Confluence, plus a thin shell
   wrapper following the `attach-sync-real-backend.sh` env-gate/transcript
   pattern. **Run it** — it may surface a real bug, same as E4 did; that's the
   point of writing the gate, not a formality.
5. **TokenWorld cleanup — verify, don't assume.** The debug artifact asserts
   TokenWorld was left clean (0 p93 pages) after the direct-curl diagnostic. I did
   NOT independently re-verify this via a live REST call (out of scope for a
   tree-hygiene task; would itself be a TokenWorld mutation/read requiring the same
   care). Before the successor's next TokenWorld-mutating run: list pages in space
   `REPOSIX`/`TokenWorld` (id `360450`) labeled `kind=test` or titled `p93 smoke *`,
   confirm the debug artifact's "space clean" claim still holds (a later run could
   have re-populated it), and preserve the durable fixtures `7766017` and `7798785`
   (`docs/reference/testing-targets.md`). Log both audit tables on any teardown
   mutation (OP-3).
6. **Once #2 lands (owner decision + implementation) and #3+#4 are GREEN:**
   re-run `python3 quality/runners/run.py --cadence pre-release-real-backend` with
   `.env`'s ATLASSIAN vars actually **exported into the test-process environment**
   (source `.env` before invoking, not just having it present on disk — the
   05:03:59Z NOT-VERIFIED run in this session's history was an env-load gap, not a
   real creds-absent state). Confirm exit 0 / all 5 real-backend arms GREEN
   (re-binding the 2 stale-PASS rows from §4 as part of this same re-run).
7. **Un-waive nothing** — there is nothing waived on the P93 real-Confluence row
   to un-waive (§4 correction). Do not spend time looking for a waiver object that
   doesn't exist.
8. **Close ritual once #6 is clean:** dispatch an unbiased verifier subagent to
   grade the 5 real-backend catalog rows from committed artifacts only; on GREEN,
   `git push origin main`; update `.planning/STATE.md` § Workstream A to mark
   owner-pre-tag-action #1 CLEARED (with commit SHA + verdict reference); report
   "real-backend GREEN, v0.13.0 tag re-authorizable" back to L0/owner — **the tag
   push itself remains L0's/owner's action, never the coordinator's**, per OD-3.
