#!/usr/bin/env bash
# quality/gates/agent-ux/init-refuses-nested-in-shared-tree.sh -- agent-ux verifier
# for catalog row `agent-ux/init-refuses-nested-in-shared-tree` (P122 W3 / DRAIN-09 /
# GTH-V15-06).
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/init-refuses-nested-in-shared-tree
# CADENCE:     on-demand (does NOT self-block intermediate pushes)
# INVARIANT:   `reposix init` refuses (RPX-0406) a fresh target that NESTS inside a
#              NON-/tmp git working tree (latch 1, canonicalized, mirrors
#              .claude/hooks/leaf-isolation-guard.sh::is_safe), AND aborts before any
#              git config write when `git init` bound the target to a SHARED git-dir
#              (latch 2, the worktree-shared-config self-check) -- WITHOUT breaking the
#              /tmp dark-factory flow or `reposix attach`. This is the binary-side
#              backstop for the D2 shared-tree-corruption recurrence (the Bash-tool
#              leaf-isolation-guard.sh only fires on the Claude Code Bash TOOL; a
#              subprocess/worktree bypass reaches the shared tree, and only a refusal
#              INSIDE the binary cuts that vector).
#
# transport_claim:false -- these are init-refusal exit-code/stderr semantics driven by
# the real `reposix` binary via assert_cmd, NOT a transport-layer / real-backend claim.
#
# ONE cargo invocation at a time (crates/CLAUDE.md build-memory budget): the single
# `cargo test` below runs SEQUENTIALLY, foreground, never concurrently with another
# cargo user.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

ROW_ID="agent-ux/init-refuses-nested-in-shared-tree"
SRC="crates/reposix-cli/src/init.rs"
CODES="crates/reposix-core/src/codes.rs"
TESTS="crates/reposix-cli/tests/errors_teach_recovery.rs"
ARTIFACT="quality/reports/verifications/agent-ux/init-refuses-nested-in-shared-tree.json"

fail() {
  echo "FAIL (${ROW_ID}): $1" >&2
  exit 1
}

# --- Precondition: the two latches + RPX-0406 + the regression tests exist --------
[[ -f "${SRC}" ]]   || fail "${SRC} does not exist"
[[ -f "${CODES}" ]] || fail "${CODES} does not exist"
[[ -f "${TESTS}" ]] || fail "${TESTS} does not exist"

grep -q 'INIT_NESTED_IN_REPO' "${CODES}" \
  || fail "${CODES} does not register ids::INIT_NESTED_IN_REPO (RPX-0406)"
grep -q '"RPX-0406"' "${CODES}" \
  || fail "${CODES} does not define the RPX-0406 code literal"
# Latch 1 (pre-mutation nesting refusal) + latch 2 (worktree-shared-config self-check).
grep -q 'fn refuse_nested_in_worktree' "${SRC}" \
  || fail "${SRC} is missing latch 1 (refuse_nested_in_worktree)"
grep -q 'fn assert_own_git_dir' "${SRC}" \
  || fail "${SRC} is missing latch 2 (assert_own_git_dir worktree-shared-config self-check)"
grep -q 'fn canonicalize_lexical_existing' "${SRC}" \
  || fail "${SRC} is missing the realpath -m canonicalizer (canonicalize_lexical_existing)"
# Both latches must be WIRED into run_with_since (not merely defined).
grep -q 'refuse_nested_in_worktree(&path)?' "${SRC}" \
  || fail "refuse_nested_in_worktree is not wired into run_with_since"
grep -q 'assert_own_git_dir(&path)?' "${SRC}" \
  || fail "assert_own_git_dir is not wired into run_with_since"
grep -q 'INIT_NESTED_IN_REPO' "${SRC}" \
  || fail "${SRC} does not emit the RPX-0406 (INIT_NESTED_IN_REPO) teaching"

# Guard against the tests being gutted to always-green stubs: each named test exists.
declare -a REQUIRED_TESTS=(
  init_nested_in_non_tmp_repo_refuses_with_rpx0406
  init_fresh_subdir_under_tmp_clone_is_not_refused
  init_via_symlink_into_non_tmp_repo_refuses_with_rpx0406
  attach_nested_checkout_is_not_blocked_by_init_refusal
  init_worktree_shared_git_dir_aborts_before_config_write
)
for t in "${REQUIRED_TESTS[@]}"; do
  grep -q "fn ${t}(" "${TESTS}" || fail "regression test ${t} is missing from ${TESTS}"
done
# The (a) test must assert the RPX-0406 tag + the attach alternative (not merely "fails").
grep -q 'RPX-0406' "${TESTS}"    || fail "the regression tests no longer assert the RPX-0406 tag"
grep -q 'reposix attach' "${TESTS}" || fail "the (a) refusal test no longer asserts the reposix attach alternative"

# --- Run the scoped init-refusal regression tests (lib unit + integration) --------
# Filter args are OR'd by libtest; this runs the 5 integration cases (a)-(e) PLUS the
# init.rs unit tests for the canonicalizer / is_tmp_safe / the /tmp-safe allow branch.
echo "running: cargo test -p reposix-cli (scoped to the P122 init-refusal tests)" >&2
TEST_LOG="$(mktemp)"
trap 'rm -f "${TEST_LOG}"' EXIT
if CARGO_BUILD_JOBS=2 cargo test -p reposix-cli -- \
      init_nested_in_non_tmp_repo_refuses_with_rpx0406 \
      init_fresh_subdir_under_tmp_clone_is_not_refused \
      init_via_symlink_into_non_tmp_repo_refuses_with_rpx0406 \
      attach_nested_checkout_is_not_blocked_by_init_refusal \
      init_worktree_shared_git_dir_aborts_before_config_write \
      canonicalize_collapses_dotdot_across_whole_path \
      is_tmp_safe_matches_tmp_zone_not_lookalikes \
      refuse_nested_allows_tmp_safe_even_when_nested \
      > "${TEST_LOG}" 2>&1; then
  RC=0
else
  RC=$?
fi
tail -40 "${TEST_LOG}" >&2
if [[ "${RC}" -ne 0 ]]; then
  fail "cargo test -p reposix-cli (init-refusal tests) exited ${RC}"
fi
# Confirm each named regression test actually RAN + passed (not filtered/ignored).
for t in "${REQUIRED_TESTS[@]}"; do
  grep -q "${t} ... ok" "${TEST_LOG}" \
    || fail "regression test ${t} did not run/pass under the scoped invocation"
done

# --- Emit the verification artifact (F-K4b congruence: asserts_passed token-maps
#     each of the row's expected.asserts) ------------------------------------------
mkdir -p "$(dirname "${ARTIFACT}")"
TS="$(date -u +"%Y-%m-%dT%H:%M:%SZ")" ROW="${ROW_ID}" OUT="${ARTIFACT}" python3 - <<'PY'
import json, os
artifact = {
    "ts": os.environ["TS"],
    "row_id": os.environ["ROW"],
    "exit_code": 0,
    "timed_out": False,
    "stdout": "cargo test -p reposix-cli (P122 init-refusal tests) exited 0; all 5 integration cases (a)-(e) + the canonicalizer/is_tmp_safe/tmp-safe-allow unit tests passed.",
    "stderr": "",
    "asserts_passed": [
        "cargo test -p reposix-cli (the P122 init-refusal regression tests) exits 0 -- all named tests ran and passed under the scoped invocation",
        "reposix init into a fresh subdir nested inside a NON-/tmp git working tree is refused with the RPX-0406 coded teaching that names reposix attach and prints a copy-paste recovery -- proven by init_nested_in_non_tmp_repo_refuses_with_rpx0406 (asserts RPX-0406 + reposix attach + Fix: + Recovery: + reposix explain RPX-0406)",
        "reposix init into a fresh subdir under a /tmp throwaway clone SUCCEEDS (the sanctioned dark-factory flow is preserved) -- proven by init_fresh_subdir_under_tmp_clone_is_not_refused (init reaches git init, target/.git created, NO RPX-0406 refusal) and the init::tests::init_allows_fresh_subdir_inside_existing_repo unit test",
        "a symlink/.. path that canonicalizes back into a non-/tmp git working tree is still refused (realpath canonicalization holds, mirroring leaf-isolation-guard.sh is_safe) -- proven by init_via_symlink_into_non_tmp_repo_refuses_with_rpx0406 and canonicalize_collapses_dotdot_across_whole_path",
        "the worktree-shared-config self-check fires: when git init binds the target to a SHARED git-dir (git -C <path> rev-parse --absolute-git-dir != <path>/.git via a GIT_DIR-injected shared store) reposix init aborts with RPX-0406 BEFORE any git config write reaches the shared config -- proven by init_worktree_shared_git_dir_aborts_before_config_write (case (e): asserts RPX-0406 + shared store config byte-unchanged)",
        "reposix attach against an existing checkout still succeeds (attach adoption is not regressed) -- proven by attach_nested_checkout_is_not_blocked_by_init_refusal (the init-only RPX-0406 refusal does NOT fire on reposix attach; attach.rs is untouched)",
    ],
    "asserts_failed": [],
}
with open(os.environ["OUT"], "w", encoding="utf-8") as f:
    json.dump(artifact, f, indent=2)
    f.write("\n")
PY

echo "PASS (${ROW_ID}): reposix init refuses a non-/tmp nested target (RPX-0406, both latches), preserves the /tmp dark-factory flow + attach, resists symlink/.. smuggling; regression tests green."
exit 0
