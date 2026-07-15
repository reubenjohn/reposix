---
phase: 115-live-mcp-benchmark-re-measurement
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - docs/benchmarks/latency.md
  - docs/benchmarks/token-economy.md
  - benchmarks/fixtures/mcp_jira_catalog.json
  - benchmarks/fixtures/mcp_jira_catalog.json.tokens.json
  - benchmarks/fixtures/reposix_session.txt
  - benchmarks/fixtures/reposix_session.txt.tokens.json
  - benchmarks/fixtures/README.md
  - benchmarks/bench-session-ledger.md
  - .planning/phases/115-live-mcp-benchmark-re-measurement/115-UNWAIVE-PATH.md
autonomous: false            # A1 session-unit ruling is a blocking checkpoint:decision to the MANAGER
requirements: [BENCH-01]
execution_mode: top-level    # fan-out → gather → interpret; the top-level coordinator IS the executor (NOT gsd-executor)

user_setup:
  - service: "Session JSONL usage records (Claude Code) — PRIMARY token-economy source [AMENDED #10 2026-07-15]"
    why: "T5 token-economy HEADLINE numbers derive from the captured Claude Code session JSONL usage records (parsed by the session-analyzer skill) — honest end-to-end session cost. NO ANTHROPIC_API_KEY, no networked count, first run included; reruns stay offline-on-CI from committed fixtures + counts. See the jsonl-usage-methodology amendment below + CONSULT-DECISIONS.md 2026-07-15 [SELF] P115-T5 token-economy methodology."
    env_vars: []
  - service: "Anthropic count_tokens — OPTIONAL later per-artifact enrichment ONLY [AMENDED #10]"
    why: "NOT required for the T5 headline. count_tokens gives isolated per-artifact token cost — an optional later enrichment. The endpoint is free of charge; subscription OAuth could authenticate it if ever wired — NO pay-as-you-go key, no new API spend."
    env_vars:
      - name: ANTHROPIC_API_KEY
        source: "NOT required (JSONL-usage methodology). Only if optional count_tokens enrichment is later wired, then via existing-subscription auth — never a new pay-as-you-go key."
  - service: "Live MCP servers + sanctioned real backends"
    why: "Track B captures a real MCP transcript + a real reposix transcript against the 3 owner-sanctioned targets."
    env_vars:
      - name: GITHUB_TOKEN
        source: "existing; reubenjohn/reposix"
      - name: ATLASSIAN_API_KEY
        source: "existing; TokenWorld (Confluence) + KAN (Jira)"
      - name: ATLASSIAN_EMAIL
        source: "existing"
      - name: REPOSIX_CONFLUENCE_TENANT
        source: "existing"
      - name: JIRA_EMAIL
        source: "existing"
      - name: JIRA_API_TOKEN
        source: "existing"
      - name: REPOSIX_JIRA_INSTANCE
        source: "existing"
      - name: REPOSIX_ALLOWED_ORIGINS
        source: "egress allowlist — must include api.github.com + the Atlassian tenant; never widen beyond sanctioned targets"

must_haves:
  truths:
    - "Fresh latency measurements (cold init + cached read) for the sim AND all 3 real backends exist in docs/benchmarks/latency.md, timestamped after 2026-07-15."
    - "The 24ms-vs-27ms cold-init discrepancy is resolved to ONE authoritative measured figure."
    - "Fresh live-MCP token-economy figures exist, captured from a REAL MCP server against the sanctioned real backends (not the synthetic 35-tool manifest)."
    - "benchmarks/fixtures/reposix_session.txt is an honest, current git-native transcript (no /mnt FUSE paths, no scripts/demo.sh provenance)."
    - "A session-spend ledger records every live-MCP session with a running total that stays ≤50."
    - "docs/benchmarks/token-economy.md provenance section names the real capture method, not 'modeled on' / 'literal output of scripts/demo.sh'."
    - "A documented un-waive path names the exact perf-targets rows + script/line a future code phase must wire."
  artifacts:
    - path: "docs/benchmarks/latency.md"
      provides: "Regenerated latency table, fresh timestamp, sim + 3 real-backend columns"
      contains: "last_measured_at"
    - path: "docs/benchmarks/token-economy.md"
      provides: "Regenerated token-economy figures + honest provenance + methodology note"
      contains: "89"
    - path: "benchmarks/fixtures/mcp_jira_catalog.json"
      provides: "Real live-MCP tool-list + tool-call/response capture (replaces synthetic manifest)"
    - path: "benchmarks/fixtures/reposix_session.txt"
      provides: "Honest git-native reposix transcript of the same task (replaces FUSE-era fixture)"
    - path: "benchmarks/bench-session-ledger.md"
      provides: "First-class session-spend ledger, one row per live-MCP session, running total ≤50"
      contains: "running_total"
    - path: ".planning/phases/115-live-mcp-benchmark-re-measurement/115-UNWAIVE-PATH.md"
      provides: "Documented path to un-waive perf/token-economy-bench + perf/headline-numbers-cross-check"
      contains: "headline-numbers-cross-check"
  key_links:
    - from: "benchmarks/bench-session-ledger.md"
      to: "each live-MCP session in Task 4"
      via: "one appended row per session, monotonic timestamps, running_total column"
      pattern: "running_total"
    - from: "docs/benchmarks/token-economy.md"
      to: "benchmarks/fixtures/{mcp_jira_catalog.json,reposix_session.txt}"
      via: "bench_token_economy.py reads the replaced fixtures; provenance section names the capture"
      pattern: "captured .* live"
    - from: ".planning/phases/115-live-mcp-benchmark-re-measurement/115-UNWAIVE-PATH.md"
      to: "quality/catalogs/perf-targets.json"
      via: "names row ids perf/token-economy-bench + perf/headline-numbers-cross-check + the script/line each needs"
      pattern: "perf/token-economy-bench"
---

<objective>
Re-measure the live token-economy + latency benchmark figures backing the **8
hero-number `doc-alignment.json` rows** (waiver HARD-EXPIRES 2026-08-15), so that
Phase 118 (DOCS-07) and DOCS-05 (Phase 117) can relabel the figures WITHOUT re-deriving
them.

The 8 rows reduce to **3 distinct hero numbers across two independent tracks**
(115-RESEARCH.md § "The 8 Rows, Exhaustively"):

- **Latency track** (5 rows — "27 ms cold init", "8 ms cached read", ids 1/2/4/6/7):
  re-run the EXISTING `quality/gates/perf/latency-bench.sh` against the sim + all 3 real
  backends. Consumes ZERO session budget. The current table is stale
  (`last_measured_at: 2026-04-27`).
- **Token-economy track** (3 rows — "89.1% fewer tokens", ids 3/5/8): needs real,
  budget-tracked **live MCP sessions** against sanctioned backends, replacing the
  synthetic `mcp_jira_catalog.json` manifest and the FUSE-era `reposix_session.txt`
  (whose `scripts/demo.sh` provenance is a lie — that script does not exist).

Purpose: unblock DOCS-05/DOCS-07 relabeling + the waived `perf/token-economy-bench` +
`perf/headline-numbers-cross-check` rows, before the deadline.

Output: regenerated `docs/benchmarks/{latency,token-economy}.md`, replaced honest
fixtures, a first-class `benchmarks/bench-session-ledger.md`, and a documented un-waive
path.

**Execution mode: top-level.** This is a fan-out → gather → interpret shape, NOT
write-code-test-commit. The top-level coordinator IS the executor; `gsd-executor` lacks
the `Task` tooling for this shape (`.planning/CLAUDE.md` § orchestration-shaped phases).
No `.py`/`.sh` gate file is edited by this phase (Pitfall 4 / Deferred Ideas).
</objective>

<execution_context>
> Top-level orchestration phase — do NOT dispatch this to `gsd-executor`. Run it as a
> top-level coordinator: fan out the two tracks to subagents, gather their captures,
> interpret against the 8 waived rows, commit. Relief + cadence doctrine:
@.planning/ORCHESTRATION.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/115-live-mcp-benchmark-re-measurement/115-RESEARCH.md

# Existing measurement infra (re-run, do NOT rewrite):
@quality/gates/perf/latency-bench.sh
@quality/gates/perf/bench_token_economy.py
@docs/benchmarks/latency.md
@docs/benchmarks/token-economy.md
@docs/reference/testing-targets.md

<interfaces>
<!-- The executor uses these directly — no code exploration needed. -->

Latency track (ZERO session budget):
  bash quality/gates/perf/latency-bench.sh
    # Regenerates docs/benchmarks/latency.md in place. Sim always runs (~15s);
    # github/confluence/jira columns populate when the matching credential bundle is in env
    # (~60s all-in). Median-of-3, real golden path. Soft-WARN only; asserts nothing.
    # Spawns reposix-sim --ephemeral and inits into its OWN mktemp dirs — the script
    # handles its own /tmp isolation, so running the SCRIPT does not violate leaf-isolation.
    # REQUIRES reposix + reposix-sim on PATH: `cargo build -p reposix-cli -p reposix-sim`
    # (ONE cargo invocation machine-wide — never parallel).

Token-economy track — HEADLINE via JSONL usage [AMENDED #10]:
  # T5 headline token numbers derive from captured Claude Code session JSONL usage records
  # (session-analyzer skill) — NO ANTHROPIC_API_KEY, no networked count. See the
  # jsonl-usage-methodology amendment above.
  # OPTIONAL later per-artifact enrichment (free-of-charge endpoint, no pay-as-you-go key):
  #   python3 quality/gates/perf/bench_token_economy.py   # count_tokens via get_or_count; SHA-keyed *.tokens.json cache
  python3 quality/gates/perf/bench_token_economy.py --offline
    # Cache-hit rerun; no network. Use to confirm the regenerated doc is cache-stable.

Substrate preflight (run BEFORE spending any session budget):
  bash scripts/preflight-real-backends.sh    # exit 0 = all 3 sanctioned targets reachable

The 8 rows (115-RESEARCH.md § "The 8 Rows, Exhaustively") — 3 hero numbers:
  - 27ms cold init (sim): rows docs/index/latency-24ms-cold-init, latency-hero-24ms-mismatch, README-md/init-24ms
  - 8ms cached read (sim): rows docs/index/latency-8ms-read, README-md/latency-8ms
  - 89.1% token reduction: rows docs/index/token-reduction-89-percent, README-md/token-89-percent, docs/why/token-economy-89-1-percent
  NOTE: several row IDs say "24ms" while their claim text says "27ms" — resolving to ONE
  measured figure is Task 2's job. The 9th NEAR-MISS row docs/why/cold-init-24ms-sim
  (mental-model-in-60-seconds.md:21) is NOT in the 8; this phase RECORDS the authoritative
  number for it but does NOT edit its prose (that is Phase 117/118).
</interfaces>
</context>

<amendment id="jsonl-usage-methodology" date="2026-07-15" rotation="#10" self-ruling="CONSULT-DECISIONS.md 2026-07-15 [SELF] P115-T5 token-economy methodology">
ADOPTED — JSONL-usage methodology (supersedes the count_tokens-first plan body below):

- **Primary source shift.** T5 token-economy HEADLINE numbers derive from the captured
  Claude Code **session JSONL usage records** (parsed by the `session-analyzer` skill), NOT
  the Anthropic `count_tokens` endpoint. JSONL usage = honest END-TO-END session cost (the
  headline); `count_tokens` = isolated PER-ARTIFACT cost, an OPTIONAL later enrichment only.
- **ANTHROPIC_API_KEY requirement DROPPED — entirely, first run included.** JSONL usage is
  read from the committed session records; no networked count is needed for the headline.
  This removes the T4/T5 owner-block on providing an existing-subscription key.
- **Reruns stay offline-on-CI** from committed fixtures + counts; the SHA-256-keyed
  `*.tokens.json` sidecar caching is UNCHANGED.
- **count_tokens is free of charge** and, if ever wired for optional per-artifact enrichment,
  could authenticate via subscription OAuth — NEVER a new pay-as-you-go key, no new API spend.

**Supersedes in the body below:** (1) the `user_setup` count_tokens/ANTHROPIC_API_KEY entry
— now optional; (2) Task 1 execution-start gate item 3 (ANTHROPIC_API_KEY presence) — NO
LONGER a gate; (3) the "Token-economy track" how-to-run `ANTHROPIC_API_KEY=<existing>`
invocation; (4) Task 5 step 1's networked count_tokens run.

**T5-executor reconciliations (post-reset):** `quality/gates/perf/bench_token_economy.py`
today counts via `count_tokens` (`get_or_count` in `bench_token_economy_io.py`) — the T5
executor adds a JSONL-usage path (via `session-analyzer`) as the HEADLINE source and demotes
count_tokens to optional enrichment; the Task 5 `<automated>` check + the token-economy.md
provenance/methodology note update accordingly. Catalog-first if a perf-row contract changes.
</amendment>

<tasks>

<task type="checkpoint:decision" gate="blocking">
  <name>Task 1 (Wave 1): A1 session-unit ruling GATE + execution-start re-verify — HARD gate on Track B</name>
  <decision>What is "one benchmark session" against the ≤50 ceiling? This unit is UNDEFINED in-repo and MUST be ruled by the MANAGER before ANY session spend or ledger population. Do NOT bake in a value.</decision>
  <context>
Locked constraint (w1:p7): ≤50 benchmark sessions on the EXISTING subscription; NO new
API spend; escalate past 50 to the MANAGER. Research (§ "What constitutes 'one session'")
found two readings, both consistent with the wording:
  - H1 (research's recommended default): one session = one live agentic run/conversation
    connected to a real MCP server, end-to-end. Under H1 a median-of-3 × ≤3 backends × 2
    arms design = ≤18 sessions — comfortable buffer under 50.
  - H2 (conservative): one session = one metered Anthropic API call. Far more binding.
The ledger's unit column is meaningless until this is ruled — getting it wrong either
wastes tracking effort (H1 when H2 meant) or silently blows the ceiling (H2 when H1 meant).

**Also re-verify at this gate (execution-start facts, research TTL expires 2026-07-29):**
  1. `bash scripts/preflight-real-backends.sh` exits 0 (all 3 sanctioned targets reachable).
  2. Live-MCP GA (A2, MEDIUM confidence, WebSearch-only): attempt the official Atlassian
     remote MCP server OAuth connect; if it stalls/fails, fall back to `sooperset/mcp-atlassian`
     (self-hosted, API-token, still a REAL MCP server). Record which server is used.
  3. ~~`ANTHROPIC_API_KEY` presence~~ — **SUPERSEDED by the jsonl-usage-methodology amendment
     (#10): NO LONGER A GATE.** T5 headline token numbers derive from committed session JSONL
     usage records (session-analyzer), needing no key. count_tokens is optional later
     enrichment only; do not block Track B on a key.
  </context>
  <options>
    <option id="h1-per-conversation">
      <name>H1 — one session = one live agentic conversation/run</name>
      <pros>Consistent with "sessions" and "API spend" being two SEPARATE constraints; leaves buffer; research's recommended default</pros>
      <cons>If wrong, a single conversation issuing dozens of tool calls silently under-counts</cons>
    </option>
    <option id="h2-per-metered-call">
      <name>H2 — one session = one metered Anthropic API call</name>
      <pros>Conservative; cannot silently blow the ceiling</pros>
      <cons>Far more binding; may force a tiny sample size</cons>
    </option>
  </options>
  <resume-signal>MANAGER rules: "h1-per-conversation", "h2-per-metered-call", or a third definition. Record the ruling verbatim in benchmarks/bench-session-ledger.md's header (Task 3) BEFORE any session is spent.</resume-signal>
  <verify>
    <automated>bash scripts/preflight-real-backends.sh; echo "exit=$?"</automated>
  </verify>
  <done>MANAGER ruling on the session unit is recorded; preflight exits 0; the MCP server to use is chosen (official or fallback). (ANTHROPIC_API_KEY presence is NO LONGER a gate — jsonl-usage-methodology amendment #10.) NO ledger rows and NO live-MCP session exist yet.</done>
</task>

<task type="auto">
  <name>Task 2 (Wave 1, PARALLEL with Task 1 — NO gate dependency, ZERO session budget): Latency track re-measurement</name>
  <files>docs/benchmarks/latency.md</files>
  <action>
Re-run the EXISTING harness — do NOT write a parallel one (Don't-Hand-Roll). This track
consumes ZERO of the 50-session ceiling and does NOT depend on the A1 ruling, so it runs
immediately, in parallel with the gate.

1. Build the binaries with ONE cargo invocation: `cargo build -p reposix-cli -p reposix-sim`
   (never run a second cargo concurrently — VM OOM risk).
2. Export all 3 real-backend credential bundles + `REPOSIX_ALLOWED_ORIGINS`
   (115-RESEARCH.md § Code Examples; same bundle CI's bench-latency-v09 job uses).
3. `bash quality/gates/perf/latency-bench.sh` — regenerates docs/benchmarks/latency.md
   with a fresh `last_measured_at` timestamp + sim and real-backend columns. The script
   spawns `reposix-sim --ephemeral` and inits into its OWN mktemp dirs, so running the
   SCRIPT does not touch the shared tree (leaf-isolation satisfied; do NOT hand-run
   `reposix init` in the shared repo).
4. Read off the measured cold-init and cached-read figures. **Resolve the 24ms-vs-27ms
   discrepancy** (row IDs say "24ms", claims say "27ms") to the ONE number the harness
   actually produces. Record that authoritative figure in the phase SUMMARY so Phase
   117/118 can relabel index.md/README.md/mental-model-in-60-seconds.md consistently.
   **Scope guard:** do NOT edit those doc prose lines here — this phase MEASURES; the
   prose relabel (incl. the 9th near-miss row) is Phase 117/118.
5. `git add` ONLY docs/benchmarks/latency.md (targeted add — shared tree).
  </action>
  <verify>
    <automated>test "$(grep -m1 last_measured_at docs/benchmarks/latency.md | grep -oE '2026-[0-9]{2}-[0-9]{2}')" '>' 2026-07-14 && grep -qiE 'cold.?init' docs/benchmarks/latency.md && echo FRESH</automated>
  </verify>
  <done>docs/benchmarks/latency.md carries a fresh (> 2026-07-14) timestamp with sim + real-backend cold-init and cached-read rows; the authoritative cold-init figure (resolving 24-vs-27) is recorded in the SUMMARY. Zero session budget consumed.</done>
</task>

<task type="auto">
  <name>Task 3 (Wave 2, depends on Task 1 ruling): Session-spend ledger scaffold — committed BEFORE any session spend</name>
  <files>benchmarks/bench-session-ledger.md</files>
  <action>
Create the first-class session-spend ledger (no analogous artifact exists in-repo —
design it fresh; Pitfall 2: commit the EMPTY schema before spending, never backfill).
Location `benchmarks/` (sibling of fixtures/) — deliberately OUTSIDE `docs/` so it does
NOT trip the mkdocs `no-orphan-docs` freshness invariant, while staying a committed,
auditable artifact.

Header MUST record, verbatim, the Task-1 MANAGER ruling on what counts as one session
(H1 / H2 / other) + the ceiling (≤50) + the escalation target (MANAGER, past 50).

One row per live-MCP session with columns:
  | # | timestamp (UTC, ISO-8601) | backend | arm (mcp-mediated / reposix-mediated) | task | unit_consumed (per ruling) | running_total | artifact_produced (which fixture) |

Commit the header + empty table (zero data rows) NOW. Task 4 appends one row immediately
after each session, incrementing running_total, before starting the next.
  </action>
  <verify>
    <automated>grep -qiE 'running_total' benchmarks/bench-session-ledger.md && grep -qiE '50' benchmarks/bench-session-ledger.md && echo LEDGER_SCAFFOLD_OK</automated>
  </verify>
  <done>benchmarks/bench-session-ledger.md exists with the ruled session-unit definition, the ≤50 ceiling, the MANAGER escalation note, the column header, and ZERO data rows.</done>
</task>

<task type="auto">
  <name>Task 4 (Wave 3, depends on Task 1 gate + Task 3 ledger): Live-MCP token capture — budget-tracked ≤50</name>
  <files>benchmarks/fixtures/mcp_jira_catalog.json, benchmarks/fixtures/reposix_session.txt, benchmarks/bench-session-ledger.md</files>
  <action>
Run a small, FIXED number of real, budget-tracked live-MCP sessions against the
sanctioned targets (TokenWorld / KAN / reubenjohn/reposix), executing the SAME task the
existing fixtures model: **"read 3 issues, edit 1, push."** Two arms per backend:

  - **MCP-mediated arm:** connect the chosen MCP server (Task 1) and capture the real
    tool-list + the tool-call/response payloads for the task. Replace
    `benchmarks/fixtures/mcp_jira_catalog.json` with this REAL capture (kills the "modeled
    on the Forge surface, not a live server" synthetic label).
  - **reposix-mediated arm:** run the equivalent task through a real `reposix` checkout of
    the SAME real-backend content, capture an ANSI-stripped shell transcript, replace
    `benchmarks/fixtures/reposix_session.txt` (kills the FUSE-era `/mnt/...` content + the
    false `scripts/demo.sh` provenance at its ROOT — ahead of DOCS-05's prose fix).
    **Leaf-isolation (project Non-negotiable — "leaf test setup runs in a throwaway `/tmp`
    clone, never the shared repo"; enforced by `.claude/hooks/leaf-isolation-guard.sh`):**
    this arm bootstraps a real `reposix init`/`attach` checkout against a LIVE sanctioned
    backend, so it MUST run in a throwaway `/tmp` clone `cd`'d into in the SAME Bash
    invocation — e.g. `cd "$(mktemp -d)" && reposix init <backend>::<project> . && <run the
    task, tee the transcript>` — NEVER hand-run `reposix init` / `git commit` / `git config`
    in the shared repo (mirrors Task 2's leaf-isolation callout). Then COPY (not git-mutate
    from the /tmp tree) the scrubbed transcript into
    `benchmarks/fixtures/reposix_session.txt` in the shared tree before `git add`; the /tmp
    checkout is disposable scratch.

Budget discipline (LOCKED): after EACH session, append one ledger row (Task 3 schema),
increment running_total, verify ≤50 BEFORE the next session. If the design would exceed
50 → STOP and escalate to the MANAGER (do NOT absorb by doing more work — Out of Scope).
Research sizing under H1: median-of-3 × ≤3 backends × 2 arms ≈ ≤18 sessions.

**Tainted-by-default (OP-1/OP-2, Pitfall 3, threat_model below):** captured
Confluence/Jira/GitHub body text is attacker-influenced. Store ONLY as inert fixtures
under `benchmarks/fixtures/`; NEVER echo captured content into a commit message, another
API call, or any outbound action outside the sanctioned allowlist. **Scrub captured
transcripts for credential material (OAuth tokens / API keys) before committing.**

`git add` ONLY the two fixture files + the ledger (targeted add — shared tree).
  </action>
  <verify>
    <automated>! grep -qE '/mnt/|scripts/demo\.sh' benchmarks/fixtures/reposix_session.txt && test "$(grep -cE '\|' benchmarks/bench-session-ledger.md)" -ge 2 && tail -1 benchmarks/bench-session-ledger.md | awk -F'|' '{v=$(NF-2); gsub(/ /,"",v); exit (v+0>50)}' && echo CAPTURE_OK</automated>
  </verify>
  <done>Both fixtures replaced with real live captures (reposix_session.txt has no /mnt or scripts/demo.sh); ledger has one row per session; the final data row's running_total is numerically ≤50 (or a MANAGER escalation is recorded if 50 would be exceeded); no credential material in committed transcripts.</done>
</task>

<task type="auto">
  <name>Task 5 (Wave 4, depends on Task 4): Regenerate token-economy.md from the real fixtures + honest provenance + methodology note</name>
  <files>docs/benchmarks/token-economy.md, benchmarks/fixtures/mcp_jira_catalog.json.tokens.json, benchmarks/fixtures/reposix_session.txt.tokens.json, benchmarks/fixtures/README.md</files>
  <action>
Regenerate the results doc from the REPLACED fixtures — do NOT hand-edit the numbers.

1. **[AMENDED #10 — JSONL usage is the headline]** Produce the headline token numbers from
   the captured Claude Code session JSONL usage records via the `session-analyzer` skill
   (end-to-end session cost) — NO ANTHROPIC_API_KEY, no networked count. Wire this as a
   `bench_token_economy.py` JSONL-usage path (headline source). OPTIONAL later per-artifact
   enrichment: `python3 quality/gates/perf/bench_token_economy.py` populates the SHA-keyed
   `*.tokens.json` sidecars via the free-of-charge count_tokens endpoint (no pay-as-you-go
   key) — do NOT block on it.
2. Confirm cache-stability: `python3 quality/gates/perf/bench_token_economy.py --offline`
   reproduces the SAME doc (no network).
3. **Rewrite the "Fixture provenance" section** (token-economy.md ~L44-52) to name the
   ACTUAL capture method — e.g. "captured 2026-07-XX via a live <server> MCP session
   against the sanctioned TokenWorld/KAN targets" — replacing "modeled on the Forge
   surface" and "the literal output of scripts/demo.sh". Mirror the same correction in
   `benchmarks/fixtures/README.md`.
4. **Add a methodology note** (top of token-economy.md, mirroring its "Honest caveats"
   style): which MCP server + its GA status/version at capture time, which real-backend
   content was read, and the exact task definition — so DOCS-07/a skeptical reader can
   trace the figure to a real, reproducible act. This is the form P118/DOCS-05 consume
   DIRECTLY (they read this committed markdown, not the transcripts).
5. Record the re-measured token-reduction figure (89.1%-or-current) as the authoritative
   number DOCS-07 cites in place of the deprecated FUSE-era "98.7%".
6. `git add` ONLY the listed files (targeted add — shared tree).
  </action>
  <verify>
    <automated>python3 quality/gates/perf/bench_token_economy.py --offline && ! grep -qE 'scripts/demo\.sh|modeled on' docs/benchmarks/token-economy.md && grep -qiE 'mcp' docs/benchmarks/token-economy.md && echo TOKENECON_OK</automated>
  </verify>
  <done>docs/benchmarks/token-economy.md regenerated from the real fixtures, offline-cache-stable, with an honest provenance section (no scripts/demo.sh / no "modeled on") and a methodology note naming the MCP server + task; benchmarks/fixtures/README.md provenance matched.</done>
</task>

<task type="auto">
  <name>Task 6 (Wave 5, depends on Task 2 + Task 5): Documented un-waive path + consumption-ready consolidation</name>
  <files>.planning/phases/115-live-mcp-benchmark-re-measurement/115-UNWAIVE-PATH.md</files>
  <action>
Success criterion 4 asks for a DOCUMENTED PATH, not the implementation (Pitfall 4 /
Deferred Ideas — do NOT wire assertions or write the absent script this phase).

Write 115-UNWAIVE-PATH.md naming, precisely, what a FUTURE code phase must do to flip
each waived row GREEN using the new figures:
  - `perf/token-economy-bench` (perf-targets.json, waived until 2026-09-15): the
    assertion the un-waive needs, in `quality/gates/perf/bench_token_economy.py`
    (name the value it must assert = the Task-5 re-measured figure).
  - `perf/headline-numbers-cross-check` (perf-targets.json, waived until 2026-09-15):
    its verifier `quality/gates/perf/headline-numbers-cross-check.py` is **CONFIRMED
    ABSENT** — the un-waive requires WRITING it (grep docs/index.md + README.md hero
    numbers against the fixtures, exit non-zero on mismatch). State this explicitly so a
    future phase doesn't rediscover the dangling row.
  - The 8 `doc-alignment.json` rows (waived until 2026-08-15): name the fresh figures
    (cold-init from Task 2, cached-read from Task 2, token-reduction from Task 5) each
    row would re-bind to, and that binding them is the FIX_IMPL_THEN_BIND future-phase
    step, not this phase's.

Then confirm consumption-readiness for P118/DOCS-05: the fresh figures live in the
committed `docs/benchmarks/{latency,token-economy}.md` (with methodology), and the final
ledger `running_total` is ≤50 (or a MANAGER escalation is recorded). Note the 9th
near-miss row (docs/why/cold-init-24ms-sim) + the authoritative cold-init figure for
Phase 117/118 to relabel — WITHOUT editing that prose here.
  </action>
  <verify>
    <automated>grep -q 'perf/token-economy-bench' .planning/phases/115-live-mcp-benchmark-re-measurement/115-UNWAIVE-PATH.md && grep -q 'headline-numbers-cross-check' .planning/phases/115-live-mcp-benchmark-re-measurement/115-UNWAIVE-PATH.md && grep -qi 'absent' .planning/phases/115-live-mcp-benchmark-re-measurement/115-UNWAIVE-PATH.md && tail -1 benchmarks/bench-session-ledger.md | awk -F'|' '{v=$(NF-2); gsub(/ /,"",v); exit (v+0>50)}' && echo UNWAIVE_OK</automated>
  </verify>
  <done>115-UNWAIVE-PATH.md names both perf-targets rows + the exact script/line each needs (incl. the confirmed-absent cross-check script), maps the 3 fresh figures to the 8 doc-alignment rows, and confirms the docs are P118/DOCS-05-consumable with a final-row running_total numerically ≤50.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| live MCP server → local capture | Confluence/Jira/GitHub body text captured through the MCP session is untrusted, attacker-influenced remote content |
| real backend REST → reposix checkout | Same tainted-byte boundary reposix already governs; the reposix-mediated arm reads the same remote content |
| captured transcript → committed fixture | Credentials (OAuth token / API key) can leak into a raw transcript before it becomes a committed fixture |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-115-01 | Information Disclosure / Tampering | Task 4 captured MCP/reposix transcript | mitigate | Treat captured content as `Tainted<T>` (docs/how-it-works/trust-model.md); confine to benchmarks/fixtures/ as inert text; NEVER echo into a commit message, another API call, or any outbound action outside REPOSIX_ALLOWED_ORIGINS (Pitfall 3) |
| T-115-02 | Information Disclosure | Task 4 committed fixture | mitigate | Scrub OAuth tokens / API keys from captured transcripts before commit; extend the existing reposix_session.txt ANSI-strip precedent to credential-stripping |
| T-115-03 | Elevation of Privilege / Access Control | live-MCP session scope | mitigate | Session touches ONLY the 3 sanctioned targets (TokenWorld, reubenjohn/reposix, JIRA KAN/TEST); reuse existing sanctioned credential bundles; never mint new tokens or widen scopes beyond docs/reference/testing-targets.md (V2/V4) |
| T-115-04 | Spoofing | fallback MCP server | accept | If the official Atlassian remote server OAuth stalls, the sooperset/mcp-atlassian fallback is self-hosted against the SAME sanctioned real backend with the existing API token — no new trust surface beyond the already-sanctioned backend |
</threat_model>

<verification>
Phase-gate checks (all must hold before `/gsd-verify-work`):
1. `bash quality/gates/perf/latency-bench.sh` regenerated docs/benchmarks/latency.md with a fresh (> 2026-07-14) timestamp + sim and real-backend rows.
2. `python3 quality/gates/perf/bench_token_economy.py --offline` reproduces docs/benchmarks/token-economy.md cache-stably; provenance names the real capture (no scripts/demo.sh, no "modeled on").
3. benchmarks/fixtures/reposix_session.txt contains no `/mnt/` and no `scripts/demo.sh`.
4. benchmarks/bench-session-ledger.md: ruled session-unit in header, one row per session, monotonic timestamps, final running_total numerically ≤50 (`tail -1 benchmarks/bench-session-ledger.md | awk -F'|' '{v=$(NF-2); gsub(/ /,"",v); exit (v+0>50)}'` exits 0), or a MANAGER escalation recorded.
5. 115-UNWAIVE-PATH.md names both perf-targets rows + the confirmed-absent cross-check script + the 3 fresh figures.
6. No `.py`/`.sh` gate file modified (Deferred Ideas / Pitfall 4).
Full mechanical signal set: `115-VALIDATION.md`.
</verification>

<success_criteria>
Maps 1:1 to ROADMAP Phase 115 success criteria + BENCH-01:

- **SC1** (fresh live measurements for all 8 rows): Task 2 (latency rows 1/2/4/6/7) + Tasks 4–5 (token rows 3/5/8). "Live" = re-run/real-backend/real-MCP, not stale/synthetic/FUSE-era.
- **SC2** (≤50 tracked ledger, escalate past 50): Task 3 (scaffold) + Task 4 (per-session increment + final-row numeric ≤50 assertion) + Task 6 (re-asserts the final-row numeric ≤50). A1 gate ensures the ledger counts the ruled unit.
- **SC3** (P118/DOCS-05-consumable form): Task 5 (regenerated token-economy.md + methodology) + Task 2 (regenerated latency.md) — committed markdown the downstream phases read directly.
- **SC4** (documented un-waive path): Task 6 (115-UNWAIVE-PATH.md).
- **BENCH-01**: the whole phase.
</success_criteria>

<output>
After completion, create
`.planning/phases/115-live-mcp-benchmark-re-measurement/115-01-SUMMARY.md` recording: the
3 authoritative re-measured figures (cold-init resolving 24-vs-27, cached-read,
token-reduction), the session-unit ruling + final ledger running_total, the MCP server
used (official vs fallback), and the noticing items (incl. the 9th near-miss row handed
to Phase 117/118). Push cadence: `git push origin main` BEFORE the verifier subagent,
then `python3 quality/runners/run.py --cadence post-push --persist`.
</output>
