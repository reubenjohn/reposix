#!/usr/bin/env bash
# quality/gates/docs-repro/container-congruence-earned.selftest.sh
#
# P124/SC1 (DRAIN-22). Selftest for container-congruence-earned.sh -- proves the
# gate is not a rubber stamp:
#   T1  the gate PASSES against the real (harvest-based) container-rehearse.sh.
#   T2  the gate FAILS when pointed (CRE_HARNESS_PATH) at a fixture harness that
#       RE-INTRODUCES the verbatim expected.asserts -> asserts_passed copy.
#   T3  the gate FAILS when pointed at a fixture harness that has NEITHER the
#       verbatim copy NOR the `^ASSERT-PASS: ` harvest path (missing harvest).
#   T4  the harvest transform itself: a real fixture (N ASSERT-PASS lines) EARNS
#       congruence; a no-op fixture (zero lines) does NOT.
#   T5  the gate FAILS when pointed at a fixture harness that HAS the harvest path
#       (grep + PREFIX) but LACKS the empty-harvest guard (`elif not harvested:`
#       -> congruent=False). This is the P124-code-review falsifiability case: a
#       harness that harvests but never forces a zero-line example to exit 1 would
#       reopen the F-K4b tautology (asserts_congruent/apply_pass_gates both no-op
#       True on an empty asserts_passed), and the gate must catch it statically.
#
# All fixtures live under /tmp (never the shared tree). Exit 0 iff all pass.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
GATE="$REPO_ROOT/quality/gates/docs-repro/container-congruence-earned.sh"
WORK="$(mktemp -d /tmp/cre-selftest-XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

fails=0
ok()   { echo "  ok   - $1"; }
bad()  { echo "  FAIL - $1"; fails=$((fails + 1)); }

# ---- T1: real repo harness -> PASS (exit 0).
if CRE_ARTIFACT_PATH="$WORK/t1.json" bash "$GATE" >/dev/null 2>&1; then
    ok "T1 gate PASSES against the real harvest-based container-rehearse.sh"
else
    bad "T1 gate did NOT pass against the real container-rehearse.sh (exit non-zero)"
fi

# ---- T2: fixture harness that RE-INTRODUCES the verbatim copy -> FAIL.
cat > "$WORK/harness-verbatim.sh" <<'EOF'
#!/usr/bin/env bash
# Fixture: the OLD tautological harness that copies expected.asserts verbatim.
EXPECTED_ASSERTS=$(python3 -c "print('[]')")
python3 - <<'PY'
data = {"asserts_passed": []}
expected_asserts = []
data["asserts_passed"].extend(str(a) for a in expected_asserts)
PY
grep '^ASSERT-PASS: ' /dev/null || true
PREFIX = "ASSERT-PASS: "
EOF
if CRE_HARNESS_PATH="$WORK/harness-verbatim.sh" CRE_ARTIFACT_PATH="$WORK/t2.json" \
        bash "$GATE" >/dev/null 2>&1; then
    bad "T2 gate PASSED a harness that re-introduces the verbatim-copy path (should FAIL)"
else
    ok "T2 gate FAILS a harness that re-introduces the verbatim expected.asserts copy"
fi

# ---- T3: fixture harness missing BOTH the copy and the harvest path -> FAIL.
cat > "$WORK/harness-noharvest.sh" <<'EOF'
#!/usr/bin/env bash
# Fixture: no verbatim copy, but also no ASSERT-PASS harvest path at all.
echo "this harness harvests nothing"
EOF
if CRE_HARNESS_PATH="$WORK/harness-noharvest.sh" CRE_ARTIFACT_PATH="$WORK/t3.json" \
        bash "$GATE" >/dev/null 2>&1; then
    bad "T3 gate PASSED a harness with no ASSERT-PASS harvest path (should FAIL)"
else
    ok "T3 gate FAILS a harness missing the '^ASSERT-PASS: ' harvest path"
fi

# ---- T4: harvest transform distinguishes real (earns) from no-op (does not).
python3 - "$REPO_ROOT" <<'PY'
import os, sys
repo_root = sys.argv[1]
sys.path.insert(0, os.path.join(repo_root, "quality", "runners"))
from _audit_field import asserts_congruent

PREFIX = "ASSERT-PASS: "

def harvest(text):
    out = []
    for line in text.splitlines():
        if line.startswith(PREFIX):
            t = line[len(PREFIX):].strip()
            if t:
                out.append(t)
    return out

def earned(expected, passed):
    if not passed:
        return False
    return asserts_congruent(expected, passed)[0]

expected = [
    "the run leaves a partial-clone working tree at /tmp/reposix-example-01",
    "the example pushes a commit and the simulator's audit log shows a helper_push_* row",
    "bash examples/01-shell-loop/run.sh exits 0",
]
real = (
    "chatter\n"
    "ASSERT-PASS: partial-clone working tree left at /tmp/reposix-example-01\n"
    "ASSERT-PASS: example pushed a commit; helper_push audit row written\n"
    "ASSERT-PASS: bash examples/01-shell-loop/run.sh completed and exits 0\n"
)
noop = ""  # exits 0, zero ASSERT-PASS lines

assert earned(expected, harvest(real)) is True, "real fixture must EARN congruence"
assert earned(expected, harvest(noop)) is False, "no-op fixture must NOT earn congruence"
print("T4-OK")
PY
if [[ $? -eq 0 ]]; then
    ok "T4 harvest transform: real fixture earns congruence, no-op does not"
else
    bad "T4 harvest transform failed to distinguish real vs no-op"
fi

# ---- T5: fixture with the harvest path PRESENT but the empty-harvest guard
#      ABSENT -> FAIL (the P124-code-review falsifiability case). NO verbatim
#      copy, HAS `grep '^ASSERT-PASS: '` + PREFIX, but NO `elif not harvested:`.
cat > "$WORK/harness-noguard.sh" <<'EOF'
#!/usr/bin/env bash
# Fixture: harvests ASSERT-PASS lines but NEVER forces a zero-line example to
# exit 1 -- the guard branch is missing, reopening the F-K4b tautology.
grep '^ASSERT-PASS: ' /dev/null || true
PREFIX = "ASSERT-PASS: "
# grading with NO `elif not harvested:` branch:
#   if not expected:
#       congruent = True
#   else:
#       congruent, unmatched = asserts_congruent(expected, harvested)
EOF
if CRE_HARNESS_PATH="$WORK/harness-noguard.sh" CRE_ARTIFACT_PATH="$WORK/t5.json" \
        bash "$GATE" >/dev/null 2>&1; then
    bad "T5 gate PASSED a harness missing the empty-harvest guard (should FAIL)"
else
    ok "T5 gate FAILS a harness with the harvest path but no empty-harvest guard"
fi

echo
if [[ "$fails" -eq 0 ]]; then
    echo "container-congruence-earned.selftest: ALL PASS"
    exit 0
fi
echo "container-congruence-earned.selftest: $fails FAILURE(S)"
exit 1
