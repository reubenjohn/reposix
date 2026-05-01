#!/usr/bin/env bash
# quality/gates/agent-ux/webhook-first-run-empty-mirror.sh
# Two-sub-fixture harness for the workflow's first-run handling
# (Q4.3 / DVCS-WEBHOOK-03 / D-07).
#
# CATALOG ROW: agent-ux/webhook-first-run-empty-mirror
# CADENCE: pre-pr (~2s wall time)
# INVARIANT:
#   (4.3.a) fresh-but-readme mirror — `git init --bare` + a working
#       tree pushes one README commit. Workflow tree fetches; mirror/main
#       IS present. The lease-push branch (`if git show-ref --verify
#       --quiet refs/remotes/mirror/main`) fires; lease push succeeds;
#       mirror's main advances from README SHA to workflow's SHA.
#   (4.3.b) truly-empty mirror — `git init --bare` only; no main
#       ref. Workflow tree fetches; the fetch's exit isn't propagated
#       (the YAML uses `2>/dev/null || echo`) but no ref is created.
#       `git show-ref --verify --quiet` returns 1; the plain-push
#       branch fires; mirror's main is created at workflow's SHA.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

# Helper: emulate the workflow's push step against a given mirror.
# Mirrors the YAML's `if git show-ref ... ; then lease-push; else
# plain-push; fi` exactly.
run_workflow_push_step() {
  local wt="$1"
  cd "$wt"
  if git show-ref --verify --quiet refs/remotes/mirror/main; then
    LEASE_SHA=$(git rev-parse refs/remotes/mirror/main)
    git push mirror "refs/heads/main:refs/heads/main" \
      --force-with-lease="refs/heads/main:${LEASE_SHA}" \
      || return 1
    echo "  -> lease-push branch fired (mirror/main was at ${LEASE_SHA:0:7})"
  else
    git push mirror "refs/heads/main:refs/heads/main" \
      || return 1
    echo "  -> plain-push branch fired (mirror/main absent locally)"
  fi
}

# ---------- Sub-case 4.3.a: fresh-but-readme ----------
echo "Sub-case 4.3.a: fresh-but-readme mirror"
git init --bare "$TMPDIR/mirror-a.git" >/dev/null
git -C "$TMPDIR/mirror-a.git" symbolic-ref HEAD refs/heads/main

# Seed mirror with one README commit (emulates `gh repo create
# --add-readme`).
git init "$TMPDIR/seed-a" >/dev/null 2>&1
git -C "$TMPDIR/seed-a" -c init.defaultBranch=main checkout -B main >/dev/null 2>&1
echo "# initial readme" > "$TMPDIR/seed-a/README.md"
git -C "$TMPDIR/seed-a" add README.md
git -C "$TMPDIR/seed-a" -c user.name=test -c user.email=test@test commit -m "initial readme" >/dev/null
git -C "$TMPDIR/seed-a" remote add mirror "$TMPDIR/mirror-a.git"
git -C "$TMPDIR/seed-a" push mirror main >/dev/null 2>&1
README_SHA=$(git -C "$TMPDIR/mirror-a.git" rev-parse refs/heads/main)

# Workflow's working tree (emulates /tmp/sot post-`reposix init`).
git init "$TMPDIR/wt-a" >/dev/null 2>&1
git -C "$TMPDIR/wt-a" -c init.defaultBranch=main checkout -B main >/dev/null 2>&1
echo "# SoT content" > "$TMPDIR/wt-a/page-1.md"
git -C "$TMPDIR/wt-a" add page-1.md
git -C "$TMPDIR/wt-a" -c user.name=test -c user.email=test@test commit -m "SoT content" >/dev/null
git -C "$TMPDIR/wt-a" remote add mirror "$TMPDIR/mirror-a.git"
git -C "$TMPDIR/wt-a" fetch mirror main >/dev/null 2>&1

# Run the push step.
run_workflow_push_step "$TMPDIR/wt-a" \
  || { echo "FAIL: 4.3.a lease-push branch failed"; exit 1; }

# Assert mirror's main advanced from README_SHA to workflow's SHA.
WORKFLOW_SHA=$(git -C "$TMPDIR/wt-a" rev-parse HEAD)
MIRROR_SHA=$(git -C "$TMPDIR/mirror-a.git" rev-parse refs/heads/main)
[ "$MIRROR_SHA" = "$WORKFLOW_SHA" ] \
  || { echo "FAIL: 4.3.a mirror/main=${MIRROR_SHA:0:7}, expected ${WORKFLOW_SHA:0:7}"; exit 1; }
[ "$MIRROR_SHA" != "$README_SHA" ] \
  || { echo "FAIL: 4.3.a mirror/main unchanged from README SHA"; exit 1; }
echo "  PASS: 4.3.a mirror/main advanced from ${README_SHA:0:7} to ${MIRROR_SHA:0:7}"

# ---------- Sub-case 4.3.b: truly-empty ----------
echo ""
echo "Sub-case 4.3.b: truly-empty mirror"
git init --bare "$TMPDIR/mirror-b.git" >/dev/null
git -C "$TMPDIR/mirror-b.git" symbolic-ref HEAD refs/heads/main

# NO seed push — mirror has no main ref.

git init "$TMPDIR/wt-b" >/dev/null 2>&1
git -C "$TMPDIR/wt-b" -c init.defaultBranch=main checkout -B main >/dev/null 2>&1
echo "# SoT content" > "$TMPDIR/wt-b/page-1.md"
git -C "$TMPDIR/wt-b" add page-1.md
git -C "$TMPDIR/wt-b" -c user.name=test -c user.email=test@test commit -m "SoT content" >/dev/null
git -C "$TMPDIR/wt-b" remote add mirror "$TMPDIR/mirror-b.git"
# Fetch fails on truly-empty (no main); the YAML's `|| echo` swallows it.
git -C "$TMPDIR/wt-b" fetch mirror main 2>/dev/null \
  || echo "  (first-run: mirror has no main yet)"

# Run the push step.
run_workflow_push_step "$TMPDIR/wt-b" \
  || { echo "FAIL: 4.3.b plain-push branch failed"; exit 1; }

# Assert mirror's main was created at workflow's SHA.
WORKFLOW_SHA=$(git -C "$TMPDIR/wt-b" rev-parse HEAD)
MIRROR_SHA=$(git -C "$TMPDIR/mirror-b.git" rev-parse refs/heads/main 2>/dev/null) \
  || { echo "FAIL: 4.3.b mirror/main not created"; exit 1; }
[ "$MIRROR_SHA" = "$WORKFLOW_SHA" ] \
  || { echo "FAIL: 4.3.b mirror/main=${MIRROR_SHA:0:7}, expected ${WORKFLOW_SHA:0:7}"; exit 1; }
echo "  PASS: 4.3.b mirror/main created at ${MIRROR_SHA:0:7}"

echo ""
echo "PASS: both Q4.3 sub-cases (4.3.a fresh-but-readme + 4.3.b truly-empty) handled correctly"
exit 0
