#!/usr/bin/env bash
# .claude/skills/reposix-quality-review/lib/dispatch_cold_reader.sh -- P61 SUBJ-02 Wave D.
#
# Cold-reader rubric dispatcher. Subprocess-invokes the doc-clarity-review
# global skill on README.md + docs/index.md with the cold-reader rubric prompt;
# parses the verdict; persists the canonical artifact via lib/persist_artifact.py.
#
# Path A end-to-end: invoked from a Claude session via SKILL.md (Task tool path).
# Path B fallback: if `claude /doc-clarity-review` is not available in PATH,
# this script writes a fallback artifact (score=0, verdict=CONFUSING) and exits 1.
#
# Anti-bloat: <=80 LOC.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
SKILL_DIR="$REPO_ROOT/.claude/skills/reposix-quality-review"
RUBRIC_ID="subjective/cold-reader-hero-clarity"
RUBRIC_PROMPT="$SKILL_DIR/rubrics/cold-reader-hero-clarity.md"
PERSIST="$SKILL_DIR/lib/persist_artifact.py"

if ! command -v claude >/dev/null 2>&1; then
    python3 "$PERSIST" \
        --rubric-id "$RUBRIC_ID" \
        --score 0 \
        --verdict CONFUSING \
        --rationale "claude CLI not in PATH; cannot invoke /doc-clarity-review subprocess. Run from a Claude session via SKILL.md (Path A)." \
        --evidence-files README.md docs/index.md \
        --dispatched-via "doc-clarity-review-unavailable" \
        --asserts-failed "claude CLI not in PATH"
    exit 1
fi

TMPLOG="$(mktemp /tmp/cold-reader-dispatch-XXXXXX.log)"
trap 'rm -f "$TMPLOG"' EXIT

if ! claude /doc-clarity-review --prompt "$RUBRIC_PROMPT" "$REPO_ROOT/README.md" "$REPO_ROOT/docs/index.md" 2>&1 | tee "$TMPLOG" >/dev/null; then
    python3 "$PERSIST" \
        --rubric-id "$RUBRIC_ID" \
        --score 0 \
        --verdict CONFUSING \
        --rationale "doc-clarity-review subprocess failed; see $TMPLOG" \
        --evidence-files README.md docs/index.md \
        --dispatched-via "doc-clarity-review-failed" \
        --asserts-failed "doc-clarity-review subprocess exited non-zero"
    exit 1
fi

VERDICT="$(grep -E '^Rate:' "$TMPLOG" | tail -1 | sed 's/^Rate: *//;s/ *$//')"
case "$VERDICT" in
    CLEAR)        SCORE=10 ;;
    "NEEDS WORK") SCORE=5  ;;
    CONFUSING)    SCORE=2  ;;
    *)            SCORE=0; VERDICT="CONFUSING" ;;
esac

RATIONALE="$(awk '/^Rationale:/,/^$/' "$TMPLOG" | tail -n +2 | head -10 | tr '\n' ' ' | sed 's/  */ /g')"
[ -z "$RATIONALE" ] && RATIONALE="(no rationale extracted from doc-clarity-review output)"

python3 "$PERSIST" \
    --rubric-id "$RUBRIC_ID" \
    --score "$SCORE" \
    --verdict "$VERDICT" \
    --rationale "$RATIONALE" \
    --evidence-files README.md docs/index.md \
    --dispatched-via "doc-clarity-review"

if [ "$SCORE" -ge 7 ]; then
    exit 0
elif [ "$SCORE" -ge 4 ]; then
    exit 2
else
    exit 1
fi
