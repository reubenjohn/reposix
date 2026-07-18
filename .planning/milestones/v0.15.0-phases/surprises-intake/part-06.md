# v0.15.0 Surprises Intake — Part 6 of 7

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.

## From P122 W2 (2026-07-18, remote-init hardening)

### P122-W2-01 — `teach_scan.py` does NOT recognize `teach_coded(` → 5 helper sites read as un-dispositioned at HEAD (P121 W3 regression)
- **Source:** NOTICED during P122 W2 while dispositioning the new `resolve_import_parent` loud-error block. Running `python3 quality/gates/agent-ux/teach_scan.py --scope helper` on the committed HEAD (`b49a0527`, before my change) already exits 1 with **5 un-dispositioned blocks**: `main.rs:115` (bail!), `bus_url.rs:63`, `backend_dispatch.rs:440`, `stateless_connect.rs:96`, `bus_handler.rs:461`. · **Severity: MED** (a P120 gate is RED; it is `on-demand` cadence so it does NOT block pre-push, hence it slipped through P121 close undetected) · STATUS: OPEN — tag agent-ux / fix-twice / P121-regression.
- **Root cause:** `feat(121-w3): wire RPX codes onto all 6 helper error sites` (`4420670c`) converted these `bail!`/`anyhow!` sites from `teach(…)` (which `teach_scan._PASS_CALL` recognizes via `\bteach\s*\(`) to `teach_coded(…)` — the P121 coded sibling — which `_PASS_CALL` does **NOT** match (`teach_coded(` fails `\bteach\s*\(` because of the `_`). So teach_scan now reads every inline `teach_coded` teaching site as teaching-LESS. Note `rpx_registry_check.py` leg 3 (M2) ALREADY hardcodes `teach_coded(` in its `is_teaching` set, so the two sibling gates disagree — teach_scan is the one with the gap.
- **Why my W2 block is NOT in this count:** I routed my RPX-0508 teaching through a helper (`import_parent_resolve_detail`, mirroring RPX-0507's `import_unreachable_detail`) and dispositioned the thin `anyhow!` wrapper with a `// teach-exempt: ok` marker (plan-sanctioned for an internal wrapper). teach_scan reports exactly the 5 pre-existing, not 6.
- **Sketched resolution (fix-twice, ~5 min, no new dependency):** add `teach_coded\s*\(` to `teach_scan._PASS_CALL` (alongside `\bteach\s*\(`) so the scanner recognizes the P121 coded-teaching idiom — this clears all 5 pre-existing blocks and re-greens `agent-ux/helper-errors-teach-recovery` leg (c). Verify `--self-test` + both `--scope cli|helper` still pass (adding a pass-pattern can only make MORE blocks pass, never newly-flag). Left un-fixed here per the gsd-executor scope boundary (pre-existing failure in files outside the W2 change set); flagged for the coordinator / a W3/W4 or absorption slot to fast-track.
- **RESOLVED (P122 W3, `92614edc`):** extended `_PASS_CALL` to `\bteach(?:_coded)?\s*\(` (recognizes both `teach(` and `teach_coded(`), mirroring `rpx_registry_check.py`. Fuller scope than the W2 sketch: running `--scope cli` at HEAD revealed **26** un-dispositioned blocks (not just the 5 helper ones) — 25 were `teach_coded(` sites cleared by the regex, and 1 residual (`init.rs:155`, the hand-rolled RPX-0401 `bail!`) was a SEPARATE P121 comment-expansion regression that pushed its `// teach-exempt: ok` marker 8 lines above the `bail!`, outside teach_scan's 2-line lookback window — fixed by hugging the marker to the `bail!`. Added a `teach_coded(...)` self-test fixture. After the fix: `--scope cli` (14 files), `--scope helper` (7 files), and `--self-test` all CLEAN (exit 0).

**STATUS:** RESOLVED (P122 W3, `92614edc`)

## 2026-07-18 11:09 | discovered-by: gsd-executor 123-01 (catalog-first Wave 1) | severity: MEDIUM

**What:** The global `gsd-sdk query state.advance-plan` call (per this repo's own `execute-plan.md`
`<state_updates>` step) failed to parse this project's STATE.md (`"Cannot parse Current Plan or
Total Plans in Phase from STATE.md"` — expected, since this repo's STATE.md is a heavily-customized
narrative document, not the generic GSD scaffold), BUT still mutated the file as a side effect
before returning that error: `last_updated` was bumped, the `progress:` frontmatter block was
overwritten with numbers computed by scanning ALL phase directories on disk (`total_phases: 21,
completed_phases: 3, total_plans: 28, completed_plans: 10, percent: 14` — nonsensical for the
active v0.15.0 milestone, which tracks 15 phases / 9 complete per the narrative body), and the
load-bearing `last_activity:` frontmatter line (referenced throughout STATE.md's own prose as the
primary machine-readable cursor) was DROPPED entirely. Root cause: `readModifyWriteStateMd()`
(`sdk/src/query/state-mutation.ts`) unconditionally calls `syncStateFrontmatter()` on every
invocation — even one whose modifier ultimately reports an error — so ANY `state.*` mutation
verb touches frontmatter as a side effect, not just the one the caller intended. Caught only
because `git status`/`git diff` was inspected before staging (per this repo's own "verify against
reality" + targeted-staging discipline); reverted via `git checkout -- .planning/STATE.md` before
any commit. `state.record-metric` was ALSO attempted and returned a clean `{recorded: false,
reason: 'Performance Metrics section not found'}` with no mutation (this repo's STATE.md has no
`## Performance Metrics` / `## Decisions` / `## Blockers` sections either) — so that verb is safe
to call, but `state.advance-plan` is NOT, on this repo's STATE.md shape.

**Why out-of-scope for the discovering session:** The fix lives in the globally-installed
`get-shit-done-cc` npm package (`sdk/src/query/state-mutation.ts`), not in this repo — not a
reposix code change, and not something a phase-execution session should patch mid-task. This repo's
`execute-plan.md` workflow (also a global `~/.claude/get-shit-done/` asset) is the OTHER half of the
fix-twice obligation.

**Sketched resolution:** Two independent fixes, either sufficient defense-in-depth together: (1)
upstream `get-shit-done-cc`: `readModifyWriteStateMd` should skip the frontmatter resync + file
write entirely when the modifier signals an error/no-op (today it writes unconditionally,
conflating "parse succeeded, no change needed" with "parse failed, do not touch the file"); (2)
this repo's `.planning/CLAUDE.md` / `ORCHESTRATION.md` should document that `gsd-sdk query
state.advance-plan` is UNSAFE to call against this repo's narrative STATE.md shape (no generic
`Current Plan`/`Total Plans in Phase` fields) and that any executor hitting the
`state.advance-plan` parse error should immediately `git diff .planning/STATE.md` and revert via
`git checkout -- .planning/STATE.md` before staging anything, rather than trusting the error
response to mean "no side effect occurred."

**STATUS:** OPEN

## 2026-07-18 06:00 | discovered-by: 123-06 (SC4/DRAIN-06, structure/verifier-script-exists gate) | severity: MEDIUM

**What:** The freshly-authored `quality/gates/structure/verifier-script-exists.sh` (scans every
`quality/catalogs/*.json` row's `verifier.script` for existence + executable bit, per its
`claim_vs_assertion_audit` committed in 123-01) finds 5 pre-existing violations that are NOT
mechanically fixable in this plan's <1h eager-fix budget, all already tracked elsewhere:
(1) `cross-platform/windows-2022-rehearsal` and `cross-platform/macos-14-rehearsal` — both rows
in `cross-platform.json` are `status: WAIVED` (renewed P90 90-05, `until: 2026-09-15`) because the
verifier scripts were never built (windows/macos GH Actions runners cost ~$0.08/min; tracked_in
P97 launch-readiness); (2) `docs-build/animation-renders` — `status: NOT-VERIFIED`, its own
`owner_hint` says "both the artifact and the verifier script are intentionally absent until
[117-07] W5"; (3+4) `docs-repro/benchmark-claim/8ms-cached-read` and
`.../89.1-percent-token-reduction` — both `status: NOT-VERIFIED`, `verifier.script: null` by
design, mechanization routed to `GOOD-TO-HAVES-04` (v0.13.0-phases) targeting the launch-readiness
milestone. None of the 5 rows are `PASS` today (2 WAIVED, 3 NOT-VERIFIED) — so none is actually
the "unbacked PASS" hazard GTH-V15-03 names — but the new gate's committed contract scans EVERY
non-docs-alignment row unconditionally (no status filter), so all 5 surface as violations
regardless. The 123-06 executor fixed 32 OTHER violations directly (a mechanical missing-`chmod
+x` class across 13 real scripts) but did not fabricate stub verifier scripts for these 5 just to
force the mechanical check green — that would be exactly the "silently waiving to force a false
PASS" anti-pattern the plan explicitly forbade, and would misrepresent catalog-first placeholders
that are deliberately not yet gradeable.

**Why out-of-scope for the discovering session:** Building real verifiers for windows/macos GH
Actions rehearsal, the playwright animation-capture pipeline, or the headline-number extraction
scripts are each separate, already-scoped efforts (P97, 117-07 W5, GOOD-TO-HAVES-04) — building
any of them inside this plan's mechanical-gate-authoring task would be scope surgery, not a <1h
fix. As a direct consequence, `structure/verifier-script-exists` cannot reach a clean `exit 0`
against the live catalog today; the plan's dispatcher explicitly instructed NOT to promote this
row's `cadences` to include `pre-commit` while the catalog isn't clean (a dirty pre-commit-tagged
row would self-block every future commit repo-wide), so 123-06 left the row's cadences unchanged
at `["pre-push", "pre-pr"]` (as minted in 123-01) rather than adding `pre-commit` as the plan's
Task 2 literally instructed.

**Sketched resolution:** Two independent, non-exclusive paths for the row owner to choose between:
(a) build out the 5 deferred verifiers on their already-tracked timelines (P97 cross-platform
decision, 117-07 W5 animation artifact, GOOD-TO-HAVES-04 headline-number mechanization) — once all
5 clear, `structure/verifier-script-exists` goes green for real and `pre-commit` can be safely
added to its cadences; OR (b) revisit `verifier-script-exists.sh`'s scope to only flag rows whose
CURRENT `status == "PASS"` (the literal GTH-V15-03 hazard — an unbacked PASS — rather than an
honestly-WAIVED/NOT-VERIFIED row admitting its own incompleteness), which would exempt these 5
catalog-first placeholders while still catching the real hazard (a stale PASS whose verifier later
vanished). (b) is a design change to the row's committed `expected.asserts`/
`claim_vs_assertion_audit` and needs the row owner's sign-off, not a silent implementation
narrowing — 123-06 deliberately did not apply it unilaterally. Until either lands, this phase's
verifier-subagent grading should treat `structure/verifier-script-exists`'s current FAIL (5
violations, all cited above, all pre-existing and independently tracked) as a KNOWN, filed,
non-regressing finding, not a new defect introduced by 123-06.

**RESOLUTION (2026-07-18, P123 close):** Path **(b)** taken — the gate's violation definition
was refined from an unconditional scan to a **graded-outcome scope**: it flags a missing OR
non-executable `verifier.script` ONLY for rows whose `status` asserts a run result
(`{PASS, FAIL, PARTIAL}`), and EXEMPTS rows that assert no verifier-backed result (status
`WAIVED`/`NOT-VERIFIED` — the `STALE` display-flavor persists as `NOT-VERIFIED` — or
`verifier.script: null`). This matches GTH-V15-03's intent exactly: the hazard is an unbacked
**graded** claim (a false-green riding on a verifier that can't run), NOT a catalog-first
placeholder that honestly admits its own incompleteness. Under the refined scope all 5 deferrals
above are exempt (2 WAIVED via status; 3 NOT-VERIFIED via status, 2 of those also via null-script)
and `structure/verifier-script-exists` grades **PASS for real** against the live catalog (155
in-scope graded rows, 0 violations, 17 exempt) — so its cadences were promoted to include
`pre-commit`. The 32 chmod-+x fixes 123-06 applied to graded rows stay in-scope and still enforced.
This was a **coordinator design call (DP-5 in-charter, coordinator-resolvable — NOT owner
escalation)**, not the "silent implementation narrowing" 123-06 correctly declined to make
unilaterally; the row's committed `expected.asserts` + `claim_vs_assertion_audit` + `comment` were
updated in the SAME commit so the GREEN contract matches the refined gate, and the selftest now
proves the full truth table (PASS/FAIL/PARTIAL + missing/non-exec → violation; WAIVED/NOT-VERIFIED
+ missing → exempt; null-script → exempt; all-good → pass). Cross-ref: this P123-close refinement
commit (gate `quality/gates/structure/verifier-script-exists.sh` + `.selftest.sh` + the
`freshness-invariants.json` row + `quality/CLAUDE.md`). The 5 deferred verifiers themselves remain
independently tracked (P97 cross-platform, 117-07 W5 animation, GOOD-TO-HAVES-04 headline numbers)
— path (a) is still available to build them out on their own timelines, now purely additive.

**STATUS:** RESOLVED

## 2026-07-18 | discovered-by: 123-07 close-wave Lane 2 (PLANNING-CLOSE + AUDIT, security audit tasked by the coordinator) | severity: MEDIUM

**What:** SC1's `.env` self-sourcing (`quality/runners/_env_load.py`, P123/DRAIN-03) loads
`GITHUB_TOKEN`/`GH_TOKEN` from the repo's `.env` into `run.py`'s process env whenever present.
Several gates shell out to the `gh` CLI, which prefers `GH_TOKEN`/`GITHUB_TOKEN` over an
operator's `gh auth login` keyring session — so a `.env`-sourced token can silently SHADOW the
operator's own `gh` auth for any gate invoked in a context where `run.py` (or a caller that
inherited its self-sourced env) shells to `gh`. `quality/gates/code/ci-green-on-main.sh` was
already hardened with `env -u GH_TOKEN -u GITHUB_TOKEN` ahead of this close wave. Audited the 7
other `gh`-mentioning gates the coordinator named, reading each file directly (not assumed) and
cross-referencing its catalog row's `cadences`:

1. **`quality/gates/code/ci-job-status.sh`** (row `code/cargo-test-pass`, cadence **on-demand**,
   `owner_hint`: "manual post-hoc audits") — REAL `gh run list` invocation. **Local/keyring-reliant.
   Candidate for `env -u GH_TOKEN -u GITHUB_TOKEN`** (P127).
2. **`quality/gates/agent-ux/webhook-trigger-dispatch.sh`** (row
   `agent-ux/webhook-trigger-dispatch`, cadence **on-demand**, `owner_hint` explicitly notes it
   "needs gh auth wired") — REAL `gh api repos/.../contents/...` invocation against the EXTERNAL
   mirror repo. **Local/keyring-reliant. Candidate for `env -u GH_TOKEN -u GITHUB_TOKEN`** (P127).
3. **`quality/gates/release/installer-asset-bytes.py`** (3 rows in `release-assets.json`, cadence
   **weekly** — cron-triggered via `.github/workflows/quality-weekly.yml`) — REAL `gh run list`
   invocation. **CI-cadence: its ONLY credential in that context is the Actions runner's
   `GITHUB_TOKEN` — CRITICAL: do NOT strip GH_TOKEN/GITHUB_TOKEN here, stripping would BREAK the
   scheduled cron job (no keyring exists on a GH Actions runner).**
4. **`quality/gates/agent-ux/p111-ci-wait-helper.sh`** (row `agent-ux/p111-ci-wait-helper`, cadence
   on-demand) — read in full: it never invokes `gh` itself; it is a STATIC `grep` check asserting
   `scripts/ci-wait.sh` (a DIFFERENT, sibling file) contains the string pattern `gh run
   (view|list)`. Not affected by the shadowing hazard at all — false-positive in the coordinator's
   candidate list (the earlier grep matched inside a header comment).
5. **`quality/gates/agent-ux/webhook-first-run-empty-mirror.sh`** (row
   `agent-ux/webhook-first-run-empty-mirror`, cadence **pre-pr**) — read in full: pure local
   git-fixture harness (`git init --bare` + local pushes); the only `gh` mentions are inside
   comments describing what a real `gh repo create --add-readme` would do. No live `gh` call.
   Not affected — false-positive.
6. **`quality/gates/docs-alignment/dvcs-mirror-setup-walkthrough.sh`** (catalogued in
   `doc-alignment.json`) — read in full: `grep`s `docs/guides/dvcs-mirror-setup.md`'s PROSE for
   the literal strings `'gh repo create'` / `'gh secret set'` / `'gh workflow disable'`. Never
   executes `gh`. Not affected — false-positive.
7. **`quality/gates/code/shell-coverage-tests/60-code-ci.sh`** (a kcov coverage-driver for
   `ci-green-on-main.sh`, itself part of the pre-push `code/shell-coverage` gate) — read in full:
   installs a STUB `gh` binary on `PATH` returning canned JSON, explicitly "WITHOUT touching the
   network." Never invokes real `gh`/real auth. Not affected — false-positive.

So of the 7 named files, only 2 (`ci-job-status.sh`, `webhook-trigger-dispatch.sh`) make a REAL
`gh` call AND run in a local/on-demand context where an operator's keyring `gh auth login` session
is the intended credential — these are the genuine `env -u` candidates. 1 (`installer-asset-bytes.py`)
makes a real `gh` call but runs ONLY in a CI-cron context where `GITHUB_TOKEN` IS the (only)
correct credential — stripping it would be a regression, not a fix. The remaining 4 never invoke
live `gh` at all, so the shadowing hazard does not apply to them regardless of cadence.

**Why out-of-scope for the discovering session:** This lane's charter (123-07 close-wave Lane 2)
is planning-artifact + intake work — modifying a CI-cadence `gh` gate right before the phase-close
push carries CI-break risk if the local-vs-CI classification above is misjudged under time
pressure; the careful per-gate patch (with a live re-run of the weekly cron's real invocation
context to double-confirm classification 3 before touching anything) belongs to its own reviewed
lane, not folded into a phase-close audit pass.

**Sketched resolution — P127 candidate:** (1) Patch `quality/gates/code/ci-job-status.sh` and
`quality/gates/agent-ux/webhook-trigger-dispatch.sh` with the same `env -u GH_TOKEN -u
GITHUB_TOKEN` wrapper `ci-green-on-main.sh` already carries, so a `.env`-sourced token cannot
shadow the operator's keyring session for these two local/on-demand gates. (2) Do **NOT** touch
`installer-asset-bytes.py` — its weekly-cron cadence needs `GITHUB_TOKEN` un-stripped; if this
gate is ever also run locally by an operator, revisit then (a mode flag, not a blanket strip). (3)
No change needed for the 4 false-positive files identified above (`p111-ci-wait-helper.sh`,
`webhook-first-run-empty-mirror.sh`, `dvcs-mirror-setup-walkthrough.sh`, `60-code-ci.sh`) — they
never shell to a real, unstubbed `gh`. **CRITICAL caveat for whoever picks this up:** re-verify the
CI-vs-local classification for each candidate against its CURRENT catalog `cadences` field before
patching (cadences can move between phases) — stripping `GH_TOKEN`/`GITHUB_TOKEN` from a gate that
turns out to run in a CI-only context with no other credential would silently break that gate's
only auth path.

**STATUS:** OPEN

