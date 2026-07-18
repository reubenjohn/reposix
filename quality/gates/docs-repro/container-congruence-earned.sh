#!/usr/bin/env bash
# quality/gates/docs-repro/container-congruence-earned.sh
#
# P124/SC1 (DRAIN-22, closes F-K4b). Meta-check that container-row congruence is
# EARNED, not emitted: it proves (1) container-rehearse.sh no longer copies a
# row's expected.asserts verbatim into asserts_passed, and (2) the harvest+
# congruence logic distinguishes a real example (one `ASSERT-PASS:` line per
# expected.assert) from a no-op `exit 0` script (zero lines) -- the no-op earns
# NO congruence while the real one does.
#
# Two legs, BOTH docker-free (this gate deliberately needs no container: the
# end-to-end container proof is the post-release rows example-01/02/04/05; this
# gate is the unit-level guard on the harvest+congruence PROPERTY):
#   STATIC leg  -- greps container-rehearse.sh: the verbatim-copy path is GONE
#                  and the `^ASSERT-PASS: ` harvest path is PRESENT.
#   LOGIC  leg  -- runs the identical harvest transform over two fixture stdouts
#                  (a real one and a no-op one) and asserts earned==True/False.
#
# The LOGIC leg re-implements the harness's harvest one-liner; the STATIC leg
# asserts the harness contains that IDENTICAL pattern (PREFIX + grep), so the
# two cannot silently drift.
#
# CRE_HARNESS_PATH overrides the harness path (the selftest points it at a
# throwaway /tmp fixture that re-introduces the verbatim copy, to prove the
# STATIC leg is not a rubber stamp).
#
# Exit: 0 PASS, 1 FAIL. (No NOT-VERIFIED path -- both legs always run.)

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
HARNESS="${CRE_HARNESS_PATH:-$REPO_ROOT/quality/gates/docs-repro/container-rehearse.sh}"
# CRE_ARTIFACT_PATH lets the selftest's negative runs write a throwaway artifact
# instead of clobbering the canonical committed-report path.
ARTIFACT="${CRE_ARTIFACT_PATH:-$REPO_ROOT/quality/reports/verifications/docs-repro/container-congruence-earned.json}"
mkdir -p "$(dirname "$ARTIFACT")"

now_iso() { date -u +"%Y-%m-%dT%H:%M:%SZ"; }

PASSED=()
FAILED=()

# ---- STATIC leg: the verbatim-copy path is gone; the harvest path is present.
if [[ ! -f "$HARNESS" ]]; then
    FAILED+=("STATIC: harness not found at $HARNESS")
else
    # (a) verbatim-copy ABSENT. The old tautology resolved row.expected.asserts
    #     into a python `expected_asserts` var and did `asserts_passed.extend(...
    #     expected_asserts)`. Either token surviving is a regression.
    if grep -Eq 'expected_asserts|EXPECTED_ASSERTS' "$HARNESS"; then
        FAILED+=("STATIC: container-rehearse.sh still references expected_asserts (the verbatim-copy path must be deleted)")
    elif grep -Eq 'asserts_passed.*extend.*expected' "$HARNESS"; then
        FAILED+=("STATIC: container-rehearse.sh still extends asserts_passed with expected.asserts (verbatim-copy path)")
    else
        PASSED+=("container-rehearse.sh contains no verbatim-copy code path: grep proves absence of copying row.expected.asserts into asserts_passed")
    fi
    # (b) harvest path PRESENT: the exact grep pattern + the prefix constant.
    if grep -q "grep '^ASSERT-PASS: '" "$HARNESS" && grep -q 'PREFIX = "ASSERT-PASS: "' "$HARNESS"; then
        : # harvest path present; congruence is earned per-step
    else
        FAILED+=("STATIC: container-rehearse.sh is missing the '^ASSERT-PASS: ' harvest path (grep pattern + PREFIX constant)")
    fi
fi

# ---- LOGIC leg: harvest transform distinguishes real from no-op.
FIX_DIR="$(mktemp -d /tmp/container-congruence-earned-XXXXXX)"
trap 'rm -rf "$FIX_DIR"' EXIT

# Real fixture stdout: chatter + one ASSERT-PASS line per stub expected.assert.
cat > "$FIX_DIR/real.stdout" <<'EOF'
triaging: issues/1.md
ASSERT-PASS: partial-clone working tree left at /tmp/reposix-example-01
some diagnostic chatter that is not an assertion
ASSERT-PASS: example pushed a commit; the helper writes a helper_push audit row
ASSERT-PASS: bash examples/01-shell-loop/run.sh completed and exits 0
Done.
EOF

# No-op fixture stdout: exits 0, prints ZERO ASSERT-PASS lines.
cat > "$FIX_DIR/noop.stdout" <<'EOF'
EOF

LOGIC_OUT="$FIX_DIR/logic.out"
python3 - "$REPO_ROOT" "$FIX_DIR/real.stdout" "$FIX_DIR/noop.stdout" > "$LOGIC_OUT" 2>&1 <<'PY'
import os, sys
repo_root, real_path, noop_path = sys.argv[1:4]
# Import the REAL congruence primitives from the runner.
sys.path.insert(0, os.path.join(repo_root, "quality", "runners"))
from _audit_field import asserts_congruent  # noqa: E402

PREFIX = "ASSERT-PASS: "

def harvest(text):
    """Byte-identical to container-rehearse.sh's harvest: keep only lines that
    start with the prefix, strip it, drop blanks."""
    out = []
    for line in text.splitlines():
        line = line.rstrip("\n")
        if line.startswith(PREFIX):
            t = line[len(PREFIX):].strip()
            if t:
                out.append(t)
    return out

def earned(expected, passed):
    """Congruence is EARNED only when at least one line was harvested AND every
    expected.assert is covered. (asserts_congruent() no-ops to True on an EMPTY
    passed list -- a backward-compat quirk for the 91 mechanical rows -- so the
    non-empty guard is what closes the zero-line tautology at this layer.)"""
    if not passed:
        return False
    ok, _ = asserts_congruent(expected, passed)
    return ok

expected = [
    "the run leaves a partial-clone working tree at /tmp/reposix-example-01",
    "the example pushes a commit and the simulator's audit log shows a helper_push_* row",
    "bash examples/01-shell-loop/run.sh exits 0",
]

real_passed = harvest(open(real_path, encoding="utf-8").read())
noop_passed = harvest(open(noop_path, encoding="utf-8").read())

real_earned = earned(expected, real_passed)
noop_earned = earned(expected, noop_passed)

# real must EARN congruence; no-op must NOT.
print(f"real_harvested={len(real_passed)} real_earned={real_earned}")
print(f"noop_harvested={len(noop_passed)} noop_earned={noop_earned}")
if real_earned and not noop_earned:
    print("LOGIC_OK")
    sys.exit(0)
sys.exit(1)
PY
LOGIC_RC=$?

if [[ "$LOGIC_RC" -eq 0 ]]; then
    PASSED+=("a fixture example exiting 0 with ZERO ASSERT-PASS lines yields asserts_passed that does NOT cover the expected.asserts, so earned congruence is False")
    PASSED+=("a fixture example printing one ASSERT-PASS line per expected.assert yields a congruent artifact")
else
    FAILED+=("LOGIC: harvest transform failed to distinguish real vs no-op fixture -- $(tr '\n' ';' < "$LOGIC_OUT")")
fi

# ---- Verdict + artifact.
EXIT_CODE=0
[[ ${#FAILED[@]} -gt 0 ]] && EXIT_CODE=1

python3 - "$ARTIFACT" "$(now_iso)" "$EXIT_CODE" "${PASSED[@]:-}" "--SEP--" "${FAILED[@]:-}" <<'PY'
import json, sys
artifact, ts, exit_code = sys.argv[1:4]
rest = sys.argv[4:]
sep = rest.index("--SEP--")
passed = [s for s in rest[:sep] if s]
failed = [s for s in rest[sep + 1:] if s]
open(artifact, "w").write(json.dumps({
    "ts": ts,
    "row_id": "docs-repro/container-congruence-earned",
    "exit_code": int(exit_code),
    "asserts_passed": passed,
    "asserts_failed": failed,
}, indent=2) + "\n")
PY

if [[ "$EXIT_CODE" -ne 0 ]]; then
    printf 'container-congruence-earned FAILED:\n' >&2
    printf '  - %s\n' "${FAILED[@]}" >&2
fi
exit "$EXIT_CODE"
