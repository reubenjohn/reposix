# v0.13.0 GOOD-TO-HAVES — Part 6 of 8

> Split from `GOOD-TO-HAVES.md` for the file-size gate (OP-8 drain). Index: `../GOOD-TO-HAVES.md`. Entries preserved verbatim.

## 2026-07-06 | env-gate/`minted_at` load-fragility: an env-missing skip advances `last_verified` to run-time, making a `minted_at`-less post-P90 row unloadable once skipped | discovered-by: P97 milestone-close (OP-9 RETROSPECTIVE distillation, verify-against-reality on the 9th-probe mint) | severity: MEDIUM

**Size:** S (a `run.py` write-path change + regression test — the READ/skip-side mirror of the write-path load-refusal item already filed in `SURPRISES-INTAKE.md`).

**Source / mechanism:** The honesty contract says an env-gated skip "fails closed to `NOT-VERIFIED` but preserves `last_real_grade` + `skip_reason: env-missing`" (quality/CLAUDE.md § Honesty rules). But in practice the env-missing skip path ALSO stamps `last_verified` to the run-time clock. For a pre-P90 legacy row that carries no write-once `minted_at`, `_audit_field.validate_row`'s anchor heuristic is `is_new = lv is None or parse_rfc3339(lv) >= CUTOFF` — so once the skip advances `last_verified` past the 2026-07-05 cutoff, the row flips `is_new=True`, now demands a `claim_vs_assertion_audit`, and FAILs/refuses at the next load. A silent time-bomb: it does not bite on the run that moves the clock, it bites the run after. This is the exact landmine `09e10c1` closed for `code/cargo-clippy-warnings` and that `f37f468` closed for `agent-ux/cadence-pre-release-real-backend` (the milestone 9th-probe mint backfilled its missing `minted_at` = its P89 89-01 mint time `2026-07-04T03:08:07Z`, its `last_verified` having just been advanced by the env-gated skip). **Two instances fixed one-at-a-time; the CLASS remains** for any other pre-P90 row with null-or-pre-cutoff `last_verified` + no `minted_at` that gets re-graded (or env-skipped) in a blocking cadence.

**NOTE on the scaffold citation:** the P97 close task scaffold attributed the instance fix to `30b4910` — that commit is the honest file-size-waiver enumeration (10→50 files), NOT a `minted_at` fix. The real instance fix is `f37f468` (grounded against `git show`); `09e10c1` is the prior sibling instance. Recorded so the next reader does not chase the wrong SHA.

**Relationship (dedupe):** DISTINCT from — but the read-side complement of — `SURPRISES-INTAKE.md` § "2026-07-05 | `--persist` mint path should refuse to write a row it would reject at load" (that is the WRITE path: the mint refusing to persist a `minted_at`-less row). This item is the READ/env-skip path: the skip that CREATES the un-loadable state by advancing `last_verified`. Also distinct from the `run_row` stale-artifact freshness LOW above (that under-states freshness harmlessly; this one hard-refuses load).

**Acceptance:** the env-gated skip path must NOT advance `last_verified` on a row lacking `minted_at` (either preserve the prior `last_verified` alongside `last_real_grade`, or backfill a pinned `minted_at` at skip time from the row's genuine first-verification). Regression: a pre-P90 `minted_at`-less row, env-skipped in a blocking cadence, must still load on the NEXT run (no `SystemExit`/FAIL from a clock-advanced `last_verified`). The P95-designed exemption retirement (make `minted_at` unconditional across all rows) is the class-closing endgame; until then this hardens the skip path.

**Why deferred:** a `quality/runners/run.py` write-path change with its own test obligation — orthogonal to the no-cargo P97 milestone-close window; routes to the next `run.py`-touching quality-framework window.

**Default disposition:** DEFERRED-v0.14.0 (runner-hardening) `[-quality-framework]`.

**STATUS:** OPEN

---

## 2026-07-06 | ORCHESTRATION.md exceeds the 20k soft char-limit (~21.7k) | discovered-by: relief-threshold/C2 doctrine review | severity: LOW

**Size:** S (a progressive-disclosure doc split + pointer, one section relocated).

**Source / mechanism:** `[low]` ORCHESTRATION.md exceeds the 20k soft char-limit (~21.7k).
Progressive-disclosure split needed — candidate: move §11's L0–L4 tier table or §3's
C1/C2 detail to a linked doc, leaving pointers. Deferred from the relief-threshold/C2
doctrine review.

**Why deferred:** the review's charter was the ~50%→~100k relief-trigger sweep + C1/C2
legibility fixes (kept the file net-negative, not a structural split); a proper
progressive-disclosure split is its own change with its own pointer-integrity + promotion-
sweep obligations (§ Provenance "Promotion sweep" standing rule).

**Default disposition:** DEFERRED-v0.14.0 (doctrine-hygiene) — XS/LOW.

**STATUS:** OPEN

---

## 2026-07-06 | C2 (coordinator-of-coordinators) recursion is doctrine-only — never exercised | discovered-by: relief-threshold/C2 doctrine review | severity: MEDIUM

**Size:** S (an observation/instrumentation charter on the first two-tier milestone run — no code).

**Source / mechanism:** `[medium]` The C2 (coordinator-of-coordinators) recursion is
doctrine-only — NEVER exercised (first introduced `2b2736e`). On the first real C2 run,
verify: (i) a relieving C1's relief report routes to its parent C2, NOT L0 (post-dispatch-
relay cross-session addressing is historically flaky — see ORCHESTRATION §8); (ii) C2 and
C1 each relieve on their OWN ~100k line (no double-counting).

**Sketch:** instrument/observe the first milestone run under the two-tier model.

**Why deferred:** requires an actual multi-phase milestone run to exercise the two-tier
relief path; cannot be verified statically — no C2 has run since the doctrine landed.

**Default disposition:** DEFERRED-v0.14.0 (doctrine-validation) — MEDIUM.

**STATUS:** OPEN

---

## 2026-07-06 | `docs/guides/troubleshooting.md` is 25.5k chars, over the 20k progressive-disclosure soft limit | discovered-by: v0.13.0-intake-disposition sweep (260706-crf cold-reader carryover) | severity: MEDIUM

**Size:** M (>1h — a progressive-disclosure nav restructure that moves anchors other docs cross-link to).

**Source:** `docs/guides/troubleshooting.md` measures **25,503 chars** (verified 2026-07-06) — over the 20k `*.md` progressive-disclosure soft limit (root CLAUDE.md OP-4). Pre-existing debt that grew slightly this session via the 260706-crf DVCS cold-reader fixes. Root CLAUDE.md routes DVCS push/pull troubleshooting here, so the file accretes one symptom class after another with no natural cap.

**Acceptance:** split into a child page per DVCS symptom class (e.g. `troubleshooting/{push-conflicts,blob-limit,mirror-lag,sparse-checkout,...}.md`) behind a thin parent index, each under the 20k budget; update every cross-link anchor that currently targets `troubleshooting.md#<symptom>` (root CLAUDE.md § Pointer map, `docs/concepts/dvcs-topology.md`, `docs/guides/dvcs-mirror-setup.md`, ADR-010) so no fragment link rots. Confirm `quality/gates/docs-build/mkdocs-strict.sh` + the file-size gate pass post-split.

**Why >1h / deferred:** this restructures anchors that OTHER docs cross-link to, so it needs a coordinated anchor-rewrite sweep + a mkdocs walk to prove no broken fragment — not a mechanical trim, past the OP-8 "<1h clean eager-fix" line.

**Dedupe / relationship:** sibling of `GOOD-TO-HAVES-15`'s `troubleshooting.md — 22339/20000` line-item — GTH-15 tracks the RAW file-size overage across 9 files under the `structure/file-size-limits` waiver; THIS entry is the specific progressive-disclosure child-page split for this one doc. Do the split here; GTH-15's waiver-renewal accounting clears once it lands.

**Default disposition:** MEDIUM — fold into a v0.14.0 docs-progressive-disclosure or `docs-build`-touching window; pairs with GOOD-TO-HAVES-15.

**STATUS:** OPEN

---

## 2026-07-06 | `codecov/project` posts a phantom -16% release-blocker when the Rust lcov upload silently fails | discovered-by: PR #61 codecov triage lane | severity: MEDIUM

**Size:** S–M (an upload-robustness change to `.github/workflows/ci.yml` coverage job; touches CI, needs a re-run to prove).

**Source:** On CI run `28819166220` (PR #61, head `2d1f55f`) the `codecov/project` check went RED with title `68.60% (-16.46%) compared to f686ab2`. This is NOT a code regression. Evidence: (i) `codecov/patch` = SUCCESS ("all modified and coverable lines are covered by tests"); (ii) the coverage diff shows Files 130→45 (-85) and Lines 18933→1000 (-17933) — the entire Rust workspace report vanished from HEAD, not a real deletion; (iii) codecov's own banner: "HEAD has 1 upload less than BASE" with flag table `|1|0|` (the blank/default flag = the Rust `lcov.info` upload); (iv) the failing project % (68.60%) EXACTLY equals `codecov/project/shell` (68.60%) — codecov computed the default project status from the shell-only subset because the Rust report never landed. Root cause: the `coverage` job (`ci.yml:411-431`) uploads `lcov.info` with `fail_ci_if_error: false` (line 429), so a flaky/failed codecov upload leaves the job GREEN while dropping the Rust report from the merged HEAD coverage — codecov then compares a full BASE (Rust+shell) against a partial HEAD (shell only), manufacturing a phantom -16.46% that reads as a release blocker.

**Acceptance:** make the Rust coverage upload robust so a silent upload drop cannot produce a phantom project-drop that blocks release triage. Options (pick one, do NOT lower any threshold): (a) add codecov-action retry / `fail_ci_if_error: true` on the `coverage` job so an upload failure turns the job RED (honest, actionable) instead of silently poisoning the project comparison; (b) tag the Rust upload with an explicit `flags:` (e.g. `rust`) so a missing-flag report is detectable and carryforward keeps the last-good Rust numbers; (c) add codecov `after_n_builds` so the status only computes once all expected uploads (rust + shell) have arrived. Verify by re-running CI and confirming BASE and HEAD have equal upload counts and project % returns to ~85%.

**Why deferred / not eager-fixed here:** this lane's charter forbids editing CI/codecov gate config, and the correct fix needs a CI re-run to prove the upload lands equally on BASE and HEAD — cannot be verified statically. Per git log, CI was already re-triggered on the current head (`90db62c`); if that re-run lands a clean Rust upload the check clears on its own, but the underlying silent-drop fragility remains and will recur.

**Default disposition:** MEDIUM — fold into a v0.14.0 CI-robustness window; independent of code coverage quality (patch coverage is green).

**STATUS:** OPEN

---

## 2026-07-06 | Durable Confluence TokenWorld fixture page 7798785 has been accidentally trashed 2+ times, losing parent linkage to 7766017 each time | discovered-by: PR #61 CI-red repair lane | severity: LOW-MEDIUM

**Size:** S (investigation + a protection mechanism; not a code bug fix).

**Source:** During the PR #61 CI-unblock effort, the durable Confluence TokenWorld fixture page `7798785` was found with its `parentId` link to `7766017` broken (page effectively orphaned/trashed), causing a live CI red. It was repaired and verified live (parentId now `7766017`). Version history on the page shows this is NOT the first occurrence — the same parent-linkage loss happened at least once before, on 2026-07-04, and now again just prior to this repair. This is a recurring drift pattern on a fixture that other tests/gates depend on being durably present with intact hierarchy, not a one-off fluke.

**Acceptance:** investigate the root cause of the repeated trashing — candidates include (a) manual owner action in the TokenWorld space, (b) a test/cleanup routine that trashes or moves pages as a side effect (e.g. a contract test's teardown, or a stale cleanup script targeting the wrong page ID), or (c) a reposix operation (create/update/delete_or_close path) with a bug that mis-targets this page. Once root-caused, add protection: either Confluence page restrictions preventing accidental trash/move on durable fixture pages, and/or a periodic freshness check (CI or a `docs-repro`/`agent-ux` catalog row) that asserts the fixture's parent linkage is intact before it's relied upon, so a third occurrence surfaces as a clear, attributable signal instead of a mystery CI red.

**Why deferred:** root-causing requires investigation (Confluence audit trail / version history correlation with commit and workflow-run timestamps across at least two incidents) that is out of scope for the CI-unblock lane's charter (fix the immediate red, file the pattern); is not itself a code bug, and any protection mechanism (page restrictions or a new freshness gate) is new scoped work.

**Default disposition:** LOW-MEDIUM — advisory/non-blocking; fold into a v0.14.0 real-backend-fixture-hardening window or the next Confluence-connector-touching phase. Not release-blocking for v0.13.0.

**STATUS:** OPEN

---

## 2026-07-07 | release-plz branch regenerations silently drop `pull_request`-triggered workflows (CI, Security audit, quality gates) | discovered-by: v0.13.0 release CI investigation | severity: MEDIUM

**What:** Every time `release-plz` force-pushes a regeneration of its release branch (e.g. `release-plz-2026-07-07T02-37-20Z`), the `pull_request`-triggered workflows (`CI`, `Security audit`, `quality gates (pre-pr)`) do not automatically re-run against the new head — GitHub's `pull_request` trigger has repeatedly failed to fire cleanly after these force-push regenerations across recent release sessions. The current mitigation, needed 2-3 times per release in recent sessions, is a manual real-actor `gh pr close`/`gh pr reopen`, which forces a fresh `pull_request` event and re-triggers the three workflows.

**Why out-of-scope for eager-resolution:** diagnosing GitHub Actions trigger semantics precisely enough to build a reliable automated re-trigger (vs. the manual close/reopen toll) is real workflow-engineering work — requires testing across multiple regen cycles to confirm any fix actually closes the gap, not a one-line change discoverable mid-release-investigation.

**Sketched resolution:** consider adding a `workflow_dispatch` trigger keyed off release-plz branch pushes, or automate the close/reopen via a scheduled action, so the toll isn't a human/agent doing it manually each regeneration. Home: a CI/workflow-touching phase in v0.14.0 or a dedicated release-tooling window.

**Default disposition:** MEDIUM — process friction, not a correctness bug; observed recurring cost (2-3x per release cycle) makes it worth automating.

**STATUS:** OPEN

## 2026-07-07 | Manually merging `origin/main` onto a live release-plz branch races the bot's own regeneration and usually loses | discovered-by: v0.13.0 release CI investigation | severity: LOW-MEDIUM (process note)

**What:** Merging `origin/main` onto a live release-plz branch races the bot's own periodic regeneration and usually loses — the merge gets superseded before it can land, because release-plz will itself re-base off `main` on its next regen and pull in the same commits anyway. The manual merge is redundant work that also risks a confusing intermediate state (commits that appear to land, then vanish under the next force-push).

**Why out-of-scope for eager-resolution:** this is a process/runbook observation, not a code or tooling gap — nothing to fix in the codebase; the fix is documentation of the correct workflow.

**Sketched resolution:** the release runbook should note "don't manually merge onto the bot branch; land the fix on `main` and let the next regen absorb it" rather than treating a manual merge as the first move.

**Default disposition:** LOW-MEDIUM — cheap doc fix; fold into the next release-runbook touch.

**STATUS:** OPEN

