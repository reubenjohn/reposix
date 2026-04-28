#!/usr/bin/env bash
# .claude/skills/reposix-quality-review/dispatch.sh -- entry point for catalog row verifiers.
#
# Per quality/catalogs/subjective-rubrics.json, each row's verifier.script is this file.
# The runner invokes:
#     bash .claude/skills/reposix-quality-review/dispatch.sh --rubric <id>
#
# Modes: --rubric <id>, --all-stale, --force, (no args -> usage).
# Path A (full subagent dispatch via Task tool) lives in SKILL.md; this entry
# point is the runner-subprocess fallback that emits a stub artifact when Task
# is unavailable. Wave G (61-07) drives the Path A end-to-end run from inside
# a Claude session.
#
# Anti-bloat: <=120 LOC. Stdlib bash + python3.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
DISPATCH_CLI="$REPO_ROOT/.claude/skills/reposix-quality-review/lib/dispatch_cli.py"

usage() {
    cat <<EOF
Usage: bash .claude/skills/reposix-quality-review/dispatch.sh [--rubric <id>] [--all-stale] [--force]

Modes:
    --rubric <id>     dispatch one rubric (e.g. --rubric subjective/cold-reader-hero-clarity)
    --all-stale       dispatch every rubric whose row is_stale OR last_verified=null
    --force           dispatch every rubric regardless of freshness

Seed rubrics:
    subjective/cold-reader-hero-clarity   (pre-release; doc-clarity-review impl)
    subjective/install-positioning        (pre-release)
    subjective/headline-numbers-sanity    (weekly)

The skill persists JSON artifacts to quality/reports/verifications/subjective/<id>.json.
The next quality runner sweep re-grades the catalog row's status from the artifact.

Path A (full subagent dispatch via Task tool) lives in SKILL.md; invoke
'/reposix-quality-review --all-stale' from a Claude session to drive that flow.
This bash entry point is the runner-subprocess fallback (Path B / stub artifact).
EOF
}

if [[ $# -eq 0 ]] || [[ "${1:-}" == "--help" ]] || [[ "${1:-}" == "-h" ]]; then
    usage
    exit 0
fi

case "${1:-}" in
    --rubric)
        if [[ $# -lt 2 ]]; then
            echo "ERROR: --rubric requires an id" >&2
            usage >&2
            exit 2
        fi
        # Stub returns 1 for FAIL (runner records FAIL until Path A re-runs).
        if python3 "$DISPATCH_CLI" stub "$2"; then
            exit 0
        else
            rc=$?
            # rc=1 is the expected stub-FAIL signal; rc=2 is bad input.
            exit "$rc"
        fi
        ;;
    --all-stale|--force)
        echo "INFO: $1 mode dispatches via Path A (Task tool); invoke /reposix-quality-review $1 from a Claude session." >&2
        if [[ "$1" == "--force" ]]; then
            ids=$(python3 "$DISPATCH_CLI" list-all)
        else
            ids=$(python3 "$DISPATCH_CLI" list-stale)
        fi
        if [[ -z "$ids" ]]; then
            echo "no rubrics in scope" >&2
            exit 0
        fi
        echo "Stale rubrics:" >&2
        echo "$ids" >&2
        echo "Path B fallback: writing stub artifacts for each rubric in scope." >&2
        rc=0
        for id in $ids; do
            python3 "$DISPATCH_CLI" stub "$id" || rc=$?
        done
        exit "$rc"
        ;;
    *)
        echo "ERROR: unknown mode: $1" >&2
        usage >&2
        exit 2
        ;;
esac
