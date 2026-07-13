# SESSION-HANDOVER.md — v0.14.0 tag: OWNER fix-first items 4-8, D2+B3 CLOSED GREEN — 2026-07-13 (→ successor #6)

For the incoming top-level workhorse (L0) — a top-level ROUTING coordinator: routes via GSD +
subagents, never leaf-works. Map, not territory — detail lives in git + linked files. HEAD =
live state only; delete closed/superseded entries rather than appending. The outer-loop MANAGER
(herdr pane w1:p7) watches this pane and relays owner decisions; `.planning/MANAGER-HANDOVER.md`
is the live owner-directive channel. Resume an agent via SendMessage, never fork.

**PROVENANCE WARNING:** the PRIOR on-disk copy of this file (commit `5a01b53`) was authored by
a ROGUE FORK — an L0-dispatched `fork` that inherited full context and became a live parallel
tree-writer (single-writer-discipline violation) instead of a safe no-op/discard. Its "#5/#6"
successor framing and "TAG STILL BLOCKED on ONE owner decision (B1)" framing are STALE/fictitious
as of this rewrite — **do not trust that file's content**; this version supersedes it entirely
from verified ground truth + the current L0's live directive. Fix-twice landed:
`.planning/ORCHESTRATION.md` §11 now carries "a `fork` is never a safe no-op or discard
placeholder" doctrine.

## 0. State (verify: `git rev-parse --short HEAD origin/main`, `git status --porcelain`, `gh run list --branch main --limit 3 --json databaseId,headSha,status,conclusion,createdAt`)
- HEAD = origin/main = `e11ba96`, tree clean. Newest `ci.yml` run for `e11ba96` = success (run
  29264702122, completed 2026-07-13T16:01:39Z; headSha verified to MATCH `e11ba96` — see the
  `ci-green-on-main` race caveat in §1 item 8 before trusting this gate blindly next time).
- **D2 + B3 CLOSED GREEN** (both landed before this rewrite; full record + 2 filed noticings live
  in commit `5a01b53` and the noticing files — reference, don't restate):
  - D2: honest UPDATE-recovery rewrite of the p93 real-Confluence smoke (`1c424d7`) + stale gate
    PASS-message fix (`e73d761`), unbiased-verified.
  - B3: attach-sync gate PASS (`e11ba96`); prior FAIL was a phantom stale-skip artifact, not a
    real regression. The re-run is **coverage-hollow / inconclusive-by-construction** re the B1
    attach gap (the gate asserts local git-config + cache-only `sync --reconcile`, never a real
    checkout/fetch round-trip) — this is NOT a caveat-bound waiver, it's a gap in what the gate
    can prove. Binding on the fix below — see §1 item 4a acceptance criteria.
- TokenWorld known-good = **EXACTLY 2 durable pages** (`7766017` + `7798785`,
  child.parentId=`7766017`). Verify: `python3 scripts/confluence_tokenworld.py list`.
- `.planning/STATE.md` frontmatter (`status:`/`last_activity:`) + the workstream_c cursor +
  top blockquote were RECONCILED in this rewrite (was stale: "3-page state, B1 RESOLVED" — reality
  is 2 pages, B1 folded into item 4a below, neither RESOLVED nor a standalone OPEN decision). Its
  Current-Focus/Session-Continuity prose blocks further down were NOT swept (lower-priority
  residual drift, non-blocking — this file is the live cursor).

## 1. ACTIVE CHARTER — OWNER fix-first ruling, items 4-8 (the prior §4 HOLD is LIFTED)

**Owner ruling (2026-07-13, `.planning/CONSULT-DECISIONS.md` `[OWNER]` "fix-first calibration
for tag-blocking product bugs"):** tag-blocking product bugs default to FIX-FIRST, no owner
consult needed, UNLESS the fix turns architectural (then STOP + escalate to manager/owner). This
supersedes the old "no product/defect fix mid-tag-sequence" HOLD and resolves the OPEN B1
non-waivable bind — B1 is now folded into item 4a, not a standalone waive-vs-redesign choice.

4. Two FABLE-named product fixes, **sim-first + tests + code-review gates, NO real-backend
   needed for the fix tests themselves** (a real-backend round-trip smoke IS required as
   acceptance evidence per item 4a's binding note below):
   - **(a) attach lineage.** Seed `refs/reposix/origin/main` at attach-time + evaluate a
     `resolve_import_parent` fallback so EXISTING attach trees heal too. **Design ratified
     BOUNDED-ELEGANT-FIX (no architectural pivot); full implementation-ready design →
     `.planning/milestones/v0.14.0-phases/attach-lineage-fix-design.md`** (persisted this
     session; §§1-7 = the design, §8 = a binding acceptance-criteria addendum). LOAD-BEARING:
     seed VALUE = the mirror merge-base `refs/remotes/<mirror>/main`, **NOT HEAD** (HEAD seeds
     cause silent-revert data loss on un-pushed Pattern-C edits — design §3.1). Heal-existing
     primary path = re-run `reposix attach` (idempotent, zero new machinery); runtime auto-heal
     = v0.15.0-optional (touches the git-trusted import path — gate/defer). If implementation
     reveals it IS architectural (synthetic-history determinism) → STOP + report manager.
     **BINDING acceptance criterion (design §8, folded from a B3 noticing):** B3's real-backend
     gate never exercises a `git checkout`/`fetch`-after-`attach` round-trip — it is
     coverage-hollow re this exact gap. The fix's acceptance evidence MUST include a real
     checkout/fetch round-trip assertion (sim-first per design §5.2, PLUS a real-backend arm),
     not just the existing config-only B3 smokes.
   - **(b) adf_to_markdown fail-closed.** Currently `translate.rs:114` / `types.rs:222`
     substitute `String::new()` for a non-doc ADF root → a push PATCHes an EMPTY body into the
     SoT (destroys attacker-influenceable page content). Make it fail closed with a teaching
     error per the exemplar `crates/reposix-cli/src/init.rs::refuse_existing_repo_root` (teach
     the fix / suggest the alternative / copy-paste recovery). Design §6.
5. **Re-green the vision litmus on the UNMODIFIED Pattern-C harness** — no harness dodges (the
   litmus greens once 4a lands).
6. **Verify recovery-promise docs read TRUE post-fix:** `docs/guides/troubleshooting.md:259-272`
   + `docs/concepts/dvcs-topology.md:93` currently OVER-CLAIM attach round-trip recovery (design
   §7 "NOTICED"). docs-alignment catalog rows fire on drift.
7. **Re-assess the p93 CREATE-recovery convergence gap under fix-first:** filed HIGH,
   `.planning/milestones/v0.14.0-phases/surprises-intake/part-03.md` (2026-07-13 entry, "Real-
   Confluence partial-failure RECOVERY does not converge"). Confident bounded fix → implement;
   architectural recovery-semantics question → keep the D2 honest-harness + documented v0.15.0
   routing and flag it PROMINENTLY in the READY-TO-TAG report.
8. **§4 mechanicals (after 4-7):** honest `pre-release-real-backend` probe exit 0 → re-mint
   `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md` GREEN → FRESH unbiased ratification
   subagent (template `quality/PROTOCOL.md` § Verifier subagent /
   `quality/dispatch/milestone-close-verdict.md`) → author
   `.planning/milestones/v0.14.0-phases/tag-v0.14.0.sh` (pattern `.../v0.13.0-phases/tag-v0.13.0.sh`)
   → **STOP at READY-TO-TAG**; the tag push is the MANAGER's.
   - **KNOWN GATE RACE (HIGH, filed, NOT YET FIXED):** `quality/gates/code/ci-green-on-main.sh`
     grades PASS off `gh run list`'s single most-recent run WITHOUT asserting its `headSha`
     equals the just-pushed HEAD — a run-indexing race can silently grade a STALE commit's
     green as the new commit's green. Full finding + sketch:
     `.planning/milestones/v0.14.0-phases/surprises-intake/part-03.md` (2026-07-13 15:52,
     "code/ci-green-on-main (P0 release gate) can grade a false PASS on a race"). The probe
     mechanic itself is owner-gated (do not edit without authorization under the fix-first
     ruling's architectural-escalation clause if the fix touches shared gate infra broadly) —
     but before trusting item 8's `pre-release-real-backend`/post-push probe for the FINAL tag
     decision, manually cross-check the graded run's `headSha` against `git rev-parse HEAD`
     (as this session did in §0).

## 2. Constraints (unchanged)

Sim-first for code; real backends only via `REPOSIX_ALLOWED_ORIGINS`; sanctioned targets ONLY —
**TokenWorld known-good = EXACTLY 2 durable pages (parent `7766017` + child `7798785`); teardown
every real-backend run; verify `python3 scripts/confluence_tokenworld.py list`** (a leaked/trashed
fixture reds CI); NO tag push ever; never open work over a red main; ONE cargo invocation
machine-wide (prefer `-p`); /tmp leaf isolation (`cd` in the SAME bash call). A `fork` is never a
safe discard — never dispatch one to "throw away," end the turn instead (ORCHESTRATION.md §11).
Relief ~100k soft / ~150k hard (absolute) → replace THIS file, commit+push, end turn. Resume a
child via SendMessage, never fork.

## 3. Ops lessons (this session)

- **Display-freeze false alarm.** A Claude Code survey-modal froze a pane's display/input for
  ~30 min while the background coordinator kept running — the "frozen at Dispatching D2
  executor" reading was a display artifact, not a stall. HEALTH-CHECK via GROUND-TRUTH git (file
  mtime, new commits, `ps` for live procs), NOT the pane view. A **file-activity** Monitor (repo
  + /tmp mtime, not cargo-only) is the reliable progress feed; a cargo-only heuristic
  false-alarms during real-backend/quality-runner phases. SendMessage to a "completed" agent
  RESUMES it (recovers a lost completion).
- **Rogue-fork tree-writer.** A dispatched `fork` inherited the coordinator's full context and
  became a live parallel tree-writer instead of a safe discard, authoring a stale/fictitious
  handover (`5a01b53`) that this rewrite supersedes. Fix-twice: `.planning/ORCHESTRATION.md` §11
  now states forks are never a safe no-op placeholder.
- **`ci-green-on-main` headSha race** — see §1 item 8 bullet. Cross-check `headSha` manually
  until the probe itself is fixed.

## 4. Filed noticings this session (not restated in full — see the files)

- `.planning/milestones/v0.14.0-phases/surprises-intake/part-03.md`: p93 CREATE-recovery
  non-convergence (HIGH, item 7), B1/attach-topology doc-lie (HIGH, folded into item 6), ADF
  empty-body data loss (HIGH, item 4b), B3 coverage-hollow attach-sync gate (MEDIUM, folded into
  item 4a's binding acceptance criterion + design §8), `ci-green-on-main` headSha race (HIGH,
  item 8), litmus-flow.sh file-size warning band (MEDIUM), litmus marker-strip hygiene (LOW),
  verdict-minting freshness guard (MEDIUM), contract-test trashed-page self-seed gap (MEDIUM).
  All routed to v0.15.0 except the ones folded into this milestone's items 4/6/7/8 above.

---
History lives in git — `git log` / `git show`, not restated here.
