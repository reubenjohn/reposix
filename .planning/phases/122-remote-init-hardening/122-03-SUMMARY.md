---
phase: 122-remote-init-hardening
plan: 03
wave: 3
subsystem: reposix-cli / init hardening (security)
tags: [security, agent-ux, init, RPX-0406, DRAIN-09, GTH-V15-06, D2, leaf-isolation]
requires:
  - "122-01 catalog-first GREEN-CONTRACT row (agent-ux/init-refuses-nested-in-shared-tree)"
provides:
  - "reposix init binary-side refusal (RPX-0406) — two latches: nested-in-worktree + worktree-shared-config self-check"
  - "canonicalize_lexical_existing (realpath -m parity with leaf-isolation-guard.sh::is_safe)"
  - "quality/gates/agent-ux/init-refuses-nested-in-shared-tree.sh (mechanical verifier, GREEN)"
  - "teach_scan.py teach_coded( recognition (P122-W2-01 resolved)"
affects:
  - "crates/reposix-cli/src/init.rs"
  - "crates/reposix-core/src/codes.rs"
  - ".claude/hooks/leaf-isolation-guard.sh (header only)"
tech-stack:
  patterns: ["realpath -m canonicalization in Rust (deepest-existing-ancestor + lexical .. collapse)", "teach_coded coded 3-part teaching errors", "belt-and-suspenders pre-mutation refusal + post-git-init self-check"]
key-files:
  created:
    - "quality/gates/agent-ux/init-refuses-nested-in-shared-tree.sh"
  modified:
    - "crates/reposix-cli/src/init.rs"
    - "crates/reposix-core/src/codes.rs"
    - "crates/reposix-cli/tests/errors_teach_recovery.rs"
    - "quality/gates/agent-ux/teach_scan.py"
    - ".claude/hooks/leaf-isolation-guard.sh"
    - "CLAUDE.md"
    - "crates/CLAUDE.md"
    - "docs/reference/error-codes.md"
    - ".planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md"
decisions:
  - "RPX-0406 emitted via teach_coded (not hand-rolled) — dogfoods the Task 0 teach_scan fix; no teach-exempt marker needed."
  - "Non-/tmp test trees built under CARGO_TARGET_TMPDIR (gitignored, a separate repo), /tmp cases under /tmp explicitly — leaf-isolated, TMPDIR-independent."
  - "Task 0 scope widened beyond the W2 sketch: teach_scan --scope cli had 26 RED blocks (not 5); the residual init.rs:155 RPX-0401 marker-window regression was hugged too."
metrics:
  tasks: 4  # Task 0 (folded eager-fix) + Tasks 1-3
  duration: "~1 session"
  completed: 2026-07-18
---

# Phase 122 Plan 03 (Wave 3): reposix init nested-in-worktree refusal (RPX-0406) Summary

Binary-side self-safety refusal in `reposix init` (GTH-V15-06 / DRAIN-09) — the
defense-in-depth backstop for the D2 shared-tree-corruption recurrence whose Bash-tool
side was closed in P102. Two latches now fire inside the binary, so a subprocess/worktree
bypass of the PreToolUse `leaf-isolation-guard.sh` is caught at the layer that can actually
stop it: (1) a pre-mutation refusal when the canonical target nests inside a non-/tmp git
working tree, and (2) a post-`git init` worktree-shared-config self-check that aborts before
any `git config` write when the git-dir is not the target's own `.git`. Both emit a
Rust-compiler-grade **RPX-0406** teaching; both are init-only (never `attach`); neither
breaks the /tmp dark-factory flow.

## What shipped

### Task 0 (folded eager-fix — resolves BLOCKER P122-W2-01)
`quality/gates/agent-ux/teach_scan.py` did not recognize the P121 `teach_coded(` idiom, so
it read every coded teaching site as teaching-LESS. Extended `_PASS_CALL` to
`\bteach(?:_coded)?\s*\(` (mirrors `rpx_registry_check.py`). **Scope was larger than the W2
sketch:** `--scope cli` had **26** RED blocks at HEAD, not the 5 helper ones the W2 agent
counted. 25 were `teach_coded(` sites (cleared by the regex); the 1 residual — `init.rs`'s
hand-rolled RPX-0401 `bail!` — was a SEPARATE P121 comment-expansion regression that pushed
its `// teach-exempt: ok` marker 8 lines above the `bail!`, outside teach_scan's 2-line
lookback window, so I hugged the marker to the `bail!`. Added a `teach_coded(...)` self-test
fixture. Result: `--scope cli` (14 files), `--scope helper` (7 files), `--self-test` all
CLEAN. P122-W2-01 marked RESOLVED (`92614edc`).

### Task 1 — RPX-0406 + latch 1 (nested-in-worktree refusal)
- `codes.rs`: `ids::INIT_NESTED_IN_REPO = "RPX-0406"` + `ExplainEntry` (cause/fix/alt/recovery,
  all `&'static`) + added to the id-resolution test list.
- `init.rs`: `canonicalize_lexical_existing` (realpath -m semantics — canonicalize the
  deepest existing ancestor, then collapse `.`/`..` lexically across the WHOLE re-appended
  tail so `newdir/../../etc` cannot climb out), `is_tmp_safe` (component-wise `/tmp` |
  `/private/tmp`), and `refuse_nested_in_worktree` (/tmp short-circuit, else walk ancestors
  for `.git` → RPX-0406 via `teach_coded`). Wired into `run_with_since` after
  `refuse_existing_repo_root`, pre-mutation.

### Task 2 — latch 2 (worktree-shared-config self-check) + tests + gate
- `assert_own_git_dir`: after `git init`, `git -C <path> rev-parse --absolute-git-dir` must
  canonicalize to `<path>/.git`; on mismatch (injected `GIT_DIR` / worktree gitfile) bail
  RPX-0406 BEFORE any config write. Wired between `git init` and the config writes.
- Tests (a)-(e) in `errors_teach_recovery.rs` (assert_cmd, real binary): (a) nested non-/tmp
  → refused RPX-0406; (b) /tmp subdir → NOT refused (reaches `git init`, target `.git`
  created); (c) symlink-into-non-/tmp → refused (canonicalization holds); (d) `attach`
  nested checkout → not blocked by the init-only refusal; (e) `GIT_DIR`-injected shared
  store → RPX-0406 abort with the shared `config` byte-unchanged.
- Unit tests in `init.rs`: dotdot-collapse-across-whole-path, is_tmp_safe zone match,
  refuse_nested /tmp-safe allow; hardened `init_allows_fresh_subdir_*` to build under /tmp.
- Gate `quality/gates/agent-ux/init-refuses-nested-in-shared-tree.sh` (mechanical, scoped
  cargo test + congruent `asserts_passed` artifact) → **PASS**.

### Task 3 — fix-twice (docs/CLAUDE.md)
- `leaf-isolation-guard.sh` header: the "REAL CUT ... NOT built here" COVERAGE-BOUNDARY note
  now says the binary-side refusal SHIPS (RPX-0406); both layers agree on the /tmp safe zone.
- root `CLAUDE.md` + `crates/CLAUDE.md`: RPX-0406 binary backstop documented.
- `docs/reference/error-codes.md`: RPX-0406 row + RPX-04xx family range extended; also fixed
  the stale RPX-05xx family range (`0507`→`0508`, a noticing eager-fix — 0508 landed in W2).

## Verify-against-reality evidence

- **Real RPX-0406 stderr** (built binary, nested non-/tmp target): refusal names the target
  and enclosing tree, `[RPX-0406]` tag, `Fix:`, `Alternative:` (names `reposix attach` + /tmp),
  `Recovery:` (2 copy-paste lines), `Explain: reposix explain RPX-0406` — meets the OD-3 #5
  bar, matching the `refuse_existing_repo_root` exemplar.
- **/tmp dark-factory flow STILL succeeds** (live sim, real binary, /tmp leaf): exit 0 —
  `reposix init: configured .../repo with remote.origin.url = reposix::.../projects/demo`.
- `cargo test -p reposix-cli`: all suites green (lib 81 passed; errors_teach_recovery 19
  passed; 0 failures across every suite).
- `cargo clippy -p reposix-core -p reposix-cli --all-targets`: 0 warnings (fixed
  match_same_arms + doc_markdown in the new code).
- Gates GREEN: `init-refuses-nested-in-shared-tree.sh`, `rpx-codes-registry.sh`,
  `structure/banned-words.sh`, `docs-build/mkdocs-strict.sh`; `teach_scan --scope {cli,helper}`
  + `--self-test` CLEAN.

## How the canonicalization mirrors leaf-isolation-guard.sh::is_safe

The hook does `realpath -m -- "$tgt"` then allows only `/tmp|/tmp/*|/private/tmp|/private/tmp/*`.
`canonicalize_lexical_existing` reproduces `realpath -m`: it `std::fs::canonicalize`s the
DEEPEST EXISTING ancestor (resolving all symlinks in the existing prefix — defeating a
`/tmp/link -> /elsewhere` smuggle), then applies the non-existent tail lexically (`.` skip,
`..` pop) onto that real ancestor, so `..` collapses across the WHOLE path (defeating the
`newdir/../../etc` escape). `is_tmp_safe` uses the SAME `/tmp` | `/private/tmp` set via
component-wise `starts_with` (so `/tmpfoo` and `/var/tmp` do NOT read as safe). Both layers
therefore agree on which targets are safe — a divergence would let one pass what the other
refuses.

## Deviations from Plan

### Auto-fixed / folded (Task 0 + noticing eager-fixes)

**1. [Rule 1 - Bug] teach_scan cli residual: init.rs:155 marker-window regression**
- **Found during:** Task 0 verification (`--scope cli` showed 26 blocks, not the 5 the W2
  sketch predicted). 25 cleared by the `teach_coded` regex; init.rs:155 (hand-rolled RPX-0401
  `bail!`) did not — its `// teach-exempt: ok` marker sat 8 lines above the `bail!`, outside
  the 2-line window (a P121 comment-expansion regression).
- **Fix:** hugged the marker to the `bail!` (comment restructured, no user-facing change).
- **Files:** `crates/reposix-cli/src/init.rs`. **Commit:** `92614edc`.

**2. [Rule 1 - Bug] Hardened init_allows_fresh_subdir_inside_existing_repo to /tmp**
- **Found during:** Task 2 — latch 1 makes that test's "allow nested subdir" behavior
  TMPDIR-dependent (only allowed when /tmp-safe). It used `tempfile::tempdir()`.
- **Fix:** build under `/tmp` explicitly + assert no RPX-0406. **Commit:** `9cdbc573`.

**3. [Noticing eager-fix] Stale RPX-05xx family range in docs**
- **Found during:** Task 3 editing error-codes.md — the family table said `0501`–`0507` but
  RPX-0508 landed in W2. **Fix:** `0507`→`0508`. **Commit:** `326e07c9`.

Two new clippy pedantic warnings in the new code (match_same_arms, doc_markdown) were fixed
before the Task 2 commit (part of `9cdbc573`).

## Noticing (OD-3 #2)

- **`teach_scan.py` is `on-demand` cadence, so its RED rotted undetected.** BOTH cli (26) and
  helper (5) scopes were RED at HEAD since P121; nothing in pre-push runs it. Consider
  promoting `teach_scan --scope {cli,helper}` to a pre-push gate (it is fast, pure-Python, no
  cargo) so a `teach_coded`-shaped or marker-window regression can't accumulate silently
  again. Filed candidate for a GOOD-TO-HAVE; not done here (adds a pre-push gate = scope).
- **The binary check and the hook now DUPLICATE the /tmp safe-zone logic in two languages**
  (Rust `is_tmp_safe` + bash `is_safe`). They agree today (tested), but a future edit to one
  could drift from the other. No shared source of truth exists; a cross-language congruence
  test (feed the same paths to both, assert equal verdicts) would lock it — filed candidate.
- **`init.rs` is now 56k chars (2.8× the 20k *.rs ceiling); `codes.rs` 55k.** Both are WAIVED
  (file-size-limits until 2026-08-08) and `codes.rs` is a deliberate single-source-of-truth
  (GTH-V15-68). `init.rs` has no such charter — its `--since` rewind + the two new latches
  make it a real split candidate (e.g. `init/refuse.rs`). Not urgent; noted for the drain.
- **assert_own_git_dir vs refuse_nested overlap on RPX-0406.** Both latches emit the same
  code with different headlines. Correct (same failure class, distinct trigger) and both are
  exercised (cases a/c vs e), but a reader may wonder why one code covers two checks — the
  ExplainEntry cause names both. Left as-is.
- **`git init <path>` with `GIT_DIR` set reinitializes the shared store** (observed in case
  e); it did NOT change the store's config bytes (git init on a complete repo is a config
  no-op), so the byte-identical assertion holds. If a future git changed that, the test's
  belt-and-suspenders "no reposix key leaked" assertion still guards the real property.

## Self-Check: PASSED

All created/modified key files exist on disk; all five task commits (`92614edc`,
`4a0aaffe`, `a71f56cf`, `9cdbc573`, `326e07c9`) are present in git history; RPX-0406 is
both registered (`codes.rs`) and emitted (`init.rs`).
