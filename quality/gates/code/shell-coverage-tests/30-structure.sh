#!/usr/bin/env bash
# 30-structure.sh — drive the self-contained structure/ gates. Every target here
# reads the REAL worktree (via a BASH_SOURCE-relative root or `cd $(git
# rev-parse --show-toplevel)`), so the harness just invokes them; the worktree
# IS the fixture. A few take flags/stdin — those branches are driven explicitly.
#
# Harness contract: exit 0 regardless of target exits. See 00-probe.sh.
set -eu

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
G="$REPO_ROOT/quality/gates/structure"
run() { "$@" >/dev/null 2>&1 || true; }

# Pure grep/find/file gates against the real tree (happy path).
run "$G/banned-production-tokens.sh"
run "$G/deferral-pointer-linter.sh"
run "$G/required-doc-surfaces.sh"
run "$G/no-loose-top-level-planning-audits.sh"
run "$G/no-pre-pivot-doc-stubs.sh"
run "$G/release-plz-config.sh"
run "$G/release-tags-present.sh"
run "$G/repo-org-audit-artifact-present.sh"
run "$G/active-milestone-matches-workspace-version.sh"

# banned-words.sh is a thin exec of scripts/banned-words-lint.sh --all.
run "$G/banned-words.sh"

# file-size-limits: --warn-only (exit 0), default, and unknown-flag (exit 2).
run "$G/file-size-limits.sh" --warn-only
run "$G/file-size-limits.sh"
run "$G/file-size-limits.sh" --bogus

# cred-hygiene: empty stdin (exit 0 fast path) + a synthetic ref-range that
# exercises the scan loop against the sandbox repo below.
printf '' | run "$G/cred-hygiene.sh" || true
SANDBOX="$(mktemp -d)"; trap 'rm -rf "$SANDBOX"' EXIT
# cred-hygiene scans the range within its own REPO_ROOT (the real worktree);
# feed a real ref-range from the worktree so `git log <range>` resolves.
RANGE_HEAD="$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || echo)"
RANGE_PREV="$(git -C "$REPO_ROOT" rev-parse HEAD~1 2>/dev/null || echo "$RANGE_HEAD")"
if [ -n "$RANGE_HEAD" ]; then
  printf 'refs/heads/main %s refs/heads/main %s\n' "$RANGE_PREV" "$RANGE_HEAD" \
    | { "$G/cred-hygiene.sh" >/dev/null 2>&1 || true; }
fi

# python-unittest gates (no cargo; stdlib unittest against quality/runners).
run "$G/claim-vs-assertion-audit-required.sh"
run "$G/runner-honesty-semantics.sh"

exit 0
