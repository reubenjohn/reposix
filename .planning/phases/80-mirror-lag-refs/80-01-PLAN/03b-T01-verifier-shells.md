← [back to index](./index.md) · [back to 03-T01-catalog-first.md](./03-T01-catalog-first.md) · phase 80 plan 01

## Task 80-01-T01 (continued) — Verifier shell sources

The three TINY verifier shell scripts authored at T01 (extracted here for chapter-budget hygiene; verbatim source from the parent chapter).

`quality/gates/agent-ux/mirror-refs-write-on-success.sh`:

```bash
#!/usr/bin/env bash
# quality/gates/agent-ux/mirror-refs-write-on-success.sh — agent-ux
# verifier for catalog row `agent-ux/mirror-refs-write-on-success`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/mirror-refs-write-on-success
# CADENCE:     pre-pr (~30s wall time)
# INVARIANT:   After a single-backend push via the existing handle_export
#              path, the cache's bare repo has BOTH refs/mirrors/<sot>-head
#              and refs/mirrors/<sot>-synced-at; the synced-at tag's
#              message body's first line matches `mirror synced at <RFC3339>`.
#
# Status until P80-01 T04: FAIL — wiring is scaffold-only in T01-T03;
# the integration tests + behavior coverage land in T04.
#
# Usage: bash quality/gates/agent-ux/mirror-refs-write-on-success.sh
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

# Pick a free high-range port (avoids cross-runner collision per M4).
PORT=$(comm -23 <(seq 49152 65535 | sort) <(ss -tan 2>/dev/null | awk 'NR>1 {print $4}' | awk -F: '{print $NF}' | sort -u) | shuf -n 1)
WORK=$(mktemp -d -t reposix-mirror-refs-write.XXXXXX)
CACHE_DIR=$(mktemp -d -t reposix-mirror-refs-cache.XXXXXX)
SIM_PID=""
cleanup() {
    if [[ -n "${SIM_PID}" ]]; then kill "${SIM_PID}" 2>/dev/null || true; fi
    rm -rf "${WORK}" "${CACHE_DIR}"
}
trap cleanup EXIT

cargo build -p reposix-sim -p reposix-cli --quiet
SIM_BIN="${REPO_ROOT}/target/debug/reposix-sim"
CLI_BIN="${REPO_ROOT}/target/debug/reposix"
"${SIM_BIN}" --bind "127.0.0.1:${PORT}" --ephemeral &
SIM_PID=$!
sleep 1

# Init a working tree against sim::demo, edit, push.
#
# NOTE (H1 fix): `reposix init` does NOT honor REPOSIX_SIM_ORIGIN — it
# hardcodes DEFAULT_SIM_ORIGIN to http://127.0.0.1:7878 in
# crates/reposix-cli/src/init.rs:23-55. Only `reposix attach` reads the
# env var. The init's trailing best-effort `git fetch` against port
# 7878 will fail (sim is on ${PORT}); we re-point remote.origin.url
# AFTER init so the subsequent `git push` reaches our test sim.
# Precedent: crates/reposix-cli/tests/agent_flow.rs::dark_factory_sim_happy_path
# (lines 115-146 — explicit "We re-point the URL to our test sim
# below for any subsequent commands" comment).
REPOSIX_CACHE_DIR="${CACHE_DIR}" \
"${CLI_BIN}" init "sim::demo" "${WORK}" > /dev/null
git -C "${WORK}" config remote.origin.url "reposix::http://127.0.0.1:${PORT}/projects/demo"
cd "${WORK}"
git checkout origin/main -B main -q
# Modify a fixture file and push (the actual edit flow + git push
# cycle drives handle_export's success path; refs are written by T03's
# wiring). REPOSIX_SIM_ORIGIN no longer needed on `git push` — the
# remote URL re-pointing carries the port.
echo "" >> issues/0001.md  # trivial trailing-newline change
git add . && git commit -q -m "trivial change for mirror-refs verifier"
REPOSIX_CACHE_DIR="${CACHE_DIR}" \
git push -q origin main

# Locate the cache's bare repo. The path derives from
# resolve_cache_path("sim", "demo"); since REPOSIX_CACHE_DIR is set,
# the cache lives at ${CACHE_DIR}/reposix/sim-demo.git or similar.
CACHE_BARE=$(find "${CACHE_DIR}" -name '*.git' -type d -print -quit)
[[ -n "${CACHE_BARE}" ]] || { echo "FAIL: cache bare repo not found under ${CACHE_DIR}" >&2; exit 1; }

git -C "${CACHE_BARE}" for-each-ref refs/mirrors/ | grep -q "refs/mirrors/sim-head" \
    || { echo "FAIL: refs/mirrors/sim-head missing" >&2; exit 1; }
git -C "${CACHE_BARE}" for-each-ref refs/mirrors/ | grep -q "refs/mirrors/sim-synced-at" \
    || { echo "FAIL: refs/mirrors/sim-synced-at missing" >&2; exit 1; }

MSG=$(git -C "${CACHE_BARE}" log refs/mirrors/sim-synced-at -1 --format=%B 2>/dev/null | head -1)
[[ "${MSG}" =~ ^mirror\ synced\ at\ [0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}.*Z$ ]] \
    || { echo "FAIL: synced-at tag message body did not match \`mirror synced at <RFC3339>\` (got: ${MSG})" >&2; exit 1; }

echo "PASS: mirror-refs written on push success; both refs resolvable; tag message body well-formed"
exit 0
```

`quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh`:

```bash
#!/usr/bin/env bash
# quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh — agent-ux
# verifier for catalog row `agent-ux/mirror-refs-readable-by-vanilla-fetch`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/mirror-refs-readable-by-vanilla-fetch
# CADENCE:     pre-pr (~30s wall time)
# INVARIANT:   After a single-backend push has populated mirror refs,
#              a fresh `git clone --bare` of the cache's bare repo
#              (or `git fetch` from an existing clone) brings BOTH
#              refs/mirrors/<sot>-head and refs/mirrors/<sot>-synced-at
#              into the new clone — proves vanilla-git readers can
#              observe mirror lag without any reposix awareness.
#
# Status until P80-01 T04: FAIL — wiring is scaffold-only in T01-T03.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

# Pick a free high-range port (M4); H1 — re-point URL after init.
PORT=$(comm -23 <(seq 49152 65535 | sort) <(ss -tan 2>/dev/null | awk 'NR>1 {print $4}' | awk -F: '{print $NF}' | sort -u) | shuf -n 1)
WORK=$(mktemp -d -t reposix-mirror-refs-fetch.XXXXXX)
CACHE_DIR=$(mktemp -d -t reposix-mirror-refs-fetch-cache.XXXXXX)
CLONE=$(mktemp -d -t reposix-mirror-refs-fetch-clone.XXXXXX)
SIM_PID=""
cleanup() {
    if [[ -n "${SIM_PID}" ]]; then kill "${SIM_PID}" 2>/dev/null || true; fi
    rm -rf "${WORK}" "${CACHE_DIR}" "${CLONE}"
}
trap cleanup EXIT

cargo build -p reposix-sim -p reposix-cli --quiet
SIM_BIN="${REPO_ROOT}/target/debug/reposix-sim"
CLI_BIN="${REPO_ROOT}/target/debug/reposix"
"${SIM_BIN}" --bind "127.0.0.1:${PORT}" --ephemeral &
SIM_PID=$!
sleep 1

# H1: REPOSIX_SIM_ORIGIN is a no-op for `reposix init`; we re-point
# remote.origin.url after init so the subsequent `git push` reaches
# our sim. See shell #1's H1 comment block for the full rationale.
REPOSIX_CACHE_DIR="${CACHE_DIR}" \
"${CLI_BIN}" init "sim::demo" "${WORK}" > /dev/null
git -C "${WORK}" config remote.origin.url "reposix::http://127.0.0.1:${PORT}/projects/demo"
cd "${WORK}"
git checkout origin/main -B main -q
echo "" >> issues/0001.md
git add . && git commit -q -m "trivial change for mirror-refs-fetch verifier"
REPOSIX_CACHE_DIR="${CACHE_DIR}" \
git push -q origin main

CACHE_BARE=$(find "${CACHE_DIR}" -name '*.git' -type d -print -quit)
[[ -n "${CACHE_BARE}" ]] || { echo "FAIL: cache bare repo not found" >&2; exit 1; }

# Vanilla `git clone --bare` — no reposix involvement.
git clone --bare -q "${CACHE_BARE}" "${CLONE}/mirror.git"
git -C "${CLONE}/mirror.git" for-each-ref refs/mirrors/ | grep -q "refs/mirrors/sim-head" \
    || { echo "FAIL: vanilla-clone missing refs/mirrors/sim-head" >&2; exit 1; }
git -C "${CLONE}/mirror.git" for-each-ref refs/mirrors/ | grep -q "refs/mirrors/sim-synced-at" \
    || { echo "FAIL: vanilla-clone missing refs/mirrors/sim-synced-at" >&2; exit 1; }

echo "PASS: vanilla-fetch brings refs/mirrors/* along to a fresh bare clone"
exit 0
```

`quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh`:

```bash
#!/usr/bin/env bash
# quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh — agent-ux
# verifier for catalog row `agent-ux/mirror-refs-cited-in-reject-hint`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/mirror-refs-cited-in-reject-hint
# CADENCE:     pre-pr (~30s wall time)
# INVARIANT:   After a successful push (refs populated), a SECOND push
#              with a stale prior triggers the conflict-reject path;
#              the helper's stderr cites refs/mirrors/<sot>-synced-at
#              with a parseable RFC3339 timestamp + `(N minutes ago)`.
#
# Status until P80-01 T04: FAIL.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

# Pick a free high-range port (M4); H1 — re-point URL after init.
PORT=$(comm -23 <(seq 49152 65535 | sort) <(ss -tan 2>/dev/null | awk 'NR>1 {print $4}' | awk -F: '{print $NF}' | sort -u) | shuf -n 1)
WORK1=$(mktemp -d -t reposix-mirror-refs-reject-w1.XXXXXX)
WORK2=$(mktemp -d -t reposix-mirror-refs-reject-w2.XXXXXX)
CACHE_DIR=$(mktemp -d -t reposix-mirror-refs-reject-cache.XXXXXX)
STDERR_CAP=$(mktemp -t reposix-mirror-refs-reject-stderr.XXXXXX)
SIM_PID=""
cleanup() {
    if [[ -n "${SIM_PID}" ]]; then kill "${SIM_PID}" 2>/dev/null || true; fi
    rm -rf "${WORK1}" "${WORK2}" "${CACHE_DIR}" "${STDERR_CAP}"
}
trap cleanup EXIT

cargo build -p reposix-sim -p reposix-cli --quiet
SIM_BIN="${REPO_ROOT}/target/debug/reposix-sim"
CLI_BIN="${REPO_ROOT}/target/debug/reposix"
"${SIM_BIN}" --bind "127.0.0.1:${PORT}" --ephemeral &
SIM_PID=$!
sleep 1

# First successful push from WORK1 — populates refs/mirrors/*.
# H1: re-point remote.origin.url after init (REPOSIX_SIM_ORIGIN no-op
# for `reposix init` — see shell #1's H1 comment block for rationale).
REPOSIX_CACHE_DIR="${CACHE_DIR}" \
"${CLI_BIN}" init "sim::demo" "${WORK1}" > /dev/null
git -C "${WORK1}" config remote.origin.url "reposix::http://127.0.0.1:${PORT}/projects/demo"
( cd "${WORK1}" && git checkout origin/main -B main -q && \
  echo "" >> issues/0001.md && git add . && git commit -q -m "first push" && \
  REPOSIX_CACHE_DIR="${CACHE_DIR}" \
  git push -q origin main )

sleep 2  # ensure non-zero "(N minutes ago)" math, even if N=0

# Second push from WORK2 against a STALE prior — conflict-reject path.
# WORK2's local clone never sees WORK1's push; pushing produces a
# version mismatch detected by handle_export's existing conflict logic.
# H1: same re-point dance for WORK2.
REPOSIX_CACHE_DIR="${CACHE_DIR}" \
"${CLI_BIN}" init "sim::demo" "${WORK2}" > /dev/null
git -C "${WORK2}" config remote.origin.url "reposix::http://127.0.0.1:${PORT}/projects/demo"
( cd "${WORK2}" && git checkout origin/main -B main -q && \
  # WORK2 is now stale — the sim has advanced one version via WORK1's push.
  # Edit the same file WORK1 just modified to trigger conflict.
  echo "stale-prior" >> issues/0001.md && git add . && \
  git commit -q -m "stale push" && \
  REPOSIX_CACHE_DIR="${CACHE_DIR}" \
  git push origin main 2> "${STDERR_CAP}" || true )

grep -q "refs/mirrors/sim-synced-at" "${STDERR_CAP}" \
    || { echo "FAIL: reject stderr missing refs/mirrors/sim-synced-at citation" >&2; cat "${STDERR_CAP}" >&2; exit 1; }
grep -qE "[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}.*Z" "${STDERR_CAP}" \
    || { echo "FAIL: reject stderr missing RFC3339 timestamp" >&2; cat "${STDERR_CAP}" >&2; exit 1; }
grep -qE "[0-9]+ minutes ago" "${STDERR_CAP}" \
    || { echo "FAIL: reject stderr missing '(N minutes ago)' rendering" >&2; cat "${STDERR_CAP}" >&2; exit 1; }

echo "PASS: conflict-reject hint cites refs/mirrors/sim-synced-at with RFC3339 + (N minutes ago)"
exit 0
```

The hint-text scenario above re-uses a single shared CACHE_DIR for
both WORK1 and WORK2 to ensure the conflict-detection logic in
`handle_export` sees the version mismatch. If during T03/T04
execution a different scenario shape proves more reliable (e.g.,
two CACHE_DIRs with the second `init` accepting a stale prior),
adjust the verifier — but the assertion contract (stderr cites
`refs/mirrors/sim-synced-at` + RFC3339 + `(N minutes ago)`) MUST
stay stable.
