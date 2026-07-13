#!/usr/bin/env bash
# scripts/refresh-tokenworld-mirror.sh — refresh the GitHub TokenWorld mirror
# to the EXACT backend-current, materialized reposix tree.
#
# (no canonical home under quality/gates/ — owner/ops fixture-repair tool run
#  before the vision litmus / pre-release-real-backend cadence; not CI-wired.)
#
# WHY THIS EXISTS
# ---------------
# The vision litmus (quality/gates/agent-ux/milestone-close-vision-litmus.sh)
# clones the GitHub mirror `reubenjohn/reposix-tokenworld-mirror`, edits a page,
# and pushes through the reposix bus. If the mirror's stored tree is STALE vs
# the Confluence SoT (e.g. pages/2818063.md at version 1 while the backend is at
# version 7, or the protected pages/7798785.md at v2 vs backend v4), the push
# carries stale bases and the ONE documented one-shot recovery
# (`git pull --rebase reposix main`) CONFLICTS on the divergent *body* content —
# a hard RED that is NOT a real coherence bug, just a stale fixture.
#
# WHY NOT THE BUS FAN-OUT ALONE (executed + rejected — see report)
# ---------------------------------------------------------------
# The mirror-head refresh promise (ADR-010 RBF-LR-04) advances the mirror head
# on an SoT-changing bus push, but the inline fan-out pushes the *pre-write
# client tree*: the edited page's frontmatter `version:` therefore trails the
# backend by exactly the version that push just minted, AND the markdown↔storage
# roundtrip re-normalizes the body. So a bus-push refresh leaves the mirror
# perpetually one version behind + body-drifted on the edited page — the litmus
# one-shot rebase still conflicts. (Filed as a mirror-sync coherence finding;
# NOT fixed in product code here.)
#
# THE MECHANISM USED HERE (exact fixture repair, no backend mutation)
# -------------------------------------------------------------------
# `git fetch reposix main` pulls the backend's MATERIALIZED tree (get_record →
# ADF→Markdown, i.e. the same canonical form the litmus clone will read). We
# REPLACE that tree's pages/ WHOLESALE onto a fresh mirror clone (git rm then
# checkout FETCH_HEAD -- pages/, so backend-side DELETIONS propagate — an
# additive `checkout -- pages/` only adds/updates and would silently retain a
# page the backend removed) and commit it as a fast-forward CHILD of
# origin/main, then `git push origin main`. Result: the mirror holds the
# byte-exact backend-current materialized tree at the SAME versions — no backend
# write, no version bump, no force-push, no trailing drift. The litmus clone
# then matches the backend and its push lands cleanly.
#
# The post-push verification is NON-CIRCULAR: it compares the re-cloned mirror
# head against a PRISTINE extract of the backend materialization (`git archive
# FETCH_HEAD`, captured BEFORE the overlay), never against the working tree we
# just overlaid onto the mirror clone (which would be comparing the clone
# against itself).
#
# We do NOT use `reposix sync --reconcile` (rebuilds only the LOCAL cache, leaves
# the external mirror byte-identical — root CLAUDE.md § mirror-head refresh
# promise) and do NOT rely on the KNOWN-BROKEN webhook mirror-sync Action.
#
# SAFETY
# ------
#   * Runs entirely in a throwaway /tmp clone (leaf isolation — never the shared
#     repo). cwd stays inside /tmp for every mutating git/reposix call.
#   * NEVER writes the Confluence backend; NEVER edits any page. The protected
#     durable fixtures 7766017 / 7798785 ride along verbatim from the backend
#     tree. Only the GitHub mirror repo (a sanctioned target) is written.
#   * Fast-forward push only (child of origin/main); no --force, no git rm.
#   * Prints BEFORE/AFTER mirror versions and asserts the mirror head moved.
#
# USAGE
#   bash scripts/refresh-tokenworld-mirror.sh
#
# Requires: ATLASSIAN_API_KEY / ATLASSIAN_EMAIL / REPOSIX_CONFLUENCE_TENANT
# (auto-sourced from ./.env if present), SSH access to the GitHub mirror, and
# the reposix + git-remote-reposix binaries (built into target/debug if absent).
#
# `set -e` is load-bearing: WITHOUT it an early failure (attach, fetch, archive,
# push) does NOT abort the script, which then falls through to the final
# "OK: mirror now byte-current …" line and exits 0 — a FALSE byte-identical
# claim the 9th-probe verifier would trust. Every mutating step already guards
# with `|| { echo ERROR…; exit 1; }`; `set -e` closes the gaps between them.
set -euo pipefail

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." &> /dev/null && pwd)"
SPACE="${REPOSIX_CONFLUENCE_SPACE_OVERRIDE:-REPOSIX}"      # == TokenWorld == space 360450
PROTECTED_IDS=" 7766017 7798785 "
MIRROR_URL="${REPOSIX_LITMUS_MIRROR:-git@github.com:reubenjohn/reposix-tokenworld-mirror.git}"

# --- Source .env if creds not already present (mirror preflight pattern) ------
if [ -f "${REPO_ROOT}/.env" ] && [ -z "${ATLASSIAN_API_KEY:-}" ]; then
  set -a; . "${REPO_ROOT}/.env"; set +a
fi
for v in ATLASSIAN_API_KEY ATLASSIAN_EMAIL REPOSIX_CONFLUENCE_TENANT; do
  if [ -z "${!v:-}" ]; then
    echo "ERROR: $v unset — populate .env (docs/reference/testing-targets.md)" >&2
    exit 2
  fi
done
TENANT="${REPOSIX_CONFLUENCE_TENANT}"
export REPOSIX_ALLOWED_ORIGINS="${REPOSIX_ALLOWED_ORIGINS:-http://127.0.0.1:*,https://${TENANT}.atlassian.net}"

# --- Build the binaries if missing (one cargo invocation) ---------------------
if [ ! -x "${REPO_ROOT}/target/debug/reposix" ] || [ ! -x "${REPO_ROOT}/target/debug/git-remote-reposix" ]; then
  ( cd "${REPO_ROOT}" && cargo build -p reposix-cli -p reposix-remote \
      --bin reposix --bin git-remote-reposix ) >&2 || { echo "ERROR: cargo build failed" >&2; exit 1; }
fi
export PATH="${REPO_ROOT}/target/debug:${PATH}"   # git-remote-reposix on PATH
BIN="${REPO_ROOT}/target/debug/reposix"

version_of() { grep -m1 '^version:' "$1" 2>/dev/null | tr -d '[:space:]' || true; }
# pipefail-safe under `set -e`: an empty pages/ glob `continue`s (status 0)
# rather than tripping a `[ -e ] &&` short-circuit non-zero exit that would
# abort the whole script inside a `VAR=$(all_versions …)` command substitution.
all_versions() {
  { for f in "$1"/pages/*.md; do
      [ -e "$f" ] || continue
      printf '%s=%s\n' "$(basename "$f" .md)" "$(version_of "$f")"
    done; } | sort | tr '\n' ' '
}

RUN="$(mktemp -d -t mirror-refresh.XXXXXX)"
export REPOSIX_CACHE_DIR="${RUN}/cache"
TREE="${RUN}/tree"

echo "== refresh-tokenworld-mirror =="
echo "space=${SPACE} tenant=${TENANT} run=${RUN}"

# --- BEFORE: clone the mirror, record stored versions -------------------------
git clone --quiet "${MIRROR_URL}" "${TREE}" || { echo "ERROR: mirror clone failed" >&2; exit 1; }
echo "BEFORE mirror versions: $(all_versions "${TREE}")"

# --- Overlay the BACKEND-MATERIALIZED pages/ onto the mirror clone ------------
"${BIN}" attach "confluence::${SPACE}" "${TREE}" --remote-name reposix --mirror-name origin >&2 \
  || { echo "ERROR: reposix attach failed" >&2; exit 1; }
cd "${TREE}" || exit 1
git fetch --quiet reposix main || { echo "ERROR: git fetch reposix main failed" >&2; exit 1; }

# Capture a PRISTINE extract of the backend materialization at FETCH_HEAD BEFORE
# any overlay — this is the non-circular reference the post-push verification
# compares against (never the working tree we overlay onto the mirror clone).
BACKEND_REF="${RUN}/backend-ref"
mkdir -p "${BACKEND_REF}"
git archive FETCH_HEAD pages/ | tar -x -f - -C "${BACKEND_REF}" \
  || { echo "ERROR: capturing backend reference tree (git archive FETCH_HEAD) failed" >&2; exit 1; }
echo "BACKEND materialized versions: $(all_versions "${BACKEND_REF}")"

# Overlay backend-current pages/ WITHOUT moving HEAD (stays on origin/main), so
# the commit below fast-forwards the mirror rather than needing --force. Replace
# the tree WHOLESALE (git rm then checkout) so backend-side DELETIONS propagate:
# an additive `git checkout FETCH_HEAD -- pages/` only adds/updates and would
# silently retain a page the backend removed (stale-mirror bug).
git rm -r --quiet --ignore-unmatch -- pages/ > /dev/null 2>&1 || true
git checkout FETCH_HEAD -- pages/ || { echo "ERROR: checkout backend pages/ failed" >&2; exit 1; }

# Protected-fixture guard: they must be present and are only ever carried
# verbatim from the backend tree — never edited by us.
for pid in 7766017 7798785; do
  [ -e "pages/${pid}.md" ] || { echo "ERROR: backend tree missing protected fixture ${pid}" >&2; exit 1; }
done

git config user.email "mirror-refresh@reposix.invalid"
git config user.name "reposix-mirror-refresh"
git add -A pages/
if git diff --cached --quiet; then
  echo "OK: mirror already byte-identical to backend-materialized tree — nothing to push"
  echo "AFTER  mirror versions: $(all_versions "${BACKEND_REF}")"
  exit 0
fi
git commit --quiet -m "mirror-refresh: sync GitHub mirror to backend-current materialized tree (v0.14.0 item 5)"
echo "\$ git push origin main   (fast-forward child; no backend write, no --force)"
git push origin main >&2 || { echo "ERROR: fast-forward push to mirror failed" >&2; exit 1; }

# --- AFTER: fresh re-clone of the mirror, verify it now matches the backend ---
# NON-CIRCULAR: AFTER is read from an independent re-clone of the pushed mirror;
# BACKEND is the pristine FETCH_HEAD extract captured above — NOT the working
# tree we overlaid (which would compare the clone against itself).
TREE2="${RUN}/verify"
git clone --quiet "${MIRROR_URL}" "${TREE2}" || { echo "ERROR: verify re-clone failed" >&2; exit 1; }
AFTER="$(all_versions "${TREE2}")"
BACKEND="$(all_versions "${BACKEND_REF}")"
echo "AFTER  mirror versions: ${AFTER}"
if [ "${AFTER}" != "${BACKEND}" ]; then
  echo "ERROR: mirror versions after push (${AFTER}) != backend materialized (${BACKEND})" >&2
  exit 1
fi
echo "OK: mirror now byte-current with the backend materialized tree (versions match exactly)"
