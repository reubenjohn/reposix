#!/usr/bin/env bash
# quality/gates/security/allowlist-enforcement.sh -- security/allowlist-enforcement verifier.
#
# CLAUDE.md threat-model "Outbound HTTP allowlist" cut (SG-01/SG-07): every
# outbound HTTP call MUST re-check REPOSIX_ALLOWED_ORIGINS BEFORE any I/O.
# This row's verifier script was dangling (P90 90-05 R2 § D/H finding #3) --
# the path named in the catalog row did not exist, even though the runtime
# behavior it claims to cover was already real and tested. Landed here as a
# thin wrapper, mirroring connector-audit-wired.sh's shape:
#
#   1. cargo test -p reposix-core --test http_allowlist -- the REAL
#      integration-test file (crates/reposix-core/tests/http_allowlist.rs,
#      13 #[tokio::test] fns, 1 #[ignore = "sleeps ~5s..."] for the timeout
#      case) drives HttpClient::request/get/post/patch/delete against a
#      non-allowlisted host and asserts Error::InvalidOrigin + a <500ms
#      short-circuit (no DNS/TCP attempted), PLUS the redirect-recheck +
#      env-override + loopback-allow cases.
#   2. A static grep confirming (a) clippy.toml's disallowed-methods bans
#      direct reqwest::Client::new/builder construction (production code
#      cannot bypass the factory), and (b) the production
#      request_with_headers_and_body function (the single body all
#      request/get/post/patch/delete wrappers funnel through) calls
#      load_allowlist_from_env() and rejects BEFORE self.inner.request(...)
#      is reached -- so a regression that moves the check after the send
#      call cannot slip through even if the test file itself weren't
#      touched.
#
# NOTE (honesty caveat, updated 2026-07-05): at P90 90-05 (2026-07-04) this
# script had NOT yet been executed by the agent that wrote it --
# CLAUDE.md's "Build memory budget" section forbade ad-hoc cargo
# invocations during framework-dimension (F-class) dispatches on this VM.
# Correctness was established at that time by manual line-by-line reading
# of crates/reposix-core/tests/http_allowlist.rs and
# crates/reposix-core/src/http.rs:294-321 (quoted verbatim in the commit).
# That gap is now CLOSED: this gate was executed via a real `cargo test`
# run on 2026-07-05 -- 12 tests passing (13 fns, 1 #[ignore]d timeout
# case) plus both static greps confirmed true -- and independently
# reconfirmed in P92 CI run 28735908764 (git 2.54, all jobs success). The
# catalog row (quality/catalogs/security-gates.json,
# security/allowlist-enforcement) reflects WAIVED→PASS as of that run; see
# its owner_hint field and quality/reports/verifications/security/
# allowlist-enforcement.json for the full artifact.
#
# Honors --row-id <id> (defaults to security/allowlist-enforcement).
# Implements catalog row security/allowlist-enforcement.
set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
readonly ARTIFACT_DIR="${REPO_ROOT}/quality/reports/verifications/security"
readonly ARTIFACT="${ARTIFACT_DIR}/allowlist-enforcement.json"
readonly HTTP_SRC="${REPO_ROOT}/crates/reposix-core/src/http.rs"
readonly CLIPPY_TOML="${REPO_ROOT}/clippy.toml"
readonly MIN_TESTS=10

row_id="security/allowlist-enforcement"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  row_id="$2"
fi

cd "$REPO_ROOT"
mkdir -p "$ARTIFACT_DIR"

stdout=$(cargo test -p reposix-core --test http_allowlist 2>&1) && cargo_exit=0 || cargo_exit=$?
ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)

ran_total=$(printf '%s\n' "$stdout" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+' | awk '{s+=$1} END {print s+0}')
failed_total=$(printf '%s\n' "$stdout" | grep -oE '[0-9]+ failed' | grep -oE '[0-9]+' | awk '{s+=$1} END {print s+0}')

# Static grep #1: clippy.toml bans direct reqwest client construction.
clippy_ok=0
if grep -q 'reqwest::Client::new' "$CLIPPY_TOML" && grep -q 'reqwest::ClientBuilder::new' "$CLIPPY_TOML"; then
  clippy_ok=1
fi

# Static grep #2: production request_with_headers_and_body (the shared body
# for request/get/post/patch/delete) rejects BEFORE self.inner.request(...).
# Isolate the function body, then require load_allowlist_from_env to appear
# textually before self.inner.request within it.
gate_ok=0
# Grab from the fn signature line to the next top-level "    }" close
# (4-space indent) -- a small deterministic window around the fn body.
fn_body=$(sed -n '/pub async fn request_with_headers_and_body/,/^    }/p' "$HTTP_SRC")
if printf '%s\n' "$fn_body" | grep -q 'load_allowlist_from_env' \
   && printf '%s\n' "$fn_body" | grep -q 'self\.inner\.request'; then
  check_line=$(printf '%s\n' "$fn_body" | grep -n 'load_allowlist_from_env' | head -1 | cut -d: -f1)
  send_line=$(printf '%s\n' "$fn_body" | grep -n 'self\.inner\.request' | head -1 | cut -d: -f1)
  if [[ "$check_line" -lt "$send_line" ]]; then
    gate_ok=1
  fi
fi

exit_code=0
asserts_passed=()
asserts_failed=()

if [[ "$cargo_exit" -ne 0 ]]; then
  exit_code=1
  asserts_failed+=("cargo test -p reposix-core --test http_allowlist exited ${cargo_exit} (nonzero)")
elif [[ "$failed_total" -gt 0 ]]; then
  exit_code=1
  asserts_failed+=("${failed_total} http_allowlist test(s) FAILED")
else
  asserts_passed+=("cargo test -p reposix-core --test http_allowlist exits 0")
fi

if [[ "$ran_total" -lt "$MIN_TESTS" ]]; then
  exit_code=1
  asserts_failed+=("only ${ran_total} test(s) ran; expected >= ${MIN_TESTS} (egress-rejected + redirect-recheck + env-override + loopback-allow cases)")
else
  asserts_passed+=("${ran_total} http_allowlist test(s) ran (>= ${MIN_TESTS} required)")
fi

if [[ "$clippy_ok" -eq 1 ]]; then
  asserts_passed+=("clippy.toml disallowed-methods bans reqwest::Client::new + ::builder outside http::client()")
else
  exit_code=1
  asserts_failed+=("clippy.toml missing reqwest::Client disallowed-methods entries")
fi

if [[ "$gate_ok" -eq 1 ]]; then
  asserts_passed+=("production request_with_headers_and_body() checks load_allowlist_from_env() BEFORE self.inner.request(...) in ${HTTP_SRC}")
else
  exit_code=1
  asserts_failed+=("could not confirm allowlist-check-before-send ordering in ${HTTP_SRC}")
fi

py_json_array() {
  if [[ "$#" -eq 0 ]]; then
    echo "[]"
  else
    python3 -c "import json,sys; print(json.dumps(sys.argv[1:]))" "$@"
  fi
}
asserts_passed_json=$(py_json_array "${asserts_passed[@]}")
asserts_failed_json=$(py_json_array "${asserts_failed[@]}")
stdout_json=$(printf '%s' "$stdout" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read()))")

cat > "$ARTIFACT" <<EOF
{
  "ts": "$ts",
  "row_id": "$row_id",
  "exit_code": $exit_code,
  "cargo_exit_code": $cargo_exit,
  "tests_ran": $ran_total,
  "tests_failed": $failed_total,
  "clippy_disallowed_methods_ok": $([ "$clippy_ok" -eq 1 ] && echo true || echo false),
  "allowlist_gate_before_send_ok": $([ "$gate_ok" -eq 1 ] && echo true || echo false),
  "stdout": $stdout_json,
  "asserts_passed": $asserts_passed_json,
  "asserts_failed": $asserts_failed_json
}
EOF

if [[ "$exit_code" -ne 0 ]]; then
  echo "$stdout" >&2
fi
exit "$exit_code"
