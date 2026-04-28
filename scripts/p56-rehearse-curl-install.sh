#!/usr/bin/env bash
# KEEP-AS-CANONICAL (P63 SIMPLIFY-12): canonical for own domain (P56 install-evidence rehearsal); CLAUDE.md P56 section names this exact path.
# P56 RELEASE-01 — container rehearsal for the curl install path.
#
# Runs the documented one-liner inside a fresh ubuntu:24.04 container and
# captures the full transcript to a log file. Used by:
#
#   - Wave 3 of P56 (this script's first call) to grade
#     install/curl-installer-sh PASS/FAIL.
#   - Wave 4's verifier subagent — re-runs this with zero session context to
#     confirm the executing agent's claim.
#
# Why a script vs ad-hoc bash: the verifier subagent (per the v0.12.0
# autonomous-execution-protocol § "the executing agent's word is not the
# verdict") needs ONE named command it can re-run. Ad-hoc heredocs in agent
# context don't survive into the next session.
#
# Inputs (env vars):
#   REPOSIX_INSTALLER_URL   override the default `releases/latest/download/...`
#                           URL — set to a pinned-tag URL if the release-plz
#                           latest-pointer caveat is biting (see
#                           release-fire-evidence.md § "Latest pointer caveat").
#   REPOSIX_LOG_PATH        where to write the container transcript
#                           (default: /tmp/p56-curl-rehearsal.log).
#
# Outputs:
#   - transcript at $REPOSIX_LOG_PATH
#   - exit 0 on PASS, non-zero on FAIL
#
# Usage:
#   bash scripts/p56-rehearse-curl-install.sh
#   REPOSIX_INSTALLER_URL=https://github.com/reubenjohn/reposix/releases/download/reposix-cli-v0.11.3/reposix-installer.sh \
#       bash scripts/p56-rehearse-curl-install.sh

set -euo pipefail

URL="${REPOSIX_INSTALLER_URL:-https://github.com/reubenjohn/reposix/releases/latest/download/reposix-installer.sh}"
LOG="${REPOSIX_LOG_PATH:-/tmp/p56-curl-rehearsal.log}"

# Pass URL into the container; the installer URL must NOT shell-expand here
# (the heredoc inside docker run is single-quoted to avoid double expansion).
docker run --rm -i \
    -e REPOSIX_INSTALLER_URL="$URL" \
    ubuntu:24.04 bash -s <<'CONTAINER_EOF' 2>&1 | tee "$LOG"
set -euo pipefail
URL="$REPOSIX_INSTALLER_URL"
apt-get update -qq && apt-get install -y -qq curl ca-certificates >/dev/null
echo "=== HEAD ${URL} ==="
# `head -20` closes its stdin after 20 lines, which gives curl SIGPIPE and
# pipefail makes the script exit. Capture into a tempfile then trim.
TMP_HEAD=$(mktemp)
curl -sLI "$URL" > "$TMP_HEAD"
head -20 "$TMP_HEAD"
echo "..."
rm -f "$TMP_HEAD"
echo "=== run installer ==="
curl --proto '=https' --tlsv1.2 -LsSf "$URL" | sh
export PATH="$HOME/.local/bin:$PATH"
echo "=== verify binaries on PATH ==="
command -v reposix
command -v git-remote-reposix
echo "=== reposix --version ==="
reposix --version
echo "=== DONE ==="
CONTAINER_EOF
EXIT=${PIPESTATUS[0]}
echo "EXIT=$EXIT"
exit "$EXIT"
