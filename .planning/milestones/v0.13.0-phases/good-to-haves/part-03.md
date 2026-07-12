# v0.13.0 GOOD-TO-HAVES — Part 3 of 8

> Split from `GOOD-TO-HAVES.md` for the file-size gate (OP-8 drain). Index: `../GOOD-TO-HAVES.md`. Entries preserved verbatim.

## GOOD-TO-HAVES-16 — `quality/runners/run.py` mutates the catalog in place with no `--dry-run` escape hatch

**Discovered during:** P91 91-06 deferred-items.md reconciliation (2026-07-04)

**Size:** S

**Source:** `run.py` writes verdicts back into the catalog JSON as a side effect of running (catalog-first state mutation), with no flag to preview what a run would change without committing the mutation. An agent (or a human) who wants to know "what would this cadence flip before I run it for real" has no way to ask without accepting the write.

**Acceptance:** Add a `--dry-run` flag to `run.py` that executes the full verifier sweep and prints the would-be verdict diff (row id, old status → new status) without writing the catalog file. Document the flag in `quality/PROTOCOL.md` alongside the existing runner-behavior description (the XS honest callout added in 91-06 names this gap; this entry is the actual flag implementation).

**Why deferred:** implementing a true dry-run mode means threading a write-suppression flag through every catalog-mutation call site in `run.py` and its shared runner helpers — real (if small) plumbing, not a one-line change, and orthogonal to 91-06's docs-only charter.

**Default disposition:** S — default-defer; natural fit for the next `quality/runners/` framework-touching phase (P95/P96 territory, alongside GOOD-TO-HAVES-06's `run.py`/`verdict.py` line-count gate).

**STATUS:** OPEN

---

## 2026-07-05 | `reposix init` should honour `REPOSIX_SIM_ORIGIN` (test-port hardcode) | discovered-by: P91 CI-red fix executor

**What:** `crates/reposix-cli/src/init.rs:55` (`translate_spec_to_url`) hardcodes `DEFAULT_SIM_ORIGIN` (`127.0.0.1:7878`) for `sim::<slug>` and — unlike `crates/reposix-cli/src/sync.rs:84` — does NOT honour the `REPOSIX_SIM_ORIGIN` env override. This forced the `agent-ux/real-git-push-e2e` gate to bind its sim on the exact default port 7878 (commit 5eae1c9): binding any other port makes init bake a 7878 URL into `remote.origin.url` while the sim listens elsewhere, so the fetch targets a dead port AND trips the egress allowlist. The task's original NOTICED item ("SIM_BIND=7781 hardcode port-collision risk") is a symptom of this asymmetry.

**Acceptance:** teach `translate_spec_to_url` (or its caller) to honour `REPOSIX_SIM_ORIGIN` when the backend is `sim`, mirroring `sync.rs:84-90` (`std::env::var("REPOSIX_SIM_ORIGIN").ok().filter(|s| !s.is_empty())`). Then `real-git-push-e2e.sh` can use a dedicated collision-proof port again by exporting `REPOSIX_SIM_ORIGIN`, and the init/sync inconsistency is closed. Add a unit test for the override (needs a non-env-mutating shape — e.g. pass the resolved origin as a param, or a serialized-env test lock as `history.rs` uses).

**Why deferred:** production-code change touching init URL generation + a non-trivial (env-mutation-safe) test; the CI-red hotfix pinned the port to 7878 instead, which is sequential-run-safe. Low value until someone needs distinct sim ports across concurrent gates.

**Default disposition:** S — default-defer; next `reposix-cli` init/sync-touching phase.

**STATUS:** OPEN

---

## 2026-07-05 | Owner may want to set the `JIRA_TEST_PROJECT` repo secret (KAN) | discovered-by: P91 CI-red fix executor

**What:** `.github/workflows/ci.yml` forwards `JIRA_TEST_PROJECT: ${{ secrets.JIRA_TEST_PROJECT }}` in the jira job env, but the repo has no such secret, so it arrives as the empty string. The code was hardened this session to treat empty-set as unset (falls back to `TEST`, commit 963f8bc), so CI is now robust either way. HOWEVER, per `docs/reference/testing-targets.md` + the ci.yml comment (D91-09), the owner's live JIRA project key is **KAN**, not `TEST` — the intent is for the real-backend smoke to target the project the tenant actually owns. Right now, absent the secret, the jira init-smoke targets `jira::TEST` (which passes because it's a config-string smoke that doesn't require the project to exist).

**Acceptance:** owner runs `gh secret set JIRA_TEST_PROJECT` (value `KAN`) so any future jira gate that lists/mutates real records targets the owned project. Purely an owner action; the code is already robust to it being present-or-absent.

**Default disposition:** XS owner-action — no code change. File for owner awareness only.

**STATUS:** OPEN (owner decision)

---

## 2026-07-05 | Coverage-as-asset: propose a `code/coverage-ratchet` catalog row | discovered-by: doctrine-coverage audit (owner request)

**What:** `cargo-llvm-cov` runs in CI (`.github/workflows/ci.yml` `coverage` job → Codecov, lines 366-386) but NO doctrine or gate watches, raises, or ratchets code coverage. Coverage is generated and then ignored — it can silently regress phase-over-phase with zero signal. The steward-window checklist (RUNBOOK ch.02 §A-L5) now carries a watch-only reminder, but watch-only is honest-but-weak: nothing prevents a slow decay.

**Acceptance:** mint one `code/coverage-ratchet` row in `quality/catalogs/` (dimension `code`, kind `mechanical`) + one verifier in `quality/gates/code/` that reads the lcov/Codecov result and flags when workspace line coverage drops below a stored ratchet floor (floor only moves UP — a green run bumps the floor; a drop fails/alerts). This is the framework's own one-row-one-verifier extension contract (the runner discovers by tag — no new top-level script, no new pre-push wiring). Start as `weekly` / `pre-pr` **alerting** to establish a trustworthy baseline, promote to blocking (`pre-release`) once the floor is stable.

**Why deferred / proposal-only:** inventing a blocking enforcement gate silently is exactly the anti-pattern the framework forbids. A coverage gate carries real design decisions — floor value, per-crate vs workspace, alert-vs-block, flaky-coverage tolerance — that deserve a scoped phase, not a bolt-on. Filed as a proposal (not a stealth gate) so the owner / next `quality/gates/code/`-touching phase scopes it deliberately.

**Default disposition:** M — default-defer; natural fit for the launch-readiness milestone (its headline-numbers work already mechanizes CI-verified metrics) or the next `quality/gates/code/` phase.

**STATUS:** OPEN

---

## 2026-07-05 | Cache delta-sync under-reports changed records, blocking `git rebase`'s lazy blob fetch | discovered-by: P92 T4 prove-before-fix executor

STATUS 2026-07-05 (D-P92-03): **REPRODUCED — CONFIRMED REAL, deterministic** (P93 Exec1, prove-before-fix lane). `not our ref <oid>` reproduced 4/4 in the same-wall-clock-second window (1 deterministic cursor-pin + 3 natural same-second runs, git 2.54 container); CLEAN in the 2s-gap negative control (`1 changed` → ordinary `CONFLICT (content)`). Exec2's non-repro was a timing straddle across a second boundary, not evidence of falseness. **Root cause + full transcripts + FAILING RED regression:** `.planning/phases/93-cache-coherence/93-DP2-REPRO-NOTES.md`. Trigger = sim `list_changed_since` seconds-truncation + strict `>` (`crates/reposix-sim/src/routes/issues.rs:138-139,180-183`); latent amplifier = `Cache::sync` Step 4 sources the tree from `list_records` (full current) while blob-materialization + `oid_map` cover only the `list_changed_since` delta (`crates/reposix-cache/src/builder.rs:293-328`), so an under-report leaves a dangling tree entry → `read_blob` `UnknownOid` → `not our ref`. NOTE: the earlier "even 2+ seconds after" phrasing was imprecise — the window is the SAME truncated second, not an unconditional lag. No production fix applied (coordinator-gated). D-P92-03 `[SELF]` close deferred to the coordinator.

**What:** During the T4 two-writer conflict repro (`.planning/phases/92-push-flow-correctness/92-T4-REPRO-NOTES.md`), after B's stale push is correctly rejected and B runs `git pull --rebase origin main`, the FETCH leg succeeds and advances `refs/reposix/origin/main` (ancestry preserved, HIGH-1 stays fixed), but `git rebase`'s own 3-way merge then fails: `fatal: git upload-pack: not our ref <oid>` / `could not fetch <oid> from promisor remote`. Root cause candidate: the cache's delta-sync (`list_changed_since` cursor query, `crates/reposix-remote/src/precheck.rs`) reports `0 changed (of 6)` even 2+ seconds after the conflicting writer's edit landed on the backend, so the specific blob the rebase's merge needs was never lazily materialized into B's cache object store — `git upload-pack` then genuinely doesn't have the object (`allowAnySHA1InWant=true` is already set per `crates/reposix-cache/src/cache.rs:713-726`, so this isn't a config gap; the object plain isn't there).

**Acceptance:** root-cause why `list_changed_since`/the cache's "since" cursor comparison misses a record that unambiguously changed (check the SIM's `updated_at` timestamp precision vs the cursor comparison operator — `>` vs `>=` — and/or whether the cache's delta-sync path actually writes new blob objects for records it DOES detect as changed vs just advancing a marker commit). Add a regression test exercising `git pull --rebase` (not just `git fetch`) to completion for the T4 two-writer scenario once root-caused.

**Why deferred:** different root cause than `cb630e5` (that fix was `Cache::open`'s git-config shell-out env pollution; this is the delta-sync cursor/materialization path) — Rule 4 territory (architectural, needs its own investigation), not a hand-wave-able quick fix. Also affects whether SC1's literal "completes step 6 + step 7" acceptance criterion can be met as worded — a later P92 wave or P93+ should decide whether to loosen that wording or fix the underlying gap.

**Default disposition:** M — default-defer; likely intersects P93's L2/L3 cache-coherence redesign (`refresh_for_mirror_head` / SotPartialFail work) since both touch the cache's post-write refresh semantics.

**Appended P96 Wave 3a — efficiency residual (LOW, distinct from the correctness question):** The
CREATE-path correctness half of the same-second under-report was proved a FALSE-ALARM this
window (SURPRISES `list_changed_since UNDER-materializes` entry → RESOLVED; CONSULT-DECISIONS
`2026-07-05 [SELF] list_changed_since ... false alarm`): a record missed by a truncated delta
window is still resolvable because ADR-010's Step-5 full-list upsert writes its `oid_map` row
and `read_blob` re-fetches lazily. What REMAINS is an efficiency cost, not a correctness gap:
the way the cache stays coherent across the `>`-boundary is precisely that `Cache::sync` Step 4
sources the tree from a FULL `list_records` recompute (`crates/reposix-cache/src/builder.rs:293-328`)
rather than a pure delta — so a single same-second write forces a whole-list recompute every
sync. A `>=`-with-dedup or a monotonic high-water cursor on `list_changed_since` would let the
delta path stay authoritative and drop the full-list fallback, saving one `list_records` call
per same-second sync. LOW: correctness is fine today; this is a hot-path allocation/IO saving
for the L2/L3 cache-hardening window (v0.14.0), not a bug.

**STATUS:** OPEN

---

## 2026-07-05 | Preamble-anchored marker scan (retire the fixed 6-line lookback in `test-name-vs-asserts.sh`) | discovered-by: P95 marker-footgun pass | severity: LOW

**What:** `test-name-vs-asserts.sh` detects a fn's `#[test]` attribute / `#[ignore]` gate / honesty marker via a FIXED `CONTEXT_LINES=6` lookback ending at the `fn` signature. A marker or `#[test]` attribute placed farther than 6 lines above the fn (e.g. behind a long `///` doc block) is silently ignored — documented as a footgun in the sibling RESOLVED entry above, mitigated by documentation but not eliminated.

**Acceptance:** Replace the fixed 6-line `sed` context extraction with a scan anchored to the fn's actual attribute/doc preamble — walk upward from the `fn` line through the contiguous `#[...]` / `//` / `///` / blank-line block and stop at the first non-preamble code line; treat THAT block (plus the signature line) as the context window. This removes the distance constraint entirely (the marker/attr can sit anywhere in the fn's preamble) while keeping the same "must be in the fn's own preamble, not a sibling's" scoping.

**Why deferred / proposal-only:** (a) Zero present value — the P95 probe found 0/85 current test-pattern fns affected, so it fixes nothing on today's tree. (b) It changes a repo-wide P0-adjacent pre-push gate's core scan loop; the upward-walk needs careful over-capture guards (don't bleed into a preceding item's body across blank lines) and a full-corpus re-run to prove no new RAISEs — more than a low-risk <1h change at milestone-close. (c) Design tension worth a deliberate decision: the original filer considered tight placement a FEATURE ("signature + 1-2 setup lines is the typical case"); a preamble-anchored scan loosens that. A future `quality/gates/agent-ux/` phase should weigh feature-vs-robustness and implement + corpus-verify in one scoped pass.

**Default disposition:** LOW — fold into the next `quality/gates/agent-ux/` framework-touching phase. No blocker; documentation already closes the invisibility harm.

**STATUS:** OPEN

---

## 2026-07-05 | Tighten `audit-immutability.sh` WAL grep to a single-line match | discovered-by: P92 security-waiver-flip executor | severity: LOW

**What:** `quality/gates/security/audit-immutability.sh` validates that `crates/reposix-cache/src/db.rs` sets `PRAGMA journal_mode=WAL` via two independent grep calls: `grep -q 'journal_mode' <db.rs> && grep -q '"WAL"' <db.rs>`. This checks both substrings exist *anywhere* in the file, not on the same statement. A `journal_mode` mention in a comment plus `"WAL"` in an unrelated log string would pass the check vacuously today.

**Acceptance:** Tighten the gate's WAL validation to a single-line or statement-scoped pattern (e.g. grep for `PRAGMA journal_mode.*WAL` or equivalent within a single line's context) so the gate confirms the pragma is actually set, not just that both words appear scattered in the file.

**Why deferred:** Low risk today (`db.rs` is stable and the current check passes correctly); the asymmetry matters only if `db.rs` becomes a high-churn area where a casual comment or log edit could trigger a false-pass.

**Default disposition:** LOW — fold into the next `quality/gates/security/` framework-touching phase (P95/P96) or when `db.rs` development density increases. No blocker today.

**STATUS:** OPEN | 2026-07-05 debt-drain triage: DEFERRED (confirmed, not actioned). Editing the gate's grep logic requires RE-RUNNING the gate (cargo) to confirm it still passes post-edit, which cannot be verified under this window's no-cargo firewall. Kept OPEN, routed to the P95/P96 security-gates window as already noted above.

---

## 2026-07-05 | Drain or consciously renew `structure/file-size-limits` before its 2026-08-08 waiver expiry | discovered-by: P92 verifier + security-waiver-flip executor | severity: LOW

**What:** The `structure/file-size-limits` catalog row is WAIVED (warn-now/block-later) until 2026-08-08. On that date, the waiver expires and the gate will start BLOCKING all pushes where files exceed their per-extension budgets (`crates/reposix-cache/src/db.rs` already listed in `GOOD-TO-HAVES-15`). The milestone-close checklist (P97) must address this waiver before the date flips: either drain the overage files back under their limits OR make a conscious decision to extend the waiver (a tracked, intentional renewal, not a silent auto-renewal).

**Acceptance:** As part of the P97 milestone-close process, either (a) complete or schedule the work in `GOOD-TO-HAVES-15` to bring overaged files under their budget before 2026-08-08, OR (b) update the waiver row in `quality/catalogs/structure.json` with a documented reason and a new expiry date (ensuring the reason and date appear in the phase that renewed it). Do not let the waiver silently start blocking pushes mid-milestone.

**Why deferred:** the actual file-size splits are scoped to `GOOD-TO-HAVES-15` (M-sized real work); this entry is a hygiene reminder to ensure the waiver is consciously renewed or the work is scheduled before the date.

**Default disposition:** LOW (waiver-management) — fold into the P97 milestone-close checklist and governance review. The actual file splits remain M-sized and default-defer per `GOOD-TO-HAVES-15`.

**STATUS:** OPEN | 2026-07-05 debt-drain triage: LEFT AS-IS. This is a P97 milestone-close governance action (renew waiver or complete GTH-15 splits); no debt-drain window action taken. Flagged that it remains a hard, non-skippable P97 gate item ahead of the 2026-08-08 expiry.

## 2026-07-05 | Malformed `last_fetched_at` cursor bricks the fetch leg but only warns the push leg (inconsistent degradation) | discovered-by: P93 Exec1 (noticed while building the D-P92-03 repro) | severity: LOW

**What:** A corrupted/malformed `meta.last_fetched_at` value degrades two ways depending on which path reads it. `Cache::read_last_fetched_at` (`crates/reposix-cache/src/cache.rs:521-540`, used by the PUSH precheck) TOLERATES it — logs a `WARN` (`cache.last_fetched_at malformed: …; falling back to first-push semantics`) and returns `None`, so the push still works. But `Cache::sync` (`crates/reposix-cache/src/builder.rs:235-238`, the FETCH leg) parses the same value with `.map_err(Error::Sqlite)` and HARD-ERRORS, aborting the whole `git pull`/`git fetch` with `git-remote-reposix: cache.sync before upload-pack tunnel: sqlite: bad last_fetched_at …: premature end of input`. Observed live while pinning the cursor during the D-P92-03 repro (a `.999999999+00:00` value with no date parsed fine as a `chrono` fixed-offset but tripped the seed-vs-delta parse asymmetry).

**Why it matters:** the cache is committed-artifact state (OP-4 "no hidden state"), but nothing guarantees the cursor is never corrupted (partial write, manual poke, future migration). A corrupted cursor should degrade the SAME way in both paths — fall back to the seed / first-push full sync — not leave `push` working while `fetch`/`pull` is bricked with a raw sqlite error the agent can't act on. It's also a teaching-free error message (violates OD-3 ownership item 2).

**Sketched resolution:** make `sync()`'s Step-1 cursor parse mirror `read_last_fetched_at`'s tolerance — on parse failure, `WARN` + treat as absent (fall through to the `build_from` seed path) instead of `?`-propagating `Error::Sqlite`. Add a unit test in `delta_sync.rs` planting a garbage cursor and asserting `sync()` recovers via the seed path rather than erroring. Small, self-contained; NOT part of the coordinator-gated delta-sync coherence fix.

**Default disposition:** LOW — real but narrow (requires an already-corrupted cursor); good first-issue-sized robustness + error-message-teaching fix. Candidate for a P93 wave or the OP-8 absorption slots.

**STATUS:** OPEN

---

