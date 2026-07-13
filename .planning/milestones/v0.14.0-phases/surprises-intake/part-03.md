# v0.14.0 Surprises Intake (P110 source-of-truth) — Part 3 of 3

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.

## 2026-07-13 | discovered-by: B2 (v0.14.0 tag-remediation lane, p93 harness fix, verify-against-reality) | severity: HIGH

**Title:** Real-Confluence partial-failure RECOVERY does not converge — `partial_failure_recovery_real_confluence` push 2 fails deterministically with `some-actions-failed`; p93 (P0) blocks the v0.14.0 tag for a NEW reason once the self-reject is cleared. **[owner decision required]**

**What:** B2 pinned the p93 verifier to the sanctioned TokenWorld space (commit `311d7fe`),
clearing the self-reject that previously exit-1'd the row BEFORE any backend call. With the pin,
the `partial_failure_recovery_real_confluence` smoke (`crates/reposix-cli/tests/agent_flow_real.rs`)
now actually runs against real Confluence — and a DEEPER, previously-MASKED failure surfaces. Push 1
(create page A top-level + page B under an unresolvable parent) partially-fails AS DESIGNED (asserts
:657-661 PASS; page A LANDS in TokenWorld). Push 2 — the RECOVERY push (re-send already-landed A +
retry B with a valid parent) — is asserted to converge (`ok refs/heads/main`) after PRECHECK B diffs
away the content-equivalent already-landed A and replans only B. Instead it fails DETERMINISTICALLY
(identical across two credentialed runs 2026-07-13T02:45Z / 02:48Z — NOT a consistency flake):

```
crates/reposix-cli/tests/agent_flow_real.rs:690:5:
push 2 (recovery) must succeed and converge; stdout=error refs/heads/main some-actions-failed
```

Strong hypothesis (not yet proven to the failing action): PRECHECK B does NOT diff away the
already-landed page A against real Confluence, so page A gets a duplicate-title CREATE that Confluence
rejects (Confluence enforces unique titles within a space) → `some-actions-failed`. The wiremock-sim
twin `crates/reposix-remote/tests/partial_failure_recovery.rs` does NOT model unique-title
enforcement, so it stays GREEN while the real backend does not — a real-backend-only
recovery-convergence gap. Evidence (both runs, verbatim transcripts):
`.planning/quick/260712-phc-author-two-missing-pre-release-real-back/B2-p93-proof.txt`.

**Why out-of-scope for B2 / why FILE not eager-fix:** B2's charter was the harness space-targeting
self-reject, which is FIXED. This is a distinct defect in load-bearing helper / PRECHECK-B export
protocol + real-backend semantics (unique-title / read-after-write). A correct fix needs its own
diagnostic (confirm WHICH action fails via the cache/confluence audit rows) + design (does PRECHECK B
match on title? id? content-hash? how does it dedupe an already-landed create on retry?) + real-backend
regression — >1h, load-bearing protocol code, Rule-4 architectural. Folding it into B2 would double
scope.

**Sketched resolution:** (1) Diagnose the failing action deterministically — instrument
`run_helper_export_real` to surface the helper's per-action stderr, or read `audit_events_cache` /
the confluence adapter audit rows for push 2, to confirm A-duplicate vs B-fail. (2) If A-duplicate:
make PRECHECK B's replan recognise the already-landed content-equivalent page A (match on the record
`id` / content-hash, not a blind re-create) so retry does not re-POST it. (3) Add the real-backend
regression back through `quality/gates/agent-ux/p93-partial-failure-recovery-real-confluence.sh`
(now correctly TokenWorld-pinned) — it is written to flip PASS (exit 0) once push 2 converges.
Test-hygiene sub-item: on the recovery-fail path the smoke leaves an orphan
`p93 smoke A (kind=test <ts>)` page in TokenWorld each run (page A lands, recovery never converges to
clean up) — a human title-sweep per `docs/reference/testing-targets.md` is needed; two orphans exist
from the B2 verification runs.

**OWNER DECISION REQUIRED (tag-blocking):** the p93 row carries `blast_radius: P0` and runs at the
`pre-release-real-backend` cadence, so with creds present it grades **exit 1 (FAIL, not
NOT-VERIFIED)** — the v0.14.0 tag remains BLOCKED by p93 for THIS new reason, not the (now-cleared)
self-reject. The owner must choose: (a) fix the recovery-convergence gap before the tag (dedicated
phase), or (b) explicitly waive p93 with a documented `until_date` per the coverage-kind honesty rule
(never a bare PASS-with-comment). Do NOT cut the tag treating p93 as green.

**STATUS:** DEFERRED — dedicated real-backend recovery-convergence fix phase (Rule-4 architectural).
Distinct from the two B1 mirror-refresh doc-gaps in `part-02.md`. Tracked as the active p93 blocker
for the v0.14.0 tag; owner decision (fix-before-tag vs documented waiver) pending.

## 2026-07-12 21:15 | discovered-by: B1 (litmus self-heal proof, verify-against-reality) | severity: HIGH

**What:** The documented recovery `git pull --rebase && git push` (root CLAUDE.md § "DVCS
push/pull issues", `docs/guides/troubleshooting.md`) does NOT recover for the canonical
vanilla-clone+attach topology — proven against live TokenWorld: fetching the bus remote shows
`[new branch] main -> reposix/main` (a freshly reconstructed SYNTHETIC history unrelated to the
mirror-clone's ancestry), so the rebase hits `add/add` conflicts on every shared file
(`pages/7798785.md`, `pages/2818063.md`) and fails. The recovery only works for trees already
tracking a bus ref with shared ancestry. Mirrors the B1 lineage gap in
`.planning/milestones/v0.14.0-phases/evidence/B1-litmus-selfheal-INSUFFICIENT-FINDINGS-2026-07-12.md`.

**Why out-of-scope for B1:** The fix is entangled with the pending owner decision on the B1
litmus-substrate redesign (bus-tracking-ref checkout) — fixing the docs claim without fixing the
underlying topology gap would just be a different doc-lie.

**Sketched resolution:** Once the substrate decision lands (owner OPEN entry,
`.planning/CONSULT-DECISIONS.md`), correct the troubleshooting doc to scope the recovery's
applicability (bus-tracking trees only) or ship the substrate fix that makes it universally true.

**STATUS:** OPEN — routed to v0.15.0, entangled with the B1 owner decision.

## 2026-07-12 21:15 | discovered-by: B1 (litmus self-heal proof, verify-against-reality) | severity: HIGH

**What:** Page 2818063's ADF body is unparseable by reposix-confluence
(`adf_to_markdown: root node type must be "doc", got ""`), and on every clean push through this
fixture the translator silently falls back to an EMPTY body (`adf_to_markdown failed; using
empty body`) rather than refusing or round-tripping the original bytes. Observed twice in the
same real-TokenWorld transcript (`quality/reports/transcripts/milestone-close-vision-litmus-real-backend.txt`).
This is a real-backend data-loss risk on any page with a non-standard ADF body, not just the
litmus fixture.

**Why out-of-scope for B1:** B1's charter was proving/disproving the self-heal, not fixing the
ADF translator; the fix needs its own design (refuse vs. best-effort round-trip) and regression
coverage.

**Sketched resolution:** `reposix_confluence::translate::adf_to_markdown` should refuse the
translation (surface a loud error) rather than silently emptying the body when the root node
type check fails, OR gain a lossless round-trip fallback that preserves the original ADF bytes
for write-back. Add a regression fixture with a non-`doc`-root ADF body.

**STATUS:** RESOLVED-in-item-4b (commit d1cc811, 2026-07-13) — fail-closed per
attach-lineage-fix-design.md §6. `translate.rs` + `types.rs` no longer substitute
`String::new()` on ADF failure: they prefer the raw `storage` HTML fallback, else substitute a
conspicuous NON-EMPTY teaching sentinel (`adf::unreadable_adf_body`, marker
`[reposix: unreadable ADF body — see recovery]`) naming the root type + page id with a
copy-paste storage-refetch recovery. The export path (`create_record`/`update_record`) FAILS
CLOSED on the sentinel via `adf::is_unreadable_adf_sentinel`, so a placeholder can never PATCH
the SoT to empty. Regression coverage added (adf.rs / translate.rs / client.rs unit tests,
including a PUT-mock `expect(0)` proof that no empty-body write escapes). The empty-root-type
`got ""` case in this entry maps to a sentinel naming root type `""`. Residual (noted, NOT part
of the item-4b hazard): a page fetched with NEITHER adf NOR storage still translates to an empty
body by design (a genuinely-empty page is not a silent substitution) — see the item-4b executor's
NOTICED report.

## 2026-07-12 21:15 | discovered-by: PRIORITY-ZERO red-CI sweep | severity: MEDIUM

**What:** `contract_confluence_live_hierarchy`'s self-seed fallback treats only a `get_record`
ERROR as "missing"; a TRASHED durable page resolves `Ok` (status trashed, parentId null) so the
fallback is bypassed and the read-only assert hard-panics instead of self-seeding. Root-caused
and worked around (restore) in
`.planning/milestones/v0.14.0-phases/evidence/PRIORITY-ZERO-red-ci-sweep-2026-07-12.md`; the
test-robustness gap itself was not fixed.

**Why out-of-scope for the sweep:** The sweep's charter was restoring red CI, not hardening the
contract test; the fix is a small but distinct code change to test logic, not a CI-health
restoration.

**Sketched resolution:** Gate the read-only assert on `status == current` (or treat `trashed` as
equivalent to "missing" so the self-seed path runs), making the contract test self-healing
against fixture drift instead of hard-panicking.

**STATUS:** OPEN — routed to v0.15.0.

## 2026-07-12 21:15 | discovered-by: B1 (litmus self-heal implementation, commit d413432) | severity: MEDIUM

**What:** `quality/gates/agent-ux/lib/litmus-flow.sh` is 9794/10000 B (97.9% — inside the
75-100% EARLY-WARNING band tracked by `structure/file-size-limits`), near the split threshold.
The self-heal added in d413432 (~28 lines) is what pushed it into the warning band.

**Why out-of-scope for B1:** Splitting the file is a structure/readability refactor orthogonal
to the self-heal feature; folding it in would double the diff for a mechanical change.

**Sketched resolution:** Split `litmus-flow.sh` along its existing STEP boundaries (the header
comment already documents `_litmus_flow` as composed of STEP 3-6 sub-functions) into a sibling
file under `quality/gates/agent-ux/lib/`, same pattern as other multi-file gate libs.

**STATUS:** OPEN — routed to v0.15.0.

## 2026-07-12 21:15 | discovered-by: B1 (litmus self-heal implementation, commit d413432) | severity: LOW

**What:** The litmus edit-and-commit step strips only `litmus-marker-` lines
(`sed -i '/litmus-marker-/d'`), leaving stale markers from other prefixes (e.g. `T2-attach-*`,
used by a different agent-ux gate) accumulating in the mirror over repeated runs.

**Why out-of-scope for B1:** Cosmetic hygiene unrelated to the self-heal's correctness; not
blocking any current gate.

**Sketched resolution:** Broaden the strip pattern (or standardize all litmus/attach marker
prefixes under one strippable convention) so repeated real-backend runs don't leave marker
litter in the mirror content.

**STATUS:** OPEN — routed to v0.15.0.

## 2026-07-13 | discovered-by: B3 (tag-remediation, attach-sync re-run, verify-against-reality) | severity: MEDIUM

**What:** The v0.14.0 milestone VERDICT's B3 row asserted an `attach-sync-real-backend`
**FAIL** (exit 1, empty asserts; VERDICT L114-119) that no fresh on-disk artifact ever
corroborated — the artifact present at verdict-time was a stale 2026-07-06 env-missing
**skip**, not a captured FAIL (the same verdict flags this at L150-156). A fresh creds-loaded
re-run (2026-07-13T15:36:55Z; `quality/reports/verifications/agent-ux/attach-sync-real-backend.json`
exit 0; `attach_real_confluence` + `sync_real_confluence` both ok) is the FIRST
empirically-grounded result for the row — a clean PASS. The "FAIL" was a phantom: B3 was
never broken and is NOT a B1-class attach-ref-seed gap.

**Why out-of-scope for B3:** B3's charter was run/classify/document a gate; this is a
process-integrity gap in how a milestone VERDICT was minted, not a code defect.

**Sketched resolution:** Verdict-minting guard — every FAIL / NOT-VERIFIED row in a
milestone VERDICT must cite a fresh artifact captured THAT session (timestamped, on-disk or
committed); a row whose only backing is a stale or env-missing *skip* artifact grades
NOT-VERIFIED (fail-closed, OD-2), never FAIL. Assert artifact-freshness in the
ratification-subagent template (`quality/dispatch/milestone-close-verdict.md`).

**STATUS:** OPEN — routed to v0.15.0.

## 2026-07-13 15:52 | discovered-by: B3 (tag-remediation, attach-sync-real-backend independent re-run, verify-against-reality) | severity: MEDIUM

**Title:** `attach-sync-real-backend` real-backend PASS is inconclusive by construction — the smokes never exercise the unseeded `refs/reposix/origin/main` round-trip, so this gate cannot settle whether the B1 product gap bites the `reposix attach` path. **[v0.15.0 caveat-bound]**

**What:** An independent B3 re-run reproduced the gate's PASS against live Confluence TokenWorld (fresh transcript `2026-07-13T15:52:28Z`: `test attach_real_confluence ... ok`, `test sync_real_confluence ... ok`, `test result: ok. 2 passed; 0 failed`, exit 0). But `run_attach_real`/`assert_attach_configured`/`assert_sync_reconcile_ok` (`crates/reposix-cli/tests/agent_flow_real.rs:237-342`) assert ONLY (1) local `git config extensions.partialClone` + `remote.<name>.url` after `reposix attach`, and (2) `reposix sync --reconcile` exits 0 and prints `"reposix sync:"` — a cache-only rebuild that never invokes `git-remote-reposix`. Neither smoke does a `git checkout`/`fetch`/`pull` against the configured `reposix` remote, so neither ever asks git to resolve `refs/reposix/origin/main`. Corroborated by reading the product code (read-only, no `src/` edit): `attach.rs:259` runs a plain `git remote add <remote_name> <url>` (default `remote_name` = `"reposix"`, `attach.rs:60-61`) with NO accompanying `remote.<name>.fetch` refspec — contrast `init.rs:323`, which DOES configure `remote.origin.fetch = refs/reposix/origin/*`. Meanwhile `reposix-remote/src/main.rs:402` `resolve_import_parent()` hardcodes the literal ref `refs/reposix/origin/main` as the fast-forward parent it looks for — a ref the attach topology's `reposix` remote has no configured path to ever populate. So the PASS is genuine (the two asserted invariants both hold) but by construction neither confirms nor contradicts the B1-class "unseeded `refs/reposix/origin/main`" gap on the attach path. Verdict: **(c) passes-but-coverage-hollow (inconclusive by construction)** — not (a) same-as-B1 (nothing failed) and not (b) harness-fixable-today (the missing coverage is a new smoke, not a harness bug).

**Why out-of-scope for this re-run:** Writing the missing round-trip smoke (a real `git checkout origin/main` / `git pull` after `reposix attach`, against live TokenWorld) is new test-authoring work entangled with the still-open B1 owner decision on the mirror/bus-tracking substrate (`.planning/CONSULT-DECISIONS.md`) — landing it now risks encoding the wrong shape twice. This re-run's charter was reproduce + classify, not design a new smoke, and the charter forbids `src/` edits.

**Sketched resolution:** Add a real-backend smoke that, after `reposix attach <spec> <repo> --no-bus`, runs `git -C <repo> fetch reposix && git -C <repo> checkout reposix/main` (or the bus-tracking equivalent once the B1 substrate decision lands) and asserts it either succeeds OR fails with the documented B1 symptom — either result closes the B3 question for real. Until that smoke exists, treat `attach-sync-real-backend` PASS as "config + cache-list only," never as "attach round-trips like init." Sibling noticing: `attach_real_confluence`/`sync_real_confluence` fn names read as "the round-trip works," which overclaims relative to what they assert — same class of gap `agent-ux/test-name-vs-asserts` exists to catch; worth a follow-up audit of whether these two should carry an explicit `test-name-honesty` marker or a rename (e.g. `attach_real_confluence_configures_remote`).

**STATUS:** OPEN — routed to v0.15.0, ties to the B1 product gap (GTH-09 / owner-gated caveat). A genuine B3 verdict needs the new round-trip smoke above before this row's PASS can be trusted as evidence either way.

## 2026-07-13 15:52 | discovered-by: B3 (tag-remediation, post-push cadence review, verify-against-reality) | severity: HIGH

**Title:** `code/ci-green-on-main` (P0 release gate) can grade a false PASS on a race — it queries `gh run list` immediately post-push, before GitHub has indexed the new `ci.yml` run for the just-pushed commit, so it can silently grade the PRIOR commit's green as the new commit's green.

**What:** `quality/gates/code/ci-green-on-main.sh` (read-only review only — probe mechanicals are owner-gated, not edited here) queries `gh run list --workflow=ci.yml --branch=main --limit=1 --json databaseId,conclusion,status` and grades PASS iff that single most-recent run's `conclusion == "success"`. It correctly returns NOT-VERIFIED (exit 75) when the returned run's `status != "completed"` — but that check only protects against a run that has ALREADY appeared in the list and is still in progress. It does nothing for the window BEFORE the new run appears at all: GitHub Actions' run creation is asynchronous relative to a push landing, so a `gh run list` called seconds after `git push origin main` can still return the run for the PREVIOUS commit (already `completed`/`success`) if the new run hasn't been indexed yet. The script never asserts the graded run's `headSha` equals the pushed commit's SHA, so it cannot distinguish "new commit is green" from "old commit was green and the new run just hasn't shown up yet." This is a correctness gap in a P0 gate that blocks milestone tags.

**Why out-of-scope for this re-run:** Fixing `ci-green-on-main.sh` is a probe-mechanicals edit, explicitly owner-gated for this charter (B3's hard constraints forbid touching it) — documenting, not editing, is the mandate.

**Sketched resolution:** Add `headSha` to the `gh run list --json` query, compare it against `git rev-parse HEAD` (the SHA just pushed); if they don't match, poll (`gh run list` again, or resolve the run id and `gh run watch <id>`) until either a run for the current HEAD appears or a timeout elapses, grading NOT-VERIFIED on timeout — never a stale-run PASS.

**STATUS:** OPEN — routed to v0.15.0, probe-integrity fix (owner must authorize touching `quality/gates/code/ci-green-on-main.sh`). Mitigated THIS session by manually cross-checking the post-push graded run's `headSha` against `git rev-parse HEAD` before trusting the PASS (see this session's post-push step).
