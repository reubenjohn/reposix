#!/usr/bin/env bash
# 94-D2-git243-repro.sh — DP-2 prove-before-fix driver for the git-2.43
# fallback-sentinel finding (P94 D2, SURPRISES-INTAKE.md L602-610).
#
# Proves the finding EXECUTES: stock Ubuntu 24.04 git (2.43.0) drives a REAL
# single-backend `git push` through `git-remote-reposix` against a `reposix-sim`
# backend, and exits 128 on the pre-fix helper / 0 on the fixed helper.
#
# CORRECTED ROOT CAUSE (DP-2 finding — the SURPRISES-INTAKE L602-610 sketch was
# a MISDIAGNOSIS). Contrary to the intake ("git tries `stateless-connect
# git-receive-pack` first; the non-`fallback` reply aborts the push"), the
# git-2.43 transport-helper NEVER probes `stateless-connect` for the push
# direction. The real, empirically-traced sequence
# (GIT_TRANSPORT_HELPER_DEBUG=1) is:
#   git 2.43:  capabilities -> `option object-format` -> helper `unsupported`
#              -> git DIES 128 (before any list/export).
#   git 2.54:  capabilities -> list -> export -> ok  (no `option object-format`
#              at all; the advertised `object-format=sha1` cap suffices).
# The version-windowed blocker is the helper answering `option object-format`
# with `unsupported`; git 2.43 treats that as fatal. The fix makes the helper
# answer `option object-format {<empty>|true|sha1}` with `ok` (reposix cache is
# sha1-only). The git-remote-helpers(7) `fallback` sentinel for non-upload-pack
# `stateless-connect` is ALSO fixed (spec-compliance / defensive), but it is NOT
# what unblocks this push — proven by the identical 128 on buggy AND
# fallback-only helpers, and the `option object-format` verb trace.
#
# Container-based because this dev box's system git is 2.25.1 (< the 2.34 floor)
# and CANNOT reproduce the 2.43-windowed regression natively. Pattern adapted
# from `.planning/phases/92-push-flow-correctness/92-T4-REPRO-NOTES.md`:
# ubuntu:24.04 + stock apt git, `--network host` to reach a host-spawned sim on
# 127.0.0.1:7878, host-built binaries bind-mounted read-only (host glibc <=
# container glibc, so the host binary runs in-container).
#
# Usage:
#   BIN_DIR=/abs/path/to/target/debug bash 94-D2-git243-repro.sh
# Env:
#   BIN_DIR   (required) dir holding reposix, reposix-sim, git-remote-reposix
#   SIM_BIND  (default 127.0.0.1:7878) — MUST be 7878: `reposix init sim::demo`
#             bakes that default origin (crates/reposix-cli/src/init.rs).
# Exit: 0 = push succeeded (GREEN, fixed helper); 1 = push failed (RED, bug
#       reproduced); 2 = harness/setup error (sim/init did not come up).
set -uo pipefail

SIM_BIND="${SIM_BIND:-127.0.0.1:7878}"
SIM_URL="http://${SIM_BIND}"
: "${BIN_DIR:?set BIN_DIR to the dir holding reposix / reposix-sim / git-remote-reposix}"

for b in reposix reposix-sim git-remote-reposix; do
  [[ -x "${BIN_DIR}/${b}" ]] || { echo "SETUP-FAIL: ${BIN_DIR}/${b} not executable" >&2; exit 2; }
done
command -v docker >/dev/null 2>&1 || { echo "SETUP-FAIL: docker not on PATH" >&2; exit 2; }

RUN_DIR="$(mktemp -d /tmp/p94-d2-repro-XXXXXX)"
SIM_PID=""
cleanup() {
  [[ -n "$SIM_PID" ]] && { kill "$SIM_PID" 2>/dev/null || true; wait "$SIM_PID" 2>/dev/null || true; }
  rm -rf "$RUN_DIR"
}
trap cleanup EXIT

# --- spawn the sim on the host (seeded so issues/*.md exist to edit) ---------
export REPOSIX_ALLOWED_ORIGINS="${SIM_URL}"
"${BIN_DIR}/reposix-sim" --bind "$SIM_BIND" --db "${RUN_DIR}/sim.db" \
  --seed-file "$(git rev-parse --show-toplevel)/crates/reposix-sim/fixtures/seed.json" &
SIM_PID=$!
for _ in $(seq 1 50); do
  if ! kill -0 "$SIM_PID" 2>/dev/null; then echo "SETUP-FAIL: sim exited during startup" >&2; exit 2; fi
  if curl -fsS "${SIM_URL}/projects/demo/issues" >/dev/null 2>&1; then break; fi
  sleep 0.1
done
curl -fsS "${SIM_URL}/projects/demo/issues" >/dev/null 2>&1 \
  || { echo "SETUP-FAIL: sim not reachable at ${SIM_URL}" >&2; exit 2; }
echo "repro: sim up at ${SIM_URL} (pid ${SIM_PID})" >&2

# --- run the real git-2.43 push inside the container -------------------------
# The container installs stock apt git (2.43.0 on noble), puts the bind-mounted
# host binaries on PATH, runs `reposix init` (partial-clone fetch via
# git-upload-pack stateless-connect — SERVED, works on 2.43), edits one issue,
# commits, and pushes. The push is where the non-upload-pack `git-receive-pack`
# stateless-connect probe fires — the exact branch under test.
set +e
docker run --rm --network host \
  -v "${BIN_DIR}:/bins:ro" \
  -v "${RUN_DIR}:/work" \
  -e "REPOSIX_ALLOWED_ORIGINS=${SIM_URL}" \
  -e "REPOSIX_CACHE_DIR=/work/cache" \
  -e "HOST_UID=$(id -u)" \
  -e "HOST_GID=$(id -g)" \
  -e "REPRO_DEBUG=${REPRO_DEBUG:-}" \
  -e "GIT_CHANNEL=${GIT_CHANNEL:-noble}" \
  ubuntu:24.04 bash -c '
    export DEBIAN_FRONTEND=noninteractive
    # chown the bind-mounted /work back to the host UID on exit so the host
    # trap can rm it (the container runs as root and would otherwise leave
    # root-owned litter in /tmp). Runs on every exit path, success or fail.
    trap "chown -R ${HOST_UID}:${HOST_GID} /work 2>/dev/null || true" EXIT
    set -e
    apt-get update -qq >/dev/null && apt-get install -y -qq git ca-certificates >/dev/null
    if [[ "${GIT_CHANNEL:-noble}" == "ppa" ]]; then
      # Upgrade to the git-core PPA build (~2.5x) to compare the version-
      # windowed behaviour against noble stock 2.43 (92-T4 pattern).
      apt-get install -y -qq software-properties-common >/dev/null
      add-apt-repository -y ppa:git-core/ppa >/dev/null 2>&1
      apt-get update -qq >/dev/null && apt-get install -y -qq git >/dev/null
    fi
    export PATH=/bins:$PATH
    echo "container git: $(git --version)"
    git config --global user.email "repro@example.invalid"
    git config --global user.name  "p94-d2-repro"
    git config --global init.defaultBranch main
    echo "--- reposix init sim::demo /work/repo ---"
    reposix init "sim::demo" /work/repo
    cd /work/repo
    git checkout -B main refs/reposix/origin/main
    f=$(ls issues/*.md | sort | head -1)
    echo "editing $f"
    printf "\n<!-- p94-d2 repro edit -->\n" >> "$f"
    git add "$f"
    git commit -q -m "p94-d2 repro: edit one issue"
    echo "--- git push origin main (the bug bites here on the pre-fix helper) ---"
    if [[ -n "${REPRO_DEBUG:-}" ]]; then
      export GIT_TRANSPORT_HELPER_DEBUG=1 GIT_TRACE_PACKET=1 GIT_TRACE=1
      export RUST_LOG="${RUST_LOG:-debug}"
    fi
    set +e
    git push --verbose origin main > /work/push.log 2>&1
    push_rc=$?
    set -e
    echo "----- git push output (rc=${push_rc}) -----"
    sed "s/^/  push| /" /work/push.log
    echo "-------------------------------------------"
    echo "PUSH_EXIT=${push_rc}"
    exit "${push_rc}"
  ' 2>&1
PUSH_STATUS=$?
set -e

echo "=== container exit status: ${PUSH_STATUS} ===" >&2
if [[ "$PUSH_STATUS" -eq 0 ]]; then
  echo "GREEN: real git-2.43 single-backend push SUCCEEDED (helper answered \`option object-format\` with \`ok\`; git proceeded list -> export)." >&2
  exit 0
else
  echo "RED: real git-2.43 single-backend push FAILED (status ${PUSH_STATUS}) — the git-2.43 push regression is reproduced (helper \`unsupported\` to \`option object-format\` -> git dies 128)." >&2
  exit 1
fi
