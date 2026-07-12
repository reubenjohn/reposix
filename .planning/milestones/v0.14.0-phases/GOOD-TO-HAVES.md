# v0.14.0 GOOD-TO-HAVES

> **Purpose.** OP-8 +2 reservation Slot 2 — improvements (clarity, perf, consistency,
> grounding) the planned phases observed but didn't fold in. Sized XS / S / M; XS items
> always close; M items default-defer to the next milestone. Drained by P111
> (good-to-haves polish + milestone close, Slot 2 of the v0.14.0 +2 reservation).

_Append below this line as polish opportunities surface._

## GOOD-TO-HAVES-01 — `leaf-isolation-nontool-backstop` — non-tool backstop for guards B/C

**Discovered during:** P102

**Size:** M (rough effort estimate)

**Source:** the P102 leaf-isolation guards live in a PreToolUse Bash-*tool* hook
(`.claude/hooks/leaf-isolation-guard.sh`). By construction the hook fires only on the
Claude Code Bash tool — a `reposix init` / `git config` write spawned by a subprocess or a
shell script bypasses it entirely. The `.githooks/pre-commit` git-native backstop closes
the *commit* leg on that bypass path (guard A), but there is NO non-tool backstop for guard
B (leaf-setup location) or guard C (shared-`.git/config` write): a script that shells out
`reposix init .` or `git config core.bare true` in the shared tree is unguarded.

**Acceptance:** a non-tool enforcement point catches guard-B/C violations regardless of how
the command is spawned. Candidate designs: (a) a `reposix`-binary-side check — `reposix
init|attach|sync` refuses to run when cwd resolves inside a repo whose `origin` is the
coordinator's shared checkout and no explicit `--allow-shared` / `/tmp` target is given;
(b) a git alias / `core.fsmonitor`-adjacent wrapper that intercepts `git config core.bare`
in the shared repo; (c) a filesystem-level guard (read-only bind of the shared `.git/config`
during autonomous runs). Proven by a transcript showing a scripted (non-Bash-tool) bypass
attempt is blocked.

**Default disposition for P111:** M default-defers to the next milestone with a named
carry-forward target (v0.15.0 fleet-safety hardening); close early only if a cheap
binary-side check (design (a)) proves < 1h.

**STATUS:** OPEN

## Entry format

```markdown
## GOOD-TO-HAVES-NN — `<short-id>` — one-line title

**Discovered during:** P<N>

**Size:** XS|S|M (rough effort estimate)

**Source:** where this was noticed.

**Acceptance:** what "done" looks like.

**Default disposition for P111:** XS always closes; S closes-or-defers; M default-defers
to the next milestone with a named carry-forward target.

**STATUS:** OPEN
```
