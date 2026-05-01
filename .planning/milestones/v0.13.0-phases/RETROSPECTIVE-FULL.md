# v0.13.0 — DVCS over REST (full retrospective)

Distilled lessons live in `.planning/RETROSPECTIVE.md`. This file holds the
full narrative (What Was Built, Cost Observations, verbose Carry-forward
prose) that was trimmed from RETROSPECTIVE.md per the file-size budget.

**Shipped:** 2026-05-01 (autonomous run; owner-driven tag pending)
**Phases:** 11 (P78–P88) | **Plans:** 14 | **Sessions:** multi-session autonomous

The thesis-shifting milestone: from "VCS over REST" (one developer, one backend) to "DVCS over REST" — confluence (or any one issues backend) remains source-of-truth, but a plain-git mirror on GitHub becomes the universal-read surface. Devs `git clone git@github.com:org/repo.git` with vanilla git (no reposix install), get all markdown, edit, commit. Install reposix only to write back; `reposix attach` reconciles the existing checkout against the SoT, then `git push` via a bus remote fans out atomically to confluence (SoT-first) and the GH mirror.

## What Was Built

- **`reposix attach <backend>::<project>` subcommand (P79)** — adopt an existing checkout (vanilla GH-mirror clone, hand-edited tree, prior `reposix init`) and bind it to a SoT backend. Builds the cache from REST, walks the working-tree HEAD, reconciles records by frontmatter `id` (5 cases per architecture-sketch). New `cache_reconciliation` table; audit-row trail via `audit_events_cache op = 'attach_walk'`. Real GH mirror endpoint wired at `reubenjohn/reposix-tokenworld-mirror`.
- **Mirror-lag refs `refs/mirrors/<sot-host>-{head,synced-at}` (P80)** — observability via plain-git refs that vanilla `git fetch` brings along. Three TINY shell verifiers as thin wrappers over `cargo test -p reposix-remote --test mirror_refs <name>` (the layered coverage shape that became the sanctioned house pattern).
- **L1 perf migration (P81)** — replaced full `list_records` walk with `list_changed_since`-based conflict detection in `handle_export`. New `reposix sync --reconcile` cache-desync escape hatch. The success branch's `refresh_for_mirror_head` is no-op-skipped when `files_touched == 0` (eager-resolution patch); perf row asserts ZERO `list_records` calls on the hot push path.
- **Bus remote URL parser + cheap prechecks (P82)** — `reposix::<sot>?mirror=<mirror-url>` form. Precheck A (mirror drift via `ls-remote`) and B (SoT drift via `list_changed_since`) bail before reading stdin. Capability branching: bus URLs omit `stateless-connect` advertisement.
- **Bus remote write fan-out (P83)** — SoT-first algorithm with mirror-best-effort fallback. NEW audit op `helper_push_partial_fail_mirror_lag`. Full fault-injection coverage (SoT mid-stream fail, post-precheck 409, mirror fail). Fixture-fix in P83-02 immunizes shell-hook fault injection from user-global `core.hooksPath`.
- **Webhook-driven mirror sync (P84)** — `.github/workflows/reposix-mirror-sync.yml` template + live copy on `reubenjohn/reposix-tokenworld-mirror`. `--force-with-lease` race protection + first-run handling. Owner-runnable `scripts/webhook-latency-measure.sh --synthetic`.
- **DVCS docs (P85)** — `docs/concepts/dvcs-topology.md` + `docs/guides/dvcs-mirror-setup.md` + troubleshooting matrix entries. Cold-reader pass via `/doc-clarity-review`.
- **Dark-factory third-arm regression (P86)** — `dvcs-third-arm` scenario in `scripts/dark-factory-test.sh`: vanilla-clone + attach + bus-push at the agent-UX surface. 17 asserts. Layered coverage: shell harness for agent UX surface + cargo tests for wire path. TokenWorld arm SUBSTRATE-GAP-DEFERRED.
- **Pre-DVCS hygiene (P78)** — gix bumped from yanked `=0.82.0` to `=0.83.0` (closes upstream gix #29 + #30). Three WAIVED structure rows resolved before TTL. MULTI-SOURCE-WATCH-01 walker schema migration: `source_hashes: Vec<String>` parallel field + per-source AND-compare closes the v0.12.1 P75 path-(a) tradeoff.
- **+2 reservation slots operational (P87, P88)** — P87 drained 5 SURPRISES-INTAKE entries with terminal STATUS + verifier honesty spot-check sampling 5 phases (exceeded the >=3 floor). P88 drained 1 GOOD-TO-HAVES entry (DEFERRED to v0.14.0 with rationale).

## Cost Observations

- Model: claude-opus-4-7[1m] (1M context, milestone-close + several phases)
- Mid-milestone phase-execution model: claude-sonnet-4-5 (per-phase work)
- Sessions: multi-session autonomous (P78–P88 spread across 2026-04-30 → 2026-05-01)
- Notable: per-phase push cadence kept the unpushed-stack from accumulating; pre-push gate caught fmt drift in the discovering phase rather than at session-end. v0.12.1's 115-commit-stack failure mode did not recur. CHANGELOG entry length (~30 non-blank lines) suggests milestones with broader scope should consider a "see RETROSPECTIVE.md" callout to keep CHANGELOG skimmable.

## Carry-forward to v0.14.0 (verbose)

See `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` for the canonical list. The condensed entries below were trimmed from RETROSPECTIVE.md:

- **DVCS-CF-01 — binstall + yanked-gix release substrate** (P84 SURPRISES-INTAKE Entry 5; severity HIGH). Cutting v0.13.x with non-yanked `gix = "=0.83.x"` (P78 already bumped the workspace pin) AND confirming `.github/workflows/release.yml` produces per-target binstall tarballs unblocks both legs of the webhook setup-guide install path. Owner-runnable `scripts/webhook-latency-measure.sh --synthetic` ready for re-measurement once v0.13.x ships.
- **DVCS-CF-02 — extend `reposix-quality bind` to all catalog dimensions** (P88 GOOD-TO-HAVES-01 DEFERRED). Closes the cleaner Principle A provenance story; today every non-`docs-alignment` row carries `_provenance_note: "Hand-edit per documented gap (NOT Principle A)"`. Operationally tolerable; provenance flag + audit trail intact. ~30-50 lines Rust + tests + cross-dimension schema design.
- **DVCS-CF-03 — L2/L3 cache-desync hardening** (P81 deferral per `architecture-sketch.md` § "Performance subtlety"). L1 ships in v0.13.0 (the `list_changed_since` precheck + the `refresh_for_mirror_head` no-op skip). L2 (background async cache rebuild on detect) and L3 (cache-vs-SoT divergence audit) defer to v0.14.0 alongside the observability dashboards.
- **CLAUDE.md "Subagent delegation rules" sign-posting for cargo-test-as-verifier shape.** The pattern is sanctioned (P80 → P86 trail) but not yet explicitly named in CLAUDE.md as the default for env-propagation-sensitive surfaces. Future planners might benefit from a "before proposing `reposix init`+`git fetch`+`git push` end-to-end shells, check CLAUDE.md § Quality Gates layered-coverage default" callout.
