---
phase: 36
status: passed
verifier: phase-runner
date: 2026-04-24
---

# Phase 36 — Verification Report

## Status

**`passed`** — every acceptance gate from the 36-CONTEXT explicit DELETE /
UPDATE / CREATE lists is verifiable in the committed tree on `main`.
Real-backend CI gates (`integration-contract-{confluence,github,jira}-v09`)
remain `pending-secrets` until the repo owner adds the secret packs; this
inherits Phase 35's known status.

## Gates

| # | Check | Status | Evidence |
|---|-------|--------|----------|
| 1 | `cargo metadata --format-version 1 --no-deps` workspace member count == 9 (no `reposix-fuse`) | passed | `python3 -c '...len(workspace_members)'` → `9 members` |
| 2 | `grep -rE 'reposix-fuse\|fuser ' Cargo.toml crates/*/Cargo.toml` empty | passed | grep rc=1 (no matches) |
| 3 | `grep -rE 'fusermount\|fuse-mount-tests' .github/ CLAUDE.md scripts/tag-v0.9.0.sh scripts/green-gauntlet.sh` empty | passed | grep rc=1 |
| 4 | `.claude/skills/reposix-agent-flow/SKILL.md` exists with `name:` + `description:` frontmatter | passed | 5523 bytes, frontmatter validated |
| 5 | `scripts/tag-v0.9.0.sh` exists, executable | passed | mode 755, 4449 bytes, 8 numbered guards |
| 6 | CHANGELOG.md has `## [v0.9.0] — 2026-04-24` section | passed | line 9 |
| 7 | CLAUDE.md no longer carries the "v0.9.0 — in progress" banner | passed | `grep "Architecture transition.*in progress"` rc=1 |
| 8 | CLAUDE.md has steady-state `## Architecture (git-native partial clone)` section | passed | line 9 |
| 9 | `cargo build --workspace` clean | passed | `Finished dev profile … in 8.37s` |
| 10 | `cargo clippy --workspace --all-targets -- -D warnings` clean | passed | `Finished dev profile … in 8.49s` |
| 11 | `cargo test --workspace` all green | passed | 49 `test result: ok` lines, zero `FAILED` |
| 12 | `git grep -i 'fuser\|fusermount\|fuse-mount-tests\|reposix mount' CLAUDE.md` empty | passed | grep rc=1 |
| 13 | Workspace version bumped to 0.9.0 | passed | `Cargo.toml`: `version = "0.9.0"` |
| 14 | Three new CI jobs (`integration-contract-{confluence,github,jira}-v09`) defined | passed | `.github/workflows/ci.yml` |
| 15 | `dark-factory` CI job replaces deleted `integration (mounted FS)` job | passed | `.github/workflows/ci.yml` |

## DELETE list (per 36-CONTEXT) — all confirmed

- `crates/reposix-fuse/` — 12 files removed (commit `1535cb0`).
- `fuser` workspace dependency — purged from `Cargo.toml`.
- `fuse-mount-tests` feature gate — no longer present anywhere in
  workspace `Cargo.toml` files or CI workflows.
- CI: `apt install fuse3`, `fusermount3`, `--features fuse-mount-tests`
  invocations all removed from `.github/workflows/ci.yml`.
- `crates/reposix-cli/src/mount.rs` — deleted; `mod mount;` removed
  from `main.rs`.
- `crates/reposix-cli/src/demo.rs` — deleted; `mod demo;` and
  `Cmd::Demo` variant removed.
- `Cmd::Mount` clap variant + `mount_emits_migration_error` test —
  removed (replaced by `mount_subcommand_is_removed` which asserts
  unrecognized-subcommand error).
- README "FUSE" prose in the v0.9.0 banner + hero alt text + release
  table v0.9 row updated. Historical sections (v0.7.x quickstart,
  Tier 1–5 demo descriptions, v0.4 release table rows) preserved per
  Phase 30 deferral to v0.10.0.

## UPDATE list — all confirmed

- CLAUDE.md fully rewritten (commit `52ce149`):
  - Banner replaced with steady-state `## Architecture (git-native
    partial clone)` section.
  - Elevator pitch: git-native partial clone narrative.
  - Workspace layout: `reposix-cache/` added; `reposix-fuse/` removed.
  - Tech stack: `fuser` row replaced with `gix 0.82` + `git >= 2.34`.
  - Commands: `reposix init`, `git clone`, dark-factory invocation,
    three real-backend test invocations.
  - Threat model: helper + cache are now the egress surfaces.
  - Quick links: `docs/reference/testing-targets.md` cross-link added.
  - Pre-v0.9.0 callouts removed throughout.
- Cargo.toml: workspace version 0.8.0 → 0.9.0; description updated.
- README.md: banner moved from "in progress" → "shipped"; hero alt-text
  updated; v0.9 release-table row finalized.
- CI workflows: `apt install fuse3` / `/dev/fuse` / `fuse-mount-tests`
  references all removed; `dark-factory` job replaces FUSE integration
  jobs; three `-v09` real-backend gates added.

## CREATE list — all confirmed

- `.claude/skills/reposix-agent-flow/SKILL.md` — created with valid
  Anthropic frontmatter (`name:`, `description:`, `argument-hint:`,
  `allowed-tools:`). Body references architecture-pivot-summary §4
  explicitly and links `scripts/dark-factory-test.sh` plus the cargo
  test variants. Description mentions "dark-factory regression"
  verbatim per 36-CONTEXT.
- `scripts/tag-v0.9.0.sh` — created; chmod +x; 8 numbered safety guards
  (the seven from v0.8.0 + an ARCH-18 guard for
  `docs/reference/testing-targets.md` existence).
- CHANGELOG `[v0.9.0] — 2026-04-24` finalized — Breaking / Added /
  Changed / Removed / Threat-model sections covering Phases 31–36.
- `.gitignore` rewrite to allow `.claude/skills/` while keeping session
  state ignored — required because the skill is committed grounding
  per OP-4 (self-improving infrastructure ships in lockstep with code).

## Real-backend CI status (carried from Phase 35)

| Job | Status | Reason |
|-----|--------|--------|
| `integration-contract-confluence-v09` | pending-secrets | secret pack not loaded in dev env |
| `integration-contract-github-v09` | pending-secrets | secret pack not loaded in dev env |
| `integration-contract-jira-v09` | pending-secrets | secret pack not loaded in dev env |

Per phase prompt: "If a real-backend live test cannot run because creds
are absent in the dev env: tag with `#[ignore]` + `skip_if_no_env!`,
mark the relevant ROADMAP success-criterion line with 'pending-secrets'
rather than 'passed', document it." Phase 36 wires the CI infra; the
secret packs are owner-driven gates.

## Threat-model delta

Deletion of `crates/reposix-fuse/` shrinks the lethal trifecta. Pre-
Phase 36, two egress surfaces existed: the FUSE daemon (a privileged
process holding `/dev/fuse`, making outbound HTTP) and the remote
helper. Post-Phase 36, only one remains: the remote helper +
`reposix-cache` materializer, both governed by the same
`REPOSIX_ALLOWED_ORIGINS` allowlist via the single
`reposix_core::http::client()` factory. The frontmatter field
allowlist (stripping `id`/`created_at`/`version`/`updated_at` on push)
is enforced on the `export` path; the `Tainted<T>` boundary is the
explicit sanitization point.

## Approval

`status: passed` — Phase 36 closes the v0.9.0 architecture pivot. Ready
for `/gsd-audit-milestone` + `/gsd-complete-milestone` invocations.
The user-driven gate is `bash scripts/tag-v0.9.0.sh` (after verifying
real-backend CI jobs run green when secrets are populated, or
accepting `pending-secrets` for tag-and-push).
