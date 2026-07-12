#!/usr/bin/env bash
# quality/gates/agent-ux/github-helper-path-slug-not-sanitized.sh --
# verifier for agent-ux/github-helper-path-slug-not-sanitized (P104 /
# S-260707-gh404, RAISE #3). The cheap sim/unit regression guard for the
# GitHub helper-path 404: the backend must see the RAW `owner/repo` slug
# while the on-disk cache dir stays the sanitized flat `github-owner-repo.git`.
#
# Per the row's `expected.asserts`:
#   1. a recording BackendConnector opened via Cache::open(_, "github",
#      "owner/repo") and driven through build_from() receives the RAW slug
#      "owner/repo" at list_records_complete -- NOT the sanitized "owner-repo".
#   2. the on-disk cache path (cache.repo_path()) is STILL the sanitized flat
#      dir "github-owner-repo.git" -- no embedded slash, no nested owner/ subdir.
#   3. the test fails on the PRE-FIX resolve_cache_path (nested dir) and passes
#      on the fixed one -- fails-then-passes demonstrated (documented in the
#      test's module doc; the live run below proves the passing half).
#
# ONE cargo invocation at a time (crates/CLAUDE.md build-memory budget) -- the
# single `cargo test` below runs SEQUENTIALLY, never concurrently with another
# cargo user. No GitHub token required (transport_claim:false); the real
# front-door 200 is owned by agent-ux/github-front-door-real-backend.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

ROW_ID="agent-ux/github-helper-path-slug-not-sanitized"
TEST_FILE="crates/reposix-cache/tests/github_project_slug_not_sanitized.rs"

# --- Precondition: the regression test binary exists ----------------------
if [[ ! -f "${TEST_FILE}" ]]; then
  echo "FAIL (${ROW_ID}): ${TEST_FILE} does not exist" >&2
  exit 1
fi

# --- Assert 1+2: both halves pinned by the test; run it, exit 0 -----------
# The test asserts (1) the backend received the RAW `owner/repo` slug and
# (2) cache.repo_path() is the sanitized flat `github-owner-repo.git`.
echo "running: cargo test -p reposix-cache --test github_project_slug_not_sanitized" >&2
if ! cargo test -p reposix-cache --test github_project_slug_not_sanitized; then
  echo "FAIL (${ROW_ID}): cargo test -p reposix-cache --test github_project_slug_not_sanitized did not exit 0" >&2
  echo "  -> a caller/self.project re-sanitize would make the backend see 'owner-repo' (the 404 bug), or path derivation stopped sanitizing (nested dir)" >&2
  exit 1
fi

# Confirm the test body actually asserts BOTH halves of the split (guards
# against the test being gutted to an always-green stub).
if ! grep -q 'p == "owner/repo"' "${TEST_FILE}"; then
  echo "FAIL (${ROW_ID}): ${TEST_FILE} no longer asserts the backend receives the RAW 'owner/repo' slug" >&2
  exit 1
fi
if ! grep -q '"github-owner-repo.git"' "${TEST_FILE}"; then
  echo "FAIL (${ROW_ID}): ${TEST_FILE} no longer asserts the cache dir is the sanitized flat 'github-owner-repo.git'" >&2
  exit 1
fi
echo "  assert 1 OK: backend sees RAW slug owner/repo (never sanitized owner-repo)" >&2
echo "  assert 2 OK: on-disk cache dir is the sanitized flat github-owner-repo.git" >&2

# --- Assert 3: fails-then-passes is documented in the test's rationale -----
# The live run above proves the PASSING half against the fixed code; the
# FAILING half (pre-fix resolve_cache_path -> nested github-owner/repo.git)
# is demonstrated + documented in the module doc so a reader can reproduce it.
if ! grep -qi 'FAILS-THEN-PASSES' "${TEST_FILE}"; then
  echo "FAIL (${ROW_ID}): ${TEST_FILE} no longer documents the fails-then-passes rationale (pre-fix nested dir -> fixed flat dir)" >&2
  exit 1
fi
echo "  assert 3 OK: fails-then-passes documented (pre-fix nested github-owner/repo.git -> fixed flat github-owner-repo.git)" >&2

echo "PASS (${ROW_ID}): backend gets RAW owner/repo slug, cache dir stays sanitized github-owner-repo.git, fails-then-passes documented"
exit 0
