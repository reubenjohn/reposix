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
