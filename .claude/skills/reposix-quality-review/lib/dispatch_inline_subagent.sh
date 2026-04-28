#!/usr/bin/env bash
# .claude/skills/reposix-quality-review/lib/dispatch_inline_subagent.sh -- P61 SUBJ-02 Wave E.
#
# Rubric dispatcher for inline-dispatch rubrics
# (subjective/install-positioning + subjective/headline-numbers-sanity).
# Path A (Task tool from Claude session) is preferred but only available
# from the SKILL.md flow; this entry point implements Path B (claude -p
# subprocess fallback). Wave G drives Path A end-to-end from a Claude session.
#
# Anti-bloat: <=120 LOC.

set -euo pipefail

RUBRIC_ID="${1:?--rubric required}"
REPO_ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
SKILL_DIR="$REPO_ROOT/.claude/skills/reposix-quality-review"
PERSIST="$SKILL_DIR/lib/persist_artifact.py"

case "$RUBRIC_ID" in
    "subjective/install-positioning")
        PROMPT="$SKILL_DIR/rubrics/install-positioning.md"
        SOURCES=("$REPO_ROOT/README.md" "$REPO_ROOT/docs/index.md")
        ;;
    "subjective/headline-numbers-sanity")
        PROMPT="$SKILL_DIR/rubrics/headline-numbers-sanity.md"
        SOURCES=("$REPO_ROOT/README.md" "$REPO_ROOT/docs/index.md")
        # Cap at <=20 source-of-truth files to keep the subagent prompt sane.
        count=0
        for f in "$REPO_ROOT/docs/benchmarks/"*.md "$REPO_ROOT/benchmarks/"*.md; do
            [ -f "$f" ] || continue
            SOURCES+=("$f")
            count=$((count + 1))
            [ "$count" -ge 18 ] && break
        done
        ;;
    *)
        echo "ERROR: unknown rubric: $RUBRIC_ID" >&2
        exit 1
        ;;
esac

if ! command -v claude >/dev/null 2>&1; then
    python3 "$PERSIST" \
        --rubric-id "$RUBRIC_ID" \
        --score 0 \
        --verdict CONFUSING \
        --rationale "claude CLI not in PATH; cannot dispatch inline subagent. Run via Path A from a Claude session." \
        --evidence-files "${SOURCES[@]##*/}" \
        --dispatched-via "claude-cli-not-in-path" \
        --asserts-failed "claude CLI not available"
    exit 1
fi

TMPLOG="$(mktemp /tmp/inline-dispatch-XXXXXX.log)"
trap 'rm -f "$TMPLOG"' EXIT

SUBAGENT_PROMPT="$(cat "$PROMPT")
Files to review (read each in full):
$(printf '%s\n' "${SOURCES[@]}")"

if ! claude -p "$SUBAGENT_PROMPT" 2>&1 | tee "$TMPLOG" >/dev/null; then
    python3 "$PERSIST" \
        --rubric-id "$RUBRIC_ID" \
        --score 0 \
        --verdict CONFUSING \
        --rationale "claude -p subprocess failed; see $TMPLOG" \
        --evidence-files "${SOURCES[@]##*/}" \
        --dispatched-via "claude-p-failed" \
        --asserts-failed "claude -p subprocess exited non-zero"
    exit 1
fi

SCORE="$(grep -E '^Rate: [0-9]+' "$TMPLOG" | tail -1 | sed 's/^Rate: *//;s/ *$//')"
VERDICT="$(grep -E '^Verdict: ' "$TMPLOG" | tail -1 | sed 's/^Verdict: *//;s/ *$//')"
[ -z "$SCORE" ] && SCORE=0
[ -z "$VERDICT" ] && VERDICT="CONFUSING"

RATIONALE="$(awk '/^Rationale:/,/^$/' "$TMPLOG" | tail -n +2 | head -10 | tr '\n' ' ' | sed 's/  */ /g')"
[ -z "$RATIONALE" ] && RATIONALE="(no rationale extracted from claude -p output)"

python3 "$PERSIST" \
    --rubric-id "$RUBRIC_ID" \
    --score "$SCORE" \
    --verdict "$VERDICT" \
    --rationale "$RATIONALE" \
    --evidence-files "${SOURCES[@]##*/}" \
    --dispatched-via "claude-p-inline"

if [ "$SCORE" -ge 7 ]; then
    exit 0
elif [ "$SCORE" -ge 4 ]; then
    exit 2
else
    exit 1
fi
