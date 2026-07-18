# v0.15.0 Surprises Intake — Part 3 of 7

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.

## 2026-07-16 06:05 | discovered-by: P115 Task-4 capture executor (L0 #39) | severity: MEDIUM

**What:** During the T4 GitHub capture, the **reposix-arm `git push` was rejected**: the `git-remote-reposix` helper returns `patch issue 60: not supported: update_record — reposix-github is read-only in this cut`. This is **intentional and documented** (`crates/reposix-github/src/lib.rs` `create_record`/`update_record`/`delete_or_close` all return not-supported; `crates/reposix-cli/src/doctor.rs:1467` states "github: read=yes, create/update/delete=— (read-only in this cut)"). So the reposix arm can read+edit+local-commit GitHub issues but **cannot write them back**. Two consequences: (1) the T4 SESSION-HANDOVER §2 recipe's assertion "the push writes back to GitHub via the helper" is **wrong for this cut** — only the mcp-arm `issue_write` persisted; the reposix-arm push does not. The token-economy comparison is UNAFFECTED (it measures agent context size, not write capability, and the failed-push tokens are negligible). (2) Any user-facing "push writes back" claim that implies GitHub is covered would be inaccurate for this cut (Confluence TokenWorld is the write-capable real backend; sim always writes).

**Why surfaced not fixed:** enabling GitHub issue writes is a real feature (REST PATCH mapping + conflict-detection + audit rows + egress/taint review), far more than <1h; and it's a deliberate cut, not a bug. The capture proceeded honestly (reposix arm = read+edit+commit+push-attempt with the documented read-only error; transcript at `benchmarks/fixtures/reposix_session.txt` shows it verbatim).

**Sketched resolution (for L0 to route):** either (a) implement `reposix-github` write-back (a scoped feature lane — pairs with the P122 `reposix-remote`/`init` hardening or a dedicated backend-write phase), or (b) if writes stay cut, audit docs/README/how-it-works so no "push writes back" claim reads as covering GitHub without the read-only caveat. Evidence: `crates/reposix-github/src/lib.rs:654/666/677`, `doctor.rs:1467`, `benchmarks/fixtures/reposix_session.txt`.

**STATUS:** OPEN

## 2026-07-16 | discovered-by: P117 W1 (noticed during execution; filed by L0 #55 intake triage) | severity: MEDIUM

**What:** `cargo nextest` is not installed in this environment, yet the P117 plans and `crates/CLAUDE.md` recommend `cargo nextest run` as the canonical full-workspace test runner. Every plan (and the cargo-mutex hook reminder) that copies the `cargo nextest` verify command verbatim fails with `error: no such command: nextest`, forcing the executor to silently substitute `cargo test`. Documentation-vs-environment mismatch erodes grounding trust — the next agent may assume the documented command works and copy it into a gate or CI step.

**Why out-of-scope for the discovering session:** Surfaced incidentally during P117 W1 execution, not a planned dev-image/tooling pass; installing `cargo-nextest` in the dev image or re-wording every `cargo nextest` reference across `crates/CLAUDE.md` + the plan verify blocks is a distinct tooling/docs change (and the fix-twice doctrine requires updating the doc references in the same change), not an eager patch inside a docs-truth wave.

**Sketched resolution:** Either (a) install `cargo-nextest` in the dev image so the documented command runs as written, or (b) soften `crates/CLAUDE.md` + the plan verify blocks to name `cargo test` as the guaranteed-available fallback with `cargo nextest` as preferred-if-present. Fix-twice: whichever path, reconcile the `crates/CLAUDE.md` recommendation and the cargo-mutex hook's "cargo nextest for full-workspace tests" reminder text with reality so the next agent does not hit the same `no such command` wall.

**STATUS:** OPEN

## 2026-07-16 07:47 | discovered-by: P115-T5 close-out executor (SURPRISES-INTAKE filing pass) | severity: MEDIUM

**What:** During T5's push, the pre-push gate summary printed "60 PASS, 1 FAIL" yet the hook exited 0. Traced the mechanism: `quality/runners/run.py::compute_exit_code` (L403-409) exits 0 "iff every P0+P1 row is PASS or WAIVED" — a FAIL row with `blast_radius: P2` (or lower) is counted in the printed summary but never flips the exit code. Separately, `run.py` L324-327/355-356 confirm a verifier timeout maps unconditionally to `row["status"] = "FAIL"` (there is no distinct TIMEOUT/WARN status) — so a timeout-class failure is visually indistinguishable from a real assertion failure in the summary line. Per the T5 executor's contemporaneous account, the specific FAILing row was an identification-class check hitting its ~5-minute timeout on the pre-pr-adjacent suite, pre-existing and unrelated to T5's changes (the `docs-repro benchmark-claim` rows T5 actually touched did NOT regress). Could not independently reproduce which exact row FAILed within this filing pass's ~10-minute budget — the exit-code MECHANISM above is verified fact (file:line, read directly from `run.py`); the specific-row attribution is hearsay from the T5 executor, recorded as such rather than re-stated as independently confirmed.

**Why out-of-scope for the discovering session:** Deciding whether a P2-blast-radius timeout-class FAIL is an intentional by-design WARN-equivalent (vs an oversight that should either block the push or get its underlying timeout fixed) requires reading the P0/P1/P2 blast-radius design rationale across the framework docs and possibly the prior RBF decisions that shaped `compute_exit_code` — a scoped semantics investigation, not a same-session mechanical fix. This close-out pass was scoped to filing + one small doc line, not framework-semantics changes.

**Sketched resolution:** (1) Decide by design: is a P2-blast-radius FAIL (timeout or otherwise) meant to be silently non-blocking? If yes, make the printed summary and/or per-row label distinguish it (e.g. `FAIL(non-blocking P2)`) or introduce a real `TIMEOUT` status distinct from `FAIL`, so a human scanning "60 PASS, 1 FAIL" does not read it as a live regression demanding triage. If no (P2 FAILs are meant to eventually get fixed, just not gate the push), the exit-0 behavior is already correct — but the summary line should still not present as ambiguous between "ignorable" and "needs attention." (2) Separately, consider whether verifier timeouts specifically deserve their own status distinct from an assertion-failure FAIL, since "timed out" and "assertions failed" are different failure classes with different triage paths and root causes.

**STATUS:** OPEN

## 2026-07-16 12:00 | discovered-by: P115-T6 Wave 2 item 2 executor (115-UNWAIVE-PATH.md inventory pass) | severity: MEDIUM

> **Corroborates, does NOT duplicate, the existing `2026-07-15 06:35` pre-push-timing
> item and its `2026-07-15 17:18` root-cause deep-dive above.** Third data point in the
> same creep; the drain phase should resolve all three together.

**What:** Pre-push hook wall-clock measured **~141s** on the push landing `d7da383`
(T6 Wave 1 item 3 agent-side retire+bind), following **~128s** on the immediately
preceding push. Both are well above the **~55–60s** budget documented in root
`CLAUDE.md` § GSD workflow ("pre-push ≈55s, dominated by kcov shell-coverage +
full-workspace clippy/mkdocs, not by what changed") and above even the **~75s**
re-baseline the `2026-07-15 17:18` entry already proposed. Pre-push cost is a fixed
whole-repo cost (NOT diff-size-scaled per the same CLAUDE.md line), so two consecutive
~128-141s runs read as the creep continuing past the point the prior entry's
re-baseline anticipated, not a one-off variance tail.

**Why out-of-scope for the discovering session:** This session's charter was a bounded
Wave-2 item-2 lane (write `115-UNWAIVE-PATH.md`, file this intake row, commit+push) —
profiling the pre-push gate stages or re-baselining `quality/CLAUDE.md` § Cadences is a
distinct investigation already owned by the two entries above; re-doing that work here
would duplicate scope rather than close it.

**Sketched resolution:** Same sketch as the two entries above, now with a third
corroborating measurement: (1) profile the pre-push gate stage-by-stage (the
`2026-07-15 17:18` entry's timed breakdown — `code/shell-coverage` 37.16s dominant — is
the starting point, re-run it against current `main` to see if the dominant row grew
further); (2) decide whether to cache kcov/clippy output across runs (biggest lever —
kcov + full-workspace clippy are both re-executed from scratch every push regardless of
diff size) or scope `mkdocs-strict` to only changed docs pages; (3) once a stable new
baseline is established, update root `CLAUDE.md` § GSD workflow's "≈55s" figure to match
reality (currently three data points — 91.7s, 128s, 141s — all exceed even the ~75s
re-baseline already proposed, suggesting the creep is ongoing past that estimate, not
just newly-discovered).

**STATUS:** OPEN

## 2026-07-16 07:50 | discovered-by: P115-T5 close-out executor (SURPRISES-INTAKE filing pass, relaying a T5 executor mid-task noticing) | severity: MEDIUM

**What:** Orchestration doctrine (`.planning/ORCHESTRATION.md` + the coordinator-dispatch skill) assumes a coordinator can `SendMessage` a running lane to deliver a mid-task scope correction, but the phase-coordinator harness exposes only `Agent`/`Bash`/`Read` as tools — no `SendMessage`. During T5, an L0 mid-task scope correction could NOT be delivered to the running executor; it was saved only because the executor's charter carried a "plan wins" clause that happened to keep it on the correct path anyway. This is a silent single-point-of-failure: if a future mid-flight correction is safety-critical (not just a redirection nicety) and the "plan wins" fallback doesn't happen to cover it, there is no delivery mechanism at all — a coordinator can only wait for the lane to finish or return, never interrupt it.

**Why out-of-scope for the discovering session:** Exposing `SendMessage` in the phase-coordinator agent definition's tool list (or formally documenting the gap + mitigation pattern in `ORCHESTRATION.md` §11) is an agent-def / orchestration-doctrine change — outside a scoped close-out pass whose charter explicitly forbids editing `ORCHESTRATION.md` or any agent def directly ("file only").

**Sketched resolution:** Either (a) add `SendMessage` to the phase-coordinator agent definition's tool list so a coordinator can interrupt/redirect a running lane mid-task, or (b) if `SendMessage` is intentionally withheld from coordinators (e.g. a deliberate isolation boundary), document the gap explicitly in `.planning/ORCHESTRATION.md` §11 alongside the "plan wins" clause as the documented mitigation pattern, so a future coordinator does not assume SendMessage works and silently rely on an undocumented fallback. Owner should decide (a) vs (b) — this is a capability/tooling decision, not a mechanical fix.

**Cross-reference (2026-07-16, filed by the P115 owner-directive lane):** per
`SESSION-HANDOVER.md` §6 finding 1, the T6 coordinator forked a fable-tier leaf to
deliver a mid-task scope correction (`SendMessage` unavailable in the subagent harness),
creating a momentary second tree-writer and a fable-at-leaf tiering violation; it executed
cleanly, no corruption. Judged the SAME underlying gap as this entry — cross-referenced
here rather than filed as a new row.

**STATUS:** OPEN

## 2026-07-16 | discovered-by: P115 owner-directive lane doc sweep | severity: MEDIUM

**What:** `docs/social/twitter.md:18` and `docs/social/linkedin.md:21` (plus
`docs/social/assets/_build_benchmark.py`, `_build_combined.py`, `benchmark.svg`) still
present the OLD 89.1% token-reduction figure as current, with no retirement language —
verified live: both files read "89.1% fewer tokens" today. Catalog rows
`docs/social/twitter/token-reduction-92pct` and `docs/social/linkedin/token-reduction-92pct`
sit at `STALE_TEST_DRIFT` with `next_action: BIND_GREEN`, actively tracking the old
number (confirmed by direct catalog read).

**Why out-of-scope for the discovering session:** The owner-directive lane's charter was
narrowly scoped to removing retirement-HISTORY narrative from user-facing docs (the
89.1%/85.5% retirement-story sections, not stale numbers still presented as current); the
social drafts are a distinct staleness class — an old live number, not a narrative — and
deciding whether to refresh/freeze/retire them is an owner call, not a mechanical doc edit
inside this lane.

**Sketched resolution:** Owner decision needed: (a) refresh the drafts + assets to the
current live ~94.3%/~75% four-axis figures, (b) freeze them as intentionally-dated
snapshots (add a "as of <date>" caveat so they read honestly), or (c) retire the two
catalog rows if the social posts themselves are considered historical artifacts not meant
to track current numbers. Whichever is chosen, update the two catalog rows to match
(re-bind if refreshed, retire if frozen/historical).

**STATUS:** OPEN

## 2026-07-16 | discovered-by: P115 owner-directive Wave-1 executor | severity: MEDIUM

**What:** 34 catalog rows (the `docs/benchmarks/latency.md` L38-44 block,
`bench-latency-cron.yml`, `docs/index.md` hero rows) carry stale `test_body_hashes` for
`quality/gates/perf/latency-bench.sh` even though the script content is identical to
HEAD — pre-existing `STALE_TEST_DRIFT` bit-rot that `walk.sh` does NOT block on (verified:
its documented blocking list is `STALE_DOCS_DRIFT` / `MISSING_TEST` / `STALE_TEST_GONE` /
`TEST_MISALIGNED` / `RETIRE_PROPOSED` — `STALE_TEST_DRIFT` is silent, `walk.sh:71-72`).

**Why out-of-scope for the discovering session:** Wave 1's charter was the doc-narrative
strip (removing retirement-history prose from four docs); re-binding 34 unrelated
`test_body_hashes` on a script that hasn't actually changed is a distinct catalog-hygiene
sweep, not a doc-narrative edit, and touching 34 rows is far larger than the Wave 1 diff.

**Sketched resolution:** Dedicated re-bind pass on the 34 affected rows (re-hash
`quality/gates/perf/latency-bench.sh` and refresh each row's `test_body_hashes` to match,
since the content itself is unchanged — this is a hash-refresh, not a content fix).
Separately, consider surfacing `STALE_TEST_DRIFT` at pre-push or in post-push reporting
(even non-blocking) so this class of drift can't decay silently across future script
edits.

**STATUS:** OPEN

## 2026-07-16 | discovered-by: P115 phase-close cold-reader pass (L0 #44) | severity: MEDIUM

**What:** `README.md:109-110` "Project status" is 3 months / 4+ versions stale on the
repo's most visible surface: it claims v0.9.0 shipped + v0.10.0 "landing 2026-04-25" as
latest, while the repo is at v0.14.0 (Cargo.toml + git tags) with v0.15.0 executing
(2026-07-16). Same lying-hero-claim class the P115 destaling lanes just purged from the
benchmark numbers — but in a narrative section no number-focused gate watches.

**Why out-of-scope for the discovering session:** the cold-reader lane was report-only;
a status-section rewrite is a framing decision (what to promise about v0.15.0, whether
README should carry a version ticker at all) — planner/owner input, not a figure swap,
and the section's shape invites recurring staleness (would rot again by v0.16.0).

**Sketched resolution:** (a) version-agnostic status line linking GitHub releases /
CHANGELOG as the single version-truth source, or (b) keep a version line but bind a
doc-alignment row asserting README version == latest git tag so drift blocks at
pre-push. (b) is the fix-twice option — P117 (doc-truth purge) is the natural home.

**STATUS:** OPEN

## 2026-07-16 | discovered-by: P115 cold-reader dispatch (L0 #44) | severity: MEDIUM (tooling, silent-failure class)

**What:** the user-global `doc-clarity-review` skill
(`/home/reuben/.claude/skills/doc-clarity-review/`) is silently broken: its documented
`claude -p "<prompt>" <file1> <file2>` invocation does NOT attach trailing file paths in
the current CLI — the subprocess receives no files and "reviews" ambient cwd context
(`~/.claude/CLAUDE.md` + project CLAUDE.md) instead, returning a plausible-looking review
of the WRONG target. Confirmed via diagnostic call (`claude -p "What files were you
given?" f1 f2` → "no files"). Dangerous class: a silent wrong-target review reads as a
pass on the right target. (The P115 pass itself completed via the leaf's manual
fallback: isolated read of only the two target files.)

**Why out-of-scope for the discovering session:** the skill lives in the user's global
`~/.claude/skills/`, not this repo; fixing it is cross-project tooling work.

**Sketched resolution:** inline file CONTENT into the `-p` prompt (cat the copied `/tmp`
files / heredoc) instead of passing paths as args; add a loud self-check ("state the
first heading of each file you were given") so a no-files run fails visibly. Distinct
from the unreproduced "intermittent Read/Edit harness failures" noticing in #43's
handover §5.4 — that stays unfiled pending a live repro.

**STATUS:** OPEN

## 2026-07-16 | discovered-by: quick 260716-fmt (GTH-V15-35) | severity: MEDIUM

**What:** `test_main_offline_regenerates_doc_from_captures` in
`quality/gates/perf/test_bench_token_economy.py` (L212-244) asserts idempotency ONLY
against a synthetic `tmp_path` fixture — it monkeypatches `bench.RESULTS` /
`bench.CAPTURES` / `bench.BENCH_DIR` to a temp dir and checks that a second
`--offline` run reproduces the same bytes as the first — and NEVER diffs the
regenerated doc against the real committed `docs/benchmarks/token-economy.md`. That is
the exact gap that let the 260716-f6o generator regression (the retired-narrative
section re-added by the template) reach a P115 phase-close gate run undetected: the
test's own idempotency check passed because it only compares the synthetic doc to
itself, not to the committed source of truth.

**Why out-of-scope for the discovering session:** 260716-fmt's charter is a docs/index.md
install-IA fix (block relocation + bootstrap prose + L93 destale); adding a new
byte-compare regression test to a different quality gate (`perf/test_bench_token_economy.py`)
is an unrelated test-authoring change outside that scope, surfaced only because the
260716-f6o regression this gap enabled was fixed in the immediately preceding commit.

**Sketched resolution:** Add a regression test that runs the offline regenerator against
the REAL committed captures (not the synthetic `tmp_path` fixture) and byte-compares
(sha256) the output against the committed `docs/benchmarks/token-economy.md` — so any
future generator/doc divergence fails a test instead of silently dirtying the working
tree at a gate run, the way the 260716-f6o regression did.

**STATUS:** OPEN

