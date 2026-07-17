# 120-HANDOVER.md — P120 C1 phase-coordinator relief, 2026-07-17

Written by the outgoing P120 C1 phase-coordinator (dispatched by L0 seat #60), relieving
at a **W0–W5 all-implementation-complete / code-review-not-yet-dispatched** boundary. All
three phase success criteria are GREEN in the working tree, but nothing has been pushed,
no code-review subagent has run, W6 (close wave) has not started, and no verifier has
graded. Successor is a **fresh C1** — single-phase rotation, no C2 in play for P120.

**Read order:** this file in full → `.planning/phases/120-cli-helper-error-hardening/120-PLAN.md`
(762 lines, 7 waves W0–W6; § W6 for the exact close-wave task list) → `crates/CLAUDE.md`
§ "Error-message convention (Rust-compiler-grade UX)" (lines 92–103, the STALE stub W6
must fix-twice) → `.planning/ORCHESTRATION.md` §3 if the template itself is in doubt.

**Do NOT touch:** the E1 launch-animation owner-gate (`GTH-V15-37`, `.planning/CONSULT-
DECISIONS.md` 2026-07-17 entry) — out of scope for P120, owner approval still PENDING.
Do not hand-edit `quality/catalogs/agent-ux.json` rows directly (mint-only convention,
`quality/CLAUDE.md` § Catalog-first rule) — the two P120 rows flip via a real gate run +
verifier, never a JSON hand edit.

## 1. Ground truth (git)

- `HEAD = 646fb223`. `origin/main = d3268a5e` (P119 close) — local `main` is **7 commits
  ahead of origin/main, 0 behind**, all UNPUSHED. `git status`: tracked tree **CLEAN**
  (verified live: `git status --branch --short` → `## main...origin/main [ahead 7]`, no
  other lines).
- Commits since `d3268a5e` (oldest → newest, `git log --oneline d3268a5e..646fb223`):
  1. `5164845f` — docs(planning): P120 plan (762 lines, 7-wave catalog-first)
  2. `8d53de59` — feat(P120-W0): catalog-first agent-ux contract + gate scripts +
     `teach_scan.py` + test scaffolds (6 files, 713 insertions, 0 deletions — first
     impl-adjacent commit, catalog-only, satisfying SC3)
  3. `953a2e90` — feat(P120-W1): shared `errmsg::teach` primitive + init/attach retrofit
  4. `44d4ef56` — feat(P120-W2): list/refresh/spaces/sync retrofit + errors.rs scan-scope fix
  5. `c35a81f8` — feat(P120-W3): gc/history/tokens/cost/worktree retrofit — CLI complete
  6. `3937404c` — feat(P120-W4): helper entry/URL/cred retrofit + latent atlassian-origin
     credential-leak fix (`redact_userinfo` minted)
  7. `646fb223` — feat(P120-W5): helper transport/fan-out retrofit + `bus_handler:452/457`
     credential-leak fix — helper complete
- **This handover's own commit makes it 8 ahead** — expected, not corruption.
- Numbered deviations the successor MUST know:
  1. **No `PROGRESS.md` exists** for this phase — only `120-PLAN.md` + this handover.
     The plan + this handover ARE the state record this rotation; starting one in W6 is
     optional, not a gap to backfill.
  2. **Build/test status was NOT independently re-run by the outgoing seat.** I ran only
     the two `teach_scan.py` scans below (writing a handover shouldn't burn the one
     machine-wide cargo slot). W1–W5 commit bodies each self-report green `cargo test`/
     `clippy` (e.g. `953a2e90`: "cargo test -p reposix-core + -p reposix-cli green;
     clippy --all-targets -D warnings clean") but code review is the first INDEPENDENT
     re-run this rotation.
  3. **Live-verified this rotation** (commands + results, not just claims):
     - `python3 quality/gates/agent-ux/teach_scan.py --scope cli` → `clean (14 files, no
       un-dispositioned bail!/anyhow! block).` exit 0.
     - `python3 quality/gates/agent-ux/teach_scan.py --scope helper` → `clean (7 files, no
       un-dispositioned bail!/anyhow! block).` exit 0.
     - `quality/catalogs/agent-ux.json`: both `agent-ux/cli-errors-teach-recovery`
       (~line 2702) and `agent-ux/helper-errors-teach-recovery` (~line 2742) currently
       `"status": "NOT-VERIFIED"`, `"last_verified": null` — code is GREEN, catalog not
       yet flipped (W6 task).
     - `backend_dispatch.rs:468` — `pub fn redact_userinfo` present; used at
       `bus_handler.rs:452`/`:457` (the W5 fix). Weaker sibling
       `reposix_core::http::strip_url_userinfo` STILL used at `bus_handler.rs:134/172/
       191/391/404` — the "standardize" hardening item (§6) is real, unaddressed.
     - `crates/CLAUDE.md:92-103` — error-convention section EXISTS but is the pre-P120
       stub: names only `init.rs::refuse_existing_repo_root`, says the phase is merely
       "scheduled." Does not mention `errmsg::teach`, `errors.rs` helpers, the `doctor.rs`
       exception, or `redact_userinfo` — exactly what W6's fix-twice must add.
     - `GOOD-TO-HAVES.md` = 102,725B; `SURPRISES-INTAKE.md` = 84,358B — both already well
       over the 20k soft ceiling (warn-only, expires 2026-08-08, pre-existing not a P120
       regression) — appends are fine, files are just large to grep.

## 2. Wave/cycle state

| Wave | Concern | State | Commits |
|---|---|---|---|
| W0 | Catalog-first: 2 agent-ux rows + 2 gate scripts + `teach_scan.py` + test scaffolds | DONE | `8d53de59` |
| W1 | Shared `reposix_core::errmsg::{Teach, teach}` + `errors.rs` + init/attach retrofit | DONE | `953a2e90` |
| W2 | list/refresh/spaces/sync retrofit + errors.rs scan-scope fix | DONE | `44d4ef56` |
| W3 | gc/history/tokens/cost/worktree/main.rs retrofit — **CLI complete (SC1)** | DONE | `c35a81f8` |
| W4 | helper entry/URL/cred retrofit + `redact_userinfo` mint (latent leak beyond plan scope) | DONE | `3937404c` |
| W5 | helper transport/fan-out retrofit + `bus_handler:452/457` leak fix — **helper complete (SC2)** | DONE | `646fb223` |
| W6 | crates/CLAUDE.md fix-twice + catalog flip + gate-seam fix + final gate-green + verifier handoff | **NOT STARTED** | — |

No named-incident post-mortem this rotation — W0–W5 landed clean, sequentially, one
tree-writer at a time, per the plan's sequential single-tree-writer design.

Live-verified SC status (commands in §1 item 3):
- **SC1** (every CLI subcommand error 3-part): `teach_scan.py --scope cli` → 0
  un-dispositioned across 14 files. GREEN.
- **SC2** (every helper error 3-part): `teach_scan.py --scope helper` → 0
  un-dispositioned across 7 files. GREEN.
- **SC3** (catalog-first ordering): `8d53de59` (catalog+gates+scanner, 0 impl lines) is
  the first commit after plan `5164845f`, strictly before first impl commit `953a2e90`.
  Confirmed via the `git log` ordering in §1.

## 3. Binding constraints (unchanged)

One tree-writer at a time; **ONE cargo invocation machine-wide, FOREGROUND only, prefer
`-p <crate>`, NEVER `run_in_background` a build** (an executor that backgrounds cargo and
ends its turn strands the mutex + leaves uncommitted work — §5 item 2); never
`--no-verify`; push only at green, then confirm CI green on `main` AFTER the push via the
`code/ci-green-on-main` P0 post-push probe — never open the next phase over a red main;
commit-trailer format (`Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`
+ `Claude-Session: <seat-id>` where applicable, per `80a4282` precedent); model tiering
(fable → opus complex/security, sonnet default, haiku mechanical — never fable at a leaf).
Leaf isolation hard-stop (`/tmp` clone, `cd` in the SAME Bash invocation) applies to any
test/fixture setup a successor lane runs — not exercised this rotation (pure Rust source
+ catalog edits, no sim/init fixture work) but binding for any W6 lane that spins up a
live fixture. Do NOT touch the E1 animation owner-gate (GTH-V15-37). Enter through GSD.

## 4. Litmus / gate / REOPEN state

**No formal `--persist` gate run has executed this rotation** — W0–W5 landed via per-wave
`cargo test`/`clippy`/`teach_scan.py` spot-checks self-reported in each commit body (§1
item 2), plus the two live `teach_scan.py` re-runs this rotation performed (§1 item 3,
both exit 0). **No pre-push run has happened** — nothing pushed yet. The fresh C1's FIRST
gate-adjacent obligation (after code review) is running the two new gate scripts
directly: `bash quality/gates/agent-ux/cli-errors-teach-recovery.sh` and `bash
quality/gates/agent-ux/helper-errors-teach-recovery.sh` — each builds the relevant
binary, drives a REAL error path, and re-runs `teach_scan.py` — before trusting the
catalog flip.

**Gate-coverage seam, confirmed this rotation (real coverage gap, do not skip):**
`quality/gates/agent-ux/helper-errors-teach-recovery.sh:27` runs leg (a) as `cargo test
-p reposix-remote --test errors_teach_recovery -- --nocapture` — INTEGRATION target
only. The W5 `bus_handler:452/457` credential-leak regression test and broader
`redact_userinfo` unit coverage live in the BIN-TARGET (`#[cfg(test)]` modules inside
`crates/reposix-remote/src/*.rs`), which a scoped `--test errors_teach_recovery`
invocation does NOT exercise. W6 must widen this gate leg (e.g. plain `cargo test -p
reposix-remote`, covering both bin and integration targets) so the actual security-
regression coverage is IN the gate, not merely a commit-body claim.

**Open waiver expiry clocks (carried, unrelated to P120):**
- GOOD-TO-HAVES.md / SURPRISES-INTAKE.md OP-8 oversize warn-only waiver: expires
  **2026-08-08** (§1 item 3 — both already past 20k; P120 will add ≥3 more entries per
  §5/§6, worsening it — flag for milestone-close, do not let it silently keep growing).
- `structure/file-size-limits` OVER-BUDGET tier warn-only waiver: also expires
  **2026-08-08** — the file-size-drain GOOD-TO-HAVE below is timed against this clock.

No REOPEN state active — nothing that previously passed has regressed this rotation.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

**De-facto decisions made live this rotation (by the W1–W5 executors, per commit bodies —
inherited, not made directly by the outgoing coordinator):**
- W4 found and fixed a **latent atlassian-origin credential leak** in `backend_dispatch.rs`
  beyond the plan's literal scope, minting `pub fn redact_userinfo` to fix it — a
  deliberate "noticing is a deliverable" (OD-3) move: the fix stayed inside the W4 file
  set and shipped with regression coverage.
- W5 reused `redact_userinfo` for the `bus_handler:452/457` leak rather than inventing a
  second helper — good DRY call, but leaves the sibling `strip_url_userinfo` call sites
  (§1 item 3) inconsistent. Left as an optional-hardening item (§6) rather than force-fit
  into W5's scope.

**Noticed-not-yet-filed by the outgoing coordinator this rotation (carried into §6 for
the fresh C1 to file in W6 — do not drop, per OP-8):**
1. **HIGH, infra bug, → SURPRISES-INTAKE:** `.claude/hooks/cargo-mutex.sh`'s `pgrep`
   pattern FALSE-MATCHED a self-deadlocked shell wait-loop orphaned from a PRIOR session
   (`claude-84b9`) whose argv contained `rustc .*reposix/target/debug` — falsely held the
   machine-wide cargo mutex, froze ALL Bash calls this session until timeout, stalled W0,
   burned ~180k tokens in the reclaim. Orphan killed during W0-reclaim; the matching bug
   is still LATENT (unfixed). Fix sketch: exact process-name match (`pgrep -x cargo`/
   `pgrep -x rustc`, not substring/argv) and/or exclude the matcher's own PID/argv.
2. **MEDIUM, executor anti-pattern, → fix-twice `.planning/ORCHESTRATION.md` +
   `crates/CLAUDE.md`:** at least one W-wave executor spawned a detached
   `run_in_background` cargo invocation and ended its turn WITHOUT committing, stranding
   the machine-wide mutex and exposed uncommitted work. Every W1–W5 dispatch prompt
   already carried an explicit "cargo FOREGROUND only, commit before ending turn"
   guardrail (why it did not recur) — but the rule isn't yet a standing instruction in
   either CLAUDE.md, so a future dispatcher who forgets to restate it repeats the failure.
   §3 restates it here; W6 should land it durably.
3. **LOW, informational, P121 forward-pointer (not a P120 gap):** `errmsg::teach().code()`
   is wired+inert (renders nothing, pinned by a `code_slot_renders_nothing` byte-identity
   test per `953a2e90`), ready for `RPX-xxxx` codes once a future phase owns error-code
   minting. The shared error inventory for that work IS the 120-PLAN.md disposition
   table. `init.rs`/`attach.rs` already share `translate_spec_to_url` spec-parse teaching.
4. **Security wins worth naming at the eventual v0.15.0 RETROSPECTIVE.md distillation
   (OP-9, not urgent now):** the W4 atlassian-origin leak fix and W5 `bus_handler:452/457`
   fix are both real security regressions caught mid-phase with regression-test coverage
   added in the same commit — name explicitly, don't fold into a generic "P120 shipped."

## 6. Precise next steps (successor runbook)

1. **Spot-check ground truth first.** Re-run `git log --oneline -8`, `git status
   --branch --short`, and the two `teach_scan.py` commands from §1 item 3 before trusting
   this document.
2. **Dispatch code review** (gsd-code-reviewer, sonnet unless it flags security concerns)
   on the full diff `d3268a5e..646fb223`. Embed the ownership charter (below) verbatim.
   Have it independently run (FOREGROUND, one invocation, per §3): `cargo build -p
   reposix-cli -p reposix-remote -p reposix-core`, `cargo test` for the same three
   crates, `cargo clippy --all-targets -D warnings` scoped to them. Verify: (a) every
   retrofitted error genuinely teaches fix+alternative+recovery, not templated filler;
   (b) no site was missed (cross-check the plan's disposition table); (c) redaction is
   sound — `git grep -n 'redact_userinfo\|strip_url_userinfo' crates/reposix-remote/src/`
   for any remaining raw `user:pass@` echo path; (d) tests assert the exact 3-part
   substrings, not just "does not panic."
3. **Dispatch W6** (gsd-executor, sonnet — hand it the plan's W6 section + this file's
   §1 item 3 / §4 / §5):
   - Fix-twice `crates/CLAUDE.md:92-103`: document `errmsg::teach`, `errors.rs` shape
     helpers, the `doctor.rs` structured-report exception (health report, not a `bail!` —
     document why it's exempt, don't just silently exclude it), `redact_userinfo`, the
     teach-exempt marker convention, and the bin-vs-integration test-location seam
     (helper tests needing a live protocol exchange live in bin-target `#[cfg(test)]`
     modules, not `tests/errors_teach_recovery.rs`).
   - Flip both `agent-ux` catalog rows (`cli-errors-teach-recovery`,
     `helper-errors-teach-recovery`) NOT-VERIFIED → PASS with fresh `last_verified` —
     ONLY after both gate scripts actually run green this rotation (never pre-emptive).
   - Fix the gate-coverage seam (§4 — real security-coverage gap, do not skip): widen
     `helper-errors-teach-recovery.sh` leg (a) to also run bin-target unit tests (e.g.
     plain `cargo test -p reposix-remote`).
   - Optional <1h hardening: standardize `bus_handler.rs:134/172/191` on
     `redact_userinfo` instead of `strip_url_userinfo` (§1 item 3).
   - File GOOD-TO-HAVES.md: (a) `.rs` file-size drain for the P120-touched files before
     the 2026-08-08 waiver lapses; (b) promote `redact_userinfo` into
     `reposix_core::http` for cross-crate reuse; (c) widen+document `teach_scan.py`'s
     2-line exempt-marker lookback window (mirrors the 6-line test-name-honesty gotcha in
     `quality/CLAUDE.md`).
   - File SURPRISES-INTAKE.md: the HIGH `cargo-mutex.sh` false-match bug (§5 item 1 —
     keep the repro detail: `claude-84b9` orphan, argv match, ~180k token cost).
   - Fix-twice the executor anti-pattern (§5 item 2) into `.planning/ORCHESTRATION.md` so
     "cargo foreground only, commit before ending turn" is a standing instruction.
4. **Push `origin main`** (Bash timeout ≥300000ms — pre-push runs full clippy+kcov).
   Report to L0: "gated on CI run `<ID>`, wave = `<SHAs>`" and END TURN if waiting is
   long — L0 owns the CI watch. Once green, run `python3 quality/runners/run.py
   --cadence post-push --persist`, confirm `code/ci-green-on-main` P0 PASSes (main's
   NEWEST run, not a prior green run).
5. **Dispatch the verifier** (gsd-verifier; fallback general-purpose, zero session
   context) to grade SC1/SC2/SC3 from committed artifacts only — including independently
   re-deriving the catalog-first ordering via `git log` (do not trust this handover's
   derivation). Verdict → `quality/reports/verdicts/p120/VERDICT.md`. RED loops back to a
   fresh executor dispatch.
6. **On GREEN close:** advance `.planning/STATE.md` — cursor → "P120 CLOSED GREEN," next
   = P121; frontmatter `completed_phases: 6 → 7`, `percent: 40 → 47`; update `## Current
   Position`/`## Current Focus` prose (follow the P119 close entry as template). Commit,
   push, re-confirm `code/ci-green-on-main` once more post-STATE-push. Report a final
   ≤400-word summary to L0: the full RAISE LIST (§5/§6 — cargo-mutex bug, executor
   anti-pattern fix-twice, optional hardening item, filed GOOD-TO-HAVES/SURPRISES-INTAKE
   IDs), intake disposition, STATE.md cursor confirmation.

## Ownership charter (OD-3 — embed verbatim in every sub-dispatch)

1. You own what you touch. Acceptance criteria are the floor, not the ceiling — done
   means "I'd defend this in review as excellent," not "plan executed."
2. Noticing is a deliverable — teaching-free errors, tests that don't assert what their
   names promise, dead code, stale comments, missing edge cases, credential leaks.
3. Eager-fix or file — never silently skip. `<1h` + no new dependency → fix in place;
   else → SURPRISES-INTAKE / GOOD-TO-HAVES with severity + sketch.
4. Verify against reality — build it, run the error path, read the stderr, assert no
   secret leaks.
5. North star: Rust-compiler-grade UX — would a skeptical first-time dev hitting this
   error come away impressed?
