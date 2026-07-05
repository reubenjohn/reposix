#!/usr/bin/env bash
# 20-scripts.sh — drive scripts/*.sh utilities (banned-words-lint, install-hooks)
# across their branches. banned-words-lint reads the real worktree docs/ via a
# BASH_SOURCE-relative root; install-hooks is exercised against BOTH a sandbox
# repo (happy path, via REPOSIX_INSTALL_HOOKS_ROOT) and the real worktree (whose
# .git is a file, so the guard branch fires).
#
# Harness contract: exit 0 regardless of target exits. See 00-probe.sh.
set -eu

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
SANDBOX="$(mktemp -d)"
export TMPDIR="$SANDBOX"
trap 'rm -rf "$SANDBOX"' EXIT

run() { "$@" >/dev/null 2>&1 || true; }

# banned-words-lint: help / default / --all / bad-flag (usage-error branch).
run "$REPO_ROOT/scripts/banned-words-lint.sh" --help
run "$REPO_ROOT/scripts/banned-words-lint.sh"
run "$REPO_ROOT/scripts/banned-words-lint.sh" --all
run "$REPO_ROOT/scripts/banned-words-lint.sh" --bogus-flag

# install-hooks happy path against a sandbox repo with a REAL .git dir + a
# .githooks dir carrying event hooks + a dotted helper (skip branch) + a
# non-file entry (continue branch).
HREPO="$SANDBOX/hooks-repo"
mkdir -p "$HREPO/.githooks/subdir"
git -C "$HREPO" init -q
printf '#!/bin/sh\n' > "$HREPO/.githooks/pre-commit"
printf '#!/bin/sh\n' > "$HREPO/.githooks/pre-push"
printf 'x\n'        > "$HREPO/.githooks/helper.sh"   # dotted -> skipped
REPOSIX_INSTALL_HOOKS_ROOT="$HREPO" run "$REPO_ROOT/scripts/install-hooks.sh"

# install-hooks "nothing to enable" branch: repo whose .githooks has only
# dotted/non-event entries.
EREPO="$SANDBOX/empty-hooks-repo"
mkdir -p "$EREPO/.githooks"
git -C "$EREPO" init -q
printf 'x\n' > "$EREPO/.githooks/only.sh"
REPOSIX_INSTALL_HOOKS_ROOT="$EREPO" run "$REPO_ROOT/scripts/install-hooks.sh"

# install-hooks guard branch: real worktree (.git is a file in a worktree) OR a
# dir with no .githooks.
NOGIT="$SANDBOX/nogit"
mkdir -p "$NOGIT"
REPOSIX_INSTALL_HOOKS_ROOT="$NOGIT" run "$REPO_ROOT/scripts/install-hooks.sh"
GITNOHOOKS="$SANDBOX/githooks-missing"
mkdir -p "$GITNOHOOKS/.git"
REPOSIX_INSTALL_HOOKS_ROOT="$GITNOHOOKS" run "$REPO_ROOT/scripts/install-hooks.sh"

exit 0
