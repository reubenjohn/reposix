# Phase 115: Live MCP Benchmark Re-Measurement - Research

**Researched:** 2026-07-15
**Domain:** Benchmark re-measurement (token-economy + latency), live MCP protocol sessions, quality-catalog waiver lifecycle
**Confidence:** MEDIUM — the WHAT (8 rows, existing scripts, real-backend reachability) is HIGH/VERIFIED; the HOW (exact "session" unit, capture-artifact schema) is LOW/ASSUMED and needs owner confirmation before execution.

## Summary

BENCH-01 must produce fresh, live-measured figures for exactly **8** `agent-ux` catalog
rows in `quality/catalogs/doc-alignment.json`, all carrying `waiver.until:
"2026-08-15T00:00:00Z"` `[VERIFIED: quality/catalogs/doc-alignment.json]`. These 8 rows
resolve to only **3 distinct hero numbers** (27ms/24ms cold init, 8ms cached read, 89.1%
token reduction) bound across `docs/index.md`, `README.md`, and one row
(`docs/why/cold-init-24ms-sim` at `docs/concepts/mental-model-in-60-seconds.md:21`) that
is a near-duplicate of the 27ms figure but is NOT one of the 8 (its own waiver, if any,
carries a different date — confirmed by exact grep-count of `"2026-08-15"` == 8
occurrences, all accounted for below).

The existing measurement infrastructure is real but split into two independently-
reproducible tracks that this phase must NOT conflate:

1. **Latency track** (5 of 8 rows) — `quality/gates/perf/latency-bench.sh` +
   `latency-bench/{sim,github,confluence,jira}.sh` already time a real golden path
   against the sim AND (when creds are present) the 3 sanctioned real backends
   `[VERIFIED: quality/gates/perf/latency-bench/github.sh:1-60]`. This script is cheap,
   fast (~15s sim-only), does NOT consume the 50-session ceiling, and the CI job
   `bench-latency-v09` already runs it with real secrets when configured
   `[VERIFIED: .github/workflows/bench-latency-cron.yml:46-52]`. The gap is NOT
   measurement capability — it's that the script never *asserts* the specific hero
   numbers (27ms/8ms), only a soft 500ms WARN threshold, so the catalog rows are
   `MISSING_TEST` rather than `BOUND` `[VERIFIED: doc-alignment.json rationale fields]`.
2. **Token-economy track** (3 of 8 rows) — `quality/gates/perf/bench_token_economy.py`
   computes a real Anthropic `count_tokens` comparison but against a **synthesized**
   MCP fixture (`benchmarks/fixtures/mcp_jira_catalog.json`, modeled on the public
   Atlassian Forge surface, "not against a live MCP server")
   `[VERIFIED: docs/concepts/reposix-vs-mcp-and-sdks.md:23-32]`, and against a reposix
   session fixture whose provenance claim is disputed — it is NOT actually the output of
   `scripts/demo.sh` (that script doesn't exist) and depicts the deleted FUSE
   architecture `[CITED: .planning/milestones/audits/2026-07-12-reality-check.md L253-259,
   LAUNCH-BLOCKER finding]`. This is the track that genuinely needs a **live MCP
   session** — a real agent, connected to a real, GA MCP server, doing the task against
   real backend content.

**The blocking condition the project itself documented no longer holds.** The MCP
comparison note explicitly deferred real-MCP numbers "once the upstream Atlassian MCP
server is GA and stable enough to bench against"
`[VERIFIED: docs/concepts/reposix-vs-mcp-and-sdks.md:30-32]`. As of this research date,
Atlassian's official remote MCP server (`atlassian-mcp-server`, Cloudflare-hosted, OAuth
2.1) reached General Availability and GitHub's official remote MCP server
(`https://api.githubcopilot.com/mcp/`) has been GA since September 2025
`[CITED: web search, MEDIUM confidence — multiple independent sources, see Sources]`.
This directly unblocks the live-MCP methodology this phase needs — it is technically
executable today, not blocked on an upstream dependency.

**Primary recommendation:** Split BENCH-01 into two independently-schedulable fan-out
tracks under `Execution mode: top-level` — (A) re-run the existing sim + real-backend
latency harness to produce a fresh, timestamped `docs/benchmarks/latency.md`, no
MCP session required, zero budget consumed against the 50-session ceiling; (B) run a
small, fixed number of real, budget-tracked live-MCP sessions (Atlassian remote MCP
server against the sanctioned Confluence "TokenWorld" / JIRA "KAN"/`TEST` targets, plus
optionally GitHub's remote MCP server against `reubenjohn/reposix`) executing the SAME
"read 3 issues, edit 1, push" task, capturing real transcripts through the existing
`count_tokens` cache-backed harness. Record the ≤50-session spend ledger as a
first-class committed artifact from session #1, not retroactively.

## User Constraints (from ROADMAP.md / REQUIREMENTS.md / handover docs — no CONTEXT.md exists for this phase; discuss step is skipped per `.planning/config.json` `skip_discuss: true`)

> No `.planning/phases/115-live-mcp-benchmark-re-measurement/*-CONTEXT.md` exists
> (`has_context: false`). The following are LOCKED, owner/manager-set constraints
> carried verbatim from `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`, and
> `.planning/SESSION-HANDOVER.md` — treat with the same authority CONTEXT.md decisions
> would carry.

### Locked Decisions
- **Spend ceiling: ≤50 benchmark sessions on the EXISTING subscription.** NO new API
  spend. Escalate to the MANAGER only past 50 sessions
  `[VERIFIED: .planning/REQUIREMENTS.md:142-147, ROADMAP.md:47-49, SESSION-HANDOVER.md:238-243]`.
- **Hard waiver-expiry deadline: 2026-08-15.** Schedule this phase EARLY in the
  milestone (it has no dependency on Phase 114 and is explicitly parallel-safe)
  `[VERIFIED: ROADMAP.md:100 "Depends on: Nothing — schedule EARLY, parallel-safe with
  Phase 114"]`.
- **`Execution mode: top-level`** — fan-out live benchmark sessions → gather results →
  interpret against the 8 waived rows. This is NOT a write-code-test-commit shape;
  `gsd-executor` lacks the tooling for this shape per `.planning/CLAUDE.md`
  `[VERIFIED: ROADMAP.md:108]`.
- **Output must be directly consumable** by Phase 118 (DOCS-07, depends explicitly on
  Phase 115) and by DOCS-05 (Phase 117 — runs independently of Phase 115 but should
  prefer the re-measured figure if available in time; DOCS-05's own text says "Relabel
  honestly ... even before BENCH-01 re-measures," i.e. DOCS-05 is NOT blocked on this
  phase, only enriched by it) `[VERIFIED: REQUIREMENTS.md:93-98, 319; ROADMAP.md:136]`.
- **Success criterion 4 asks for a "documented path to un-waive," not implementation of
  the un-waive itself.** Wiring the actual assertion into
  `bench_token_economy.py`/`latency-bench.sh`/the (currently absent)
  `headline-numbers-cross-check.py` is FIX_IMPL_THEN_BIND work for a future code phase,
  not P115's deliverable `[VERIFIED: ROADMAP.md:106]`.
- **Escalation past 50 sessions is explicitly Out of Scope** for this milestone's work
  product — it is an owner decision point, not something P115 can absorb by just doing
  more work `[VERIFIED: REQUIREMENTS.md:294-295]`.

### Claude's Discretion
- Exact definition of "one benchmark session" (see Open Questions — flagged, not
  discretionary in the sense of being safely guessable; needs explicit owner/manager
  confirmation before the session ledger is designed, but the planner may propose a
  default interpretation for the manager to confirm/reject).
- Which of the 3 sanctioned real backends (GitHub / Confluence / JIRA) to run the live
  MCP comparison against — all three are reachable today (see Environment Availability).
  Recommend GitHub + Confluence at minimum (Atlassian's server covers both Jira and
  Confluence in one connection) to keep the per-backend reduction table in
  `docs/benchmarks/token-economy.md` populated.
- Exact capture-artifact schema (recommended below, not mandated).

### Deferred Ideas (OUT OF SCOPE for this phase)
- Wiring the actual `FIX_IMPL_THEN_BIND` assertions into the bench scripts (deferred to a
  future code phase; P115 only needs to document the path).
- `perf/headline-numbers-cross-check.py` full implementation — the verifier script is
  confirmed ABSENT from `quality/gates/perf/` (dangling row)
  `[VERIFIED: quality/catalogs/perf-targets.json:114 "confirmed ABSENT entirely"]` —
  out of scope to write it in this phase; only note the path.
- DOCS-05/DOCS-07 relabeling prose itself (Phases 117/118).

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| BENCH-01 | Re-measure the live token-economy + latency benchmark figures backing the 8 hero-number doc-alignment-waived rows before 2026-08-15; ≤50 benchmark sessions; output consumable by DOCS-05/DOCS-07 | § "The 8 Rows, Exhaustively" (identifies exact targets), § "Prior Methodology" (identifies what's reproducible vs needs live capture), § "What Is a Live MCP Session Here" (defines substrate + session unit — flagged LOW confidence, needs confirmation), § "Recommended Capture Artifact" (the consumable form), § "Validation Architecture" (session-ledger + un-waive path) |
</phase_requirements>

## Architectural Responsibility Map

> Standard browser/frontend/API/CDN/DB tiers don't apply to a benchmark-measurement
> phase. Substituted with the tiers actually load-bearing here.

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Latency timing (cold init, cached read) | Local benchmark harness (`quality/gates/perf/latency-bench.sh` + `reposix` binary) | Real backend REST API (github/confluence/jira, when creds present) | Script already spawns real processes and times them; no MCP/agent involved at all |
| Live MCP-mediated token capture | External MCP service (Atlassian remote MCP server / GitHub remote MCP server, OAuth) | Live agent session (Claude Code or Claude API with tool-use) | This is the ONLY capability requiring a real MCP protocol round-trip — the thing the "50 sessions" ceiling actually rations |
| reposix-side token capture | Local benchmark harness + real backend REST API | — | A fresh, honestly-labeled shell-session transcript against a real (not FUSE-era, not fixture) reposix checkout of the same real-backend content used for the MCP side |
| Token counting | Anthropic `count_tokens` API via `quality/gates/perf/bench_token_economy_io.py::get_or_count` | SHA-256 content-keyed cache (`benchmarks/fixtures/*.tokens.json`) | Already built, cheap, already the project's standard — do not hand-roll a tokenizer estimate |
| Results consumption | Committed docs artifact (`docs/benchmarks/*.md`) | Quality catalog (`quality/catalogs/doc-alignment.json` + `perf-targets.json`) | Planner's downstream consumers (P118/DOCS-05) read committed markdown, not session transcripts directly |
| Session-spend tracking | New committed ledger artifact (recommended, does not yet exist) | — | No existing "spend ledger" convention found in the repo (`grep` for session-log/spend_tracker turned up nothing analogous); must be designed fresh |

## The 8 Rows, Exhaustively

Confirmed by exact string count: `grep -c '"2026-08-15T00:00:00Z"'
quality/catalogs/doc-alignment.json` → **8** `[VERIFIED]`. Each row below cites its
`id`, `claim`, and `source` fields verbatim from the catalog.

| # | Row id | Claim | Source file:line | Underlying hero number |
|---|--------|-------|-------------------|------------------------|
| 1 | `docs/index/latency-24ms-cold-init` | "27 ms cold init against simulator" | `docs/index.md:18` | 27ms cold init (sim) |
| 2 | `docs/index/latency-8ms-read` | "8 ms cached read against simulator" | `docs/index.md:18` | 8ms cached read (sim) |
| 3 | `docs/index/token-reduction-89-percent` | "89.1% fewer tokens vs MCP for same 3-issue read+edit+push workflow" | `docs/index.md:17` | 89.1% token reduction |
| 4 | `latency-hero-24ms-mismatch` | "Homepage hero: reposix cold init = 27 ms" | `docs/index.md:18` | 27ms cold init (dup framing of #1) |
| 5 | `README-md/token-89-percent` | "Input context token reduction is 89.1% vs MCP-tool-catalog baseline" | `README.md:27` | 89.1% token reduction |
| 6 | `README-md/latency-8ms` | "Read one issue from local cache takes 8ms (measured on simulator)" | `README.md:23` | 8ms cached read (sim) |
| 7 | `README-md/init-24ms` | "reposix init cold bootstrap against simulator takes 27ms" | `README.md:26` | 27ms cold init (sim) |
| 8 | `docs/why/token-economy-89-1-percent` | "Token reduction is 89.1% vs MCP" | `docs/index.md:17` | 89.1% token reduction (dup framing of #3) |

**Reduces to 3 distinct hero numbers, 8 catalog rows.** Rows 1/4/7 are the same 27ms
cold-init figure (docs/index.md hero + README.md + a row explicitly labeled "same line,
alternate framing" pattern elsewhere in this catalog — near-duplicate rows for one
source line are an established, intentional catalog convention here, not a defect).
Rows 2/6 are the same 8ms cached-read figure. Rows 3/8 are the same 89.1% figure. All 8
share the identical `waiver.reason`: *"interim qualified figure; retired/re-bound by
funded live MCP re-measurement, Q1 2026-07-12"* and the identical `tracked_in`:
`.planning/milestones/audits/2026-07-12-reality-check.md §5 Q1`
`[VERIFIED: doc-alignment.json]`.

**A near-miss NOT in the 8:** `docs/why/cold-init-24ms-sim` (claim: "reposix init cold
takes 24ms against simulator", source `docs/concepts/mental-model-in-60-seconds.md:21`)
sits immediately adjacent to row 7 in the catalog file but does not carry a
`2026-08-15` waiver in this pass — confirmed by the exhaustive 8-count above. It IS the
same underlying 24ms-vs-27ms inconsistency the reality-check audit flagged as POLISH
("27ms vs 24ms cold-init inconsistency", L287) — worth the re-measurement resolving
both numbers to one consistent figure even though only the 7-row instance is formally
in the waiver's scope. **Noticing, not a fix:** flag for the planner to decide whether
to touch this 9th row opportunistically while already re-measuring cold-init.

## Prior Methodology

### Latency (`docs/benchmarks/latency.md`)
- **Reproducer:** `bash quality/gates/perf/latency-bench.sh` — regenerates the file
  in place `[VERIFIED: docs/benchmarks/latency.md:8,69-71]`.
- **What's measured:** median-of-3 wall-clock timings of `reposix init` cold, list
  records, get one record, PATCH (no-op), and a local-only `capabilities` probe, against
  the sim (always) and github/confluence/jira (when the matching credential bundle is
  present in env) `[VERIFIED: quality/gates/perf/latency-bench.sh:1-38]`.
- **Reproducible, real infra — the gap is assertion, not measurement.** The script only
  soft-WARNs at >500ms; it never asserts the specific 27ms/8ms hero numbers
  `[VERIFIED: doc-alignment.json rationale, e.g. row 1: "IMPL_GAP: sim.sh measures
  SIM_INIT_MS but the only check is a non-fatal soft WARN at >500ms; never asserts
  27ms"]`.
- **Currently stale, not currently live:** the committed table's timestamp is
  `2026-04-27T13:40:53Z` — the reality-check audit flagged this as "~2.5 months old"
  hand-pasted snapshot `[VERIFIED: docs/benchmarks/latency.md:2,7; CITED: reality-check
  L278-280]`. Re-running the existing script produces a genuinely fresh, genuinely live
  measurement — no new tooling required for this track.
- **CI already runs the real-backend variant when secrets exist:** `bench-latency-v09`
  job in `.github/workflows/ci.yml` (see also the weekly `bench-latency-cron.yml`) wires
  `GITHUB_TOKEN`, `ATLASSIAN_API_KEY`/`ATLASSIAN_EMAIL`/`REPOSIX_CONFLUENCE_TENANT`,
  `JIRA_EMAIL`/`JIRA_API_TOKEN`/`REPOSIX_JIRA_INSTANCE` from `secrets.*`
  `[VERIFIED: .github/workflows/bench-latency-cron.yml:46-52]`.

### Token economy (`docs/benchmarks/token-economy.md`)
- **Reproducer:** `python3 quality/gates/perf/bench_token_economy.py` (the
  `scripts/bench_token_economy.py` path is a thin migrated shim, both work identically)
  `[VERIFIED: quality/gates/perf/bench_token_economy.py:1-43; scripts/bench_token_economy.py]`.
- **Real tokenizer, fixture inputs.** Token counts ARE real — via Anthropic's
  `count_tokens` endpoint (`anthropic==0.72.0` pinned in `requirements-bench.txt`), cached
  content-addressed in `benchmarks/fixtures/*.tokens.json` so `--offline` reruns (incl.
  CI) need no network call `[VERIFIED: docs/benchmarks/token-economy.md:4,42;
  requirements-bench.txt]`. **CI never sets `ANTHROPIC_API_KEY`** — grep of all
  `.github/workflows/*.yml` found zero references — confirming the live-network
  `count_tokens` call has only ever been run locally/manually by a human or agent with a
  key, then committed as cache `[VERIFIED: grep across .github/workflows/]`.
- **The MCP-side input is synthetic, admittedly so.** `mcp_jira_catalog.json` is "a
  representative manifest of 35 Jira tools, modeled on the public Atlassian Forge
  surface and the schemas produced by the `mcp-atlassian` server" — explicitly labeled
  "not against a live MCP server" `[VERIFIED: docs/benchmarks/token-economy.md:46-49;
  docs/concepts/reposix-vs-mcp-and-sdks.md:23-32]`.
- **The reposix-side input's provenance claim is FALSE, not just synthetic.**
  `token-economy.md:51-52` claims `reposix_session.txt` is "the literal output of
  `scripts/demo.sh`" — that script does not exist in the repo, and the fixture's content
  depicts the deleted FUSE architecture (`/mnt/reposix/...` paths) with internally
  inconsistent IDs `[CITED: reality-check L253-259, graded LAUNCH-BLOCKER]`. This is
  DOCS-05's target (Phase 117), separate from but informed by this phase's fresh capture.
- **Per-backend table (BENCH-02 legacy) already has github/confluence synthetic
  fixtures too** (`github_issues.json`, `confluence_pages.json` — both labeled
  "synthetic" in their own provenance section) plus a placeholder row for "Jira (real
  adapter)" marked "(pending re-measurement)" `[VERIFIED: docs/benchmarks/token-economy.md:21-28,53-56]`
  — this placeholder row is effectively what BENCH-01 is now funded to fill in.
- **Catalog rows for this script are separately WAIVED** (`perf/token-economy-bench`
  until 2026-09-15, `perf/headline-numbers-cross-check` until 2026-09-15, the latter's
  verifier script CONFIRMED ABSENT) in `quality/catalogs/perf-targets.json` — a
  DIFFERENT waiver track from the 8 `doc-alignment.json` rows this phase targets, with a
  later deadline. Success criterion 4 (documented un-waive path) should reference these
  two rows explicitly since they are the actual gate that would flip GREEN once the
  bench scripts assert the new figures `[VERIFIED: quality/catalogs/perf-targets.json:48-125]`.

## What Is a Live MCP Session Here

**The blocking condition is resolved.** `docs/concepts/reposix-vs-mcp-and-sdks.md:30-32`
explicitly deferred real-MCP numbers to when "the upstream Atlassian MCP server is GA
and stable enough to bench against." As of this research pass:
- Atlassian's official remote MCP server (`atlassian/atlassian-mcp-server`,
  Cloudflare-hosted, OAuth 2.1 or API token, covers Jira + Confluence + JSM + Bitbucket +
  Compass) reached General Availability
  `[CITED: MEDIUM confidence, multiple sources — see Sources; not independently
  confirmed via Context7/official Atlassian docs in this session, WebSearch-only]`.
- GitHub's official remote MCP server (`https://api.githubcopilot.com/mcp/`) has been GA
  since September 2025 `[CITED: github.blog changelog, MEDIUM-HIGH confidence]`.

Both are usable from Claude Code (this project's own agent runtime) via standard
`mcpServers` client config. **This is new information not reflected anywhere in the
repo** — worth a `docs/concepts/reposix-vs-mcp-and-sdks.md` prose update once BENCH-01
lands (candidate for Phase 117/118, not this phase's job to edit).

### Substrate recommendation
Run the live-MCP arm against the **sanctioned real backends already reachable in this
environment** (verified live during this research session):
```
$ bash scripts/preflight-real-backends.sh
PASS  Confluence key=TokenWorld — space "TokenWorld reposix demo space" reachable
PASS  GitHub — reubenjohn/reposix open_issues=3 private=False
PASS  JIRA — KAN "My Kanban Space" software
RESULT: PASS — sanctioned real-backend targets reachable. Safe to start P91+.
```
`[VERIFIED: live command run this session, exit 0]`. Do NOT use the sim as the live-MCP
substrate — the sim has no MCP server in front of it and was never the disputed claim's
target; the disputed claim is specifically about a REAL MCP server against REAL backend
content (per the reality-check's "not a live MCP transcript" framing).

### What constitutes "one session" — LOW CONFIDENCE, FLAGGED

No file in this repo defines "benchmark session" as a unit. Two competing readings,
both consistent with the locked-constraint wording ("≤50 benchmark sessions on the
EXISTING subscription... NO new API spend"):

- **H1 (recommended default):** one session = one live agentic run/conversation
  (e.g. one Claude Code session, or one Claude API tool-use conversation) connected to a
  real MCP server, executing the task end-to-end. "Existing subscription" = the Claude
  subscription/API account already in use for this project's own agent sessions (not a
  new pay-as-you-go key). "No new API spend" = don't provision a SEPARATE metered API
  key or paid MCP tier beyond what's already available. Under H1, a design of median-of-3
  samples × up to 3 backends × 2 arms (MCP-mediated, reposix-mediated) = **at most 18
  sessions**, comfortably under 50, leaving buffer for pilot/debug runs.
- **H2:** one session = one metered Anthropic API call (e.g. each `count_tokens` or
  completion call counts individually). Under H2 the ceiling is far more binding since a
  single agentic conversation can issue dozens of tool calls.

**Recommendation for the planner:** treat H1 as the working default (it is the reading
most consistent with "spend ceiling" + "no new API spend" being stated as two SEPARATE
constraints — sessions ration a subscription-plan resource, spend rations a
pay-as-you-go resource), but surface this ambiguity to the owner/manager explicitly
before executing — the session-ledger artifact (below) must be designed to count
whichever unit is confirmed, and getting it wrong either wastes budget-tracking effort
or silently blows the ceiling.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Token counting | A manual `len(text)/4` estimate or a new tokenizer wrapper | `quality/gates/perf/bench_token_economy_io.py::get_or_count` + its SHA-256 content-cache | Already the project's standard, already network-free on cache hit, already the thing DOCS-05 will point to as "honest" |
| Latency timing | A new stopwatch script | `quality/gates/perf/latency-bench.sh` (median-of-3, real golden path, already wired to all 4 backends) | Re-running it is strictly simpler and more consistent than writing a parallel one-off |
| Real-backend credential handling | New env-var names or ad hoc secrets | The existing `GITHUB_TOKEN` / `ATLASSIAN_API_KEY`+`ATLASSIAN_EMAIL`+`REPOSIX_CONFLUENCE_TENANT` / `JIRA_EMAIL`+`JIRA_API_TOKEN`+`REPOSIX_JIRA_INSTANCE` bundle, gated by `REPOSIX_ALLOWED_ORIGINS` | This is the project's tainted-by-default egress control (OP-1/OP-2); a new ad hoc credential path bypasses the audited allowlist |
| Real-backend test targets | New throwaway test data | The 3 sanctioned targets in `docs/reference/testing-targets.md` (Confluence "TokenWorld", GitHub `reubenjohn/reposix`, JIRA `KAN`/`TEST`) | "Go crazy, it's safe" — owner-blessed, cleanup procedure already documented |

**Key insight:** every piece of measurement infrastructure this phase needs already
exists and is real (not stubbed) — the work is orchestration (spin up live MCP sessions,
capture transcripts) and documentation (record figures + methodology in a consumable
form), not new tooling.

## Recommended Capture Artifact

Two-part output, modeled on the existing `docs/benchmarks/*.md` structure so P118/DOCS-05
can consume it the same way they already read `token-economy.md`/`latency.md`:

1. **`docs/benchmarks/token-economy.md`** — regenerate via
   `bench_token_economy.py` after replacing the two fixture inputs:
   - `benchmarks/fixtures/mcp_jira_catalog.json` → a captured (or faithfully
     transcribed) real tool-list + tool-call/response payload from an actual live MCP
     session against Atlassian's remote MCP server, doing the "read 3 issues, edit 1"
     task against real Confluence/Jira content.
   - `benchmarks/fixtures/reposix_session.txt` → a freshly captured, honestly-labeled
     ANSI-stripped shell transcript of the equivalent task run through a real `reposix`
     checkout against the SAME real backend (resolves the DOCS-05 provenance lie at the
     source, not just in prose).
   - Keep the "Fixture provenance" section but rewrite it to name the actual capture
     method (e.g. "captured 2026-07-XX via a live Claude Code session connected to
     Atlassian's remote MCP server against the TokenWorld/KAN sanctioned targets") in
     place of the "modeled on" / "literal output of `scripts/demo.sh`" language.
2. **`docs/benchmarks/latency.md`** — regenerate via `latency-bench.sh` with all three
   real-backend credential bundles exported (already reachable per the preflight check
   above), producing a fresh timestamp and fresh sim + real-backend rows.
3. **New: a session-spend ledger** (recommended path:
   `docs/benchmarks/bench-session-ledger.md` or a sibling JSON/CSV, planner's call) —
   one row per live-MCP session consumed, columns: `timestamp`, `backend`, `arm`
   (MCP-mediated / reposix-mediated), `task`, `session unit consumed` (per the H1/H2
   resolution above), `running total`, `artifact produced` (which fixture file it fed).
   This is the "tracked, first-class artifact" the locked constraints require — it does
   not exist yet anywhere in the repo (no analogous convention found by grep for
   session-log/spend-ledger patterns).
4. **A short methodology note** (can live at the top of the regenerated
   `token-economy.md`, mirroring its existing "Honest caveats" section) stating: which
   MCP server was used, its GA status/version at capture time, which real-backend
   content was read, and the exact task definition — so a skeptical reader (or DOCS-07)
   can trace the figure back to a real, reproducible act rather than a fixture.

## Common Pitfalls

### Pitfall 1: Conflating the two tracks and treating all 8 rows as "needing MCP"
**What goes wrong:** Reading the phase name ("Live MCP benchmark re-measurement")
literally causes the executor to try to route the pure-latency rows (1/2/4/6/7 — 27ms
cold init, 8ms cached read) through an MCP session, which doesn't make sense (reposix's
OWN latency has nothing to do with MCP).
**Why it happens:** ROADMAP's success criterion 1 says "captured via live MCP sessions"
for "every one of the 8," which reads as if MCP applies to all 8.
**How to avoid:** Read "live" as the operative word (opposite of stale/hand-pasted/
synthetic), not "MCP" — 5 rows need a live (re-run, real-backend) LATENCY measurement;
3 rows need a live MCP-mediated TOKEN measurement.
**Warning signs:** A plan that tries to time `reposix init` cold-init latency via an MCP
tool call.

### Pitfall 2: Burning session budget before the ledger exists
**What goes wrong:** Running exploratory/pilot live-MCP sessions to "see what happens"
before the spend-tracking artifact is committed, making the eventual total untrackable
and impossible to prove ≤50.
**Why it happens:** Natural tendency to experiment first, document later.
**How to avoid:** Commit the empty ledger schema in the FIRST commit of this phase
(catalog-first-rule analog), increment it after every session, before moving to the next.
**Warning signs:** A session ledger with gaps, or backfilled timestamps.

### Pitfall 3: Trusting MCP tool-call output as safe context
**What goes wrong:** Feeding a live Confluence/Jira page body captured through the MCP
session directly into any outbound action (e.g. echoing it into a commit message,
another API call, or an uncontrolled prompt) without treating it as tainted.
**Why it happens:** The whole point of this exercise is to capture and use real remote
content — easy to forget the project's own tainted-by-default rule (OP-1/OP-2) applies
to MCP-sourced bytes exactly as much as to reposix-sourced bytes.
**How to avoid:** Treat captured MCP transcripts the same as any other remote byte per
`docs/how-it-works/trust-model.md` — fine to store as benchmark fixtures (inert text),
never route into a side-effecting action outside the sanctioned test targets.
**Warning signs:** A captured transcript containing content that gets forwarded
somewhere other than `benchmarks/fixtures/`.

### Pitfall 4: Assuming the un-waive is this phase's job
**What goes wrong:** Spending phase time wiring assertions into
`bench_token_economy.py`/`latency-bench.sh` or writing the missing
`headline-numbers-cross-check.py`, which is explicitly deferred (success criterion 4
only asks for a "documented path").
**Why it happens:** It's tempting to "finish the job" once the figures are captured.
**How to avoid:** Stop at "documented path to un-waive" — name the exact script + line
that needs the assertion, cite the catalog row id, and stop.
**Warning signs:** A diff touching `.py`/`.sh` gate files during this "top-level,
fan-out-and-interpret" phase.

## Code Examples

### Re-running the latency bench with all real backends (no MCP, no session budget)
```bash
# Source: quality/gates/perf/latency-bench.sh:76-86 (docs/benchmarks/latency.md reproduce section)
export GITHUB_TOKEN=…
export ATLASSIAN_API_KEY=… ATLASSIAN_EMAIL=… REPOSIX_CONFLUENCE_TENANT=…
export JIRA_EMAIL=… JIRA_API_TOKEN=… REPOSIX_JIRA_INSTANCE=…
export REPOSIX_ALLOWED_ORIGINS='https://api.github.com,https://reuben-john.atlassian.net'
bash quality/gates/perf/latency-bench.sh
```

### Re-running the token-economy bench once fixtures are replaced
```bash
# Source: docs/benchmarks/token-economy.md:58, benchmarks/README.md
ANTHROPIC_API_KEY=<key> python3 quality/gates/perf/bench_token_economy.py
# subsequent reruns (no network, cache hit):
python3 quality/gates/perf/bench_token_economy.py --offline
```

### Pre-flight check before spending any live-MCP session budget
```bash
# Source: docs/reference/testing-targets.md; verified live this research session
bash scripts/preflight-real-backends.sh
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| MCP baseline modeled from a synthesized 35-tool manifest (not a live server) | Real MCP servers from Atlassian and GitHub are GA and reachable via OAuth | Atlassian GA reported this year; GitHub GA Sept 2025 | The project's own documented blocker for a real MCP re-measurement no longer applies — this phase is now technically executable, not just funded |
| `reposix_session.txt` fixture claimed as "literal output of `scripts/demo.sh`" (false; script doesn't exist; FUSE-era content) | A freshly captured, honestly-labeled real session transcript against current git-native architecture | This phase's expected output | Fixes the provenance lie at its root, ahead of DOCS-05's prose relabeling |

**Deprecated/outdated:**
- The "~150k→~2k (98.7%)" FUSE-era north-star figure at `docs/research/initial-report/performance.md:9` — separately flagged for retraction under DOCS-07 (Phase 118), not this phase's job, but this phase's 89.1%-or-better re-measured figure is the number DOCS-07 will cite in its place.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | "One benchmark session" = one live agentic conversation/run against a real MCP server (H1), not one metered API call (H2) | § "What constitutes 'one session'" | If H2 is correct, a naive plan sized at "18 sessions" could actually mean 18 conversations × many API calls each, silently blowing past 50 on the true metering unit — the planner must get this confirmed before designing the ledger |
| A2 | Atlassian's remote MCP server is GA and usable today with the sanctioned TokenWorld/KAN credentials | § "What Is a Live MCP Session Here" | WebSearch-only, not independently confirmed via Context7 or by actually connecting an MCP client in this session (out of scope for research — would spend budget); if GA status is wrong or OAuth setup fails, the plan needs a fallback (e.g. `sooperset/mcp-atlassian` self-hosted, also found in search results, as a non-official but real MCP server against the same real backend) |
| A3 | "Existing subscription" refers to the Claude/Anthropic subscription this agent runtime already runs under, not a separate line item | § User Constraints, § "What constitutes 'one session'" | If wrong, budget tracking targets the wrong resource entirely |

**If this table is empty:** N/A — see rows above; none are empty.

## Open Questions

1. **What exactly counts as one "benchmark session" against the ≤50 ceiling?**
   - What we know: it's distinct from "API spend" (both constraints stated separately);
     it's tied to "the existing subscription."
   - What's unclear: whether it's per-conversation, per-tool-call, or per-backend-probe.
   - Recommendation: the planner should surface H1/H2 (above) to the owner/manager as an
     explicit confirm-before-execute question in the phase plan itself, and default to
     the more conservative reading (H2, count every metered call) if no answer arrives
     before execution must start, to stay safely under budget.

2. **Which MCP server(s) to actually connect for the live session — official Atlassian
   remote MCP, official GitHub remote MCP, or a self-hosted alternative
   (`sooperset/mcp-atlassian`)?**
   - What we know: both official servers are reported GA; sanctioned real-backend creds
     for all 3 backends are reachable right now.
   - What's unclear: whether OAuth setup for the official Atlassian remote server is
     feasible within this phase's time/session budget, versus using the existing API-key
     based `sooperset/mcp-atlassian` (unofficial but real, and already the model the
     current synthetic fixture is based on).
   - Recommendation: try the official server first (closer to what an end-user would
     actually experience); fall back to the unofficial one if OAuth setup consumes
     disproportionate session budget — either way it is a REAL MCP server, resolving the
     "not a live MCP transcript" complaint.

3. **Does the 9th near-miss row (`docs/why/cold-init-24ms-sim`, 24ms vs the 8 rows' 27ms)
   get resolved opportunistically in this phase or deferred?**
   - What we know: it's the same underlying figure, flagged as a POLISH inconsistency by
     the reality-check audit, but not formally in the 8-row waiver scope for this
     deadline.
   - What's unclear: whether fixing it now (while already re-measuring cold-init) is
     cheaper than a separate future phase touching the same doc line.
   - Recommendation: resolve it in the same pass if the re-measured cold-init figure
     naturally produces one consistent number across `docs/index.md`, `README.md`, AND
     `docs/concepts/mental-model-in-60-seconds.md:21` — low marginal cost, but do not let
     it expand scope into a full docs-alignment refresh (that's Phase 117/126's job).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| GitHub creds (`GITHUB_TOKEN`) | latency + MCP tracks, `reubenjohn/reposix` | ✓ (verified live this session) | — | — |
| Confluence creds (TokenWorld) | latency + MCP tracks | ✓ (verified live this session) | — | — |
| JIRA creds (`KAN`/`TEST`) | latency + MCP tracks | ✓ (verified live this session) | — | — |
| `reposix` binary + sim | latency track | Not directly checked this session (requires `cargo build -p reposix-cli -p reposix-sim`); assume buildable per project norms | — | — |
| Anthropic API key for `count_tokens` | token-economy fixture (re)cache | ✗ in THIS research session (`ANTHROPIC_API_KEY` unset/renamed in env) — execution session must independently confirm | — | If genuinely unavailable, cache-hit `--offline` reruns still work for UNCHANGED fixtures, but new live-captured fixtures need at least one networked run to populate their `.tokens.json` sidecar |
| Official Atlassian remote MCP server (OAuth) | live-MCP token-economy capture | Not connected/tested this session (would consume session budget — out of scope for research) | GA per WebSearch (MEDIUM confidence) | `sooperset/mcp-atlassian` (self-hosted, API-token auth) |
| GitHub official remote MCP server | live-MCP token-economy capture (optional 3rd backend) | Not connected/tested this session | GA since 2025-09 per WebSearch | GitHub CLI (`gh`)-mediated manual session if remote MCP setup stalls |

**Missing dependencies with no fallback:** None identified — every dependency has at
least a documented fallback.

**Missing dependencies with fallback:** Anthropic API key (fallback: reuse existing
cached fixtures for anything unchanged, only the NEW live-captured fixtures need one
fresh networked run); official MCP servers (fallback: unofficial self-hosted
alternative, still a real live MCP server).

## Validation Architecture

> `workflow.nyquist_validation` is not explicitly `false` in `.planning/config.json` (it
> is `true`) — section included. Adapted: this phase produces no new source code, so
> "tests" here means the catalog-row verification path, not unit/integration tests.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | None (measurement/documentation phase, not a code phase) — verification is via the quality-catalog `WAIVED`→`BOUND` lifecycle |
| Config file | `quality/catalogs/doc-alignment.json` (8 target rows), `quality/catalogs/perf-targets.json` (2 related-but-separate rows: `perf/token-economy-bench`, `perf/headline-numbers-cross-check`) |
| Quick run command | `bash scripts/preflight-real-backends.sh` (confirms substrate reachable before spending session budget) |
| Full suite command | `bash quality/gates/perf/latency-bench.sh && python3 quality/gates/perf/bench_token_economy.py --offline` (regenerates both results docs from whatever fixtures/backends are current) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| BENCH-01 (latency rows 1/2/4/6/7) | Fresh sim + real-backend latency table | manual-only (re-run + eyeball diff against old table; no assertion exists yet) | `bash quality/gates/perf/latency-bench.sh` | ✅ script exists; ❌ no assertion of the specific hero numbers (this is the "path to un-waive," not this phase's job to close) |
| BENCH-01 (token rows 3/5/8) | Fresh live-MCP + fresh reposix-session token counts | manual-only (live session capture, then `--offline` rerun to confirm cache-stable) | `python3 quality/gates/perf/bench_token_economy.py --offline` (after replacing fixtures) | ✅ script exists; ❌ `headline-numbers-cross-check.py` confirmed ABSENT |
| BENCH-01 (session ledger) | ≤50-session spend stays tracked | manual-only (no automated counter exists) | N/A — new artifact this phase must create | ❌ Wave 0 gap |

### Sampling Rate
- **Per session:** append one row to the new session-spend ledger immediately after
  each live-MCP session completes — do not batch.
- **Per track completion:** regenerate the corresponding `docs/benchmarks/*.md` and diff
  against the previous committed version.
- **Phase gate:** both results docs regenerated + fresh timestamps + session ledger
  total ≤50 + a written "path to un-waive" note referencing the exact catalog row ids
  and the exact script/line each still needs, before `/gsd-verify-work`.

### Wave 0 Gaps
- [ ] Session-spend ledger artifact — does not exist anywhere in the repo; must be
  created in this phase's first commit (see § Recommended Capture Artifact item 3).
- [ ] `quality/gates/perf/headline-numbers-cross-check.py` — confirmed absent; NOT this
  phase's job to write, but the "documented path to un-waive" must explicitly name this
  gap so a future phase doesn't have to rediscover it.
- [ ] No existing harness captures a live MCP session transcript into a fixture file —
  this phase is the first to need that capture step; no committed tooling for it exists
  yet (manual: capture via Claude Code's own conversation transcript export, or a
  wrapper script, planner's call).

*(Framework install: none needed — `anthropic` package already pinned in
`requirements-bench.txt`; MCP client capability is Claude Code's own built-in
`mcpServers` config, no additional install.)*

## Security Domain

> `security_enforcement` is absent from `.planning/config.json` → treat as enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | Reuse existing sanctioned OAuth/API-token bundles (`GITHUB_TOKEN`, `ATLASSIAN_API_KEY`+`ATLASSIAN_EMAIL`, `JIRA_EMAIL`+`JIRA_API_TOKEN`) for both the reposix side AND the MCP client side; never mint new tokens or widen scopes beyond what `docs/reference/testing-targets.md` already sanctions |
| V4 Access Control | yes | MCP session must only touch the 3 sanctioned targets (TokenWorld, `reubenjohn/reposix`, JIRA `KAN`/`TEST`) — same "go crazy, it's safe" boundary that already governs reposix's own real-backend tests |
| V5 Input Validation | yes | Captured MCP tool-call output is tainted per project threat model — safe to store as an inert benchmark fixture, never fed into a side-effecting action outside the sanctioned targets |
| V6 Cryptography | n/a | No new crypto surface introduced by this phase |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Captured remote content (Confluence/Jira/GitHub body text via MCP) routed into an unintended outbound action | Tampering / Information Disclosure | Treat as `Tainted<T>` per `docs/how-it-works/trust-model.md`; keep captured transcripts confined to `benchmarks/fixtures/`, never echoed into `git push` targets outside the allowlist |
| New MCP client credentials committed accidentally (OAuth token/API key in a captured transcript) | Information Disclosure | Scrub captured session transcripts for credential material before committing as a fixture (the existing `reposix_session.txt` precedent already does ANSI-stripping; extend to credential-stripping) |

## Sources

### Primary (HIGH confidence)
- `quality/catalogs/doc-alignment.json` — direct read, all 8 waived rows + the
  near-miss 9th row, exact grep-count confirmation
- `quality/catalogs/perf-targets.json:48-125` — the 2 related-but-separate WAIVED rows
  and the confirmed-absent verifier script
- `quality/gates/perf/latency-bench.sh`, `quality/gates/perf/latency-bench/github.sh`,
  `quality/gates/perf/bench_token_economy.py`, `quality/gates/perf/bench_token_economy_io.py`
  — direct read of the actual (non-stub) measurement code
- `docs/benchmarks/latency.md`, `docs/benchmarks/token-economy.md`,
  `benchmarks/README.md` — direct read of the current committed results + methodology
- `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, `.planning/SESSION-HANDOVER.md`,
  `.planning/MANAGER-HANDOVER.md` — direct read of the locked constraints and phase scope
- `.planning/milestones/audits/2026-07-12-reality-check.md` — direct read of the launch-
  blocker findings that motivated this phase
- Live command execution this session: `bash scripts/preflight-real-backends.sh` (exit 0,
  all 3 sanctioned targets PASS)

### Secondary (MEDIUM confidence)
- GitHub official remote MCP server GA status —
  https://github.blog/changelog/2025-09-04-remote-github-mcp-server-is-now-generally-available/
  (changelog, dated, credible primary-source blog)

### Tertiary (LOW confidence)
- Atlassian remote MCP server GA status — WebSearch aggregate only (MindStudio blog,
  Atlassian community forum thread, Atlassian marketing page); NOT independently
  confirmed via Context7 or by connecting an actual MCP client in this session. Flagged
  as Assumption A2 — the planner/executor should re-verify directly (e.g. attempt the
  OAuth connect flow) before relying on it as fact.

## Metadata

**Confidence breakdown:**
- Standard stack (existing scripts, catalog rows, 8-row enumeration): HIGH — all
  directly read from committed files with file:line citations
- Architecture (2-track split, tier mapping): HIGH — derived directly from reading the
  actual (non-stub) measurement scripts
- Session-unit semantics / live-MCP feasibility: LOW — no repo artifact defines "session";
  MCP-server GA status is WebSearch-only, not independently verified against an actual
  connection attempt

**Research date:** 2026-07-15
**Valid until:** 2026-07-29 (short TTL — the hard waiver deadline is 2026-08-15, and the
MCP-server GA claims are fast-moving external facts that should be re-verified close to
execution time, not trusted from this research pass alone)
