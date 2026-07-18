#!/usr/bin/env bash
# quality/gates/structure/container-rehearse-binary-provenance.sh
# Verifier for catalog row structure/container-rehearse-binary-provenance (P124 SC3, DRAIN-24).
#
# Backs the invariant: `.github/workflows/quality-post-release.yml` must reach
# `target/debug/reposix` on the runner via an EXPLICIT build/artifact-download step,
# not an unconfirmed implicit cache. The `post-release` cadence grades every
# `kind: container` docs-repro row (example-01/02/04/05); each host-mounts
# `target/debug/reposix` into an ubuntu:24.04 container (container-rehearse.sh
# `-v $REPO_ROOT/target:/workspace/target`). If the binary is absent on a cold
# runner those rows silently degrade to NOT-VERIFIED for a missing host binary
# (never a false green, but a silent coverage hole) -- DRAIN-24.
#
# Three asserts (mirroring the backing row's expected.asserts):
#   1. an explicit `cargo build -p reposix-cli` (or artifact-download) step exists
#      and PRECEDES the `run.py --cadence post-release` step, in the same job;
#   2. it is a hard dependency of the grading job (a prior sequential step -> a
#      build failure fails the job before any container row is graded);
#   3. an inline provenance comment names WHY the step exists.
#
# Static/structural check only -- parses the workflow with a stdlib-only structural
# step parser (NOT PyYAML: CI's setup-python has no PyYAML, and this row runs at
# pre-push/pre-pr). Docker-free, no cargo, no network, no sim. A teaching failure
# names the file + the missing step + the copy-paste fix.
#
# Exit: 0 -> PASS; 1 -> FAIL. Usage: [--row-id <id>]
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
cd "$REPO_ROOT"

ROW_ID="structure/container-rehearse-binary-provenance"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  ROW_ID="$2"
fi

WORKFLOW=".github/workflows/quality-post-release.yml"
ARTIFACT="${REPO_ROOT}/quality/reports/verifications/structure/container-rehearse-binary-provenance.json"
mkdir -p "$(dirname "$ARTIFACT")"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# asserts_passed token-map the backing row's 3 expected.asserts
# (F-K4b per-expected-assert congruence, _audit_field.asserts_congruent).
PASSED=(
  ".github/workflows/quality-post-release.yml contains an explicit \`cargo build -p reposix-cli\` (or artifact-download) step that runs BEFORE \`run.py --cadence post-release\`"
  "the step is a hard dependency of the job that grades container rows -- no container docs-repro row can silently degrade to NOT-VERIFIED for a missing host binary on a cold runner"
  "the provenance decision is documented inline in the workflow (a comment naming why the step exists)"
)

emit_artifact() {  # <exit_code> <status> <failed_json>
  local ec="$1" st="$2" failed="${3:-[]}"
  local pj
  pj="$(printf '%s\n' "${PASSED[@]:-}" | python3 -c 'import json,sys; print(json.dumps([l for l in sys.stdin.read().splitlines() if l]))')"
  cat > "$ARTIFACT" <<EOF
{
  "ts": "$TS", "row_id": "$ROW_ID", "exit_code": $ec, "status": "$st",
  "asserts_passed": ${pj},
  "asserts_failed": ${failed}
}
EOF
}
fail() {
  local desc="$1"
  echo "FAIL (${ROW_ID}): ${desc}" >&2
  PASSED=()  # a RED run proves no assert; emit an empty asserts_passed
  emit_artifact 1 FAIL "$(python3 -c 'import json,sys; print(json.dumps([sys.argv[1]]))' "$desc")"
  exit 1
}

if [[ ! -f "$WORKFLOW" ]]; then
  fail "workflow not found: $WORKFLOW -- the post-release cadence workflow is missing entirely"
fi

# ---- Structural parse (stdlib-only, docker-free) --------------------------------
# The python emits nothing + exits 0 on all-pass, or prints ONE teaching line to
# stderr + exits 1 on the first failing assert. Stdlib-only structural step parse
# (no PyYAML) + a raw-text inline-comment scan (YAML parsers strip `#` comments,
# so assert 3 MUST read raw text).
CHECK_OUT="$(python3 - "$WORKFLOW" <<'PY' 2>&1
import re, sys

path = sys.argv[1]
with open(path, encoding="utf-8") as f:
    lines = f.read().splitlines()

def die(msg):
    print(msg)
    sys.exit(1)

# 1. Locate the job-level `steps:` block.
steps_idx = steps_indent = None
for i, ln in enumerate(lines):
    m = re.match(r"^(\s*)steps:\s*$", ln)
    if m:
        steps_idx, steps_indent = i, len(m.group(1))
        break
if steps_idx is None:
    die(path + " has no `steps:` block -- cannot locate the grading job's steps.")

# 2. Determine the step-item indent (first `- ` line under steps:).
item_indent = None
for i in range(steps_idx + 1, len(lines)):
    m = re.match(r"^(\s*)-\s", lines[i])
    if m and len(m.group(1)) > steps_indent:
        item_indent = len(m.group(1))
        break
if item_indent is None:
    die(path + " has an empty `steps:` block.")

# 3. Collect step start lines (skip blanks + `#` comment lines) until the region
#    ends (a non-blank, non-comment line indented <= steps_indent, or EOF).
step_starts, region_end = [], len(lines)
step_re = re.compile(r"^\s{%d}-\s" % item_indent)
for i in range(steps_idx + 1, len(lines)):
    ln = lines[i]
    stripped = ln.strip()
    if stripped == "" or stripped.startswith("#"):
        continue
    indent = len(ln) - len(ln.lstrip())
    if indent <= steps_indent:
        region_end = i
        break
    if step_re.match(ln):
        step_starts.append(i)
if not step_starts:
    die(path + " has no step items under `steps:`.")

steps = []
for k, s in enumerate(step_starts):
    e = step_starts[k + 1] if k + 1 < len(step_starts) else region_end
    # Predicate-match against CODE only: strip `#` comment lines so a rationale
    # comment (which lands in the preceding step's raw slice) can never masquerade
    # as a real `run:` command -- e.g. the "cargo build -p reposix-cli first"
    # recovery hint in the build step's own leading comment.
    code_body = "\n".join(l for l in lines[s:e] if not l.strip().startswith("#"))
    steps.append((s, e, code_body))

def find_step(pred):
    for idx, (s, e, body) in enumerate(steps):
        if pred(body):
            return idx
    return -1

def is_build(b):
    if "cargo build -p reposix-cli" in b:
        return True
    # artifact-download alternative: a download-artifact action pulling the binary.
    if "download-artifact" in b and ("reposix" in b or "target" in b):
        return True
    return False

def is_runpy(b):
    return "run.py --cadence post-release" in b or (
        "run.py" in b and "--cadence" in b and "post-release" in b
    )

build_idx = find_step(is_build)
run_idx = find_step(is_runpy)

FIX = (
    "FIX: add BEFORE the 'Run post-release cadence gates' step:\n"
    "      # <inline comment naming WHY: container docs-repro rows host-mount\n"
    "      # target/debug/reposix; without this they degrade to NOT-VERIFIED on a\n"
    "      # cold runner -- DRAIN-24>\n"
    "      - name: Build reposix (host binary the container docs-repro rows mount)\n"
    "        run: cargo build -p reposix-cli"
)

# assert 1 (existence): explicit build/artifact step present.
if build_idx < 0:
    die(
        path + " has NO explicit `cargo build -p reposix-cli` (or artifact-download) "
        "step. The `post-release` cadence host-mounts `target/debug/reposix` into the "
        "container docs-repro rows (example-01/02/04/05); without an explicit build the "
        "binary is an unconfirmed implicit cache and those rows silently degrade to "
        "NOT-VERIFIED on a cold runner (DRAIN-24).\n" + FIX
    )

# assert 1b + assert 2 (ordering / hard-dependency): build precedes the grading step.
if run_idx < 0:
    die(
        path + " has no `run.py --cadence post-release` step -- cannot confirm the "
        "build precedes the grading step (did the workflow change shape?)."
    )
if build_idx >= run_idx:
    die(
        "the `cargo build -p reposix-cli` step (step index %d) does NOT precede the "
        "`run.py --cadence post-release` step (step index %d) -- the binary must be "
        "built BEFORE the gates that mount it, so a build failure fails the job before "
        "any container row is graded. FIX: move the build step above 'Run post-release "
        "cadence gates'." % (build_idx, run_idx)
    )

# assert 3 (inline provenance comment): scan the raw comment lines directly above
# the build step for a rationale token (YAML parsers strip comments -> raw text).
b_start = steps[build_idx][0]
comment_block = []
for j in range(b_start - 1, max(-1, b_start - 15), -1):
    s = lines[j].strip()
    if s == "":
        continue
    if s.startswith("#"):
        comment_block.append(s.lower())
        continue
    break  # first non-blank, non-comment line ends the block
tokens = ("drain-24", "provenance", "container docs-repro", "host-mount",
          "cold runner", "implicit cache", "target/debug")
if not any(tok in c for c in comment_block for tok in tokens):
    die(
        "the `cargo build -p reposix-cli` step has no inline provenance comment naming "
        "WHY it exists. FIX: add a `#` comment directly above the step referencing "
        "DRAIN-24 / that container docs-repro rows host-mount target/debug/reposix (so "
        "the provenance question is not re-opened)."
    )

sys.exit(0)
PY
)"
CHECK_RC=$?

if [[ $CHECK_RC -ne 0 ]]; then
  fail "${CHECK_OUT:-provenance structural check failed for $WORKFLOW}"
fi

emit_artifact 0 PASS
echo "PASS (${ROW_ID}): $WORKFLOW builds target/debug/reposix explicitly before the post-release gates (provenance-guaranteed for container docs-repro rows)." >&2
exit 0
