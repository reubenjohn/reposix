# Phase 115 — Validation Strategy (Nyquist)

**Phase:** 115 Live MCP Benchmark Re-Measurement · **Requirement:** BENCH-01
**Nature:** measurement/documentation phase (no new source code). "Tests" here = the
quality-catalog `WAIVED`→`BOUND` lifecycle + mechanical file/ledger checks, NOT
unit/integration tests. Each signal below is something a verifier can run and read a
pass/fail from — no human eyeballing required beyond the one A1 ruling.

> Framework: none needed (`anthropic` already pinned in `requirements-bench.txt`; MCP
> client is Claude Code's built-in `mcpServers`). Quick substrate check:
> `bash scripts/preflight-real-backends.sh` (exit 0). Full regen suite:
> `bash quality/gates/perf/latency-bench.sh && python3 quality/gates/perf/bench_token_economy.py --offline`.

---

## A1 gate (precondition — must hold before ANY session spend)

| Signal | Mechanical check | Pass condition |
|--------|------------------|----------------|
| MANAGER ruled the session unit before spend | Ledger header records the ruling; the FIRST ledger data row's timestamp is strictly AFTER the ruling was recorded | ruling present in `benchmarks/bench-session-ledger.md` header; no data row predates it |
| Substrate reachable before spend | `bash scripts/preflight-real-backends.sh; echo $?` | exit `0` (all 3 sanctioned targets PASS) |
| `ANTHROPIC_API_KEY` confirmed (existing subscription) | presence check at gate; else escalation recorded | key present, OR a MANAGER escalation note exists in the SUMMARY |

---

## SC1 — Fresh live measurements exist for every one of the 8 waived rows

**Latency rows 1/2/4/6/7 (27ms cold init, 8ms cached read):**
| Signal | Mechanical check | Pass condition |
|--------|------------------|----------------|
| Fresh timestamp (not the stale 2026-04-27 snapshot) | `grep -m1 last_measured_at docs/benchmarks/latency.md \| grep -oE '2026-[0-9]{2}-[0-9]{2}'` | date `> 2026-07-14` |
| Real re-run, not hand-paste | `git diff --stat` shows docs/benchmarks/latency.md changed after a `latency-bench.sh` run | file modified; contains sim + ≥1 real-backend column |
| Cold-init + cached-read rows present | `grep -iE 'cold.?init' docs/benchmarks/latency.md && grep -iE 'read' docs/benchmarks/latency.md` | both match |
| 24-vs-27 resolved to ONE figure | SUMMARY records the single authoritative cold-init number the harness produced | one number stated; discrepancy called out |

**Token rows 3/5/8 (89.1% reduction):**
| Signal | Mechanical check | Pass condition |
|--------|------------------|----------------|
| Fixtures actually replaced with live captures | `git diff --stat` on `benchmarks/fixtures/mcp_jira_catalog.json` + `reposix_session.txt` | both changed this phase |
| reposix fixture is honest git-native, not FUSE-era | `grep -E '/mnt/\|scripts/demo\.sh' benchmarks/fixtures/reposix_session.txt` | ZERO matches |
| Doc regenerated from real fixtures, cache-stable | `python3 quality/gates/perf/bench_token_economy.py --offline` | exit 0, doc unchanged on rerun |

---

## SC2 — ≤50-session spend tracked; escalate past 50

| Signal | Mechanical check | Pass condition |
|--------|------------------|----------------|
| Ledger exists as a first-class artifact | `test -f benchmarks/bench-session-ledger.md` | present |
| Every session has a row (unit per ruling) | count data rows; each has timestamp/backend/arm/unit/running_total/artifact | no gaps; every session row complete |
| No backfill / batching | ledger timestamps are strictly monotonic (Pitfall 2) | monotonic, no out-of-order or duplicated stamps |
| Ceiling respected | `tail -1 benchmarks/bench-session-ledger.md \| awk -F'\|' '{v=$(NF-2); gsub(/ /,"",v); exit (v+0>50)}'; echo $?` (parses the final data row's `running_total` column and exits non-zero if it exceeds 50) | exit `0` (final running_total numerically ≤ 50), OR a MANAGER escalation note is recorded BEFORE the row that would exceed 50 |
| Escalation is out-of-band, not absorbed | if 50 would be exceeded, SUMMARY shows escalation, not "did more sessions" | escalation recorded; no silent >50 |

---

## SC3 — Figures + methodology in a form P118/DOCS-07 + DOCS-05 consume DIRECTLY

| Signal | Mechanical check | Pass condition |
|--------|------------------|----------------|
| Provenance lie removed at source | `grep -E 'scripts/demo\.sh\|modeled on' docs/benchmarks/token-economy.md benchmarks/fixtures/README.md` | ZERO matches |
| Real capture named | `grep -iE 'captured .* (live\|mcp)' docs/benchmarks/token-economy.md` | ≥1 match (server + date + task) |
| Methodology traceable | methodology note names: MCP server + GA status + real content read + exact task | all 4 present |
| Authoritative figures committed as markdown | `docs/benchmarks/{latency,token-economy}.md` hold the 3 fresh numbers | P118/DOCS-05 can `grep` each figure from committed docs (not transcripts) |

---

## SC4 — Documented un-waive path for the perf rows

| Signal | Mechanical check | Pass condition |
|--------|------------------|----------------|
| Both perf-targets rows named | `grep -q 'perf/token-economy-bench' … && grep -q 'headline-numbers-cross-check' …` (115-UNWAIVE-PATH.md) | both present |
| Dangling verifier flagged | `grep -i 'absent' 115-UNWAIVE-PATH.md` (headline-numbers-cross-check.py confirmed absent) | present |
| Exact assertion target named | path names the script/line each row needs + which fresh figure it binds | present for both perf rows + the 8 doc-alignment rows |
| Deferred-scope honored | `git diff --name-only` for the phase touches NO `quality/gates/perf/*.py` or `*.sh` | zero gate-file edits (Pitfall 4) |

---

## Sampling rate (from 115-RESEARCH.md § Validation Architecture)

- **Per session:** append one ledger row immediately after each live-MCP session — never batch.
- **Per track completion:** regenerate the matching `docs/benchmarks/*.md` and `git diff` against the prior committed version.
- **Phase gate:** both results docs regenerated + fresh timestamps + ledger `running_total ≤ 50` + 115-UNWAIVE-PATH.md complete, BEFORE `/gsd-verify-work`.

## Wave-0 gaps closed by this phase

- [x] Session-spend ledger — created in Task 3 (first commit, empty schema) before any spend.
- [ ] `quality/gates/perf/headline-numbers-cross-check.py` — confirmed ABSENT; NOT written this phase; its absence is NAMED in 115-UNWAIVE-PATH.md so a future phase doesn't rediscover it.
- [x] Live-MCP transcript capture — first done in Task 4 (manual capture via the MCP client; no committed harness previously existed).
