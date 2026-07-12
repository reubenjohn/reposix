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

## GOOD-TO-HAVES-02 — `file-size-drain-residual` — 56 files still over the progressive-disclosure budget

**Discovered during:** P103 Lane B (OP-8 file-size debt drain)

**Size:** M (rough effort estimate — 56 files across three extensions)

**Source:** `bash quality/gates/structure/file-size-limits.sh` (blocking mode). Lane B
split the two BIG offenders — `.planning/milestones/v0.13.0-phases/{SURPRISES-INTAKE,
GOOD-TO-HAVES}.md` (109K + 129K) — into per-part child dirs (`surprises-intake/`,
`good-to-haves/`, byte-exact, INDEX rewritten), dropping the count 58 → 56 and adding the
reusable `scripts/split_ledger.py`. The residual 56 (45 `*.md` / 6 `*.py` / 5 `*.sh`) were
deliberately deferred, each for a concrete reason:

- **6 `*.py` runners** — `quality/runners/{run,verdict,_audit_field,test_audit_field,
  test_realbackend}.py` + `scripts/shell_coverage.py`. `_audit_field.py` + `test_audit_field.py`
  were being modified by a CONCURRENT cargo/runner lane during P103; splitting them is a
  module refactor with collision risk (charter #3 — new coupling, not <1h-safe). Sketch:
  extract `_freshness.py`/`_row.py` helpers from `run.py`/`verdict.py`; move `_audit_field`
  parsing into a sibling module; split the two big test files by fixture group.
- **5 `*.sh` gates/hooks** — `.claude/hooks/leaf-isolation-guard.sh`, `.githooks/test-pre-push.sh`,
  `quality/gates/agent-ux/{dark-factory/dvcs-third-arm,real-git-push-e2e,zero-shot-onboarding}.sh`.
  Each is a single-responsibility script; splitting needs a `lib/` source-helper convention
  (new dependency between scripts) — factor in one focused pass, not mid-flight.
- **`.planning/milestones/*/ROADMAP.md` (v0.13.0 60K, v0.13.2 30K, v0.14.0 30K)** — GSD
  tooling + the `no-loose-roadmap-or-requirements` invariant couple to the monolithic
  ROADMAP.md path; splitting risks the layout rule (`.planning/CLAUDE.md` § Milestones
  layout). Defer until a ROADMAP-shard convention exists.
- **Remaining `*.md`** — archived phase bundles / research chapters (`.planning/phases/89-91/*`,
  `.planning/research/v0.13.0-real-backend-frictions/*`), live planning ledgers
  (`STATE.md` — concurrent-writer, never hand-edit; `PROJECT.md`, `CONSULT-DECISIONS.md`,
  `ORCHESTRATION.md`, `RETROSPECTIVE.md`), and docs (`docs/guides/troubleshooting.md`,
  `docs/reference/cli.md`, `quality/PROTOCOL.md`). Each is a real split but per-file
  judgement; `scripts/split_ledger.py` handles the `## `-delimited ledgers directly.

**Acceptance:** `bash quality/gates/structure/file-size-limits.sh` exits 0 with NO waiver.
Drive the residual toward zero the same way Lane B did the intakes: reuse
`scripts/split_ledger.py` for ledgers, factor a `lib/` convention for the `*.sh`, extract
runner helpers for the `*.py`. The `structure/file-size-limits` waiver in
`freshness-invariants.json` was NARROWED by Lane B to name only these residual files (the
two split intakes were removed from it).

**Default disposition for P111:** M default-defers to the next milestone (post-v0.14.0);
close individual extensions early (e.g. the 5 `*.sh` in one pass) if < 1h each.

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
