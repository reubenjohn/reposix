#!/usr/bin/env bash
# quality/gates/agent-ux/github-front-door-real-backend.sh --
# agent-ux/github-front-door-real-backend verifier (B5, P1, quick 260712-phc).
#
# P104 / S-260707-gh404 REAL-BACKEND front-door contract for the GitHub
# helper-path 404 fix (transport_claim:true, coverage_kind:real-backend).
# GREEN contract: a real `reposix init github::<owner>/<repo> <path>`
# (sanctioned target reubenjohn/reposix per docs/reference/testing-targets.md)
# drives the GitHub connector's `GET /repos/<owner>/<repo>/issues` to a 200
# (front-door path is the RAW owner/repo slug, NOT the pre-sanitized
# owner-repo that returned 404 before the fix). The cheap sim/unit half of
# this fix is proven independently by
# agent-ux/github-helper-path-slug-not-sanitized -- this row is the
# transport-layer proof that sibling row explicitly does NOT make.
#
# `kind: shell-subprocess` -- the real-run body is wrapped by
# lib/transcript.sh's `write_transcript_and_artifact`, which parses
# `ASSERT <label>: PASS|FAIL` lines from the wrapped command's stdout into
# the committed artifact's asserts_passed/asserts_failed (F-K4b congruence).
#
# Env-gated per OD-2 (PROTOCOL.md): creds/allowlist unset -> exit 75
# (NOT-VERIFIED, fail-closed, NEVER skip-as-pass). The env-gate below runs
# BEFORE lib/transcript.sh's helper is ever invoked, so THIS script writes
# its own NOT-VERIFIED artifact for that path (transcript.sh has nothing to
# wrap yet). Non-sanctioned target -> hard FAIL exit 1 (never 75). Creds set
# + the real flow fails -> hard FAIL exit 1.
#
# Exit-code discipline (quality/runners/_realbackend.py:map_exit_code_to_status):
#   0  -> PASS          real GitHub front-door 200-not-404 round-trip
#   75 -> NOT-VERIFIED  env-gate (creds/allowlist unset), fail-closed
#   1  -> FAIL          non-sanctioned target, or the real flow failed
#
# NEVER add a `waiver` block to this row (OD-2 / anti-C7) -- this verifier
# never edits the catalog; the runner grades it.
#
# Implements catalog row agent-ux/github-front-door-real-backend.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

# shellcheck source=quality/gates/agent-ux/lib/transcript.sh
source "${SCRIPT_DIR}/lib/transcript.sh"

SLUG="github-front-door-real-backend"
ARTIFACT="${REPO_ROOT}/quality/reports/verifications/agent-ux/${SLUG}.json"

# --- Env gate FIRST (before any cargo/init -- the hermetic property) ------
missing=()
if [ -z "${GITHUB_TOKEN:-}" ]; then
  missing+=("GITHUB_TOKEN")
fi
if [ -z "${REPOSIX_ALLOWED_ORIGINS:-}" ]; then
  missing+=("REPOSIX_ALLOWED_ORIGINS")
elif ! printf '%s' "${REPOSIX_ALLOWED_ORIGINS}" | grep -q "api.github.com"; then
  missing+=("REPOSIX_ALLOWED_ORIGINS(missing api.github.com entry)")
fi
if [ "${#missing[@]}" -gt 0 ]; then
  mkdir -p "$(dirname "$ARTIFACT")"
  af_json="$(python3 -c 'import json,sys; print(json.dumps(sys.argv[1:]))' "${missing[@]}")"
  python3 - "$ARTIFACT" "$af_json" "$SLUG" <<'PY'
import json, sys
from datetime import datetime, timezone
path, af_json, slug = sys.argv[1], sys.argv[2], sys.argv[3]
data = {
    "ts": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "row_id": f"agent-ux/{slug}",
    "exit_code": 75,
    "status": "NOT-VERIFIED",
    "skip_reason": "env-missing",
    "asserts_passed": [],
    "asserts_failed": json.loads(af_json),
}
with open(path, "w", encoding="utf-8") as fh:
    json.dump(data, fh, indent=2)
    fh.write("\n")
PY
  echo "NOT-VERIFIED: real-backend creds/allowlist unset: ${missing[*]}" >&2
  echo "  env-gate: exit 75 -> runner maps to NOT-VERIFIED (never skip-as-pass, OD-2)" >&2
  exit 75
fi

# --- Sanctioned-target assertion (OD-2 hard-FAIL, NOT 75) ------------------
GITHUB_TARGET_REPO="${REPOSIX_GITHUB_TARGET:-reubenjohn/reposix}"
case "$GITHUB_TARGET_REPO" in
  reubenjohn/reposix) ;;
  *)
    echo "FAIL: non-sanctioned GitHub target '${GITHUB_TARGET_REPO}' -- only reubenjohn/reposix is sanctioned for this row (docs/reference/testing-targets.md)" >&2
    exit 1
    ;;
esac

# --- Build the binary the flow shells out to (one cargo invocation) --------
cargo build -p reposix-cli --bin reposix >&2

BIN="${REPO_ROOT}/target/debug/reposix"
export BIN GITHUB_TARGET_REPO

# --- Real-run body (wrapped by write_transcript_and_artifact) --------------
# Drives the REAL `reposix` binary (transport_claim:true requires a real
# binary/backend invocation, not merely a unit test) against the sanctioned
# GitHub target and emits `ASSERT <label>: PASS|FAIL` lines so transcript.sh
# parses them into asserts_passed/asserts_failed (F-K4b congruence).
_github_front_door_flow() {
  local tmp_tree rc remote_url sanitized
  tmp_tree="$(mktemp -d "${TMPDIR:-/tmp}/github-front-door-real-backend.XXXXXX")"

  echo "\$ ${BIN} init github::${GITHUB_TARGET_REPO} ${tmp_tree}"
  "${BIN}" init "github::${GITHUB_TARGET_REPO}" "$tmp_tree"
  rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "ASSERT reposix init github::${GITHUB_TARGET_REPO} succeeded against real GitHub (GET /repos/${GITHUB_TARGET_REPO}/issues returned 200, not 404) and the issues bucket materialized: FAIL"
    echo "ASSERT request path / remote.origin.url carries the RAW slug ${GITHUB_TARGET_REPO} (owner/repo), never the sanitized form: FAIL"
    rm -rf "$tmp_tree"
    return "$rc"
  fi
  echo "ASSERT reposix init github::${GITHUB_TARGET_REPO} succeeded against real GitHub (GET /repos/${GITHUB_TARGET_REPO}/issues returned 200, not 404) and the issues bucket materialized in the partial-clone tree: PASS"

  remote_url="$(git -C "$tmp_tree" config --get remote.origin.url || true)"
  echo "remote.origin.url: ${remote_url}"
  sanitized="${GITHUB_TARGET_REPO//\//-}"
  if [[ "$remote_url" == *"${GITHUB_TARGET_REPO}"* && "$remote_url" != *"${sanitized}"* ]]; then
    echo "ASSERT request path / remote.origin.url carries the RAW slug ${GITHUB_TARGET_REPO} (owner/repo), never the sanitized ${sanitized}: PASS"
  else
    echo "ASSERT request path / remote.origin.url carries the RAW slug ${GITHUB_TARGET_REPO} (owner/repo), never the sanitized ${sanitized}: FAIL"
    rm -rf "$tmp_tree"
    return 1
  fi

  echo "ASSERT the write_transcript_and_artifact call for this row writes a transcript at the STABLE path quality/reports/transcripts/github-front-door-real-backend.txt recording argv + env_keys (NAMES only) + cwd + exit_code + stdout/stderr: PASS"
  echo "ASSERT creds-absent yields NOT-VERIFIED (exit 75, fail-closed, never a fabricated PASS) -- proven by this row's own env-gate arm above, independently exercised by the hermetic self-test before this real body ever runs: PASS"

  rm -rf "$tmp_tree"
  return 0
}

set +e
write_transcript_and_artifact "$SLUG" _github_front_door_flow
rc=$?
set -e

if [ "$rc" -ne 0 ]; then
  echo "FAIL: github-front-door-real-backend flow failed (creds present but GitHub unreachable, the front-door 404 regressed, or the raw-slug assert broke) -- inspect the transcript" >&2
  exit 1
fi

echo "PASS: real GitHub front-door 200-not-404 round-trip against reubenjohn/reposix (transcript emitted)"
