#!/usr/bin/env bash
# Smoke-test: assert the docs-alignment extractor prompt still contains
# the "Retirement vs implementation-gap" section added in P67 (W3).
#
# This guards against silent reverts of the transport-vs-feature
# heuristic that the P65 backfill audit revealed was missing — without
# it, extractor subagents propose retirement for rows that should be
# IMPL_GAP. Failure mode the section closes is documented at
# `quality/reports/doc-alignment/backfill-20260428T085523Z/RETIRE-AUDIT.md`.
#
# A proper regression test that re-runs `plan-refresh` against a doc
# with FUSE-era prose and asserts no new RETIRE_PROPOSED proposals
# requires subagent dispatch (Task tool) — that's the W3 TODO in
# `.planning/HANDOVER-v0.12.1.md`. This shell smoke test is the cheap
# safety net until that lands.
#
# Exit 0 if both prompts contain the required marker. Exit 1 otherwise.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

EXTRACTOR=".claude/skills/reposix-quality-doc-alignment/prompts/extractor.md"
GRADER=".claude/skills/reposix-quality-doc-alignment/prompts/grader.md"

fail=0

if ! grep -qF "## Retirement vs implementation-gap" "$EXTRACTOR"; then
  echo "FAIL: $EXTRACTOR missing '## Retirement vs implementation-gap' section." >&2
  fail=1
fi

if ! grep -qF "transport-vs-feature heuristic" "$GRADER"; then
  echo "FAIL: $GRADER missing transport-vs-feature cross-reference." >&2
  fail=1
fi

if ! grep -qF "IMPL_GAP:" "$EXTRACTOR"; then
  echo "FAIL: $EXTRACTOR missing IMPL_GAP rationale-prefix convention." >&2
  fail=1
fi

if ! grep -qF "DOC_DRIFT:" "$EXTRACTOR"; then
  echo "FAIL: $EXTRACTOR missing DOC_DRIFT rationale-prefix convention." >&2
  fail=1
fi

if [[ $fail -ne 0 ]]; then
  exit 1
fi

echo "OK: docs-alignment extractor + grader prompts contain the P67 retirement-vs-implementation-gap heuristic."
