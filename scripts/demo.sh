#!/usr/bin/env bash
# scripts/demo.sh — backwards-compatible shim. The demo lives at
# scripts/demos/full.sh as of Phase 8-A; this file exists so existing
# "bash scripts/demo.sh" references in docs, README, and user muscle
# memory keep working.
exec bash "$(dirname "$0")/demos/full.sh" "$@"
