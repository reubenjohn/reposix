#!/usr/bin/env bash
# P56 RELEASE-01 — asset-existence check for a release artifact URL.
#
# Performs a HEAD request and grabs the first 32 bytes of the asset.
# Emits status=<HTTP_STATUS> length=<CONTENT_LENGTH> + leading bytes.
#
# Used by Task 56-03-C (powershell-installer-ps1 asset-existence path —
# container rehearsal for the windows installer is deferred to v0.12.1
# per cost decision).
#
# Inputs:
#   $1                      asset URL (required)
#
# Outputs:
#   stdout: status=<n> length=<n>
#           leading bytes: <first 32 bytes>
#   exit 0 if HTTP 200, exit 1 otherwise.
#
# Usage:
#   bash scripts/p56-asset-existence.sh \
#       https://github.com/reubenjohn/reposix/releases/latest/download/reposix-installer.ps1

set -euo pipefail

URL="${1:?usage: $0 <asset-url>}"

HEAD_OUT=$(curl -sLI "$URL")
HTTP_STATUS=$(echo "$HEAD_OUT" | grep -E "^HTTP/" | tail -1 | awk '{print $2}')
CONTENT_LENGTH=$(echo "$HEAD_OUT" | grep -iE "^content-length:" | tail -1 | awk '{print $2}' | tr -d '\r')
LEADING_BYTES=$(curl -sL --range 0-31 "$URL")

echo "status=$HTTP_STATUS length=$CONTENT_LENGTH"
echo "leading bytes: $LEADING_BYTES"

[[ "$HTTP_STATUS" == "200" ]]
