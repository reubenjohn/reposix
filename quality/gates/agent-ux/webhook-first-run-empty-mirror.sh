#!/usr/bin/env bash
# CATALOG ROW: agent-ux/webhook-first-run-empty-mirror
# CADENCE: pre-pr (~2s wall time)
# INVARIANT: First-run handling per Q4.3:
#   (4.3.a) fresh-but-readme mirror (gh repo create --add-readme):
#     workflow's "if git show-ref --verify --quiet
#     refs/remotes/mirror/main; then lease-push" branch fires;
#     lease push succeeds; mirror's main advances.
#   (4.3.b) truly-empty mirror (gh repo create, no --add-readme):
#     plain-push branch fires; main is created on mirror.
#   Both fixtures are file:// bare repos.
#
# Status until P84-01 T03: FAIL (stub). T03 replaces with the full
# ~80-line two-sub-fixture harness.
set -euo pipefail
echo "FAIL: T03 not yet shipped (first-run handling harness)"
exit 1
