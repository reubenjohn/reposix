#!/usr/bin/env bash
# .claude/skills/reposix-quality-review/lib/dispatch_dvcs_cold_reader.sh -- P90 90-03 (D90-10 B2).
#
# DVCS cold-reader rubric dispatcher. Subprocess-invokes the doc-clarity-review
# global skill on the 3 DVCS docs (docs/concepts/dvcs-topology.md +
# docs/guides/dvcs-mirror-setup.md + docs/guides/troubleshooting.md, the
# latter scoped by its rubric prompt to the "DVCS push/pull issues" section)
# against a reader profile who has read only docs/index.md +
# docs/concepts/mental-model-in-60-seconds.md; parses the verdict; persists
# via lib/persist_artifact.py. Mirrors dispatch_cold_reader.sh's Path A/B
# shape: this is the runner-subprocess fallback (Path B); Path A drives via
# SKILL.md from a Claude session (Task tool).
#
# Row: subjective/dvcs-cold-reader (quality/catalogs/subjective-rubrics.json).
# Before this file existed, `--rubric subjective/dvcs-cold-reader` had no
# dispatch.sh case and fell through to the generic Path B stub — the
# actually-decorative subagent-graded row R2 § B/H2 identified. This wires
# real grading (kind stays subagent-graded; intent WAS real grading).
#
# Anti-bloat: <=90 LOC.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
SKILL_DIR="$REPO_ROOT/.claude/skills/reposix-quality-review"
RUBRIC_ID="subjective/dvcs-cold-reader"
PERSIST="$SKILL_DIR/lib/persist_artifact.py"

DOC1="$REPO_ROOT/docs/concepts/dvcs-topology.md"
DOC2="$REPO_ROOT/docs/guides/dvcs-mirror-setup.md"
DOC3="$REPO_ROOT/docs/guides/troubleshooting.md"
EVIDENCE_FILES=(
    "docs/concepts/dvcs-topology.md"
    "docs/guides/dvcs-mirror-setup.md"
    "docs/guides/troubleshooting.md:DVCS push/pull issues section"
)

if ! command -v claude >/dev/null 2>&1; then
    python3 "$PERSIST" \
        --rubric-id "$RUBRIC_ID" \
        --score 0 \
        --verdict CONFUSING \
        --rationale "claude CLI not in PATH; cannot invoke /doc-clarity-review subprocess. Run from a Claude session via SKILL.md (Path A)." \
        --evidence-files "${EVIDENCE_FILES[@]}" \
        --dispatched-via "doc-clarity-review-unavailable" \
        --asserts-failed "claude CLI not in PATH"
    exit 1
fi

RUBRIC_PROMPT="$(mktemp /tmp/dvcs-cold-reader-rubric-XXXXXX.md)"
TMPLOG="$(mktemp /tmp/dvcs-cold-reader-dispatch-XXXXXX.log)"
trap 'rm -f "$RUBRIC_PROMPT" "$TMPLOG"' EXIT

cat > "$RUBRIC_PROMPT" <<'RUBRIC_EOF'
You are reviewing one or more files in complete isolation. You have NOT seen
the codebase, the project history, or any other files. Do not follow any
links. Do not request additional context. Do not use any tools. Read only
what is provided.

Reader profile: has read only docs/index.md +
docs/concepts/mental-model-in-60-seconds.md; has never seen the DVCS docs
before; understands single-backend reposix but not DVCS (multi-remote)
topology; goal: install the mirror sync against a Confluence space without
escalating to a maintainer.

Judge the provided files against these criteria:
1. The three roles (SoT-holder, mirror-only consumer, round-tripper) are
   introduced before any of them is used in a sentence.
2. The mirror-lag refs explanation does not let the reader confuse "last
   synced" with "current SoT state".
3. The when-to-choose-which-pattern guidance gives a clear self-routing path.
4. The mirror-setup walk-through is runnable as-written end to end (Steps
   1-5 + cleanup) without consulting external docs beyond linked Atlassian
   webhook docs.
5. The cleanup procedure is complete enough to tear down without leaving
   orphan refs/secrets/webhooks.
6. The troubleshooting entries name the symptom in concrete stderr text and
   the recovery in concrete shell commands.
7. Zero plumbing-jargon leaks above Layer 3 (FUSE/kernel/partial-clone/
   promisor/stateless-connect/fast-import/protocol-v2).

Report the following sections:

## What I learned in 30 seconds

## Friction Points

## Unanswered Questions

## Verdict

In 2 sentences: would this reader complete the DVCS mirror-sync walk-through
without escalating to a maintainer?

Rate: CLEAR / NEEDS WORK / CONFUSING

Rationale: one paragraph explaining the rating.
RUBRIC_EOF

if ! claude /doc-clarity-review --prompt "$RUBRIC_PROMPT" "$DOC1" "$DOC2" "$DOC3" 2>&1 | tee "$TMPLOG" >/dev/null; then
    python3 "$PERSIST" \
        --rubric-id "$RUBRIC_ID" \
        --score 0 \
        --verdict CONFUSING \
        --rationale "doc-clarity-review subprocess failed; see $TMPLOG" \
        --evidence-files "${EVIDENCE_FILES[@]}" \
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
    --evidence-files "${EVIDENCE_FILES[@]}" \
    --dispatched-via "doc-clarity-review"

if [ "$SCORE" -ge 7 ]; then
    exit 0
elif [ "$SCORE" -ge 4 ]; then
    exit 2
else
    exit 1
fi
