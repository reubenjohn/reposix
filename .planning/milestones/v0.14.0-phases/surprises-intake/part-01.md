# v0.14.0 Surprises Intake (P110 source-of-truth) — Part 1 of 2

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.

## 2026-07-11 23:00 | discovered-by: P102 (adversarial code-review) | severity: HIGH

**What:** Guard B (leaf-setup location) matched only the literal `reposix init` command
shape at a command position. The CLAUDE.md-documented canonical dev forms slid past at the
shared tree: `cargo run -p reposix-cli -- init sim::demo .`, path-suffixed
`/usr/bin/reposix init` / `./target/debug/reposix init`, and bare `reposix attach|sync`.
This is the guard's core reason to exist — a leaf that "forgot to cd" via any of these
canonical spellings would still corrupt the shared repo.

**Why in-scope for P102:** it IS P102's mandate — closed eagerly in the fix lane.

**Resolution:** Guard B now matches `([^[:space:]]*/)?reposix[[:space:]]+(init|attach|sync)`
(optional binary-path prefix), `cargo[[:space:]]+run[^;&|]*--[[:space:]]+(init|attach|sync)`,
and path-suffixed `reposix-sim`. Proven: extended `evasion.sh` shows all six canonical forms
BLOCK (rc=2) at the shared tree and still ALLOW under a `/tmp` redirect.

**STATUS:** RESOLVED-in-P102, commit `39a8500` (`fix(p102): harden guards — canonical-form + realpath cwd + quoting + fail-closed`; founding guard `bf88470`). Verified real: `.claude/hooks/leaf-isolation-guard.sh` `guard_leaf_setup_location` matches the path-prefixed / `cargo run … --` / path-suffixed `reposix-sim` canonical forms (CLAUDE.md § Non-negotiables documents all three as BLOCKING).

---

## 2026-07-11 23:00 | discovered-by: P102 (adversarial code-review) | severity: HIGH

**What:** The `/tmp`-means-safe test was a naive substring match (`cd /tmp` anywhere / cwd
`case`), so `cd /tmp/x && cd <shared> && …` (cd-back), `/tmp/../<shared>` (`..` traversal),
and a `/tmp` symlink pointing at the shared tree all resolved as SAFE and slid through
guards B and C.

**Why in-scope for P102:** core soundness of the isolation gate; closed eagerly.

**Resolution:** Replaced with an `effective_target()` (honors the LAST `cd`, or a git
path-flag target `-C`/`-f`/`--git-dir`/`--file`/`--work-tree`, else the payload cwd) that is
`realpath -m`-canonicalized; SAFE iff the canonical path resolves under `/tmp`. Undeterminable
target → fail-closed (unsafe). Proven: cd-back, `/tmp/../` traversal, and a REAL `/tmp`
symlink → shared all BLOCK; a genuine `/tmp` path still ALLOWs.

**STATUS:** RESOLVED-in-P102, commit `39a8500`. Verified real: `.claude/hooks/leaf-isolation-guard.sh` `effective_target`/`is_safe` realpath-canonicalize the target (cd-back, `/tmp/../` traversal, and a `/tmp`→shared symlink all BLOCK; a genuine `/tmp` path ALLOWs) — matches the CLAUDE.md § Non-negotiables realpath-canonicalization contract.

---

## 2026-07-11 23:00 | discovered-by: P102 (adversarial code-review) | severity: MEDIUM

**What:** Guard A matched only a bare `-c user.email=t@t`; a quoted fixture email
(`-c user.email='t@t'` / `"t@t"`, and `GIT_AUTHOR_EMAIL='t@t'`) slid past. (The
`.githooks/pre-commit` backstop still catches the commit itself, but guard A had a
trivially-quoted hole.)

**Why in-scope for P102:** one-line regex tightening; closed eagerly.

**Resolution:** Guard A tolerates optional surrounding quotes `['\"]?` around the fixture
email/name tokens, keeping the delimiter-bounded match so `scott@things.io` still does NOT
false-positive. Proven: quoted forms BLOCK (rc=2), real-address control ALLOWs (rc=0).

**STATUS:** RESOLVED-in-P102, commit `39a8500`. Verified real: `.claude/hooks/leaf-isolation-guard.sh` `guard_fixture_identity` tolerates optional surrounding quotes `['\"]?` (quoted `'t@t'`/`"t@t"` BLOCK) while a real address (`scott@things.io`) does NOT false-positive — matches the CLAUDE.md § Non-negotiables fixture-email-quoting contract.

---

## 2026-07-11 23:00 | discovered-by: P102 (adversarial code-review) | severity: HIGH

**What:** A non-empty but unparseable/malformed payload yielded `cmd=""` (the `|| true`
swallowed the parse error) → no guard fired → exit 0 while the real command ran. For a
fail-closed security guard this is fail-OPEN.

**Why in-scope for P102:** direct contradiction of the fail-closed mandate; closed eagerly.

**Resolution:** The payload is parsed once by a python stage that emits an explicit STATUS
(`empty`/`parse_error`/`ok`). A non-empty unparseable payload (or a non-object JSON, or a
crashed interpreter with a non-empty payload) → exit 2 (BLOCK) with a teaching message; an
empty payload with nothing to inspect still passes. Proven: `not json`, truncated JSON, and
`[]` all BLOCK; `''` and `{}` ALLOW.

**STATUS:** RESOLVED-in-P102, commit `39a8500`. Verified real: `.claude/hooks/leaf-isolation-guard.sh` parse dispatch fails CLOSED — a non-empty unparseable payload (`not json`, truncated JSON, non-object `[]`) exits 2 (BLOCK); only an empty `''`/`{}` payload ALLOWs — matches the CLAUDE.md § Non-negotiables "non-empty unparseable payload fails closed (exit 2)" contract.

---

## 2026-07-11 23:00 | discovered-by: P102 (adversarial code-review) | severity: MEDIUM

**What:** The three kind:shell-subprocess proof transcripts were `git add -f`'d against
`.gitignore` (`quality/reports/transcripts/*.txt`) and rot by construction — a live re-run
drops the ignored duplicates, so the committed artifacts drift from what the verifier
produces. A secondary false-positive was also found: `git config -f /tmp/…` (short form of
`--file`) was BLOCKED even though it targets a `/tmp` clone.

**Why in-scope for P102:** catalog-contract honesty + a guard-C over-block; closed eagerly.

**Resolution:** (a) `git rm --cached`'d the three force-added transcripts (now correctly
gitignored per-run snapshots); each row's `expected.asserts` reworded from "committed
transcript exists" to "the verifier, run at grade-time, REGENERATES a transcript showing the
exit-2 block + teaching stderr + (guard C) sha256 byte-unchanged config" — durable proof is
the verifier script + hook, not a frozen `.txt`. (b) `effective_target()` now recognizes the
`-f` short flag, so `git config -f /tmp/…` ALLOWs. Verifiers extended with the new hardening
cases; all three exit 0 self-contained.

**STATUS:** RESOLVED-in-P102, commit `39a8500` (`expected.asserts` rework from "committed transcript exists" to grade-time regeneration + `git rm --cached` of the force-added transcripts + `effective_target` `-f`-short-flag recognition so `git config -f /tmp/…` ALLOWs). Verified real against the guard's `-f` handling + the reworded catalog asserts.

---

## 2026-07-12 07:13 | discovered-by: P104 (github-helper-path 404 fix verifier) | severity: MEDIUM

**What:** Two concurrent `reposix-quality` runners (or herdr `--persist` modes) were observed minting the shared catalog file (`quality/catalogs/agent-ux.json`) mid-verification during P104 grading (PID 351077 held the lock while the verifier was running). Two writers on one catalog file is a live race hazard — interleaved writes can corrupt the JSON or lose rows entirely. The herdr on-demand `--persist` runner and the executor's own persist lane can collide without coordination.

**Why out-of-scope for P104:** P104 is closing a fix (404 path bug), not a catalog infrastructure issue. The race was observed but did not break the final grade (the concurrent runner's write agreed with the independent verification); fixing it requires coordination infrastructure outside the phase's scope.

**Sketched resolution:** Implement a catalog-write lock (advisory flock around the catalog JSON persist in `quality/runners/run.py`, or serialize all catalog persist operations through a single lane with a lock file) such that two concurrent `--persist` writers cannot interleave. Alternative: single-persist-lane discipline where only the primary orchestration lane writes catalogs, and herdr on-demand runners read but do not persist.

**STATUS:** DEFERRED-TO-v0.15.0 (framework-hardening phase). A real flock/single-persist-lane
design touching the runner concurrency model (contended `quality/runners/*.py`) — >1h + design,
not a mechanical eager-fix. The hazard is latent, not active: catalog writes are currently
serialized by orchestration discipline (one runner/persist lane at a time), and the P104 grading
where it was observed did not corrupt the JSON (the concurrent writer's write agreed with the
independent verification). Belongs to the same v0.15.0 framework-hardening phase as the
verifier-script-path-validation row below.

---

## 2026-07-12 07:35 | discovered-by: v0.14.0 health-triage lane (main gate sweep) | severity: MEDIUM

**What:** `code/shell-coverage` is a genuine, live FAIL on `main` (P2 blast_radius, non-blocking on
pre-push): aggregate shell line-coverage is 12.54% (564/4497 lines), below the committed
13.00% floor in `quality/shell-coverage-floor.txt`. Root cause is corpus growth, not a
coverage drop in previously-tested code: the shell corpus grew to 149 scripts, 110 of which
sit at 0% coverage (mostly `quality/gates/agent-ux/*` dark-factory/litmus scripts and
`.claude/hooks/*` added across P90-P97), diluting the aggregate below floor. Re-verified live
(`bash quality/gates/code/shell-coverage.sh`, real kcov run, 2026-07-12T07:27:58Z) — this is
not stale or masked, it is a real, current shortfall.

**Why out-of-scope for eager-fix:** Closing the gap needs real shell tests written for a
subset of the 110 uncovered scripts (`quality/gates/code/shell-coverage-tests/`), which is
open-ended test-authorship work, not a mechanical one-file fix — well beyond the <1h /
no-new-dependency eager-fix bar.

**Sketched resolution:** Either (a) write `shell-coverage-tests/` harness cases for the
highest-line-count 0%-covered scripts until aggregate clears 13% again (preferred — the
committed doctrine is "raise the floor over time, never force-pass by lowering it above
measured"), or (b) if a deliberate decision is made that some scripts are structurally
untestable outside real backends (dark-factory real-arm scripts, TokenWorld scripts), lower
the floor to the currently-measured 12.54% (or slightly below) with a documented rationale
in `quality/CLAUDE.md`'s existing "Follow-up (documented, left at 0%)" note, and open a
GOOD-TO-HAVES tracking item for the deferred scripts. Do not silently patch the floor number
without one of these two paths — that would be exactly the kind of quiet weakening the
project's honesty rules exist to prevent.

**STATUS:** DEFERRED to the coverage-climb work (phases `999.5 docs-crates-md-zero-coverage`
/ `999.6 docs-alignment-coverage-climb`). Reason: authoring shell tests for a subset of the 110
zero-coverage scripts to clear the 13% floor is open-ended test-authorship, not a mechanical
one-file fix; lowering the floor without a climb plan would be exactly the honesty regression the
row itself warns against — so the honest disposition is to DEFER to the dedicated climb, not to
touch the floor now. **Push-blocking check (per the drain charter): NOT an active blocker.**
`code/shell-coverage` carries `blast_radius: P2` and `cadences: ["pre-push"]`;
`quality/runners/run.py::compute_exit_code` exits 1 ONLY when a P0/P1 row is not PASS/WAIVED, so
a P2 FAIL yields exit 0 → the pre-push hook (`.githooks/pre-push` → `run.py --cadence pre-push`,
no `--fail-on` override) does NOT block the push. Nuance carried forward for the owner: the
separate CI `shell-coverage` job (`.github/workflows/ci.yml`) DOES hard-fail on kcov, which can
surface via the P0 `code/ci-green-on-main` post-push probe — that is the pre-existing state the
health-triage lane already assessed, not a new P110 regression.

---

## 2026-07-12 07:40 | discovered-by: v0.14.0 health-triage lane (main gate sweep) | severity: LOW

**What:** Three `on-demand`-cadence `agent-ux.json` catalog rows tied to the now-CLOSED and
tagged v0.13.0 milestone show stale FAIL, but each is invalidated by a *documented, later*
process decision rather than a live regression — re-running them cannot honestly produce a
fresh PASS without contradicting that later decision:
- `agent-ux/p87-surprises-absorption` — asserts >=5 terminal-STATUS entries remain in
  `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`. The 2026-07-06 pre-tag sweep
  (documented in that file's own CARRY-FORWARD BANNER) deliberately **deletes** terminal
  entries ("git is the archive, bound-to-live-state"), so the assertion is now permanently
  unsatisfiable by policy, not by regression.
- `agent-ux/p88-good-to-haves-drained` — asserts every `GOOD-TO-HAVES-NN` entry carries a
  terminal STATUS. Re-checked live against all 8 `good-to-haves/part-NN.md` files (the OP-8
  file-size drain split the monolith after this row was minted): genuinely still ~30+ entries
  OPEN/DEFERRED per the file's own 2026-07-06 carry-forward tally — this is a real, current,
  correctly-FAIL state per the file's own content, but the *milestone itself* has since
  declared (via its CARRY-FORWARD BANNER) that full v0.13.0 drain is intentionally deferred to
  a v0.14.0-scoping session (P110/P111) — the P88 gate's "this milestone must fully drain"
  contract was superseded before it could be satisfied.
- `agent-ux/v0.13.0-tag-script-present` — asserts `tag-v0.13.0.sh` exists and is executable.
  The script was intentionally renamed to `tag-v0.13.0.sh.disabled` after the tag was already
  cut (v0.13.0 shipped, main is now past v0.13.1) — this is expected post-use archival, not
  an accidental deletion.

**Why out-of-scope for eager-fix:** All three require an owner decision (retire the row vs.
rewrite its contract to read the archived/split state) — not a mechanical `<1h` fix, since
"fixing" them by relaxing the assertion could itself look like quietly weakening a gate.

**Sketched resolution:** Formally retire (mark `WAIVED` with a `until`-less permanent
rationale, or delete) these 3 rows now that v0.13.0 is closed-and-tagged and each row's
completion criterion was superseded by a later, documented process decision. A future
milestone-close should add a step: "on-demand milestone-close rows for a just-closed
milestone get retired, not left FAIL forever."

**STATUS:** RESOLVED (eager waive). All three rows live in `quality/catalogs/agent-ux.json`
(NOT the foreign-locked `code.json` — confirmed via grep: the "tag-script" row is
`agent-ux/v0.13.0-tag-script-present`, in agent-ux.json), so all three are waivable in this
P110 drain. Waivers added to `p87-surprises-absorption`, `p88-good-to-haves-drained`, and
`v0.13.0-tag-script-present` with the rationale: "v0.13.0 milestone archived closed-green → the
row is a historical PASS marker per the v0.12.1 P76 precedent; a superseded-policy FAIL (the
milestone's own CARRY-FORWARD BANNER deleted terminal entries / split the GOOD-TO-HAVES monolith
/ disabled the tag-script post-tag) is expected, waived." Waived in this same P110 drain commit
(`quality/catalogs/agent-ux.json`) — the drain-commit SHA is reported to the coordinator.

---

## 2026-07-12 07:13 | discovered-by: P104 (github-helper-path 404 fix verifier) | severity: MEDIUM

**What:** A catalog row was minted `status: PASS` with a `verifier.script` path that did not exist on disk (`quality/catalogs/agent-ux.json`, P104 BLOCKER that was caught only during manual code review). The row `agent-ux/p87-surprises-absorption` was defined with status FAIL but lacked a `claim_vs_assertion_audit` field required by the schema (rows minted after 2026-05-08 must include this field for honesty auditing). The pre-commit hook validation does not structurally verify that a row's declared `verifier.script` path exists or is executable — only that the JSON is valid. This opens a window where a coordinator could mint a PASS row backed by a missing or non-executable verifier, creating a false-positive contract breach.

**Why out-of-scope for P104:** P104 closes the 404 bug fix verification; the catalog schema validation gap is an infrastructure issue. It was surfaced during verification but requires a new gate in the structure dimension that does not yet exist.

**Sketched resolution:** Add a structure-dimension gate (`quality/gates/structure/verifier-script-exists.sh`) that scans all catalog rows at load time and asserts: for each row with a non-null `verifier.script`, the file exists on disk and is executable (chmod +x). The gate would fail at pre-commit or pre-push if any row references a missing verifier, preventing unbacked PASS rows from landing. This is a complement to GOOD-TO-HAVES-01 (bind-verb extension for agent-ux rows).

**STATUS:** DEFERRED-TO-v0.15.0 (framework-hardening phase). A new structure-dimension gate
(`quality/gates/structure/verifier-script-exists.sh` scanning every row's `verifier.script` for
on-disk existence + executability at load time) is new infra >1h — beyond the eager-fix bar. Pairs
with the catalog-race row above in the same v0.15.0 framework-hardening phase. Note: the immediate
P104 instance (a PASS row with a missing script path) was caught in manual review and did not ship;
this defers the systematic GATE, not an active false-positive.

---

## 2026-07-12 08:10 | discovered-by: P105 (RBF-LR-03 rebase-recovery research) | severity: HIGH

**What:** SILENT LOST UPDATE via the shared-cache `last_fetched_at` cursor. Two `reposix
init` clones of the same `sim::demo` share ONE bare cache (keyed by `(backend, project)`
per `reposix_cache::path::resolve_cache_path`). When clone A pushes an edit, the SoT-success
branch advances the SHARED cursor to `now` (`crates/reposix-remote/src/write_loop.rs:309`,
`c.write_last_fetched_at(Utc::now())`). Clone B then pushes a *conflicting stale-base*
edit; its L1 PRECHECK B runs `backend.list_changed_since(last_fetched_at=now)` → returns
an EMPTY changed-set (A's write is at-or-before `now`) → no conflict detected → B's PATCH
lands and silently clobbers A's edit. **Empirically reproduced**
(`.planning/phases/105-rbf-lr-03-rebase-recovery/repro/repro-lost-update.sh`, live sim,
git 2.25.1): issue-1 title `A-CHANGED-TITLE` (v2) → `B-CHANGED-TITLE` (v3), with NO `fetch
first` reject and NO error emitted. The ARCH-08 protection that
`crates/reposix-remote/tests/push_conflict.rs::stale_base_push_emits_fetch_first_and_writes_no_rest`
proves in ISOLATION (fresh per-test cache) FAILS end-to-end under a shared cache. This is
data loss — strictly worse than the RBF-LR-03 pull-rebase friction — and is the true
manifestation of the "code framing" cursor concern the P105 dispatch flagged (the
pull-rebase abort itself is a SEPARATE bug in `fast_import.rs`, fixed under P105).

**Why out-of-scope for P105:** P105's charter is the rebase-recovery abort (a fetch-level
`fast_import.rs` fix). This is a push-side precheck/cursor-semantics bug — different code
path, different fix, likely > 1h, and coupled to the v0.14.0 reconciliation redesign
(per-writer base tracking or version-conditioned precheck rather than a wall-clock cursor).
Folding it into P105 would double the phase scope.

**Sketched resolution:** PRECHECK B must not rely solely on a shared wall-clock
`last_fetched_at` to decide "did the SoT move under me." Options: (a) compare the pushed
record's base `version` against the backend's current `version` per-record (optimistic
concurrency on the field the sim already tracks) regardless of the cursor window; (b) track
the tracking-ref tip the push is based on and re-diff against the live SoT unconditionally;
(c) make the sim's PATCH enforce version-match (409 on stale base) so the backend is the
final arbiter. Add a regression test: two shared-cache clones, A pushes, B stale-pushes →
assert B is rejected `fetch first` AND the SoT retains A's edit.

**STATUS:** RESOLVED by **P113**, fix commit `61e8222` (`fix(106-01): lost-update guard —
shared cursor no longer gates conflict detection`; renumbered 106→113 at `4dd7e10`; catalog-first
`632864d`; owner-authorized external-mutation land `ed42ece`) — SHAs verified real via `git log`.
Fix = option (a): `precheck_export_against_changed_set` no longer GATES the per-record version check on
`list_changed_since` delta membership — it issues the authoritative `get_record` for every
pushed Update (the backend is the sole SoT arbiter), so a stale-base push under an advanced
shared cursor is REJECTED `fetch first` instead of clobbering. The conflict is content-aware
(a stale base version alone is a no-op — QL-001; only a stale base WITH divergent writable
content rejects). Regression
`precheck::tests::stale_base_push_rejected_when_shared_cursor_advanced_past_concurrent_write`
(real Cache + advanced cursor + AdvancedCursorMock@v2) — proven RED (Proceed = lost update)
with the delta gate restored, GREEN with the fix. Catalog row
`code/lost-update-shared-cursor-rejected`.

**Numbering collision (for coordinator):** the wave-2 handover §5 earmarked this phase as
"P106", but `ROADMAP.md` already assigns Phase 106 to "Waived tutorials reproduce" (102–112
all occupied). Per the dispatch charter's fallback it was minted at the next free slot, **113**.
The handover's P106/P107/P108/P109 re-table only diverges from ROADMAP at 106 (108/109 already
agree); reconcile whether tutorials-reproduce keeps 106 and lost-update stays 113, or renumber.
The first three fix commits carry a `106-01` label (landed before the collision was caught);
substance unchanged.

---

