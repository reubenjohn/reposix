#!/usr/bin/env bash
# 40-agent-ux-lint.sh — drive the self-contained agent-ux/ presence gates and
# the no-cargo lint-invariants/ gates. All read the REAL worktree (BASH_SOURCE
# root or `cd $(git rev-parse --show-toplevel)`); the python-unittest gates use
# stdlib unittest against quality/runners (no cargo).
#
# Harness contract: exit 0 regardless of target exits. See 00-probe.sh.
set -eu

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
AX="$REPO_ROOT/quality/gates/agent-ux"
LI="$REPO_ROOT/quality/gates/code/lint-invariants"
run() { "$@" >/dev/null 2>&1 || true; }

# agent-ux presence / content gates (grep/awk/sha256 vs the real tree).
run "$AX/absorption-honesty-template-present.sh"
run "$AX/p87-surprises-absorption.sh"
run "$AX/p88-good-to-haves-drained.sh"
run "$AX/v0.13.0-changelog-entry-present.sh"
run "$AX/v0.13.0-retrospective-distilled.sh"
run "$AX/v0.13.0-tag-script-present.sh"
run "$AX/test-name-vs-asserts.sh"
run "$AX/ql-001-canonical-path.sh"
run "$AX/milestone-adversarial-pass.sh"   # python3 -m unittest

# lint-invariants gates that are pure grep/file (no cargo).
run "$LI/demo-script-exists.sh"
run "$LI/git-version-requirement-documented.sh"
run "$LI/rust-msrv.sh"
run "$LI/rust-stable-channel.sh"
run "$LI/clippy-pedantic-targeted-allows.sh"
run "$LI/doctor-categories-covered.sh"

exit 0
