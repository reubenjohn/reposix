# Phase 36: FUSE deletion + CLAUDE.md update + `reposix-agent-flow` skill + final integration tests + release - Context

**Gathered:** 2026-04-24
**Status:** Ready for planning
**Mode:** Auto-generated (discuss skipped via workflow.skip_discuss=true)

<domain>
## Phase Boundary

Demolish FUSE entirely and ship v0.9.0. Three things land in lockstep — there can be no window where they're out of sync:

1. **Code deletion** — `crates/reposix-fuse/` removed, `fuser` dependency purged, `fuse-mount-tests` feature gate removed, CI `apt install fuse3` step dropped.
2. **CLAUDE.md rewrite** — replace the v0.9.0-in-progress banner with steady-state "Architecture (git-native partial clone)" content. FUSE references purged from elevator pitch, Operating Principles, Workspace layout, Tech stack, Commands, Threat model.
3. **Skill + release infrastructure** — `.claude/skills/reposix-agent-flow/SKILL.md` ships (the dark-factory regression skill that Phase 35's test enables). `scripts/tag-v0.9.0.sh` ships. CHANGELOG `[v0.9.0]` is finalized. CI jobs `integration-contract-{confluence,github,jira}-v09` wired.

Per OP-4 ("Self-improving infrastructure"): agent grounding must match shipped reality. There can be no window where CLAUDE.md describes deleted code (fail-stop for any orchestrator), and no window where the project lacks the dark-factory regression skill that v0.9.0's architecture is supposed to enable.

Per OP-5 ("Reversibility enables boldness"): execute via `gsd-pr-branch` or worktree so a botched FUSE deletion can be reverted in one move.

</domain>

<decisions>
## Implementation Decisions

### Operating-principle hooks (non-negotiable — per project CLAUDE.md)

- **Self-improving infrastructure (OP-4).** CLAUDE.md + `reposix-agent-flow` skill ship in the same phase as FUSE deletion. NOT a follow-up. The grounding must already match reality at the moment FUSE goes away.
- **Close the feedback loop (OP-1).** `gh run view` on the release tag must show green CI without the `apt install fuse3` step. Without seeing green, "released" is a hallucination.
- **Reversibility enables boldness (OP-5).** Execute via `gsd-pr-branch` or worktree so a botched FUSE deletion can be reverted in one move.
- **Audit log non-optional (OP-3).** Real-backend CI jobs write audit rows; the artifact upload from those jobs includes the audit-log dump for diffability.
- **Real-backend testing targets (OP-6).** Three CI jobs (TokenWorld / `reubenjohn/reposix` / JIRA `TEST`) wired here per ARCH-19. `pending-secrets` status when creds unavailable; green when present.

### Explicit DELETE list

- **`crates/reposix-fuse/`** — entire directory recursively. After deletion, `cargo metadata --format-version 1 | grep -i fuser` returns empty.
- **`fuser` dependency** — remove from every `Cargo.toml` in the workspace. Assertable: `grep -r '\bfuser\b' Cargo.toml crates/*/Cargo.toml` returns empty.
- **`fuse-mount-tests` feature gate** — remove from any `[features]` block where it appears, plus any `#[cfg(feature = "fuse-mount-tests")]` annotations.
- **CI `apt install fuse3`** — drop from `.github/workflows/*.yml`. Drop `/dev/fuse` mount requirements. Drop `--features fuse-mount-tests` test-step invocations.
- **`crates/reposix-cli/src/mount.rs`** — Phase 35 replaced its body with a migration-error stub; Phase 36 deletes the file. Remove `mod mount;` from `lib.rs`/`main.rs`.
- **All FUSE-only integration tests** — delete the `tests/` files gated on `fuse-mount-tests`.
- **README / docs FUSE references** — purge any remaining `mount`-era code samples (Phase 30 docs are deferred to v0.10.0, so README is the only user-facing doc requiring same-phase update).

### Explicit UPDATE list (CLAUDE.md)

Per ARCH-14:

- **Banner removal** — the existing `## Architecture transition (v0.9.0 — in progress)` banner (added by Phase 30 v0.9.0-pivot research commit) is replaced with a steady-state `## Architecture (git-native partial clone)` section describing cache + helper + agent UX flow.
- **Elevator pitch** — purge "exposes REST-based issue trackers as a POSIX directory tree via FUSE" → "exposes REST-based issue trackers as a git-native partial clone, served by `git-remote-reposix` from a local bare-repo cache built from REST responses."
- **Operating Principles** — update OP #1 (simulator is default backend — keep), OP #5 ("Mount point = git repo" → restate as "Working tree IS a real git checkout — `.git/` is real, not synthetic; `git diff` is the change set, by construction, not by emulation").
- **Workspace layout** — remove `reposix-fuse/` row; add `reposix-cache/` row.
- **Tech stack** — remove the `fuser` row entirely; add a row stating `git >= 2.34` is a runtime requirement (architecture-pivot-summary §7 risks).
- **Commands you'll actually use** — remove the FUSE block (`cargo run -p reposix-fuse`, `cargo test -p reposix-fuse --features fuse-mount-tests`); replace with the new commands: `reposix init`, `git clone reposix::sim/proj-1 /tmp/repo`. Add a "Testing against real backends" sub-block naming TokenWorld / `reubenjohn/reposix` / JIRA `TEST` with env-var setup, cross-referenced to `docs/reference/testing-targets.md`.
- **Threat model** — update the table: the helper (NOT the FUSE daemon) is now the egress surface. Allowlist `REPOSIX_ALLOWED_ORIGINS` applies to the helper + cache, not the FUSE daemon. The "No shell escape from FUSE writes" row becomes "No shell escape from `export` / cache writes — bytes-in-bytes-out, no template expansion."
- `git grep -i 'fuser\|fusermount\|fuse-mount-tests\|reposix mount' CLAUDE.md` MUST return empty.

### Explicit CREATE list

- **`.claude/skills/reposix-agent-flow/SKILL.md`** — Claude Code skill convention: directory + `SKILL.md` with YAML frontmatter:
  ```yaml
  ---
  name: reposix-agent-flow
  description: Spawn the reposix dark-factory regression — a fresh subprocess agent given only `reposix init` and a goal must complete clone+grep+edit+commit+push using pure git/POSIX, including the conflict-rebase and blob-limit recovery cycles. Invoked from CI release-gate and from local dev (/reposix-agent-flow).
  ---
  ```
  Body: documents the test pattern, references `architecture-pivot-summary.md §4 "Agent UX: pure git, zero in-context learning"`, links Phase 35's `scripts/dark-factory-test.sh`, names the canonical test targets (TokenWorld / `reubenjohn/reposix` / JIRA `TEST`).
- **`scripts/tag-v0.9.0.sh`** — mirrors `scripts/tag-v0.8.0.sh`. Six safety guards minimum: clean tree, on `main`, version match in `Cargo.toml`, CHANGELOG `[v0.9.0]` exists, tests green, signed tag.
- **CHANGELOG `[v0.9.0]` finalized** — all six phases (31–36) summarized + breaking-change migration note (`reposix mount` → `reposix init`) + reference to `docs/reference/testing-targets.md`.
- **CI workflow updates** — three new jobs `integration-contract-confluence-v09`, `integration-contract-github-v09`, `integration-contract-jira-v09` per ARCH-19. Pattern mirrors existing `integration-contract-confluence` from Phase 11. Each job runs the ARCH-16 smoke suite + uploads latency rows from ARCH-17 as run artifacts.
- **`docs/benchmarks/v0.9.0-latency.md`** — Phase 35 created the file with a sim column; Phase 36 adds at least one real-backend column populated from the integration-contract job artifacts.

### Test surface

- `cargo_metadata_no_fuser` — `cargo metadata --format-version 1` output contains no package named `fuser`.
- `grep_workspace_no_fuser` — `grep -r '\bfuser\b' Cargo.toml crates/*/Cargo.toml` returns empty.
- `cargo_check_clippy_clean` — `cargo check --workspace && cargo clippy --workspace --all-targets -- -D warnings` clean.
- `claude_md_no_fuse` — `git grep -i 'fuser\|fusermount\|fuse-mount-tests\|reposix mount' CLAUDE.md` returns empty.
- `skill_exists_and_has_frontmatter` — `.claude/skills/reposix-agent-flow/SKILL.md` exists and parses as valid YAML frontmatter with `name:` and `description:`.
- `tag_script_has_six_guards` — `scripts/tag-v0.9.0.sh` parsed for the six safety checks.
- `changelog_has_v090` — `[v0.9.0]` section present + migration note + six-phase summary.
- `dark_factory_passes_post_deletion` — Phase 35's test re-runs against the FUSE-deleted codebase; green.
- `ci_jobs_present` — `.github/workflows/*.yml` contains the three `integration-contract-*-v09` job IDs.
- `benchmark_has_real_backend_column` — `docs/benchmarks/v0.9.0-latency.md` parses as a Markdown table with ≥1 real-backend column (TokenWorld OR `reubenjohn/reposix` OR JIRA `TEST`).

### Claude's Discretion

Whether the CHANGELOG `[v0.9.0]` section is a single big section or broken up by phase is at Claude's discretion. The migration note must be unmissable; everything else is style.

The exact text of the `reposix-agent-flow` skill body (beyond the locked frontmatter) is at Claude's discretion. Must reference architecture-pivot-summary §4 explicitly and link to `scripts/dark-factory-test.sh`.

Whether the worktree/branch strategy is a feature branch + PR or a worktree + squash-merge is at Claude's discretion (per OP-5: pick whichever is most easily revertible).

</decisions>

<code_context>
## Existing Code Insights

### Reusable assets

- `scripts/tag-v0.8.0.sh` — template for `tag-v0.9.0.sh`. Existing six-guard pattern matches what ARCH requires.
- `scripts/tag-v0.6.0.sh`, `tag-v0.5.0.sh`, `tag-v0.4.0.sh`, etc. — earlier examples for shape comparison.
- Existing `integration-contract-confluence` GitHub Actions job (from Phase 11) — template for the three new `-v09` jobs.
- `CLAUDE.md` current structure — read it as the diff target. Sections to mutate (in order): elevator pitch, Operating Principles, Workspace layout, Tech stack, Commands, Threat model. The `## What to do when context fills` and `## Quick links` sections at the bottom should also be sanity-checked for stale FUSE references.
- Existing `.claude/` directory layout — `settings.json` and `worktrees/` exist; `.claude/skills/` does NOT yet exist and will be created by this phase. Confirmed via `ls /home/reuben/workspace/reposix/.claude/`.
- Phase 30 narrative-vignettes notes (deferred to v0.10.0) — informs the "Architecture (git-native partial clone)" prose tone but does NOT block this phase.

### Established patterns

- Tag scripts use bash `set -euo pipefail` + sequential safety checks with explicit error messages.
- CHANGELOG entries are reverse-chronological with `[vX.Y.Z] - YYYY-MM-DD` headers.
- CI integration jobs use `pending-secrets` status pattern (existing Phase 11 / Phase 28 idiom).
- Skills directory convention: `.claude/skills/<name>/SKILL.md` with YAML frontmatter `name:` and `description:`. Confirmed against Anthropic Claude Code skill format.

### Integration points

- Phase 35's `scripts/dark-factory-test.sh` is the entry point the new skill invokes.
- Phase 35's `docs/reference/testing-targets.md` is the canonical link target from CLAUDE.md's new "Testing against real backends" sub-block.
- Phase 35's `docs/benchmarks/v0.9.0-latency.md` is finalized here with real-backend columns.

</code_context>

<specifics>
## Specific Ideas

- After `crates/reposix-fuse/` is removed, `Cargo.toml`'s `[workspace] members = [...]` array shrinks by one entry. Confirm `cargo check --workspace` after the edit.
- The CLAUDE.md threat-model table column "Where it shows up here" needs the FUSE row replaced — the helper is now the egress surface; the cache materializer is the second egress. Both honour `REPOSIX_ALLOWED_ORIGINS`.
- The `reposix-agent-flow` skill MUST mention it is the StrongDM/dark-factory regression harness for the v0.9.0 architecture (per ARCH-15) and reference architecture-pivot-summary §4.
- `tag-v0.9.0.sh` should mirror the existing tag-v0.8.0.sh checks plus a new check: `docs/reference/testing-targets.md` exists (so we never tag a release that lost the test-target documentation).
- CI matrix entries for the three `-v09` jobs use distinct secret blocks (`secrets.GITHUB_TOKEN_REPOSIX_TEST`, `secrets.ATLASSIAN_*`, `secrets.JIRA_*`); job runs `pending-secrets` when its block is absent.
- `docs/benchmarks/v0.9.0-latency.md` regressions are flagged inline in the Markdown (a "Regression?" column with explicit emoji or text marker — Claude's discretion). Not CI-blocking per ARCH-17.
- The skill's `description:` field must fit on one line (Claude Code convention) and mention "dark-factory regression test" verbatim so it's discoverable via the skill picker.

</specifics>

<deferred>
## Deferred Ideas

- Phase 30 docs rewrite (DOCS-01..09) — v0.10.0. CLAUDE.md changes here are a stop-gap; full doc-site rewrite is later.
- `import` capability removal — kept one release cycle past v0.9.0; CHANGELOG announces deprecation, actual removal in v0.10.0.
- Background reconciler for the REST→cache divergence window — v0.10.0+ (architecture-pivot-summary §7 Q2).
- Threat-model write-up update (`research/threat-model-and-critique.md`) for the helper-as-egress-surface model — kicked to v0.10.0 per architecture-pivot-summary §7 Q3.
- `reposix gc` for cache eviction — v0.10.0+ observability/maintenance milestone.
- Spawned-subprocess-Claude agent harness for the dark-factory test — v0.10.0 (Phase 35 ships the simpler scripted version).

</deferred>
</content>
