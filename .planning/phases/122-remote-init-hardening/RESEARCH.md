# P122 RESEARCH — `reposix-remote` + `init` hardening

> Inline research answering the three open questions the PLAN required. All claims
> verified against committed source / the live environment on 2026-07-17.

## GTH-V15-04 (DRAIN-07) — stateless-connect verification on git ≥ 2.34

### What git does CI actually run?
`.github/workflows/ci.yml` job `quality-pre-pr` (name "quality gates (pre-pr)") runs on
`runs-on: ubuntu-latest`. `ubuntu-latest` is currently Ubuntu 24.04 (noble), which ships
git ≥ 2.43; even the prior 22.04 image ships git ≥ 2.34. So **CI already runs a
modern-git (≥ 2.34) runner.**

### Does the gate run on that runner today, and does it hit stateless-connect?
The catalog row `agent-ux/rebase-recovery-reconciles` carries
`cadences: ["pre-push","pre-pr","on-demand"]` — so the CI `quality-pre-pr` job **already
runs the gate on modern git**. BUT the gate (`rebase-recovery-reconciles.sh:157-159`)
exports `GIT_CONFIG protocol.version=0` for EVERY git subprocess, forcing the legacy
`import` fetch path even on modern git. On a `git ≥ 2.34` run it does NOT exercise the
`stateless-connect` (protocol-v2) read path — it only writes a **TODO** to the transcript
(`rebase-recovery-reconciles.sh:485-491`). So today's "13/13 asserts PASS on modern-git
CI" proves ONLY the import path; the stateless-connect read path is unexercised. This is
a coverage gap, not an unfixed push-correctness defect (per GTH-V15-04 wording).

### The environment moved: local git is now 2.50.1
The gate's header comments ("only git 2.25.1 is installed") are **STALE**. `git --version`
on this VM reports **2.50.1** (≥ 2.34). The stateless-connect path is therefore
exercisable **locally**, in a /tmp clone, without waiting for CI. The executor verifies
against reality locally AND confirms on CI's ubuntu-latest.

### P105 PLAN §5 — the exact open item to resolve (quoted)
`.planning/milestones/v0.14.0-phases/105-rbf-lr-03-rebase-recovery/105-PLAN.md:171-183`:

> ## 5. Open question the executor MUST resolve (does NOT block the fix)
> **Does the `stateless-connect` (protocol-v2) fetch path — used by git ≥ 2.34 — ALSO
> break, or does it already fast-forward off the cache's chained history?** The cache's
> `refs/heads/main` is a correct linear chain (verified), so a stateless-connect fetch
> *likely* fast-forwards and the bug may be `import`-path-only (i.e. git-version-scoped).
> … **Executor action:** on CI's git, run both scenarios via the *stateless-connect* path
> and record the result in the gate. If stateless-connect already works, the `import` fix
> still ships (old-git support) and the gate must force the `import` path to keep guarding
> it. If stateless-connect ALSO breaks, that is a SECOND fix site (cache-side) — file it,
> do not silently expand this lane.

P105 §10.4 additionally notes the git ≥ 2.34 stateless-connect path emits no ref writes
(git owns ref placement via protocol-v2 + `remote.origin.fetch`), so the §5 **hypothesis
is convergence** — but it is UNVERIFIED and must be exercised, not assumed.

### Decision: EXTENSION of the existing gate (not a new gate, not a CI-matrix change)
The resolution is to **extend `rebase-recovery-reconciles.sh`** so that on `git ≥ 2.34`
it ALSO runs both drift scenarios via the real stateless-connect path (protocol.version
UNSET) and emits a deterministic verdict per scenario — never a bare TODO. The existing
import-path legs stay (they guard old-git support). No CI-matrix change is needed (CI is
already modern git; the gate self-detects and runs the extra legs). Two possible end
states, both acceptable per ROADMAP SC1:
- **Converge** (expected): add stateless-connect PASS asserts; the gate now proves BOTH
  paths on modern git.
- **Diverge**: file a HIGH SURPRISES-INTAKE row naming the cache-side second fix site,
  and gate the divergence with a labelled, deterministic known-divergence guard (NOT a
  silent skip, NOT a faked green). Do NOT expand this lane to fix the cache side.

## GTH-V15-05 (DRAIN-08) — `resolve_import_parent()` loud failure

### Exact location + current behavior
`crates/reposix-remote/src/main.rs:450-469` (the ROADMAP's `400-419` / `main.rs:400-419`
citation is off-by-position — the fn body is **450-469**; the `422` call site is inside
`handle_import_batch`). The inner `rev_parse` closure (455-463) returns `None` when:
1. `Command::new("git")…output().ok()?` — the git SPAWN itself fails (binary missing /
   I-O error) → swallowed as "no parent".
2. `!out.status.success()` — ANY non-zero exit (ref-absent exit-1 AND a corrupt-repo /
   bad-object exit-128) → all collapse to "no parent".

`resolve_import_parent()` returns `Option<ImportParent>`; the caller at `main.rs:422`
(`let parent = resolve_import_parent();`) treats `None` as the legitimate parentless
first-fetch seed. A future non-absence rev-parse failure would silently re-open the
RBF-LR-03 non-descendant `does not contain` abort with no operator-facing error.

### How git errors surface
`git rev-parse --verify --quiet <ref>`: ref genuinely absent → exit **1**, empty stdout,
spawn succeeded. Non-absence fault (not-a-repo / corrupt object / bad revision) → exit
**128** (with stderr). git-not-on-PATH → `Command…output()` returns `Err` (no exit code).

### Fix + test-injection strategy
Change `rev_parse` to distinguish the three: return `Ok(None)` ONLY for spawn-success +
exit-1 + empty-stdout (genuine absence); return `Err(...)` for a spawn failure OR a
non-1 non-zero exit. Lift `resolve_import_parent()` to `Result<Option<ImportParent>>` and
have the `handle_import_batch` caller propagate the `Err` loudly through the existing
`fail_push` teaching path (mirror `import_unreachable_detail` at `main.rs:362-374`).
Test-injection (bin-target `#[cfg(test)] mod tests` in `main.rs`, graded via the bare
`cargo test -p reposix-remote` — a `--test <name>` scope would MISS bin-target unit tests,
per crates/CLAUDE.md § "Bin-target vs integration-target test location"): factor the
git-invocation behind a small injectable seam (a closure/fn param, or point `PATH` at a
throwaway `/tmp` dir holding a fake `git` script that exits 128) so a NON-absence failure
is simulated and the assertion proves the function returns `Err` (loud) rather than
`Ok(None)` (silent). Run any such /tmp git-shim in a /tmp dir, `cd` in the same invocation
(leaf isolation).

### RPX code
Reuse the P121 registry (`reposix_core::codes`). 05xx (helper) is used through **RPX-0507**
(`HELPER_IMPORT_UNREACHABLE`); the next free helper code is **RPX-0508**. Mint
`ids::HELPER_IMPORT_PARENT_RESOLVE` = `"RPX-0508"` + one `ExplainEntry` in `REGISTRY`.
The `rpx_registry_check.py` reverse-completeness leg requires every registered code to be
EMITTED (`EMISSION_EXEMPT` is empty) — the new loud-fail path emits it, satisfying the gate.

## GTH-V15-06 (DRAIN-09) — binary-side self-safety refusal in `reposix init`

### What already exists
`refuse_existing_repo_root()` (`init.rs:135-168`, called at `run_with_since:273` BEFORE any
mutation) already refuses when the target **IS** a git working-tree root (`path/.git`
exists), carrying **RPX-0401** + a 3-part teaching (names the corruption, points at
`reposix attach`, prints the /tmp recovery). A git worktree root (gitfile `.git`) is also
caught (`.exists()` is true for a gitfile).

### The residual gap
The current guard deliberately ALLOWS "a fresh subdir nested INSIDE an existing working
tree" (docstring `init.rs:127-130`) because that is the sanctioned `/tmp` flow shape. It
does NOT distinguish "nested inside a /tmp throwaway clone" (safe) from "nested inside the
shared source tree" (corruption). The leaf-isolation Bash-tool hook already draws this
line via realpath: `is_safe()` (`.claude/hooks/leaf-isolation-guard.sh:170-184`) allows a
target that canonicalizes under `/tmp` (or `/private/tmp`) and refuses everything else,
after resolving cd-back (`cd /tmp/x && cd <shared>`), `/tmp/../<shared>` traversal, and a
`/tmp`-symlink-to-shared via `realpath -m`. The hook's own COVERAGE-BOUNDARY comment
(`leaf-isolation-guard.sh:46-56`) names the missing cut explicitly: *"THE REAL CUT is a
binary-side refusal in `reposix init` … scoped for v0.14.0 Wave 2, NOT built here."* —
that is DRAIN-09.

### Precise definition of "effective target nests inside shared source tree"
Mirror `is_safe()` in Rust, added to `reposix init` only (never `attach`):
1. Resolve the effective target's canonical path with `realpath -m` semantics
   (`std::fs::canonicalize` the deepest EXISTING ancestor — the leaf may not exist — then
   re-append the non-existent tail). This resolves symlinks on existing ancestors and
   collapses `..`, defeating the cd-back / traversal / symlink-to-shared smuggles.
2. If the canonical target is under `/tmp` or `/private/tmp` → **allow** (sanctioned
   throwaway zone — this is what keeps the /tmp dark-factory flow working).
3. Otherwise, walk UP from the target's parent; if any ancestor holds a `.git` (dir or
   gitfile), the target nests inside an existing git working tree → **refuse** with a
   Rust-compiler-grade teaching (new code **RPX-0406**): teach the fix (use a fresh
   non-nested path), suggest the alternative (`reposix attach` to adopt; `/tmp` for
   throwaway), and give a copy-paste recovery. This mirrors the guard's fail-closed,
   product-sensible line (nesting a partial-clone tree inside another repo is a mistake).
4. No `.git` ancestor found → allow (legit fresh end-user init anywhere on disk).

### The "worktree-shared config" self-safety check (belt-and-suspenders)
After `git init <path>` and BEFORE the `git config` writes (`init.rs:301-350`), assert the
resulting git-dir is the target's OWN freshly-created `.git` — i.e.
`git -C <path> rev-parse --absolute-git-dir` canonicalizes to `<path>/.git`, not a shared
`…/.git/worktrees/…` or the shared repo's object store. If it resolves elsewhere (a
worktree-shared config), abort loudly (RPX-0406) BEFORE the config writes can corrupt the
shared config. The pre-mutation nesting refusal (step 3) is the primary cut; this is the
independent second latch the charter asked to "pair with".

### RPX code
Next free init (04xx) code is **RPX-0406** (0401-0405 used). Mint
`ids::INIT_NESTED_IN_REPO` = `"RPX-0406"` + one `ExplainEntry`; emit it at both new refusal
sites. Update the user-facing index `docs/reference/error-codes.md` in the same phase
(docs discipline / fix-twice).

### Tests
New tests in `crates/reposix-cli/tests/errors_teach_recovery.rs` (mirror
`init_at_existing_repo_root_teaches_attach_and_recovery:31`), all using isolated tempdirs:
(a) init into a fresh subdir nested inside a NON-/tmp git working tree → refuses + teaches
+ RPX-0406 + names `reposix attach`; (b) init into a fresh subdir under a `/tmp` clone →
SUCCEEDS (dark-factory flow preserved); (c) a symlink/`..` path that resolves back into a
non-/tmp repo → refuses (canonicalization holds); (d) `reposix attach` against an existing
checkout still succeeds (no regression). If any test itself drives `reposix init`/git
setup via a shell step, it must `cd /tmp/...` in the same invocation (leaf isolation) —
but Rust `#[test]` tempdir setup driven by `cargo test` is safe (no shared-tree mutation).
