---
phase: 122-remote-init-hardening
reviewed: 2026-07-18T00:00:00Z
depth: deep
files_reviewed: 12
files_reviewed_list:
  - crates/reposix-cli/src/init.rs
  - crates/reposix-remote/src/main.rs
  - .claude/hooks/leaf-isolation-guard.sh
  - crates/reposix-core/src/codes.rs
  - crates/reposix-cli/tests/errors_teach_recovery.rs
  - quality/gates/agent-ux/init-refuses-nested-in-shared-tree.sh
  - quality/gates/agent-ux/import-parent-resolve-fails-loud.sh
  - quality/gates/agent-ux/rebase-recovery-reconciles.sh
  - quality/gates/agent-ux/teach_scan.py
  - quality/catalogs/agent-ux.json
  - crates/CLAUDE.md
  - docs/reference/error-codes.md
findings:
  critical: 0
  warning: 2
  info: 3
  total: 5
status: issues_found
---

# Phase 122: Code Review Report

**Reviewed:** 2026-07-18
**Depth:** deep (cross-file: init.rs ↔ leaf-isolation-guard.sh ↔ codes.rs ↔ tests ↔ gates ↔ catalog)
**Files Reviewed:** 12
**Status:** issues_found (no BLOCKER / no HIGH — SHIP-WITH-NITS)

## Summary

P122 adds two binary-side hardening cuts against the D2 shared-tree-corruption
recurrence and a helper-side loud-fail for the import-parent resolve. I traced
every security-critical path against the stated threat model.

**The core security logic is sound and well-tested:**

- **`resolve_import_parent` tri-state (RPX-0508)** correctly partitions
  spawn-fail / exit-0-empty / non-1 exit / signal → loud `Err`, vs genuine
  exit-1 ref-absence → `Ok(None)`. No silent-degradation path remains; the
  error is routed through `fail_push` at the call site (not swallowed). The
  injected-runner tests exercise every arm. `what` is built only from local git
  exit codes / `io::Error` / static strings — `real_rev_parse` reads only
  `out.stdout`, never `out.stderr`, so no git message or remote byte can ride
  the RPX-0508 diag (OP-2 clean).
- **Init latch 1 (`refuse_nested_in_worktree`)** canonicalizes via
  `canonicalize_lexical_existing` (realpath-`-m` semantics): relative paths,
  `..` collapsed across the FULL path, and symlinks resolved through the deepest
  existing ancestor are all handled — the symlink-smuggle (test c) and
  whole-path `..` collapse (unit test) prove it. The `/tmp` safe zone is
  component-wise (`/tmpfoo` ≠ `/tmp`) and mirrors the hook's `is_safe`.
- **Init latch 2 (`assert_own_git_dir`)** is the *precise* cut: it aborts
  fail-closed BEFORE any `git config` write whenever the resolved git-dir ≠
  `<path>/.git` (GIT_DIR injection / worktree gitfile). Any rev-parse failure
  → refuse. Test (e) genuinely fires it (GIT_DIR-injected shared store, config
  byte-unchanged assertion), not a no-op.
- **Hook agreement:** the `leaf-isolation-guard.sh` +9 is a comment-only change
  (documentation of the now-shipped binary cut); `is_safe` logic is byte-identical.
  The binary and hook AGREE on the load-bearing `/tmp` decision. The binary is
  deliberately *stricter* than the hook on GIT_DIR binding (latch 2) and
  *looser* on non-nested non-/tmp inits (intentional, so real end-users can init
  outside /tmp where no shared tree exists) — neither divergence reopens the
  corruption vector.
- **Teaching quality:** RPX-0406 (both sites) and RPX-0508 are 3-part (fix /
  alternative / recovery); codes.rs extended explanations match behavior; both
  codes are emitted (registry reverse-completeness holds); `teach_scan.py`
  `teach_coded(` recognition landed with a self-test that passes.
- **Gate honesty:** both new gates RUN `cargo test` and grep `... ok` per named
  test (not shape-only greps); the rebase gate's stateless-connect legs carry a
  real `GIT_TRACE_PACKET` `command=fetch`+`version 2` wire proof and adjudicate
  divergence loudly (NOT-VERIFIED), never a faked green.

Findings below are quality / doc-truth / UX-friction items, none security-incorrect.

## Warnings

### WR-01 (MEDIUM): latch 1 is broader than the corruption vector, and its stated cause overstates the mechanism

**File:** `crates/reposix-cli/src/init.rs:235-294` (`refuse_nested_in_worktree`);
`crates/reposix-core/src/codes.rs:552-576` (RPX-0406 `cause`)

**Issue:** Latch 1 refuses *any* fresh target nested inside a non-/tmp git
working tree. But a fresh `git init` in a subdir of an enclosing repo does NOT
corrupt the enclosing repo in standard git — it creates an independent nested
repo. The actual D2 corruption is *git-dir binding* (GIT_DIR / worktree gitfile
/ cwd-reset landing config writes on the shared `.git/config`), which latch 2
(`assert_own_git_dir`) covers precisely. Consequently:

1. The RPX-0406 `cause` ("initializing a repo there risks corrupting the
   enclosing repository") and the `refuse_nested_in_worktree` docstring
   ("initializing a repo there risks corrupting the enclosing repository — the
   exact shape behind the 2026-07-12 shared-tree incident") overstate latch 1's
   mechanism. Latch 1 is a conservative heuristic; latch 2 is the precise cut.
2. It is an over-broad refusal that will block *legitimate* end-users: anyone
   running `reposix init github::x/y ./demo` inside their own project checkout,
   or under a git-managed `$HOME` (dotfiles/yadm), outside /tmp — a
   Rust-compiler-grade-UX concern (OD-3 #5). A skeptical first-time dev hitting
   RPX-0406 on a non-corrupting operation may be confused.

Mitigations already present: the refusal teaches /tmp + `reposix attach` with
copy-paste recovery, and fail-closed conservatism is defensible in the
dark-factory. This is over-*safe*, not under-safe — hence WARNING, not BLOCKER.

**Fix:** Make the cause honest that latch 1 is a *conservative heuristic*
backstop and that the precise corruption cut is the git-dir self-check (latch 2)
— OR narrow latch 1 to only refuse when the enclosing repo's git-dir is the one
`git init` would actually bind (which converges on latch 2). At minimum, soften
the RPX-0406 `cause` / docstring so the doc-truth claim matches the mechanism.

### WR-02 (LOW): the new refusal couples init-driving gates to `/tmp` via `TMPDIR`; `is_tmp_safe` omits macOS `/var/folders` despite the `/private/tmp` "(macOS)" claim

**File:** `crates/reposix-cli/src/init.rs:226-233` (`is_tmp_safe`);
gates using `mktemp -d`/`mktemp -d -t` (e.g. `quality/gates/agent-ux/zero-shot-onboarding.sh:27`)

**Issue:** Now that non-/tmp nested inits are refused, any gate that
`reposix init`s into a `mktemp -d`/`-t` dir depends on `TMPDIR` resolving under
`/tmp`. On Linux/CI (ubuntu, `TMPDIR` unset → `/tmp`) this holds, and the
load-bearing `dark-factory/sim.sh` hardcodes `/tmp/…` — so the actual targets
are green. But it is latent fragility: a runner with `TMPDIR` set to a non-/tmp
path inside the repo checkout would newly RED. Separately, `is_tmp_safe` claims
macOS support via `/private/tmp` (the realpath of an explicit `/tmp`), yet
macOS's actual `mktemp` default is `/var/folders/…`, which is NOT covered — so
the "(macOS)" safe-zone claim is incomplete. (Project is Linux-only, so this is
LOW.)

**Fix:** Either document that init-driving gates MUST use an explicit `/tmp`
path (not `mktemp -d`), or extend `is_tmp_safe` / the comment to reflect the
real macOS temp root if macOS is ever a target. The unit tests already worked
around this by forcing `.tempdir_in("/tmp")`, which confirms the coupling.

## Info

### IN-01 (LOW): import (fetch) path emits a push-shaped protocol line

**File:** `crates/reposix-remote/src/main.rs:424-435` + `fail_push:341-353`

**Issue:** On the `import` verb, the RPX-0508 failure is surfaced via
`fail_push(proto, state, "import-parent-resolve-failed", …)`, which writes
`error refs/heads/main import-parent-resolve-failed` to stdout — a *push-shaped*
reject line on a *fetch* verb (git's fast-import reader expects a stream). The
actual teaching correctly rides `diag` on stderr, so the user still sees the
3-part RPX-0508 message. This mirrors the pre-existing RPX-0507 arm exactly, so
it is consistent with established (presumably-tested) behavior — flagged only so
the pattern is on record, not as a P122 regression.

**Fix:** None required. If the import-path protocol line is ever revisited,
consider whether git surfaces a `error refs/heads/main …` line on the import
capability or merely reports a generic malformed-stream error.

### IN-02 (LOW): latch 2 guarantees "no config write reaches shared", not "no git command touches shared"

**File:** `crates/reposix-cli/src/init.rs:497-503` (`git init` runs before `assert_own_git_dir`)

**Issue:** `run_git(&["init", path_str])` executes BEFORE `assert_own_git_dir`.
In the GIT_DIR-injection case, `git init <path>` re-initializes the *shared*
store first. This is benign (`git init` re-init does not flip `core.bare`, alter
config, or move refs — test (e) confirms config is byte-unchanged), and the
config writes that WERE the D2 corruption are correctly gated behind latch 2.
Worth stating explicitly: latch 2's guarantee is "no `git config` write reaches
the shared config," not "no git subprocess touches the shared dir."

**Fix:** None required (the residual is provably non-destructive). Optionally
tighten the `assert_own_git_dir` doc comment to state the guarantee precisely.

### IN-03 (NIT): RPX-0406 cause prose duplicates the D2 narrative across three surfaces

**File:** `crates/reposix-core/src/codes.rs:552-576`, `crates/reposix-cli/src/init.rs:235-249`, `crates/CLAUDE.md`

**Issue:** The "2026-07-12 shared-tree incident" narrative + realpath-mirroring
rationale is restated in the RPX-0406 registry `cause`, the `refuse_nested_in_worktree`
docstring, and crates/CLAUDE.md. Fine per fix-twice, but if WR-01's mechanism
wording is corrected, all three must be updated together to avoid re-drift.

**Fix:** When addressing WR-01, correct the mechanism claim in all three places
in the same commit (fix-twice / doc-alignment).

---

## Noticing (OD-3 §2)

- **Over-broad refusal vs precise cut (WR-01):** the two latches are not
  peers — latch 2 is the real corruption cut; latch 1 is conservative
  belt-and-suspenders that will bite legit users. The security posture is
  fail-*safe* (over-refuses), which is the right direction, but the docs present
  latch 1 as the corruption mechanism when it is not.
- **Hook +9 is comment-only** — verified `is_safe` logic byte-identical;
  the binary/hook AGREE on the /tmp zone and the binary adds strictness (latch 2
  catches GIT_DIR binding the hook cannot see). No divergence reopens the vector.
- **Tests assert what their names promise:** case (e) genuinely fires latch 2
  (config byte-unchanged), the tri-state tests drive every arm, and both gates
  RUN the tests + grep per-test `... ok` (not vacuous shape greps). The rebase
  gate's stateless-connect legs carry a real `GIT_TRACE_PACKET` transport proof.
- **teach_scan `teach_coded(` regex** (`\bteach(?:_coded)?\s*\(`) does not
  over-match (`foo_teach(` / `reteach(` blocked by `\b`); the new anyhow! in
  `resolve_import_parent_with` is correctly dispositioned with a `teach-exempt`
  marker within the 2-line window; self-test passes.
- **No taint leak:** neither change routes a remote/tainted byte outbound; the
  egress allowlist is untouched; RPX-0402 redacts git stderr via `redact_userinfo`,
  and RPX-0508 never surfaces git stderr at all.
- **Could not independently run cargo** (one-cargo mutex held for phase-close) —
  correctness of the Rust tests is inferred from static reading + the committed
  gate scripts that execute them; the phase-close cadence run is the authority.

---

_Reviewed: 2026-07-18_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: deep_
