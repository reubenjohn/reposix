#!/usr/bin/env bash
# quality/gates/agent-ux/rebase-recovery-reconciles.sh
#
# Catalog row: agent-ux/rebase-recovery-reconciles  (RBF-LR-03, Phase 105)
# Kind: shell-subprocess  (real git-remote-reposix helper vs live sim + transcript)
#
# CONTRACT (graded by the unbiased verifier subagent — see the catalog row's
# expected.asserts): drives the helper against a live sim through two SoT-drift
# scenarios and asserts the documented `git pull --rebase && git push` recovery
# reconciles (no `does not contain` / `fatal: error while running fast-import`),
# plus a NEGATIVE GUARD proving the pre-fix parentless emission RED-s.
#
# STATUS: SKELETON. This is the catalog-first placeholder (Phase 105 Lane 0).
# It exits 75 (sysexits.h EX_TEMPFAIL) so the runner grades the row
# NOT-VERIFIED — the honest catalog-first state — rather than FAIL, which would
# block pre-push before the gate is real (quality/PROTOCOL.md § "runner maps
# verifier exit codes": 75 → NOT-VERIFIED). Lane 2 ports repro2.sh into the real
# end-to-end gate + wires the transcript emission via
# quality/gates/agent-ux/lib/transcript.sh, then the UNBIASED verifier flips the
# row to PASS. Do NOT flip to PASS on the strength of this file.
#
# NOTE on the import-vs-stateless-connect path (PLAN §5 open question): the gate
# Lane 2 lands MUST force git's `import` capability path (the broken path the fix
# targets), and record whether the `stateless-connect` path on CI's modern git
# also breaks. git 2.25 selects `import` unaided; on modern git pin the helper's
# non-stateless path (executor to confirm the exact knob and document it here).
set -euo pipefail

echo "rebase-recovery-reconciles: SKELETON — real gate lands in Phase 105 Lane 2 (port of repro2.sh)." >&2
echo "Exiting 75 (EX_TEMPFAIL) so the runner grades this row NOT-VERIFIED (honest catalog-first state)," >&2
echo "not FAIL — the gate is not real yet; Lane 2 + the unbiased verifier flip it to PASS." >&2
exit 75
