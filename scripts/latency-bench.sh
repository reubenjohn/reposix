#!/usr/bin/env bash
# scripts/latency-bench.sh -- migrated to quality/gates/perf/latency-bench.sh per SIMPLIFY-11 (P59).
#
# Shim preserves the old path for OP-5 reversibility; CI workflows + docs
# continue invoking this command. P63 SIMPLIFY-12 may delete this shim.
exec bash "$(dirname "$0")/../quality/gates/perf/latency-bench.sh" "$@"
