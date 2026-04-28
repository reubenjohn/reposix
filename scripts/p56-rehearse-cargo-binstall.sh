#!/usr/bin/env bash
# KEEP-AS-CANONICAL (P63 SIMPLIFY-12): canonical for own domain (P56 install-evidence rehearsal); CLAUDE.md P56 section names this exact path.
# P56 RELEASE-01 — container rehearsal for the `cargo binstall` install path.
#
# Runs `cargo binstall reposix-cli reposix-remote` inside a fresh
# rust:1.82-slim container, captures the transcript, and asserts that
# binstall picked the prebuilt GH-release binary (NOT a source build
# fallback — which is functionally fine but slow and doesn't exercise the
# pipeline we care about).
#
# Used by:
#   - Wave 3 of P56 (this script's first call) to grade
#     install/cargo-binstall PASS/PARTIAL/FAIL.
#   - Wave 4's verifier subagent — same re-run-with-zero-context contract
#     as p56-rehearse-curl-install.sh.
#
# Inputs (env vars):
#   REPOSIX_LOG_PATH        where to write the container transcript
#                           (default: /tmp/p56-binstall-rehearsal.log).
#
# Outputs:
#   - transcript at $REPOSIX_LOG_PATH
#   - exit 0 on PASS (prebuilt-binary path used)
#   - exit 2 on PARTIAL (binstall succeeded but fell back to source build —
#     functionally fine but binstall metadata in
#     crates/reposix-cli/Cargo.toml [package.metadata.binstall] needs work)
#   - exit 1 on FAIL (binstall didn't install at all)
#
# Usage:
#   bash scripts/p56-rehearse-cargo-binstall.sh

set -euo pipefail

LOG="${REPOSIX_LOG_PATH:-/tmp/p56-binstall-rehearsal.log}"

set +e
docker run --rm -i rust:1.82-slim bash -s <<'CONTAINER_EOF' 2>&1 | tee "$LOG"
set -euo pipefail
apt-get update -qq && apt-get install -y -qq curl ca-certificates >/dev/null 2>&1
curl -L --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo binstall --no-confirm reposix-cli reposix-remote
command -v reposix
command -v git-remote-reposix
reposix --version
CONTAINER_EOF
DOCKER_EXIT=${PIPESTATUS[0]}
set -e

echo "DOCKER_EXIT=$DOCKER_EXIT"
if [[ "$DOCKER_EXIT" -ne 0 ]]; then
    echo "FAIL: container exited non-zero — binstall did not install reposix"
    exit 1
fi

if grep -qE '(Downloading|Installing).+github\.com/reubenjohn/reposix/releases' "$LOG"; then
    echo "PASS: binstall used prebuilt-binary path"
    exit 0
else
    echo "PARTIAL: binstall succeeded but fell back to source build"
    echo "  (binstall metadata in crates/reposix-cli/Cargo.toml may need pkg-url tweaks)"
    exit 2
fi
