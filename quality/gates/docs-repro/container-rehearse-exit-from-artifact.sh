#!/usr/bin/env bash
# quality/gates/docs-repro/container-rehearse-exit-from-artifact.sh
#
# P124/SC4 (DRAIN-13 + DRAIN-14). Proves two properties of container-rehearse.sh:
#   (A) DRAIN-13 -- its exit is derived STRICTLY from the PERSISTED artifact exit_code, so a
#       docker rc that DISAGREES can never mask an artifact exit_code=1 (the rc=0-masks-
#       exit_code=1 gap W1a's live run reproduced). The exit_code is congruence-graded (rc=0
#       AND every expected.assert earned -> 0, else 1), and the sim-reachability readiness
#       leg surfaces a sim-not-reachable failure NON-ZERO instead of masking it as success.
#   (B) DRAIN-14 -- the per-run `.sim-*.log` runtime logs under
#       quality/reports/verifications/docs-repro/ are git-ignored (no longer leak untracked).
#
# Docker-FREE by construction (NEVER spins a container): it grades the exit-derivation via a
# PRE-WRITTEN fixture artifact through the harness's own `--selftest-exit-from-artifact` hook,
# which runs the REAL exit_from_artifact() code (not a copy). The end-to-end container proof
# is the post-release example rows; THIS gate is the unit guard on the exit-from-artifact
# PROPERTY. Legs: STATIC (grep the mechanism) + DYNAMIC (python3+bash: fixture exit_code=1 ->
# exit 1, 0 -> 0, unreadable -> fail-closed 1) + GITIGNORE (git check-ignore the .sim-*.log).
#
# CEFA_HARNESS_PATH / CEFA_ARTIFACT_PATH override the harness / artifact path (the selftest
# points HARNESS at a /tmp fixture re-introducing the docker-rc terminal, proving STATIC is
# not a rubber stamp). Degrades to NOT-VERIFIED (exit 75, never a false PASS -- OP-2) if
# python3/git is absent. Exit: 0 PASS, 1 FAIL, 75 NOT-VERIFIED.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
HARNESS="${CEFA_HARNESS_PATH:-$REPO_ROOT/quality/gates/docs-repro/container-rehearse.sh}"
ARTIFACT="${CEFA_ARTIFACT_PATH:-$REPO_ROOT/quality/reports/verifications/docs-repro/container-rehearse-exit-from-artifact.json}"
mkdir -p "$(dirname "$ARTIFACT")"
EXIT_NOT_VERIFIED=75

now_iso() { date -u +"%Y-%m-%dT%H:%M:%SZ"; }

PASSED=()
FAILED=()
SKIPPED=()

WORK="$(mktemp -d /tmp/cefa-XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

# ======================================================================= STATIC
if [[ ! -f "$HARNESS" ]]; then
    FAILED+=("STATIC: harness not found at $HARNESS")
else
    # (S1) exit_from_artifact() exists AND re-reads exit_code from the persisted artifact
    #      and exits with THAT value (grep the function body, first 14 lines after header).
    EFA_BODY="$(grep -A 14 '^exit_from_artifact() {' "$HARNESS")"
    if grep -q '^exit_from_artifact() {' "$HARNESS" \
       && grep -q 'exit_code' <<<"$EFA_BODY" \
       && grep -qE 'exit "\$code"' <<<"$EFA_BODY"; then
        PASSED+=("container-rehearse.sh defines exit_from_artifact() which re-reads exit_code from the persisted artifact and exits with THAT value")
    else
        FAILED+=("STATIC S1: exit_from_artifact() is missing or does not re-read exit_code from the artifact and exit with it")
    fi
    # (S2) the harness's TERMINAL exit is exit_from_artifact, NOT the raw docker rc.
    if grep -qE '^exit_from_artifact$' "$HARNESS" && ! grep -qE '^exit "\$EXIT_CODE"$' "$HARNESS"; then
        PASSED+=("the harness terminal exit is exit_from_artifact (the persisted exit_code), never exit \"\$EXIT_CODE\" (the raw docker/timeout rc) -- a rc=0 cannot mask an artifact exit_code=1")
    else
        FAILED+=("STATIC S2: the harness still terminates on 'exit \"\$EXIT_CODE\"' (docker rc) instead of exit_from_artifact")
    fi
    # (S3) the persisted exit_code is AUTHORITATIVE: congruence-graded, not the bare rc.
    if grep -q 'asserts_congruent' "$HARNESS" && grep -q 'authoritative' "$HARNESS"; then
        PASSED+=("the persisted exit_code is authoritative: a docker rc=0 with missing/uncongruent ASSERT-PASS lines is recorded exit_code=1 (asserts_congruent-graded), never a silent pass")
    else
        FAILED+=("STATIC S3: the harness does not compute an authoritative congruence-graded exit_code (missing asserts_congruent + authoritative)")
    fi
    # (S4) DRAIN-13 sim-readiness leg: the SIM_READY-fail branch surfaces NON-ZERO
    #      (write_fail_artifact + exit_from_artifact), never a masking `exit 0`.
    SIM_BRANCH="$(awk '/SIM_READY" -ne 1/{f=1} f{print} f&&/^fi$/{exit}' "$HARNESS")"
    if grep -q 'SIM_READY' "$HARNESS" \
       && grep -q 'curl' "$HARNESS" \
       && grep -q 'write_fail_artifact' <<<"$SIM_BRANCH" \
       && grep -q 'exit_from_artifact' <<<"$SIM_BRANCH" \
       && ! grep -qE '^[[:space:]]*exit 0[[:space:]]*$' <<<"$SIM_BRANCH"; then
        PASSED+=("the pre-docker-run readiness gate requires sim REACHABILITY (curl loop -> SIM_READY) and a sim-not-reachable failure surfaces NON-ZERO (write_fail_artifact + exit_from_artifact), not masked as exit 0 -- the DRAIN-13 sim-readiness leg")
    else
        FAILED+=("STATIC S4: the SIM_READY-fail branch does not surface non-zero (missing write_fail_artifact/exit_from_artifact, or still 'exit 0') -- a sim-not-reachable failure would be masked as success")
    fi
fi

# ====================================================================== DYNAMIC
DYN_OK=1
DYN_REASON=""
if ! command -v python3 >/dev/null 2>&1; then
    DYN_OK=0; DYN_REASON="python3 unavailable (needed to write/parse fixture artifacts)"
elif [[ ! -f "$HARNESS" ]]; then
    DYN_OK=0; DYN_REASON="harness not found"
fi

if [[ "$DYN_OK" -eq 1 ]]; then
    # D1: a persisted artifact with exit_code=1 (while a docker rc would be 0) -> harness exits 1.
    printf '{"row_id":"cefa-selftest","exit_code":1,"asserts_passed":[],"asserts_failed":["forced fail: docker rc=0 would disagree"]}\n' > "$WORK/fail.json"
    bash "$HARNESS" --selftest-exit-from-artifact "$WORK/fail.json" >/dev/null 2>&1
    RC1=$?
    # D2: a persisted artifact with exit_code=0 -> harness exits 0.
    printf '{"row_id":"cefa-selftest","exit_code":0,"asserts_passed":["x"],"asserts_failed":[]}\n' > "$WORK/pass.json"
    bash "$HARNESS" --selftest-exit-from-artifact "$WORK/pass.json" >/dev/null 2>&1
    RC0=$?
    # D3: an unreadable/missing artifact -> fail closed to exit 1 (never a silent 0).
    bash "$HARNESS" --selftest-exit-from-artifact "$WORK/does-not-exist.json" >/dev/null 2>&1
    RCX=$?
    if [[ "$RC1" -eq 1 && "$RC0" -eq 0 && "$RCX" -eq 1 ]]; then
        PASSED+=("runtime-proven docker-free: a persisted artifact exit_code=1 (while docker rc would be 0) makes the harness exit 1; exit_code=0 makes it exit 0; an unreadable artifact fails closed to exit 1 -- the harness exit is derived STRICTLY from the persisted exit_code")
    else
        FAILED+=("DYNAMIC: harness exit did not track the persisted exit_code (exit_code=1 -> rc=$RC1 [want 1]; exit_code=0 -> rc=$RC0 [want 0]; unreadable -> rc=$RCX [want 1])")
    fi
fi

# ==================================================================== GITIGNORE
GI_OK=1
GI_REASON=""
if ! command -v git >/dev/null 2>&1; then
    GI_OK=0; GI_REASON="git unavailable"
fi

if [[ "$GI_OK" -eq 1 ]]; then
    # DRAIN-14: a .sim-*.log under docs-repro/ must be git-ignored. check-ignore needs no
    # on-disk file -- it grades the path against the ignore rules. Also assert none slipped
    # into the index (a tracked .sim log would silently defeat the ignore rule).
    PROBE_REL="quality/reports/verifications/docs-repro/.sim-cefa-selftest-probe.log"
    TRACKED_SIM="$(git -C "$REPO_ROOT" ls-files 'quality/reports/verifications/docs-repro/.sim-*.log' 2>/dev/null)"
    if git -C "$REPO_ROOT" check-ignore -q "$PROBE_REL" && [[ -z "$TRACKED_SIM" ]]; then
        PASSED+=("git check-ignore confirms .sim-*.log under quality/reports/verifications/docs-repro/ is git-ignored (DRAIN-14) and none is tracked -- the ephemeral sim logs no longer leak as untracked")
    elif [[ -n "$TRACKED_SIM" ]]; then
        FAILED+=("GITIGNORE: a .sim-*.log is TRACKED in the index ($TRACKED_SIM) -- remove it with 'git rm --cached' so the ignore rule takes effect (DRAIN-14)")
    else
        FAILED+=("GITIGNORE: git check-ignore does NOT match $PROBE_REL -- the .sim-*.log gitignore pattern is missing or mis-scoped (DRAIN-14)")
    fi
fi

# ============================================================= verdict + artifact
STATUS_EXIT=0
if [[ ${#FAILED[@]} -gt 0 ]]; then
    STATUS_EXIT=1
elif [[ "$DYN_OK" -ne 1 || "$GI_OK" -ne 1 ]]; then
    # Static mechanism present but a load-bearing runtime leg could not run: NOT-VERIFIED,
    # not PASS (OP-2: a skipped runtime proof is never a green).
    STATUS_EXIT="$EXIT_NOT_VERIFIED"
    [[ "$DYN_OK" -ne 1 ]] && SKIPPED+=("DYNAMIC exit-derivation leg skipped ($DYN_REASON) -- NOT-VERIFIED per OP-2")
    [[ "$GI_OK" -ne 1 ]] && SKIPPED+=("GITIGNORE leg skipped ($GI_REASON) -- NOT-VERIFIED per OP-2")
fi

python3 - "$ARTIFACT" "$(now_iso)" "$STATUS_EXIT" "${PASSED[@]:-}" "--SEP--" "${FAILED[@]:-}" "--SEP--" "${SKIPPED[@]:-}" <<'PY'
import json, sys
artifact, ts, exit_code = sys.argv[1:4]
rest = sys.argv[4:]
i1 = rest.index("--SEP--")
i2 = rest.index("--SEP--", i1 + 1)
passed = [s for s in rest[:i1] if s]
failed = [s for s in rest[i1 + 1:i2] if s]
skipped = [s for s in rest[i2 + 1:] if s]
open(artifact, "w").write(json.dumps({
    "ts": ts,
    "row_id": "docs-repro/container-rehearse-exit-from-artifact",
    "exit_code": int(exit_code),
    "asserts_passed": passed,
    "asserts_failed": failed,
    "skipped": skipped,
}, indent=2) + "\n")
PY

if [[ "$STATUS_EXIT" == "1" ]]; then
    printf 'container-rehearse-exit-from-artifact FAILED:\n' >&2
    printf '  - %s\n' "${FAILED[@]}" >&2
elif [[ "$STATUS_EXIT" == "$EXIT_NOT_VERIFIED" ]]; then
    printf 'container-rehearse-exit-from-artifact NOT-VERIFIED (static passed, a runtime leg skipped):\n' >&2
    printf '  - %s\n' "${SKIPPED[@]:-}" >&2
fi
exit "$STATUS_EXIT"
