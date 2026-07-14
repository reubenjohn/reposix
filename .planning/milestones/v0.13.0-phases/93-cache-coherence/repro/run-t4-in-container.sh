#!/usr/bin/env bash
# RBF-LR-05 (P93 mid-stream litmus, T4 leg) container driver.
#
# Runs the REAL gate script `quality/gates/agent-ux/t4-conflict-rebase-ancestry.sh`
# unmodified inside a git >= 2.34 container (this host is git 2.25.1, below the
# script's own version floor -- see that script's "GIT VERSION GATE" comment).
# Reuses the `reposix-repro:git254` image (ubuntu:24.04 + git-core PPA) built
# from ./Dockerfile, same pattern as run-repro.sh (D-P92-03 investigation).
#
# The gate script's own `build_and_resolve_bins()` (dark-factory/lib.sh) always
# shells out to `cargo build --workspace --bins -q` before resolving
# target/{debug,release}/reposix. This container has no rust toolchain (adding
# one would balloon the container setup with a full workspace rebuild under an
# unrelated libc/rustc pairing). Instead: the workspace is bind-mounted at the
# IDENTICAL absolute path (so the script's own WORKSPACE_ROOT/PATH resolution
# is unchanged), pre-built HOST binaries already sit in target/debug (built by
# this same session's earlier steps, with the host's real cargo -- forward-
# compatible glibc: host is Ubuntu 20.04/glibc 2.31, container is Ubuntu
# 24.04/glibc 2.39, and `ldd` on the reposix binaries shows only glibc/libgcc
# deps, no libssl/libsqlite3 dynamic linkage), and a one-line no-op `cargo`
# shim (./cargo-shim/cargo, NOT committed -- caller supplies via $1) is placed
# earlier on PATH so the build step is skipped without touching the gate
# script or lib.sh.
#
# Usage: run-t4-in-container.sh <cargo-shim-dir>
set -euo pipefail

CARGO_SHIM_DIR="${1:?usage: run-t4-in-container.sh <cargo-shim-dir> (dir containing an executable no-op 'cargo')}"
[[ -x "${CARGO_SHIM_DIR}/cargo" ]] || { echo "FATAL: ${CARGO_SHIM_DIR}/cargo missing or not executable" >&2; exit 2; }

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$HERE/../../../.." && pwd)"
IMG=reposix-repro:git254

docker image inspect "$IMG" >/dev/null 2>&1 || docker build -t "$IMG" "$HERE"

echo "== T4 in git-2.54 container: repo mounted at ${REPO} (identical path) =="
docker run --rm \
  -v "${REPO}:${REPO}" \
  -v "${CARGO_SHIM_DIR}:/cargo-shim-ro:ro" \
  -w "${REPO}" \
  "$IMG" bash -c '
    set -euo pipefail
    echo "== in-container git --version =="
    git --version
    echo "== installing python3 (needed by t4-conflict-rebase-ancestry.sh JSON asserts_passed encoding) =="
    apt-get update -qq
    apt-get install -y -qq --no-install-recommends python3 >/dev/null
    export PATH="/cargo-shim-ro:${PATH}"
    echo "== running quality/gates/agent-ux/t4-conflict-rebase-ancestry.sh (UNMODIFIED) =="
    bash quality/gates/agent-ux/t4-conflict-rebase-ancestry.sh
  '
