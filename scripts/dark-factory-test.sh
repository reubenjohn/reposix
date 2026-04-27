#!/usr/bin/env bash
# scripts/dark-factory-test.sh -- migrated to quality/gates/agent-ux/dark-factory.sh per SIMPLIFY-07 (P59).
#
# This shim preserves the old path for OP-5 reversibility; CLAUDE.md "Local
# dev loop" continues documenting the old command. P63 SIMPLIFY-12 audit
# may delete this shim. Tracked in: P63 SIMPLIFY-12.
exec bash "$(dirname "$0")/../quality/gates/agent-ux/dark-factory.sh" "$@"
