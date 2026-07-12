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

## 2026-07-12 07:13 | discovered-by: P104 (github-helper-path 404 fix verifier) | severity: MEDIUM

**What:** A catalog row was minted `status: PASS` with a `verifier.script` path that did not exist on disk (`quality/catalogs/agent-ux.json`, P104 BLOCKER that was caught only during manual code review). The row `agent-ux/p87-surprises-absorption` was defined with status FAIL but lacked a `claim_vs_assertion_audit` field required by the schema (rows minted after 2026-05-08 must include this field for honesty auditing). The pre-commit hook validation does not structurally verify that a row's declared `verifier.script` path exists or is executable — only that the JSON is valid. This opens a window where a coordinator could mint a PASS row backed by a missing or non-executable verifier, creating a false-positive contract breach.

**Why out-of-scope for P104:** P104 closes the 404 bug fix verification; the catalog schema validation gap is an infrastructure issue. It was surfaced during verification but requires a new gate in the structure dimension that does not yet exist.

**Sketched resolution:** Add a structure-dimension gate (`quality/gates/structure/verifier-script-exists.sh`) that scans all catalog rows at load time and asserts: for each row with a non-null `verifier.script`, the file exists on disk and is executable (chmod +x). The gate would fail at pre-commit or pre-push if any row references a missing verifier, preventing unbacked PASS rows from landing. This is a complement to GOOD-TO-HAVES-01 (bind-verb extension for agent-ux rows).

**STATUS:** OPEN
