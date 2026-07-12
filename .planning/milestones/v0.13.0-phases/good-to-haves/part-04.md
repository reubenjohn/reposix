# v0.13.0 GOOD-TO-HAVES — Part 4 of 8

> Split from `GOOD-TO-HAVES.md` for the file-size gate (OP-8 drain). Index: `../GOOD-TO-HAVES.md`. Entries preserved verbatim.

## 2026-07-05 | Consider a dedicated read-only "runner" subagent type distinct from `gsd-executor` | discovered-by: grounding-bug fix (coordinator dispatched `subagent_type: "executor"`, got "Agent type not found") | severity: LOW

**What:** ORCHESTRATION.md's coordinator rule 1 names four dispatch roles — reader, runner (build/test/litmus), executor (file write/edit), reviewer (diffs) — but only three of those roles have a dedicated registered `subagent_type` (`reader-digester`, `gsd-executor`, `gsd-code-reviewer`). "runner" has no dedicated type; today it's covered by overloading `gsd-executor` for routine build/test runs during implementation, and `gsd-verifier` for phase-close gate/litmus grading (`quality/PROTOCOL.md` Step 6/7). This was resolved as a DOCS fix (canonical role→subagent_type table added to `.claude/skills/coordinator-dispatch/SKILL.md` §2) rather than a new agent def, per the "don't invent, file it" rule.

**Why it might be worth a real type:** `gsd-executor` has `Edit`+`Write` tools. A coordinator that wants an arms-length "did the tests actually pass" signal — distinct from "the same agent that could edit the code to make it look like it passed" — currently has no `Bash`+`Read`-only, no-`Edit` runner to dispatch for that purpose. `gsd-verifier` fills this need for phase-close grading (it's genuinely a fresh, unbiased dispatch per `quality/PROTOCOL.md`), but mid-execution "run the test suite and report" (e.g. a `cargo nextest run -p <crate>` sanity check between plan waves) has no read-only equivalent — a coordinator either does it itself (violates ROUTE, DON'T WORK) or dispatches a full `gsd-executor` (which can silently patch a failing assertion instead of just reporting it failed).

**Sketched resolution:** a new `.claude/agents/gsd-runner.md`-style def (or reuse `reader-digester`'s tool set: `Read, Grep, Glob, Bash`, no `Edit`/`Write`) scoped to "run a named build/test/litmus command, report pass/fail + tail of output, never touch files." Small, mechanical, haiku-tier.

**Default disposition:** LOW — current `gsd-executor`/`gsd-verifier` split already covers correctness (verifier is the ungameable grading step); this is a defense-in-depth / mid-execution-hygiene nice-to-have, not a gap that blocks anything today. Owner-gated spend — do not create without explicit approval.

**STATUS:** OPEN

---

## 2026-07-05 debt-drain triage

The docs-only, <1h-sized GTH orphans (GOOD-TO-HAVES-02, -05, -06, -07, -09, -10, -12, -13; and the 2026-07-05 test-name-honesty marker entry above) were reviewed this window and LEFT to their already-routed phases (P90/P91/P95/refresh) — each either touches a linter/runner needing a real test run (cargo-adjacent) or a `docs/**` file under the mkdocs + doc-alignment regime routed to P95, so none were safe to eager-fix in a no-cargo, no-docs-alignment debt-drain window. See the companion `SURPRISES-INTAKE.md` for this same window's disposition of the surprises backlog and the branch-hygiene/PR-triage housekeeping entry.

---

## 2026-07-05 | `.git/hooks/pre-push` is a dead symlink to a nonexistent target | discovered-by: 2026-07-05 debt-drain branch-hygiene triage | severity: LOW

**What:** `.git/hooks/pre-push` is a symlink to `../../scripts/hooks/pre-push`, which does not exist (`ls` on the target errors ENOENT). It is currently INERT because `core.hooksPath` is set to `.githooks`, so the real active pre-push hook is `.githooks/pre-push` — the dead symlink never fires. Cosmetic debt only: no functional impact today, but a confusing artifact for anyone inspecting `.git/hooks/` directly (e.g. `git config --unset core.hooksPath` would silently resurrect a hook pointing at nothing).

**Acceptance:** Delete the dead `.git/hooks/pre-push` symlink (or replace it with a thin delegator to `.githooks/pre-push` if defense-in-depth against a future `core.hooksPath` unset is wanted).

**Why deferred:** `.git/` is not tracked by git itself (deleting a file there isn't a normal commit), so this needs either a `scripts/install-hooks.sh` update (if that script created the dangling symlink) or a one-off local cleanup step, not a tree-writer commit. Out of scope for a docs-only debt-drain window.

**Default disposition:** LOW — tidiness only, zero functional impact while `core.hooksPath=.githooks` remains set. Fold into the next `scripts/install-hooks.sh` touch or P95/P97 housekeeping pass.

**STATUS:** OPEN

---

## 2026-07-05 | Strategy 2 (defense-in-depth): reclassify delete-time `NotFound` as idempotent success | discovered-by: P93 DP-2 FIX lane (D-P93-02) | severity: LOW

**Deliberate deferral, NOT an oversight.** This is the SECOND candidate fix for the D-P93-01 ghost-`oid_map`-row HIGH. The coordinator chose **Strategy 1 (prune `oid_map` on sync)** as the shipped root-cause fix (commit `272882c`, ledger `D-P93-02`); Strategy 2 is filed here as a considered, independent defense-in-depth layer that was intentionally NOT taken this lane.

**What:** In `git-remote-reposix`'s write path, `execute_action`'s `PlannedAction::Delete` arm currently treats an `Error::NotFound` from `BackendConnector::delete_or_close` as a FAILURE (feeding `write_loop`'s `failed_ids` → `SotPartialFail`). Strategy 2 would reclassify it as idempotent success: a `Delete` whose target is already absent has reached its desired end state, so it should count as a no-op success, not a partial failure.

**Why it was NOT chosen as the primary fix (see D-P93-02 rationale):** Strategy 2 masks the symptom rather than fixing the root cause — the ghost `oid_map` row would still survive, `Cache::list_record_ids()` would still resurrect the dead id, and the planner would still emit + dispatch a phantom `DELETE` to the SoT on every push (wasted request + audit noise; a latent hazard against a real backend with soft-delete/restore semantics). It also broadens `SotPartialFail` semantics generally: ANY future NotFound-on-delete would silently reclassify to success, masking genuine "the record I meant to delete isn't there" bugs. Strategy 1 leaves the cache coherent and emits zero phantom Deletes.

**Why it's still worth having (defense-in-depth):** even with Strategy 1 shipped, a genuine two-writer delete race exists — agent A pushes a `Delete` for issue N while agent B's push (or an upstream actor) already removed N between A's precheck and A's write. That is NOT a ghost row; it's a real concurrent-delete collision that Strategy 1 does not address, and today it would surface as a `SotPartialFail` for a delete that actually achieved its goal. Strategy 2 would make that race degrade gracefully.

**Sketched resolution (~10-20 lines + a test):** in `execute_action`'s `Delete` arm, map `Err(Error::NotFound)` (from `delete_or_close`) to the same success outcome as a 2xx delete, with a `WARN`-level audit note distinguishing "deleted" from "already-absent". Add an integration test (mirror the `partial_failure_recovery.rs` wiremock harness) asserting a `Delete` against an already-404 id yields `ok refs/heads/main`, NOT `SotPartialFail`. Confirm the reclassification is scoped to the `Delete` arm only (a `NotFound` on Create/Update remains a real failure).

**Default disposition:** LOW — real but narrow (requires a genuine concurrent-delete race now that the ghost-row root cause is fixed); a good defense-in-depth follow-up. Default-defer to a v0.14.0 push-flow-robustness or the OP-8 absorption slots. Reversible (one match arm).

**STATUS:** OPEN (deliberate deferral — Strategy 1 shipped as the primary fix)

---

## 2026-07-05 | Intake files don't name meta-infra (orchestration/agents/skills/hooks/runner-infra/coordinator-discipline) as in-scope | discovered-by: P93 Wave 1 de-risk executor | severity: LOW (deferred tangent)

**What:** `SURPRISES-INTAKE.md` and `GOOD-TO-HAVES.md`'s own framing describes what
phases discover in code/docs/catalogs, but neither file's scope language explicitly
names the ORCHESTRATION/agent-definition/skill/hook/runner-infra/coordinator-discipline
layer as fair game for an intake entry — a finding about, say, a hook footgun or a
coordinator-discipline gap could plausibly be read as "out of these files' scope" by a
literal reader, even though such findings are exactly the kind of thing this project's
dark-factory doctrine wants surfaced (cf. CLAUDE.md OP-4's "self-improving infrastructure"
and the OD-3 meta-rule "fix it twice"). A 4-edit proposal to close this gap has already
been drafted: (i) a scope note in `PRACTICES.md`, (ii) an addition to `ORCHESTRATION.md`
§5, (iii) a new `decision-procedures` entry DP-5, and (iv) a fix-it-twice cross-reference
in root `CLAUDE.md`.

**Acceptance:** Land the 4-edit proposal (PRACTICES.md scope note; ORCHESTRATION.md §5;
decision-procedures DP-5; root CLAUDE.md fix-it-twice pointer) as a tracked `/gsd-quick`,
scheduled AFTER the v0.13.0 tag lands (P97 GREEN) — NOT during the active P92→P97 close-out
drive, to avoid touching orchestration doctrine mid-milestone-close.

**Why deferred:** this is itself a meta-infra/tangent proposal (per root CLAUDE.md
Operating Principle 4, "high-leverage tangents are first-class to propose... the owner
gates the spend, never the surfacing") — surfacing it now, landing it later, post-tag,
keeps the active close-out drive's blast radius small.

**Default disposition:** LOW/deferred-tangent — land as a `/gsd-quick` AFTER the
v0.13.0 tag, not during this drive.

**STATUS:** OPEN

---

## 2026-07-05 | `.claude/hooks/dispatch-doctrine.sh` re-fires its full text on EVERY Agent dispatch with no session-scoped guard | discovered-by: P93 Wave 1 de-risk executor | severity: LOW (cheap fix)

**What:** `.claude/hooks/dispatch-doctrine.sh` fires its full doctrine text on every
single `Agent`/subagent dispatch within a session, with no "already applied this
session" marker to suppress repeats. This is the root cause of the dispatch-doctrine
reminder text re-appearing on every dispatch observed across recent sessions — it's
working as coded, just coded to repeat unconditionally.

**Acceptance:** Add a session-scoped marker file (e.g. under the session's scratch/temp
dir, or keyed off a session-id env var already available to hooks) that the hook checks
before firing; once set, subsequent dispatches within the same session skip the full
text (a one-line "doctrine already applied this session" ack is fine, or full silence).
Verify: dispatch two Agents in the same session, confirm the doctrine text fires once
and is suppressed (or abbreviated) on the second.

**Why deferred:** small (~5-15 lines shell), but touches `.claude/hooks/` — a
tooling/infra surface outside this wave's `.planning/` ledger + push-derisk scope, and
warrants its own tiny verification pass (confirm session-marker semantics don't leak
across genuinely separate sessions) rather than a rushed inline edit here.

**Default disposition:** cheap fix — pick up in the P94–P97 debt window or the next
`.claude/hooks/`-touching phase.

**STATUS:** OPEN

---

## 2026-07-05 | Confluence connector's `Record::labels` is not wired to real Confluence labels | discovered-by: P93 Wave 2a executor | severity: P2

**What:** The Confluence `BackendConnector` implementation does not populate
`Record::labels` from the real Confluence page label API — `Record::labels` reads back
empty/default regardless of what labels the page actually carries in Confluence Cloud.
Feature gap, not a correctness regression (nothing currently reads `Record::labels` for
Confluence records downstream), but the field silently lies about page state today.

**Acceptance:** Map the page's label API response (Confluence Cloud REST `GET
/wiki/rest/api/content/{id}/label` or equivalent) into `Record::labels` in the confluence
adapter (`crates/reposix-confluence/src/lib.rs`), for both the read path
(`get_record`/`list_records`) and any write-path round-trip. Add a contract test
(mirroring the existing `contract.rs` pattern) asserting a page with a real label
round-trips through `Record::labels`.

**Why deferred:** feature-gap noticed while filing Wave 2a's tracked items — wiring the
label API is real connector work (new REST call + response mapping + a contract test
against the live tenant) deserving its own scoped task, not a rider on the push-unblock
lane.

**Default disposition:** P2 — fold into the next `reposix-confluence`-touching phase or
a v0.14.0 connector-completeness window.

**STATUS:** OPEN

---

## 2026-07-05 | Sim same-second `list_changed_since` under-report (defense-in-depth precision fix) | discovered-by: P93 Wave 2b executor | severity: P3

**What:** The D-P92-03 *trigger* — the sim (and every second-resolution backend) drops
same-wall-clock-second writes from `list_changed_since` — is still live. The *amplifier*
(the load-bearing cache-coherence invariant break) is CLOSED and verified GREEN this wave:
`Cache::sync` Step 5 now upserts `oid_map` for the full `list_records` set (`e2a7297`), so
a dropped delta no longer produces an unservable OID. ADR-010's sub-decision ratified the
sim-precision fix as **optional defense-in-depth, explicitly NOT load-bearing for
coherence** ("a precision fix alone would NOT fix the amplifier and MUST NOT ship as the
sole remedy"). It is filed here rather than shipped in the Wave-2b minimal-fix window
because (a) it is optional per the ratified ADR, (b) it does NOT help the real backends —
Confluence/JIRA/GitHub `updated_at` are inherently second-resolution, so a sim-only
precision fix cannot close the trigger there (the committed invariant fix already does),
and (c) done correctly it is a multi-crate change with a wiremock-contract and
regression-test-comment ripple, past the OP-8 "<1h clean eager-fix" line.

**Benefit if done:** eliminates the *natural* same-second under-report on the sim (fewer
missed changes → more eager pre-materialization on the delta path) and de-risks OTHER
`list_changed_since` consumers, notably the push precheck's changed-set. Pure efficiency /
belt-and-suspenders; the coherence guarantee does not depend on it.

**Acceptance (the "sub-second precision on `updated_at`" option — minimal correct):**

- `crates/reposix-sim/src/routes/issues.rs:139` `now_rfc3339()` → `SecondsFormat::Nanos`
  (fixed-width, so SQLite lexicographic `>` stays monotonic).
- `crates/reposix-sim/src/routes/issues.rs:180` cutoff → `SecondsFormat::Nanos` to MATCH
  storage (mixed widths break lexicographic monotonicity — e.g. `10:08:10Z` sorts after
  `10:08:10.5…Z` because `'Z' > '.'`), and update the seconds-precision comment at
  `:175-178`.
- `crates/reposix-sim/src/seed.rs:99` seed timestamp → `SecondsFormat::Nanos` (keep ALL
  stored `updated_at`/`created_at` at one fixed width).
- `crates/reposix-core/src/backend/sim.rs:290` — the SimBackend CLIENT also truncates the
  OUTGOING cursor to `SecondsFormat::Secs`; it must send sub-second too, else the server
  cannot do better than seconds regardless of its own precision. THIS is why a
  server-only change would be a broken half-fix.
- Update the wiremock contract assertion at `crates/reposix-core/src/backend/sim.rs:1089`
  (`query_param("since", "2026-04-24T00:00:00Z")` → the Nanos rendering).
- Update the now-stale "the sim truncates the cursor to seconds" comments in
  `crates/reposix-cache/tests/delta_sync.rs` (~L497-498) and
  `crates/reposix-cache/tests/cache_coherence.rs` (~L303-305). The two coherence
  regressions SURVIVE this change unmodified in behavior: both pin the cursor to
  `upd.with_nanosecond(999_999_999)` (the max nanosecond of the write-second), so a
  full-precision compare still drops the write (`actual_ns < 999_999_999`) and
  `changed_ids.len() == 0` still holds — the invariant is then proven independent of
  timestamp resolution. Note: `>` with Nanos needs no de-dup (nanosecond wall-clock
  collisions across two HTTP writes are effectively impossible; the cursor is set to
  `Utc::now()` strictly after any observed write).
- Migration caveat: persistent sim DBs seeded by a pre-fix binary hold seconds-format
  rows; mixed widths would mis-sort. Acceptable — the sim DB is a rebuildable runtime
  artifact (`runtime/`), not a stability contract; re-seed on upgrade.

**Why deferred:** optional per ratified ADR-010; multi-crate (reposix-sim + reposix-core
client + a wiremock contract test) with no real-backend benefit; the Wave-2b charter
scoped a single-tree-writer minimal fix and the load-bearing coherence fix already shipped
and verified GREEN. Half-shipping (server-only) would be a lying "fix" that does not make
same-second writes visible.

**Default disposition:** P3 — pick up in the P94–P97 debt window or the next
`reposix-sim`-touching phase; re-dispatchable as its own scoped wave using the sketch above.

**STATUS:** OPEN

---

