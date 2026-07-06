# 93-RELIEF-HANDOFF.md — P93 coordinator relief handoff, 2026-07-05

Written by the outgoing P93 coordinator, relieved at a wave boundary (~50%+ context
used) per ORCHESTRATION.md §3. **Phase is NOT closed and must NOT be pushed or
verified yet** — a fresh DP-2 re-review REOPENED the phase with a new HIGH (#5) after
the correctness implementation had already landed and self-tested GREEN on the sim.

**Required reading order for the successor:** (1) this file in full; (2) §4 below
(the blocker) before touching anything; (3) `.planning/CONSULT-DECISIONS.md` entries
`D-P93-01` and `D-P93-02` (lines ~173–291) for the ghost-`oid_map`-row fix history
this phase already shipped; (4) `docs/decisions/010-l2-l3-cache-coherence.md`
(ADR-010, Status: ACCEPTED — implement, do not re-litigate); (5)
`.planning/phases/93-cache-coherence/93-DP2-REPRO-NOTES.md` for the (separate, already
CONFIRMED and fixed) D-P92-03 delta-sync bug context.

**Do-not-touch guardrails:**
- Do **not** `git push origin main`, un-waive any P93 catalog row, or dispatch the
  phase-close verifier until HIGH #5 (§4) is resolved and re-reviewed clean.
- Do **not** self-decide an E2-class fix for #5 (see §4 decision tree) — package and
  escalate to L0 if the fix requires a connector-contract change or reverting
  `272882c`.
- Do **not** rebase/merge `origin/main`'s divergent commit into this branch without
  understanding §1's note on it first (it's owner-staged housekeeping, not a conflict
  with this phase's work, but the successor must reconcile it before the close-push).

## 1. Ground truth (git)

Captured directly (`git rev-parse HEAD`, `git status`, `git rev-list --left-right
--count origin/main...HEAD`, `git log --oneline`) at handoff time — **not** taken on
faith from any prior report:

- **HEAD:** `f72771c851ce5ed0de99cd02dbd9ca0b31804a3b` (`f72771c`)
- **Tree:** clean (`git status` → "nothing to commit, working tree clean")
- **Branch:** `main`, diverged from `origin/main`: **20 ahead, 1 behind** (a briefing
  I received said "~18 ahead" — the verified count is **20**; use 20).
- **The 1 commit only on `origin/main`:** `5118ed1` "chore(deps): Bump
  codecov/codecov-action from 6 to 7 (#62)" — this is exactly the PR #62 merge that
  `c7138e9` (below) staged as an owner proposal; it appears the owner (or an external
  actor) merged it directly on GitHub. This is **known, expected divergence**, not
  breakage — but it means a plain `git push` will be rejected ("fetch first") and the
  close-push step needs a `git pull --rebase origin main` (or equivalent merge) first.
  Verify `merge-base HEAD origin/main` == `2df6603` before assuming anything else
  diverged.

**Per-commit one-liners, HEAD back to the merge-base `2df6603` (20 commits, newest
first):**

| SHA | Message |
|---|---|
| `f72771c` | docs(93): ratify D-P93-02 (Strategy 1 prune) + file Strategy 2 defense-in-depth |
| `272882c` | fix(93): prune ghost oid_map rows on sync — kill false SotPartialFail (D-P93-02) **— UNSAFE per HIGH #5, see §4** |
| `9a7c57b` | docs(93): ledger D-P93-01 — deleted-record ghost oid_map row CONFIRMED (DP-2) |
| `0b20c6c` | test(93): repro deleted-record ghost oid_map row forces false SotPartialFail (RED, ignored) |
| `fa9819f` | test(93): add partial_failure_recovery_real_confluence smoke (RBF-LR-03) |
| `819add6` | docs(93): qualify the mirror-head refresh promise honestly (RBF-LR-04) |
| `d5df26f` | docs(93): fix stale L1/L2/L3 troubleshooting cross-reference (ADR-010) |
| `6058c71` | docs(93): reframe L1 no-op-push skip as semantic, not coherence, no-op |
| `51567ae` | fix(93): reposix sync --reconcile forces a full build_from rebuild |
| `dfec0c9` | docs(93): drop out-of-docs_dir markdown link to DP2 repro notes |
| `fe2b726` | feat(93): SoT partial-fail OP-3 audit + PRECHECK-B recovery test (RBF-LR-03) |
| `299ade0` | fix(93): restore tree↔oid_map coherence in Cache::sync (ADR-010/RBF-LR-01) |
| `c7138e9` | docs(intake): stage branch-deletion + PR#62 merge proposal for owner (2026-07-05 debt-drain) |
| `846df93` | docs(intake): record 2026-07-05 debt-drain triage dispositions (surprises + good-to-haves) |
| `ae93cfb` | docs(security-gates): refresh stale "not executed" caveats in gate script headers (P92 GTH) |
| `f9beb8f` | docs(grounding): trim doctrine text under file-size budget + file runner-type good-to-have |
| `7ebc2c6` | docs(grounding): map coordinator role names to registered subagent_types |
| `5f5078a` | docs(93): ratify ADR-010 ACCEPTED (reversible internal strategy, no E2 escalation) |
| `543bfb4` | test(93): waive catalog-first P93 cache-coherence rows pending verifier+implementation |
| `4114448` | docs(93): ADR-010 L2/L3 cache-coherence + SotPartialFail recovery (PROPOSED) |

(merge-base / bottom of stack: `2df6603` docs(93): adjudicate D-P92-03
REPRODUCED/CONFIRMED — this and everything below it is already on `origin/main`.)

**Deviations the successor must know:**
1. Count is 20-ahead/1-behind, not 18-ahead as I was briefed — verified directly, use
   this file's number.
2. `origin/main`'s one extra commit is inert w.r.t. P93 (a dependabot bump merge), but
   it means the close-push is a `pull --rebase` + push, not a fast-forward push.
3. I independently verified (by reading the code, not by re-running the DP-2 review)
   that HIGH #5 (§4) is real: `crates/reposix-github/src/lib.rs` line 498
   (`MAX_ISSUES_PER_LIST = 500`) and line 502 (`MAX_RAW_ITEMS_PER_LIST`) both `return
   Ok(out)` / `break` on truncation with only a `tracing::warn!`, never an `Err` or a
   completeness flag on the returned `Vec<Record>`. `jira/lib.rs` L109-179 has the same
   shape (non-strict `list_issues_impl`). `crates/reposix-cache/src/builder.rs` line
   139 and line 442 both build `keep_ids` from that same possibly-truncated
   `list_records()` result and feed it straight into `meta::prune_oid_map`'s DELETE.
   This is a genuine, code-confirmed hazard, not a false alarm.
4. I also checked the `structure/file-size-limits` waiver precisely: the current
   `quality/gates/structure/file-size-limits.sh` **excludes all `crates/**/*.rs` files
   from the gate entirely** (`EXCLUDED_PATTERNS` includes `^crates/.*\.rs$`, "DEFERRED
   to next milestone (bulk refactor)") — `builder.rs` (27056 bytes, over the notional
   20000-byte `.rs` budget) is therefore **not currently one of the 47 WAIVED
   violations** the gate reports; it's simply out of scope for this gate today. The
   briefing I received attributed the 2026-08-08 waiver to "builder.rs overage" —
   that's not what the catalog row's actual waiver reason says (it lists 10
   research-doc/`AGENTS.md` files, none of which is `builder.rs`). Don't let this
   confuse the P97 milestone-close waiver-drain work; it's a correct carry-forward
   item, just not for the reason stated.

## 2. Wave/cycle state

P93 has no formal `93-0N-PLAN.md` wave files (unlike P91's pattern) — work proceeded
as a ledger-driven sequence of lanes tracked in `.planning/CONSULT-DECISIONS.md`. Wave
table reconstructed from commits + the ledger:

| Lane | Scope | State | Commits |
|---|---|---|---|
| ADR-010 authoring | Author ADR-010 (PROPOSED) for the L2/L3 cache-coherence tradeoff + D-P92-03 delta-sync repro/adjudication + catalog-first waived rows | DONE | `4114448`, `543bfb4`, `9c46e49`, `2d0fdf7`, `2df6603` (below merge-base, already on origin) |
| Ratify ADR-010 | Coordinator ratifies ADR-010 PROPOSED→ACCEPTED (reversible internal strategy, no E2) | DONE | `5f5078a` |
| Lane 1 (fetch-side fix) | RBF-LR-01: restore tree↔oid_map coherence in `Cache::sync` (addition direction); SoT partial-fail OP-3 audit + PRECHECK-B recovery test (RBF-LR-03) | DONE | `299ade0`, `fe2b726` |
| Lane 2 (docs + CLI + real-smoke) | `reposix sync --reconcile` full rebuild; L1 no-op-push honesty reframe; stale cross-ref fix; RBF-LR-04 mirror-head promise qualification; real Confluence smoke (RBF-LR-03) | DONE | `51567ae`, `6058c71`, `d5df26f`, `819add6`, `fa9819f`, `dfec0c9` |
| DP-2 prove-before-fix (D-P93-01) | Execute a real sim-backed repro of the ghost-`oid_map`-row HIGH via the actual `git-remote-reposix export` path (not a unit shortcut) | DONE, CONFIRMED | `0b20c6c`, `9a7c57b` |
| DP-2 fix (D-P93-02) | Choose + ship Strategy 1 (prune `oid_map` on sync) over Strategy 2 (reclassify delete-NotFound); file Strategy 2 as deliberate GOOD-TO-HAVE deferral | DONE, self-verified GREEN on sim | `272882c`, `f72771c` |
| **DP-2 fresh re-review** | Independent re-review of the just-shipped fix | **REOPENED — new HIGH #5, code-read only, not yet executed** | none (finding not yet committed anywhere — see §4/§5) |
| Mechanical close wave | 5 gate scripts, un-waive 6 catalog rows, fix stale PROPOSED→ACCEPTED catalog comment, fix-twice `quality/CLAUDE.md`, push, verifier, STATE.md advance | **NOT STARTED — blocked on #5** | none |
| Parallel debt-drain (non-cargo firewalled lane) | Branch/PR triage, intake dispositions, security-gate header refresh, grounding fix-twice | DONE (finished before this handoff) | `c7138e9`, `846df93`, `ae93cfb`, `f9beb8f`, `7ebc2c6` |

No named-incident post-mortem this phase (unlike P91's Wave-5.5 sibling-process
incident) — the one "incident" worth flagging is the DP-2 REOPEN itself (§4), which is
process working as designed (prove-before-fix + fresh-review discipline catching a real
gap after the first fix's own tests all passed), not a mistake to root-cause.

## 3. Binding constraints (unchanged)

- **One tree-writer at a time.** No live sibling `claude` process confirmed as another
  P93 tree-writer at handoff; two `claude` processes were observed on the box (PIDs
  8917, 93936) — verify neither is a stray tree-writer sharing this cwd before granting
  any executor "sole writer" status (P91's Wave-5.5 incident is the cautionary
  precedent).
- **ONE cargo invocation machine-wide.** Prefer `-p <crate>`, `-j 2` / `CARGO_BUILD_JOBS=2`;
  never `--workspace` unless truly required; never run cargo concurrently with anything
  else touching the tree.
- **No `--no-verify`, no `-c commit.gpgsign=false`.**
- **Push cadence:** `git push origin main` happens BEFORE the verifier dispatch, not
  after — and only once #5 is resolved. The verifier grades RED if the phase shipped
  without the push landing, and RED if pushed before #5 is fixed (phase isn't done).
- **Commit trailer:** `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`
  (or the equivalent model tag for whichever model is executing) on every commit.
- **Model tiering:** fable → opus (complex/security) / sonnet (default) / haiku
  (mechanical). The #5 repro-and-decide lane is complex/security-adjacent (a real data-
  loss hazard against live upstream records) — opus-tier, not sonnet.
- **External mutations need owner-named-target approval** — the staged branch-deletion
  and PR #62 merge proposal in `SURPRISES-INTAKE.md` remain UNEXECUTED pending the
  owner; do not act on them from this phase.

## 4. Litmus / gate / REOPEN state — THE BLOCKER

**REOPEN: HIGH #5**, raised by a DP-2 fresh re-review of the just-shipped `272882c`
Strategy-1 prune fix. **This finding was relayed to me by the coordinator who briefed
this task; I independently re-verified the code-level claim myself (see §1 item 3) and
confirm it is real. I could not find a committed transcript or file for the re-review
itself ("afb0a0a7" was named as its session/reference id, but it is not a git object,
not a file in this repo, and not a string appearing anywhere in the tree) — treat the
underlying CODE FINDING as verified-by-me, but the re-review session's own transcript
is NOT a committed artifact and does not exist on disk. The successor's first repro
lane should produce one.**

**The finding, precisely:** `272882c`'s `meta::prune_oid_map` DELETEs `oid_map` rows
whose `issue_id` is absent from a `keep_ids` set built from `self.backend
.list_records(&self.project)`. But `list_records()` on the GitHub, JIRA, and Confluence
connectors can silently return a **truncated** `Ok(partial_list)` at pagination/size
caps (`github/lib.rs` `MAX_ISSUES_PER_LIST=500` / `MAX_RAW_ITEMS_PER_LIST`, `jira/lib.rs`
non-strict `list_issues_impl`, Confluence equivalent) — the caller has no way to
distinguish "the project only has 40 records" from "the project has 4000 records and we
truncated at 500." Feeding a truncated `keep_ids` set into the prune's DELETE wipes
`oid_map` rows for **live records that exist beyond the cap**, reintroducing exactly the
coherence break the fix was meant to close (a live record now looks ghost/deleted to
`list_record_ids()`) — and it recurs on every sync, not once. **The sim backend (the
project default, per OP-1) never truncates**, so every sim-run gate — including all of
Lanes 1/2/DP-2's own GREEN test runs — is structurally blind to this. Before `272882c`,
a truncated list only under-populated the working tree (an accepted, documented HARD-02
tradeoff); the prune fix turned that same truncation into an active data-loss operation.
**No existing test guards this** (all P93 tests are sim-backed with small record
counts).

**Gate/catalog state (verified via `jq` against `quality/catalogs/agent-ux.json`):**

| Catalog row | RBF-LR id | Status | Waiver until |
|---|---|---|---|
| `agent-ux/p93-l2-l3-coherence-adr` | RBF-LR-01 | WAIVED | 2026-08-05 |
| `agent-ux/p93-cache-coherence-refresh-honest` | RBF-LR-02 | WAIVED | 2026-08-05 |
| `agent-ux/p93-delta-sync-coherence-invariant` | D-P92-03 gate | WAIVED | 2026-08-05 |
| `agent-ux/p93-partial-failure-recovery-real-confluence` | RBF-LR-03 | NOT-VERIFIED | n/a (env-gated, expected without creds) |
| `agent-ux/p93-l1-promise-reconciled` | RBF-LR-04 | NOT-VERIFIED | n/a |
| `agent-ux/p93-mid-stream-litmus-t1-t4` | RBF-LR-05 | NOT-VERIFIED | n/a |

None of these 6 rows may be un-waived/flipped, and **none of the 5 planned gate
scripts** (`p93-l2-l3-coherence-adr.sh`, `p93-cache-coherence.sh`,
`p93-delta-sync-coherence.sh`, `p93-partial-failure-recovery-real-confluence.sh`,
`p93-l1-promise-reconciled.sh`) exist yet on disk (confirmed: `ls
quality/gates/agent-ux/` has none of these names) — all mechanical-close work is
blocked on #5, not merely pending.

**Also confirmed stale:** `quality/catalogs/agent-ux.json` line ~1801 (the
`p93-l2-l3-coherence-adr` row's `comment` field) still reads "...coordinator ratifies
PROPOSED->ACCEPTED" even though ADR-010 was ratified ACCEPTED three commits later
(`5f5078a`) — needs a wording fix in the mechanical close wave (§6).

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

**Not yet formalized:**
- HIGH #5 itself has **no `D-P93-0N` ledger entry yet** in `CONSULT-DECISIONS.md` — the
  successor's first action (§6 step 1) should open one once the repro executes, mirroring
  the D-P93-01/02 CONFIRMED-then-fix pattern.
- The E2-escalation valve for #5 (package a fable-consult vs. fix under DP-2) is a live
  decision NOT yet made — see §6 step 2 for the decision tree exactly as briefed to me.

**Noticed, not yet filed to `SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md`** (both live at
`.planning/milestones/v0.13.0-phases/`; I checked both files directly — neither of the
following three appears in them yet):
1. **Runner side-effect hazard (repeats across lanes).** `quality/runners/run.py` (the
   `verdict.py`/walker path) mutates committed `quality/catalogs/*.json` +
   `quality/reports/` IN PLACE as a side effect of grading, and can mint FAIL rows
   without a `minted_at` field. Both Lane 1 and Lane 2 of this phase had to
   `git checkout HEAD -- quality/catalogs quality/reports` to discard transient drift
   before committing their real work. This is the same class of issue the P91 handover
   noted informally ("`run.py` in-place catalog mutation with no `--dry-run` guard") but
   it has never been filed as its own intake row with severity — do that.
   **RESOLVED P96 (D-P96-01):** filed + fixed. `run.py` now splits GRADE from
   PERSIST — a bare cadence run is validate-only and never writes
   `quality/catalogs/`; only `--persist` mints. The
   `git checkout HEAD -- quality/catalogs quality/reports` workaround is retired
   (regression-locked by `structure/catalog-immutable-on-read`). Do NOT re-apply it.
2. **`mkdocs-strict.sh` swallows broken-anchor findings.** `--strict` logs broken-anchor
   problems at INFO level, which is invisible in normal gate output; Lane 2 hit this
   firsthand while fixing the stale L1/L2/L3 cross-reference (`d5df26f`) and had to look
   harder than the gate's own exit code to confirm the fix. Not filed.
3. **Confluence `Record::labels` isn't wired to real labels.** `docs/reference/testing-targets.md`'s
   documented `kind=test` cleanup convention is a no-op for Confluence-created test
   fixtures because the connector doesn't actually set/read labels yet. Not filed.

Strategy 2 (reclassify delete-time `NotFound` as idempotent success — the defense-in-depth
alternative NOT chosen for D-P93-02) **is already correctly filed** in
`GOOD-TO-HAVES.md` (bottom of file, "2026-07-05 | Strategy 2 (defense-in-depth)...") —
no action needed there.

## 6. Precise next steps (successor runbook)

1. **PROVE #5 before fixing anything** (DP-2 discipline, same as D-P93-01). Build a
   mock/capped-backend test (or a connector-level test using a small
   `MAX_ISSUES_PER_LIST`-style constant, or a test double implementing
   `BackendConnector` that truncates) that demonstrates: (a) `list_records()` returns a
   partial list at a cap, and (b) `Cache::sync`/`build_from`'s prune subsequently DELETEs
   an `oid_map` row for a record that is still live upstream (beyond the cap), i.e. a
   real coherence break / phantom-Delete reproduction — not just a code-read assertion.
   In the same lane, answer: **does any connector already expose a completeness signal**
   (a `has_more`/`is_complete` flag, or a paginate-all variant) that a truncation-safe fix
   could key off? Write this up as a new `D-P93-0N` entry in `CONSULT-DECISIONS.md`
   (CONFIRMED via execution, per the D-P93-01 template) once it executes.
2. **Decide the fix strategy** (per DP-2 + the escalation valve — do not self-decide an
   E2):
   - **(a) INTERNAL/reversible path** — if a truncation-safe fix is achievable without a
     connector-contract change (e.g., a completeness flag already exists or is trivially
     derivable, or the prune can be gated to only run when the listing is known-complete,
     preserving the original HARD-02 under-populate tradeoff on truncation instead of
     wiping live rows) → fix it under DP-2 discipline, get a fresh re-review, proceed to
     step 3.
   - **(b) E2 path** — if the fix requires either a **connector-contract change** (adding
     a truncation/completeness signal to `BackendConnector::list_records`'s return type
     or a sibling method) OR a **load-bearing write-loop change** (reverting `272882c`
     back to Strategy 2's idempotent delete-NotFound reclassification) → this is an
     architectural/API-surface decision. Package a fable-consult with: the EXECUTED
     repro from step 1, the binding constraints (ADR-010's coherence charter; the HARD-02
     truncation tradeoff it's built on), and at minimum these four options for the
     consult to choose among: **A** — add a completeness signal to the `list_records`
     contract and gate the prune on it; **B** — revert `272882c`, ship Strategy 2
     instead (accepting its previously-documented downsides, see D-P93-02's rationale
     for why it was NOT chosen the first time); **C** — restrict the prune to a
     dedicated full-paginated reconcile path (e.g. `reposix sync --reconcile`, which
     already forces a full rebuild per `51567ae`) and skip pruning on the normal delta
     path entirely; **D** — a cap-count heuristic that skips the prune when
     `keep_ids.len()` is suspiciously close to a known cap. **Escalate this package to
     L0 (top-level) — do not self-decide.**
3. **Only after #5 is fixed GREEN and re-reviewed clean**, run the mechanical close wave
   (slice into sub-100-call lanes):
   - **Lane 3 (gates + catalog hygiene):** create the 5 gate scripts under
     `quality/gates/agent-ux/`: `p93-l2-l3-coherence-adr.sh`; `p93-cache-coherence.sh`
     (must run BOTH `cargo test -p reposix-cache --test cache_coherence` AND `-p
     reposix-remote --test partial_failure_recovery`, PLUS the now-un-ignored
     `deleted_record_ghost_oid_map_row_forces_false_partial_fail`); `p93-delta-sync-
     coherence.sh`; `p93-partial-failure-recovery-real-confluence.sh` (env-gated, OD-2
     semantics — NOT-VERIFIED not FAIL without creds); `p93-l1-promise-reconciled.sh`.
     Un-waive the 6 rows in `quality/catalogs/agent-ux.json` listed in §4. Fix the stale
     "PROPOSED" wording at `agent-ux.json` ~line 1801 (§4). Fix-twice `quality/CLAUDE.md`
     to list these 5 new gates in the agent-ux dimension row. Then `git pull --rebase
     origin main` (to absorb `5118ed1`, §1) and `git push origin main` **BEFORE** the
     verifier.
   - **Intake filings:** file the three noticed-not-filed items from §5 into
     `SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md` with severity + sketch (do not silently
     skip, per OP-8/OD-3 item 3).
   - **Verifier dispatch (`gsd-verifier`, zero session context):** grade RBF-LR-01..05 +
     the D-P92-03 gate row → `quality/reports/verdicts/p93/VERDICT.md`. Charter the
     verifier explicitly with: RBF-LR-03 and RBF-LR-05's TokenWorld arm grade
     NOT-VERIFIED without creds (expected, OD-2 env-gated, never FAIL); the "recovery
     push 2" scenario models the post-`git pull --rebase` tree; the pre-existing
     cross-link-audit phase-58 FAIL is explicitly OUT of P93 scope (do not let the
     verifier conflate it); the full fix history to check is D-P93-01/02 (ghost-row fix
     + its regression guard) **plus** whatever #5's fix commit(s) turn out to be.
   - **STATE.md** advances only on a GREEN verdict (currently still shows P93 as "next
     agent action: plan + execute" — stale relative to all the above work; do not
     advance it early).
4. **Final coordinator report** (once GREEN): verdict + path, full commit list from
   `2df6603..HEAD` (by then including #5's fix + the mechanical wave), the #5
   REOPEN-and-resolve narrative as a first-class finding (this IS the interesting story
   of this phase, not a footnote), intake items drained/filed, carry-forwards restated
   (git-2.43 fallback-sentinel → P94; TokenWorld two-writer verifier → P97; marker
   footgun → P95 — none of these are P93's to fix, just confirm they're still correctly
   routed), and the P97 waiver-drain reminder for `structure/file-size-limits`
   (2026-08-08) with the §1-item-4 correction (it's not about `builder.rs`; it's the 10
   research-doc/`AGENTS.md` files already named in the catalog row's own waiver reason).
