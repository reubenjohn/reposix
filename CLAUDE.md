# CLAUDE.md — reposix project guide

This file is read by every agent (Claude Code, Codex, Cursor, etc.) that opens this repo. It's the local extension of the user's global CLAUDE.md (`~/.claude/CLAUDE.md`) and overrides nothing — it adds project-specific rules.

## Project elevator pitch

reposix exposes REST-based issue trackers (and similar SaaS systems) as a git-native partial clone, served by `git-remote-reposix` from a local bare-repo cache built from REST responses. Agents use `cat`, `grep`, `sed`, and `git` on real workflows — no MCP tool schemas, no custom CLI, no FUSE mount. See `docs/research/initial-report.md` for the architectural argument and `docs/research/agentic-engineering-reference.md` for the dark-factory pattern that motivates the simulator-first approach.

## Architecture (git-native partial clone)

> **Source of truth:** `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` (ratified 2026-04-24).

The reposix runtime has three pieces:

- **`reposix-cache`** — a real on-disk bare git repo built from REST responses via the `BackendConnector` trait. The cache produces a fully populated tree (filenames, directory structure, blob OIDs) but materializes blobs lazily — only when the helper requests one on git's behalf. Every materialization writes a row to the SQLite audit table; bytes return wrapped in `reposix_core::Tainted<Vec<u8>>`.
- **`git-remote-reposix`** — a hybrid git remote helper. It advertises `stateless-connect` (read path: tunnels protocol-v2 fetch traffic to the cache's bare repo with `--filter=blob:none`) and `export` (push path: parses the fast-import stream, runs push-time conflict detection against the backend, applies REST writes on success). Refspec namespace is `refs/heads/*:refs/reposix/*`.
- **`reposix init <backend>::<project> <path>`** — bootstraps a partial-clone working tree: `git init`, `extensions.partialClone=origin`, `remote.origin.url=reposix::<scheme>://<host>/projects/<project>`, then `git fetch --filter=blob:none origin`.
- **`reposix attach <backend>::<project>`** — adopt an existing checkout (vanilla `git clone` mirror, hand-edited tree, prior `reposix init`) and bind it to a `SoT` backend. Builds the cache from REST, walks the working-tree HEAD, reconciles records by frontmatter `id` (5 cases per architecture-sketch: match / no-id / backend-deleted / duplicate-id / mirror-lag), then sets `extensions.partialClone=reposix` (NOT `origin`) and `remote.reposix.url=reposix::<sot>?mirror=<plain-mirror-url>`. Re-attach against the same `SoT` is idempotent (Q1.3); against a different `SoT` is rejected (Q1.2 — multi-SoT not supported in v0.13.0). `REPOSIX_SIM_ORIGIN` overrides the default sim port (used by integration tests).

After `init`, agent UX is pure git: `cd <path> && git checkout origin/main && cat issues/<id>.md && grep -r TODO . && <edit> && git add . && git commit && git push`. Zero reposix CLI awareness required beyond `init` (or `attach` for adopted trees).

**Cache reconciliation table.** `reposix-cache` adds a `cache_reconciliation` table (`record_id PRIMARY KEY, oid, local_path, attached_at`) populated by `reposix attach` via `walk_and_reconcile`. One row per matched local record. `INSERT OR REPLACE` on re-attach (idempotent per Q1.3). NOT an audit table — it's reconciliation state, the append-only triggers in `audit_events_cache` do not apply. Audit-row trail for the attach walk itself lands in `audit_events_cache` with `op = 'attach_walk'` (OP-3 unconditional).

Two guardrails are load-bearing for the dark-factory pattern:

- **Push-time conflict detection.** The helper checks backend state when `git push` runs and rejects with the standard git "fetch first" error if the remote drifted. The agent recovers via `git pull --rebase && git push` — no custom protocol.
- **Blob limit.** The helper refuses `command=fetch` requests that would materialize more than `REPOSIX_BLOB_LIMIT` blobs (default 200), with a stderr error that names `git sparse-checkout` as the recovery move. An agent unfamiliar with reposix observes the error and recovers without prompt engineering.

**Mirror-lag refs.** `crates/reposix-cache/` writes two refs per SoT-host on every successful `handle_export` push (and, post-P83, on every successful bus push and webhook-driven mirror sync): `refs/mirrors/<sot-host>-head` (direct ref pointing at the cache's post-write synthesis-commit OID) and `refs/mirrors/<sot-host>-synced-at` (annotated tag whose message-body first line is `mirror synced at <RFC3339>`). `<sot-host>` is the SoT backend slug (`sim` | `github` | `confluence` | `jira`). Refs live in the **cache's bare repo**, NOT in the working tree's `.git/`; vanilla `git fetch` brings them along via the helper's `stateless-connect` advertisement (`git upload-pack --advertise-refs` propagates every non-hidden ref naturally — `transfer.hideRefs` only hides `refs/reposix/sync/*`). `git log refs/mirrors/<sot>-synced-at -1` reveals when the mirror last caught up, and the conflict-reject stderr cites the ref by name with a `(N minutes ago)` rendering for staleness diagnosis. **Important (Q2.2 doc-clarity contract):** `refs/mirrors/<sot>-synced-at` is the timestamp the mirror last caught up to `<sot>` — it is NOT a "current SoT state" marker. The staleness window the refs measure IS the gap between SoT-edit and webhook-fire. Full docs treatment defers to P85 (`docs/concepts/dvcs-topology.md`). Audit-row trail for the ref-write attempt lands in `audit_events_cache` with `op = 'mirror_sync_written'` (OP-3 unconditional — written even on ref-write failure). Ref writes themselves are best-effort `tracing::warn!` (mirroring the `Cache::log_*` family).

**L1 conflict detection (P81+).** On every push, the helper reads its cache cursor (`meta.last_fetched_at`), calls `backend.list_changed_since(since)`, and only conflict-checks records that overlap the push set with the changed-set. The cache is trusted as the prior; the agent's PATCH against a backend-deleted record fails at REST time with a 404 — recoverable via `reposix sync --reconcile` (DVCS-PERF-L1-02). On the cursor-present hot path, the precheck does ONE `list_changed_since` REST call plus ONE `get_record` per record in `changed_set ∩ push_set` (typically zero or one); the legacy unconditional `list_records` walk in `handle_export` is gone. First-push fallback (no cursor yet) and steady-state pushes with no actions executed (`files_touched == 0`) skip the post-write `refresh_for_mirror_head` to keep the no-op cost at zero list-records calls. L2/L3 hardening (background reconcile / transactional cache writes) defers to v0.14.0 per `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § Performance subtlety.

**Bus URL form (P82+).** `reposix::<sot-spec>?mirror=<mirror-url>` per Q3.3 — the SoT side dispatches via the existing `BackendConnector` pipeline (sim / confluence / github / jira); the mirror is a plain-git URL consumed as a shell-out argument to `git ls-remote` / `git push`. Bus is PUSH-only (Q3.4) — fetch on a bus URL falls through to the single-backend code path, so the helper does NOT advertise `stateless-connect` for bus URLs. The `+`-delimited form is dropped; unknown query keys (anything other than `mirror=`) are rejected. Mirror URLs containing `?` must be percent-encoded (the first unescaped `?` in the bus URL is the bus query-string boundary). On push, the bus handler runs two cheap prechecks BEFORE reading stdin: PRECHECK A (`git ls-remote -- <mirror>` versus local `refs/remotes/<name>/main`) and PRECHECK B (`list_changed_since` against the SoT cursor); both reject with `error refs/heads/main fetch first` on drift. P82 ships the URL-recognition + dispatch + precheck surface; the SoT-first write fan-out lands in P83. See `.planning/research/v0.13.0-dvcs/architecture-sketch.md § 3` and `decisions.md § Q3.3-Q3.6` for the algorithm + open-question resolutions.

**Bus write fan-out (P83-01+).** A bus push (`git push <reposix-remote> main` against a `reposix::<sot>?mirror=<url>` remote) runs the architecture-sketch's bus algorithm steps 4-9 after P82's prechecks pass: read fast-import from stdin, apply REST writes to `SoT` via the shared `write_loop::apply_writes` (single-backend `handle_export` calls the same function), then `git push <mirror_remote_name> main` to the GH mirror — plain push, NO `--force-with-lease` (P84 owns force-with-lease for the webhook race; D-08 RATIFIED). On `SoT`-success + mirror-success: both `refs/mirrors/<sot>-head` and `refs/mirrors/<sot>-synced-at` advance; `mirror_sync_written` audit row written; helper acks `ok refs/heads/main`. On `SoT`-success + mirror-FAIL: `head` advances but `synced-at` is FROZEN at the last successful mirror sync (observable lag for the vanilla-`git`-only operator); the new audit op `helper_push_partial_fail_mirror_lag` records SoT SHA + `git push` exit code + 3-line stderr tail (T-83-02 trim); helper still acks `ok refs/heads/main` — Q3.6 RATIFIED no helper-side retry (surface, audit, recover on next push or via webhook sync). On any `SoT`-fail: mirror push NEVER attempted; refs unchanged. Confluence partial state across actions (PATCH 1 succeeds, PATCH 2 fails) is NOT 2PC — recovery is next-push reads new `SoT` via PRECHECK B's `list_changed_since` (D-09 / Pitfall 3 in `.planning/phases/83-bus-write-fan-out/83-PLAN-OVERVIEW.md`). P83-01 ships steps 4-9 of the algorithm + the new audit op + the 3-line stderr tail trim; P83-02 lands fault-injection coverage + audit-completeness assertions. The `git push` shell-out inherits the helper's cwd (Pitfall 6) — same git-invocation context that resolved `<mirror_remote_name>` in P82's STEP 0. Fault-injection coverage (DVCS-BUS-WRITE-06): four integration tests under `crates/reposix-remote/tests/bus_write_*.rs` exercise mirror-fail (`bus_write_mirror_fail.rs`) / SoT-mid-stream-fail (`bus_write_sot_fail.rs`) / post-precheck-409 (`bus_write_post_precheck_409.rs`) / dual-table audit-completeness (`bus_write_audit_completeness.rs`) scenarios per RESEARCH.md § "Audit Completeness Contract" — every push end-state writes audit rows to BOTH `audit_events_cache` (cache-internal) AND `audit_events` (backend mutations), enforcing the OP-3 dual-table contract.

**Webhook-driven mirror sync (v0.13.0 P84+).** A reference GitHub Action workflow lives in the mirror repo's `.github/workflows/reposix-mirror-sync.yml` (NOT in `reubenjohn/reposix`; the canonical repo carries the template at `docs/guides/dvcs-mirror-setup-template.yml`). The two copies are byte-equal modulo whitespace; the catalog row `agent-ux/webhook-trigger-dispatch` enforces the invariant. Triggers: `repository_dispatch` (event_type=`reposix-mirror-sync`) for the webhook path + cron `*/30 * * * *` (literal — GH Actions parses schedule blocks BEFORE evaluating `${{ vars.* }}`, so cadence overrides require editing the YAML directly per Q4.1) for the safety net. Secrets convention: `gh secret set ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT` on the **mirror repo** (per `ci.yml:114-120` precedent). The workflow uses `cargo binstall reposix-cli` (NOT bare `reposix` — that's the workspace name; the binstall metadata lives in `crates/reposix-cli/Cargo.toml`). First-run handling (Q4.3) branches on `git show-ref --verify --quiet refs/remotes/mirror/main`: present → `--force-with-lease=...` push; absent → plain push. Race protection: a concurrent bus push (P82+P83) landing between the workflow's fetch and its push triggers a clean lease rejection — the mirror is already in sync. Latency target: p95 ≤ 120s (falsifiable threshold per ROADMAP P84 SC4). **Substrate gating:** the workflow's install step requires a published `reposix-cli` crate version with working binstall artifacts AND non-yanked `gix` deps; v0.12.0 fails on both legs (no binstall artifact + yanked `gix=0.82.0`). See `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md § 2026-05-01 16:43`. Full owner walk-through: `docs/guides/dvcs-mirror-setup.md` (P85).

## Operating Principles (project-specific)

The user's global Operating Principles in `~/.claude/CLAUDE.md` are bible. The following are project-specific reinforcements, not replacements:

1. **Simulator is the default / testing backend.** The simulator at `crates/reposix-sim/` is the default backend for every demo, unit test, and autonomous agent loop. Real backends (GitHub via `reposix-github`, Confluence via `reposix-confluence`, JIRA via `reposix-jira`) are guarded by the `REPOSIX_ALLOWED_ORIGINS` egress allowlist and require explicit credential env vars (`GITHUB_TOKEN`, `ATLASSIAN_API_KEY` + `ATLASSIAN_EMAIL` + `REPOSIX_CONFLUENCE_TENANT`, `JIRA_EMAIL` + `JIRA_API_TOKEN` + `REPOSIX_JIRA_INSTANCE`). Autonomous mode never hits a real backend unless the user has put real creds in `.env` AND set a non-default allowlist. This is both a security constraint (fail-closed by default) and the StrongDM dark-factory pattern.
2. **Tainted by default.** Any byte that came from a remote (simulator counts) is tainted. Tainted content must not be routed into actions with side effects on other systems (e.g. don't echo issue bodies into `git push` to remotes outside an explicit allowlist). The lethal-trifecta mitigation matters even against the simulator, because the simulator is *seeded* by an agent and seed data is itself attacker-influenced.
3. **Audit log is non-optional, and lives in TWO append-only tables.** `audit_events_cache` (cache-internal events — blob materialization, gc, helper RPC fetch/push, sync-tag writes, mirror-refs sync writes) lives in the cache crate (`reposix-cache::audit`); `audit_events` (backend mutations — `create_record` / `update_record` / `delete_or_close`) lives in the core crate (`reposix-core::audit`) and is written by the sim/confluence/jira adapters. A complete forensic query reads both. Either schema missing a row for a network-touching action means the feature isn't done. The dual-table shape is intentional; physical unification behind a `dyn AuditSink` trait is deferred.
4. **No hidden state.** Cache state, simulator state, and git remote helper state all live in committed-or-fixture artifacts. No "it works in my session" bugs.
5. **Working tree IS a real git checkout.** The whole point of v0.9.0 is that `.git/` is real, not synthetic; `git diff` is the change set by construction, not by emulation. The partial clone (`extensions.partialClone=origin`) makes blobs lazy, but everything else is upstream git.
6. **Real backends are first-class test targets.** Three canonical targets are sanctioned for aggressive testing: **Confluence space "TokenWorld"** (owned by the user; safe to mutate freely), **GitHub repo `reubenjohn/reposix` issues** (ours; safe to create/close issues during tests), and **JIRA project `TEST`** (default key; overridable via `JIRA_TEST_PROJECT` or `REPOSIX_JIRA_PROJECT`). See `docs/reference/testing-targets.md` for env-var setup, owner permission statement, and cleanup procedure. Simulator remains the default (OP-1), but "simulator-only coverage" does NOT satisfy acceptance for transport-layer or performance claims.
7. **Phase-close means catalog-row PASS.** No phase ships on the executing agent's word. An unbiased verifier subagent grades the catalog rows — if RED, the phase loops back. See "Verifier subagent dispatch" in the Quality Gates section below.
8. **Plans accommodate surprises (the +2 phase practice).** Every
   milestone reserves its **last two phases** as absorption slots for
   what reality surfaces during planned-phase execution:
   - **Slot 1 — Surprises absorption.** Issues a planned phase
     discovered but couldn't fix without doubling its scope. The
     discovering phase appends to
     `.planning/milestones/<v>-phases/SURPRISES-INTAKE.md` (one entry
     per item: severity + what + whf(c)zy-out-of-scope + sketched
     resolution) instead of silently skipping or expanding scope.
     The surprises-absorption phase drains the file: each entry → RESOLVED |
     DEFERRED | WONTFIX, each with a commit SHA or rationale.
   - **Slot 2 — Good-to-haves polish.** Improvements (clarity, perf,
     consistency, grounding) the planned phases observed but didn't
     fold in. Same intake mechanism, separate file
     (`GOOD-TO-HAVES.md`). Sized XS / S / M; XS items always close;
     M items default-defer to next milestone.

   **Eager-resolution preference (load-bearing):** when a planned
   phase observes a surprise or polish item, prefer fixing it inside
   the discovering phase IF (a) < 1 hour incremental work, (b) no
   new dependency introduced. The +2 reservation is for items that
   genuinely don't fit the discovering phase. **What this practice
   prevents:** the "found-it-but-skipped-it" failure mode where good
   signal gets dropped to keep a phase tight; AND the
   "scope-creep-to-fit-the-finding" failure mode where a phase grows
   to twice its planned size to absorb every drift discovered. The
   intake split makes "I saw it, here's what I think, P<last-2> will
   handle it" the default move.

   **Verifier honesty check:** the surprises-absorption phase's
   verifier subagent spot-checks the previous phases' plans + verdicts
   and asks "did this phase honestly look for out-of-scope items?" An
   empty intake is acceptable IF the running phases produced
   "Eager-resolution" decisions in their plans; an empty intake when
   the verdicts show skipped findings is a RED signal. This prevents
   the practice from degrading into a no-op.

   The +2 reservation is in addition to whatever planned phases the
   milestone scopes; if the milestone has 8 planned phases, it
   actually has 10 (planned + 2 reservation). Roadmap entries for the
   reservation phases name them explicitly so they're not omitted by
   accident.

   **v0.13.0 surprises-absorption (P87, 2026-05-01)** — completed:
   5-entry intake drained (2 RESOLVED-on-discovery via eager-resolution +
   3 P87-drained as RESOLVED|WONTFIX|DEFERRED); honesty spot-check
   GREEN at `.planning/phases/87-surprises-absorption/honesty-spot-check.md`
   (sampled 5 phases, exceeded the ≥3 floor). One v0.13.0 → v0.13.x
   carry-forward: binstall + yanked-gix release substrate (P84
   SURPRISES Entry 5; release-pipeline territory; owner-runnable
   `scripts/webhook-latency-measure.sh` ready). Full v0.13.0 milestone
   retrospective lands in P88 per OP-9.
9. **Milestone-close ritual: distill before archiving.** Each
   milestone's `*-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES}.md` entries
   AND the autonomous-run session findings get distilled into a new
   section of `.planning/RETROSPECTIVE.md` BEFORE the milestone
   archives — using the existing template (What Was Built / What
   Worked / What Was Inefficient / Patterns Established / Key
   Lessons). Raw intake files travel with the milestone archive into
   `*-phases/`; distilled lessons live permanently and discoverably
   in `RETROSPECTIVE.md`. **Why:** without this step, learnings get
   lost in milestone archives — the +2 phase practice produces signal
   that's worth keeping cross-milestone (failure modes, patterns,
   process gaps) but the raw intake format is too granular for future
   readers to skim. The ratification subagent for milestone-close
   should verify a RETROSPECTIVE.md section exists for the milestone
   and grade RED if it doesn't.

## Workspace layout

```
crates/
├── reposix-core/        # Shared types: Record, Project, RemoteSpec, Error, Tainted<T>.
├── reposix-sim/         # In-process axum HTTP simulator.
├── reposix-cache/       # On-disk bare-repo cache backed by gix; lazy blob materialization.
├── reposix-remote/      # git-remote-reposix binary (stateless-connect + export).
├── reposix-cli/         # Top-level `reposix` CLI (`init`, `sim`, `list`, `refresh`, `spaces`).
├── reposix-github/      # GitHub Issues BackendConnector.
├── reposix-confluence/  # Confluence Cloud BackendConnector.
├── reposix-jira/        # JIRA Cloud BackendConnector.
└── reposix-swarm/       # Multi-agent contention/swarm test harness.

.planning/               # GSD project state. Do not hand-edit; use /gsd-* commands.
docs/                    # User-facing docs (reference, benchmarks, demos, testing-targets).
research/                # Long-form research notes + red-team reports.
runtime/                 # gitignored — local sim DB, scratch working trees.
```

### `.planning/milestones/` convention (HANDOVER §0.5 / Option B)

Per-milestone planning artifacts live INSIDE the matching `*-phases/` dir,
never loose at `.planning/milestones/` top-level:

```
.planning/milestones/
├── v0.8.0-phases/
│   ├── ARCHIVE.md             # condensed milestone log (per POLISH2-21)
│   ├── ROADMAP.md             # milestone-level scoping (kept intact)
│   ├── REQUIREMENTS.md        # milestone-level scoping (kept intact)
│   └── tag-v0.8.0.sh          # historical one-shot release script
├── v0.9.0-phases/
│   ├── ROADMAP.md
│   └── tag-v0.9.0.sh
└── v0.10.0-phases/
    ├── ROADMAP.md
    └── tag-v0.10.0.sh
```

The `freshness/no-loose-roadmap-or-requirements` claim in
`scripts/end-state.py` enforces this — any `*ROADMAP*.md` or
`*REQUIREMENTS*.md` placed loose at `.planning/milestones/v*.0-*.md`
fails the verifier. New milestones must scaffold their planning docs
inside `*-phases/` from day one.

## Tech stack

- Rust stable (1.82+ via `rust-toolchain.toml`).
- Async: `tokio` 1.
- Web: `axum` 0.7 + `reqwest` 0.12 (rustls only, never openssl-sys).
- Git: `gix` 0.83 (pinned with `=` because gix is pre-1.0; bumped from 0.82 in P78 — issues #29 + #30 yanked the prior pin). **Runtime requirement: `git >= 2.34`** for `extensions.partialClone` + `stateless-connect`.
- Storage: `rusqlite` 0.32 with `bundled` feature (no system libsqlite3).
- Errors: `thiserror` for typed crate errors, `anyhow` only at binary boundaries.

## Commands you'll actually use

```bash
# Local dev loop
cargo check --workspace                                   # fast type check
cargo test --workspace                                    # unit + integration tests
cargo clippy --workspace --all-targets -- -D warnings     # CI lint
cargo fmt --all                                           # CI fmt

# Run the stack
cargo run -p reposix-sim                                  # start simulator on :7878
cargo run -p reposix-cli -- init sim::demo /tmp/repo      # bootstrap a partial-clone working tree
cd /tmp/repo && git checkout origin/main                  # agent UX from here is pure git
cat issues/0001.md && grep -ril TODO . && git push        # cat, grep, edit, push

# Attach an existing checkout (vanilla GH-mirror clone or hand-edited tree, v0.13.0+)
git clone git@github.com:org/issues-repo.git /tmp/issues  # vanilla mirror clone (no reposix needed)
cd /tmp/issues
reposix attach sim::demo --remote-name reposix            # build cache from REST; reconcile by frontmatter id; add reposix remote
git push reposix main                                     # push via reposix remote (single-SoT shape; bus URL form requires P82+)

# L1 escape hatch (v0.13.0+): rebuild the cache from REST when a push reject suggests cache desync
reposix sync --reconcile                                  # full list_records walk + cache rebuild (DVCS-PERF-L1-02)

# Bus push (v0.13.0+ P83-01): URL form `reposix::<sot>?mirror=<mirror-url>` recognized + dispatched; cheap prechecks (mirror drift + SoT drift) gate the push BEFORE reading stdin. Write fan-out shipped in P83-01 — SoT-first + mirror-best-effort + lag-tracking (DVCS-BUS-WRITE-01..05); fault-injection coverage lands in P83-02.
git push reposix main                                     # bus push (URL: reposix::<sot>?mirror=<url>; SoT-first + mirror-best-effort + lag-tracking — DVCS-BUS-WRITE-01..05 in v0.13.0)

# Webhook-driven mirror sync (v0.13.0 P84+; mirror repo only)
gh api repos/reubenjohn/reposix-tokenworld-mirror/dispatches \
  -f event_type=reposix-mirror-sync                       # manually trigger mirror sync (synthetic; cron is */30min)
bash scripts/webhook-latency-measure.sh                    # owner-runnable n=10 real-TokenWorld latency measurement (gated on v0.13.x release with working binstall — see SURPRISES-INTAKE)

# Dark-factory regression (proves agent UX is pure git, zero in-context learning)
bash scripts/dark-factory-test.sh sim                          # v0.9.0 arm — init + partial-clone + helper teaching strings (local + CI)
bash quality/gates/agent-ux/dark-factory.sh dvcs-third-arm     # v0.13.0 P86 arm — vanilla-clone + reposix attach + bus URL composition + cache audit (local + CI)

# Testing against real backends — see docs/reference/testing-targets.md for env-var setup.
# Confluence — TokenWorld space (safe to mutate)
export ATLASSIAN_API_KEY=… ATLASSIAN_EMAIL=… REPOSIX_CONFLUENCE_TENANT=reuben-john
export REPOSIX_ALLOWED_ORIGINS='https://reuben-john.atlassian.net'
cargo test -p reposix-cli --test agent_flow_real -- --ignored dark_factory_real_confluence

# GitHub — reubenjohn/reposix issues (safe to mutate)
export GITHUB_TOKEN=…
export REPOSIX_ALLOWED_ORIGINS='https://api.github.com'
cargo test -p reposix-cli --test agent_flow_real -- --ignored dark_factory_real_github

# JIRA — default project key TEST (overridable via JIRA_TEST_PROJECT or REPOSIX_JIRA_PROJECT)
export JIRA_EMAIL=… JIRA_API_TOKEN=… REPOSIX_JIRA_INSTANCE=…
export JIRA_TEST_PROJECT=TEST
cargo test -p reposix-cli --test agent_flow_real -- --ignored dark_factory_real_jira
```

## GSD workflow

This project uses GSD (`get-shit-done`) for planning and execution. Workflow rule:

> **Always enter through a GSD command.** Never edit code outside a GSD-tracked phase or quick.

Entry points:

- `/gsd-quick` — small fix or doc tweak.
- `/gsd-execute-phase <n>` — run a planned phase end-to-end with subagents.
- `/gsd-debug` — investigate a bug.
- `/gsd-progress` — what's the state of the project right now.

The auto-mode bootstrap from 2026-04-13 set `mode: yolo`, `granularity: coarse`, and enabled all workflow gates (research / plan_check / verifier / nyquist / code_review). Do not silently downgrade these.

### Push cadence — per-phase (codified 2026-04-30, closes backlog 999.4)

**Rule:** every phase closes with `git push origin main` BEFORE the verifier-subagent dispatch. Pre-push gate-passing is part of the phase-close criterion, not an end-of-session sweep.

- **Why:** v0.12.1's autonomous run accumulated 115 unpushed commits — drift compounded invisibly until session-end (P73/P75 fmt drift sat in 7 commits before pre-push caught it). Per-phase push closes the feedback loop while phase context is still warm. DVCS phases will be longer than v0.12.1's clusters; the same +N-stack pattern would compound 5-10×.
- **Operationally:** the executing subagent pushes inside the phase; if the gate blocks, treat it as a phase-internal failure (fix and re-push) — not a deferral. The verifier subagent grades RED if the phase shipped without the push landing.
- **Eager-resolution carve-out:** trivial in-phase chores (single-line typo fix, comment cleanup) discovered mid-phase do not require their own push round-trip — they ride to origin with the phase's terminal push.
- **Pre-commit fmt hook (a25f6ff)** stays on as the secondary safety net; it catches drift at commit time before the per-phase push has anything to discover.

- `#![forbid(unsafe_code)]` in every crate.
- `#![warn(clippy::pedantic)]` in every crate. Allow-list specific lints with rationale; never blanket-allow `pedantic`.
- All public items documented; missing-doc lint is on for `reposix-core`.
- All `Result`-returning functions have a `# Errors` doc section.
- Tests live next to the code (`#[cfg(test)] mod tests`). Integration tests in `tests/`.
- Frontmatter uses `serde_yaml` 0.9 + Markdown body. Never JSON-on-disk for issues.
- Times are `chrono::DateTime<Utc>`. No `SystemTime` in serialized form.

## Build memory budget (load-bearing — read before parallelizing)

**The VM has crashed twice from RAM pressure caused by parallel cargo workspace builds.** The reposix workspace links large crates (`gix` chain, `rusqlite-bundled`, `reqwest+rustls`); a single `cargo check --workspace` peaks rustc + linker around 4–6 GB; `cargo test --workspace` is worse because rustc + N test binaries all link in parallel by default.

**Hard rules — both apply, no exceptions without explicit user override:**

1. **Never run more than one cargo invocation at a time.** This includes `cargo check`, `cargo build`, `cargo test`, `cargo clippy`. If two subagents both need to compile, they run sequentially, not in parallel. Coordinator-level safe rule: at most one phase-executor subagent doing cargo work at a time. Doc-only / planning-only subagents can still run in parallel with one cargo subagent.
2. **Prefer per-crate over workspace-wide:** `cargo check -p reposix-cli` instead of `cargo check --workspace` whenever the change is scoped. The pre-push hook covers the workspace-wide gate; you don't need to re-run it locally per commit.

**Soft rules:**

- `cargo build --jobs 2` (or `CARGO_BUILD_JOBS=2`) is safer than the default `-j$(nproc)` on this machine — set it in `.cargo/config.toml` if you find yourself fighting OOM during a single workspace build.
- `cargo nextest run` materializes test binaries one at a time vs `cargo test`'s parallel link; prefer it for full-workspace test runs.
- Doc-site work (`mkdocs build`, playwright) is cheap on RAM but runs Chromium — combined with cargo, it adds another ~500 MB. Schedule docs work in a no-cargo window.
- `rust-analyzer` in an editor pinned to this repo can use 2–3 GB on its own; close non-active editor instances during heavy build sessions.

**If the VM crashes again:** suspect parallel cargo, suspect rust-analyzer, suspect leftover background processes (`ps aux | grep -E "cargo|rustc"`). Don't blame the linker; blame the orchestration that let two of them run at once.

## Docs-site validation

Any change to `mkdocs.yml` or `docs/**` MUST pass `bash scripts/check-docs-site.sh` before commit (pre-push enforces). Mermaid SVG assertions: `scripts/check-mermaid-renders.sh` (also pre-push). For playwright walk rules and scoping (which pages to re-check after a change), see `/reposix-quality-doc-alignment` skill.

## Cold-reader pass on user-facing surfaces

Before declaring any user-facing surface shipped (hero copy, install instructions, headline numbers, README, docs landing page), dispatch `/doc-clarity-review` on the affected pages. For automated rubric grading, use `/reposix-quality-review` (`--rubric <id>` / `--all-stale`). The catalog at `quality/catalogs/subjective-rubrics.json` enforces 30-day freshness TTL.

## Freshness invariants

All invariants are enforced by `scripts/end-state.py` (pre-push hook). When a push is blocked, read the error — it names the violated invariant and the fix. The invariants: no version-pinned filenames, install path leads with package manager, benchmarks in mkdocs nav, no loose ROADMAP/REQUIREMENTS outside `*-phases/`, no orphan docs.

## Release pipeline

**`release-plz.toml`** — `git_release_enable = false` at the workspace level so release-plz does NOT create per-package GitHub releases. Why: each per-package release (zero assets) was published *after* the canonical `v*` release and stole the `releases/latest` pointer, 404'ing the user-facing installer URLs and 3 catalog rows (`release/gh-assets-present`, `install/curl-installer-sh`, `install/powershell-installer-ps1`). Per-package tags and crates.io publishes are unaffected. The canonical multi-platform release lives at `.github/workflows/release.yml` (tag `v*`).

## Subagent delegation rules

Per the user's global OP #2: "Aggressive subagent delegation." Specifics for this project:

- `gsd-phase-researcher` for any "how do I implement X" question that would consume >100 lines of orchestrator context.
- `gsd-planner` for phase planning. Do not write `PLAN.md` by hand.
- `gsd-executor` for phase execution unless the work is trivially small.
- `gsd-code-reviewer` after every phase ships, before declaring done.
- Run multiple subagents in parallel whenever they're operating on disjoint files.
- **Never delegate `gh pr checkout` to a bash subagent without isolation.** Bash subagents share the coordinator's working tree; `gh pr checkout` switches the local branch behind the coordinator's back, which already caused the cherry-pick mess at commit `5a91ae2`. Either spawn a worktree first (`git worktree add /tmp/pr-N pr-N-branch`) and have the subagent `cd` into it, or have the subagent operate inside `/tmp/<branch>-checkout`. The coordinator's checkout is shared state — treat it that way.
- **Verifier subagent on every phase close** — see "Verifier subagent dispatch" in Quality Gates section + `quality/PROTOCOL.md` § "Verifier subagent prompt template".
- **Dispatching subjective rubrics (cold-reader, install-positioning, headline-numbers)** — `/reposix-quality-review` skill. Invocation: `bash .claude/skills/reposix-quality-review/dispatch.sh --rubric <id>` / `--all-stale` / `--force`. Path A (Task tool from Claude session) preferred for unbiased grading; Path B (claude -p subprocess) fallback.
- **Orchestration-shaped phases run at top-level, not under `/gsd-execute-phase`.** When a phase's work shape is "fan out → gather → interpret → resolve" rather than "write code → run tests → commit," the top-level coordinator IS the executor. `gsd-executor` lacks `Task` and depth-2 spawning is forbidden, so subagent fan-out cannot live inside it. Mark such phases `Execution mode: top-level` in ROADMAP and provide a research brief the orchestrator follows verbatim. Docs-alignment backfill is the canonical example; retroactive audits follow the same pattern. Refresh runs on stale docs (`/reposix-quality-refresh <doc>`) are also top-level — pre-push that BLOCKS mid-`gsd-execute-phase` must be resolved by checkpointing the executor and invoking the slash command from a fresh top-level session.

The orchestrator's job is to route, decide, and integrate — not to type code that a subagent could type.

### Meta-rule: when an owner catches a quality miss, fix it twice

When the owner catches a quality issue the agent missed, the fix is
two-fold: (1) fix the issue in the code/docs, and (2) update CLAUDE.md
(and/or the §0.8 SESSION-END-STATE framework) so the next agent's
session reads the tightened rule. Just shipping the fix without
updating the instructions guarantees recurrence.

## Threat model

This project is a textbook lethal-trifecta machine:

| Leg of trifecta | Where it shows up here |
| --- | --- |
| Private data | The partial-clone working tree exposes issue bodies, internal field values, attachments. |
| Untrusted input | Every issue body / comment / title is attacker-influenced text. |
| Exfiltration | `git push` can target arbitrary remotes; the helper + cache make outbound HTTP. |

Cuts that are mandatory and tested:

- **Outbound HTTP allowlist.** The remote helper (`git-remote-reposix`) and the cache materializer (`reposix-cache`) refuse to talk to any origin not in `REPOSIX_ALLOWED_ORIGINS` (env var, defaults to `http://127.0.0.1:*` only). All HTTP construction goes through the single `reposix_core::http::client()` factory; clippy's `disallowed_methods` catches direct `reqwest::Client::new()` call sites.
- **No shell escape from `export` / cache writes.** Bytes-in-bytes-out; no rendering, no template expansion. The `Tainted<T>` → `Untainted<T>` conversion is the explicit `sanitize` step where escaping happens.
- **Frontmatter field allowlist.** Server-controlled fields (`id`, `created_at`, `version`, `updated_at`) cannot be overridden by client writes; they are stripped on the inbound `export` path before the REST call.
- **Audit log is append-only — and dual-schema by design.** SQLite WAL, no UPDATE/DELETE on either audit table. `audit_events_cache` (in `reposix-cache::audit`) records every blob materialization, helper fetch turn, helper push (accept and reject), gc eviction, and sync-tag write. `audit_events` (in `reposix-core::audit`, written by the sim/confluence/jira adapters) records every backend-side mutating REST call. Forensic queries that need both layers (e.g., "which JIRA write came from which `git push`?") read both tables. See OP-3 above for why the split is intentional.

See `research/threat-model-and-critique.md` (produced by red-team subagent) for the full analysis.

## What to do when context fills

If you (the agent) notice this CLAUDE.md getting hard to keep in working memory:

1. Read `.planning/STATE.md` first — it's the entry point.
2. Read the most recent `.planning/phases/*/PLAN.md`.
3. Skim `git log --oneline -20` to know what's recently shipped.
4. Don't read this file linearly; grep for the section you need.

## Quality Gates — dimension/cadence/kind taxonomy

The Quality Gates framework lives at `quality/`. Runtime contract: `quality/PROTOCOL.md`. Catalog schema: `quality/catalogs/README.md`. Pivot journal: `quality/SURPRISES.md`. Historical build narrative: `.planning/milestones/v0.12.0-phases/archive/` (per-phase files).

Working on a quality-gates task? Read `quality/PROTOCOL.md` first.

**9 dimensions** — the regression classes the project has:

| Dimension | Checks | Home |
|---|---|---|
| code | clippy, fmt, cargo nextest | `quality/gates/code/` |
| docs-alignment | claims have tests; hash drift detection | `quality/gates/docs-alignment/` |
| docs-build | mkdocs strict, mermaid renders, link resolve, badges resolve | `quality/gates/docs-build/` |
| docs-repro | snippet extract, container rehearse, tutorial replay | `quality/gates/docs-repro/` |
| release | gh assets present, brew formula current, crates.io max version, installer bytes | `quality/gates/release/` |
| structure | freshness invariants, banned words, top-level scope | `quality/gates/structure/` |
| agent-ux | dark-factory regression (sim arm + DVCS third arm) + reposix-attach + bus URL prechecks + webhook YAML | `quality/gates/agent-ux/` |
| perf | latency, token economy | `quality/gates/perf/` |
| security | allowlist enforcement, audit immutability | `quality/gates/security/` |

**6 cadences** — when the gate runs:

`pre-push` (local, every push, <60s, blocking) · `pre-pr` (PR CI, <10min, blocking) · `weekly` (cron, alerting) · `pre-release` (on tag, <15min, blocking) · `post-release` (after assets ship, alerting) · `on-demand` (manual / subagent).

**5 kinds** — how the gate is verified:

`mechanical` (deterministic shell + asserts) · `container` (fresh ubuntu/alpine + post-conditions) · `asset-exists` (HEAD/GET URL + min-bytes) · `subagent-graded` (rubric-driven subagent) · `manual` (human-only with TTL freshness).

**Adding a new gate** is one catalog row + one verifier in the right dimension dir. The runner discovers + composes by tag. No new top-level script, no new pre-push wiring.

**Catalog-first rule.** Every phase's FIRST commit writes the catalog rows that define this phase's GREEN contract; subsequent commits cite the row id. The verifier subagent reads catalog rows that exist BEFORE the implementation lands.

**Verifier subagent dispatch.** Phase close MUST dispatch an unbiased subagent that grades catalog rows from artifacts with zero session context. The executing agent does NOT grade itself — see `quality/PROTOCOL.md` § "Verifier subagent prompt template" for the verbatim prompt.

**CLAUDE.md stays current.** Each phase introducing a new file/convention/gate updates CLAUDE.md in the same PR. The update means *revising existing sections* to reflect the new state — not appending a narrative. Ask: "if an agent opens this file cold, what do they need to know?" and edit the document to say that, wherever it naturally fits.

**Meta-rule extension** (when an owner catches a quality miss): fix the issue, update CLAUDE.md, AND **tag the dimension**. The dimension tag routes to the right catalog file + `quality/gates/<dim>/` verifier directory.

### Docs-alignment dimension

Binary: `reposix-quality doc-alignment {bind, propose-retire, confirm-retire, mark-missing-test, plan-refresh, plan-backfill, merge-shards, walk, status}`. Run `status --top 10` for gap targeting. Pre-push gate: `quality/gates/docs-alignment/walk.sh`. Full spec: `quality/catalogs/README.md` § "docs-alignment dimension".

Two axes: `alignment_ratio` (bound / non-retired) and `coverage_ratio` (lines_covered / total_eligible). Walker BLOCKs when either drops below floor; recovery is `/reposix-quality-backfill`.

**Slash commands** (top-level only — unreachable from inside `gsd-executor`):
- `/reposix-quality-refresh <doc>` — refresh stale rows for one doc.
- `/reposix-quality-backfill` — full extraction across docs/ + archived REQUIREMENTS.md.

## v0.12.1 — in flight

### P72 — Lint-config invariants

Verifier home: `quality/gates/code/lint-invariants/` (sub-area README at `quality/gates/code/lint-invariants/README.md`). 8 shell verifiers bind 9 `MISSING_TEST` rows in `quality/catalogs/doc-alignment.json` covering README + `docs/development/contributing.md` workspace-level invariants.

| Verifier                    | Catalog row(s)                                                                                          |
| --------------------------- | ------------------------------------------------------------------------------------------------------- |
| `forbid-unsafe-code.sh`     | `README-md/forbid-unsafe-code` + `docs-development-contributing-md/forbid-unsafe-per-crate` (D-01) |
| `rust-msrv.sh`              | `README-md/rust-1-82-requirement`                                                                       |
| `tests-green.sh`            | `README-md/tests-green` (compile-only, D-05)                                                            |
| `errors-doc-section.sh`     | `docs-development-contributing-md/errors-doc-section-required` (clippy lint, D-07)                      |
| `rust-stable-channel.sh`    | `docs-development-contributing-md/rust-stable-no-nightly`                                               |
| `cargo-check-workspace.sh`  | `docs-development-contributing-md/cargo-check-workspace-available`                                      |
| `cargo-test-count.sh`       | `docs-development-contributing-md/cargo-test-133-tests` (>= 368 floor, re-measured P72 per D-06)        |
| `demo-script-exists.sh`     | `docs-development-contributing-md/demo-script-exists`                                                   |

Prose updated: `docs/development/contributing.md:20` re-measured to `>= 368 test binaries` BEFORE the bind (D-06 audit trail). Verifiers minted via `bind` per `quality/PROTOCOL.md` § "Subagents propose; tools validate and mint" (Principle A). Cargo invocations serialized by the runner per CLAUDE.md "Build memory budget" (D-04).

### P73 — Connector contract gaps

Closed 4 `MISSING_TEST` rows asserting connector authoring + JIRA-shape contracts. Two new wiremock-based Rust tests live next to existing per-crate contract tests:

- `crates/reposix-confluence/tests/auth_header.rs::auth_header_basic_byte_exact`
- `crates/reposix-github/tests/auth_header.rs::auth_header_bearer_byte_exact`
- `crates/reposix-jira/tests/list_records_excludes_attachments.rs::list_records_excludes_attachments_and_comments`

The auth-header tests use `wiremock::matchers::header(K, V)` (returns `HeaderExactMatcher`, NOT `HeaderRegexMatcher`) for byte-exact assertion — the canonical idiom for any future connector contract test of this kind. Plan-time prose cited `header_exact` as the function name; the actual public API in wiremock 0.6.5 is `header(K, V)` (same byte-exact semantics). The JIRA test asserts at the **rendering boundary** (Record.body markdown + Record.extensions allowlist), not at the JSON parse layer — that's where the deferral in `docs/decisions/005-jira-issue-mapping.md:79-87` is observable to a downstream consumer.

The `real-backend-smoke-fixture` row was a pure rebind to the existing `crates/reposix-cli/tests/agent_flow_real.rs::dark_factory_real_confluence` `#[ignore]` smoke (TokenWorld is sanctioned for free mutation per `docs/reference/testing-targets.md`).

The stale `docs/benchmarks/token-economy.md:23-28` JIRA row was resolved via path (a) per D-05: prose updated to acknowledge the adapter shipped v0.11.x, plus a 5-line shell verifier at `quality/gates/docs-alignment/jira-adapter-shipped.sh` asserting the manifest exists. Bench-number re-measurement remains deferred to perf-dim P67.

No SURPRISES-INTAKE / GOOD-TO-HAVES entries appended (no out-of-scope items observed during execution; no auth-header bug surfaced in either backend; OP-8 honesty check intentional).

See: `quality/PROTOCOL.md` § "Principle A"; CLAUDE.md "Build memory budget" (D-09 sequential per-crate cargo).

### P74 — Narrative cleanup + UX bindings + linkedin prose fix

Closed the docs-alignment narrative+UX cluster: 4 propose-retires (qualitative
design rows), 5 hash-shape binds (UX claims on docs/index.md + REQUIREMENTS
rows), and a one-line linkedin.md prose fix dropping the v0.4-era FUSE
framing. Five new shell verifiers under `quality/gates/docs-alignment/`
(FLAT placement, sibling of `jira-adapter-shipped.sh`):

- `install-snippet-shape.sh` — asserts `docs/index.md:19` lists curl/brew/cargo binstall/irm.
- `audit-trail-git-log.sh` — asserts `git log --oneline -n 1` returns >=1 line.
- `three-backends-tested.sh` — counts `dark_factory_real_*` fns in `agent_flow_real.rs`; asserts >=3.
- `connector-matrix-on-landing.sh` — greps `docs/index.md` for `## ...connector|backend` heading + table.
- `cli-spaces-smoke.sh` — asserts `target/release/reposix spaces --help` exits 0 + expected header.

Each verifier is 10-30 lines (D-02 TINY shape). Body-hash drift on prose
OR verifier file fires `STALE_DOCS_DRIFT` via the walker. No deep workflow
logic. Bind sweep promoted to `scripts/p74-bind-ux-rows.sh` (CLAUDE.md §4
"Ad-hoc bash is a missing-tool signal").

Catalog deltas: `claims_missing_test` 9 -> 0 (5 BOUND + 4 RETIRE_PROPOSED
awaiting owner-TTY confirm-retire per HANDOVER step 1); `claims_bound`
324 -> 328 (+4 net; +5 binds offset by linkedin row tipping to
STALE_DOCS_DRIFT); `alignment_ratio` 0.9050 -> 0.9162. Linkedin row's
post-edit STALE_DOCS_DRIFT did NOT auto-heal on second walk — logged to
SURPRISES-INTAKE.md as confirmation of the P75 hash-overwrite bug.

No new test files in `crates/` (D-10). Phase is shell + prose + catalog only.

### P75 — bind-verb hash-overwrite fix

Invariant: `row.source_hash == hash(first source)`. `verbs::bind` previously
overwrote `source_hash` with the newly-cited source's hash on every re-bind,
breaking the walker's first-source compare on every Single→Multi promotion
(false `STALE_DOCS_DRIFT` after every cluster sweep). Fix: refresh
`source_hash` only when the result is `Source::Single`; preserve it on
`Multi` paths. Single re-bind with the same citation is the heal path
(P74 linkedin row).

**Path-(a) tradeoff (closed in P78-03):** path (a) — the walker only watches
`source.as_slice()[0]`, so drift in non-first sources of a `Multi` row does
not fire `STALE_DOCS_DRIFT` — was the v0.12.1 P75 shape. Path (b) — parallel
`source_hashes: Vec<String>` + per-source walker AND-compare — closed in
v0.13.0 P78-03 (commit `28ed9be`); non-first-source drift now fires
`STALE_DOCS_DRIFT` per the regression test
`crates/reposix-quality/tests/walk.rs::walk_multi_source_non_first_drift_fires_stale`.
Legacy multi-source rows backfilled at load: `source_hashes` left empty
("no-hash-recorded-yet" semantic) until the next bind heals the row through
the P78-aware path. Backfill of single-source legacy rows is automatic
(`source_hash` → `source_hashes[0]`).

Regression tests: `crates/reposix-quality/tests/walk.rs::walk_multi_source_*`
(stable / first-drift / single-rebind-heal).

### P76 — Surprises absorption

Drained `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md` (3 LOW
entries discovered during P72 + P74). The +2 phase practice (OP-8) is now
operational: every intake entry has a terminal STATUS footer.

Resolutions:
- **Entry 1 (P72):** 2 pre-existing STALE rows healed.
  polish-03-mermaid-render → RESOLVED | 0467373 (rebind, source_hash
  c88cd0f9 → 6ec37650). cli-subcommand-surface → RESOLVED | fbc3caa
  (rebind, b9700827 → 89b925f5). Both claims verified verbatim against
  current source via `sed`; no propose-retire needed.
- **Entry 2 (P74):** linkedin Source::Single → RESOLVED | healed by P75
  commit 9e07028 (audit-trail annotation only; row already BOUND).
- **Entry 3 (P74):** connector-matrix synonym → WONTFIX | regex widening
  (c8e4111) is the complete fix; heading rename filed as P77 GOOD-TO-HAVE
  (size XS, impact clarity).

Honesty spot-check (D-05): sampled P74 + P75 plan/verdict pairs. Aggregate
finding GREEN — intake yield (P72: 1, P74: 2, P73: 0, P75: 0) is consistent
with phases honestly looking. P74's verifier independently graded OP-8
PASS; P75's verifier executable-cross-checked the falsifiable empty-intake
claim. Evidence at `quality/reports/verdicts/p76/honesty-spot-check.md`.

Catalog deltas: claims_bound 329 → 331 (+2 entry-1 rebinds);
alignment_ratio 0.9190 → 0.9246 (+0.0056); claims_stale_docs_drift 2 → 0.
Live walker post-resolution: zero net new STALE_DOCS_DRIFT.

### P77 — good-to-haves polish (closed)

P77 closed GOOD-TO-HAVES-01 by draining the v0.12.1 intake (1 XS
clarity item discovered by P74).

Closure: docs/index.md:95 heading renamed "What each backend can do"
→ "Connector capability matrix" (5f3a6fc); verifier regex narrowed
back to literal `[Cc]onnector` (fb8bd28), reversing P74's eager-widen
(c8e4111). Walk + rebind round-trip in 4ac9206.

Walk-after verdict: `quality/reports/verdicts/p77/walk-after.txt`.
polish2-06-landing remains BOUND; alignment_ratio unchanged at
0.9246; zero new STALE rows.

**D-09 load-bearing note:** P77 is the LAST phase of the v0.12.1
autonomous run. HANDOVER-v0.12.1.md is intentionally LEFT IN PLACE at
P77 close. Its self-deletion criteria (HANDOVER §"Cleanup criterion")
require all 6 phases verifier-GREEN AND owner pushed v0.12.0 tag AND
owner confirmed retires AND v0.12.1 milestone-close verdict GREEN —
only criterion 1 is true at P77 close. The session-end commit that
removes HANDOVER-v0.12.1.md is an orchestrator-level action OUTSIDE
the phase, written by the top-level coordinator after the verifier
subagent grades P77 GREEN.

See `quality/reports/verdicts/p77/VERDICT.md` for unbiased grading.

## v0.13.0 — DVCS over REST (SHIPPED 2026-05-01, owner tag-cut pending)

Eleven phases (P78–P88) shipped the thesis-shifting DVCS topology: confluence (or any one issues backend) remains source-of-truth; a plain-git GitHub mirror becomes the universal-read surface for vanilla-git consumers; `reposix attach` reconciles existing checkouts against the SoT; `git push` via a bus remote (`reposix::<sot>?mirror=<mirror-url>`) fans out atomically to SoT-first then mirror-best-effort. P79 attach + P80 mirror-lag refs + P81 L1 perf + P82 bus URL parser + P83 bus write fan-out + P84 webhook sync + P85 DVCS docs + P86 dark-factory third arm. P87 + P88 = +2 reservation slots (surprises absorption + good-to-haves polish).

**Milestone-close ratification:** `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` (dispatched after P88 push). RETROSPECTIVE distillation: `.planning/RETROSPECTIVE.md § "Milestone: v0.13.0 — DVCS over REST"` (OP-9 ritual; What Was Built / What Worked / What Was Inefficient / Patterns Established / Key Lessons + 6 distilled cross-phase lessons + carry-forward bundle to v0.14.0).

**Tag-cut:** owner runs `bash .planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` (8 guards) then `git push origin v0.13.0`. Orchestrator does NOT push the tag (ROADMAP P88 SC6 — STOP at tag boundary).

**Carry-forward to v0.14.0:**
- DVCS-CF-01 binstall + yanked-gix release substrate (HIGH; P84 SURPRISES Entry 5).
- DVCS-CF-02 GOOD-TO-HAVES-01 — extend `reposix-quality bind` to all catalog dimensions.
- DVCS-CF-03 L2/L3 cache-desync hardening (P81 deferral).
- CLAUDE.md sign-posting for cargo-test-as-verifier shape.

## Quick links

- `docs/research/initial-report.md` — full architectural argument for git-remote-helper + partial clone.
- `docs/research/agentic-engineering-reference.md` — dark-factory pattern, lethal trifecta, simulator-first.
- `docs/reference/testing-targets.md` — sanctioned real-backend test targets (TokenWorld, `reubenjohn/reposix`, JIRA `TEST`).
- `docs/benchmarks/latency.md` — golden-path latency envelope per backend.
- `docs/concepts/dvcs-topology.md` (P85) — three roles (SoT-holder / mirror-only consumer / round-tripper) + mirror-lag refs explained; the canonical DVCS mental model.
- `docs/guides/dvcs-mirror-setup.md` (P85) — owner walk-through for webhook + GH Action setup; cron-only fallback (Q4.2); cleanup procedure.
- `docs/guides/troubleshooting.md` § "DVCS push/pull issues" (P85) — bus `fetch first` rejections, attach reconciliation warnings, webhook race conditions, cache-desync recovery.
- `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` — ratified design doc for the v0.9.0 pivot.
- `.planning/PROJECT.md` — current scope.
- `.planning/STATE.md` — current cursor.
