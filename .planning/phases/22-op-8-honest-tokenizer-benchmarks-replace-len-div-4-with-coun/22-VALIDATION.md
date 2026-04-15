---
phase: 22
slug: op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-15
---

# Phase 22 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | pytest (Python) + manual benchmark verification |
| **Config file** | none — no pytest config yet (Wave 0 installs) |
| **Quick run command** | `python3 -m pytest scripts/ benchmarks/ -x -q 2>/dev/null || python3 scripts/bench_token_economy.py --help` |
| **Full suite command** | `python3 scripts/bench_token_economy.py --offline && cat benchmarks/RESULTS.md` |
| **Estimated runtime** | ~10 seconds (offline mode) |

---

## Sampling Rate

- **After every task commit:** Run `python3 scripts/bench_token_economy.py --help` (sanity check)
- **After every plan wave:** Run full suite command above
- **Before `/gsd-verify-work`:** Full suite must produce valid RESULTS.md
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 22-bench-01 | A | 1 | BENCH-01 | — | count_tokens replaces len/4 | unit | `grep -q "count_tokens" scripts/bench_token_economy.py` | ✅ | ⬜ pending |
| 22-cache-01 | A | 1 | BENCH-01 | — | Cache file written with sha256 key | unit | `python3 -c "import json; f=open('benchmarks/fixtures/reposix_session.tokens.json'); d=json.load(f); assert 'sha256' in d"` | ❌ W0 | ⬜ pending |
| 22-table-01 | B | 2 | BENCH-02 | — | Per-backend table in RESULTS.md | manual | `grep -q "github\|confluence" benchmarks/RESULTS.md` | ❌ W0 | ⬜ pending |
| 22-docs-01 | C | 3 | BENCH-04 | — | docs/why.md updated with real number | manual | `grep -qE "[0-9]+\.[0-9]+%" docs/why.md` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `benchmarks/fixtures/` directory created
- [ ] `anthropic` Python package available (`pip3 install anthropic`)
- [ ] `benchmarks/fixtures/github_issues.json` fixture created (raw GitHub REST JSON sample)
- [ ] `benchmarks/fixtures/confluence_pages.json` fixture created (raw Confluence REST JSON sample)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real token count API call | BENCH-01 | Requires ANTHROPIC_API_KEY | Run `ANTHROPIC_API_KEY=<key> python3 scripts/bench_token_economy.py` and verify no len/4 in output |
| Benchmark numbers honest | BENCH-04 | Requires human judgment | Compare new % in docs/why.md against script output; verify they match |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
