# CLAUDE.md — reposix project guide

Read by every agent (Claude Code, Codex, Cursor, …) that opens this repo. Local
extension of the user's global `~/.claude/CLAUDE.md`; overrides nothing, adds
project-specific rules. **This file holds only what every agent needs before its
first tool call.** Detail lives in scoped `CLAUDE.md`s (auto-load by directory:
`crates/`, `.planning/`, `quality/`) and the linked long-form homes. Working on X?
→ read Y (see § Pointer map).

## Project elevator pitch

reposix exposes REST-based issue trackers (and similar SaaS) as a git-native partial
clone, served by `git-remote-reposix` from a local bare-repo cache built from REST
responses. Agents use `cat`, `grep`, `sed`, `git` on real workflows — no MCP tool
schemas, no custom CLI, no FUSE mount. Architectural argument:
`docs/research/initial-report.md`. Dark-factory / simulator-first motivation:
`docs/research/agentic-engineering-reference.md`.

## Architecture (git-native partial clone)

> **Source of truth:** `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary/index.md`
> + `.planning/research/v0.13.0-dvcs/architecture-sketch/index.md` (DVCS extensions,
> ratified P78–P88). Mental model + mirror-lag refs: `docs/concepts/dvcs-topology.md`.

Three runtime pieces:

- **`reposix-cache`** — real on-disk bare git repo built from REST via the
  `BackendConnector` trait. Materializes blobs lazily; every materialization writes
  `audit_events_cache`; bytes return wrapped in `reposix_core::Tainted<Vec<u8>>`.
- **`git-remote-reposix`** — git remote helper. Advertises `stateless-connect` (read:
  tunnels protocol-v2 fetch with `--filter=blob:none`) and `export` (push: fast-import
  → push-time conflict detection → REST writes). Refspec `refs/heads/*:refs/reposix/*`.
- **`reposix init <backend>::<project> <path>`** bootstraps a partial-clone tree;
  **`reposix attach <backend>::<project>`** adopts an existing checkout (vanilla clone,
  hand-edited tree, or prior `init`) and binds it to a `SoT` backend (all four backends
  wired P91). Reconciles records by frontmatter `id`; idempotent on same-SoT re-attach,
  rejects a different SoT. Record paths are bucket-aware: `issues/<id>.md` for
  sim/GitHub/JIRA, `pages/<id>.md` for Confluence — `reposix_core::path::{record_path,
  bucket_for_backend}` is the single source of truth; never hand-pick a bucket string.

After bootstrap, agent UX is pure git: `git checkout origin/main && cat issues/<id>.md
&& grep -r TODO . && <edit> && git add . && git commit && git push`. Zero reposix CLI
awareness beyond `init` / `attach`.

**Load-bearing agent-facing recovery moves** (full DVCS detail:
`docs/concepts/dvcs-topology.md`; recovery playbook: `docs/guides/troubleshooting.md`
§ "DVCS push/pull issues"):

- **Push-time conflict detection.** Helper rejects with the standard git "fetch first"
  error on remote drift; recover via `git pull --rebase && git push`.
- **Blob limit.** Helper refuses a `command=fetch` that would materialize more than
  `REPOSIX_BLOB_LIMIT` blobs (default 200); the stderr error names `git sparse-checkout`
  as the recovery move.
- **Bus URL / mirror fan-out / webhook sync** (`reposix::<sot>?mirror=<url>`, SoT-first
  then mirror-best-effort, mirror-lag refs, webhook-driven sync, p95 ≤ 120s): compressed
  out of root — see `docs/concepts/dvcs-topology.md` + `docs/guides/dvcs-mirror-setup.md`.

**Runtime prerequisite: `git >= 2.34`** (for `extensions.partialClone` +
`stateless-connect`). Full tech stack: `crates/CLAUDE.md`.

## Orchestration doctrine (how autonomous sessions run)

Full doctrine — delegation, coordinator discipline, relief, cadence, durable state —
lives in **`.planning/ORCHESTRATION.md`** (read before dispatching any agent).

- Top-level delegates ONLY to **fable** coordinators, which tier down to **opus**
  (complex/security), **sonnet** (default), **haiku** (mechanical). Never fable at a leaf.
- Coordinators **route, don't work**; relieve past **~50% context** at a wave boundary
  (write+commit a handover first).
- **Uncommitted = didn't happen** — enforced by `.claude/hooks/` (cargo mutex,
  stop-on-dirty, precompact-persist). External mutations need owner-named-target approval.
- Understand **intention over faithful plan execution**.
- Subagent-dispatch specifics (gh-pr-checkout isolation, top-level orchestration-shaped
  phases, milestone 9th probe, subjective-rubric dispatch): `.planning/CLAUDE.md`.

## Operating Principles (project-specific)

The user's global Operating Principles are bible; these are reinforcements, not replacements.

1. **Simulator is the default / testing backend.** The sim at `crates/reposix-sim/` is
   the default backend for every demo, unit test, and autonomous loop. Real backends
   (GitHub, Confluence, JIRA) are guarded by the `REPOSIX_ALLOWED_ORIGINS` egress
   allowlist and require explicit credential env vars (`GITHUB_TOKEN`;
   `ATLASSIAN_API_KEY` + `ATLASSIAN_EMAIL` + `REPOSIX_CONFLUENCE_TENANT`; `JIRA_EMAIL` +
   `JIRA_API_TOKEN` + `REPOSIX_JIRA_INSTANCE`). Autonomous mode never hits a real backend
   unless the user put real creds in `.env` AND set a non-default allowlist. Security
   constraint (fail-closed) + the StrongDM dark-factory pattern.
2. **Tainted by default.** Any byte from a remote (simulator counts) is tainted and must
   not be routed into side-effecting actions on other systems (e.g. don't echo issue
   bodies into `git push` to remotes outside an explicit allowlist). The trifecta
   mitigation matters even against the sim, because the sim is *seeded* by an agent and
   seed data is itself attacker-influenced.
3. **Audit log is non-optional, in TWO append-only tables.** `audit_events_cache`
   (cache-internal: blob materialization, gc, helper RPC fetch/push, sync/mirror-ref
   writes) in `reposix-cache::audit`; `audit_events` (backend mutations: create/update/
   delete_or_close) in `reposix-core::audit`, written by the sim/confluence/jira adapters.
   A complete forensic query reads both. A network-touching action missing a row in
   either means the feature isn't done. Dual-table shape is intentional; `dyn AuditSink`
   unification deferred.
4. **No hidden state.** Cache, simulator, and helper state all live in committed-or-
   fixture artifacts. No "works in my session" bugs.
5. **Working tree IS a real git checkout.** `.git/` is real, not synthetic; `git diff`
   is the change set by construction. Partial clone makes blobs lazy; everything else is
   upstream git.
6. **Real backends are first-class test targets.** Three sanctioned targets: **Confluence
   space "TokenWorld"** (owned by user; mutate freely), **GitHub repo `reubenjohn/reposix`
   issues** (create/close during tests), **JIRA project `TEST`** (override via
   `JIRA_TEST_PROJECT` / `REPOSIX_JIRA_PROJECT`). Setup + cleanup:
   `docs/reference/testing-targets.md`. Sim stays the default (OP-1), but "simulator-only
   coverage" does NOT satisfy transport-layer or performance claims.
7. **Phase-close means catalog-row PASS.** No phase ships on the executing agent's word;
   an unbiased verifier subagent grades the catalog rows — RED loops back (see § Quality
   Gates + `quality/CLAUDE.md`).
8. **Plans accommodate surprises (+2 phase practice).** Each milestone reserves its last
   two phases as absorption slots (Slot 1 drains `SURPRISES-INTAKE.md`, Slot 2 drains
   `GOOD-TO-HAVES.md`). Eager-fix if < 1h + no new dependency, else file — never silently
   skip, never scope-creep. Long-form: `.planning/PRACTICES.md` § OP-8.
9. **Milestone-close: distill before archiving.** Intakes + run findings distill into a
   new `.planning/RETROSPECTIVE.md` section BEFORE archive; the ratification subagent
   grades RED if it's missing. Long-form: `.planning/PRACTICES.md` § OP-9.

## GSD workflow

This project uses GSD for planning and execution.

> **Always enter through a GSD command.** Never edit code or planning artifacts outside
> a GSD-tracked phase or quick.

Entry points: `/gsd-quick` (small fix/doc), `/gsd-execute-phase <n>` (planned phase),
`/gsd-debug` (bug), `/gsd-progress` (state). Mode config (yolo / coarse / all gates on)
and push cadence: `.planning/CLAUDE.md`. Do not silently downgrade the gates.

**Push cadence.** Every phase closes with `git push origin main` BEFORE the verifier
subagent; the verifier grades RED if the phase shipped without the push landing.
Milestone-close additionally requires `python3 quality/runners/run.py --cadence
pre-release-real-backend` exit 0 (non-skippable 9th probe — `.planning/CLAUDE.md`).

## Commands you'll actually use

Full matrix in `CONTRIBUTING.md`; per-crate build discipline in `crates/CLAUDE.md`.

```bash
bash scripts/install-hooks.sh                        # one-time after fresh clone
cargo check -p <crate> && cargo nextest run          # prefer per-crate (build memory budget)
cargo run -p reposix-sim                              # start simulator on :7878
cargo run -p reposix-cli -- init sim::demo /tmp/repo  # bootstrap a partial-clone tree
cd /tmp/repo && git checkout origin/main             # agent UX from here is pure git
bash quality/gates/agent-ux/dark-factory.sh sim              # v0.9 dark-factory regression
bash quality/gates/agent-ux/dark-factory.sh dvcs-third-arm   # v0.13 attach+bus arm
bash scripts/preflight-real-backends.sh              # 0=reachable 1=auth/net gap 2=no creds
```

Real-backend export blocks + `dark_factory_real_{confluence,github,jira}`:
`docs/reference/testing-targets.md`.

## Build memory budget (load-bearing — one-liner; full doctrine in `crates/CLAUDE.md`)

**Never run more than one cargo invocation (check/build/test/clippy) at a time,
machine-wide** — the VM has OOM-crashed three times from parallel cargo workspace
builds. Enforced by `.claude/hooks/cargo-mutex.sh` (backstop; orchestration discipline
is the primary control). Prefer per-crate over `--workspace`. Rationale, soft rules,
crash triage: `crates/CLAUDE.md`.

## Cold-reader pass on user-facing surfaces

Before declaring any user-facing surface shipped (hero copy, install instructions,
headline numbers, README, docs landing), dispatch `/doc-clarity-review` on the affected
pages. Automated rubric grading: `/reposix-quality-review` (`--rubric <id>` /
`--all-stale`); catalog `quality/catalogs/subjective-rubrics.json` enforces a 30-day TTL.

## Docs-site + freshness gates (pre-push enforced)

- Any change to `mkdocs.yml` or `docs/**` MUST pass
  `bash quality/gates/docs-build/mkdocs-strict.sh` + `.../mermaid-renders.sh`. Playwright
  walk scoping: `/reposix-quality-doc-alignment` skill.
- Freshness invariants (no version-pinned filenames, install path leads with package
  manager, benchmarks in mkdocs nav, no loose ROADMAP/REQUIREMENTS outside `*-phases/`,
  no orphan docs) enforced by `quality/runners/verdict.py`. A blocked push names the
  violated invariant + fix.

## Release pipeline

`release-plz.toml` sets `git_release_enable = false` at workspace level so release-plz
does NOT create per-package GitHub releases (each zero-asset release used to steal
`releases/latest`, 404'ing installer URLs). Per-package tags + crates.io publishes are
unaffected. Canonical multi-platform release: `.github/workflows/release.yml` (tag `v*`).

## Ownership charter for dispatched subagents

Every subagent (executor, verifier, researcher, code-reviewer) that touches a real surface owns it, not just its acceptance criteria: (1) acceptance criteria are the floor, not the ceiling — done means "I'd defend this in review as excellent," not "plan executed"; (2) noticing is a deliverable — every report names what it noticed near its work (lying doc claims, tests that don't assert what their names promise, error messages that don't teach recovery, dead code, stale comments, missing edge cases), and an empty noticing section from code-touching work is itself a red flag (mirrors the verifier honesty check); (3) eager-fix or file, never silently skip — `<1h` + no new dependency → fix in place, else → `SURPRISES-INTAKE`/`GOOD-TO-HAVES` with severity + sketch (OP-8); (4) verify against reality — run the thing, render the page, hit the backend; a claim without an artifact is not done (OP-1); (5) north star — polish for adoption: would a skeptical dev hitting this surface for the first time come away impressed? (Owner mandate OD-3, 2026-07-03.)

The orchestrator's job is to route, decide, and integrate — not to type code that a subagent could type.

### Meta-rule: when an owner catches a quality miss, fix it twice

Fix the issue in code/docs, AND update the relevant CLAUDE.md (root or scoped) /
ORCHESTRATION.md so the next agent's session reads the tightened rule — AND tag the
dimension (routes to the right catalog + `quality/gates/<dim>/`). Shipping the fix
without updating the instructions guarantees recurrence. Each phase introducing a new
file/convention/gate revises the relevant CLAUDE.md in the same PR (revise the doc to
reflect the new state — not an appended narrative).

## Threat model

Textbook lethal-trifecta machine: private data (issue bodies in the working tree) +
untrusted input (every body/comment/title is attacker-influenced) + exfiltration paths
(`git push` to arbitrary remotes; helper + cache make outbound HTTP). The mandatory,
tested cuts: `REPOSIX_ALLOWED_ORIGINS` egress allowlist through the single
`reposix_core::http::client()` factory (clippy `disallowed_methods` bans direct
`reqwest::Client::new()`); bytes-in-bytes-out export path with `Tainted<T>` →
`sanitize()` as the only escape hatch; frontmatter field allowlist stripping
server-controlled fields on inbound writes; append-only dual audit tables (OP-3); and
the env-gated `pre-release-real-backend` cadence that makes allowlist enforcement
testable end-to-end (fails closed). Full per-cut table with code locations:
`docs/how-it-works/trust-model.md`. Red-team analysis: `research/threat-model-and-critique.md`.

## Quality Gates

Framework at `quality/`. Runtime contract: `quality/PROTOCOL.md` (read first for any
quality-gates task). Routing summary, catalog-first rule, verifier dispatch, the 9
dimensions / 8 cadences / 6 kinds taxonomy, honesty rules, structure-dimension gates,
and the docs-alignment dimension: **`quality/CLAUDE.md`** + `quality/catalogs/README.md`.
Adding a gate = one catalog row + one verifier in `quality/gates/<dim>/`; the runner
discovers by tag (no new top-level script). Catalog-first: a phase's FIRST commit writes
the rows defining its GREEN contract; the verifier reads rows that existed BEFORE the
implementation landed.

## What to do when context fills

Read `.planning/STATE.md` first (the entry point), then the most recent
`.planning/phases/*/PLAN.md`, then `git log --oneline -20`. Don't read this file
linearly; grep for the section you need.

## Pointer map — working on X? read Y

| Working on… | Read |
|---|---|
| Rust crates / build / code conventions | `crates/CLAUDE.md` |
| Planning, GSD, orchestration dispatch | `.planning/CLAUDE.md`, `.planning/ORCHESTRATION.md` |
| Quality gates / catalogs / verifiers | `quality/CLAUDE.md`, `quality/PROTOCOL.md`, `quality/catalogs/README.md` |
| OP-8/OP-9 long-form (surprises, retro) | `.planning/PRACTICES.md` |
| Threat model / security cuts | `docs/how-it-works/trust-model.md`, `research/threat-model-and-critique.md` |
| DVCS topology / mirror-lag refs | `docs/concepts/dvcs-topology.md` |
| Mirror + webhook owner setup | `docs/guides/dvcs-mirror-setup.md` |
| DVCS push/pull troubleshooting | `docs/guides/troubleshooting.md` |
| Real-backend test targets | `docs/reference/testing-targets.md` |
| Latency envelope | `docs/benchmarks/latency.md` |
| Architectural argument | `docs/research/initial-report.md` |
| Dark-factory / trifecta / sim-first | `docs/research/agentic-engineering-reference.md` |
| Current scope / cursor | `.planning/PROJECT.md`, `.planning/STATE.md` |
