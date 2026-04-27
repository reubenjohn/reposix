#!/usr/bin/env bash
# scripts/check-docs-site.sh -- thin shim post-P60 SIMPLIFY-08.
#
# The canonical home is quality/gates/docs-build/mkdocs-strict.sh. This
# shim survives for one merge cycle per OP-5 reversibility (any hidden
# caller surfaces). P63 SIMPLIFY-12 audits whether to delete or keep.
set -euo pipefail
exec bash "$(dirname "${BASH_SOURCE[0]}")/../quality/gates/docs-build/mkdocs-strict.sh" "$@"
