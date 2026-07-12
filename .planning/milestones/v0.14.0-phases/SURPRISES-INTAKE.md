# v0.14.0 Surprises Intake (P110 source-of-truth)

> **Append-only intake for surprises discovered during P102–P109 execution.**
> Each entry is something the discovering phase chose NOT to fix eagerly because it was
> massively out-of-scope. P110 drains this file (per CLAUDE.md OP-8 — Slot 1 of the v0.14.0
> +2 reservation).
>
> **Eager-resolution preference:** if a surprise can be closed inside its discovering
> phase without doubling the phase's scope (rough heuristic: < 1 hour incremental work, no
> new dependency introduced, no new file created outside the phase's planned set), do it
> there. The intake file is for items that genuinely don't fit.
>
> **Distinction from `GOOD-TO-HAVES.md`:** entries here fix something that's BROKEN,
> RISKY, or BLOCKING. Improvements/polish go in `GOOD-TO-HAVES.md` (drained by P111, Slot
> 2).

## Entry format

```markdown
## YYYY-MM-DD HH:MM | discovered-by: P<N> | severity: BLOCKER|HIGH|MEDIUM|LOW

**What:** One-paragraph description of what was found.

**Why out-of-scope for P<N>:** Why eager-resolution wasn't possible (scope, time, dependency).

**Sketched resolution:** One paragraph proposing how P110 should resolve.

**STATUS:** OPEN  (← P110 updates to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA)
```

---

## Entries

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

**STATUS:** RESOLVED-in-P102 (`.claude/hooks/leaf-isolation-guard.sh` guard_leaf_setup_location).

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

**STATUS:** RESOLVED-in-P102 (`.claude/hooks/leaf-isolation-guard.sh` effective_target/is_safe).

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

**STATUS:** RESOLVED-in-P102 (`.claude/hooks/leaf-isolation-guard.sh` guard_fixture_identity).

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

**STATUS:** RESOLVED-in-P102 (`.claude/hooks/leaf-isolation-guard.sh` parse dispatch).

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

**STATUS:** RESOLVED-in-P102 (catalog `expected.asserts` rework + `git rm --cached` + `-f` flag).

---

## 2026-07-12 07:13 | discovered-by: P104 (github-helper-path 404 fix verifier) | severity: MEDIUM

**What:** Two concurrent `reposix-quality` runners (or herdr `--persist` modes) were observed minting the shared catalog file (`quality/catalogs/agent-ux.json`) mid-verification during P104 grading (PID 351077 held the lock while the verifier was running). Two writers on one catalog file is a live race hazard — interleaved writes can corrupt the JSON or lose rows entirely. The herdr on-demand `--persist` runner and the executor's own persist lane can collide without coordination.

**Why out-of-scope for P104:** P104 is closing a fix (404 path bug), not a catalog infrastructure issue. The race was observed but did not break the final grade (the concurrent runner's write agreed with the independent verification); fixing it requires coordination infrastructure outside the phase's scope.

**Sketched resolution:** Implement a catalog-write lock (advisory flock around the catalog JSON persist in `quality/runners/run.py`, or serialize all catalog persist operations through a single lane with a lock file) such that two concurrent `--persist` writers cannot interleave. Alternative: single-persist-lane discipline where only the primary orchestration lane writes catalogs, and herdr on-demand runners read but do not persist.

**STATUS:** OPEN

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

**STATUS:** OPEN

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

**STATUS:** OPEN

---

## 2026-07-12 07:13 | discovered-by: P104 (github-helper-path 404 fix verifier) | severity: MEDIUM

**What:** A catalog row was minted `status: PASS` with a `verifier.script` path that did not exist on disk (`quality/catalogs/agent-ux.json`, P104 BLOCKER that was caught only during manual code review). The row `agent-ux/p87-surprises-absorption` was defined with status FAIL but lacked a `claim_vs_assertion_audit` field required by the schema (rows minted after 2026-05-08 must include this field for honesty auditing). The pre-commit hook validation does not structurally verify that a row's declared `verifier.script` path exists or is executable — only that the JSON is valid. This opens a window where a coordinator could mint a PASS row backed by a missing or non-executable verifier, creating a false-positive contract breach.

**Why out-of-scope for P104:** P104 closes the 404 bug fix verification; the catalog schema validation gap is an infrastructure issue. It was surfaced during verification but requires a new gate in the structure dimension that does not yet exist.

**Sketched resolution:** Add a structure-dimension gate (`quality/gates/structure/verifier-script-exists.sh`) that scans all catalog rows at load time and asserts: for each row with a non-null `verifier.script`, the file exists on disk and is executable (chmod +x). The gate would fail at pre-commit or pre-push if any row references a missing verifier, preventing unbacked PASS rows from landing. This is a complement to GOOD-TO-HAVES-01 (bind-verb extension for agent-ux rows).

**STATUS:** OPEN

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

**STATUS:** RESOLVED by **P113** (`113-lost-update-shared-cursor/PLAN.md`). Fix = option
(a): `precheck_export_against_changed_set` no longer GATES the per-record version check on
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

## 2026-07-12 08:35 | discovered-by: P105 (RBF-LR-03 rebase-recovery gate, Lane 2) | severity: HIGH

**What:** The P105 parent-chaining fix (`90ddaff`, `fast_import.rs` emits `from
<tracking-tip>`) is INCOMPLETE. It eliminated the `fatal: error while running fast-import` /
`does not contain` abort — verified — but driving the FULL documented recovery
end-to-end through a real `git pull --rebase` (not the isolated `git fast-import` that
`verify_fix.sh` / the `git_fast_import_roundtrip_with_parent_fast_forwards` unit test
exercise) surfaces a SECOND fetch-time abort:
```
error: cannot lock ref 'refs/reposix/origin/main': is at <X> but expected <Y>
 ! <Y>..<X>  main -> refs/reposix/origin/main  (unable to update local ref)
```
**Root cause (file:line):** the `import` helper's fast-import stream writes `commit
refs/reposix/origin/main` DIRECTLY (`crates/reposix-remote/src/fast_import.rs:165`, and
the no-op-guard `reset` at `:142`), while the helper ALSO advertises `refspec
refs/heads/*:refs/reposix/origin/*` (`crates/reposix-remote/src/main.rs:193`). So git
applies its OWN refspec update to `refs/reposix/origin/main` AFTER import. On drift the
stream fast-forwards the ref underneath git; git's post-import ref transaction (old-value
= the PRE-fetch tip) then finds the ref already moved → `cannot lock ref` → `git fetch`
(hence `git pull --rebase`) exits non-zero. Because the documented recovery is the single
command `git pull --rebase && git push`, the non-zero pull SHORT-CIRCUITS at the `&&` and
`git push` never runs — the agent's unpushed edit is LOST until an UNDOCUMENTED second
`git pull --rebase` (the ref is now settled + the no-op guard makes the re-fetch a clean
fast-forward, so pull #2 exits 0 and push converges). **Empirically reproduced** (live
sim, git 2.25.1, independent caches, both scenarios — peer git-push AND external REST
PATCH drift): `git pull --rebase && git push` exits 1, SoT record version unchanged; a
plain `git fetch origin` on drift ALSO exits 1 with the same `cannot lock ref`; a second
`git pull --rebase && git push` exits 0 and converges. `reposix sync --reconcile` does NOT
help (it rebuilds the CACHE, not the client's tracking ref). Repro:
`.planning/phases/105-rbf-lr-03-rebase-recovery/repro/repro-fetch-ref-lock.sh`. Gate that
catches it (currently NOT-VERIFIED): `quality/gates/agent-ux/rebase-recovery-reconciles.sh`
(transcript under `quality/reports/transcripts/rebase-recovery-reconciles-*.txt`).

**Why out-of-scope for P105:** P105's charter + landed fix target the parentless
non-descendant commit (the `does not contain` abort), which is fixed. This second abort is
in git's remote-helper `import` ref-update contract — the helper double-writes the tracking
ref that git also owns. A correct fix (e.g. NOT naming `refs/reposix/origin/main` in the
stream and letting git's refspec do the single authoritative update, or writing to a
neutral staging ref) touches load-bearing protocol code where a naive change risks
CLOBBERING the caller's local branch on a lights-out system — it needs its own design +
cross-git-version testing (import vs stateless-connect, §5 still unverified on modern git).
That is a dedicated fix phase, not a <1h in-lane eager fix; folding it into P105 would
double the phase scope and re-open the exact "client-side special-case patch" smell the
P105 dispatch forbade.

**Sketched resolution:** (a) Stop emitting `commit/reset refs/reposix/origin/main` in the
`import` stream; write the commit to the ref name git requested (the `import <ref>` LHS,
e.g. `refs/heads/main`) or a helper-private staging ref, and let git's advertised refspec
perform the SINGLE authoritative `refs/reposix/origin/main` update — matching the
git-remote-testgit reference-helper pattern — so there is no double-write. Verify the
caller's local `main` branch is NOT touched. (b) Failing that, drop `refs/reposix/origin/*`
from the advertised import refspec so git does not attempt its own update after the stream
writes the ref. Either way: re-run `quality/gates/agent-ux/rebase-recovery-reconciles.sh`
— it is written to flip to PASS (exit 0) WITHOUT edits once the single documented
`git pull --rebase && git push` exits 0 and the edit reaches the SoT. Also resolve PLAN §5
(does stateless-connect on git >= 2.34 exhibit the same or a different failure?) on a
modern-git CI runner before closing.

**STATUS:** OPEN

---

## 2026-07-12 09:40 | discovered-by: P105 (RBF-LR-03 docs fix-twice lane, ownership noticing) | severity: MEDIUM

**What:** `resolve_import_parent()` (`crates/reposix-remote/src/main.rs:400-419`) degrades
to the parentless path (`None` → no `from`, no `deleteall`) on **any** git error, not just
ref-absence. Two conflations: (1) the `rev_parse` closure returns `None` via `.ok()?`
(`main.rs:407`) when `Command::new("git").output()` itself fails — git binary missing, spawn
/ I/O error — swallowing a real environmental fault as "no parent." (2) `!out.status.success()`
(`main.rs:408`) treats *every* non-zero rev-parse exit as ref-absent, when
`rev-parse --verify --quiet` also exits non-zero for other failure modes. Either way the
fetch silently falls back to a parentless overlay — precisely the non-descendant
"does not contain" abort P105 just fixed (`fast_import.rs` parent-chaining). A future
regression that makes rev-parse fail for a non-absence reason would silently re-open the
RBF-LR-03 bug with no error surfaced to the operator.

**Sketch:** Distinguish ref-absent (the legitimate parentless case: `rev-parse --verify
--quiet <ref>` exits 1 with empty stdout AND the git spawn succeeded) from other rev-parse
/ spawn failures. On the latter, error the fetch loudly (`fatal:` + recovery hint) instead
of degrading to parentless — a spawn failure or a malformed-arg exit is an operator-facing
fault, not a valid "fresh clone, no tracking tip yet" signal. Keep the existing empty-stdout
→ `None` path for the genuine first-fetch case. Add a unit test that injects a non-absence
git failure and asserts the fetch errors rather than emitting a parentless overlay.

**Why out-of-scope for the P105 docs lane:** code change in load-bearing helper protocol
(cargo build + a new test) — this lane is docs-only, cargo-free. Small (<1h) but needs a
cargo window; file rather than eager-fix.

**STATUS:** OPEN

---

## 2026-07-12 | discovered-by: D2 re-seal Wave 1 (shell/planning lane) | severity: HIGH

**Title:** live D2 repro — post-P102 shared-tree corruption via subprocess/worktree bypass.

**What:** A live recurrence of the founding S-260707-pr-08 corruption class occurred
AFTER P102 shipped GREEN. A P106 leaf subagent created a git worktree INSIDE
`.claude/worktrees/agent-a98058321e3f649a7` of the SHARED repo (not a `/tmp` clone) and ran
`reposix init` / sim-seed via a path that does NOT go through the Claude Code Bash tool, so
the PreToolUse leaf-isolation hook (Bash-tool-only — coverage boundary documented at
`.claude/hooks/leaf-isolation-guard.sh` header, "COVERAGE BOUNDARY") never fired. Symptoms:
shared `.git/config` `core.bare=true`, `origin` repointed to the sim at
`127.0.0.1:7988`, `HEAD` thrashed to `e18df81`, and `refs/reposix/*` polluted. The shared
tree was repaired at commit `9d78d62`. This is a NEW recurrence, distinct from the founding
`S-260707-pr-08` — same corruption end-state, different bypass path (non-Bash-tool
subprocess + in-shared-repo worktree rather than a "forgot-to-cd" Bash-tool leaf).

**Guard defects found + FIXED in this Wave-1 shell/planning lane** (Bash-tool-layer
defense-in-depth, `.claude/hooks/leaf-isolation-guard.sh`; regression asserts in
`quality/gates/agent-ux/fleet-safety-leaf-isolation-enforce.sh` Cases 9-11):
- **(A) config-read false-positive** — Guard C misclassified a `git config` READ as a WRITE
  whenever the guarded key was followed by ANY trailing token (`2>/dev/null`, `&& echo`,
  `| grep`), LIVE-BLOCKing coordinators that merely read `core.bare`/`user.email`. Fixed:
  read-flag detection (`--get*`/`--list`/`-l`) + segment isolation (cut at first shell
  separator, strip redirections) before the value-token write-heuristic. Real writes
  (`git config core.bare true`, `--unset`, `--replace-all`) still BLOCK.
- **(B) git-init-bare gap** — bare/`--bare` `git init` in the shared tree (the founding
  `core.bare=true` end-state) was not blocked by any guard. Fixed: added to Guard B.
- **(C) cargo sim-seed spelling gap** — `cargo run -p reposix-sim -- seed …` slipped Guard B
  (`reposix-sim` sits at an ARGUMENT position under `cargo run`, not command position).
  Fixed: added `cargo run … -p reposix-sim` + `cargo run … -- seed` to Guard B.

**Why the hook fix is NOT the whole cut:** the hook is Bash-tool-only by construction. The
live repro bypassed it entirely (subprocess + in-repo worktree). The hook hardening closes
the Bash-tool spellings, but a subprocess bypass remains reachable — the honest coverage
boundary is preserved in the hook header, not deleted.

**THE REAL CUT (follow-up — v0.14.0 Wave 2):** a BINARY-SIDE refusal in `reposix init` /
sim-seed itself, since only that layer can stop a subprocess bypass. Sketch: `reposix init`
(NOT `attach` — attach legitimately adopts an existing checkout) refuses when its effective
target would nest inside the reposix SOURCE checkout / shared dev tree, WITHOUT breaking the
sanctioned `/tmp` dark-factory flow. Pair with a self-safety check that refuses to operate
when the effective `.git` is the shared repo's object store (worktree-shared config detected).

**STATUS:** OPEN

## 2026-07-12 15:57 | discovered-by: C2-wave-2 (CI-gate fix-twice) | severity: MEDIUM

**What:** `.github/workflows/release.yml` (tag `v*` trigger) is CI-UNGATED — a tag cut over
a red main would still publish crates. This session gated the phase-close path and `docs.yml`
(D-CONV-4 `workflow_run`/`if success` pattern), but the tag-publish path has no CI-green
precondition.

**Why out-of-scope for the discovering work:** surfaced during the CI-gating fix-twice work,
not owned by any wave-2 implementation phase; the aggregate `v*` tag is owner-cut at
milestone-close (P111), so the fix belongs on the P110/P111 radar, not inside P106.

**Sketched resolution:** gate `release.yml` on CI green BEFORE the next `v*` tag — mirror the
`docs.yml` D-CONV-4 `workflow_run` + `if: success` pattern, or add a pre-tag CI-green
assertion step. If gating turns out <1h with no new dependency during P110/P111, eager-fix;
otherwise leave filed for the owner (the tag itself is owner-cut).

**STATUS:** OPEN

## 2026-07-12 | discovered-by: GSD-quick (release-plz RED fix) | severity: MEDIUM

**What:** Fold release-plz (and other required workflows) into the ci-green-on-main phase-close
bar. This session fixed a persistently-RED release-plz on main (it refused a dirty CI checkout
re-dirtied by self-regenerating fleet-safety verdict JSONs) — but the RED sat UNNOTICED because
the phase-close `code/ci-green-on-main` (P0) probe hardcodes `WORKFLOW=ci.yml` and watches ONLY
ci.yml. release-plz was never on the phase-close radar, so an unwatched red release workflow
rotted silently (Global CLAUDE.md: health is a maintained asset; never let a metric you don't
watch decay). Sibling of the release.yml-CI-ungated entry above (that one is about GATING the
tag-publish on green; this one is about WATCHING release-plz's outcome at phase-close).

**Why out-of-scope for the discovering quick:** the quick's charter was the dirty-checkout fix;
widening the phase-close bar is a catalog + verifier change with open semantic questions (below)
that warrant an owner gate before P0-wiring — not inline scope.

**Sketched resolution:** parameterize `quality/gates/code/ci-green-on-main.sh`'s hardcoded
`WORKFLOW=ci.yml` into a required-workflow LIST, OR add a sibling `code/release-green-on-main`
row at post-push cadence reusing the same latest-run-conclusion logic. Catalog-first (write the
GREEN-contract row before impl) + a verifier grade.

**Open questions to resolve FIRST (a false-RED would block UNRELATED phases → owner gate):**
(1) Does release-plz run on EVERY push to main? (2) Is a 'no release needed' run concluded
`success` / `skipped` / other — so the probe treats non-failure correctly and does not false-RED
unrelated phases?

**STATUS:** OPEN

## 2026-07-12 | discovered-by: GSD-quick (fleet-safety untrack fix) | severity: MEDIUM

**Title:** Runner unit tests (`quality/runners/test_*.py`) are not collected by CI — durable
guards never run automatically.

**What:** The fleet-safety untrack fix (`3d3e60e`) shipped its DP-2 regression guard
`quality/runners/test_fleet_safety_verdicts_untracked.py` — but none of the 6
`quality/runners/test_*.py` unittest files are wired into `.github/workflows/ci.yml`. CI never
collects them, so the guard is inert in CI: a metric generated-but-not-watched, GREEN only on a
local run.

**Why out-of-scope for the discovering quick:** the quick's charter was the dirty-checkout /
untrack fix; wiring a CI collection step (with per-file env-safety triage + an owner gate on
which subset is hermetic) is a workflow + catalog change, not inline scope.

**Sketched resolution:** triage the 6 test files for env-safety (which need creds/network, e.g.
`test_realbackend`, vs pure/hermetic) then add a CI step running the env-safe subset (e.g.
`python3 -m unittest` over the vetted files) OR promote them to catalog gates under an
appropriate cadence. `test_fleet_safety_verdicts_untracked.py` MUST be one that gets wired. Do
NOT blanket `unittest discover` — that surfaces env-dependent tests.

**STATUS:** OPEN
