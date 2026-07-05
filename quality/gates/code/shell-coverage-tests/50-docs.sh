#!/usr/bin/env bash
# 50-docs.sh — drive the self-contained docs-alignment/, docs-build/, and
# docs-repro/ gates. All read the REAL worktree docs/ (BASH_SOURCE root or `cd
# $(git rev-parse --show-toplevel)`). container-rehearse.sh is deliberately NOT
# driven here (docker is present on some hosts → not hermetic).
#
# Harness contract: exit 0 regardless of target exits. See 00-probe.sh.
set -eu

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
DA="$REPO_ROOT/quality/gates/docs-alignment"
run() { "$@" >/dev/null 2>&1 || true; }

# docs-alignment presence/grep gates (install-snippet-shape covered by 00-probe).
run "$DA/audit-trail-git-log.sh"
run "$DA/connector-matrix-on-landing.sh"
run "$DA/dvcs-mirror-setup-walkthrough.sh"
run "$DA/dvcs-topology-three-roles.sh"
run "$DA/dvcs-troubleshooting-matrix.sh"
run "$DA/jira-adapter-shipped.sh"
run "$DA/three-backends-tested.sh"

# docs-build: mermaid-renders is a python/jq artifact check (no cargo).
run "$REPO_ROOT/quality/gates/docs-build/mermaid-renders.sh"

# docs-repro/manual-spec-check: stdlib markdown-spec checker. Happy path (known
# row) + unknown-row usage-error branch (exit 2).
run "$REPO_ROOT/quality/gates/docs-repro/manual-spec-check.sh" docs-repro/example-03-claude-code-skill
run "$REPO_ROOT/quality/gates/docs-repro/manual-spec-check.sh" docs-repro/bogus-row

exit 0
