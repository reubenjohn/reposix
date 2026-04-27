#!/usr/bin/env python3
"""scripts/bench_token_economy.py -- migrated to quality/gates/perf/bench_token_economy.py per SIMPLIFY-11 (P59).

Shim preserves the old path for OP-5 reversibility; docs/benchmarks/token-economy.md
continues to document this entry point. P63 SIMPLIFY-12 may delete this shim.
"""
import subprocess
import sys
from pathlib import Path

target = Path(__file__).resolve().parent.parent / "quality" / "gates" / "perf" / "bench_token_economy.py"
sys.exit(subprocess.run([sys.executable, str(target), *sys.argv[1:]]).returncode)
